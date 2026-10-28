import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { ArrowLeft, ArrowRight, BookOpen, Check, Database, Fingerprint, LockKeyhole, Mail, MessageSquare, ShieldCheck, Workflow } from 'lucide-react';
import { api, post } from './api';
import type { Setup } from './api';
import { authenticateCredential, passkeysSupported } from './passkey';
import type { RequestJSON } from './passkey';
import { authenticationError, startGoogle } from './security-api';
import type { AuthOptions } from './security-api';
import { GoogleMark } from './Security';
import { Brand, Button, ErrorNotice, Field, Modal, Status, useText } from './ui';

export default function Auth({ setup, verifyingEmail = false }: { setup: Setup; verifyingEmail?: boolean }) {
  const { t } = useText(); const client = useQueryClient(); const first = !setup.initialized;
  const options = useQuery({ queryKey: ['auth-options'], enabled: !first, queryFn: ({ signal }) => api<AuthOptions>('/auth/options', { signal }) });
  const [username, setUsername] = useState(''); const [password, setPassword] = useState('');
  const [workspace, setWorkspace] = useState(''); const [token, setToken] = useState('');
  const [busy, setBusy] = useState(''); const [error, setError] = useState<unknown>(() => new URLSearchParams(location.search).get('auth') === 'failed' ? new Error(t('Google 登录未完成，请重试或选择其他登录方式。', 'Google sign-in did not complete. Try again or choose another sign-in method.')) : null);
  const [forgot, setForgot] = useState(false); const controller = useRef<AbortController | null>(null); const usernameInput = useRef<HTMLInputElement>(null);
  const [database, setDatabase] = useState(''); const [dbAction, setDbAction] = useState(''); const [dbMessage, setDbMessage] = useState('');
  const currentDatabase = setup.database?.toLowerCase().includes('postgres') ? 'postgres' : 'sqlite';
  const [databaseChoice, setDatabaseChoice] = useState(currentDatabase);
  useLayoutEffect(() => () => controller.current?.abort(), []);
  async function signedIn() {
    setPassword(''); setToken('');
    // An identity change must discard every prior account's cached view.
    await client.cancelQueries(); client.clear();
    const url = new URL(location.href); url.searchParams.delete('auth'); if (verifyingEmail) url.hash = 'account'; window.location.assign(url.pathname + url.search + url.hash);
  }
  async function submit(event: React.FormEvent) {
    event.preventDefault(); if (busy || dbAction || first && databaseChoice !== currentDatabase) return;
    setError(null); setBusy('password'); const operation = new AbortController(); controller.current = operation;
    try { await api(first ? '/setup' : '/auth/login', { method: 'POST', body: JSON.stringify(first ? { token, username, password, workspace: workspace || username } : { username: username.trim(), password }), signal: operation.signal }); if (!operation.signal.aborted) await signedIn(); }
    catch (err) { setError(err); } finally { setBusy(''); }
  }
  async function signInPasskey() {
    if (busy) return;
    if (!username.trim()) { usernameInput.current?.focus(); setError(new Error(t('先输入用户名或已验证邮箱，再使用通行密钥。', 'Enter your username or verified email before using a passkey.'))); return; }
    setBusy('passkey'); setError(null); controller.current = new AbortController();
    try {
      const challenge = await post<{ challenge_id: string; publicKey: RequestJSON }>('/auth/passkeys/login/options', { username: username.trim() });
      const credential = await authenticateCredential(challenge.publicKey, 'any', controller.current.signal);
      await api('/auth/passkeys/login/finish', { method: 'POST', body: JSON.stringify({ challenge_id: challenge.challenge_id, credential }), signal: controller.current.signal }); if (!controller.current.signal.aborted) await signedIn();
    } catch (err) { setError(authenticationError(err, t)); } finally { setBusy(''); }
  }
  async function google() { setBusy('google'); setError(null); controller.current = new AbortController(); try { await startGoogle('login', 'account', controller.current.signal); } catch (err) { setError(err); setBusy(''); } }
  async function configureDatabase(apply: boolean) {
    setDbAction(apply ? 'apply' : 'test'); setError(null); setDbMessage('');
    try {
      const result = await post<{ can_select?: boolean }>(`/setup/${apply ? 'database' : 'test-database'}`, { token, database_url: database });
      setDbMessage(apply ? t('数据库已切换，可以完成设置。', 'Database selected. You can complete setup.') : result.can_select === false ? t('可以连接，但此目标已有数据，不能用于首次初始化。请保留原数据并选择空数据库。', 'Connected, but this target already contains data and cannot be initialized. Preserve it and choose an empty database.') : t('只读连接测试通过，可以选择此数据库。', 'Read-only connection test passed. This database can be selected.'));
      if (apply) await client.invalidateQueries({ queryKey: ['setup'] });
    } catch (err) { setError(err); } finally { setDbAction(''); }
  }
  return <main className={`auth-page ${first ? 'auth-setup' : ''}`}><AuthIdentity />
    <section className="auth-panel"><div className="auth-form-wrap"><p className="auth-kicker">{first ? t('首次设置', 'INITIAL SETUP') : t('你的工作区', 'YOUR WORKSPACE')}</p><h2>{first ? t('建立你的工作区', 'Create your workspace') : t('欢迎回来', 'Welcome back')}</h2><p className="auth-description">{first ? t('建立管理员账号，选择数据库，然后接入模型与工具。', 'Create an administrator account, select your database, then connect models and tools.') : t('选择一种方式登录，继续你的工作。', 'Choose how to sign in and continue your work.')}</p>
      {verifyingEmail && <div className="notice"><Mail size={19} /><p>{t('此链接需要发起请求时的原始登录会话。请登录账号后重新发送验证邮件。', 'This link requires the original session that requested it. Sign in, then request a new verification email.')}</p></div>}
      <ErrorNotice error={error} />
      {!first && options.data?.google_enabled && <><Button className="auth-provider" disabled={!!busy} busy={busy === 'google'} onClick={() => void google()}><GoogleMark />{t('使用 Google 登录', 'Continue with Google')}</Button><div className="auth-divider"><span>{t('或使用账号登录', 'or use your account')}</span></div></>}
      <form onSubmit={submit} className="stack auth-credentials">
        {first && <Field label={t('初始化凭据', 'Setup token')} hint={t('从启动 RstrBot 的终端取得，只能使用一次。', 'Get this one-time token from the terminal running RstrBot.')}><input required value={token} onChange={event => setToken(event.target.value)} type="password" autoComplete="off" /></Field>}
        <Field label={first ? t('用户名', 'Username') : t('用户名或已验证邮箱', 'Username or verified email')}><input ref={usernameInput} required name="username" autoComplete="username" autoCapitalize="none" spellCheck={false} value={username} onChange={event => setUsername(event.target.value)} maxLength={first ? 80 : 254} autoFocus={!first} /></Field>
        {!first && options.data?.passkey_enabled && <div className="passkey-choice"><Button type="button" className="auth-provider" onClick={() => void signInPasskey()} disabled={!!busy || !passkeysSupported()} busy={busy === 'passkey'}><Fingerprint size={20} />{t('使用通行密钥登录', 'Sign in with a passkey')}</Button><p>{passkeysSupported() ? t('安全密钥、此设备或密码管理器', 'Security key, this device or password manager') : t('请通过 HTTPS 或 localhost 使用兼容的浏览器。', 'Use a compatible browser over HTTPS or localhost.')}</p><div className="auth-divider"><span>{t('或使用密码', 'or use a password')}</span></div></div>}
        <Field label={t('密码', 'Password')} hint={first ? t('至少 12 个字符。', 'At least 12 characters.') : undefined}><input required name="password" type="password" autoComplete={first ? 'new-password' : 'current-password'} minLength={first ? 12 : undefined} value={password} onChange={event => setPassword(event.target.value)} maxLength={1024} /></Field>
        {first && <><Field label={t('工作区名称', 'Workspace name')}><input required value={workspace} onChange={e => setWorkspace(e.target.value)} maxLength={120} placeholder={t('我的工作区', 'My workspace')} /></Field><section className="setup-database" aria-label={t('数据库与可选服务', 'Database & optional services')}><h3><Database size={16} />{t('数据库与可选服务', 'Database & optional services')}</h3><fieldset className="database-options"><legend>{t('数据库类型', 'Database type')}</legend>{[['sqlite', 'SQLite', t('个人与单实例', 'Personal / single instance')], ['postgres', 'PostgreSQL', t('团队服务', 'Shared server')]].map(([id, label, description]) => <label key={id}><input type="radio" name="database-type" value={id} checked={databaseChoice === id} onChange={() => { setDatabaseChoice(id); setDatabase(''); setDbMessage(''); }} disabled={!!busy || !!dbAction} /><span><strong>{label}</strong><small>{description}</small></span></label>)}</fieldset><p>{t('当前数据库：', 'Current database: ')}<strong>{setup.database ?? 'SQLite'}</strong></p><p className="small muted">{t('SQLite 适合个人与单实例；PostgreSQL 适合团队服务。Valkey 可选，当前用于在线状态缓存；关闭后基础功能仍可使用。', 'SQLite suits personal and single-instance use; PostgreSQL suits shared servers. Optional Valkey currently caches presence; core features work without it.')}</p><Field label={t('更换数据库连接', 'Select a database connection')} hint={t('已有 SQLite 文件或空 PostgreSQL 数据库。新的 SQLite 文件请在启动命令中选择。', 'Use an existing SQLite file or an empty PostgreSQL database. Select a new SQLite file with the startup command.')}><input type="password" autoComplete="off" value={database} onChange={e => setDatabase(e.target.value)} placeholder={databaseChoice === 'postgres' ? 'postgres://user:password@localhost/rstrbot' : 'sqlite:///path/to/existing.db'} /></Field><div className="row wrap"><Button type="button" disabled={!token || !database || !!busy || !!dbAction} busy={dbAction === 'test'} onClick={() => configureDatabase(false)}>{t('测试连接', 'Test connection')}</Button><Button type="button" disabled={!token || !database || !!busy || !!dbAction} busy={dbAction === 'apply'} onClick={() => configureDatabase(true)}>{t('使用此数据库', 'Use database')}</Button></div>{dbMessage && <p className="success" role="status"><Check size={16} />{dbMessage}</p>}<p className="small muted">Valkey: {setup.valkey ? <Status state={setup.valkey_status ?? 'connecting'} /> : t('关闭', 'Off')}</p>{databaseChoice !== currentDatabase && <p className="connection-warning" role="status">{t('请先输入连接地址并选择“使用此数据库”，再建立工作区。', 'Enter a connection and select “Use database” before creating the workspace.')}</p>}</section></>}
        <Button type="submit" variant="primary" busy={busy === 'password'} disabled={!!busy || !!dbAction || first && databaseChoice !== currentDatabase}>{first ? t('建立工作区', 'Create workspace') : t('登录', 'Sign in')}<ArrowRight size={18} /></Button>
      </form>
      {!first && options.data?.email_enabled && <Button variant="text forgot-password" onClick={() => setForgot(true)}>{t('忘记密码？', 'Forgot password?')}</Button>}
      <p className="auth-security"><ShieldCheck size={16} />{t('由你的 RstrBot 实例管理账号与访问权限', 'Accounts and access are managed by your RstrBot instance')}</p>
    </div></section>
    {forgot && <RecoveryRequest close={() => setForgot(false)} />}
  </main>;
}

