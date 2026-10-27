#!/usr/bin/env python3
"""Verify stored credentials and network grants remain bound to their destination.

All actual network calls go to this test's loopback receiver. Keys are synthetic,
kept in memory and represented only by boolean/label observations in evidence.
Reserved .invalid names exercise public HTTPS configuration without a DNS call.
"""
import argparse
import datetime
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import socket
import threading
import time
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, fixture_directory


class Receiver(http.server.BaseHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def do_POST(self):
        body = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        authorization = self.headers.get("Authorization")
        observation = {
            "path": self.path,
            "authorization_present": bool(authorization),
            "matching_fixture_keys": [
                name for name, value in self.server.fixture_keys.items()
                if authorization == "Bearer " + value
            ],
        }
        with self.server.lock:
            self.server.records.append(observation)
        if self.path == "/hook/unknown":
            self.close_connection = True
            self.connection.shutdown(socket.SHUT_RDWR)
            self.connection.close()
            return
        if self.path.endswith("/chat/completions"):
            response = {
                "id": "credential-target-fixture", "model": body.get("model"),
                "choices": [{"index": 0, "message": {"role": "assistant", "content": "OK"},
                             "finish_reason": "stop"}],
                "usage": {"prompt_tokens": 4, "completion_tokens": 1, "total_tokens": 5},
            }
        else:
            response = {"accepted": True}
        encoded = json.dumps(response).encode()
        self.send_response(200)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(encoded)))
        self.end_headers()
        self.wfile.write(encoded)


