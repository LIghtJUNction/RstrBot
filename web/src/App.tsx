import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { Activity, ArrowUpRight, BookOpen, Brain, CalendarClock, ChevronDown, Cpu, LogOut, Menu, MessageSquare, Moon, Plus, Plug, Send, Settings, Sun, Users, Workflow, X } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { api, ApiError, items, post } from './api';
import type { Setup, User, Workspace } from './api';
import Auth from './Auth';
import Chat from './Chat';
import Resources from './Resources';
import { Members, Runs, Settings as SettingsPage } from './Operations';
import { safePreference, savePreference } from './data';
import { Brand, Button, Empty, ErrorNotice, Field, LanguageContext, Loading, Modal, useText } from './ui';
import type { Language } from './ui';
import { roleAllows } from './permissions';

type Page = 'chat' | 'runs' | 'knowledge' | 'memories' | 'schedules' | 'agents' | 'routes' | 'models' | 'plugins' | 'members' | 'settings';
type NavItem = { page: Page; title: [string, string]; icon: LucideIcon; management?: boolean };
const navigation: NavItem[] = [
  { page: 'chat', title: ['对话', 'Conversations'], icon: MessageSquare },
  { page: 'runs', title: ['运行与收件箱', 'Runs & inbox'], icon: Activity },
  { page: 'knowledge', title: ['知识库', 'Knowledge'], icon: BookOpen },
  { page: 'memories', title: ['记忆', 'Memory'], icon: Brain },
  { page: 'schedules', title: ['计划任务', 'Schedules'], icon: CalendarClock },
  { page: 'agents', title: ['代理', 'Agents'], icon: Workflow },
  { page: 'routes', title: ['机器人与渠道', 'Channels'], icon: Send, management: true },
  { page: 'models', title: ['模型', 'Models'], icon: Cpu, management: true },
  { page: 'plugins', title: ['插件', 'Plugins'], icon: Plug, management: true },
  { page: 'members', title: ['成员', 'Members'], icon: Users, management: true },
  { page: 'settings', title: ['设置与审计', 'Settings & audit'], icon: Settings, management: true },
];

export default function App() {
  const [language, setLanguage] = useState<Language>(() => safePreference('language', navigator.language.startsWith('zh') ? 'zh' : 'en') as Language);
  const [theme, setTheme] = useState(() => safePreference('theme', 'dark'));
  useEffect(() => { document.documentElement.dataset.theme = theme; savePreference('theme', theme); }, [theme]);
  useEffect(() => { document.documentElement.lang = language === 'zh' ? 'zh-CN' : 'en'; savePreference('language', language); }, [language]);
  return <LanguageContext value={language}><div className="preferences"><Button variant="ghost icon" aria-label={theme === 'dark' ? 'Light theme / 浅色主题' : 'Dark theme / 深色主题'} onClick={() => setTheme(theme === 'dark' ? 'light' : 'dark')}>{theme === 'dark' ? <Sun size={17} /> : <Moon size={17} />}</Button><Button variant="ghost language" onClick={() => setLanguage(language === 'zh' ? 'en' : 'zh')} aria-label="Change language / 切换语言">{language === 'zh' ? 'EN' : '中'}</Button></div><Entry /></LanguageContext>;
}

function Entry() {
  const { t } = useText();
  const [sessionExpired, setSessionExpired] = useState(false);
  useEffect(() => {
    const expire = () => setSessionExpired(true);
    window.addEventListener('rstrbot:session-expired', expire);
    return () => window.removeEventListener('rstrbot:session-expired', expire);
  }, []);
  const setup = useQuery({ queryKey: ['setup'], queryFn: ({ signal }) => api<Setup>('/setup', { signal }) });
  const me = useQuery({ queryKey: ['me'], enabled: setup.data?.initialized === true, queryFn: ({ signal }) => api<{ user: User }>('/auth/me', { signal }) });
  if (setup.isPending || (setup.data?.initialized && me.isPending)) return <main className="boot"><Brand /><Loading text={t('正在连接你的工作区…', 'Connecting to your workspace…')} /></main>;
  if (setup.error) return <main className="boot"><Brand /><ErrorNotice error={setup.error} onRetry={() => setup.refetch()} /></main>;
  // React Query retains the last successful user after a refetch fails. A 401
  // must leave the old account's shell even while that cached value exists.
  if (!setup.data?.initialized || !me.data?.user || sessionExpired || (me.error instanceof ApiError && me.error.status === 401)) return <Auth setup={setup.data!} />;
  return <Shell user={me.data.user} />;
}