function AuthIdentity() {
  const { t } = useText();
  return <section className="auth-story" aria-label="RstrBot workspace"><Brand /><div className="auth-copy"><p className="auth-kicker">{t('自托管 AI 工作区', 'SELF-HOSTED AI WORKSPACE')}</p><h1>{t('专注工作，', 'Your work,')}<br /><span>{t('从这里开始。', 'all in one place.')}</span></h1><p>{t('连接你的模型、知识与工具。在自己的基础设施上，让想法成为行动。', 'Connect your models, knowledge and tools. Turn ideas into action on your own infrastructure.')}</p></div><div className="auth-features"><div><MessageSquare size={19} /><span>{t('对话', 'Conversations')}</span><small>{t('保持上下文', 'Keep the context')}</small></div><div><BookOpen size={19} /><span>{t('知识', 'Knowledge')}</span><small>{t('随时找到资料', 'Bring your sources')}</small></div><div><Workflow size={19} /><span>{t('自动化', 'Automation')}</span><small>{t('安排下一步', 'Move work forward')}</small></div></div><footer className="auth-foot"><span>RstrBot</span><span>{t('你的实例 · 你的工作', 'Your instance. Your workspace.')}</span></footer></section>;
}

function RecoveryRequest({ close }: { close: () => void }) {
  const { t } = useText(); const [email, setEmail] = useState(''); const [sent, setSent] = useState(false); const [busy, setBusy] = useState(false); const [error, setError] = useState<unknown>(null);
  async function submit(event: React.FormEvent) { event.preventDefault(); setBusy(true); setError(null); try { await post('/auth/password/reset/request', { email: email.trim() }); setSent(true); } catch (err) { setError(err); } finally { setBusy(false); } }
  return <Modal title={t('找回密码', 'Reset your password')} close={close}>{sent ? <div className="stack"><div className="recovery-icon"><Mail size={25} /></div><p role="status">{t('如果这个邮箱关联了可找回的账号，我们会发送重置链接。请检查收件箱与垃圾邮件。', 'If this email belongs to an account that can be recovered, a reset link will be sent. Check your inbox and spam folder.')}</p><Button variant="primary" onClick={close}>{t('返回登录', 'Back to sign in')}</Button></div> : <form className="stack" onSubmit={submit}><p className="muted">{t('输入已绑定并验证的邮箱地址。', 'Enter the verified email address linked to your account.')}</p><Field label={t('邮箱地址', 'Email address')}><input type="email" autoComplete="email" required autoFocus value={email} onChange={event => setEmail(event.target.value)} maxLength={254} /></Field><ErrorNotice error={error} /><Button type="submit" variant="primary" busy={busy}>{t('发送重置链接', 'Send reset link')}<ArrowRight size={16} /></Button></form>}</Modal>;
}

