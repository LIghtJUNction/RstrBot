#!/usr/bin/env python3
"""Run the real-service HTTP matrix against already-built executables.

Provide a disposable PostgreSQL database (the harness creates unique schemas)
and a running local Valkey server. The fault-injection test starts its own
additional Valkey process. Connection credentials remain in child environment
variables and are never written to the evidence command list.
"""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import time
import urllib.parse


ROOT = Path(__file__).resolve().parents[1]


def endpoint(url):
    parsed = urllib.parse.urlsplit(url)
    return {"scheme":parsed.scheme,"hostname":parsed.hostname,"port":parsed.port}


def candidate_hashes(binary):
    hashes = {}
    for path in (binary, *(binary.parent/name for name in ("rstrbot-openai","rstrbot-diagnostic","rstrbot-webhook"))):
        with path.open("rb") as stream:
            hashes[path.name] = hashlib.file_digest(stream,"sha256").hexdigest()
    return hashes


def harness_hashes():
    # Every direct test module can be imported as shared fixture support.
    return {path.relative_to(ROOT).as_posix():hashlib.sha256(path.read_bytes()).hexdigest()
            for path in sorted((ROOT/"tests").glob("*.py"))}


def matches_candidate(evidence, expected):
    recorded = evidence.get("binary_sha256")
    if isinstance(recorded, str):
        return recorded == expected.get("rstrbot")
    return (isinstance(recorded, dict) and bool(recorded)
            and all(expected.get(name) == digest for name, digest in recorded.items()))


