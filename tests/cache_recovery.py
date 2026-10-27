#!/usr/bin/env python3
"""Disposable real Valkey process: clear/stop/restart cannot erase durable facts."""
import argparse
import datetime
import hashlib
import json
from pathlib import Path
import socket
import subprocess
import tempfile
import time

from integration import Client, Instance, PASSWORD, ROOT, wait_for


def exercise(options, report):
    with tempfile.TemporaryDirectory(prefix="rstrbot-cache-") as directory:
        directory = Path(directory)
        with socket.socket() as probe:
            probe.bind(("127.0.0.1",0))
            port=probe.getsockname()[1]
        cache=None
        core=None
        cache_log=(directory/"valkey.log").open("ab")

        def start_cache():
            process = subprocess.Popen([str(options.valkey_server),"--bind","127.0.0.1","--port",str(port),"--protected-mode","yes","--save","","--appendonly","no","--dir",str(directory)],stdout=cache_log,stderr=subprocess.STDOUT)
            wait_for(lambda:subprocess.run([str(options.valkey_cli),"-p",str(port),"PING"],capture_output=True,text=True).stdout.strip(),lambda value:value=="PONG",timeout=5)
            info=subprocess.run([str(options.valkey_cli),"-p",str(port),"INFO","server"],capture_output=True,text=True,check=True).stdout.splitlines()
            server_info=dict(line.split(":",1) for line in info if ":" in line)
            assert process.poll() is None and server_info.get("process_id")==str(process.pid),"Refuse to clear a cache that is not this harness's disposable process"
            return process

        def stop_cache(process):
            if process and process.poll() is None:
                process.terminate()
                process.wait(timeout=5)

        try:
            cache=start_cache()
            core=Instance(directory/"core",options.binary,valkey_url=f"redis://127.0.0.1:{port}")
            core.start()
            alice,bob=Client(core.base),Client(core.base)
            proof=(core.directory/"bootstrap.token").read_text()
            alice.post("/api/v1/setup",{"token":proof,"username":"alice","password":PASSWORD,"workspace":"Cache resilience"})
            workspace=alice.get("/api/v1/workspaces")["items"][0]["id"]
            base=f"/api/v1/workspaces/{workspace}"
            wait_for(lambda:alice.get("/api/v1/setup"),lambda result:result.get("valkey_status")=="available",timeout=12)
            bob_user=alice.post("/api/v1/admin/users",{"username":"bob","password":PASSWORD})["user"]
            alice.post(base+"/members",{"user_id":bob_user["id"],"role":"member"})
            bob.login("bob")
            model=alice.post(base+"/resources/models",{"visibility":"workspace","data":{"name":"Diagnostic","provider":"diagnostic","model":"diagnostic"}})
            conversation=alice.post(base+"/resources/conversations",{"data":{"title":"Durable task"}})
            schedule=alice.post(base+"/resources/schedules",{"data":{"title":"Survives cache loss","prompt":"cache-outage-sentinel","conversation_id":conversation["id"],"model_id":model["id"],"schedule_type":"once","at":int(time.time())+8,"enabled":True}})
            alice.request("DELETE",base+"/members/"+bob_user["id"],expect=(204,))
            report["checks"].append("real Valkey availability was observed before storing a task and revoking a member")

            cleared=subprocess.run([str(options.valkey_cli),"-p",str(port),"FLUSHDB"],capture_output=True,text=True,check=True)
            assert cleared.stdout.strip()=="OK"
            stop_cache(cache)
            cache=None
            wait_for(lambda:alice.get("/api/v1/setup"),lambda result:result.get("valkey_status")=="degraded",timeout=12)
            bob.get(base+"/events",expect=(403,))
            finished=wait_for(lambda:alice.get(base+"/resource/"+schedule["id"]),lambda result:result["data"]["status"]=="completed",timeout=15)
            run_id=finished["data"]["run_id"]
            assert alice.get(base+"/runs/"+run_id)["data"]["status"]=="completed"
            report["checks"].append("cache flush and process shutdown preserve revocation; a durable scheduled run completes while cache is offline")

            cache=start_cache()
            wait_for(lambda:alice.get("/api/v1/setup"),lambda result:result.get("valkey_status")=="available",timeout=12)
            bob.get(base+"/events",expect=(403,))
            assert alice.get(base+"/resource/"+schedule["id"])["data"]["run_id"]==run_id
            messages=alice.get(base+f"/conversations/{conversation['id']}/messages")["items"]
            assert sum(message["role"]=="user" and message["content"]=="cache-outage-sentinel" for message in messages)==1
            report["checks"].append("empty-cache restart restores reported availability without restoring revoked access or duplicating the completed task")

            core.stop()
            core.start()
            assert alice.get(base+"/resource/"+schedule["id"])["data"]["run_id"]==run_id
            assert alice.get(base+f"/conversations/{conversation['id']}/messages")["items"]==messages
            bob.get(base+"/events",expect=(403,))
            report["checks"].append("core restart keeps the same task receipt and authorization after the cache failure sequence")
        finally:
            if core:
                core.stop()
            stop_cache(cache)
            cache_log.close()


def main():
    parser=argparse.ArgumentParser()
    parser.add_argument("--binary",type=Path,default=ROOT/"target/debug/rstrbot")
    parser.add_argument("--valkey-server",type=Path,required=True)
    parser.add_argument("--valkey-cli",type=Path,required=True)
    parser.add_argument("--output",type=Path)
    options=parser.parse_args()
    options.binary=options.binary.resolve()
    report={"database":"sqlite","cache":"real disposable Valkey","started_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),"checks":[],"status":"running","harness_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
    with options.binary.open("rb") as stream:
        report["binary_sha256"]=hashlib.file_digest(stream,"sha256").hexdigest()
    report["valkey_version"]=subprocess.run([str(options.valkey_server),"--version"],capture_output=True,text=True,check=True).stdout.strip()
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
