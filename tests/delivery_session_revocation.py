#!/usr/bin/env python3
"""Real HTTP authorization leases across the plugin queue and external sends.

Eight receiver-observed blocked sends fill the actual host's eight worker slots.
A ninth send is revoked before those slots are released. Barriers establish the
ordering; no guessed sleep is used to infer that a request is queued or sent.
"""
import argparse
import datetime
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import shutil
import sqlite3
import subprocess
import tempfile
import threading
import time
import traceback
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, wait_for


class Bearer:
    def __init__(self, base, token):
        self.base = base
        self.token = token

    def request(self, method, path, data=None, expect=(200,)):
        return Client(self.base).request(method, path, data, expect=expect,
                                        browser=False, extra={"Authorization": "Bearer " + self.token})

    def get(self, path, **kwargs):
        return self.request("GET", path, **kwargs)

    def post(self, path, data=None, **kwargs):
        return self.request("POST", path, data, **kwargs)


def login(base, username, password=PASSWORD):
    result = Client(base).post("/api/v1/auth/login", {"username": username, "password": password},
                               browser=False, extra={"X-RstrBot-Request": "1"})
    return Bearer(base, result["token"])


class Receiver(http.server.BaseHTTPRequestHandler):
    observed = []
    gates = {}
    condition = threading.Condition()

    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        label = body["text"]
        with self.condition:
            self.observed.append(label)
            gate = self.gates.get(label)
            self.condition.notify_all()
        if gate is not None:
            gate.wait(timeout=15)
        response = json.dumps({"id": label}).encode()
        self.send_response(201)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(response)))
        self.end_headers()
        try:
            self.wfile.write(response)
        except (BrokenPipeError, ConnectionResetError):
            pass


def observed(labels):
    with Receiver.condition:
        assert Receiver.condition.wait_for(lambda: all(label in Receiver.observed for label in labels), timeout=10), labels


def send_thread(client, path, body):
    outcome = {}
    def run():
        try:
            outcome["receipt"] = client.post(path, body)
        except Exception as error:
            outcome["error"] = str(error)
    thread = threading.Thread(target=run, daemon=True)
    thread.start()
    return thread, outcome


def joined(job):
    thread, outcome = job
    thread.join(timeout=10)
    assert not thread.is_alive(), "send did not finish after its barrier was released"
    assert "error" not in outcome, outcome
    return outcome["receipt"]


