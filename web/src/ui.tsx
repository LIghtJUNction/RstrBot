import { cloneElement, isValidElement, createContext, useContext, useEffect, useId, useRef } from 'react';
import type { ButtonHTMLAttributes, ReactNode } from 'react';
import { AlertCircle, ArrowRight, LoaderCircle, X } from 'lucide-react';
import { friendlyError } from './api';

export type Language = 'zh' | 'en';
export const LanguageContext = createContext<Language>('zh');
export function useText() {
  const language = useContext(LanguageContext);
  return { language, t: (zh: string, en: string) => language === 'zh' ? zh : en };
}

export function Button({ children, className = '', busy, variant = '', ...rest }: ButtonHTMLAttributes<HTMLButtonElement> & { busy?: boolean; variant?: string }) {
  return <button {...rest} disabled={rest.disabled || busy} className={`button ${variant} ${className}`}>
    {busy && <LoaderCircle size={16} className="spin" aria-hidden="true" />}{children}
  </button>;
}

export function Brand({ small = false }: { small?: boolean }) {
  return <span className={`brand ${small ? 'small' : ''}`}><svg viewBox="0 0 32 32" aria-hidden="true"><path d="M7 26V6h14l5 7-11 6 11 7h-8L7 18l14-6h-8v14" fill="none" stroke="currentColor" strokeWidth="2.3" strokeLinejoin="round" /></svg><span>RstrBot<span className="brand-dot">.</span></span></span>;
}

export function ErrorNotice({ error, onRetry }: { error: unknown; onRetry?: () => void }) {
  const { t } = useText();
  if (!error) return null;
  return <div className="notice error" role="alert"><AlertCircle size={18} aria-hidden="true" /><div><strong>{t('未能完成操作', 'The operation could not be completed')}</strong><p>{friendlyError(error)}</p></div>{onRetry && <Button onClick={onRetry}>{t('重试', 'Retry')}</Button>}</div>;
}

export function Loading({ text }: { text?: string }) {
  const { t } = useText();
  return <div className="loading" role="status"><LoaderCircle size={20} className="spin" aria-hidden="true" />{text ?? t('正在读取…', 'Loading…')}</div>;
}

export function Empty({ title, description, action }: { title: string; description?: string; action?: ReactNode }) {
  return <div className="empty"><div className="empty-mark" aria-hidden="true">↗</div><h2>{title}</h2>{description && <p>{description}</p>}{action}</div>;
}

export function LoadMore({ query, label }: { query: { hasNextPage: boolean; isFetchingNextPage: boolean; fetchNextPage: () => Promise<unknown> }; label?: string }) {
  const { t } = useText();
  if (!query.hasNextPage) return null;
  return <div className="load-more"><Button type="button" busy={query.isFetchingNextPage} onClick={() => void query.fetchNextPage()}>{label ?? t('加载更多', 'Load more')}</Button></div>;
}

export function Modal({ title, children, close, wide = false, closeDisabled = false }: { title: string; children: ReactNode; close: () => void; wide?: boolean; closeDisabled?: boolean }) {
  const ref = useRef<HTMLDialogElement>(null);
  const titleId = useId();
  const { t } = useText();
  useEffect(() => { const node = ref.current; node?.showModal(); return () => node?.close(); }, []);
  return <dialog ref={ref} className={`dialog ${wide ? 'wide' : ''}`} onCancel={e => { e.preventDefault(); if (!closeDisabled) close(); }} onClick={e => { if (e.target === ref.current && !closeDisabled) close(); }} aria-labelledby={titleId}>
    <div className="dialog-inner"><header><h2 id={titleId}>{title}</h2><Button variant="ghost icon" disabled={closeDisabled} onClick={close} aria-label={t('关闭', 'Close')}><X size={20} /></Button></header>{children}</div>
  </dialog>;
}

export function Field({ label, hint, children }: { label: string; hint?: string; children: ReactNode }) {
  const generated = useId();
  const control = isValidElement<{ id?: string; 'aria-describedby'?: string }>(children) ? children : null;
  const id = control?.props.id ?? generated;
  const description = [control?.props['aria-describedby'], hint ? `${generated}-hint` : null].filter(Boolean).join(' ') || undefined;
  return <div className="field"><label htmlFor={id}>{label}</label>{control ? cloneElement(control, { id, 'aria-describedby': description }) : children}{hint && <small id={`${generated}-hint`}>{hint}</small>}</div>;
}

export function PageHeader({ eyebrow, title, description, action }: { eyebrow?: string; title: string; description?: string; action?: ReactNode }) {
  return <header className="page-heading"><div>{eyebrow && <p className="eyebrow">{eyebrow}</p>}<h1>{title}</h1>{description && <p>{description}</p>}</div>{action}</header>;
}

export function Status({ state }: { state?: string }) {
  const { t } = useText();
  const labels: Record<string, [string, string]> = {
    completed: ['已完成', 'Completed'], succeeded: ['已完成', 'Completed'], running: ['运行中', 'Running'], queued: ['已排队', 'Queued'], cancelled: ['已停止', 'Stopped'], failed: ['失败', 'Failed'], unknown: ['待核对', 'Needs review'], interrupted: ['已中断', 'Interrupted'],
    enabled: ['已启用', 'Enabled'], disabled: ['已停用', 'Disabled'], installed: ['已安装', 'Installed'], ready: ['可用', 'Ready'], indexed: ['可检索', 'Indexed'], indexing: ['建立索引', 'Indexing'], uploaded: ['已上传', 'Uploaded'], active: ['使用中', 'Active'], archived: ['已归档', 'Archived'], paused: ['已暂停', 'Paused'], pending: ['待执行', 'Pending'], degraded: ['已降级', 'Degraded'], accepted: ['已接受', 'Accepted'], rejected: ['已拒绝', 'Rejected'], prepared: ['待发送', 'Prepared'], sending: ['发送中', 'Sending'], not_sent: ['未发送', 'Not sent'], saved: ['已保存', 'Saved'], not_configured: ['未配置', 'Not configured'],
  };
  const pair = labels[state ?? ''];
  return <span className={`status state-${state ?? 'unknown'}`}><span aria-hidden="true" />{pair ? t(...pair) : state ?? t('未知', 'Unknown')}</span>;
}

export function LinkAction({ children, onClick }: { children: ReactNode; onClick: () => void }) {
  return <Button variant="text" onClick={onClick}>{children}<ArrowRight size={16} aria-hidden="true" /></Button>;
}
