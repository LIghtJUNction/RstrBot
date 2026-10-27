#!/usr/bin/env python3
"""SQLite <-> PostgreSQL snapshot migration with real CLI and provider traffic.

The provider is a local HTTP protocol fixture, not a commercial model. It checks
the decrypted synthetic credential in memory and never writes it to evidence.
PostgreSQL work uses disposable schemas, not existing application tables.
"""
import argparse
import copy
import datetime
import hashlib
import http.server
import json
import os
from pathlib import Path
import platform
import shutil
import threading
import time
import uuid

from integration import Client, Database, Instance, PASSWORD, ROOT, fixture_directory


class Provider(http.server.BaseHTTPRequestHandler):
    expected_key = None
    observations = []

    def log_message(self, *args):
        pass

    def do_POST(self):
        payload = json.loads(self.rfile.read(int(self.headers.get("Content-Length","0"))))
        authorized = self.headers.get("Authorization")=="Bearer "+self.expected_key
        self.observations.append({"authorized":authorized,"path":self.path,"model":payload.get("model"),"stream":payload.get("stream"),"max_tokens":payload.get("max_tokens")})
        response = {"id":"local-migration-receipt","model":"migration-protocol-fixture","choices":[{"index":0,"message":{"role":"assistant","content":"OK"},"finish_reason":"stop"}],"usage":{"prompt_tokens":4,"completion_tokens":1,"total_tokens":5}}
        body = json.dumps(response).encode()
        self.send_response(200 if authorized else 401)
        self.send_header("Content-Type","application/json")
        self.send_header("Content-Length",str(len(body)))
        self.end_headers()
        self.wfile.write(body)


