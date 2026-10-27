#!/usr/bin/env python3
"""Exercise schedule ownership through real HTTP, workers and database state.

A shared schedule is visible to its workspace, but its configuration remains an
execution grant from its owner. The same test uses SQLite or a disposable
PostgreSQL schema. Diagnostic responses are fixtures, not a real model service.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import time

from integration import (
    Client,
    Database,
    Instance,
    PASSWORD,
    ROOT,
    TERMINAL,
    fixture_directory,
    wait_for,
)


def exercise(options, report):
    database = Database(options.database_url)
    with fixture_directory(database, options.keep_on_failure, report) as directory:
        server = Instance(Path(directory) / "server", options.binary, database.fresh())
        try:
            server.start()
            alice, bob = Client(server.base), Client(server.base)
            alice.post("/api/v1/setup", {
                "token": (server.directory / "bootstrap.token").read_text(),
                "username": "alice", "password": PASSWORD, "workspace": "Schedules",
            })
            workspace = alice.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"
            bob_id = alice.post("/api/v1/admin/users", {
                "username": "bob", "password": PASSWORD,
            })["user"]["id"]
            bob.login("bob")

            def role(value):
                alice.post(base + "/members", {"user_id": bob_id, "role": value})

            def create(client, kind, data, visibility="private"):
                return client.post(base + "/resources/" + kind, {
                    "visibility": visibility, "data": data,
                })

            def read(client, item):
                return client.get(base + "/resource/" + item["id"])

            def update(client, item, config, **kwargs):
                return client.request("PATCH", base + "/resource/" + item["id"], {
                    "version": item["version"], "data": config,
                }, **kwargs)

            role("operator")
            model = create(alice, "models", {
                "name": "Owner private diagnostic", "provider": "diagnostic",
                "model": "diagnostic", "max_tokens": 256,
            })
            conversation = create(alice, "conversations", {"title": "Owner private conversation"})
            config = {
                "title": "Visible job, private execution identity",
                "prompt": "owner-approved-original",
                "conversation_id": conversation["id"], "model_id": model["id"],
                "schedule_type": "once", "at": int(time.time()) + 3600,
                "enabled": False,
            }
            schedule = create(alice, "schedules", config, "workspace")
            for member_role in ("operator", "admin", "owner"):
                role(member_role)
                assert read(bob, schedule)["owner_id"] == schedule["owner_id"]
                for proposed in (dict(config, prompt="injected-by-another-user", enabled=True), config):
                    update(bob, schedule, proposed, expect=(403,))
                bob.post(base + f"/schedules/{schedule['id']}/run", {}, expect=(403,))
            assert read(alice, schedule) == schedule
            assert alice.get(base + "/resources/runs")["items"] == []
            assert alice.get(base + f"/conversations/{conversation['id']}/messages")["items"] == []
            report["checks"].append("operator/admin/other-owner may read a shared schedule but cannot change, re-enable, no-op update or trigger its owner's intention")

            updated = update(alice, schedule, dict(config, prompt="owner-approved-revision"))
            update(alice, schedule, config, expect=(409,))
            accepted = alice.post(base + f"/schedules/{schedule['id']}/run", {})
            run = wait_for(
                lambda: alice.get(base + "/runs/" + accepted["run"]["id"]),
                lambda value: value["data"]["status"] in TERMINAL,
            )
            assert run["data"]["status"] == "completed", run
            messages = alice.get(base + f"/conversations/{conversation['id']}/messages")["items"]
            assert any(message.get("content") == "owner-approved-revision" for message in messages)
            assert "injected-by-another-user" not in json.dumps(messages)
            assert run["owner_id"] == updated["owner_id"]
            bob.get(base + "/runs/" + run["id"], expect=(404,))
            report["checks"].append("owner updates use CAS and the real scheduler/provider executes exactly the approved prompt with a private run receipt")

            for member_role in ("operator", "admin"):
                role(member_role)
                shared = create(alice, "schedules", config, "workspace")
                bob.request("DELETE", base + f"/resource/{shared['id']}?version={shared['version']}", expect=(204,))
                alice.get(base + "/resource/" + shared["id"], expect=(404,))
                private = create(alice, "schedules", config)
                bob.request("DELETE", base + f"/resource/{private['id']}?version={private['version']}", expect=(404,))
            report["checks"].append("moderation can delete shared jobs but cannot reach the owner's private jobs")

            shared_model = create(alice, "models", {
                "name": "Shared diagnostic", "provider": "diagnostic",
                "model": "diagnostic", "max_tokens": 256,
            }, "workspace")
            role("member")
            bob_conversation = create(bob, "conversations", {"title": "Bob scheduled conversation"})
            bob_config = dict(config, model_id=shared_model["id"], conversation_id=bob_conversation["id"],
                              prompt="must-not-run-after-revocation", at=int(time.time()) + 3, enabled=True)
            bob_schedule = create(bob, "schedules", bob_config, "workspace")
            update(alice, bob_schedule, dict(bob_config, prompt="instance-owner-injection"), expect=(403,))
            role("viewer")
            update(bob, bob_schedule, bob_config, expect=(403,))
            bob.post(base + f"/schedules/{bob_schedule['id']}/run", {}, expect=(403,))
            paused = wait_for(
                lambda: read(alice, bob_schedule),
                lambda value: value["data"].get("last_error") == "Schedule owner no longer has execution permission",
                timeout=15,
            )
            assert paused["data"]["enabled"] is False
            assert paused["data"]["status"] == "paused"
            assert bob.get(base + f"/conversations/{bob_conversation['id']}/messages")["items"] == []
            report["checks"].append("instance/workspace owner cannot replace a member's schedule; demotion blocks new claims and automatically pauses due jobs without model execution")

            alice.request("DELETE", base + "/members/" + bob_id, expect=(204,))
            update(bob, paused, bob_config, expect=(404,))
            bob.post(base + f"/schedules/{bob_schedule['id']}/run", {}, expect=(404,))
            unchanged = read(alice, bob_schedule)
            assert unchanged == paused
            report["checks"].append("full membership revocation denies schedule update/trigger and preserves the paused intention")
        finally:
            server.stop()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--keep-on-failure", action="store_true")
    parser.add_argument("--output", type=Path)
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    if not options.binary.is_file():
        parser.error("Build the workspace binaries first")
    with options.binary.open("rb") as binary:
        binary_sha256 = hashlib.file_digest(binary, "sha256").hexdigest()
    report = {
        "database": "postgresql" if options.database_url else "sqlite",
        "model": "diagnostic process fixture, not a live AI provider",
        "platform": platform.platform(),
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": binary_sha256,
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "checks": [], "status": "running",
    }
    started = time.monotonic()
    try:
        exercise(options, report)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = str(error)
    report["duration_seconds"] = round(time.monotonic() - started, 3)
    report["passed_checks"] = len(report["checks"])
    encoded = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if options.output:
        options.output.parent.mkdir(parents=True, exist_ok=True)
        options.output.write_text(encoded)
    print(encoded, end="")
    return int(report["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