export function ResetPassword({ token }: { token: string }) {
  const { t } = useText(); const client = useQueryClient(); const [password, setPassword] = useState(''); const [confirm, setConfirm] = useState(''); const [done, setDone] = useState(false); const [busy, setBusy] = useState(false); const [error, setError] = useState<unknown>(null);
  const controller = useRef<AbortController | null>(null);
  useLayoutEffect(() => () => controller.current?.abort(), [token]);
  async function submit(event: React.FormEvent) {
    event.preventDefault(); if (busy) return; setError(null);
    if (password !== confirm) { setError(new Error(t('两次输入的密码不一致。', 'The passwords do not match.'))); return; }
    setBusy(true); const operation = new AbortController(); controller.current = operation;
    try { await api('/auth/password/reset/finish', { method: 'POST', body: JSON.stringify({ token, new_password: password }), signal: operation.signal }); if (operation.signal.aborted) return; setPassword(''); setConfirm(''); await client.cancelQueries(); if (operation.signal.aborted) return; client.clear(); history.replaceState(null, '', location.pathname); setDone(true); }
    catch (err) { if (!operation.signal.aborted) setError(err); } finally { if (!operation.signal.aborted) setBusy(false); }
  }
  return <main className="auth-page"><AuthIdentity /><section className="auth-panel"><div className="auth-form-wrap"><p className="auth-kicker">{t('账号恢复', 'ACCOUNT RECOVERY')}</p><div className="recovery-icon"><LockKeyhole size={27} /></div><h2>{done ? t('密码已更新', 'Password updated') : t('设置新密码', 'Choose a new password')}</h2><p className="auth-description">{done ? t('所有设备上的旧会话已退出。请使用新密码登录。', 'Previous sessions on all devices are signed out. Sign in with your new password.') : t('使用新的强密码恢复账号。重置后需要在所有设备上重新登录。', 'Recover access with a new strong password. You will need to sign in again on every device.')}</p>{done ? <Button variant="primary" className="auth-provider" onClick={() => window.location.assign('/')}>{t('返回登录', 'Back to sign in')}<ArrowRight size={17} /></Button> : <form className="stack" onSubmit={submit}><Field label={t('新密码', 'New password')} hint={t('至少 12 个字符。', 'At least 12 characters.')}><input required type="password" autoComplete="new-password" minLength={12} maxLength={1024} autoFocus value={password} onChange={event => setPassword(event.target.value)} /></Field><Field label={t('确认新密码', 'Confirm new password')}><input required type="password" autoComplete="new-password" minLength={12} maxLength={1024} value={confirm} onChange={event => setConfirm(event.target.value)} /></Field><ErrorNotice error={error} /><Button type="submit" variant="primary" busy={busy}>{t('保存新密码', 'Save new password')}<ArrowRight size={16} /></Button></form>}<Button variant="text" className="forgot-password" onClick={() => window.location.assign('/')}><ArrowLeft size={15} />{t('返回登录', 'Back to sign in')}</Button></div></section></main>;
}
