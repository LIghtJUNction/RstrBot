import test from 'node:test';
import assert from 'node:assert/strict';
import { creationOptions, requestOptions, encodeBase64url, decodeBase64url, credentialJSON } from '../src/passkey.ts';

const bytes = values => new Uint8Array(values).buffer;
test('base64url preserves leading zeros and non-ASCII bytes without padding', () => {
  const original = bytes([0, 0, 255, 251, 128, 1]);
  const encoded = encodeBase64url(original);
  assert.match(encoded, /^[A-Za-z0-9_-]+$/);
  assert.deepEqual(new Uint8Array(decodeBase64url(encoded)), new Uint8Array(original));
  assert.deepEqual(new Uint8Array(decodeBase64url('AA==')), new Uint8Array([0]));
});

test('malformed byte strings are rejected before calling the authenticator', () => {
  for (const value of ['a', 'bad+base64', 'contains space', '=AA', 'AA===', 'AA\n', 'a'.repeat(131073)]) assert.throws(() => decodeBase64url(value));
});

test('registration decodes all credential identifiers and preserves server verification policy', () => {
  const input = { rp: { id: 'example.test', name: 'RstrBot' }, challenge: 'AAE', user: { id: 'AgM', name: 'alice', displayName: 'Alice' }, pubKeyCredParams: [{ type: 'public-key', alg: -7 }], authenticatorSelection: { residentKey: 'preferred', userVerification: 'required' }, excludeCredentials: [{ id: 'BAU', type: 'public-key', transports: ['usb'] }] };
  const any = creationOptions(input);
  assert.equal(any.authenticatorSelection.userVerification, 'required');
  assert.equal(any.authenticatorSelection.authenticatorAttachment, undefined);
  assert.deepEqual([...new Uint8Array(any.excludeCredentials[0].id)], [4, 5]);
  const hardware = creationOptions(input, 'security-key');
  assert.equal(hardware.authenticatorSelection.authenticatorAttachment, 'cross-platform');
  assert.deepEqual(hardware.hints, ['security-key']);
  assert.equal(creationOptions(input, 'client-device').authenticatorSelection.authenticatorAttachment, 'platform');
  assert.equal(input.authenticatorSelection.authenticatorAttachment, undefined, 'Conversion cannot mutate the challenge snapshot');
});

test('authentication preserves RP and verification while converting allowed credential IDs', () => {
  const converted = requestOptions({ challenge: 'AAE', rpId: 'example.test', userVerification: 'required', allowCredentials: [{ type: 'public-key', id: 'AgM' }] });
  assert.equal(converted.rpId, 'example.test');
  assert.equal(converted.userVerification, 'required');
  assert.deepEqual([...new Uint8Array(converted.allowCredentials[0].id)], [2, 3]);
});

test('registration and login JSON contain the actual signed binary fields', () => {
  const common = { id: 'credential-id', rawId: bytes([0, 1]), type: 'public-key', authenticatorAttachment: 'cross-platform', getClientExtensionResults: () => ({ credProps: { rk: true } }) };
  const registration = credentialJSON({ ...common, response: { clientDataJSON: bytes([2, 3]), attestationObject: bytes([4, 5]), getTransports: () => ['usb'] } });
  assert.equal(registration.response.attestationObject, 'BAU');
  assert.deepEqual(registration.response.transports, ['usb']);
  const assertion = credentialJSON({ ...common, response: { clientDataJSON: bytes([2, 3]), authenticatorData: bytes([6, 7]), signature: bytes([8, 9]), userHandle: null } });
  assert.equal(assertion.response.authenticatorData, 'Bgc');
  assert.equal(assertion.response.signature, 'CAk');
  assert.equal(assertion.response.userHandle, null);
  assert.equal(assertion.rawId, 'AAE');
});

test('passkey names honor the shared UTF-8 limit for multilingual names', async () => {
  const { validPasskeyName } = await import('../src/passkey.ts');
  assert.equal(validPasskeyName('  日常安全密钥  '), true);
  assert.equal(validPasskeyName('a'.repeat(100)), true);
  assert.equal(validPasskeyName('名'.repeat(33)), true);
  assert.equal(validPasskeyName('名'.repeat(34)), false);
  assert.equal(validPasskeyName('🔑'.repeat(25)), true);
  assert.equal(validPasskeyName('🔑'.repeat(26)), false);
  assert.equal(validPasskeyName('has\u0000control'), false);
  assert.equal(validPasskeyName('  '), false);
});
