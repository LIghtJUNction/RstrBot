#!/usr/bin/env python3
"""Real HTTP webhook acceptance: authorized routes, stable receipts, no unknown replay.

Uses a local HTTP receiver, not a real messaging account. The receiver records a
request before intentionally dropping some responses, so the test distinguishes
an observed side effect from an unconfirmed platform receipt.
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
import tempfile
import threading
import time
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, wait_for


class Receiver(http.server.BaseHTTPRequestHandler):
    records = []
    lock = threading.Lock()
    inflight_seen = threading.Event()
    release_inflight = threading.Event()

    def log_message(self, *args):
        pass

    def do_POST(self):
        body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
        with self.lock:
            self.records.append({"path":self.path,"body":json.loads(body),"authorization_present":bool(self.headers.get("Authorization"))})
            sequence = len(self.records)
        if self.path == "/uncertain":
            self.close_connection = True
            self.connection.shutdown(socket.SHUT_RDWR)
            self.connection.close()
            return
        if self.path == "/crash-pending":
            self.inflight_seen.set()
            self.release_inflight.wait(timeout=20)
        reply = json.dumps({"id":f"receiver-{sequence}"}).encode()
        self.send_response(201)
        self.send_header("Content-Type","application/json")
        self.send_header("Content-Length",str(len(reply)))
        self.end_headers()
        try:
            self.wfile.write(reply)
        except (BrokenPipeError, ConnectionResetError):
            pass


def exercise(options, report):
    database = Database(options.database_url)
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1",0),Receiver)
    receiver.daemon_threads = True
    worker = threading.Thread(target=receiver.serve_forever,daemon=True)
    worker.start()
    server = None
    with tempfile.TemporaryDirectory(prefix="rstrbot-delivery-") as directory:
        try:
            server = Instance(Path(directory)/"core",options.binary,database.fresh(),allow_private_network=True)
            server.start()
            alice,bob,outsider = Client(server.base),Client(server.base),Client(server.base)
            proof = (server.directory/"bootstrap.token").read_text()
            alice.post("/api/v1/setup",{"token":proof,"username":"alice","password":PASSWORD,"workspace":"Delivery workspace"})
            workspace = alice.get("/api/v1/workspaces")["items"][0]["id"]
            base = f"/api/v1/workspaces/{workspace}"
            bob_user = alice.post("/api/v1/admin/users",{"username":"bob","password":PASSWORD})["user"]
            alice.post(base+"/members",{"user_id":bob_user["id"],"role":"member"})
            alice.post("/api/v1/admin/users",{"username":"outsider","password":PASSWORD})
            bob.login("bob")
            outsider.login("outsider")
            destination = f"http://127.0.0.1:{receiver.server_port}"
            route_data = {"name":"Authorized receiver","url":destination+"/accepted","adapter":"webhook","enabled":True,"allow_private_network":True}
            bob.post(base+"/resources/routes",{"visibility":"workspace","data":route_data},expect=(403,))
            route = alice.post(base+"/resources/routes",{"visibility":"workspace","data":route_data})
            send_path = base+f"/routes/{route['id']}/send"
            body = {"text":"accepted-sentinel","idempotency_key":str(uuid.uuid4())}
            before = len(Receiver.records)
            outsider.post(send_path,body,expect=(403,404))
            bob.post(send_path,dict(body,url=destination+"/injected"),expect=(400,422))
            assert len(Receiver.records) == before
            report["checks"].append("workspace membership and administrator-approved destinations are required; body target override is rejected")

            receipt = bob.post(send_path,body)
            assert receipt["data"]["status"] == "accepted",receipt
            assert receipt["data"]["deduplication_guaranteed"] is False
            assert len(Receiver.records) == before+1
            duplicate = bob.post(send_path,body)
            assert duplicate["id"] == receipt["id"] and len(Receiver.records) == before+1
            bob.post(send_path,dict(body,text="changed-intention"),expect=(409,))
            assert len(Receiver.records) == before+1
            report["checks"].append("actual HTTP delivery stores an accepted receipt and identical retries do not resend")
            alice.get(base+"/resource/"+receipt["id"],expect=(404,))
            assert receipt["id"] not in json.dumps(alice.get(base+"/resources/deliveries"))
            assert "accepted-sentinel" not in json.dumps(alice.get(base+"/events?limit=500"))
            report["checks"].append("a workspace owner cannot inspect another member's private delivery receipt or replay text")

            uncertain_data = dict(route_data,name="Dropped response receiver",url=destination+"/uncertain")
            uncertain_route = alice.post(base+"/resources/routes",{"visibility":"workspace","data":uncertain_data})
            uncertain_path = base+f"/routes/{uncertain_route['id']}/send"
            uncertain_body = {"text":"uncertain-sentinel","idempotency_key":str(uuid.uuid4())}
            uncertain = bob.post(uncertain_path,uncertain_body)
            assert uncertain["data"]["status"] == "unknown",uncertain
            assert sum(r["path"]=="/uncertain" for r in Receiver.records)==1
            repeated = bob.post(uncertain_path,uncertain_body)
            assert repeated["id"]==uncertain["id"] and repeated["data"]["status"]=="unknown"
            assert sum(r["path"]=="/uncertain" for r in Receiver.records)==1
            report["checks"].append("receiver-side effect followed by a lost response stays unknown and is not retried")

            # Kill the core after the receiver observes an effect but before
            # any delivery acknowledgement can be persisted by the caller.
            pending_data = dict(route_data,name="Crash during send",url=destination+"/crash-pending")
            pending_route = alice.post(base+"/resources/routes",{"visibility":"workspace","data":pending_data})
            pending_path = base+f"/routes/{pending_route['id']}/send"
            pending_body = {"text":"crash-pending-sentinel","idempotency_key":str(uuid.uuid4())}
            pending_outcome = []

            def send_pending():
                try:
                    pending_outcome.append(bob.post(pending_path,pending_body))
                except (OSError, ConnectionError) as error:
                    pending_outcome.append(type(error).__name__)

            sender = threading.Thread(target=send_pending,daemon=True)
            sender.start()
            assert Receiver.inflight_seen.wait(timeout=10),"Receiver did not observe the pending effect"
            pending_receipt = wait_for(lambda:next((item for item in bob.get(base+"/resources/deliveries")["items"] if item["data"].get("route_id")==pending_route["id"]),None),lambda item:item is not None and item["data"]["status"]=="sending")
            server.stop(crash=True)
            Receiver.release_inflight.set()
            sender.join(timeout=5)
            assert not sender.is_alive()
            server.start()
            recovered_pending = bob.post(pending_path,pending_body)
            assert recovered_pending["id"]==pending_receipt["id"] and recovered_pending["data"]["status"]=="unknown"
            assert sum(r["path"]=="/crash-pending" for r in Receiver.records)==1
            after_restart = bob.post(uncertain_path,uncertain_body)
            assert after_restart["id"]==uncertain["id"] and after_restart["data"]["status"]=="unknown"
            assert sum(r["path"]=="/uncertain" for r in Receiver.records)==1
            assert bob.post(send_path,body)["id"]==receipt["id"]
            assert len(Receiver.records)==3
            report["checks"].append("hard kill during an observed send recovers it as unknown; accepted/unknown receipts survive and none are redispatched")

            alice.request("DELETE",base+"/members/"+bob_user["id"],expect=(204,))
            bob.post(send_path,{"text":"revoked-send","idempotency_key":str(uuid.uuid4())},expect=(403,404))
            assert len(Receiver.records)==3
            report["checks"].append("revoked membership blocks new external sends")
        finally:
            Receiver.release_inflight.set()
            if server:
                server.stop()
            receiver.shutdown()
            receiver.server_close()
            worker.join(timeout=3)
            database.cleanup()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--output",type=Path)
    options = parser.parse_args()
    options.binary=options.binary.resolve()
    report={"database":"postgresql" if options.database_url else "sqlite","platform":platform.platform(),"destination":"local HTTP receiver; no third-party messaging account","started_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"binary_sha256":{},"harness_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"checks":[],"status":"running"}
    for executable in (options.binary,options.binary.parent/"rstrbot-webhook"):
        with executable.open("rb") as stream:
            report["binary_sha256"][executable.name]=hashlib.file_digest(stream,"sha256").hexdigest()
    started=time.monotonic()
    try:
        exercise(options,report)
        report["status"]="passed"
    except Exception as error:
        report["status"]="failed"
        report["error"]=str(error)
    report["duration_seconds"]=round(time.monotonic()-started,3)
    report["passed_checks"]=len(report["checks"])
    encoded=json.dumps(report,ensure_ascii=False,indent=2)+"\n"
    if options.output:
        options.output.parent.mkdir(parents=True,exist_ok=True)
        options.output.write_text(encoded)
    print(encoded,end="")
    return int(report["status"]!="passed")


if __name__=="__main__":
    raise SystemExit(main())