def evidence_file(directory, name):
    """A receipt must be a direct child of its matrix's directory."""
    if not isinstance(name, str) or Path(name).name != name:
        raise ValueError("Invalid evidence filename in previous matrix")
    return directory / name


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary",type=Path,default=ROOT/"target/debug/rstrbot")
    parser.add_argument("--database-url",default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"),required=not os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--valkey-url",default=os.environ.get("RSTRBOT_TEST_VALKEY_URL"),required=not os.environ.get("RSTRBOT_TEST_VALKEY_URL"))
    parser.add_argument("--unreachable-valkey-url",default="redis://127.0.0.1:9")
    parser.add_argument("--valkey-server",type=Path,required=True)
    parser.add_argument("--valkey-cli",type=Path,required=True)
    parser.add_argument("--output-directory",type=Path,default=ROOT/"docs/evidence")
    parser.add_argument("--keep-on-failure",action="store_true")
    parser.add_argument("--resume-from",type=Path,help="Reuse passing rows only when candidate, runner, harnesses and service endpoints still match")
    options = parser.parse_args()
    output = options.output_directory.resolve()
    output.mkdir(parents=True,exist_ok=True)
    binary = options.binary.resolve()
    report = {"started_at":datetime.datetime.now(datetime.timezone.utc).isoformat(),
              "runner_sha256":hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),
              "harness_sha256":harness_hashes(),
              "scope":"Real SQLite/PostgreSQL, real Valkey and local HTTP receivers; diagnostic model subprocess and local OpenAI-compatible protocol fixture, not a paid model provider or live messaging account",
              "database_endpoint":endpoint(options.database_url),
              "cache_endpoint":endpoint(options.valkey_url),
              "degraded_cache_endpoint":endpoint(options.unreachable_valkey_url),
              "candidate_sha256":candidate_hashes(binary),
              "cases":[],"status":"running"}
    cases = [
        ("http-sqlite-off","tests/integration.py",False,None,["--expected-cache-state","disabled"]),
        ("http-postgres-off","tests/integration.py",True,None,["--expected-cache-state","disabled"]),
        ("http-sqlite-valkey","tests/integration.py",False,options.valkey_url,["--expected-cache-state","available"]),
        ("http-postgres-valkey","tests/integration.py",True,options.valkey_url,["--expected-cache-state","available"]),
        ("http-sqlite-degraded-valkey","tests/integration.py",False,options.unreachable_valkey_url,["--expected-cache-state","degraded"]),
        ("http-postgres-degraded-valkey","tests/integration.py",True,options.unreachable_valkey_url,["--expected-cache-state","degraded"]),
        ("stream-persistence-sqlite","tests/stream_persistence.py",False,None,[]),
        ("history-recovery-sqlite","tests/history_recovery.py",False,None,[]),
        ("delivery-sqlite","tests/delivery_integration.py",False,None,[]),
        ("delivery-postgres","tests/delivery_integration.py",True,None,[]),
        ("delivery-session-revocation-sqlite","tests/delivery_session_revocation.py",False,None,[]),
        ("delivery-session-revocation-postgres","tests/delivery_session_revocation.py",True,None,[]),
        # Valkey's packaged server is a multicall executable: preserve the
        # symlink basename because its argv[0] selects server vs RDB-check mode.
        ("cache-recovery","tests/cache_recovery.py",False,None,["--valkey-server",str(options.valkey_server.absolute()),"--valkey-cli",str(options.valkey_cli.absolute())]),
        ("migration-cross-backend","tests/migration_integration.py",True,None,[]),
        ("shared-context-privacy-sqlite","tests/shared_context_privacy.py",False,None,[]),
        ("shared-context-privacy-postgres","tests/shared_context_privacy.py",True,None,[]),
        ("schedule-authority-sqlite","tests/schedule_authority.py",False,None,[]),
        ("schedule-authority-postgres","tests/schedule_authority.py",True,None,[]),
        ("session-revocation-sqlite","tests/session_revocation.py",False,None,[]),
        ("session-revocation-postgres","tests/session_revocation.py",True,None,[]),
        ("model-target-authority-sqlite","tests/model_target_authority.py",False,None,[]),
        ("model-target-authority-postgres","tests/model_target_authority.py",True,None,[]),
        ("model-test-revocation-sqlite","tests/model_test_revocation.py",False,None,[]),
        ("model-test-revocation-postgres","tests/model_test_revocation.py",True,None,[]),
        ("identity-sqlite","tests/identity_integration.py",False,None,[]),
        ("identity-postgres","tests/identity_integration.py",True,None,[]),
    ]
    report["planned_cases"] = [case[0] for case in cases]
    previous = {}
    previous_directory = None
    if options.resume_from:
        prior = json.loads(options.resume_from.read_text())
        for key in ("candidate_sha256", "runner_sha256", "harness_sha256", "database_endpoint", "cache_endpoint", "degraded_cache_endpoint", "planned_cases"):
            if prior.get(key) != report[key]:
                parser.error(f"Cannot resume evidence with changed {key}")
        previous_directory = options.resume_from.resolve().parent
        previous = {row["case"]:row for row in prior["cases"]}
        report["resumed_from"]=str(options.resume_from.resolve())
    started = time.monotonic()
    matrix_path = output/"integration-matrix.json"
    matrix_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
    for name,script,postgres,cache,arguments in cases:
        prior_row = previous.get(name)
        if prior_row and prior_row.get("status")=="passed" and prior_row.get("exit_code")==0:
            prior_evidence_path = evidence_file(previous_directory, prior_row["evidence"])
            prior_evidence = json.loads(prior_evidence_path.read_text()) if prior_evidence_path.exists() else {}
            prior_log_path = evidence_file(previous_directory, prior_row["log"])
            if (prior_evidence.get("status") == "passed" and prior_log_path.is_file()
                    and prior_evidence.get("harness_sha256") == report["harness_sha256"][script]
                    and matches_candidate(prior_evidence, report["candidate_sha256"])):
                if previous_directory != output:
                    shutil.copy2(prior_evidence_path, output/prior_evidence_path.name)
                    shutil.copy2(prior_log_path, output/prior_log_path.name)
                row = dict(prior_row,reused_from=str(options.resume_from.resolve()))
                report["cases"].append(row)
                matrix_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
                print(json.dumps({"case":name,"status":"reused passing evidence","passed_checks":row["passed_checks"]}),flush=True)
                continue
        environment = dict(os.environ)
        for key in ("RSTRBOT_TEST_DATABASE_URL","RSTRBOT_TEST_VALKEY_URL"):
            environment.pop(key,None)
        if postgres:
            environment["RSTRBOT_TEST_DATABASE_URL"] = options.database_url
        if cache:
            environment["RSTRBOT_TEST_VALKEY_URL"] = cache
        command = [sys.executable,str(ROOT/script),"--binary",str(binary),*arguments,"--output",str(output/(name+".json"))]
        if options.keep_on_failure and script in ("tests/integration.py","tests/migration_integration.py"):
            command.append("--keep-on-failure")
        evidence_path = output/(name+".json")
        if evidence_path.exists():
            evidence_path.unlink()
        with (output/(name+".log")).open("w") as log:
            result = subprocess.run(command,cwd=ROOT,env=environment,stdout=log,stderr=subprocess.STDOUT)
        try:
            evidence = json.loads(evidence_path.read_text()) if evidence_path.exists() else {}
            if not isinstance(evidence, dict):
                raise ValueError("Test receipt must be a JSON object")
        except (ValueError, OSError) as error:
            evidence = {"status":"invalid_report", "error":str(error)}
        if evidence.get("status") == "passed" and not matches_candidate(evidence, report["candidate_sha256"]):
            evidence = dict(evidence, status="failed", error="Receipt does not match the fixed candidate binaries")
        if evidence.get("status") == "passed" and evidence.get("harness_sha256") != report["harness_sha256"][script]:
            evidence = dict(evidence, status="failed", error="Receipt does not match the fixed test harness")
        row = {"case":name,"command":command,"environment_keys":[key for key in ("RSTRBOT_TEST_DATABASE_URL","RSTRBOT_TEST_VALKEY_URL","RSTRBOT_PSQL","LD_LIBRARY_PATH") if environment.get(key)],
               "exit_code":result.returncode,"status":evidence.get("status","missing_report"),
               "passed_checks":evidence.get("passed_checks",len(evidence.get("checks",[]))),"duration_seconds":evidence.get("duration_seconds"),
               "binary_sha256":evidence.get("binary_sha256"),"evidence":evidence_path.name,
               "log":name+".log","error":evidence.get("error")}
        report["cases"].append(row)
        matrix_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
        print(json.dumps({key:row[key] for key in ("case","exit_code","status","passed_checks","duration_seconds")}),flush=True)
        if candidate_hashes(binary)!=report["candidate_sha256"]:
            report["status"]="failed"
            report["error"]="Candidate binaries changed during acceptance; rerun against one fixed candidate"
            break
        if result.returncode or row["status"]!="passed":
            report["status"]="failed"
            break
    else:
        report["status"]="passed"
    if candidate_hashes(binary) != report["candidate_sha256"]:
        report.update(status="failed", error="Candidate binaries changed during acceptance; rerun against one fixed candidate")
    if harness_hashes() != report["harness_sha256"] or hashlib.sha256(Path(__file__).read_bytes()).hexdigest() != report["runner_sha256"]:
        report.update(status="failed", error="Test harness changed during acceptance; rerun against fixed verification scripts")
    report["duration_seconds"]=round(time.monotonic()-started,3)
    report["total_case_duration_seconds"]=round(sum(row.get("duration_seconds") or 0 for row in report["cases"]),3)
    report["scenario_group_executions"]=sum(row["passed_checks"] for row in report["cases"])
    report["scenario_count_note"]="Repeated across database/cache modes; these are scenario groups, not counts of distinct requirements or individual assertions"
    matrix_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+"\n")
    return int(report["status"]!="passed")


if __name__=="__main__":
    raise SystemExit(main())