def exercise_direction(options, report, receiver, source_backend, target_backend):
    database = Database(options.database_url)
    instances = []
    label = source_backend+" -> "+target_backend
    with fixture_directory(database,options.keep_on_failure,report) as directory:
        directory = Path(directory)
        try:
            source = Instance(directory/"source",options.binary,database.fresh() if source_backend=="postgresql" else None,allow_private_network=True)
            instances.append(source)
            source.start()
            alice,bob = Client(source.base),Client(source.base)
            proof = (source.directory/"bootstrap.token").read_text()
            original_user = alice.post("/api/v1/setup",{"token":proof,"username":"alice","password":PASSWORD,"workspace":"Migrated workspace"})["user"]
            workspace = alice.get("/api/v1/workspaces")["items"][0]
            base = f"/api/v1/workspaces/{workspace['id']}"
            bob_user = alice.post("/api/v1/admin/users",{"username":"bob","password":PASSWORD})["user"]
            alice.post(base+"/members",{"user_id":bob_user["id"],"role":"member"})
            bob.login("bob")
            private = bob.post(base+"/resources/conversations",{"visibility":"private","data":{"title":"Migration private sentinel"}})
            alice.get(base+"/resource/"+private["id"],expect=(404,))
            synthetic_secret = "migration-fixture-"+uuid.uuid4().hex
            Provider.expected_key = synthetic_secret
            model = alice.post(base+"/resources/models",{"visibility":"workspace","data":{"name":"Migrated encrypted provider","provider":"openai","model":"migration-protocol-fixture","base_url":f"http://127.0.0.1:{receiver.server_port}/v1","allow_private_network":True,"api_key":synthetic_secret}})
            assert model["data"]["has_api_key"] and synthetic_secret not in json.dumps(model)
            source.stop()
            snapshot_path = directory/"snapshot.json"
            source.maintenance("backup","--output",str(snapshot_path))
            snapshot = json.loads(snapshot_path.read_text())
            assert snapshot["source_backend"]==source_backend
            assert snapshot["includes_external_state"] is False and "sessions" not in snapshot["tables"]
            assert synthetic_secret not in snapshot_path.read_text()
            report["checks"].append(label+": real CLI snapshot contains encrypted provider configuration, stable identities and no authentication sessions")

            target = Instance(directory/"target",options.binary,database.fresh() if target_backend=="postgresql" else None,allow_private_network=True)
            instances.append(target)
            target.maintenance("restore","--input",str(snapshot_path))
            shutil.copyfile(source.directory/"secrets.key",target.directory/"secrets.key")
            os.chmod(target.directory/"secrets.key",0o600)
            target.start()
            stale_session = Client(target.base)
            for cookie in alice.cookies:
                stale_session.cookies.set_cookie(copy.copy(cookie))
            stale_session.get("/api/v1/auth/me",expect=(401,))
            restored_alice,restored_bob = Client(target.base),Client(target.base)
            assert restored_alice.login("alice")["id"]==original_user["id"]
            assert restored_bob.login("bob")["id"]==bob_user["id"]
            assert restored_alice.get("/api/v1/workspaces")["items"][0]["id"]==workspace["id"]
            restored_private = restored_bob.get(base+"/resource/"+private["id"])
            for key in ("id","owner_id","workspace_id","kind","visibility","version","data"):
                assert restored_private[key]==private[key],key
            restored_alice.get(base+"/resource/"+private["id"],expect=(404,))
            restored_model = restored_alice.get(base+"/resource/"+model["id"])
            assert restored_model==model
            report["checks"].append(label+": empty-target import preserves user/workspace/resource IDs, private ownership and same-workspace access denial")

            calls_before = len(Provider.observations)
            restored_alice.post(base+f"/models/{model['id']}/test",{},expect=(409,))
            restored_bob.post(base+"/chat",{"conversation_id":private["id"],"model_id":model["id"],"content":"must remain quarantined","idempotency_key":str(uuid.uuid4())},expect=(409,))
            assert len(Provider.observations)==calls_before
            report["checks"].append(label+": old sessions fail and recovery quarantine prevents both chat and provider tests before explicit resume")

            restored_alice.post("/api/v1/admin/recovery/complete",{},expect=(204,))
            invocation = restored_alice.post(base+f"/models/{model['id']}/test",{})
            assert invocation["success"] is True and invocation["fixture"] is False
            assert invocation["message"]["content"]=="OK" and invocation["usage"]["total_tokens"]==5
            assert len(Provider.observations)==calls_before+1
            assert Provider.observations[-1]=={"authorized":True,"path":"/v1/chat/completions","model":"migration-protocol-fixture","stream":False,"max_tokens":8}
            assert synthetic_secret not in json.dumps(restored_alice.get(base+"/events?limit=500"))
            report["checks"].append(label+": original encryption key decrypts the migrated credential and the real OpenAI-compatible worker sends authenticated HTTP after resume")

            after_path = directory/"source-after.json"
            source.maintenance("backup","--output",str(after_path))
            assert json.loads(after_path.read_text())["sha256"]==snapshot["sha256"]
            report["checks"].append(label+": source snapshot checksum remains unchanged after target restore and invocation")
        finally:
            for instance in reversed(instances):
                instance.stop()


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"target/debug/rstrbot")
    parser.add_argument("--database-url",default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"),required=not os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--keep-on-failure",action="store_true")
    parser.add_argument("--output",type=Path)
    options = parser.parse_args()
    options.binary=options.binary.resolve()
    report={"database":"SQLite <-> PostgreSQL","platform":platform.platform(),"provider":"local OpenAI-compatible HTTP protocol fixture, not a commercial provider","started_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"binary_sha256":{},"harness_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),"checks":[],"status":"running"}
    for executable in (options.binary,options.binary.parent/"rstrbot-openai"):
        with executable.open("rb") as stream:
            report["binary_sha256"][executable.name]=hashlib.file_digest(stream,"sha256").hexdigest()
    receiver = http.server.ThreadingHTTPServer(("127.0.0.1",0),Provider)
    receiver.daemon_threads=True
    worker=threading.Thread(target=receiver.serve_forever,daemon=True)
    worker.start()
    started=time.monotonic()
    try:
        exercise_direction(options,report,receiver,"sqlite","postgresql")
        exercise_direction(options,report,receiver,"postgresql","sqlite")
        report["status"]="passed"
    except Exception as error:
        report["status"]="failed"
        report["error"]=str(error)
    finally:
        receiver.shutdown()
        receiver.server_close()
        worker.join(timeout=3)
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
