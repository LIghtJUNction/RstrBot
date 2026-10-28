#!/usr/bin/env node
import assert from 'node:assert/strict';
import { createHash, randomUUID } from 'node:crypto';
import { spawn } from 'node:child_process';
import { once } from 'node:events';
import fs from 'node:fs/promises';
import { createServer } from 'node:net';
import { createRequire } from 'node:module';
import os from 'node:os';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '../..');
const argument = (name, fallback) => process.argv.includes(name) ? process.argv[process.argv.indexOf(name) + 1] : fallback;
const binary = path.resolve(argument('--binary', path.join(root, 'target/debug/rstrbot')));
const executable = argument('--chromium', process.env.RSTRBOT_TEST_CHROMIUM);
const localRequire = createRequire(import.meta.url);
let load = localRequire;
try { load.resolve('playwright'); } catch {
  if (!process.env.CODEX_PRIMARY_RUNTIME_NODE_MODULES) throw new Error('Run npm ci in tests/browser, or provide CODEX_PRIMARY_RUNTIME_NODE_MODULES.');
  load = createRequire(path.join(process.env.CODEX_PRIMARY_RUNTIME_NODE_MODULES, 'package.json'));
}
const playwrightVersion = load('playwright/package.json').version;
assert.equal(playwrightVersion, '1.62.1', 'Use the pinned Playwright version');
const { chromium } = load('playwright');

const password = 'Browser fixture password 2026!';
const screenshots = path.join(root, 'artifacts/screenshots/security');
const evidence = path.resolve(argument('--evidence-dir', path.join(root, 'docs/evidence')));
await fs.mkdir(screenshots, { recursive: true });
await fs.mkdir(evidence, { recursive: true });
const sha256 = bytes => createHash('sha256').update(bytes).digest('hex');
async function filesBelow(directory) {
  const result = [];
  for (const entry of await fs.readdir(directory, { withFileTypes: true })) {
    const file = path.join(directory, entry.name);
    if (entry.isDirectory()) result.push(...await filesBelow(file));
    else if (entry.isFile()) result.push(file);
  }
  return result.sort();
}
const inputs = [binary, ...['rstrbot-diagnostic', 'rstrbot-openai', 'rstrbot-webhook'].map(name => path.join(path.dirname(binary), name)), ...await filesBelow(path.join(root, 'web/dist'))];
async function inputHashes() {
  const hashes = {};
  for (const file of inputs) hashes[file.startsWith(root + path.sep) ? path.relative(root, file) : path.basename(file)] = sha256(await fs.readFile(file));
  return hashes;
}
const report = {
  status: 'running', started_at: new Date().toISOString(),
  environment: { os: os.platform(), architecture: os.arch(), node: process.version, playwright: playwrightVersion, browser: '', database: 'SQLite', valkey: 'off', locale: 'en-US', timezone: 'Asia/Taipei', reduced_motion: true },
  input_sha256: await inputHashes(), harness_sha256: sha256(await fs.readFile(fileURLToPath(import.meta.url))),
  checks: [], screenshots: [], console_errors: [], network_errors: [],
  limitations: ['Virtual CTAP2 authenticators sign real WebAuthn challenges with UV; physical hardware and real cloud synchronization are not exercised.', 'Chromium only. No screen-reader, native desktop, production load or full browser-matrix acceptance.', 'SMTP is a local fixture. Google buttons and state boundaries are covered without contacting Google or claiming a real Google account login.'],
};
const directory = await fs.mkdtemp(path.join(os.tmpdir(), 'rstrbot-security-browser-'));
const listener = createServer(); listener.listen(0, '127.0.0.1'); await once(listener, 'listening');
const port = listener.address().port; await new Promise(resolve => listener.close(resolve));
const base = `http://localhost:${port}`;
const environment = { ...process.env };
for (const key of Object.keys(environment)) if (key.startsWith('RSTRBOT_')) delete environment[key];
const server = spawn(binary, ['--data-dir', directory, '--listen', `127.0.0.1:${port}`, '--web-dir', path.join(root, 'web/dist')], { env: environment, detached: os.platform() !== 'win32', stdio: ['ignore', 'pipe', 'pipe'] });
let serverOutput = '';
server.stdout.on('data', data => { serverOutput = (serverOutput + data).slice(-200000); });
server.stderr.on('data', data => { serverOutput = (serverOutput + data).slice(-200000); });
let browser, context, page, token = '', workspace, firstModel, privateConversation;
let currentCheck = '';
const wait = milliseconds => new Promise(resolve => setTimeout(resolve, milliseconds));
async function until(fn, description, timeout = 20000) {
  const end = Date.now() + timeout;
  let last;
  while (Date.now() < end) {
    try { const result = await fn(); if (result) return result; } catch (error) { last = error; }
    await wait(100);
  }
  throw new Error(`Timed out: ${description}${last ? ` (${last.message})` : ''}`);
}
async function api(method, route, data, expected = [200], request = context.request) {
  await paceAuthentication(route, method);
  const response = await request.fetch(base + '/api/v1' + route, { method, ...(data === undefined ? {} : { data }), headers: { Origin: base, 'X-RstrBot-Request': '1' } });
  const body = response.status() === 204 ? null : await response.json();
  assert.ok(expected.includes(response.status()), `${method} ${route}: ${response.status()} ${JSON.stringify(body).slice(0, 600)}`);
  return body;
}
const wp = () => `/workspaces/${workspace}`;
async function create(kind, data, visibility = 'private') { return api('POST', `${wp()}/resources/${kind}`, { visibility, data }, [200, 201]); }
async function nav(name) { await page.getByRole('navigation', { name: 'Main navigation' }).getByRole('button', { name, exact: true }).click(); }
async function screenshot(name, target = page) {
  const file = path.join(screenshots, `${name}.png`);
  await target.screenshot({ path: file, fullPage: true, animations: 'disabled' });
  const bytes = await fs.readFile(file);
  report.screenshots.push({ file: path.relative(root, file), sha256: sha256(bytes), bytes: bytes.length, viewport: target.viewportSize() });
}
async function check(name, fn) {
  currentCheck = name;
  const started = performance.now();
  await fn();
  report.checks.push({ name, status: 'passed', duration_ms: Math.round(performance.now() - started) });
  process.stdout.write(JSON.stringify({ passed: name }) + '\n');
}
async function assertNoOverflow() {
  const size = await page.evaluate(() => ({ width: innerWidth, document: document.documentElement.scrollWidth, body: document.body.scrollWidth }));
  assert.ok(size.document <= size.width + 1 && size.body <= size.width + 1, `Horizontal overflow: ${JSON.stringify(size)}`);
}
// Respect the production limiter instead of weakening it for acceptance.
// Negative proofs and registrations deliberately consume the same budget as
// normal sign-ins. Leave room for the isolated fixture setup and request jitter.
const authenticationAttempts = [];
async function paceAuthentication(route, method = 'POST') {
  const pathname = route.replace(/^\/api\/v1/, '');
  if (method !== 'POST' || !/^\/auth\/(reauth|login|passkeys\/(register|login|reauth)\/(options|finish))$/.test(pathname)) return;
  while (true) {
    const now = Date.now();
    while (authenticationAttempts.length && now - authenticationAttempts[0] >= 61000) authenticationAttempts.shift();
    if (authenticationAttempts.length < 9) { authenticationAttempts.push(now); return; }
    const remaining = 61000 - (now - authenticationAttempts[0]);
    process.stdout.write(JSON.stringify({ waiting_for: 'production authentication rate window', remaining_ms: remaining }) + '\n');
    await wait(Math.min(10000, remaining));
  }
}