def exercise(options, report, receiver):
    database = Database(options.database_url)
    with fixture_directory(database, options.keep_on_failure, report) as directory:
        server = Instance(Path(directory) / "server", options.binary, database.fresh(),
                          allow_private_network=True)
        try:
            server.start()
            alice, bob = Client(server.base), Client(server.base)
            alice.post("/api/v1/setup", {
                "token": (server.directory / "bootstrap.token").read_text(),
                "username": "alice", "password": PASSWORD, "workspace": "Credential scope",
            })
            workspace = alice.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"
            bob_id = alice.post("/api/v1/admin/users", {
                "username": "bob", "password": PASSWORD,
            })["user"]["id"]
            alice.post(base + "/members", {"user_id": bob_id, "role": "operator"})
            bob.login("bob")

            def create(kind, config):
                return alice.post(base + "/resources/" + kind,
                                  {"visibility": "workspace", "data": config})

            def read(item):
                return alice.get(base + "/resource/" + item["id"])

            def update(client, item, config, **kwargs):
                return client.request("PATCH", base + "/resource/" + item["id"],
                                      {"version": item["version"], "data": config}, **kwargs)

            def test_model(client, model, fixture_key, path):
                count = len(receiver.records)
                result = client.post(base + f"/models/{model['id']}/test", {})
                assert result["success"] is True and result["message"]["content"] == "OK"
                assert len(receiver.records) == count + 1
                observed = receiver.records[-1]
                assert observed["path"] == path, observed
                assert observed["matching_fixture_keys"] == ([] if fixture_key is None else [fixture_key]), observed
                assert observed["authorization_present"] is (fixture_key is not None), observed

            def denied(client, item, config, status):
                before = read(item)
                calls = len(receiver.records)
                update(client, item, config, expect=(status,))
                assert read(item) == before, "A rejected edit changed persisted configuration"
                assert len(receiver.records) == calls, "An edit performed an unsolicited outbound request"

            endpoint = f"http://127.0.0.1:{receiver.server_port}"
            original = {
                "name": "Instance-approved local provider", "provider": "openai",
                "model": "credential-scope-fixture", "base_url": endpoint + "/v1",
                "allow_private_network": True, "max_tokens": 64,
            }
            model = create("models", dict(original, api_key=receiver.fixture_keys["model_original"]))
            test_model(alice, model, "model_original", "/v1/chat/completions")
            assert model["data"]["has_api_key"] is True
            assert all(key not in json.dumps(model) for key in receiver.fixture_keys.values())
            denied(bob, model, dict(original, base_url=endpoint + "/operator-target"), 403)
            report["checks"].append("instance-approved connection uses a synthetic encrypted key; operator target edits are denied without changing stored state")

            alice.post(base + "/members", {"user_id": bob_id, "role": "admin"})
            # URL parsing canonicalizes this IPv4 spelling to 127.0.0.1. It
            # does not authorize a different scheme, port, path or hostname.
            equivalent = dict(original, name="Renamed by workspace admin",
                              base_url=f"http://127.1:{receiver.server_port}/v1")
            model = update(bob, model, equivalent)
            test_model(bob, model, "model_original", "/v1/chat/completions")
            report["checks"].append("same normalized URL and provider permit ordinary edits and preserve the original key for a real authenticated model test")

            probe = dict(equivalent, base_url=endpoint + "/unapproved-target")
            try:
                denied(bob, model, probe, 403)
            except AssertionError:
                # If run against the original vulnerable binary, demonstrate
                # the risk using only the synthetic key and loopback receiver.
                changed = read(model)
                if changed["data"].get("base_url") == probe["base_url"]:
                    test_model(bob, changed, "model_original", "/unapproved-target/chat/completions")
                    report["unsafe_baseline_observation"] = {
                        "workspace_admin_changed_instance_approved_target": True,
                        "retained_synthetic_key_forwarded_to_changed_target": True,
                        "receiver": "isolated loopback fixture",
                    }
                raise
            for target in (
                endpoint + "/v1/another-tenant", endpoint + "/v10",
                f"http://127.0.0.1:{receiver.server_port + 1}/v1",
                f"http://127.0.0.2:{receiver.server_port}/v1",
                f"https://127.0.0.1:{receiver.server_port}/v1",
            ):
                denied(bob, model, dict(equivalent, base_url=target,
                                       api_key=receiver.fixture_keys["model_replacement"]), 403)
            report["checks"].append("workspace admin cannot carry a private-network grant to another path, prefix, host, port or scheme, even with a replacement key")

            public = dict(original, base_url="https://provider.example.invalid/v1", allow_private_network=False)
            public_model = create("models", dict(public, api_key=receiver.fixture_keys["model_original"]))
            for target in ("https://capture.example.invalid/v1", "https://provider.example.invalid/v2",
                           "https://provider.example.invalid:8443/v1"):
                denied(bob, public_model, dict(public, base_url=target), 400)
            public_model = update(bob, public_model,
                                  dict(public, base_url="HTTPS://PROVIDER.EXAMPLE.INVALID:443/v1"))
            assert public_model["data"]["has_api_key"] is True
            denied(bob, public_model, dict(public, provider="diagnostic"), 400)
            diagnostic = update(bob, public_model, dict(public, provider="diagnostic", clear_api_key=True,
                                                       allow_private_network=True))
            assert not diagnostic["data"].get("has_api_key")
            assert diagnostic["data"]["allow_private_network"] is False
            denied(bob, diagnostic, dict(original, api_key=receiver.fixture_keys["model_replacement"]), 403)
            report["checks"].append("public HTTPS and provider switches cannot retain someone else's key; host case/default-port normalization is allowed, and diagnostic cannot retain a network grant")

            destination = dict(original, base_url=endpoint + "/new-model-target")
            for credential in ({}, {"api_key": ""}, {"api_key": None}):
                denied(alice, model, dict(destination, **credential), 400)
            model = update(alice, model, dict(destination, clear_api_key=True))
            assert not model["data"].get("has_api_key")
            test_model(alice, model, None, "/new-model-target/chat/completions")
            replaced_config = dict(original, base_url=endpoint + "/replacement-model-target")
            model = update(alice, model, dict(replaced_config, api_key=receiver.fixture_keys["model_replacement"]))
            test_model(alice, model, "model_replacement", "/replacement-model-target/chat/completions")
            report["checks"].append("instance administrator must explicitly clear/replace a key for destination changes; blank/null do not count as clearing; actual requests contain only the chosen credential")

            def send(route, fixture_key, path, status="accepted"):
                calls = len(receiver.records)
                body = {
                    "text": "synthetic target-binding check", "idempotency_key": str(uuid.uuid4()),
                }
                receipt = alice.post(base + f"/routes/{route['id']}/send", body)
                assert receipt["data"]["status"] == status, receipt
                assert len(receiver.records) == calls + 1
                observed = receiver.records[-1]
                assert observed["path"] == path, observed
                assert observed["matching_fixture_keys"] == ([] if fixture_key is None else [fixture_key]), observed
                assert observed["authorization_present"] is (fixture_key is not None), observed
                return receipt, body

            route_config = {"name": "Approved webhook", "url": endpoint + "/hook/original",
                            "enabled": True, "allow_private_network": True}
            route = create("routes", dict(route_config, token=receiver.fixture_keys["webhook_original"]))
            send(route, "webhook_original", "/hook/original")
            disabled = update(bob, route, dict(route_config, enabled=False))
            denied(bob, disabled, route_config, 403)
            route = update(alice, disabled, route_config)
            denied(bob, route, dict(route_config, url=endpoint + "/hook/unapproved",
                                    token=receiver.fixture_keys["webhook_replacement"]), 403)
            for credential in ({}, {"token": ""}, {"token": None}):
                denied(alice, route, dict(route_config, url=endpoint + "/hook/new", **credential), 400)
            send(route, "webhook_original", "/hook/original")
            report["checks"].append("workspace admins may disable an approved route but cannot change its target or re-enable it; even instance admins cannot move a retained webhook token")

            route = update(alice, route, dict(route_config, url=endpoint + "/hook/cleared", clear_token=True))
            assert not route["data"].get("has_token")
            send(route, None, "/hook/cleared")
            route = update(alice, route, dict(route_config, url=endpoint + "/hook/replaced",
                                             token=receiver.fixture_keys["webhook_replacement"]))
            accepted_receipt, accepted_body = send(route, "webhook_replacement", "/hook/replaced")
            report["checks"].append("webhook target changes with explicit clear/replacement make actual requests with absent/new tokens, never the previous token")

            public_views = [alice.get(base + "/resources/models"), alice.get(base + "/resources/routes"),
                            alice.get(base + "/events?limit=500"), alice.get(base + "/export")]
            serialized = json.dumps(public_views)
            assert all(value not in serialized for value in receiver.fixture_keys.values())
            assert "api_key_cipher" not in serialized and "token_cipher" not in serialized
            report["checks"].append("resource lists, replay and ordinary workspace export contain no fixture plaintext keys or encrypted credential fields")

            unknown_route = create("routes", dict(route_config, url=endpoint + "/hook/unknown",
                                                  token=receiver.fixture_keys["webhook_replacement"]))
            unknown_receipt, unknown_body = send(unknown_route, "webhook_replacement", "/hook/unknown", "unknown")

            def restart_policy(allowed):
                server.stop()
                config_path = server.directory / "config.json"
                config = json.loads(config_path.read_text())
                config["allow_private_network"] = allowed
                config_path.write_text(json.dumps(config, indent=2) + "\n")
                server.environment.pop("RSTRBOT_ALLOW_PRIVATE_NETWORK", None)
                server.start()

            reservations_before = alice.get(base + "/reservations")
            calls_before = len(receiver.records)
            restart_policy(False)
            blocked_model = alice.post(base + f"/models/{model['id']}/test", {}, expect=(403,))
            assert blocked_model["error"]["code"] == "private_network_disabled"
            assert alice.get(base + "/reservations") == reservations_before
            blocked_body = {"text": "must remain unsent under disabled policy",
                            "idempotency_key": str(uuid.uuid4())}
            blocked_send = alice.post(base + f"/routes/{route['id']}/send", blocked_body, expect=(403,))
            assert blocked_send["error"]["code"] == "private_network_disabled"
            unsent_receipt = alice.post(base + f"/routes/{route['id']}/send", blocked_body)
            assert unsent_receipt["data"]["status"] == "not_sent"
            assert unsent_receipt["data"]["delivery"] == "not_sent"
            for saved_route, saved_body, saved_receipt in (
                (route, accepted_body, accepted_receipt),
                (unknown_route, unknown_body, unknown_receipt),
            ):
                replay = alice.post(base + f"/routes/{saved_route['id']}/send", saved_body)
                assert replay["id"] == saved_receipt["id"]
                assert replay["data"]["status"] == saved_receipt["data"]["status"]
            assert len(receiver.records) == calls_before
            report["checks"].append("disabling the global private-network policy across restart blocks saved model/route grants with 403 before traffic; no model reservation is added and accepted/unknown receipts remain retrievable")

            restart_policy(True)
            assert alice.post(base + f"/routes/{route['id']}/send", blocked_body)["id"] == unsent_receipt["id"]
            assert alice.post(base + f"/routes/{unknown_route['id']}/send", unknown_body)["id"] == unknown_receipt["id"]
            assert len(receiver.records) == calls_before
            test_model(alice, model, "model_replacement", "/replacement-model-target/chat/completions")
            send(route, "webhook_replacement", "/hook/replaced")
            report["checks"].append("explicitly re-enabling global policy permits fresh model/send operations while previously blocked and unknown delivery intentions are never automatically replayed")
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
        "provider": "local HTTP fixture with synthetic credentials; no commercial model or external messaging account",
        "platform": platform.platform(),
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": binary_sha256,
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "checks": [], "status": "running",
    }
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Receiver)
    receiver.daemon_threads = True
    receiver.lock = threading.Lock()
    receiver.records = []
    receiver.fixture_keys = {name: "synthetic-" + uuid.uuid4().hex for name in (
        "model_original", "model_replacement", "webhook_original", "webhook_replacement",
    )}
    thread = threading.Thread(target=receiver.serve_forever, daemon=True)
    thread.start()
    started = time.monotonic()
    try:
        exercise(options, report, receiver)
        report["status"] = "passed"
    except Exception as error:
        report["status"] = "failed"
        report["error"] = str(error)
    finally:
        receiver.shutdown()
        receiver.server_close()
        thread.join(timeout=5)
    report["duration_seconds"] = round(time.monotonic() - started, 3)
    report["passed_checks"] = len(report["checks"])
    report["loopback_requests"] = len(receiver.records)
    encoded = json.dumps(report, ensure_ascii=False, indent=2) + "\n"
    if options.output:
        options.output.parent.mkdir(parents=True, exist_ok=True)
        options.output.write_text(encoded)
    print(encoded, end="")
    return int(report["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
