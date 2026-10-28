#!/usr/bin/env python3
"""Real HTTP email/configuration/recovery and OAuth boundary acceptance.

Uses an in-process loopback SMTP receiver with synthetic addresses/secrets.
No real email or Google request is sent. Test-only SQL advances expiry/cooldown
boundaries; all identity changes under test go through the public HTTP API.
"""
import argparse
import datetime
from email import policy
from email.parser import BytesParser
import hashlib
import http.cookiejar
import json
import os
from pathlib import Path
import re
import shutil
import socketserver
import sqlite3
import subprocess
import tempfile
import threading
import time
import traceback
import urllib.parse
import urllib.request

from integration import Client, Database, Instance, PASSWORD, ROOT, wait_for


class MailReceiver(socketserver.StreamRequestHandler):
    messages = []
    lock = threading.Lock()

    def handle(self):
        self.connection.settimeout(10)
        self.wfile.write(b"220 localhost RstrBot test MTA\r\n")
        data = False
        body = []
        for line in self.rfile:
            if data:
                if line == b".\r\n":
                    with self.lock:
                        self.messages.append(BytesParser(policy=policy.default).parsebytes(b"".join(body)))
                    self.wfile.write(b"250 queued\r\n")
                    data = False
                    body = []
                else:
                    body.append(line[1:] if line.startswith(b"..") else line)
                continue
            command = line.split(b" ", 1)[0].strip().upper()
            if command in (b"EHLO", b"HELO"):
                self.wfile.write(b"250-localhost\r\n250-AUTH PLAIN LOGIN\r\n250 8BITMIME\r\n")
            elif command == b"AUTH":
                self.wfile.write(b"235 authentication successful\r\n")
            elif command == b"DATA":
                self.wfile.write(b"354 send message\r\n")
                data = True
            elif command == b"QUIT":
                self.wfile.write(b"221 bye\r\n")
                return
            else:
                self.wfile.write(b"250 OK\r\n")


class SmtpServer(socketserver.ThreadingTCPServer):
    allow_reuse_address = True
    daemon_threads = True


class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, *args, **kwargs):
        return None


def callback(client, query):
    opener = urllib.request.build_opener(urllib.request.HTTPCookieProcessor(client.cookies),
                                        urllib.request.ProxyHandler({}), NoRedirect())
    try:
        response = opener.open(client.base + "/api/v1/auth/google/callback?" + urllib.parse.urlencode(query), timeout=10)
    except urllib.error.HTTPError as error:
        response = error
    with response:
        assert response.status == 303
        assert response.headers["Location"] == "/?auth=failed"
        assert response.headers["Cache-Control"] == "no-store"


