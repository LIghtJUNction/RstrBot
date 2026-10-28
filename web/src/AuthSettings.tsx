import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, CircleCheck, Fingerprint, Globe, Mail, Save, Send, ShieldCheck } from 'lucide-react';
import { api, post } from './api';
import type { User } from './api';
import { authorizedRead } from './data';
import type { AuthSettings } from './security-api';
import { GoogleMark, useRecentAuth } from './Security';
import { Button, ErrorNotice, Field, Loading, PageHeader, useText } from './ui';

export default function AuthenticationSettings({ user, back }: { user: User; back: () => void }) {
  const { t } = useText(); const client = useQueryClient(); const [formRevision, setFormRevision] = useState(0); const key = ['auth-settings', user.id];
  const query = useQuery<AuthSettings | null>({ queryKey: key, queryFn: ({ signal }) => authorizedRead(client, key, signal, () => api<AuthSettings>('/admin/auth-settings', { signal })) });
  const reload = async () => { const latest = await query.refetch(); if (latest.data) setFormRevision(version => version + 1); };
  return <section className="page security-page"><Button variant="text back-button" onClick={back}><ArrowLeft size={16} />{t('账号与安全', 'Account & security')}</Button><PageHeader title={t('认证与邮件设置', 'Authentication & email')} description={t('配置此实例的登录入口和邮件发送。更改对所有账号生效。', 'Configure sign-in and outgoing email for this instance. Changes affect every account.')} /><ErrorNotice error={query.error} onRetry={() => void query.refetch()} />{query.isPending ? <Loading /> : query.data && <SettingsForm key={formRevision} initial={query.data} user={user} reload={reload} reset={() => setFormRevision(version => version + 1)} />}</section>;
}

