import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowRight, Check, CircleCheck, Cloud, Fingerprint, KeyRound, LockKeyhole, Mail, Pencil, Plus, RefreshCw, ShieldCheck, Trash2 } from 'lucide-react';
import { api, formatTime, patch, post, remove } from './api';
import type { User } from './api';
import { authorizedRead } from './data';
import { authenticateCredential, passkeysSupported, registerCredential, validPasskeyName } from './passkey';
import type { CreationJSON, PasskeyHint, RequestJSON } from './passkey';
import { authenticationError, startGoogle } from './security-api';
import type { AuthOptions, GoogleAccount, Passkey, SecurityState } from './security-api';
import { Button, ErrorNotice, Field, Loading, Modal, PageHeader, useText } from './ui';

export function useSecurity(user: User) {
  const client = useQueryClient(); const key = ['security', user.id];
  const query = useQuery<SecurityState | null>({ queryKey: key, queryFn: ({ signal }) => authorizedRead(client, key, signal, () => api<SecurityState>('/auth/security', { signal })) });
  return { ...query, data: query.data ?? undefined };
}

type SecureAction = () => void | Promise<void>;
export function useRecentAuth(user: User) {
  const state = useSecurity(user); const client = useQueryClient();
  const [open, setOpen] = useState<number | null>(null); const generation = useRef(0);
  const pending = useRef<{ generation: number; action: SecureAction } | null>(null);
  useLayoutEffect(() => () => { generation.current++; pending.current = null; }, [user.id]);
  const run = (action: SecureAction) => {
    if ((state.data?.recent_auth_until ?? 0) > Date.now() / 1000 + 5) { void action(); return; }
    const id = ++generation.current; pending.current = { generation: id, action }; setOpen(id);
  };
  const close = () => { generation.current++; pending.current = null; setOpen(null); };
  const verified = async (id: number) => {
    if (generation.current !== id) return;
    await client.invalidateQueries({ queryKey: ['security', user.id] });
    if (generation.current !== id) return;
    const request = pending.current; pending.current = null; generation.current++; setOpen(null);
    if (request?.generation === id) await request.action();
  };
  const verify = () => { const id = ++generation.current; pending.current = null; setOpen(id); };
  return { state, run, verify, dialog: open !== null && <Reauthenticate key={open} state={state.data} done={() => verified(open)} close={close} /> };
}

function Reauthenticate({ state, done, close }: { state?: SecurityState; done: () => Promise<void>; close: () => void }) {
  const { t } = useText(); const [password, setPassword] = useState(''); const [busy, setBusy] = useState(''); const [error, setError] = useState<unknown>(null);
  const controller = useRef<AbortController | null>(null); const attempt = useRef(0);
  useLayoutEffect(() => () => { attempt.current++; controller.current?.abort(); }, []);
  async function verify(method: 'password' | 'passkey', event?: React.FormEvent) {
    event?.preventDefault(); if (busy) return; setBusy(method); setError(null);
    const id = ++attempt.current; const operation = new AbortController(); controller.current = operation;
    const active = () => attempt.current === id && !operation.signal.aborted;
    const request = <T,>(path: string, body: unknown = {}) => api<T>(path, { method: 'POST', body: JSON.stringify(body), signal: operation.signal });
    try {
      if (method === 'password') await request('/auth/reauth', { password });
      else {
        const challenge = await request<{ challenge_id: string; publicKey: RequestJSON }>('/auth/passkeys/reauth/options');
        if (!active()) return;
        const credential = await authenticateCredential(challenge.publicKey, 'any', operation.signal);
        if (!active()) return;
        await request('/auth/passkeys/reauth/finish', { challenge_id: challenge.challenge_id, credential });
      }
      if (!active()) return;
      setPassword(''); await done();
    } catch (err) { if (active()) setError(authenticationError(err, t)); } finally { if (active()) setBusy(''); }
  }
  return <Modal title={t('确认是你本人', 'Verify your identity')} close={() => { controller.current?.abort(); close(); }}>
    <div className="stack"><p className="muted">{t('修改登录方式或邮件设置前，请再确认一次身份。验证在短时间内有效。', 'Verify again before changing sign-in methods or email settings. Verification remains valid briefly.')}</p><ErrorNotice error={error} />
      {state?.has_password !== false && <form className="stack" onSubmit={event => void verify('password', event)}><Field label={t('当前密码', 'Current password')}><input type="password" autoComplete="current-password" required value={password} onChange={event => setPassword(event.target.value)} autoFocus /></Field><Button variant="primary" busy={busy === 'password'} disabled={!!busy} type="submit">{t('确认身份', 'Verify identity')}<ArrowRight size={16} /></Button></form>}
      {!!state?.passkeys.length && <Button onClick={() => void verify('passkey')} disabled={!!busy || !passkeysSupported()} busy={busy === 'passkey'}><Fingerprint size={18} />{t('使用通行密钥确认', 'Verify with a passkey')}</Button>}

    </div>
  </Modal>;
}

