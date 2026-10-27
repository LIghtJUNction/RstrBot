import { lazy, Suspense, useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { ArrowRight, Check, Database, ShieldCheck } from 'lucide-react';
import { post } from './api';
import type { Setup } from './api';
import { Brand, Button, ErrorNotice, Field, Status, useText } from './ui';

const Sculpture = lazy(() => import('./Sculpture'));

export default function Auth({ setup }: { setup: Setup }) {
  const { t } = useText(); const client = useQueryClient();
  const [username, setUsername] = useState(''); const [password, setPassword] = useState('');
  const [workspace, setWorkspace] = useState(''); const [token, setToken] = useState('');
  const [busy, setBusy] = useState(false); const [error, setError] = useState<unknown>(null);
  const [database, setDatabase] = useState(''); const [dbAction, setDbAction] = useState(''); const [dbMessage, setDbMessage] = useState('');
  const first = !setup.initialized;
  const currentDatabase = setup.database?.toLowerCase().includes('postgres') ? 'postgres' : 'sqlite';
  const [databaseChoice, setDatabaseChoice] = useState(currentDatabase);
  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (dbAction || first && databaseChoice !== currentDatabase) return;
    setError(null); setBusy(true);
    try {
      await post(first ? '/setup' : '/auth/login', first ? { token, username, password, workspace: workspace || username } : { username, password });
      setPassword(''); setToken('');
      // An expired session may be replaced by a different account in this tab.
      // No workspace-only query cache may survive that identity boundary.
      await client.cancelQueries(); client.clear(); window.location.reload();
    } catch (e) { setError(e); } finally { setBusy(false); }
  }
  async function configureDatabase(apply: boolean) {
    setDbAction(apply ? 'apply' : 'test'); setError(null); setDbMessage('');
    try {
      const result = await post<{ can_select?: boolean }>(`/setup/${apply ? 'database' : 'test-database'}`, { token, database_url: database });
      setDbMessage(apply ? t('数据库已切换，可以完成设置。', 'Database selected. You can complete setup.') : result.can_select === false ? t('可以连接，但此目标已有数据，不能用于首次初始化。请保留原数据并选择空数据库。', 'Connected, but this target already contains data and cannot be initialized. Preserve it and choose an empty database.') : t('只读连接测试通过，可以选择此数据库。', 'Read-only connection test passed. This database can be selected.'));
      if (apply) await client.invalidateQueries({ queryKey: ['setup'] });
    } catch (e) { setError(e); } finally { setDbAction(''); }
  }
  return <main className="auth-page"><section className="auth-story"><Brand /><div className="auth-copy"><p className="eyebrow">A LITTLE ORDER. A LOT OF POSSIBILITY.</p><h1>{t('把想法接起来。', 'Bring it all together.')}<br /><span>{t('让事情向前走。', 'Make room for what’s next.')}</span></h1><p>{t('你的对话、模型、工具和资料，在一个安静、清楚、由你掌控的工作区里。', 'Your conversations, models, tools and knowledge. One quiet workspace, with you in control.')}</p></div><Suspense fallback={<div className="sculpture-static" aria-hidden="true">↗</div>}><Sculpture /></Suspense><p className="auth-foot">RstrBot / {t('为个人与团队而造', 'For you. And the people you work with.')}</p></section>
    <section className="auth-panel"><div className="auth-form-wrap"><p className="eyebrow">{first ? 'START SOMETHING GOOD' : 'YOUR WORKSPACE AWAITS'}</p><h2>{first ? t('建立你的工作区', 'Create your workspace') : t('欢迎回来', 'Welcome back')}</h2><p className="muted">{first ? t('先建立管理员账户，再接入你的模型与工具。', 'Create the administrator account, then connect your models and tools.') : t('登录，接着做你正在做的事。', 'Sign in and pick up where you left off.')}</p>
      <form onSubmit={submit} className="stack"><ErrorNotice error={error} />
        {first && <Field label={t('初始化凭据', 'Setup token')} hint={t('从启动 RstrBot 的终端取得，只能使用一次。', 'Get this one-time token from the terminal running RstrBot.')}><input required value={token} onChange={e => setToken(e.target.value)} type="password" autoComplete="off" /></Field>}
        <Field label={t('用户名', 'Username')}><input required name="username" autoComplete="username" value={username} onChange={e => setUsername(e.target.value)} maxLength={80} autoFocus={!first} /></Field>
        <Field label={t('密码', 'Password')} hint={first ? t('至少 12 个字符。', 'At least 12 characters.') : undefined}><input required name="password" type="password" autoComplete={first ? 'new-password' : 'current-password'} minLength={first ? 12 : undefined} value={password} onChange={e => setPassword(e.target.value)} maxLength={1024} /></Field>
        {first && <><Field label={t('工作区名称', 'Workspace name')}><input required value={workspace} onChange={e => setWorkspace(e.target.value)} maxLength={120} placeholder={t('我的工作区', 'My workspace')} /></Field><section className="setup-database" aria-label={t('数据库与可选服务', 'Database & optional services')}><h3><Database size={16} />{t('数据库与可选服务', 'Database & optional services')}</h3><fieldset className="database-options"><legend>{t('数据库类型', 'Database type')}</legend>{[['sqlite', 'SQLite', t('个人与单实例', 'Personal / single instance')], ['postgres', 'PostgreSQL', t('团队服务', 'Shared server')]].map(([id, label, description]) => <label key={id}><input type="radio" name="database-type" value={id} checked={databaseChoice === id} onChange={() => { setDatabaseChoice(id); setDatabase(''); setDbMessage(''); }} disabled={busy || !!dbAction} /><span><strong>{label}</strong><small>{description}</small></span></label>)}</fieldset><p>{t('当前数据库：', 'Current database: ')}<strong>{setup.database ?? 'SQLite'}</strong></p><p className="small muted">{t('SQLite 适合个人与单实例；PostgreSQL 适合团队服务。Valkey 可选，当前用于在线状态缓存；关闭后基础功能仍可使用。', 'SQLite suits personal and single-instance use; PostgreSQL suits shared servers. Optional Valkey currently caches presence; core features work without it.')}</p><Field label={t('更换数据库连接', 'Select a database connection')} hint={t('已有 SQLite 文件或空 PostgreSQL 数据库。新的 SQLite 文件请在启动命令中选择。', 'Use an existing SQLite file or an empty PostgreSQL database. Select a new SQLite file with the startup command.')}><input type="password" autoComplete="off" value={database} onChange={e => setDatabase(e.target.value)} placeholder={databaseChoice === 'postgres' ? 'postgres://user:password@localhost/rstrbot' : 'sqlite:///path/to/existing.db'} /></Field><div className="row wrap"><Button type="button" disabled={!token || !database || busy || !!dbAction} busy={dbAction === 'test'} onClick={() => configureDatabase(false)}>{t('测试连接', 'Test connection')}</Button><Button type="button" disabled={!token || !database || busy || !!dbAction} busy={dbAction === 'apply'} onClick={() => configureDatabase(true)}>{t('使用此数据库', 'Use database')}</Button></div>{dbMessage && <p className="success" role="status"><Check size={16} />{dbMessage}</p>}<p className="small muted">Valkey: {setup.valkey ? <Status state={setup.valkey_status ?? 'connecting'} /> : t('关闭', 'Off')}</p>{databaseChoice !== currentDatabase && <p className="connection-warning" role="status">{t('请先输入连接地址并选择“使用此数据库”，再建立工作区。', 'Enter a connection and select “Use database” before creating the workspace.')}</p>}</section></>}
        <Button type="submit" variant="primary" busy={busy} disabled={!!dbAction || first && databaseChoice !== currentDatabase}>{first ? t('建立工作区', 'Create workspace') : t('登录', 'Sign in')}<ArrowRight size={18} /></Button>
      </form><p className="auth-security"><ShieldCheck size={16} />{t('你的私有会话，仅对你开放。', 'Your private conversations stay private.')}</p></div></section>
  </main>;
}
