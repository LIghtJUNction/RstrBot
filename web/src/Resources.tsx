import { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { BookOpen, Brain, CalendarClock, Check, ChevronRight, Cpu, FileText, FlaskConical, LockKeyhole, MoreHorizontal, Pencil, Plug, Plus, RefreshCw, Search, Send, Trash2, Users } from 'lucide-react';
import type { LucideIcon } from 'lucide-react';
import { api, formatTime, items, number, patch, post, remove, value } from './api';
import type { Json, Resource, User, Workspace } from './api';
import { collectionPath, resourcePath, useResources, workspacePath } from './data';
import { DeliveryHistory, SendDialog } from './Delivery';
import { canCreateResource, canWriteResource, roleAllows } from './permissions';
import { Button, Empty, ErrorNotice, Field, Loading, LoadMore, Modal, PageHeader, Status, useText } from './ui';

type ResourceKind = 'models' | 'plugins' | 'knowledge' | 'memories' | 'schedules' | 'routes' | 'agents';
type FieldDefinition = { key: string; label: [string, string]; type?: string; required?: boolean; placeholder?: string; options?: [string, string, string][]; hint?: [string, string]; default?: unknown };
const icons: Record<ResourceKind, LucideIcon> = { models: Cpu, plugins: Plug, knowledge: BookOpen, memories: Brain, schedules: CalendarClock, routes: Send, agents: Users };
const pageTitles: Record<ResourceKind, [string, string]> = { models: ['模型', 'Models'], plugins: ['插件', 'Plugins'], knowledge: ['知识库', 'Knowledge'], memories: ['长期记忆', 'Memory'], schedules: ['计划任务', 'Schedules'], routes: ['机器人与渠道', 'Channels'], agents: ['代理', 'Agents'] };
const descriptions: Record<ResourceKind, [string, string]> = {
  models: ['把合适的模型，放在合适的事情上。', 'The right model for the work at hand.'],
  plugins: ['独立的能力，清楚的授权。每个工作区各自配置。', 'Independent capabilities, explicit permissions. Configured per workspace.'],
  knowledge: ['保留原文，让回答有据可查。', 'Keep the source. Give every answer a reference.'],
  memories: ['记住什么，由你决定。', 'What stays with your assistant is up to you.'],
  schedules: ['安排好下一件事，回来看结果。', 'Make a plan. Come back to the result.'],
  routes: ['把结果送到经过授权的地方。', 'Deliver results to destinations you authorize.'],
  agents: ['为常做的事，设定一套做法。', 'Give recurring work its own instructions.'],
};

function definitions(kind: ResourceKind): FieldDefinition[] {
  switch (kind) {
    case 'models': return [
      { key: 'name', label: ['显示名称', 'Display name'], required: true },
      { key: 'provider', label: ['供应商插件', 'Provider plugin'], type: 'select', default: 'openai', options: [['openai', 'OpenAI 兼容', 'OpenAI compatible'], ['diagnostic', '诊断测试（非 AI）', 'Diagnostic fixture (not AI)']] },
      { key: 'base_url', label: ['API 地址', 'API base URL'], placeholder: 'https://api.example.com/v1', hint: ['本地模型需要实例管理员明确允许私有网络。', 'Local models require explicit private-network authorization by the instance administrator.'] },
      { key: 'model', label: ['模型 ID', 'Model ID'], required: true, placeholder: 'model-name' },
      { key: 'api_key', label: ['API 密钥', 'API key'], type: 'password', hint: ['只写入，不回显。留空保留密钥；更换 API 地址或供应商时，需填写新密钥或勾选清除。', 'Write-only. Blank keeps the key. Changing the API endpoint or provider requires a new key or explicit clearing.'] },
      { key: 'clear_api_key', label: ['清除已有 API 密钥', 'Clear the saved API key'], type: 'checkbox', default: false },
      { key: 'max_tokens', label: ['最大输出 token', 'Maximum output tokens'], type: 'number', default: 2048 },
      { key: 'input_price_usd', label: ['输入单价（USD / 百万 token）', 'Input price (USD / million tokens)'], placeholder: '0.00', hint: ['可留空。没有价格时费用保持未知。', 'Optional. Costs remain unknown without pricing.'] },
      { key: 'output_price_usd', label: ['输出单价（USD / 百万 token）', 'Output price (USD / million tokens)'], placeholder: '0.00' },
      { key: 'allow_private_network', label: ['允许本地 / 私有网络（仅实例管理员）', 'Allow local / private network (instance administrator only)'], type: 'checkbox', default: false },
    ];
    case 'plugins': return [{ key: 'name', label: ['实例名称', 'Instance name'], required: true }, { key: 'package_id', label: ['已安装的插件包', 'Installed package'], type: 'package', required: true }, { key: 'enabled', label: ['启用此实例', 'Enable this instance'], type: 'checkbox', default: false }];
    case 'knowledge': return [{ key: 'title', label: ['文档标题', 'Document title'], required: true }, { key: 'text', label: ['原文', 'Source text'], type: 'textarea', required: true, hint: ['支持 UTF-8 文本与 Markdown。上传后由服务端建立可检索分块。', 'UTF-8 text and Markdown are supported. The server builds searchable chunks after upload.'] }];
    case 'memories': return [{ key: 'title', label: ['标题', 'Title'], required: true }, { key: 'content', label: ['记忆内容', 'Memory'], type: 'textarea', required: true }, { key: 'source', label: ['来源', 'Source'], placeholder: '用户明确保存 / Explicitly saved by user' }, { key: 'enabled', label: ['参与后续对话', 'Use in future conversations'], type: 'checkbox', default: true }, { key: 'pinned', label: ['固定重要记忆', 'Pin important memory'], type: 'checkbox', default: false }];
    case 'schedules': return [{ key: 'title', label: ['任务名称', 'Schedule name'], required: true }, { key: 'prompt', label: ['要做的事', 'Instructions'], type: 'textarea', required: true }, { key: 'model_id', label: ['使用模型', 'Model'], type: 'model', required: true }, { key: 'conversation_id', label: ['结果会话', 'Conversation for results'], type: 'conversation', required: true }, { key: 'schedule_type', label: ['时间安排', 'Schedule'], type: 'select', default: 'once', options: [['once', '指定时间，一次', 'Once, at a specific time'], ['interval', '固定间隔', 'Fixed interval'], ['cron', '日历计划', 'Calendar schedule']] }, { key: 'at', label: ['执行时间（当前设备时区）', 'Run at (device time zone)'], type: 'datetime-local', required: true }, { key: 'interval_seconds', label: ['间隔（秒）', 'Interval (seconds)'], type: 'number', default: 3600 }, { key: 'cron', label: ['日历表达式', 'Calendar expression'], placeholder: '0 0 9 * * *', hint: ['六段：秒 分 时 日 月 星期。示例为每日 09:00。', 'Six fields: second minute hour day month weekday. The example runs daily at 09:00.'] }, { key: 'timezone', label: ['时区', 'Time zone'], default: Intl.DateTimeFormat().resolvedOptions().timeZone, placeholder: 'Asia/Taipei', required: true }, { key: 'enabled', label: ['启用任务', 'Enable schedule'], type: 'checkbox', default: true }];
    case 'routes': return [{ key: 'name', label: ['渠道名称', 'Channel name'], required: true }, { key: 'url', label: ['Webhook 地址', 'Webhook URL'], type: 'url', required: true, hint: ['目标地址须由实例管理员保存并授权。', 'The instance administrator must save and authorize the destination.'] }, { key: 'token', label: ['授权令牌（可选）', 'Authorization token (optional)'], type: 'password', hint: ['仅写入。留空保留令牌；更换目标地址时，需填写新令牌或勾选清除。', 'Write-only. Blank keeps the token. Changing the destination requires a new token or explicit clearing.'] }, { key: 'clear_token', label: ['清除已有授权令牌', 'Clear the saved authorization token'], type: 'checkbox', default: false }, { key: 'allow_private_network', label: ['允许本地 / 私有网络（仅实例管理员）', 'Allow local / private network (instance administrator only)'], type: 'checkbox', default: false }, { key: 'enabled', label: ['启用投递', 'Enable delivery'], type: 'checkbox', default: false }];
    case 'agents': return [{ key: 'name', label: ['代理名称', 'Agent name'], required: true }, { key: 'system_prompt', label: ['工作方式', 'Instructions'], type: 'textarea', required: true }, { key: 'model_id', label: ['默认模型', 'Default model'], type: 'model', required: true }, { key: 'max_steps', label: ['最多执行轮次', 'Maximum execution steps'], type: 'number', default: 6 }, { key: 'knowledge', label: ['检索获准知识库', 'Search authorized knowledge'], type: 'checkbox', default: true }, { key: 'memory', label: ['使用获准记忆', 'Use authorized memory'], type: 'checkbox', default: true }];
  }
}

export function ResourceEditor({ workspace, kind, resource, close, saved }: { workspace: string; kind: ResourceKind; resource?: Resource; close: () => void; saved: () => void }) {
  const { t } = useText(); const fields = definitions(kind);
  const [draft, setDraft] = useState<Json>(() => ({ ...Object.fromEntries(fields.map(f => [f.key, f.default ?? ''])), ...(resource?.data ?? {}), api_key: '', token: '', ...(kind === 'models' ? { input_price_usd: typeof resource?.data.input_microusd_per_million === 'number' ? String(resource.data.input_microusd_per_million / 1000000) : '', output_price_usd: typeof resource?.data.output_microusd_per_million === 'number' ? String(resource.data.output_microusd_per_million / 1000000) : '' } : {}) }));
  const [visibility, setVisibility] = useState<'private' | 'workspace'>(resource?.visibility ?? (['models', 'plugins', 'knowledge', 'agents', 'routes'].includes(kind) ? 'workspace' : 'private'));
  const [busy, setBusy] = useState(false); const [error, setError] = useState<unknown>(null);
  const modelQuery = useResources(workspace, 'models'); const conversationQuery = useResources(workspace, 'conversations');
  const packageQuery = useQuery({ queryKey: ['plugin-packages'], enabled: kind === 'plugins', queryFn: ({ signal }) => api<{ items: Json[] }>('/plugin-packages', { signal }).then(items) });
  const set = (key: string, val: unknown) => setDraft(prev => ({ ...prev, [key]: val, ...(key === 'clear_api_key' && val ? { api_key: '' } : {}), ...(key === 'clear_token' && val ? { token: '' } : {}) }));
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true); setError(null);
    const data = { ...draft };
    if (!data.api_key) delete data.api_key;
    if (!data.token) delete data.token;
    if (kind === 'memories') data.archived = resource?.data.archived ?? false;
    if (kind === 'plugins') data.config = resource?.data.config ?? {};
    if (kind === 'routes') data.adapter = 'webhook';
    if (kind === 'schedules') {
      data.missed_policy = 'skip';
      if (typeof data.at === 'string' && data.at) data.at = Math.floor(new Date(data.at).getTime() / 1000);
      if (data.schedule_type !== 'once') delete data.at;
      if (data.schedule_type !== 'interval') delete data.interval_seconds;
      if (data.schedule_type !== 'cron') delete data.cron;
    }
    try {
      if (kind === 'models') {
        for (const direction of ['input', 'output']) {
          const text = String(data[`${direction}_price_usd`] ?? '').trim();
          delete data[`${direction}_price_usd`];
          if (!text) { delete data[`${direction}_microusd_per_million`]; continue; }
          if (!/^\d+(\.\d{1,6})?$/.test(text)) throw new Error(t('价格必须为非负数，最多六位小数。', 'Prices must be non-negative with up to six decimal places.'));
          const [whole, fraction = ''] = text.split('.'); const exact = BigInt(whole) * 1000000n + BigInt(fraction.padEnd(6, '0'));
          if (exact > BigInt(Number.MAX_SAFE_INTEGER)) throw new Error(t('价格过大。', 'Price is too large.'));
          data[`${direction}_microusd_per_million`] = Number(exact);
        }
      }
      if (resource) await patch(resourcePath(workspace, resource.id), { version: resource.version, data });
      else await post(collectionPath(workspace, kind), { visibility, data });
      saved(); close();
    } catch (err) { setError(err); } finally { setBusy(false); }
  }
  async function readFile(file: File | undefined) {
    if (!file) return;
    if (file.size > 512 * 1024) { setError(new Error(t('文本文件上限为 512 KiB。', 'Text files must be 512 KiB or smaller.'))); return; }
    if (!/\.(txt|md|markdown)$/i.test(file.name)) { setError(new Error(t('此版本支持 .txt 和 .md 文档。', 'This version supports .txt and .md documents.'))); return; }
    try {
      const text = new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer());
      setDraft(prev => ({ ...prev, text, title: prev.title || file.name })); setError(null);
    } catch { setError(new Error(t('文件不是有效的 UTF-8 文本。', 'The file is not valid UTF-8 text.'))); }
  }
  return <Modal title={`${resource ? t('编辑', 'Edit') : t('添加', 'Add')} ${t(...pageTitles[kind])}`} close={close} wide={kind === 'knowledge'}><form className="stack" onSubmit={submit}>
    <ErrorNotice error={error || (kind === 'plugins' ? packageQuery.error : null) || (fields.some(f => f.type === 'model') ? modelQuery.error : null) || (fields.some(f => f.type === 'conversation') ? conversationQuery.error : null)} />
    {kind === 'knowledge' && <label className="file-drop"><FileText size={24} /><span>{t('选择文本文件，或在下面粘贴内容', 'Choose a text file, or paste content below')}</span><input type="file" accept=".txt,.md,.markdown,text/plain,text/markdown" onChange={e => void readFile(e.target.files?.[0])} /></label>}
    {fields.filter(f => !((f.key === 'clear_api_key' && resource?.data.has_api_key !== true) || (f.key === 'clear_token' && resource?.data.has_token !== true)) && !(kind === 'schedules' && ((f.key === 'at' && draft.schedule_type !== 'once') || (f.key === 'interval_seconds' && draft.schedule_type !== 'interval') || (f.key === 'cron' && draft.schedule_type !== 'cron') || (f.key === 'timezone' && draft.schedule_type !== 'cron')))).map(f => {
      const raw = draft[f.key]; const text = raw === undefined || raw === null ? '' : String(raw);
      if (f.type === 'checkbox') return <label className="check-field" key={f.key}><input type="checkbox" checked={!!raw} onChange={e => set(f.key, e.target.checked)} /><span>{t(...f.label)}</span></label>;
      let options = f.options?.map(([id, zh, en]) => ({ id, name: t(zh, en) }));
      if (f.type === 'model') options = modelQuery.data?.map(r => ({ id: r.id, name: value(r.data, 'name', r.id) }));
      if (f.type === 'conversation') options = conversationQuery.data?.map(r => ({ id: r.id, name: value(r.data, 'title', r.id) }));
      if (f.type === 'package') options = packageQuery.data?.map(r => ({ id: value(r, 'id', value(r, 'package_id')), name: value(r, 'name', value(r, 'id', value(r, 'package_id'))) }));
      const select = ['select', 'model', 'conversation', 'package'].includes(f.type ?? '');
      const timestamp = f.type === 'datetime-local' && typeof raw === 'number' ? new Date(raw * 1000) : null;
      const date = timestamp && !Number.isNaN(timestamp.getTime()) ? new Date(timestamp.getTime() - timestamp.getTimezoneOffset() * 60000).toISOString().slice(0, 16) : text;
      return <Field key={f.key} label={t(...f.label)} hint={f.hint ? t(...f.hint) : undefined}>{select ? <select value={text} required={f.required} onChange={e => set(f.key, e.target.value)}><option value="">{t('请选择…', 'Choose…')}</option>{options?.map(o => <option key={o.id} value={o.id}>{o.name}</option>)}</select> : f.type === 'textarea' ? <textarea value={text} onChange={e => set(f.key, e.target.value)} required={f.required} rows={kind === 'knowledge' ? 11 : 5} maxLength={500000} /> : <input type={f.type ?? 'text'} disabled={(f.key === 'api_key' && draft.clear_api_key === true) || (f.key === 'token' && draft.clear_token === true)} required={f.required} placeholder={f.placeholder} autoComplete={f.type === 'password' ? 'new-password' : 'off'} min={f.type === 'number' ? 1 : undefined} value={f.type === 'datetime-local' ? date : text} onChange={e => set(f.key, f.type === 'number' ? (e.target.value ? Number(e.target.value) : '') : e.target.value)} />}</Field>;
    })}
    {fields.some(f => f.type === 'model') && <LoadMore query={modelQuery} label={t('加载更多模型', 'Load more models')} />}
    {fields.some(f => f.type === 'conversation') && <LoadMore query={conversationQuery} label={t('加载更早的会话', 'Load older conversations')} />}
    {!resource && <Field label={t('可见范围', 'Visibility')} hint={t('共享给当前工作区的成员；私有资源仅本人可见。', 'Workspace resources are shared with its members; private resources are visible only to you.')}><select value={visibility} onChange={e => setVisibility(e.target.value as 'private' | 'workspace')}><option value="private">{t('仅自己', 'Only me')}</option><option value="workspace">{t('当前工作区', 'This workspace')}</option></select></Field>}
    <div className="dialog-actions"><Button type="button" onClick={close}>{t('取消', 'Cancel')}</Button><Button type="submit" variant="primary" busy={busy}>{t('保存', 'Save')}<Check size={16} /></Button></div>
  </form></Modal>;
}