export default function Account({ user, goAdmin, verifyToken, clearVerifyToken }: { user: User; goAdmin?: () => void; verifyToken?: string; clearVerifyToken: () => void }) {
  const { t, language } = useText(); const client = useQueryClient(); const { state, run, verify, dialog } = useRecentAuth(user);
  const googleController = useRef<AbortController | null>(null);
  useLayoutEffect(() => () => googleController.current?.abort(), []);
  const options = useQuery({ queryKey: ['auth-options'], queryFn: ({ signal }) => api<AuthOptions>('/auth/options', { signal }) });
  const [error, setError] = useState<unknown>(null); const [notice, setNotice] = useState(() => new URLSearchParams(location.search).get('auth') === 'success' ? t('Google 操作已完成。当前绑定结果显示在下方。', 'The Google operation completed. Your current linked accounts are shown below.') : ''); const [busy, setBusy] = useState('');
  const [add, setAdd] = useState<{ replace?: Passkey } | null>(null); const [rename, setRename] = useState<Passkey | null>(null); const [name, setName] = useState('');
  const [deleting, setDeleting] = useState<{ kind: 'passkey'; value: Passkey } | { kind: 'google'; value: GoogleAccount } | null>(null);
  const [emailOpen, setEmailOpen] = useState(false); const [email, setEmail] = useState(''); const [pendingEmail, setPendingEmail] = useState(''); const [passwordOpen, setPasswordOpen] = useState(false);
  const [currentPassword, setCurrentPassword] = useState(''); const [newPassword, setNewPassword] = useState('');
  const refresh = () => client.invalidateQueries({ queryKey: ['security', user.id] });
  async function action(id: string, operation: () => Promise<unknown>, message: string) {
    setBusy(id); setError(null); setNotice('');
    try { await operation(); await refresh(); setNotice(message); return true; }
    catch (err) { setError(authenticationError(err, t)); return false; } finally { setBusy(''); }
  }
  async function deleteCredential() {
    if (!deleting) return;
    const item = deleting;
    if (await action(`delete:${item.value.id}`, () => item.kind === 'passkey' ? remove(`/auth/passkeys/${encodeURIComponent(item.value.id)}`) : post('/auth/google/unlink', { id: item.value.id }), t('登录方式已移除。', 'Sign-in method removed.'))) setDeleting(null);
  }
  async function changeEmail(event: React.FormEvent) {
    event.preventDefault();
    if (await action('email', () => post('/auth/email/start', { email: email.trim() }), t('验证邮件已发送。请在此浏览器打开邮件中的链接并确认。', 'Verification sent. Open the email link in this browser and confirm.'))) { setPendingEmail(email.trim()); setEmailOpen(false); }
  }
  async function changePassword(event: React.FormEvent) {
    event.preventDefault(); setBusy('password'); setError(null);
    try { await post('/auth/password', { current_password: currentPassword, new_password: newPassword }); setCurrentPassword(''); setNewPassword(''); await client.cancelQueries(); client.clear(); window.location.assign('/'); }
    catch (err) { setError(err); } finally { setBusy(''); }
  }
  const security = state.data;
  return <section className="page security-page"><PageHeader title={t('账号与安全', 'Account & security')} description={t('管理你的邮箱、密码与登录凭据。这些设置适用于你的账号。', 'Manage your email, password and sign-in credentials across workspaces.')} action={<Button variant="ghost" onClick={() => void refresh()} aria-label={t('刷新账号信息', 'Refresh account')}><RefreshCw size={17} /></Button>} />
    <ErrorNotice error={error || state.error || options.error} onRetry={() => void refresh()} />{notice && <div className="notice success" role="status"><CircleCheck size={18} />{notice}</div>}
    {verifyToken && <EmailVerification key={verifyToken} user={user} token={verifyToken} done={() => { clearVerifyToken(); setPendingEmail(''); void refresh(); setNotice(t('邮箱验证完成。', 'Email verified.')); }} cancel={clearVerifyToken} />}
    <div className="account-overview"><span className="account-avatar">{user.username.slice(0, 2).toUpperCase()}</span><div><h2>{user.username}</h2><p className="muted">{security?.email?.email ?? t('尚未绑定邮箱', 'No verified email')}</p><code>{user.id}</code></div><span className="pill">{user.instance_admin ? t('实例管理员', 'Instance administrator') : t('个人账号', 'Personal account')}</span></div>
    {state.isPending ? <Loading /> : <>
      <section className="security-section"><div className="security-section-heading"><Mail size={20} /><div><h2>{t('邮箱', 'Email address')}</h2><p>{t('用于登录、验证与找回密码。替换前，原邮箱仍然有效。', 'Used for sign-in, verification and recovery. Your current email stays active until its replacement is verified.')}</p></div><Button disabled={!options.data?.email_enabled || !!busy} onClick={() => run(() => { setEmail(''); setEmailOpen(true); })}>{security?.email ? t('更换邮箱', 'Change email') : t('绑定邮箱', 'Add email')}</Button></div>
        <div className="security-detail">{security?.email ? <><span>{security.email.email}</span><span className="pill verified"><Check size={13} />{t('已验证', 'Verified')}</span></> : <span className="muted">{t('未设置', 'Not set')}</span>}</div>
        {pendingEmail && <p className="inline-note"><Mail size={16} />{t('等待验证：', 'Verification pending: ')}{pendingEmail}</p>}
        {!options.data?.email_enabled && <p className="inline-note">{t('管理员配置邮件发送后即可使用。', 'Available after an administrator configures outgoing email.')}</p>}
      </section>
      <section className="security-section"><div className="security-section-heading"><Fingerprint size={21} /><div><h2>{t('通行密钥', 'Passkeys')}</h2><p>{t('使用安全密钥、设备锁屏或密码管理器登录，可绑定多枚凭据。', 'Sign in with a security key, device unlock or password manager. Keep more than one credential for access.')}</p></div><Button disabled={!options.data?.passkey_enabled || !passkeysSupported() || !!busy} onClick={() => run(() => setAdd({}))}><Plus size={16} />{t('添加通行密钥', 'Add passkey')}</Button></div>
        {!options.data?.passkey_enabled && <p className="inline-note">{t('管理员需先配置此服务的公开访问地址。', 'An administrator must configure the public address of this server first.')}</p>}
        {!passkeysSupported() && <p className="inline-note">{t('请使用支持通行密钥的浏览器，并通过 HTTPS 或 localhost 访问。', 'Use a compatible browser over HTTPS or localhost to manage passkeys.')}</p>}
        <div className="credential-list">{security?.passkeys.map(passkey => <article className="credential-row" key={passkey.id}><span className="credential-icon">{passkey.backed_up ? <Cloud size={21} /> : <KeyRound size={21} />}</span><div className="credential-copy"><h3>{passkey.name}</h3><p>{passkey.backed_up === true ? t('认证器报告已备份', 'Authenticator reports a backup') : passkey.backup_eligible === true ? t('支持备份，尚未报告已备份', 'Backup capable; no backup reported') : passkey.backup_eligible === false ? t('凭据绑定于认证器', 'Bound to its authenticator') : t('备份状态未提供', 'Backup state not provided')}</p><p>{t('添加于 ', 'Added ')}{formatTime(passkey.created_at, language)}{passkey.last_used_at ? ` · ${t('最近使用 ', 'Last used ')}${formatTime(passkey.last_used_at, language)}` : ''}</p></div><div className="credential-actions"><Button variant="ghost icon" aria-label={`${t('重命名', 'Rename')} ${passkey.name}`} onClick={() => run(() => { setRename(passkey); setName(passkey.name); })}><Pencil size={16} /></Button><Button variant="ghost" onClick={() => run(() => setAdd({ replace: passkey }))}>{t('替换', 'Replace')}</Button><Button variant="ghost icon" aria-label={`${t('移除', 'Remove')} ${passkey.name}`} onClick={() => run(() => setDeleting({ kind: 'passkey', value: passkey }))}><Trash2 size={16} /></Button></div></article>)}</div>
        {!security?.passkeys.length && <p className="security-empty">{t('还没有通行密钥。添加一枚，之后可不输入密码登录。', 'No passkeys yet. Add one to sign in without typing your password.')}</p>}
        <p className="security-footnote">{t('云同步由系统或密码管理器负责。这里仅显示认证器报告的状态，不识别或推测云服务商。', 'Your system or password manager handles synchronization. These labels reflect authenticator-reported status, not a detected cloud provider.')}</p>
      </section>
      <section className="security-section"><div className="security-section-heading"><GoogleMark /><div><h2>{t('Google 账号', 'Google accounts')}</h2><p>{t('绑定后可使用 Google 登录。不会仅凭相同邮箱自动合并账号。', 'Link an account to sign in with Google. Matching email addresses never link accounts automatically.')}</p></div><Button disabled={!options.data?.google_enabled || !!busy} onClick={() => run(() => action('google', () => { googleController.current = new AbortController(); return startGoogle('link', 'account', googleController.current.signal); }, '').then(() => undefined))}><Plus size={16} />{t('绑定 Google', 'Link Google')}</Button></div>
        {security?.google.map(account => <article className="credential-row" key={account.id}><span className="credential-icon"><GoogleMark /></span><div className="credential-copy"><h3>{account.email || t('已绑定的 Google 账号', 'Linked Google account')}</h3><p>{t('绑定于 ', 'Linked ')}{formatTime(account.created_at, language)}</p></div><Button variant="ghost" onClick={() => run(() => setDeleting({ kind: 'google', value: account }))}>{t('解除绑定', 'Unlink')}</Button></article>)}
        {!security?.google.length && <p className="security-empty">{options.data?.google_enabled ? t('尚未绑定 Google 账号。', 'No Google account linked.') : t('管理员配置 Google 登录后即可绑定。', 'Available after an administrator configures Google sign-in.')}</p>}
      </section>
      <section className="security-section"><div className="security-section-heading"><LockKeyhole size={20} /><div><h2>{t('密码', 'Password')}</h2><p>{t('修改后会退出所有登录设备，需要重新登录。', 'Changing your password signs out every session. You will need to sign in again.')}</p></div><Button disabled={security?.has_password === false} onClick={() => { setError(null); setPasswordOpen(true); }}>{t('修改密码', 'Change password')}</Button></div></section>
      <div className="security-bottom"><Button variant="ghost" onClick={verify}><ShieldCheck size={16} />{t('重新确认身份', 'Verify identity again')}</Button>{goAdmin && <Button onClick={goAdmin}>{t('认证与邮件设置', 'Authentication & email')}<ArrowRight size={16} /></Button>}</div>
    </>}
    {dialog}
    {add && <PasskeyEditor authorize={run} replace={add.replace} close={() => setAdd(null)} saved={() => { setAdd(null); void refresh(); setNotice(t('通行密钥已保存。', 'Passkey saved.')); }} />}
    {rename && <Modal title={t('重命名通行密钥', 'Rename passkey')} close={() => setRename(null)}><form className="stack" onSubmit={async event => { event.preventDefault(); if (!validPasskeyName(name)) { setError(new Error(t('名称过长或包含不可用字符，请使用简短的名称。', 'Use a shorter name without control characters.'))); return; } if (await action('rename', () => patch(`/auth/passkeys/${encodeURIComponent(rename.id)}`, { name: name.trim() }), t('名称已更新。', 'Name updated.'))) setRename(null); }}><Field label={t('通行密钥名称', 'Passkey name')}><input required maxLength={100} value={name} onChange={event => setName(event.target.value)} autoFocus /></Field><ErrorNotice error={error} /><Button type="submit" variant="primary" busy={busy === 'rename'}>{t('保存名称', 'Save name')}</Button></form></Modal>}
    {deleting && <Modal title={t('移除此登录方式？', 'Remove this sign-in method?')} close={() => setDeleting(null)}><div className="stack"><p>{deleting.kind === 'passkey' ? deleting.value.name : deleting.value.email}</p><p className="muted">{t('移除后，这个凭据将无法登录。请确认仍有其他可用的登录方式。', 'This credential will no longer sign you in. Make sure another sign-in method is available.')}</p><ErrorNotice error={error} /><div className="dialog-actions"><Button onClick={() => setDeleting(null)}>{t('取消', 'Cancel')}</Button><Button variant="danger" busy={busy.startsWith('delete:')} onClick={() => void deleteCredential()}>{t('确认移除', 'Remove sign-in method')}</Button></div></div></Modal>}
    {emailOpen && <Modal title={t('绑定验证邮箱', 'Verify an email address')} close={() => setEmailOpen(false)}><form className="stack" onSubmit={changeEmail}><p className="muted">{t('新地址验证成功前，已有邮箱继续有效。请在当前浏览器中打开验证邮件。', 'Your current email stays active until verification succeeds. Open the verification email in this browser.')}</p><Field label={t('新邮箱地址', 'New email address')}><input type="email" autoComplete="email" required value={email} onChange={event => setEmail(event.target.value)} maxLength={254} autoFocus /></Field><ErrorNotice error={error} /><Button variant="primary" busy={busy === 'email'} type="submit">{t('发送验证邮件', 'Send verification email')}<ArrowRight size={16} /></Button></form></Modal>}
    {passwordOpen && <Modal title={t('修改密码', 'Change password')} close={() => setPasswordOpen(false)}><form className="stack" onSubmit={changePassword}><Field label={t('当前密码', 'Current password')}><input type="password" autoComplete="current-password" required value={currentPassword} onChange={event => setCurrentPassword(event.target.value)} /></Field><Field label={t('新密码', 'New password')} hint={t('至少 12 个字符。', 'At least 12 characters.')}><input type="password" autoComplete="new-password" required minLength={12} maxLength={1024} value={newPassword} onChange={event => setNewPassword(event.target.value)} /></Field><ErrorNotice error={error} /><Button type="submit" variant="primary" busy={busy === 'password'}>{t('保存并重新登录', 'Save and sign in again')}</Button></form></Modal>}
  </section>;
}

