import { api } from './api';

export type AuthOptions = { google_enabled: boolean; passkey_enabled: boolean; email_enabled: boolean; username_required?: boolean; public_origin?: string | null; rp_id?: string | null };
export type VerifiedEmail = { email: string; verified_at: number; version: number };
export type Passkey = { id: string; name: string; created_at: number; last_used_at?: number | null; backup_eligible?: boolean | null; backed_up?: boolean | null };
export type GoogleAccount = { id: string; email: string; created_at: number };
export type SecurityState = { email: VerifiedEmail | null; google: GoogleAccount[]; passkeys: Passkey[]; recent_auth_until: number; has_password: boolean };
export type AuthSettings = {
  revision: number; public_origin: string | null; rp_id: string | null; passkey_count: number;
  google: { enabled: boolean; client_id: string; has_client_secret: boolean; redirect_uri: string | null };
  smtp: { enabled: boolean; host: string; port: number; security: 'starttls' | 'tls' | 'local'; username: string; has_password: boolean; from_email: string; from_name: string };
};

export async function startGoogle(mode: 'login' | 'link', returnPage = 'account', signal?: AbortSignal) {
  const result = await api<{ url: string }>('/auth/google/start', { method: 'POST', body: JSON.stringify({ mode }), signal });
  if (signal?.aborted) return;
  // The server owns the OAuth destination and signed, single-use state.
  try { if (mode !== 'login') sessionStorage.setItem('rstrbot:auth-return', returnPage); } catch { /* The return hint is optional. */ }
  window.location.assign(result.url);
}

export function authenticationError(error: unknown, t: (zh: string, en: string) => string): unknown {
  if (error instanceof DOMException && ['NotAllowedError', 'AbortError'].includes(error.name)) return new Error(t('验证已取消或未完成，请重新选择登录方式。', 'Verification was cancelled or did not complete. Choose a sign-in method and try again.'));
  if (error instanceof DOMException && error.name === 'InvalidStateError') return new Error(t('这枚通行密钥已经绑定。请选择其他密钥或设备。', 'This passkey is already registered. Choose another key or device.'));
  return error;
}