function SettingsForm({ initial: loaded, user, reload, reset }: { initial: AuthSettings; user: User; reload: () => Promise<void>; reset: () => void }) {
  // Keep the version that produced this draft. A focus refetch must never
  // erase typed secrets or silently advance the optimistic concurrency token.
  const [initial] = useState(loaded);
  const { t } = useText(); const client = useQueryClient(); const recent = useRecentAuth(user);
  const [origin, setOrigin] = useState(initial.public_origin ?? window.location.origin); const [rpId, setRpId] = useState(initial.rp_id ?? window.location.hostname);
  const [googleEnabled, setGoogleEnabled] = useState(initial.google.enabled); const [clientId, setClientId] = useState(initial.google.client_id ?? ''); const [clientSecret, setClientSecret] = useState(''); const [clearClientSecret, setClearClientSecret] = useState(false);
  const [smtp, setSmtp] = useState(initial.smtp); const [password, setPassword] = useState(''); const [clearPassword, setClearPassword] = useState(false); const [testEmail, setTestEmail] = useState('');
  const [busy, setBusy] = useState(''); const [error, setError] = useState<unknown>(null); const [notice, setNotice] = useState(''); const [dirty, setDirty] = useState(false);
  const change = <T,>(setter: (value: T) => void, value: T) => { setter(value); setDirty(true); setNotice(''); };
  const mail = <K extends keyof AuthSettings['smtp']>(key: K, value: AuthSettings['smtp'][K]) => change(setSmtp, { ...smtp, [key]: value });
  async function save() {
    setBusy('save'); setError(null); setNotice('');
    const payload = {
      revision: initial.revision, public_origin: origin.trim() || null, rp_id: rpId.trim() || null,
      google: { enabled: googleEnabled, client_id: clientId.trim(), ...(clientSecret ? { client_secret: clientSecret } : {}), clear_client_secret: clearClientSecret },
      smtp: { enabled: smtp.enabled, host: smtp.host.trim(), port: smtp.port, security: smtp.security, username: smtp.username, from_email: smtp.from_email.trim(), from_name: smtp.from_name, ...(password ? { password } : {}), clear_password: clearPassword },
    };
    try {
      const result = await api<AuthSettings>('/admin/auth-settings', { method: 'PUT', body: JSON.stringify(payload) });
      setClientSecret(''); setPassword(''); setDirty(false);
      client.setQueryData<AuthSettings | null>(['auth-settings', user.id], previous => previous === null ? null : result);
      await client.invalidateQueries({ queryKey: ['auth-options'] });
      // Only our completed save or an explicit reload replaces this draft.
      reset();
    } catch (err) { setError(err); } finally { setBusy(''); }
  }
  async function test() {
    setBusy('test'); setError(null); setNotice('');
    try { await post('/admin/auth-settings/test-email', { email: testEmail.trim() }); setNotice(t('测试邮件已提交给邮件服务器，请检查收件箱。', 'The mail server accepted the test message. Check the destination inbox.')); }
    catch (err) { setError(err); } finally { setBusy(''); }
  }
  return <><form onSubmit={event => { event.preventDefault(); recent.run(save); }} className="auth-settings-form"><fieldset className="settings-fieldset" disabled={!!busy}><div className="instance-banner"><ShieldCheck size={19} /><div><strong>{t('实例管理员设置', 'Instance administrator settings')}</strong><p>{t('密钥仅写入，不会从服务端回显。留空保留已保存密钥。', 'Secrets are write-only. Leave a secret field blank to keep its saved value.')}</p></div><span className="pill">{t('配置版本', 'Revision')} {initial.revision}</span></div>
    {loaded.revision !== initial.revision && <div className="notice" role="status"><p>{t('服务端配置已经更新。你的输入已保留；保存前请丢弃草稿并重新加载最新版本。', 'The server configuration changed. Your inputs are preserved. Discard this draft and reload the latest version before saving.')}</p></div>}
    <ErrorNotice error={error} />{notice && <div className="notice success" role="status"><CircleCheck size={18} />{notice}</div>}
    <section className="security-section"><div className="security-section-heading"><Globe size={21} /><div><h2>{t('服务地址与通行密钥', 'Server address & passkeys')}</h2><p>{t('使用用户实际访问的 HTTPS 地址。地址与域名决定通行密钥归属。', 'Use the HTTPS address people visit. The origin and domain determine where passkeys can be used.')}</p></div><Fingerprint size={20} /></div>
      <div className="settings-grid"><Field label={t('公开访问地址', 'Public origin')} hint="https://bot.example.com"><input type="url" required value={origin} onChange={event => change(setOrigin, event.target.value)} disabled={initial.passkey_count > 0} /></Field><Field label={t('通行密钥域名', 'Passkey RP ID')} hint={t('通常与公开地址的主机名一致，不包含协议或端口。', 'Usually the public hostname, without a scheme or port.')}><input required value={rpId} onChange={event => change(setRpId, event.target.value)} disabled={initial.passkey_count > 0} /></Field></div>
      {initial.passkey_count > 0 ? <p className="inline-note">{t(`已有 ${initial.passkey_count} 枚通行密钥，地址与域名已锁定。`, `${initial.passkey_count} passkeys are registered. Origin and RP ID are locked.`)}</p> : <p className="inline-note">{t('保存后才会启用。通行密钥需要域名：本机用 localhost，线上用 HTTPS 域名，不支持 IP 地址。', 'Initial values take effect after saving. Passkeys need a DNS hostname: use localhost locally or your HTTPS domain, not an IP address.')}</p>}
    </section>
    <section className="security-section"><div className="security-section-heading"><GoogleMark /><div><h2>{t('Google 登录', 'Google sign-in')}</h2><p>{t('使用你在 Google Cloud 建立的 OAuth 客户端。', 'Use the OAuth client you created in Google Cloud.')}</p></div><label className="switch-field"><input type="checkbox" checked={googleEnabled} onChange={event => change(setGoogleEnabled, event.target.checked)} /><span>{t('启用 Google 登录', 'Enable Google sign-in')}</span></label></div>
      <div className="settings-grid"><Field label="Google Client ID"><input value={clientId} onChange={event => change(setClientId, event.target.value)} required={googleEnabled} autoComplete="off" /></Field><Field label="Google Client secret" hint={initial.google.has_client_secret ? t('已保存。留空保留，填写新值可替换。', 'Saved. Leave blank to retain it or enter a replacement.') : t('尚未设置。', 'Not configured.')}><input type="password" autoComplete="new-password" value={clientSecret} disabled={clearClientSecret} onChange={event => change(setClientSecret, event.target.value)} /></Field></div>
      {initial.google.has_client_secret && <label className="check-field"><input type="checkbox" checked={clearClientSecret} onChange={event => { change(setClearClientSecret, event.target.checked); if (event.target.checked) setClientSecret(''); }} /><span>{t('清除已保存的 Google 密钥', 'Clear the saved Google secret')}</span></label>}
      <div className="copy-value"><span>{t('授权回调地址', 'Authorized redirect URI')}</span><code>{initial.google.redirect_uri ?? t('保存服务地址后生成', 'Generated after saving the public origin')}</code></div>
      <p className="security-footnote">{t('将上面的完整回调地址填入 Google OAuth 客户端。绑定以 Google 的账号标识为准，不按邮箱自动合并用户。', 'Add the exact redirect URI to your Google OAuth client. Accounts are linked by Google identity, never automatically by email.')}</p>
    </section>
    <section className="security-section"><div className="security-section-heading"><Mail size={21} /><div><h2>{t('邮件发送', 'Outgoing email')}</h2><p>{t('用于邮箱验证与密码找回。使用你的 SMTP 服务。', 'Deliver email verification and password recovery through your SMTP service.')}</p></div><label className="switch-field"><input type="checkbox" checked={smtp.enabled} onChange={event => mail('enabled', event.target.checked)} /><span>{t('启用邮件', 'Enable email')}</span></label></div>
      <div className="settings-grid"><Field label={t('SMTP 主机', 'SMTP host')}><input value={smtp.host} required={smtp.enabled} onChange={event => mail('host', event.target.value)} placeholder="smtp.example.com" /></Field><div className="settings-pair"><Field label={t('端口', 'Port')}><input type="number" min={1} max={65535} value={smtp.port} onChange={event => mail('port', Number(event.target.value))} required /></Field><Field label={t('连接安全', 'Connection security')}><select value={smtp.security} onChange={event => mail('security', event.target.value as AuthSettings['smtp']['security'])}><option value="starttls">STARTTLS</option><option value="tls">TLS</option><option value="local">{t('本机邮件服务', 'Local mail relay')}</option></select></Field></div><Field label={t('SMTP 用户名', 'SMTP username')}><input value={smtp.username} onChange={event => mail('username', event.target.value)} autoComplete="off" /></Field><Field label={t('SMTP 密码', 'SMTP password')} hint={initial.smtp.has_password ? t('已保存。更换目标主机、连接安全或用户名时须重新填写或明确清除。', 'Saved. Changing host, security or username requires a replacement or explicit clearing.') : t('不会向前端回显已保存的密码。', 'Saved passwords are never returned to the browser.')}><input type="password" value={password} disabled={clearPassword} onChange={event => change(setPassword, event.target.value)} autoComplete="new-password" /></Field><Field label={t('发件邮箱', 'Sender email')}><input type="email" value={smtp.from_email} required={smtp.enabled} onChange={event => mail('from_email', event.target.value)} /></Field><Field label={t('发件人名称', 'Sender name')}><input value={smtp.from_name} onChange={event => mail('from_name', event.target.value)} /></Field></div>
      {initial.smtp.has_password && <label className="check-field"><input type="checkbox" checked={clearPassword} onChange={event => { change(setClearPassword, event.target.checked); if (event.target.checked) setPassword(''); }} /><span>{t('清除已保存的 SMTP 密码', 'Clear the saved SMTP password')}</span></label>}
      {smtp.security === 'local' && <p className="inline-note">{t('本机模式仅允许 127.0.0.1 或 ::1，不用于远程 SMTP。', 'Local mode accepts only 127.0.0.1 or ::1, not a remote SMTP server.')}</p>}
    </section>
    <div className="settings-save"><p className="muted small">{dirty ? t('有尚未保存的更改。', 'You have unsaved changes.') : t('保存时将再次校验权限和配置版本。', 'Permissions and configuration revision are checked when saving.')}</p><div className="row wrap"><Button type="button" disabled={!!busy} onClick={() => void reload()}>{dirty ? t('丢弃草稿并重载', 'Discard draft & reload') : t('重载已保存设置', 'Reload saved settings')}</Button><Button type="submit" variant="primary" busy={busy === 'save'} disabled={!!busy}><Save size={17} />{t('保存设置', 'Save settings')}</Button></div></div>
  </fieldset></form>
  <section className="security-section"><div className="security-section-heading"><Send size={20} /><div><h2>{t('发送测试邮件', 'Send a test email')}</h2><p>{t('测试使用已保存的配置，会向填写的地址发送一封邮件。', 'Uses the saved configuration and sends one email to the address below.')}</p></div></div><form className="inline-form" onSubmit={event => { event.preventDefault(); recent.run(test); }}><Field label={t('测试收件邮箱', 'Test recipient email')}><input type="email" required disabled={!!busy} value={testEmail} onChange={event => setTestEmail(event.target.value)} /></Field><Button type="submit" busy={busy === 'test'} disabled={dirty || !initial.smtp.enabled || !!busy}>{t('发送测试邮件', 'Send test email')}</Button></form>{dirty && <p className="inline-note">{t('请先保存配置，再发送测试邮件。', 'Save your changes before sending the test email.')}</p>}</section>{recent.dialog}</>;
}