// Fixture messages never leave loopback and are kept only in this process.
const mail = []; const smtpConnections = new Set();
const smtpServer = createServer(connection => {
  smtpConnections.add(connection); connection.on('close', () => smtpConnections.delete(connection));
  connection.setEncoding('utf8'); connection.write('220 localhost RstrBot browser fixture\r\n');
  let buffer = '', data = false, lines = [];
  connection.on('data', chunk => {
    buffer += chunk;
    while (buffer.includes('\r\n')) {
      const end = buffer.indexOf('\r\n'); const line = buffer.slice(0, end); buffer = buffer.slice(end + 2);
      if (data) {
        if (line === '.') { mail.push(lines.join('\r\n')); lines = []; data = false; connection.write('250 queued\r\n'); }
        else lines.push(line.startsWith('..') ? line.slice(1) : line);
      } else if (/^(EHLO|HELO)\b/.test(line)) connection.write('250-localhost\r\n250-AUTH PLAIN LOGIN\r\n250 8BITMIME\r\n');
      else if (/^AUTH\b/.test(line)) connection.write('235 authenticated\r\n');
      else if (line === 'DATA') { data = true; connection.write('354 send message\r\n'); }
      else if (line === 'QUIT') { connection.end('221 bye\r\n'); }
      else connection.write('250 OK\r\n');
    }
  });
  connection.on('error', () => {});
});
smtpServer.listen(0, '127.0.0.1'); await once(smtpServer, 'listening');
const smtpPort = smtpServer.address().port;
function emailToken(kind, recipient) {
  for (const message of [...mail].reverse()) {
    if (!message.includes(recipient)) continue;
    const split = message.indexOf('\r\n\r\n'); const header = message.slice(0, split); let body = message.slice(split + 4);
    if (/Content-Transfer-Encoding: base64/i.test(header)) body = Buffer.from(body.replace(/\s/g, ''), 'base64').toString();
    else body = body.replace(/=\r\n/g, '').replace(/=([\da-f]{2})/gi, (_, hex) => String.fromCharCode(parseInt(hex, 16)));
    const match = body.match(new RegExp('#' + kind + '=([a-f0-9]{64})')); if (match) return match[1];
  }
  return null;
}
let cdp, authenticator, physicalKey, syncedKey, replacementKey, physicalCredential, syncedCredential, replacementCredential;
const username = 'security_alice';
const providerSecret = 'synthetic-browser-google-secret';
const smtpPassword = 'synthetic-browser-smtp-password';
const credentials = [];
async function virtualAuthenticator(transport, backup = false, existing) {
  if (authenticator) await cdp.send('WebAuthn.removeVirtualAuthenticator', { authenticatorId: authenticator });
  const added = await cdp.send('WebAuthn.addVirtualAuthenticator', { options: {
    protocol: 'ctap2', ctap2Version: 'ctap2_1', transport,
    hasResidentKey: true, hasUserVerification: true, isUserVerified: true,
    automaticPresenceSimulation: true, defaultBackupEligibility: backup, defaultBackupState: backup,
  } });
  authenticator = added.authenticatorId;
  if (existing) await cdp.send('WebAuthn.addCredential', { authenticatorId: authenticator, credential: existing });
}
async function register(name, method = 'any', replaceName) {
  if (replaceName) await page.locator('.credential-row').filter({ hasText: replaceName }).getByRole('button', { name: 'Replace', exact: true }).click();
  else await page.getByRole('button', { name: 'Add passkey', exact: true }).click();
  await page.getByLabel('Passkey name', { exact: true }).fill(name);
  await page.getByLabel('Registration method', { exact: true }).selectOption(method);
  const done = page.waitForResponse(response => response.url().endsWith('/auth/passkeys/register/finish'));
  await page.getByRole('button', { name: 'Continue registration', exact: true }).click();
  const response = await done; const result = await response.json();
  assert.equal(response.status(), 200, JSON.stringify(result));
  await page.getByRole('heading', { name, exact: true }).waitFor();
  return { result, request: response.request().postDataJSON() };
}
async function signOut() {
  await page.getByRole('button', { name: 'Sign out', exact: true }).click();
  await page.getByRole('button', { name: 'Sign in', exact: true }).waitFor();
}
async function signInPasskey() {
  await page.getByLabel('Username or verified email', { exact: true }).fill(username);
  await page.getByRole('button', { name: 'Sign in with a passkey', exact: true }).click();
  await page.locator('.app-shell').waitFor();
}
async function security() { return api('GET', '/auth/security'); }
async function expireRecentFixture() {
  const program = "import sqlite3,sys; c=sqlite3.connect(sys.argv[1],timeout=10); c.execute('UPDATE auth_session_security SET reauthenticated_at=0'); c.commit()";
  const process = spawn('python3', ['-c', program, path.join(directory, 'rstrbot.db')], { stdio: ['ignore', 'pipe', 'pipe'] });
  const [code] = await once(process, 'exit'); assert.equal(code, 0, 'Fixture recent-auth expiry update');
}
function writableSettings(value) {
  return { revision: value.revision, public_origin: value.public_origin, rp_id: value.rp_id,
    google: { enabled: value.google.enabled, client_id: value.google.client_id },
    smtp: { enabled: value.smtp.enabled, host: value.smtp.host, port: value.smtp.port, security: value.smtp.security, username: value.smtp.username, from_email: value.smtp.from_email, from_name: value.smtp.from_name } };
}