function PasskeyEditor({ replace, close, saved, authorize }: { replace?: Passkey; close: () => void; saved: () => void; authorize: (action: SecureAction) => void }) {
  const { t } = useText(); const [name, setName] = useState(''); const [authenticator, setAuthenticator] = useState('any'); const [busy, setBusy] = useState(false); const [saving, setSaving] = useState(false); const [error, setError] = useState<unknown>(null);
  const controller = useRef<AbortController | null>(null); const generation = useRef(0);
  useLayoutEffect(() => () => { generation.current++; controller.current?.abort(); }, []);
  function submit(event: React.FormEvent) {
    event.preventDefault(); if (busy) return;
    if (!validPasskeyName(name)) { setError(new Error(t('名称过长或包含不可用字符，请使用简短的名称。', 'Use a shorter name without control characters.'))); return; }
    authorize(register);
  }
  async function register() {
    setBusy(true); setError(null); const id = ++generation.current;
    const operation = new AbortController(); controller.current = operation;
    const active = () => generation.current === id && !operation.signal.aborted;
    let submitted = false;
    try {
      const challenge = await api<{ challenge_id: string; publicKey: CreationJSON }>('/auth/passkeys/register/options', { method: 'POST', body: JSON.stringify({ name: name.trim(), authenticator, ...(replace ? { replace_id: replace.id } : {}) }), signal: operation.signal });
      if (!active()) return;
      const hint: PasskeyHint = authenticator === 'security-key' ? 'security-key' : authenticator === 'platform' ? 'client-device' : 'any';
      const credential = await registerCredential(challenge.publicKey, hint, operation.signal);
      if (!active()) return;
      submitted = true; setSaving(true);
      await api('/auth/passkeys/register/finish', { method: 'POST', body: JSON.stringify({ challenge_id: challenge.challenge_id, credential }), signal: operation.signal });
      if (active()) saved();
    } catch (err) {
      if (active()) setError(submitted && err instanceof TypeError ? new Error(t('网络中断，保存结果尚未确认。请关闭并刷新凭据列表后再继续。', 'The network interrupted confirmation. Close this dialog and refresh the credential list before continuing.')) : authenticationError(err, t));
    } finally { if (active()) { setBusy(false); setSaving(false); } }
  }
  return <Modal title={replace ? t('替换通行密钥', 'Replace passkey') : t('添加通行密钥', 'Add passkey')} closeDisabled={saving} close={() => { controller.current?.abort(); close(); }}><form className="stack" onSubmit={submit}>
    <p className="muted">{replace ? t(`新凭据注册成功后，才会停用“${replace.name}”。在浏览器验证阶段取消会保留旧凭据；提交保存后请等待结果。`, `“${replace.name}” is removed only after the replacement is registered. Cancelling during browser verification keeps it usable. Once saving begins, wait for the result.`) : t('给凭据起个容易辨认的名字。浏览器会让你选择安全密钥、设备或密码管理器。', 'Choose a recognizable name. Your browser will ask for a security key, device or password manager.')}</p>
    <Field label={t('通行密钥名称', 'Passkey name')}><input required maxLength={100} autoFocus value={name} onChange={event => setName(event.target.value)} placeholder={t('例如：日常安全密钥', 'For example: everyday security key')} /></Field>
    <Field label={t('注册方式', 'Registration method')}><select value={authenticator} onChange={event => setAuthenticator(event.target.value)} disabled={busy}><option value="any">{t('由浏览器选择', 'Let the browser choose')}</option><option value="security-key">{t('USB / NFC 安全密钥', 'USB / NFC security key')}</option><option value="platform">{t('此设备或同步的通行密钥', 'This device or a synced passkey')}</option><option value="google-password-manager">{t('Android Google 密码管理器兼容模式', 'Android Google Password Manager compatibility')}</option></select></Field>
    {authenticator === 'security-key' && <p className="small muted">{t('安全密钥需要支持 PIN 或生物识别等用户验证方式。', 'Your security key must support user verification, such as a PIN or biometrics.')}</p>}
    <p className="small muted">{t('同步能力由认证器决定；选择“此设备”不保证凭据已备份到云端。', 'Synchronization depends on the authenticator. Selecting this device does not guarantee a cloud backup.')}</p>{saving && <p className="inline-note" role="status">{t('正在保存凭据，请等待确认结果。', 'Saving your credential. Wait for confirmation.')}</p>}<ErrorNotice error={error} /><Button variant="primary" type="submit" busy={busy}><Fingerprint size={18} />{t('继续注册', 'Continue registration')}</Button>
  </form></Modal>;
}

