#!/usr/bin/env python3
"""Exercise the real webhook MCP executable, including durable replay refusal."""
import argparse
import http.server
import json
import os
from pathlib import Path
import subprocess
import tempfile
import threading

ROOT = Path(__file__).resolve().parents[1]


class Destination(http.server.BaseHTTPRequestHandler):
    receipts = []

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        self.receipts.append(json.loads(body))
        self.send_response(202)
        self.send_header("Content-Length", "0")
        self.end_headers()

    def log_message(self, *args):
        pass


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot-webhook")
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Destination)
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    with tempfile.TemporaryDirectory(prefix="rb-mcp-") as directory:
        environment = {"PATH": os.environ.get("PATH", ""),
                       "RSTRBOT_WEBHOOK_URL": f"http://127.0.0.1:{server.server_port}/messages",
                       "RSTRBOT_WEBHOOK_STATE": directory, "RSTRBOT_ALLOW_PRIVATE_NETWORK": "1"}

        def process():
            return subprocess.Popen([str(options.binary), "--mcp-stdio"],
                                    env=environment, stdin=subprocess.PIPE, stdout=subprocess.PIPE,
                                    stderr=subprocess.DEVNULL, text=True)

        def request(worker, method, params=None):
            worker.stdin.write(json.dumps({"jsonrpc":"2.0","id":1,"method":method,"params":params or {}}) + "\n")
            worker.stdin.flush()
            result = json.loads(worker.stdout.readline())
            assert result["jsonrpc"] == "2.0" and result["id"] == 1, result
            return result

        worker = process()
        try:
            assert "error" in request(worker,"tools/list")
            assert request(worker,"initialize",{"protocolVersion":"2025-06-18"})["result"]["serverInfo"]["name"] == "rstrbot-webhook"
            assert request(worker,"tools/list")["result"]["tools"][0]["name"] == "send_message"
            escaped = request(worker,"tools/call",{"name":"send_message","arguments":{
                "text":"hello","operation_id":"mcp-1","url":"http://another-route.invalid"}})
            assert "error" in escaped and not Destination.receipts, escaped
            params = {"name":"send_message","arguments":{"text":"hello","operation_id":"mcp-1"}}
            result = request(worker,"tools/call",params)
            assert not result["result"]["isError"],result
            assert json.loads(result["result"]["content"][0]["text"])["delivery"] == "accepted",result
            assert not request(worker,"tools/call",params)["result"]["isError"]
            assert len(Destination.receipts) == 1,Destination.receipts
        finally:
            worker.stdin.close()
            worker.wait(timeout=5)
            worker.stdout.close()
        worker = process()
        try:
            request(worker,"initialize",{"protocolVersion":"2025-06-18"})
            assert not request(worker,"tools/call",params)["result"]["isError"]
            assert len(Destination.receipts) == 1,Destination.receipts
            params["arguments"]["text"] = "changed"
            assert request(worker,"tools/call",params)["result"]["isError"]
            assert len(Destination.receipts) == 1,Destination.receipts
        finally:
            worker.stdin.close()
            worker.wait(timeout=5)
            worker.stdout.close()
    server.shutdown()
    server.server_close()
    print(json.dumps({"status":"passed","checks":["initialization required","tool discovery","destination override refused",
        "actual HTTP delivery","duplicate not resent","restart not resent","changed parameters refused"]},indent=2))


if __name__ == "__main__":
    main()