try {
  await until(async () => { if (server.exitCode !== null) throw new Error('Core exited'); return (await fetch(base + '/health/live')).ok; }, 'server startup');
  token = (await fs.readFile(path.join(directory, 'bootstrap.token'), 'utf8')).trim();
  browser = await chromium.launch({ headless: true, ...(executable ? { executablePath: executable } : {}), args: ['--no-sandbox', '--disable-dev-shm-usage'] });
  report.environment.browser = browser.version();
  context = await browser.newContext({ viewport: { width: 1440, height: 1000 }, locale: 'en-US', timezoneId: 'Asia/Taipei', reducedMotion: 'reduce' });
  await context.route('**/*', async route => {
    if (new URL(route.request().url()).origin !== base) { report.network_errors.push(`Blocked unexpected origin: ${new URL(route.request().url()).origin}`); return route.abort(); }
    await paceAuthentication(new URL(route.request().url()).pathname, route.request().method());
    return route.continue();
  });
  page = await context.newPage(); page.setDefaultTimeout(90000);
  page.on('pageerror', error => report.console_errors.push(error.message));
  page.on('console', message => { if (message.type() === 'error' && !message.text().includes('Failed to load resource')) report.console_errors.push(message.text()); });
  cdp = await context.newCDPSession(page); await cdp.send('WebAuthn.enable');
  await api('POST', '/setup', { token, username, password, workspace: 'Studio workspace' }, [200, 201]);
  workspace = (await api('GET', '/workspaces')).items[0].id;

  await check('administrator configures a real origin, Google and loopback SMTP without secret echo', async () => {
    await page.goto(base + '/#auth-settings');
    await page.getByLabel('Public origin', { exact: true }).fill(base);
    await page.getByLabel('Passkey RP ID', { exact: true }).fill('localhost');
    await page.getByLabel('Enable Google sign-in', { exact: true }).check();
    await page.getByLabel('Google Client ID', { exact: true }).fill('123456789012-browserfixture.apps.googleusercontent.com');
    await page.getByLabel('Google Client secret', { exact: true }).fill(providerSecret);
    await page.getByLabel('Enable email', { exact: true }).check();
    await page.getByLabel('SMTP host', { exact: true }).fill('127.0.0.1');
    await page.getByLabel('Port', { exact: true }).fill(String(smtpPort));
    await page.getByLabel('Connection security', { exact: true }).selectOption('local');
    await page.getByLabel('SMTP username', { exact: true }).fill('fixture-mail-user');
    await page.getByLabel('SMTP password', { exact: true }).fill(smtpPassword);
    await page.getByLabel('Sender email', { exact: true }).fill('rstrbot@example.test');
    const saved = page.waitForResponse(response => response.url().endsWith('/admin/auth-settings') && response.request().method() === 'PUT');
    await page.getByRole('button', { name: 'Save settings', exact: true }).click();
    const response = await saved; const result = await response.json(); assert.equal(response.status(), 200, JSON.stringify(result));
    await until(async () => await page.getByLabel('Google Client secret', { exact: true }).inputValue() === '', 'secret cleared after save');
    assert.equal(await page.getByLabel('SMTP password', { exact: true }).inputValue(), '');
    assert.equal(result.google.has_client_secret, true); assert.equal(result.smtp.has_password, true);
    assert.ok(!JSON.stringify(result).includes(providerSecret) && !JSON.stringify(result).includes(smtpPassword));
    assert.equal(result.google.redirect_uri, `${base}/api/v1/auth/google/callback`);
    const options = await api('GET', '/auth/options'); assert.equal(options.passkey_enabled, true); assert.equal(options.google_enabled, true); assert.equal(options.email_enabled, true);
    await page.getByLabel('Test recipient email', { exact: true }).fill('ui-test@example.test');
    await page.getByRole('button', { name: 'Send test email', exact: true }).click();
    await page.getByText('The mail server accepted the test message. Check the destination inbox.', { exact: true }).waitFor();
    assert.ok(mail.some(message => message.includes('ui-test@example.test')));
    await screenshot('01-auth-settings-desktop');
  });

  await check('configuration refetch preserves local drafts and saving locks the submitted fields', async () => {
    const previous = await api('GET', '/admin/auth-settings');
    await page.getByLabel('Sender name', { exact: true }).fill('Unsubmitted local sender');
    await page.getByLabel('Google Client secret', { exact: true }).fill('unsubmitted-draft-secret');
    const concurrent = writableSettings(previous); concurrent.smtp.from_name = 'Concurrent administrator sender';
    const changed = await api('PUT', '/admin/auth-settings', concurrent, [200]);
    await until(async () => {
      await page.evaluate(() => window.dispatchEvent(new Event('visibilitychange')));
      return await page.getByText('The server configuration changed. Your inputs are preserved. Discard this draft and reload the latest version before saving.', { exact: true }).isVisible();
    }, 'focus refetch detects the remote revision');
    assert.equal(await page.getByLabel('Sender name', { exact: true }).inputValue(), 'Unsubmitted local sender');
    assert.equal(await page.getByLabel('Google Client secret', { exact: true }).inputValue(), 'unsubmitted-draft-secret');
    const conflict = page.waitForResponse(response => response.url().endsWith('/admin/auth-settings') && response.request().method() === 'PUT');
    await page.getByRole('button', { name: 'Save settings', exact: true }).click();
    assert.equal((await conflict).status(), 409);
    assert.equal((await api('GET', '/admin/auth-settings')).smtp.from_name, changed.smtp.from_name);
    await page.getByRole('button', { name: 'Discard draft & reload', exact: true }).click();
    await until(async () => await page.getByLabel('Sender name', { exact: true }).inputValue() === changed.smtp.from_name, 'explicit draft discard');
    assert.equal(await page.getByLabel('Google Client secret', { exact: true }).inputValue(), '');
    let release; const held = new Promise(resolve => { release = resolve; });
    let received; const pending = new Promise(resolve => { received = resolve; });
    await page.route('**/api/v1/admin/auth-settings', async route => {
      if (route.request().method() !== 'PUT') return route.continue();
      const response = await route.fetch(); received(); await held;
      await route.fulfill({ response });
    });
    await page.getByLabel('Sender name', { exact: true }).fill('Saved browser sender');
    const saved = page.waitForResponse(response => response.url().endsWith('/admin/auth-settings') && response.request().method() === 'PUT');
    await page.getByRole('button', { name: 'Save settings', exact: true }).click(); await pending;
    assert.equal(await page.getByLabel('Sender name', { exact: true }).isDisabled(), true);
    assert.equal(await page.getByLabel('Google Client secret', { exact: true }).isDisabled(), true);
    assert.equal(await page.getByLabel('Enable email', { exact: true }).isDisabled(), true);
    release(); assert.equal((await saved).status(), 200);
    await until(async () => await page.getByLabel('Sender name', { exact: true }).isEnabled(), 'save acknowledgement unlocks fields');
    await page.unroute('**/api/v1/admin/auth-settings');
    assert.equal(await page.getByLabel('Sender name', { exact: true }).inputValue(), 'Saved browser sender');
  });

  await check('incorrect reauthentication preserves the current account and a password proof succeeds', async () => {
    await nav('Account & security');
    await page.getByRole('button', { name: 'Verify identity again', exact: true }).click();
    await page.getByLabel('Current password', { exact: true }).fill('incorrect-fixture-password');
    await page.getByRole('button', { name: 'Verify identity', exact: true }).click();
    await page.locator('dialog[open] [role=alert]').waitFor();
    assert.equal((await api('GET', '/auth/me')).user.username, username);
    assert.equal(await page.locator('.app-shell').count(), 1);
    assert.equal(await page.getByRole('button', { name: 'Verify with Google', exact: true }).count(), 0);
    await page.getByLabel('Current password', { exact: true }).fill(password);
    await page.getByRole('button', { name: 'Verify identity', exact: true }).click();
    await page.locator('dialog[open]').waitFor({ state: 'hidden' });
    assert.ok((await security()).recent_auth_until > Date.now() / 1000);
  });

  await check('closing a pending password proof cannot authorize a different newly opened action', async () => {
    await expireRecentFixture();
    const refreshed = page.waitForResponse(response => response.url().endsWith('/auth/security') && response.status() === 200);
    await page.getByRole('button', { name: 'Refresh account', exact: true }).click(); await refreshed;
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
    assert.ok((await security()).recent_auth_until < Date.now() / 1000);
    let release; const held = new Promise(resolve => { release = resolve; });
    let received; const pending = new Promise(resolve => { received = resolve; });
    await page.route('**/api/v1/auth/reauth', async route => {
      await paceAuthentication('/auth/reauth');
      const response = await route.fetch(); received(); await held;
      try { await route.fulfill({ response }); } catch { /* A closed dialog aborts its own browser request. */ }
    });
    await page.getByRole('button', { name: 'Add passkey', exact: true }).click();
    await page.getByRole('heading', { name: 'Verify your identity', exact: true }).waitFor();
    await page.getByLabel('Current password', { exact: true }).fill(password);
    await page.getByRole('button', { name: 'Verify identity', exact: true }).click(); await pending;
    await page.locator('dialog[open]').getByRole('button', { name: 'Close', exact: true }).click();
    await page.getByRole('button', { name: 'Add email', exact: true }).click();
    await page.getByRole('heading', { name: 'Verify your identity', exact: true }).waitFor();
    release(); await page.unroute('**/api/v1/auth/reauth');
    // The first proof really succeeded on the server; only its detached UI
    // callback is forbidden to execute the second dialog's pending action.
    assert.ok((await security()).recent_auth_until > Date.now() / 1000);
    await page.evaluate(() => new Promise(resolve => requestAnimationFrame(resolve)));
    assert.equal(await page.getByRole('heading', { name: 'Verify an email address', exact: true }).count(), 0);
    assert.equal(await page.getByRole('heading', { name: 'Add passkey', exact: true }).count(), 0);
    assert.equal(await page.getByRole('heading', { name: 'Verify your identity', exact: true }).count(), 1);
    await page.getByLabel('Current password', { exact: true }).fill(password);
    await page.getByRole('button', { name: 'Verify identity', exact: true }).click();
    await page.getByRole('heading', { name: 'Verify an email address', exact: true }).waitFor();
    await page.locator('dialog[open]').getByRole('button', { name: 'Close', exact: true }).click();
  });

  await check('a USB authenticator registers a UV passkey and its consumed challenge cannot be replayed', async () => {
    await virtualAuthenticator('usb', false);
    const created = await register('Daily security key', 'security-key');
    physicalKey = (await security()).passkeys.find(key => key.name === 'Daily security key'); assert.ok(physicalKey);
    assert.equal(physicalKey.backup_eligible, false); assert.equal(physicalKey.backed_up, false);
    await page.getByText('Bound to its authenticator', { exact: true }).waitFor();
    physicalCredential = (await cdp.send('WebAuthn.getCredentials', { authenticatorId: authenticator })).credentials[0];
    assert.ok(physicalCredential && physicalCredential.rpId === 'localhost');
    await api('POST', '/auth/passkeys/register/finish', created.request, [400, 403]);
    assert.equal((await security()).passkeys.length, 1);
    credentials.push({ transport: 'usb', backup_eligible: false, backed_up: false, registered: true });
  });

  await check('a backup-capable platform authenticator adds a second credential and names can be changed', async () => {
    await virtualAuthenticator('internal', true);
    await register('Synced passkey', 'platform');
    syncedKey = (await security()).passkeys.find(key => key.name === 'Synced passkey'); assert.ok(syncedKey);
    assert.equal(syncedKey.backup_eligible, true); assert.equal(syncedKey.backed_up, true);
    await page.getByText('Authenticator reports a backup', { exact: true }).waitFor();
    syncedCredential = (await cdp.send('WebAuthn.getCredentials', { authenticatorId: authenticator })).credentials[0];
    assert.ok(syncedCredential.isResidentCredential);
    await page.getByRole('button', { name: 'Rename Daily security key', exact: true }).click();
    await page.getByLabel('Passkey name', { exact: true }).fill('Travel security key');
    await page.getByRole('button', { name: 'Save name', exact: true }).click();
    await page.getByRole('heading', { name: 'Travel security key', exact: true }).waitFor();
    assert.equal((await security()).passkeys.find(key => key.id === physicalKey.id).name, 'Travel security key');
    credentials.push({ transport: 'internal', backup_eligible: true, backed_up: true, registered: true, simulated_cloud_flags: true });
  });

  await check('email verification is delivered locally and requires a deliberate same-session confirmation', async () => {
    await page.getByRole('button', { name: 'Add email', exact: true }).click();
    await page.getByLabel('New email address', { exact: true }).fill('alice@example.test');
    await page.getByRole('button', { name: 'Send verification email', exact: true }).click();
    await page.getByText('Verification pending: alice@example.test', { exact: false }).waitFor();
    const verification = await until(() => emailToken('verify-email', 'alice@example.test'), 'actual verification mail');
    assert.equal((await security()).email, null);
    let confirmations = 0; const observer = request => { if (new URL(request.url()).pathname === '/api/v1/auth/email/verify') confirmations++; };
    page.on('request', observer);
    await page.goto(base + '/#verify-email=' + verification);
    await page.getByRole('button', { name: 'Confirm verification', exact: true }).waitFor();
    assert.equal(confirmations, 0, 'Opening an email link cannot consume the token');
    await page.getByRole('button', { name: 'Confirm verification', exact: true }).click();
    await page.getByText('Email verified.', { exact: true }).waitFor();
    assert.equal(confirmations, 1); page.off('request', observer);
    assert.equal((await security()).email.email, 'alice@example.test');
    assert.ok(!page.url().includes(verification));
    await screenshot('02-account-desktop');
    await page.setViewportSize({ width: 1440, height: 1700 });
    await page.getByRole('button', { name: 'Change language / 切换语言', exact: true }).click();
    await page.reload();
    await page.getByRole('heading', { name: 'Google 账号', exact: true }).waitFor();
    await screenshot('02b-account-full-desktop-zh');
    await page.getByRole('button', { name: 'Change language / 切换语言', exact: true }).click();
    await page.setViewportSize({ width: 1440, height: 1000 });
    await page.getByRole('button', { name: 'Light theme / 浅色主题', exact: true }).click();
    await screenshot('03-account-light-desktop');
    await page.getByRole('button', { name: 'Dark theme / 深色主题', exact: true }).click();
  });

  await check('a passkey can reauthenticate and cancelling replacement leaves both existing keys usable', async () => {
    await page.getByRole('button', { name: 'Verify identity again', exact: true }).click();
    await page.getByRole('button', { name: 'Verify with a passkey', exact: true }).click();
    await page.locator('dialog[open]').waitFor({ state: 'hidden' });
    assert.ok((await security()).recent_auth_until > Date.now() / 1000);
    // Refresh the virtual copy after its signature counter changes.
    syncedCredential = (await cdp.send('WebAuthn.getCredentials', { authenticatorId: authenticator })).credentials[0];
    await virtualAuthenticator('usb', false);
    await cdp.send('WebAuthn.setAutomaticPresenceSimulation', { authenticatorId: authenticator, enabled: false });
    await page.locator('.credential-row').filter({ hasText: 'Travel security key' }).getByRole('button', { name: 'Replace', exact: true }).click();
    await page.getByLabel('Passkey name', { exact: true }).fill('Cancelled replacement');
    await page.getByLabel('Registration method', { exact: true }).selectOption('security-key');
    const options = page.waitForResponse(response => response.url().endsWith('/auth/passkeys/register/options'));
    await page.getByRole('button', { name: 'Continue registration', exact: true }).click(); await options;
    await page.locator('dialog[open]').getByRole('button', { name: 'Close', exact: true }).click();
    await page.locator('dialog[open]').waitFor({ state: 'hidden' });
    const keys = (await security()).passkeys;
    assert.deepEqual(keys.map(key => key.id).sort(), [physicalKey.id, syncedKey.id].sort());
    await cdp.send('WebAuthn.setAutomaticPresenceSimulation', { authenticatorId: authenticator, enabled: true });
  });

  await check('replacement is atomic and the old authenticator cannot sign in with a fresh signed challenge', async () => {
    await register('Replacement security key', 'security-key', 'Travel security key');
    const keys = (await security()).passkeys;
    replacementKey = keys.find(key => key.name === 'Replacement security key');
    assert.ok(replacementKey && keys.some(key => key.id === syncedKey.id) && !keys.some(key => key.id === physicalKey.id));
    replacementCredential = (await cdp.send('WebAuthn.getCredentials', { authenticatorId: authenticator })).credentials[0];
    await signOut();
    await virtualAuthenticator('usb', false, physicalCredential);
    // A malicious client may request its removed credential from the browser.
    // Keep the real server challenge, sign with that actual UV authenticator,
    // and verify that the server rejects the credential removed by replacement.
    await page.route('**/api/v1/auth/passkeys/login/options', async route => {
      await paceAuthentication('/auth/passkeys/login/options');
      const response = await route.fetch(); const body = await response.json();
      body.publicKey.allowCredentials = [{ type: 'public-key', id: Buffer.from(physicalCredential.credentialId, 'base64').toString('base64url'), transports: ['usb'] }];
      await route.fulfill({ response, json: body });
    });
    await page.getByLabel('Username or verified email', { exact: true }).fill(username);
    const rejected = page.waitForResponse(response => response.url().endsWith('/auth/passkeys/login/finish'));
    await page.getByRole('button', { name: 'Sign in with a passkey', exact: true }).click();
    assert.equal((await rejected).status(), 401);
    await page.getByRole('alert').waitFor();
    assert.equal(await page.locator('.app-shell').count(), 0);
    await page.unroute('**/api/v1/auth/passkeys/login/options');
    await virtualAuthenticator('usb', false, replacementCredential);
    await signInPasskey();
    replacementCredential = (await cdp.send('WebAuthn.getCredentials', { authenticatorId: authenticator })).credentials[0];
    await nav('Account & security');
    assert.equal((await api('GET', '/auth/me')).user.username, username);
  });

  await check('removing one credential preserves the other and a removed key stays absent', async () => {
    await page.getByRole('button', { name: 'Remove Synced passkey', exact: true }).click();
    await page.getByRole('button', { name: 'Remove sign-in method', exact: true }).click();
    await page.getByRole('heading', { name: 'Synced passkey', exact: true }).waitFor({ state: 'hidden' });
    const keys = (await security()).passkeys;
    assert.deepEqual(keys.map(key => key.id), [replacementKey.id]);
    await signOut(); await signInPasskey(); await nav('Account & security');
    await page.getByRole('heading', { name: 'Replacement security key', exact: true }).waitFor();
    await page.getByRole('button', { name: 'Authentication & email', exact: true }).last().click();
    assert.equal(await page.getByLabel('Public origin', { exact: true }).isDisabled(), true);
    assert.equal(await page.getByLabel('Passkey RP ID', { exact: true }).isDisabled(), true);
  });

  await check('missing user verification and tampered signed origin are rejected without losing the main session', async () => {
    await nav('Account & security');
    await page.getByRole('button', { name: 'Verify identity again', exact: true }).click();
    await cdp.send('WebAuthn.setResponseOverrideBits', { authenticatorId: authenticator, isBadUV: true });
    let rejected = page.waitForResponse(response => response.url().endsWith('/auth/passkeys/reauth/finish'));
    await page.getByRole('button', { name: 'Verify with a passkey', exact: true }).click();
    assert.equal((await rejected).status(), 403);
    await page.locator('dialog[open] [role=alert]').waitFor();
    assert.equal((await api('GET', '/auth/me')).user.username, username);
    await cdp.send('WebAuthn.setResponseOverrideBits', { authenticatorId: authenticator });
    await page.route('**/api/v1/auth/passkeys/reauth/finish', async route => {
      await paceAuthentication('/auth/passkeys/reauth/finish');
      const body = route.request().postDataJSON();
      const client = JSON.parse(Buffer.from(body.credential.response.clientDataJSON, 'base64url').toString());
      client.origin = 'https://wrong-origin.example.invalid';
      body.credential.response.clientDataJSON = Buffer.from(JSON.stringify(client)).toString('base64url');
      await route.continue({ postData: JSON.stringify(body) });
    });
    rejected = page.waitForResponse(response => response.url().endsWith('/auth/passkeys/reauth/finish'));
    await page.getByRole('button', { name: 'Verify with a passkey', exact: true }).click();
    assert.equal((await rejected).status(), 403);
    assert.equal((await api('GET', '/auth/me')).user.username, username);
    await page.unroute('**/api/v1/auth/passkeys/reauth/finish');
    await page.getByRole('button', { name: 'Verify with a passkey', exact: true }).click();
    await page.locator('dialog[open]').waitFor({ state: 'hidden' });
  });

  await check('320px account and navigation remain usable in Chinese and login exposes configured methods', async () => {
    await nav('Account & security');
    await page.setViewportSize({ width: 320, height: 900 });
    await page.getByRole('button', { name: 'Change language / 切换语言', exact: true }).click();
    await page.getByRole('heading', { name: '账号与安全', exact: true }).waitFor();
    await page.locator('.main-content').evaluate(element => element.scrollTo({ top: 0 }));
    await assertNoOverflow(); await screenshot('04-account-dark-320-zh');
    await page.getByRole('button', { name: '打开导航', exact: true }).click();
    await page.getByRole('navigation', { name: '主导航', exact: true }).getByRole('button', { name: '认证与邮件', exact: true }).scrollIntoViewIfNeeded();
    await assertNoOverflow(); await screenshot('05-navigation-dark-320-zh');
    await page.keyboard.press('Escape');
    await page.getByRole('button', { name: 'Change language / 切换语言', exact: true }).click();
    await page.setViewportSize({ width: 1440, height: 1000 });
    await signOut();
    await page.getByRole('button', { name: 'Continue with Google', exact: true }).waitFor();
    assert.equal(await page.getByRole('button', { name: 'Sign in with a passkey', exact: true }).isEnabled(), true);
    await assertNoOverflow(); await screenshot('06-login-dark-desktop');
    await page.setViewportSize({ width: 320, height: 900 });
    await assertNoOverflow(); await screenshot('07-login-dark-320');
    await page.getByRole('button', { name: 'Light theme / 浅色主题', exact: true }).click();
    await screenshot('08-login-light-320');
    await page.getByRole('button', { name: 'Dark theme / 深色主题', exact: true }).click();
    await page.setViewportSize({ width: 1440, height: 1000 });
    await signInPasskey();
  });

  await check('a shared-cookie account change blocks cached private history and reloads the identity boundary', async () => {
    const secondUser = (await api('POST', '/admin/users', { username: 'cross_tab_bob', password }, [200, 201])).user;
    await api('POST', `${wp()}/members`, { user_id: secondUser.id, role: 'member' }, [200, 201]);
    const model = await create('models', { name: 'Identity boundary model', provider: 'diagnostic', model: 'diagnostic' });
    const conversation = await create('conversations', { title: 'CROSS TAB ALICE PRIVATE TITLE' });
    const run = await api('POST', `${wp()}/chat`, { conversation_id: conversation.id, model_id: model.id, content: 'CROSS TAB ALICE PRIVATE HISTORY', idempotency_key: randomUUID() });
    await until(async () => (await api('GET', `${wp()}/resource/${run.run.id}`)).data.status === 'completed', 'private history fixture');
    await nav('Conversations');
    await page.getByRole('button', { name: 'CROSS TAB ALICE PRIVATE TITLE', exact: true }).click();
    await page.locator('.message.user .message-body').filter({ hasText: 'CROSS TAB ALICE PRIVATE HISTORY' }).waitFor();
    await page.getByLabel('Message', { exact: true }).fill('CROSS TAB ALICE PRIVATE DRAFT');
    const leaks = [];
    await page.exposeFunction('recordIdentityLeak', () => leaks.push(true));
    await page.evaluate(() => {
      new MutationObserver(() => {
        if (document.querySelector('.account strong')?.textContent === 'cross_tab_bob' && /CROSS TAB ALICE PRIVATE/.test(document.body.innerText)) window.recordIdentityLeak();
      }).observe(document.body, { childList: true, subtree: true, characterData: true });
    });
    const secondTab = await context.newPage(); await secondTab.goto(base);
    const switchAccount = async name => {
      const response = await secondTab.evaluate(async ({ name, password }) => {
        const response = await fetch('/api/v1/auth/login', { method: 'POST', headers: { 'Content-Type': 'application/json', 'X-RstrBot-Request': '1' }, body: JSON.stringify({ username: name, password }) });
        return { status: response.status, result: await response.json() };
      }, { name, password });
      assert.equal(response.status, 200);
      // The production query freshness window is five seconds. Once stale,
      // focusing the original tab must observe the cookie changed in this tab.
      await wait(5100);
      const reload = page.waitForEvent('framenavigated', frame => frame === page.mainFrame());
      await page.bringToFront(); await page.evaluate(() => window.dispatchEvent(new Event('visibilitychange')));
      await reload; await page.locator('.app-shell').waitFor();
      await page.locator('.account strong').filter({ hasText: new RegExp(`^${name}$`) }).waitFor();
    };
    try {
      await switchAccount('cross_tab_bob');
      assert.deepEqual(leaks, [], 'Private content cannot appear under the incoming identity, even transiently');
      assert.ok(!(await page.locator('body').innerText()).includes('CROSS TAB ALICE PRIVATE'));
      assert.equal(await page.getByLabel('Message', { exact: true }).inputValue(), '');
      assert.equal((await api('GET', '/auth/me')).user.id, secondUser.id);
      await switchAccount(username);
    } finally { await secondTab.close(); }
  });

  await check('account settings remain accessible to a user with no workspace membership', async () => {
    await api('POST', '/admin/users', { username: 'unassigned_member', password }, [200, 201]);
    const isolated = await browser.newContext({ viewport: { width: 1280, height: 900 }, locale: 'en-US', timezoneId: 'Asia/Taipei' });
    try {
      await api('POST', '/auth/login', { username: 'unassigned_member', password }, [200], isolated.request);
      const member = await isolated.newPage(); await member.goto(base + '/#account');
      await member.getByRole('heading', { name: 'Account & security', exact: true }).waitFor();
      await member.getByRole('heading', { name: 'unassigned_member', exact: true }).waitFor();
      assert.equal(await member.getByRole('button', { name: 'Authentication & email', exact: true }).count(), 0);
      assert.deepEqual((await api('GET', '/workspaces', undefined, [200], isolated.request)).items, []);
      await screenshot('09-account-without-workspace', member);
    } finally { await isolated.close(); }
  });

  await check('password recovery uses delivered email, rejects mismatched inputs, and returns to sign-in', async () => {
    await signOut();
    await page.getByRole('button', { name: 'Forgot password?', exact: true }).click();
    await page.getByLabel('Email address', { exact: true }).fill('alice@example.test');
    await page.getByRole('button', { name: 'Send reset link', exact: true }).click();
    await page.getByText('If this email belongs to an account that can be recovered, a reset link will be sent. Check your inbox and spam folder.', { exact: true }).waitFor();
    const reset = await until(() => emailToken('reset-password', 'alice@example.test'), 'actual recovery mail');
    await page.goto(base + '/#reset-password=' + reset);
    await page.getByRole('heading', { name: 'Choose a new password', exact: true }).waitFor();
    const nextPassword = 'Updated browser fixture password 2026!';
    await page.getByLabel('New password', { exact: false }).first().fill(nextPassword);
    await page.getByLabel('Confirm new password', { exact: true }).fill('This does not match the password');
    await page.getByRole('button', { name: 'Save new password', exact: true }).click();
    await page.getByText('The passwords do not match.', { exact: true }).waitFor();
    await page.getByLabel('Confirm new password', { exact: true }).fill(nextPassword);
    await page.getByRole('button', { name: 'Save new password', exact: true }).click();
    await page.getByRole('heading', { name: 'Password updated', exact: true }).waitFor();
    assert.ok(!page.url().includes(reset));
    await page.getByRole('button', { name: 'Back to sign in', exact: true }).first().click();
    await page.getByLabel('Username or verified email', { exact: true }).fill('alice@example.test');
    await page.getByLabel('Password', { exact: true }).fill(nextPassword);
    await page.getByRole('button', { name: 'Sign in', exact: true }).click();
    await page.locator('.app-shell').waitFor();
    assert.equal((await api('GET', '/auth/me')).user.username, username);
    await screenshot('10-workspace-desktop');
  });

  await check('runtime, local-only requests and immutable acceptance inputs remain clean', async () => {
    assert.deepEqual(report.console_errors, []); assert.deepEqual(report.network_errors, []);
    assert.deepEqual(await inputHashes(), report.input_sha256);
    report.virtual_authenticators = credentials;
  });
  report.status = 'passed';
  await fs.rm(path.join(screenshots, 'failure.png'), { force: true });
} catch (error) {
  report.status = 'failed';
  report.checks.push({ name: currentCheck || 'startup', status: 'failed', error: error.stack ?? String(error) });
  if (page) { try { await screenshot('failure'); } catch { /* Preserve the primary failure. */ } }
  process.exitCode = 1;
  process.stderr.write(String(error.stack ?? error) + '\n');
} finally {
  if (browser) await browser.close();
  if (server.exitCode === null) {
    server.kill('SIGTERM');
    await Promise.race([once(server, 'exit'), wait(5000)]);
    if (server.exitCode === null) { try { process.kill(os.platform() === 'win32' ? server.pid : -server.pid, 'SIGKILL'); } catch { /* Already exited. */ } }
  }
  for (const connection of smtpConnections) connection.destroy();
  await new Promise(resolve => smtpServer.close(resolve));
  report.finished_at = new Date().toISOString();
  try {
    report.inputs_unchanged = JSON.stringify(await inputHashes()) === JSON.stringify(report.input_sha256);
    if (!report.inputs_unchanged) { report.status = 'invalidated'; process.exitCode = 1; }
  } catch (error) { report.status = 'invalidated'; report.input_read_error = error.message; process.exitCode = 1; }
  await fs.writeFile(path.join(evidence, 'security-browser.json'), JSON.stringify(report, null, 2) + '\n');
  await fs.writeFile(path.join(evidence, 'security-browser-server.log'), token ? serverOutput.split(token).join('[redacted initialization token]') : serverOutput);
  await fs.rm(directory, { recursive: true, force: true });
  process.stdout.write(JSON.stringify({ status: report.status, passed: report.checks.filter(check => check.status === 'passed').length, evidence: path.relative(root, path.join(evidence, 'security-browser.json')) }) + '\n');
}