def database_sql(instance, query):
    url = instance.environment.get("RSTRBOT_DATABASE_URL")
    if url:
        subprocess.run([os.environ.get("RSTRBOT_PSQL") or shutil.which("psql"), url,
                        "-X", "-v", "ON_ERROR_STOP=1", "-qc", query],
                       check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    else:
        with sqlite3.connect(instance.directory / "rstrbot.db") as db:
            db.executescript(query)


def reset_cooldown(instance):
    database_sql(instance, "DELETE FROM auth_delivery_limits")


def mail_token(recipient, kind, minimum_count=1):
    def found():
        with MailReceiver.lock:
            return [m for m in MailReceiver.messages if recipient in str(m["To"])
                    and f"#{kind}=" in m.get_content()]
    messages = wait_for(found, lambda items: len(items) >= minimum_count)
    match = re.search(r"#" + re.escape(kind) + r"=([0-9a-f]{64})", messages[-1].get_content())
    assert match, "Actual SMTP delivery did not contain the expected verification link"
    return match.group(1)


def login(client, identifier, password=PASSWORD):
    return client.post("/api/v1/auth/login", {"username": identifier, "password": password})


def exercise(options, report):
    database = Database(options.database_url)
    smtp = SmtpServer(("127.0.0.1", 0), MailReceiver)
    threading.Thread(target=smtp.serve_forever, daemon=True).start()
    server = None
    checks = report["checks"]
    try:
        with tempfile.TemporaryDirectory(prefix="rstrbot-identity-") as temporary:
            server = Instance(Path(temporary) / "core", options.binary, database.fresh())
            server.start()
            owner = Client(server.base)
            owner.post("/api/v1/setup", {"token": (server.directory / "bootstrap.token").read_text(),
                "username": "identity-owner", "password": PASSWORD, "workspace": "Identity acceptance"})
            public = Client(server.base)
            available = public.get("/api/v1/auth/options")
            assert not any(available[k] for k in ("google_enabled", "passkey_enabled", "email_enabled"))
            public.get("/api/v1/admin/auth-settings", expect=(401,))
            owner.post("/api/v1/admin/users", {"username": "identity-member", "password": PASSWORD})
            member = Client(server.base)
            login(member, "identity-member")
            member.get("/api/v1/admin/auth-settings", expect=(403,))
            checks.append("disabled providers stay unavailable; instance configuration is administrator-only")

            config = {"revision": 0, "public_origin": server.base, "rp_id": "127.0.0.1",
                "google": {"enabled": True, "client_id": "123456789012-testfixture.apps.googleusercontent.com", "client_secret": "synthetic-google-client-secret"},
                "smtp": {"enabled": True, "host": "127.0.0.1", "port": smtp.server_address[1], "security": "local",
                    "username": "fixture-mta-user", "password": "synthetic-smtp-password", "from_email": "robot@example.test", "from_name": "RstrBot tests"}}
            saved = owner.request("PUT", "/api/v1/admin/auth-settings", config)
            assert saved["revision"] == 1 and saved["google"]["has_client_secret"] and saved["smtp"]["has_password"]
            assert not any(secret in json.dumps(saved) for secret in ("synthetic-", "_cipher"))
            owner.request("PUT", "/api/v1/admin/auth-settings", config, expect=(409,))
            config["revision"] = 1
            backup = owner.get("/api/v1/admin/backup")
            assert "synthetic-google-client-secret" not in json.dumps(backup)
            assert "synthetic-smtp-password" not in json.dumps(backup)
            public_settings = public.get("/api/v1/auth/options")
            assert public_settings["google_enabled"] and public_settings["email_enabled"]
            assert "client_id" not in public_settings and "smtp" not in public_settings
            checks.append("encrypted provider secrets are redacted and configuration saves use revision CAS")

            bad = json.loads(json.dumps(config))
            bad["smtp"]["host"] = "mail.example.test"
            owner.request("PUT", "/api/v1/admin/auth-settings", bad, expect=(400,))
            bad = json.loads(json.dumps(config)); bad["rp_id"] = "other.example.test"
            owner.request("PUT", "/api/v1/admin/auth-settings", bad, expect=(400,))
            bad = json.loads(json.dumps(config)); bad["smtp"]["port"] += 1; del bad["smtp"]["password"]
            owner.request("PUT", "/api/v1/admin/auth-settings", bad, expect=(400,))
            owner.post("/api/v1/admin/auth-settings/test-email", {"email": "delivery@example.test"})
            wait_for(lambda: len(MailReceiver.messages), lambda n: n == 1)
            checks.append("unsafe mail/origin settings and secret forwarding are rejected; saved SMTP actually delivers")

            database_sql(server, "UPDATE auth_session_security SET reauthenticated_at = 1; UPDATE sessions SET created_at = 1")
            denied = owner.request("PUT", "/api/v1/admin/auth-settings", config, expect=(403,))
            assert denied["error"]["code"] == "reauth_required"
            owner.post("/api/v1/auth/reauth", {"password": PASSWORD})
            current = owner.get("/api/v1/auth/security")
            assert current["recent_auth_until"] > int(time.time())
            checks.append("sensitive configuration requires fresh authentication without extending session expiry")

            response = owner.post("/api/v1/auth/email/start", {"email": "owner@example.test"})
            assert response["pending"] and "token" not in response
            email_token = mail_token("owner@example.test", "verify-email")
            other_session = Client(server.base); login(other_session, "identity-owner")
            other_session.post("/api/v1/auth/email/verify", {"token": email_token}, expect=(403,))
            result = owner.post("/api/v1/auth/email/verify", {"token": email_token})
            assert result["email"] == "owner@example.test"
            owner.post("/api/v1/auth/email/verify", {"token": email_token}, expect=(403,))
            login(Client(server.base), "OWNER@example.test")
            checks.append("email is verified once by its initiating session and becomes a working login identifier")

            owner.post("/api/v1/auth/email/start", {"email": "new-owner@example.test"}, expect=(429,))
            reset_cooldown(server)
            owner.post("/api/v1/auth/email/start", {"email": "new-owner@example.test"})
            new_token = mail_token("new-owner@example.test", "verify-email")
            assert owner.get("/api/v1/auth/security")["email"]["email"] == "owner@example.test"
            database_sql(server, "UPDATE auth_challenges SET expires_at = 1 WHERE purpose = 'email_change'")
            owner.post("/api/v1/auth/email/verify", {"token": new_token}, expect=(403,))
            assert owner.get("/api/v1/auth/security")["email"]["email"] == "owner@example.test"
            checks.append("pending/expired email replacements preserve the prior verified email and resend is bounded")

            unknown = public.post("/api/v1/auth/password/reset/request", {"email": "unknown@example.test"})
            accepted = public.post("/api/v1/auth/password/reset/request", {"email": "owner@example.test"})
            throttled = public.post("/api/v1/auth/password/reset/request", {"email": "owner@example.test"})
            assert unknown == accepted == throttled == {"accepted": True}
            reset_token = mail_token("owner@example.test", "reset-password")
            replacement = PASSWORD + " changed"
            public.post("/api/v1/auth/password/reset/finish", {"token": reset_token, "new_password": replacement}, expect=(204,))
            owner.get("/api/v1/auth/me", expect=(401,))
            other_session.get("/api/v1/auth/me", expect=(401,))
            public.post("/api/v1/auth/password/reset/finish", {"token": reset_token, "new_password": replacement}, expect=(400,))
            public.post("/api/v1/auth/login", {"username": "identity-owner", "password": PASSWORD}, expect=(401,))
            login(owner, "owner@example.test", replacement)
            checks.append("generic email recovery is single-use, changes the password and revokes every prior session")

            reset_cooldown(server)
            public.post("/api/v1/auth/password/reset/request", {"email": "owner@example.test"})
            stale = mail_token("owner@example.test", "reset-password", minimum_count=2)
            latest = replacement + " latest"
            owner.post("/api/v1/auth/password", {"current_password": replacement, "new_password": latest}, expect=(204,))
            public.post("/api/v1/auth/password/reset/finish", {"token": stale, "new_password": "stale replacement password"}, expect=(400,))
            login(owner, "identity-owner", latest)
            checks.append("a mailed recovery link cannot overwrite a newer password")

            # Every callback here fails before token exchange; no call to
            # Google's real endpoints is necessary or allowed in this fixture.
            def start_google():
                url = owner.post("/api/v1/auth/google/start", {"mode": "link"})["url"]
                parsed = urllib.parse.urlsplit(url)
                assert parsed.scheme == "https" and parsed.hostname == "accounts.google.com"
                params = urllib.parse.parse_qs(parsed.query)
                assert params["response_type"] == ["code"]
                assert params["code_challenge_method"] == ["S256"]
                assert len(params["code_challenge"][0]) == 43 and len(params["nonce"][0]) >= 32
                assert params["redirect_uri"] == [server.base + "/api/v1/auth/google/callback"]
                assert set(params["scope"][0].split()) == {"openid", "email"}
                return params["state"][0]

            google_state = start_google()
            callback(Client(server.base), {"state": google_state, "code": "synthetic-never-exchanged"})
            # A second browser's valid but unrelated binding must fail too.
            wrong_browser = Client(server.base)
            wrong_browser.post("/api/v1/auth/google/start", {"mode": "login"})
            callback(wrong_browser, {"state": google_state, "code": "synthetic-never-exchanged"})
            callback(owner, {"state": google_state, "code": "synthetic-never-exchanged"})
            assert owner.get("/api/v1/auth/security")["google"] == []
            checks.append("Google code flow carries state/nonce/PKCE; missing/wrong browser binding and replay cannot link")

            before = owner.get("/api/v1/admin/auth-settings")
            server.stop(); server.start()
            assert owner.get("/api/v1/admin/auth-settings") == before
            assert owner.get("/api/v1/auth/security")["email"]["email"] == "owner@example.test"
            login(Client(server.base), "owner@example.test", latest)
            checks.append("verified email, provider configuration and valid sessions survive a core restart")
    finally:
        if server:
            server.stop()
        smtp.shutdown(); smtp.server_close()
        database.cleanup()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, default=ROOT / "target/debug/rstrbot")
    parser.add_argument("--database-url", default=os.environ.get("RSTRBOT_TEST_DATABASE_URL"))
    parser.add_argument("--output", type=Path)
    options = parser.parse_args()
    options.binary = options.binary.resolve()
    started = time.monotonic()
    report = {"started_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
        "scope": "Real HTTP and loopback SMTP with synthetic addresses; negative OAuth callbacks only, no live Google account or external email",
        "database": "postgresql" if options.database_url else "sqlite",
        "binary_sha256": hashlib.sha256(options.binary.read_bytes()).hexdigest(),
        "harness_sha256": hashlib.sha256(Path(__file__).read_bytes()).hexdigest(), "checks": []}
    try:
        exercise(options, report)
        report["status"] = "passed"
    except Exception as error:
        report.update(status="failed", error=str(error), traceback=traceback.format_exc())
    report.update(passed_checks=len(report["checks"]), duration_seconds=round(time.monotonic() - started, 3))
    encoded = json.dumps(report, indent=2) + "\n"
    if options.output:
        options.output.parent.mkdir(parents=True, exist_ok=True)
        options.output.write_text(encoded)
    print(encoded, end="")
    return int(report["status"] != "passed")


if __name__ == "__main__":
    raise SystemExit(main())