export function EmailVerification({ user, token, done, cancel }: { user: User; token: string; done: () => void; cancel: () => void }) {
  const { t } = useText(); const [busy, setBusy] = useState(false); const [error, setError] = useState<unknown>(null);
  const controller = useRef<AbortController | null>(null);
  useLayoutEffect(() => () => controller.current?.abort(), [token]);
  async function confirm() {
    if (busy) return; setBusy(true); setError(null); const operation = new AbortController(); controller.current = operation;
    try { await api('/auth/email/verify', { method: 'POST', body: JSON.stringify({ token }), signal: operation.signal }); if (!operation.signal.aborted) done(); }
    catch (err) { if (!operation.signal.aborted) setError(err); } finally { if (!operation.signal.aborted) setBusy(false); }
  }
  return <section className="verification-panel"><div><Mail size={24} /><h2>{t('确认验证邮箱', 'Confirm your email')}</h2><p>{t(`这次验证将更新账号 ${user.username} 的邮箱。仅在你本人发起此请求时确认。`, `This will update the email for ${user.username}. Confirm only if you requested this change.`)}</p><p className="small muted">{t('请保持发送验证邮件时的浏览器登录状态。链接不会自动执行验证。', 'Use the same browser session that requested the email. Opening the link does not verify it automatically.')}</p></div><ErrorNotice error={error} /><div className="row wrap"><Button variant="primary" busy={busy} onClick={() => void confirm()}>{t('确认验证', 'Confirm verification')}</Button><Button onClick={cancel}>{t('暂不验证', 'Not now')}</Button></div></section>;
}

export function GoogleMark() {
  return <svg className="google-mark" viewBox="0 0 24 24" width="20" height="20" aria-hidden="true"><path fill="currentColor" d="M21.6 12.2c0-.7-.1-1.4-.2-2.1H12v4h5.4a4.6 4.6 0 0 1-2 3v2.5h3.3c1.9-1.8 2.9-4.3 2.9-7.4ZM12 22c2.7 0 5-.9 6.7-2.4L15.4 17c-.9.6-2 1-3.4 1-2.6 0-4.9-1.8-5.7-4.2H2.9v2.6A10 10 0 0 0 12 22ZM6.3 13.8a6 6 0 0 1 0-3.6V7.6H2.9a10 10 0 0 0 0 8.8l3.4-2.6ZM12 6c1.5 0 2.8.5 3.8 1.5l2.9-2.9A9.7 9.7 0 0 0 12 2a10 10 0 0 0-9.1 5.6l3.4 2.6A6 6 0 0 1 12 6Z" /></svg>;
}
