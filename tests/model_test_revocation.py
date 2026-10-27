#!/usr/bin/env python3
"""Real model-test credential/role leases across worker queueing and invocation.

The receiver records actual upstream requests and blocks their receipts behind
explicit barriers. Model-test reservations prove admission to a full worker
queue; no guessed delay is used to infer that a model request has been sent.
"""
import argparse
import datetime
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import tempfile
import threading
import time
import traceback

from delivery_session_revocation import Bearer, expire, login
from integration import Client, Database, Instance, PASSWORD, ROOT, wait_for


class Receiver(http.server.BaseHTTPRequestHandler):
    observed = []
    gates = {}
    condition = threading.Condition()

    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers["Content-Length"])))
        label = body["model"]
        with self.condition:
            self.observed.append(label)
            gate = self.gates.get(label)
            self.condition.notify_all()
        if gate is not None:
            gate.wait(timeout=15)
        payload = json.dumps({
            "choices": [{"message": {"role": "assistant", "content": "OK"},
                         "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 1},
        }).encode()
        try:
            self.send_response(200)
            self.send_header("Content-Type", "application/json")
            self.send_header("Content-Length", str(len(payload)))
            self.end_headers()
            self.wfile.write(payload)
        except (BrokenPipeError, ConnectionResetError):
            pass


def observed(label, count=1):
    with Receiver.condition:
        assert Receiver.condition.wait_for(
            lambda: Receiver.observed.count(label) >= count, timeout=10
        ), (label, count)


def test_thread(client, path):
    outcome = {}

    def run():
        try:
            outcome["response"] = client.post(path, expect=(200, 401, 403, 409, 502))
        except Exception as error:
            outcome["error"] = str(error)

    thread = threading.Thread(target=run, daemon=True)
    thread.start()
    return thread, outcome


def joined(job, success):
    thread, outcome = job
    thread.join(timeout=6)
    assert not thread.is_alive(), "model test did not settle after authority was revoked"
    assert "error" not in outcome, outcome
    result = outcome["response"]
    if success:
        assert result.get("success") is True, result
    else:
        assert "error" in result and not result.get("success"), result
    return result


def exercise(options, report):
    databases = Database(options.database_url)
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Receiver)
    receiver.daemon_threads = True
    receiver_thread = threading.Thread(target=receiver.serve_forever, daemon=True)
    receiver_thread.start()
    instance = None
    jobs = []
    try:
        with tempfile.TemporaryDirectory(prefix="rstrbot-model-test-lease-") as temporary:
            database_url = databases.fresh()
            instance = Instance(Path(temporary) / "core", options.binary, database_url,
                                allow_private_network=True)
            instance.start()
            setup = Client(instance.base).post("/api/v1/setup", {
                "token": (instance.directory / "bootstrap.token").read_text(),
                "username": "owner", "password": PASSWORD, "workspace": "Model test leases",
            }, browser=False, extra={"X-RstrBot-Request": "1"})
            owner = Bearer(instance.base, setup["token"])
            workspace = owner.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"

            def new_model(label):
                return owner.post(base + "/resources/models", {"visibility": "workspace", "data": {
                    "name": label, "provider": "openai", "model": label,
                    "base_url": f"http://127.0.0.1:{receiver.server_port}/v1",
                    "allow_private_network": True, "max_tokens": 32,
                    "input_microusd_per_million": 1000000,
                    "output_microusd_per_million": 1000000,
                }})

            def new_admin(username):
                user = owner.post("/api/v1/admin/users", {"username": username, "password": PASSWORD})["user"]
                owner.post(base + "/members", {"user_id": user["id"], "role": "admin"})
                return user, login(instance.base, username)

            def test_path(model):
                return base + f"/models/{model['id']}/test"

            def reservations():
                return owner.get(base + "/reservations")["items"]

            # Keep all eight actual worker slots busy. The ninth model-test
            # reservation proves its request passed HTTP admission while it
            # cannot yet have started a provider worker.
            keeper = new_model("keeper")
            queued_model = new_model("revoked-in-queue")
            _, victim = new_admin("queued-tester")
            Receiver.gates["keeper"] = threading.Event()
            before = len(reservations())
            keepers = [test_thread(owner, test_path(keeper)) for _ in range(8)]
            jobs.extend(keepers)
            observed("keeper", 8)
            queued = test_thread(victim, test_path(queued_model))
            jobs.append(queued)
            wait_for(reservations, lambda items: len(items) == before + 9)
            assert "revoked-in-queue" not in Receiver.observed
            victim.post("/api/v1/auth/logout", expect=(204,))
            victim.get("/api/v1/auth/me", expect=(401,))
            Receiver.gates["keeper"].set()
            joined(queued, success=False)
            for keeper_job in keepers:
                joined(keeper_job, success=True)
            assert "revoked-in-queue" not in Receiver.observed, Receiver.observed
            report["checks"].append(
                "a logged-out queued model test never reaches the provider; eight independent owner tests complete"
            )

            for mode in ("logout", "password", "expiry", "demotion"):
                username = "tester-" + mode
                user, victim = new_admin(username)
                label = "inflight-" + mode
                model = new_model(label)
                Receiver.gates[label] = threading.Event()
                known = {item["id"] for item in reservations()}
                job = test_thread(victim, test_path(model))
                jobs.append(job)
                observed(label)
                if mode == "logout":
                    victim.post("/api/v1/auth/logout", expect=(204,))
                elif mode == "password":
                    victim.post("/api/v1/auth/password", {
                        "current_password": PASSWORD,
                        "new_password": "Replacement model-test password 2026!",
                    }, expect=(204,))
                elif mode == "expiry":
                    expire(instance, database_url, victim.token)
                else:
                    owner.post(base + "/members", {"user_id": user["id"], "role": "member"})
                # The provider's successful response is still withheld. A
                # revoked test must finish without waiting for that response.
                joined(job, success=False)
                current = [item for item in reservations() if item["id"] not in known]
                assert len(current) == 1 and current[0]["state"] == "unknown", current
                assert current[0]["actual_microusd"] is None, current
                assert Receiver.observed.count(label) == 1
                Receiver.gates[label].set()
                report["checks"].append(
                    f"in-flight {mode} revocation terminates the model test; an already sent request retains unknown cost and is not retried"
                )

            # Fresh authority remains usable after one session/role was revoked.
            healthy = new_model("healthy-after-revocation")
            result = owner.post(test_path(healthy))
            assert result["success"] is True and result["cost_microusd"] == 11, result
            report["checks"].append("an independent valid administrator still receives and settles a model-test result")
    finally:
        for gate in Receiver.gates.values():
            gate.set()
        for thread, _ in jobs:
            thread.join(timeout=2)
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
    report = {
        "status": "running", "checks": [],
        "database": "postgresql" if options.database_url else "sqlite",
        "platform": platform.platform(),
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": hashlib.sha256(options.binary.read_bytes()).hexdigest(),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "scope": "real local HTTP and provider workers with a barrier-controlled mock upstream; no commercial account",
    }
    try:
        exercise(options, report)
        report["status"] = "passed"
    except Exception:
        report["status"] = "failed"
        report["error"] = traceback.format_exc()
    report["upstream_requests"] = list(Receiver.observed)
    report["exit_code"] = int(report["status"] != "passed")
    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return report["exit_code"]


if __name__ == "__main__":
    raise SystemExit(main())
