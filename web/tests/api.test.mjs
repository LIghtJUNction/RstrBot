import test from 'node:test';
import assert from 'node:assert/strict';
import { api, ApiError, isResourceUnavailable } from '../src/api.ts';

test('credential proof failures preserve a session while unauthorized expires it', async () => {
  const originalFetch = globalThis.fetch; const originalWindow = globalThis.window;
  const events = new EventTarget(); let expired = 0;
  events.addEventListener('rstrbot:session-expired', () => expired++);
  globalThis.window = events;
  try {
    for (const [status, code, expected] of [[403, 'reauth_failed', 0], [403, 'reauth_required', 0], [401, 'passkey_verification_failed', 0], [401, 'unauthorized', 1]]) {
      globalThis.fetch = async () => new Response(JSON.stringify({ error: { code, message: 'Test failure' } }), { status });
      await assert.rejects(api('/auth/reauth'), error => error instanceof ApiError && error.status === status && error.code === code);
      assert.equal(expired, expected, `${code} must preserve or expire the correct session`);
    }
  } finally { globalThis.fetch = originalFetch; if (originalWindow === undefined) delete globalThis.window; else globalThis.window = originalWindow; }
});

test('resource access barriers still recognize forbidden and missing responses', () => {
  for (const status of [401, 403, 404]) assert.equal(isResourceUnavailable(new ApiError(status, 'test', 'test')), true);
  for (const status of [400, 429, 500, 503]) assert.equal(isResourceUnavailable(new ApiError(status, 'test', 'test')), false);
});
