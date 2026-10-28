/** WebAuthn uses bytes in the browser and base64url strings over JSON. */
export type PasskeyHint = 'any' | 'security-key' | 'client-device';
type CredentialIdJSON = Omit<PublicKeyCredentialDescriptor, 'id'> & { id: string };
export type CreationJSON = Omit<PublicKeyCredentialCreationOptions, 'challenge' | 'user' | 'excludeCredentials'> & {
  challenge: string;
  user: Omit<PublicKeyCredentialUserEntity, 'id'> & { id: string };
  excludeCredentials?: CredentialIdJSON[];
};
export type RequestJSON = Omit<PublicKeyCredentialRequestOptions, 'challenge' | 'allowCredentials'> & {
  challenge: string;
  allowCredentials?: CredentialIdJSON[];
};

export function decodeBase64url(value: string): ArrayBuffer {
  if (typeof value !== 'string' || value.length > 131072 || !/^[A-Za-z0-9_-]*={0,2}$/.test(value) || value.replace(/=+$/, '').length % 4 === 1) {
    throw new Error('Invalid WebAuthn byte encoding');
  }
  const text = value.replace(/=+$/, '').replace(/-/g, '+').replace(/_/g, '/');
  const decoded = atob(text + '='.repeat((4 - text.length % 4) % 4));
  return Uint8Array.from(decoded, character => character.charCodeAt(0)).buffer;
}

export function encodeBase64url(value: ArrayBuffer): string {
  const bytes = new Uint8Array(value);
  let text = '';
  for (const byte of bytes) text += String.fromCharCode(byte);
  return btoa(text).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export function creationOptions(json: CreationJSON, hint: PasskeyHint = 'any'): PublicKeyCredentialCreationOptions {
  const result: PublicKeyCredentialCreationOptions & { hints?: string[] } = {
    ...json, challenge: decodeBase64url(json.challenge),
    user: { ...json.user, id: decodeBase64url(json.user.id) },
    excludeCredentials: json.excludeCredentials?.map(credential => ({ ...credential, id: decodeBase64url(credential.id) })),
  };
  if (hint !== 'any') {
    result.hints = [hint];
    result.authenticatorSelection = { ...result.authenticatorSelection, authenticatorAttachment: hint === 'security-key' ? 'cross-platform' : 'platform' };
  }
  return result;
}

export function requestOptions(json: RequestJSON, hint: PasskeyHint = 'any'): PublicKeyCredentialRequestOptions {
  return { ...json, challenge: decodeBase64url(json.challenge), allowCredentials: json.allowCredentials?.map(credential => ({ ...credential, id: decodeBase64url(credential.id) })), ...(hint === 'any' ? {} : { hints: [hint] }) };
}

export function credentialJSON(credential: PublicKeyCredential) {
  const response = credential.response;
  const common = { clientDataJSON: encodeBase64url(response.clientDataJSON) };
  return {
    id: credential.id, rawId: encodeBase64url(credential.rawId), type: credential.type,
    authenticatorAttachment: credential.authenticatorAttachment,
    clientExtensionResults: credential.getClientExtensionResults(),
    response: 'attestationObject' in response ? {
      ...common, attestationObject: encodeBase64url((response as AuthenticatorAttestationResponse).attestationObject),
      transports: (response as AuthenticatorAttestationResponse).getTransports?.() ?? [],
    } : {
      ...common, authenticatorData: encodeBase64url((response as AuthenticatorAssertionResponse).authenticatorData),
      signature: encodeBase64url((response as AuthenticatorAssertionResponse).signature),
      userHandle: (response as AuthenticatorAssertionResponse).userHandle ? encodeBase64url((response as AuthenticatorAssertionResponse).userHandle!) : null,
    },
  };
}

export function passkeysSupported(): boolean {
  return typeof window !== 'undefined' && window.isSecureContext && typeof PublicKeyCredential !== 'undefined' && !!navigator.credentials;
}

export async function registerCredential(json: CreationJSON, hint: PasskeyHint, signal?: AbortSignal) {
  if (!passkeysSupported()) throw new Error('Passkeys require HTTPS (or localhost) and a compatible browser.');
  const credential = await navigator.credentials.create({ publicKey: creationOptions(json, hint), signal });
  if (!(credential instanceof PublicKeyCredential)) throw new Error('No passkey was created.');
  return credentialJSON(credential);
}

export async function authenticateCredential(json: RequestJSON, hint: PasskeyHint = 'any', signal?: AbortSignal) {
  if (!passkeysSupported()) throw new Error('Passkeys require HTTPS (or localhost) and a compatible browser.');
  const credential = await navigator.credentials.get({ publicKey: requestOptions(json, hint), signal });
  if (!(credential instanceof PublicKeyCredential)) throw new Error('No passkey was returned.');
  return credentialJSON(credential);
}

export function validPasskeyName(value: string): boolean {
  const name = value.trim();
  return name.length > 0 && new TextEncoder().encode(name).byteLength <= 100 && !/\p{Cc}/u.test(name);
}
