#!/usr/bin/env python3
"""Credential leases on real in-flight HTTP runs, using the diagnostic worker."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import tempfile
import time
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, TERMINAL, wait_for


def exercise(options, report):
    database = Database(options.database_url)
    server = None
    with tempfile.TemporaryDirectory(prefix="rstrbot-session-") as directory:
        try:
            server = Instance(Path(directory) / "core", options.binary, database.fresh())
            server.start()
            first, second, observer = (Client(server.base) for _ in range(3))
            first.post("/api/v1/setup", {
                "token": (server.directory / "bootstrap.token").read_text(),
                "username": "alice", "password": PASSWORD, "workspace": "Credential lease fixture",
            })
            second.login("alice")
            observer.login("alice")
            wid = first.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{wid}"
            model = first.post(base + "/resources/models", {
                "visibility": "workspace", "data": {
                    "name": "Diagnostic (not AI)", "provider": "diagnostic", "model": "diagnostic",
                },
            })

            def start(client, title):
                conversation = client.post(base + "/resources/conversations", {
                    "visibility": "private", "data": {"title": title},
                })
                admitted = client.post(base + "/chat", {
                    "conversation_id": conversation["id"], "model_id": model["id"],
                    "content": "/diagnostic/wait", "idempotency_key": str(uuid.uuid4()),
                })
                wait_for(lambda: observer.get(base + "/runs/" + admitted["run"]["id"]),
                         lambda item: item["data"]["status"] == "running")
                return admitted["run"]

            one = start(first, "First credential")
            two = start(second, "Independent second credential")
            first.post("/api/v1/auth/logout", expect=(204,))
            first.get("/api/v1/auth/me", expect=(401,))
            stopped = wait_for(lambda: observer.get(base + "/runs/" + one["id"]),
                               lambda item: item["data"]["status"] in TERMINAL, timeout=6)
            assert stopped["data"]["status"] == "cancelled", stopped
            assert observer.get(base + "/runs/" + two["id"])["data"]["status"] == "running"
            report["checks"].append("logout cancels only runs leased to that credential; another session of the same user remains active")
            observer.post(base + "/runs/" + two["id"] + "/cancel", expect=(204,))
            wait_for(lambda: observer.get(base + "/runs/" + two["id"]),
                     lambda item: item["data"]["status"] in TERMINAL, timeout=6)

            replacement = "Replacement fixture password 2026!"
            one = start(second, "Password change first session")
            two = start(observer, "Password change second session")
            second.post("/api/v1/auth/password", {"current_password": PASSWORD, "new_password": replacement}, expect=(204,))
            observer.get("/api/v1/auth/me", expect=(401,))
            second.get("/api/v1/auth/me", expect=(401,))
            fresh = Client(server.base)
            fresh.post("/api/v1/auth/login", {"username": "alice", "password": replacement})
            for run in (one, two):
                stopped = wait_for(lambda: fresh.get(base + "/runs/" + run["id"]),
                                   lambda item: item["data"]["status"] in TERMINAL, timeout=6)
                assert stopped["data"]["status"] == "cancelled", stopped
                conversation = fresh.get(base + "/resource/" + stopped["data"]["conversation_id"])
                assert not any(message.get("role") == "assistant" for message in conversation["data"]["messages"])
            report["checks"].append("password change revokes all user sessions and stops their in-flight runs without publishing late assistant output")

            # Schedules are explicit owner delegation, independent from the
            # browser session that happened to create or trigger the schedule.
            background_conversation = fresh.post(base + "/resources/conversations", {
                "visibility": "private", "data": {"title": "Explicit background delegation"},
            })
            schedule = fresh.post(base + "/resources/schedules", {
                "visibility": "private", "data": {
                    "title": "Explicit background delegation", "schedule_type": "once",
                    "at": int(time.time()) + 3600, "enabled": False,
                    "prompt": "/diagnostic/wait", "model_id": model["id"],
                    "conversation_id": background_conversation["id"],
                },
            })
            triggered = fresh.post(base + "/schedules/" + schedule["id"] + "/run")
            # trigger returns the run admission, like a scheduled dispatch.
            background = triggered["run"]
            fresh.post("/api/v1/auth/logout", expect=(204,))
            relogged = Client(server.base)
            relogged.post("/api/v1/auth/login", {"username": "alice", "password": replacement})
            time.sleep(0.8)
            assert relogged.get(base + "/runs/" + background["id"])["data"]["status"] == "running"
            relogged.post(base + "/runs/" + background["id"] + "/cancel", expect=(204,))
            wait_for(lambda: relogged.get(base + "/runs/" + background["id"]),
                     lambda item: item["data"]["status"] in TERMINAL, timeout=6)
            report["checks"].append("explicit schedule delegation survives browser logout and remains cancellable under current owner permissions")
        finally:
            if server:
                server.stop()
            database.cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--output", type=Path, required=True)
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    report = {
        "status": "running", "checks": [], "database": "postgresql" if options.database_url else "sqlite",
        "platform": platform.platform(), "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": hashlib.sha256(options.binary.read_bytes()).hexdigest(),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "scope": "local real HTTP, isolated database and diagnostic process; no commercial model account",
    }
    try:
        exercise(options, report)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = str(error)
    options.output.parent.mkdir(parents=True, exist_ok=True)
    options.output.write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    return int(report["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