def expire(instance, database_url, token):
    digest = hashlib.sha256(token.encode()).hexdigest()
    if database_url:
        psql = os.environ.get("RSTRBOT_PSQL") or shutil.which("psql")
        assert psql, "PostgreSQL test expiry requires psql"
        # The only interpolated value is a local SHA-256 hexadecimal digest.
        subprocess.run([psql, database_url, "-X", "-v", "ON_ERROR_STOP=1", "-qc",
                        f"UPDATE sessions SET expires_at=0 WHERE token_hash='{digest}'"],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    else:
        with sqlite3.connect(instance.directory / "rstrbot.db") as database:
            database.execute("UPDATE sessions SET expires_at=0 WHERE token_hash=?", (digest,))


def exercise(options, report):
    databases = Database(options.database_url)
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Receiver)
    receiver.daemon_threads = True
    receiver_thread = threading.Thread(target=receiver.serve_forever, daemon=True)
    receiver_thread.start()
    instance = None
    try:
        with tempfile.TemporaryDirectory(prefix="rstrbot-delivery-lease-") as temporary:
            database_url = databases.fresh()
            instance = Instance(Path(temporary) / "core", options.binary, database_url, allow_private_network=True)
            instance.start()
            setup = Client(instance.base).post("/api/v1/setup", {
                "token": (instance.directory / "bootstrap.token").read_text(),
                "username": "owner", "password": PASSWORD, "workspace": "Leases",
            }, browser=False, extra={"X-RstrBot-Request": "1"})
            owner = Bearer(instance.base, setup["token"])
            workspace = owner.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"
            route = owner.post(base + "/resources/routes", {"visibility": "workspace", "data": {
                "name": "Local barrier receiver", "url": f"http://127.0.0.1:{receiver.server_port}/send",
                "enabled": True, "allow_private_network": True,
            }})
            path = base + f"/routes/{route['id']}/send"

            for mode in ("logout", "password", "expiry"):
                username = "queued-" + mode
                user = owner.post("/api/v1/admin/users", {"username": username, "password": PASSWORD})["user"]
                owner.post(base + "/members", {"user_id": user["id"], "role": "member"})
                victim = login(instance.base, username)
                observer = login(instance.base, username)
                labels = [f"keeper-{mode}-{index}" for index in range(8)]
                for label in labels:
                    Receiver.gates[label] = threading.Event()
                keepers = [send_thread(owner, path, {"text": label, "idempotency_key": str(uuid.uuid4())}) for label in labels]
                observed(labels)
                label = "revoked-" + mode
                body = {"text": label, "idempotency_key": str(uuid.uuid4())}
                queued = send_thread(victim, path, body)
                pending = wait_for(lambda: next((item for item in observer.get(base + "/resources/deliveries")["items"]
                                                if item["data"].get("text") == label), None),
                                   lambda item: item is not None and item["data"]["status"] == "sending")
                assert label not in Receiver.observed
                replacement = PASSWORD
                if mode == "logout":
                    victim.post("/api/v1/auth/logout", expect=(204,))
                elif mode == "password":
                    replacement = "Replacement delivery password 2026!"
                    victim.post("/api/v1/auth/password", {"current_password": PASSWORD, "new_password": replacement}, expect=(204,))
                else:
                    expire(instance, database_url, victim.token)
                victim.get("/api/v1/auth/me", expect=(401,))
                # Release immediately after revocation. The after-slot gate,
                # not an arbitrary monitor interval, must prevent first dispatch.
                for keeper in labels:
                    Receiver.gates[keeper].set()
                receipt = joined(queued)
                assert receipt["id"] == pending["id"] and receipt["data"]["status"] == "unknown", receipt
                assert label not in Receiver.observed, Receiver.observed
                for keeper in keepers:
                    assert joined(keeper)["data"]["status"] == "accepted"
                fresh = login(instance.base, username, replacement)
                repeat = fresh.post(path, body)
                assert repeat["id"] == receipt["id"] and repeat["data"]["status"] == "unknown"
                assert label not in Receiver.observed
                report["checks"].append(f"queued {mode} revocation prevents first outbound request; independent credentials finish; retry preserves the original conservative receipt without dispatch")

            # The receiver has already observed this operation before logout.
            # Cancelling it cannot turn that observed effect into 'not_sent'.
            inflight = login(instance.base, "queued-logout")
            label = "observed-before-logout"
            Receiver.gates[label] = threading.Event()
            body = {"text": label, "idempotency_key": str(uuid.uuid4())}
            job = send_thread(inflight, path, body)
            observed([label])
            inflight.post("/api/v1/auth/logout", expect=(204,))
            receipt = joined(job)
            assert receipt["data"]["status"] == "unknown", receipt
            Receiver.gates[label].set()
            fresh = login(instance.base, "queued-logout")
            assert fresh.post(path, body)["id"] == receipt["id"]
            assert Receiver.observed.count(label) == 1
            report["checks"].append("an already observed external send remains unknown after logout and identical retries do not repeat it")
    finally:
        for gate in Receiver.gates.values():
            gate.set()
        if instance:
            instance.stop()
        receiver.shutdown()
        receiver.server_close()
        receiver_thread.join(timeout=3)
        databases.cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    report = {"status": "running", "checks": [], "database": "postgresql" if options.database_url else "sqlite",
              "platform": platform.platform(), "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "binary_sha256": {name: hashlib.sha256((options.binary.parent / name).read_bytes()).hexdigest()
                                for name in ("rstrbot", "rstrbot-webhook")},
              "scope": "real authenticated HTTP and actual eight-slot worker queue with local receiver barriers; synthetic credentials only"}
    started = time.monotonic()
    try:
        exercise(options, report)
        report["status"] = "passed"
    except Exception as error:
        report.update(status="failed", error=str(error), traceback=traceback.format_exc())
    report["duration_seconds"] = round(time.monotonic() - started, 3)
    report["passed_checks"] = len(report["checks"])
    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return int(report["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
