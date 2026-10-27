import { useState } from 'react';
import { Send } from 'lucide-react';
import { formatTime, post, value } from './api';
import type { Resource } from './api';
import { useResources, workspacePath } from './data';
import { Button, ErrorNotice, Field, Loading, LoadMore, Modal, Status, useText } from './ui';

type Intent = { key: string; text: string; started: boolean; receipt?: Resource };

export function SendDialog({ workspace, user, route, close, saved }: {
  workspace: string; user: string; route: Resource; close: () => void; saved: () => void;
}) {
  const { t } = useText();
  const storageKey = `rstrbot:delivery:${user}:${workspace}:${route.id}`;
  const fresh = (): Intent => ({ key: crypto.randomUUID(), text: '', started: false });
  const [intent, setIntent] = useState<Intent>(() => {
    try {
      const stored = JSON.parse(sessionStorage.getItem(storageKey) ?? 'null') as Intent | null;
      if (stored && typeof stored.key === 'string' && typeof stored.text === 'string') return stored;
    } catch { /* The request key remains stable in this dialog when storage is unavailable. */ }
    return fresh();
  });
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<unknown>(null);
  const remember = (next: Intent) => {
    setIntent(next);
    try { sessionStorage.setItem(storageKey, JSON.stringify(next)); } catch { /* Optional recovery aid. */ }
  };
  async function submit(e: React.FormEvent) {
    e.preventDefault(); setBusy(true); setError(null);
    const next = { ...intent, started: true }; remember(next);
    try {
      const receipt = await post<Resource>(`${workspacePath(workspace)}/routes/${encodeURIComponent(route.id)}/send`, {
        text: next.text, idempotency_key: next.key,
      });
      remember({ ...next, receipt }); saved();
    } catch (err) { setError(err); } finally { setBusy(false); }
  }
  function newMessage() { remember(fresh()); setError(null); }
  const status = intent.receipt ? value(intent.receipt.data, 'status', 'unknown') : '';
  const needsReview = status && !['accepted', 'rejected', 'not_sent'].includes(status);
  return <Modal title={t('发送到渠道', 'Send to channel')} close={() => { if (!busy) close(); }}>
    <form className="stack" onSubmit={submit}>
      <p><strong>{value(route.data, 'name', route.id)}</strong></p>
      <p className="muted small">{value(route.data, 'url')}</p>
      <Field label={t('消息内容', 'Message')} hint={t('只发送到此渠道保存的地址。接收端可能记录这条消息。', 'This sends to the saved destination. The receiver may store your message.')}>
        <textarea value={intent.text} rows={6} maxLength={16000} required disabled={intent.started}
          onChange={e => setIntent({ ...intent, text: e.target.value })} />
      </Field>
      <ErrorNotice error={error} />
      {intent.started && !intent.receipt && !busy && <p className="info-line">{t('响应尚未确认。再次查询将使用同一个请求编号，不会自动发起第二次投递。', 'The response is unconfirmed. Querying again uses the same request ID and does not start a second delivery.')}</p>}
      {intent.receipt && <div className="stack" role="status"><Status state={status} />
        <p className="small muted">{t('投递编号：', 'Delivery ID: ')}{intent.receipt.id}</p>
        <p>{status === 'accepted' ? t('接收端已接受请求。此回执不表示最终收件人已经阅读。', 'The receiver accepted the request. This receipt does not confirm that the final recipient read it.') : needsReview ? t('结果需要核对。请先凭投递编号检查目标端；系统不会自动重发。', 'Reconcile this delivery ID with the destination first. The system will not resend it automatically.') : t('本次未确认成功投递，请检查渠道或接收端。', 'This delivery was not confirmed as successful. Check the channel or receiver.')}</p>
      </div>}
      <div className="dialog-actions"><Button type="button" disabled={busy} onClick={close}>{t('关闭', 'Close')}</Button>
        {!intent.receipt && <Button type="submit" variant="primary" busy={busy} disabled={!intent.text.trim()}><Send size={16} />{intent.started ? t('查询原投递结果', 'Query original delivery') : t('发送', 'Send')}</Button>}
        {intent.receipt && <Button type="button" onClick={newMessage}>{needsReview ? t('已核对，创建另一条消息', 'Reconciled; create another message') : t('创建另一条消息', 'Create another message')}</Button>}
      </div>
    </form>
  </Modal>;
}

export function DeliveryHistory({ workspace }: { workspace: string }) {
  const { t, language } = useText();
  const query = useResources(workspace, 'deliveries', 5000);
  return <section className="stack" aria-label={t('我的投递记录', 'My deliveries')}>
    <h2>{t('我的投递记录', 'My deliveries')}</h2>
    <p className="muted small">{t('回执仅本人可见。需要核对的记录不会自动重发。', 'Receipts are private to you. Deliveries requiring review are never automatically replayed.')}</p>
    <ErrorNotice error={query.error} onRetry={() => { void query.refetch(); }} />
    {query.isPending ? <Loading /> : <div className="resource-list">{query.data?.map(receipt => <article className="resource-row" key={receipt.id}>
      <div className="resource-main"><p className="resource-description">{value(receipt.data, 'text')}</p>
        <p className="small muted">{receipt.id} · {formatTime(receipt.updated_at, language)}</p></div>
      <Status state={value(receipt.data, 'status', 'unknown')} />
    </article>)}{!query.data?.length && <p className="muted">{t('还没有投递记录。', 'No deliveries yet.')}</p>}</div>}
    <LoadMore query={query} label={t('加载更早的投递', 'Load older deliveries')} />
  </section>;
}