export default function Resources({ workspace, kind, user }: { workspace: Workspace; kind: ResourceKind; user: User }) {
  const { t, language } = useText(); const client = useQueryClient(); const query = useResources(workspace.id, kind, kind === 'schedules' ? 5000 : undefined);
  const [search, setSearch] = useState(''); const [editor, setEditor] = useState<Resource | 'new' | null>(null); const [deleting, setDeleting] = useState<Resource | null>(null);
  const [sending, setSending] = useState<Resource | null>(null);
  const [detail, setDetail] = useState<Resource | null>(null); const [busy, setBusy] = useState(''); const [error, setError] = useState<unknown>(null); const [notice, setNotice] = useState('');
  const Icon = icons[kind];
  const write = roleAllows(workspace.role, 'member');
  const allowedCreate = canCreateResource(workspace.role, kind, user);
  const refresh = () => { void client.invalidateQueries({ queryKey: [workspace.id] }); };
  const records = query.data?.filter(r => `${value(r.data, 'name')} ${value(r.data, 'title')} ${value(r.data, 'content')} ${value(r.data, 'text')} ${value(r.data, 'prompt')} ${value(r.data, 'model')}`.toLocaleLowerCase().includes(search.toLocaleLowerCase())) ?? [];
  async function action(record: Resource, name: string) {
    setBusy(`${record.id}:${name}`); setError(null); setNotice('');
    try {
      if (name === 'delete') { await remove(resourcePath(workspace.id, record.id), record.version); setDeleting(null); }
      else if (name === 'toggle' || name === 'archive') await patch(resourcePath(workspace.id, record.id), { version: record.version, data: { ...record.data, [name === 'toggle' ? 'enabled' : 'archived']: !record.data[name === 'toggle' ? 'enabled' : 'archived'] } });
      else { const result = await post<Json>(`${workspacePath(workspace.id)}/${kind}/${record.id}/${name}`); setNotice(name === 'test' ? t('实际调用测试通过。', 'The actual model request succeeded.') : t('操作已接受，请查看最新状态。', 'The operation was accepted. Check the updated status.')); if (result?.error) setNotice(String(result.error)); }
      refresh();
    } catch (e) { setError(e); } finally { setBusy(''); }
  }
  return <section className="page"><PageHeader eyebrow="YOUR WORKSPACE" title={t(...pageTitles[kind])} description={t(...descriptions[kind])} action={allowedCreate && <Button variant="primary" onClick={() => setEditor('new')}><Plus size={17} />{t('添加', 'Add')}</Button>} />
    {kind === 'knowledge' && <p className="info-line">{t('当前可导入文本与 Markdown，并进行词项检索。此版本尚未提供向量嵌入与 PDF 解析。', 'Import text or Markdown and use lexical retrieval. Vector embeddings and PDF parsing are not available in this version.')}</p>}
    {kind === 'plugins' && <p className="info-line">{t('选择由实例管理员安装的可信插件包。配置保存、启用与实际调用是独立状态。', 'Choose trusted packages installed by the instance administrator. Saved configuration, enabled state and actual invocation are separate.')}</p>}
    <ErrorNotice error={error || query.error} onRetry={refresh} />{notice && <div className="notice success" role="status"><Check size={18} />{notice}</div>}
    <div className="collection-toolbar"><label className="search"><Search size={17} aria-hidden="true" /><input aria-label={t('搜索已加载的条目', 'Search loaded items')} placeholder={t('搜索已加载的条目…', 'Search loaded items…')} value={search} onChange={e => setSearch(e.target.value)} /></label><span className="muted small">{records.length} {t('项', 'items')}</span><Button variant="ghost icon" aria-label={t('刷新', 'Refresh')} onClick={refresh}><RefreshCw size={17} /></Button></div>
    {query.isPending ? <Loading /> : records.length === 0 ? <Empty title={search ? t('没有找到匹配项', 'No matches') : t('这里还有很多可能', 'Room for something useful')} description={search ? t('试试其他关键词。', 'Try a different search.') : t('添加第一个条目，开始使用。', 'Add your first item to get started.')} action={allowedCreate && !search && <Button onClick={() => setEditor('new')}><Plus size={16} />{t('添加', 'Add')}</Button>} /> : <div className="resource-list">{records.map(record => {
      const title = value(record.data, 'name', value(record.data, 'title', record.id));
      const state = value(record.data, 'status') || (record.data.archived ? 'archived' : record.data.enabled === false ? 'disabled' : record.data.enabled === true ? 'enabled' : undefined);
      const canDelete = canWriteResource(workspace.role, user, record);
      const canEdit = canDelete && (kind !== 'schedules' || record.owner_id === user.id) && (kind !== 'routes' || user.instance_admin === true);
      const description = kind === 'models' ? `${value(record.data, 'provider')} / ${value(record.data, 'model')}` : kind === 'plugins' ? value(record.data, 'package_id') : value(record.data, 'content', value(record.data, 'text', value(record.data, 'prompt', value(record.data, 'system_prompt', value(record.data, 'url')))));
      return <article className="resource-row" key={record.id}><span className="resource-icon"><Icon size={22} /></span><div className="resource-main"><button className="resource-title" onClick={() => setDetail(record)}>{title}<ChevronRight size={15} /></button><p className="resource-description">{description || t('已保存配置', 'Configuration saved')}</p><div className="resource-meta">{record.visibility === 'private' ? <span><LockKeyhole size={12} />{t('私有', 'Private')}</span> : <span><Users size={12} />{t('工作区共享', 'Workspace')}</span>}<span>{formatTime(record.updated_at, language)}</span>{state && <Status state={state} />}{kind === 'models' && record.data.has_api_key === true && <span>{t('密钥已配置', 'Key configured')}</span>}{kind === 'schedules' && !!record.data.next_run && <span>{t('下次：', 'Next: ')}{formatTime(record.data.next_run as number, language)}</span>}</div></div><div className="resource-actions">
        {kind === 'models' && canEdit && <Button variant="ghost" busy={busy === `${record.id}:test`} onClick={() => action(record, 'test')}><FlaskConical size={16} /><span>{t('测试', 'Test')}</span></Button>}
        {kind === 'plugins' && canEdit && <Button variant="ghost icon" aria-label={t('重载', 'Reload')} busy={busy === `${record.id}:reload`} onClick={() => action(record, 'reload')}><RefreshCw size={16} /></Button>}
        {kind === 'routes' && write && record.data.enabled === true && <Button variant="ghost" onClick={() => setSending(record)}><Send size={16} />{t('发送消息', 'Send message')}</Button>}
        {kind === 'routes' && canDelete && record.data.enabled === true && <Button variant="ghost" onClick={() => action(record, 'toggle')} busy={busy === `${record.id}:toggle`}>{t('停用', 'Disable')}</Button>}
        {kind === 'schedules' && canEdit && <Button variant="ghost" onClick={() => action(record, 'run')} busy={busy === `${record.id}:run`}>{t('立即运行', 'Run now')}</Button>}
        {['plugins', 'schedules', 'memories'].includes(kind) && canEdit && <Button variant="ghost" onClick={() => action(record, 'toggle')} busy={busy === `${record.id}:toggle`}>{record.data.enabled ? t('停用', 'Disable') : t('启用', 'Enable')}</Button>}
        {canEdit && <Button variant="ghost icon" aria-label={`${t('编辑', 'Edit')} ${title}`} onClick={() => setEditor(record)}><Pencil size={16} /></Button>}
        {canDelete && <Button variant="ghost icon" aria-label={`${t('删除', 'Delete')} ${title}`} onClick={() => setDeleting(record)}><Trash2 size={16} /></Button>}
      </div></article>;
    })}</div>}
    <LoadMore query={query} />
    {kind === 'routes' && <DeliveryHistory workspace={workspace.id} />}
    {sending && <SendDialog workspace={workspace.id} user={user.id} route={sending} close={() => setSending(null)} saved={refresh} />}
    {editor && <ResourceEditor workspace={workspace.id} kind={kind} resource={editor === 'new' ? undefined : editor} close={() => setEditor(null)} saved={refresh} />}
    {deleting && <Modal title={t('删除此条目？', 'Delete this item?')} close={() => setDeleting(null)}><p>{value(deleting.data, 'name', value(deleting.data, 'title', deleting.id))}</p><p className="muted">{t('此操作将删除保存的条目。', 'This removes the saved item.')}</p><ErrorNotice error={error} /><div className="dialog-actions"><Button onClick={() => setDeleting(null)}>{t('取消', 'Cancel')}</Button><Button variant="danger" busy={busy === `${deleting.id}:delete`} onClick={() => action(deleting, 'delete')}>{t('删除', 'Delete')}</Button></div></Modal>}
    {detail && <Modal title={value(detail.data, 'name', value(detail.data, 'title', t('详情', 'Details')))} close={() => setDetail(null)} wide><div className="stack"><p className="muted small">{detail.id} · v{detail.version}</p><div className="document-text">{value(detail.data, 'content', value(detail.data, 'text', value(detail.data, 'prompt', value(detail.data, 'system_prompt', value(detail.data, 'base_url'))))) || t('此条目没有正文。', 'This item has no body text.')}</div>{kind === 'memories' && <><p className="small muted">{t('来源：', 'Source: ')}{value(detail.data, 'source', t('未注明', 'Not specified'))}</p>{canWriteResource(workspace.role, user, detail) && <Button onClick={() => { void action(detail, 'archive'); setDetail(null); }}>{detail.data.archived ? t('恢复使用', 'Restore') : t('归档', 'Archive')}</Button>}</>}{kind === 'knowledge' && <p className="muted small">{t('索引版本：', 'Index version: ')}{number(detail.data, 'index_version', 1)} · {Array.isArray(detail.data.chunks) ? detail.data.chunks.length : '—'} {t('个分块', 'chunks')}</p>}<Button onClick={() => setDetail(null)}>{t('关闭', 'Close')}</Button></div></Modal>}
  </section>;
}