function Shell({ user }: { user: User }) {
  const { t } = useText(); const client = useQueryClient();
  const workspaces = useQuery({ queryKey: ['workspaces', user.id], queryFn: ({ signal }) => api<{ items: Workspace[] }>('/workspaces', { signal }).then(items), refetchInterval: 10000 });
  const [wid, setWid] = useState(() => safePreference(`workspace:${user.id}`));
  const [page, setPage] = useState<Page>('chat'); const [drawer, setDrawer] = useState(false); const [newWorkspace, setNewWorkspace] = useState(false); const [name, setName] = useState('');
  const [error, setError] = useState<unknown>(null); const [busy, setBusy] = useState(false);
  const lastMembership = useRef<{ id: string; role: string } | null>(null);
  const workspace = workspaces.data?.find(w => w.id === wid) ?? workspaces.data?.[0];
  const capabilities = useQuery({ queryKey: [workspace?.id, 'capabilities'], enabled: !!workspace, refetchInterval: 5000, queryFn: ({ signal }) => api<Record<string, { supported: boolean; authorized: boolean; available: boolean }>>(`/workspaces/${workspace!.id}/capabilities`, { signal }) });
  const visibleNavigation = navigation.filter(item => {
    if (['members', 'plugins', 'routes'].includes(item.page)) {
      const capability = capabilities.data?.[item.page];
      const minimum = item.page === 'routes' ? 'member' : 'admin';
      return roleAllows(workspace?.role ?? '', minimum) && (!capability || capability.supported && capability.authorized);
    }
    return true;
  });
  useLayoutEffect(() => {
    const previous = lastMembership.current;
    if (workspace && previous?.id === workspace.id && previous.role !== workspace.role) {
      // A downgrade invalidates cached private/admin views as well as buttons.
      void client.resetQueries({ queryKey: [workspace.id] });
    }
    lastMembership.current = workspace ? { id: workspace.id, role: workspace.role } : null;
  }, [workspace?.id, workspace?.role, client]);
  const currentPage = visibleNavigation.some(item => item.page === page) ? page : 'chat';
  useEffect(() => { if (currentPage !== page) setPage(currentPage); }, [currentPage, page]);
  useEffect(() => { if (workspace && workspace.id !== wid) setWid(workspace.id); }, [workspace, wid]);
  async function switchWorkspace(id: string) { await client.cancelQueries({ queryKey: [wid] }); setWid(id); savePreference(`workspace:${user.id}`, id); setDrawer(false); setError(null); }
  async function logout() { setError(null); try { await post('/auth/logout'); await client.cancelQueries(); client.clear(); window.location.reload(); } catch (e) { setError(e); } }
  async function create(e: React.FormEvent) {
    e.preventDefault(); setBusy(true); setError(null);
    try { const response = await post<Workspace | { workspace: Workspace }>('/workspaces', { name }); const created = 'workspace' in response ? response.workspace : response; setWid(created.id); savePreference(`workspace:${user.id}`, created.id); await client.invalidateQueries({ queryKey: ['workspaces'] }); setNewWorkspace(false); setName(''); }
    catch (e) { setError(e); } finally { setBusy(false); }
  }
  const nav = <><div className="workspace-chooser"><label><span className="eyebrow">{t('当前工作区', 'WORKSPACE')}</span><select aria-label={t('切换工作区', 'Switch workspace')} value={workspace?.id ?? ''} onChange={e => void switchWorkspace(e.target.value)}>{workspaces.data?.map(w => <option key={w.id} value={w.id}>{w.name}</option>)}</select></label><Button variant="ghost icon" aria-label={t('建立工作区', 'Create workspace')} onClick={() => setNewWorkspace(true)}><Plus size={17} /></Button></div><nav aria-label={t('主导航', 'Main navigation')}><div className="nav-group">{visibleNavigation.filter(n => !n.management).map(({ page: id, title, icon: Icon }) => <button key={id} className={`nav-item ${currentPage === id ? 'active' : ''}`} aria-current={currentPage === id ? 'page' : undefined} onClick={() => { setPage(id); setDrawer(false); }}><Icon size={18} /><span>{t(...title)}</span>{currentPage === id && <span className="nav-active-dot" />}</button>)}</div><p className="nav-section-label">{t('连接与管理', 'CONNECT & MANAGE')}</p><div className="nav-group">{visibleNavigation.filter(n => n.management).map(({ page: id, title, icon: Icon }) => <button key={id} className={`nav-item ${currentPage === id ? 'active' : ''}`} aria-current={currentPage === id ? 'page' : undefined} onClick={() => { setPage(id); setDrawer(false); }}><Icon size={18} /><span>{t(...title)}</span>{currentPage === id && <span className="nav-active-dot" />}</button>)}</div></nav><div className="sidebar-bottom"><div className="account"><span className="avatar">{user.username.slice(0, 2).toUpperCase()}</span><div><strong>{user.username}</strong><span>{workspace?.role ?? t('账户', 'Account')}</span></div><Button variant="ghost icon" aria-label={t('退出登录', 'Sign out')} onClick={logout}><LogOut size={17} /></Button></div><div className="sidebar-version"><span>RSTRBOT / 0.1</span><span>{t('由你掌控', 'IN YOUR CONTROL')}</span></div></div></>;
  return <div className="app-shell"><a className="skip-link" href="#main-content">{t('跳到主要内容', 'Skip to main content')}</a><aside className="main-sidebar"><a className="brand-link" href="#" onClick={e => { e.preventDefault(); setPage('chat'); }} aria-label="RstrBot"><Brand /></a>{nav}</aside><div className="workspace-shell"><header className="topbar"><div className="row"><Button variant="ghost icon mobile-only" aria-label={t('打开导航', 'Open navigation')} aria-expanded={drawer} onClick={() => setDrawer(true)}><Menu size={20} /></Button><span className="topbar-workspace">{workspace?.name ?? 'RstrBot'}</span><span className="topbar-separator">/</span><span>{t(...navigation.find(n => n.page === currentPage)!.title)}</span></div><span className="topbar-note">{t('一处工作区，连接你的可能。', 'Your work, connected.')}</span></header><main id="main-content" className={`main-content ${currentPage === 'chat' ? 'chat-content' : ''}`} tabIndex={-1}><ErrorNotice error={error || workspaces.error} onRetry={() => workspaces.refetch()} />{workspaces.isPending ? <Loading /> : !workspace ? <Empty title={t('你的第一个工作区', 'Your first workspace')} description={t('建立工作区，或请管理员把你的用户 ID 加入现有团队。', 'Create a workspace, or ask an administrator to add your user ID to an existing team.')} action={<Button variant="primary" onClick={() => setNewWorkspace(true)}><Plus size={17} />{t('建立工作区', 'Create workspace')}</Button>} /> : <div className="page-mount" key={`${user.id}:${workspace.id}:${workspace.role}:${currentPage}`}>{currentPage === 'chat' ? <Chat workspace={workspace} user={user} goModels={() => setPage('models')} /> : currentPage === 'runs' ? <Runs workspace={workspace} user={user} /> : currentPage === 'members' ? <Members workspace={workspace} user={user} /> : currentPage === 'settings' ? <SettingsPage workspace={workspace} user={user} /> : <Resources kind={currentPage} workspace={workspace} user={user} />}</div>}</main></div>
    {drawer && <Modal title="RstrBot" close={() => setDrawer(false)}><div className="mobile-nav">{nav}</div></Modal>}
    {newWorkspace && <Modal title={t('建立工作区', 'Create workspace')} close={() => setNewWorkspace(false)}><form className="stack" onSubmit={create}><ErrorNotice error={error} /><Field label={t('名称', 'Name')}><input required value={name} onChange={e => setName(e.target.value)} autoFocus maxLength={120} placeholder={t('给这段工作起个名字', 'Give this work a name')} /></Field><Button type="submit" variant="primary" busy={busy}>{t('建立', 'Create')}<ArrowUpRight size={17} /></Button></form></Modal>}
  </div>;
}
