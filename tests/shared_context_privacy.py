#!/usr/bin/env python3
"""Real HTTP regression for the audience of derived conversation content.

A loopback OpenAI-compatible fixture returns its system context verbatim. This
deliberately exposes any incorrectly supplied source in model output, durable
stream events and history; it is not a commercial model or an AI-quality test.
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
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, TERMINAL, wait_for


class Provider(http.server.BaseHTTPRequestHandler):
    records = []
    lock = threading.Lock()

    def log_message(self, *args):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers.get("Content-Length", "0"))))
        with self.lock:
            self.records.append(payload)
        system = "\n".join(
            message.get("content", "")
            for message in payload["messages"]
            if message.get("role") == "system"
        )
        aliases = [tool["function"]["name"] for tool in payload.get("tools", [])]
        content = "[Loopback context fixture; not AI]\n" + system + "\nTOOLS=" + json.dumps(aliases)
        chunk = {
            "id": "context-privacy-fixture",
            "model": "context-privacy-fixture",
            "choices": [{"index": 0, "delta": {"role": "assistant", "content": content}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 10, "completion_tokens": 10, "total_tokens": 20},
        }
        body = ("data: " + json.dumps(chunk) + "\n\ndata: [DONE]\n\n").encode()
        self.send_response(200)
        self.send_header("Content-Type", "text/event-stream")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def exercise(options, report):
    database = Database(options.database_url)
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Provider)
    receiver.daemon_threads = True
    worker = threading.Thread(target=receiver.serve_forever, daemon=True)
    worker.start()
    instance = None
    with tempfile.TemporaryDirectory(prefix="rstrbot-context-privacy-") as directory:
        try:
            instance = Instance(Path(directory) / "core", options.binary, database.fresh(), allow_private_network=True)
            instance.start()
            alice, bob = Client(instance.base), Client(instance.base)
            proof = (instance.directory / "bootstrap.token").read_text()
            alice.post("/api/v1/setup", {"token": proof, "username": "alice", "password": PASSWORD, "workspace": "Audience isolation"})
            workspace = alice.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"
            bob_user = alice.post("/api/v1/admin/users", {"username": "bob", "password": PASSWORD})["user"]
            alice.post(base + "/members", {"user_id": bob_user["id"], "role": "member"})
            bob.login("bob")

            def create(client, kind, data, visibility="private"):
                return client.post(base + "/resources/" + kind, {"visibility": visibility, "data": data})

            def execute(client, conversation, model, content="audienceprobe", **extra):
                body = {"conversation_id": conversation["id"], "model_id": model["id"], "content": content, "idempotency_key": str(uuid.uuid4()), **extra}
                admitted = client.post(base + "/chat", body)
                run = wait_for(lambda: client.get(base + "/runs/" + admitted["run"]["id"]), lambda value: value["data"]["status"] in TERMINAL)
                assert run["data"]["status"] == "completed", run
                wait_for(lambda: client.get(base + "/resource/" + conversation["id"]), lambda value: not value["data"].get("active_run"))
                return run

            model = create(alice, "models", {"name": "Loopback context fixture (not AI)", "provider": "openai", "model": "context-privacy-fixture", "base_url": f"http://127.0.0.1:{receiver.server_port}/v1", "allow_private_network": True}, "workspace")
            shared_doc = create(alice, "knowledge", {"title": "audienceprobe shared document", "text": "audienceprobe SHARED_DOCUMENT_SENTINEL"}, "workspace")
            create(alice, "memories", {"title": "Shared memory", "content": "SHARED_MEMORY_SENTINEL"}, "workspace")
            private_plugin = create(alice, "plugins", {"name": "Private diagnostic tool", "package_id": "diagnostic", "enabled": True, "config": {}})
            public_plugin = create(alice, "plugins", {"name": "Workspace diagnostic tool", "package_id": "diagnostic", "enabled": True, "config": {}}, "workspace")
            private_alias = "p_" + private_plugin["id"][:8] + "_echo"
            public_alias = "p_" + public_plugin["id"][:8] + "_echo"
            records = []
            for client, label in ((alice, "OWNER"), (bob, "MEMBER")):
                document_secret = "PRIVATE_" + label + "_DOCUMENT_SENTINEL"
                memory_secret = "PRIVATE_" + label + "_MEMORY_SENTINEL"
                document = create(client, "knowledge", {"title": "audienceprobe " + label + " private", "text": "audienceprobe " + document_secret})
                memory = create(client, "memories", {"title": label + " private memory", "content": memory_secret})
                records.append((client, bob if client is alice else alice, label, document, memory, document_secret, memory_secret))

            for client, observer, label, document, memory, document_secret, memory_secret in records:
                conversation = create(client, "conversations", {"title": label + " shared conversation"}, "workspace")
                shared = execute(client, conversation, model)
                with Provider.lock:
                    request = Provider.records[-1]
                request_text = json.dumps(request)
                tool_names = [tool["function"]["name"] for tool in request.get("tools", [])]
                visible = {
                    "run": observer.get(base + "/runs/" + shared["id"]),
                    "messages": observer.get(base + "/conversations/" + conversation["id"] + "/messages"),
                    "events": observer.get(base + "/events?limit=500"),
                    "export": observer.get(base + "/export"),
                }
                report["observations"].append({
                    "initiator": label.lower(),
                    "conversation_audience": "workspace",
                    "private_document_sent_to_provider": document_secret in request_text,
                    "private_memory_sent_to_provider": memory_secret in request_text,
                    "private_tool_advertised": private_alias in tool_names,
                    "shared_document_sent_to_provider": "SHARED_DOCUMENT_SENTINEL" in request_text,
                    "shared_memory_sent_to_provider": "SHARED_MEMORY_SENTINEL" in request_text,
                    "other_member_private_document_surfaces": [
                        surface for surface, value in visible.items()
                        if document_secret in json.dumps(value)
                    ],
                    "other_member_private_memory_surfaces": [
                        surface for surface, value in visible.items()
                        if memory_secret in json.dumps(value)
                    ],
                })
                assert document_secret not in request_text, "Shared conversation sent a private document to the model"
                assert memory_secret not in request_text, "Shared conversation sent a private memory to the model"
                assert private_alias not in tool_names, "Shared conversation advertised a private tool instance"
                assert public_alias in tool_names, "Shared conversation lost its authorized workspace tool"
                assert "SHARED_DOCUMENT_SENTINEL" in request_text and "SHARED_MEMORY_SENTINEL" in request_text
                assert shared_doc["id"] in json.dumps(shared["data"]["citations"])
                for surface, content in visible.items():
                    encoded = json.dumps(content)
                    for forbidden in (document_secret, memory_secret, document["id"], memory["id"]):
                        assert forbidden not in encoded, f"{surface} exposed {label.lower()}'s private source"
                report["checks"].append(label.lower() + ": shared model inputs, tools, messages, run, events and ordinary export contain no private source")

                private_conversation = create(client, "conversations", {"title": label + " private conversation"})
                private = execute(client, private_conversation, model)
                with Provider.lock:
                    request = Provider.records[-1]
                request_text = json.dumps(request)
                assert document_secret in request_text and memory_secret in request_text, "Private chat lost its owner's authorized context"
                assert document["id"] in json.dumps(private["data"]["citations"])
                tool_names = [tool["function"]["name"] for tool in request.get("tools", [])]
                assert (private_alias in tool_names) == (client is alice)
                observer.get(base + "/runs/" + private["id"], expect=(404,))
                observer.get(base + "/conversations/" + private_conversation["id"] + "/messages", expect=(404,))
                for path in ("/events?limit=500", "/export"):
                    encoded = json.dumps(observer.get(base + path))
                    assert document_secret not in encoded and memory_secret not in encoded
                personal_search = client.get(base + "/knowledge/search?q=audienceprobe")
                assert document["id"] in json.dumps(personal_search), "Personal search must retain private document access"
                report["checks"].append(label.lower() + ": private chat and personal search retain authorized private context; other members cannot read derived output")

            diagnostic = create(alice, "models", {"name": "Diagnostic (not AI)", "provider": "diagnostic", "model": "diagnostic"}, "workspace")
            public_plugin = alice.get(base + "/resource/" + public_plugin["id"])
            alice.request("PATCH", base + "/resource/" + public_plugin["id"], {"version": public_plugin["version"], "data": {**public_plugin["data"], "enabled": False}})
            shared_tool_chat = create(alice, "conversations", {"title": "Shared tool scope"}, "workspace")
            shared_tools = execute(alice, shared_tool_chat, diagnostic, "/diagnostic/tool private-tool-sentinel")
            assert shared_tools["data"]["tool_results"] == [], "A private tool ran for a shared conversation"
            private_tool_chat = create(alice, "conversations", {"title": "Private tool scope"})
            private_tools = execute(alice, private_tool_chat, diagnostic, "/diagnostic/tool private-tool-sentinel")
            assert len(private_tools["data"]["tool_results"]) == 1
            assert private_tools["data"]["tool_results"][0]["name"] == private_alias
            assert "Tool result received" in private_tools["data"]["content"]
            report["checks"].append("real diagnostic tool execution is excluded from shared conversations and preserved for its owner's private conversation")

            private_agent = create(alice, "agents", {"name": "Private agent", "system_prompt": "PRIVATE_AGENT_PROMPT_SENTINEL"})
            shared_agent = create(alice, "agents", {"name": "Workspace agent", "system_prompt": "SHARED_AGENT_PROMPT_SENTINEL"}, "workspace")
            agent_chat = create(alice, "conversations", {"title": "Shared agent admission"}, "workspace")
            body = {"conversation_id": agent_chat["id"], "model_id": diagnostic["id"], "content": "agent-admission", "agent_id": private_agent["id"], "idempotency_key": str(uuid.uuid4())}
            before = alice.get(base + "/conversations/" + agent_chat["id"] + "/messages")
            alice.post(base + "/chat", body, expect=(403,))
            assert before == alice.get(base + "/conversations/" + agent_chat["id"] + "/messages")
            shared_agent_run = execute(alice, agent_chat, diagnostic, "agent-admission", agent_id=shared_agent["id"], idempotency_key=body["idempotency_key"])
            assert shared_agent_run["data"]["agent_snapshot"]["system_prompt"] == "SHARED_AGENT_PROMPT_SENTINEL"
            assert "PRIVATE_AGENT_PROMPT_SENTINEL" not in json.dumps(bob.get(base + "/export"))
            private_agent_chat = create(alice, "conversations", {"title": "Private agent admission"})
            private_agent_run = execute(alice, private_agent_chat, diagnostic, agent_id=private_agent["id"])
            assert private_agent_run["data"]["agent_snapshot"]["system_prompt"] == "PRIVATE_AGENT_PROMPT_SENTINEL"
            report["checks"].append("private agent cannot be published through shared run admission; rejection leaves no history and the key can be retried; private chat still accepts it")
        finally:
            if instance:
                instance.stop()
            receiver.shutdown()
            receiver.server_close()
            worker.join(timeout=3)
            database.cleanup()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--output", type=Path)
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    report = {
        "database": "postgresql" if options.database_url else "sqlite",
        "platform": platform.platform(),
        "scope": "real local HTTP/SSE and diagnostic processes; synthetic source data; no commercial model",
        "started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "binary_sha256": {},
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
        "checks": [], "observations": [], "status": "running",
    }
    for executable in (options.binary, options.binary.parent / "rstrbot-openai", options.binary.parent / "rstrbot-diagnostic"):
        with executable.open("rb") as stream:
            report["binary_sha256"][executable.name] = hashlib.file_digest(stream, "sha256").hexdigest()
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
