import type { Event, Json, Message } from './api';

/** Preserve valid history even if one stored record is malformed or duplicated. */
export function normalizeMessages(input: unknown): Message[] {
  if (!Array.isArray(input)) return [];
  const seen = new Set<string>();
  return input.flatMap((entry: unknown) => {
    if (!entry || typeof entry !== 'object' || !('role' in entry) || typeof entry.role !== 'string') return [];
    const raw = entry as Json;
    const id = typeof raw.id === 'string' ? raw.id : undefined;
    if (id && seen.has(id)) return [];
    if (id) seen.add(id);
    const content = typeof raw.content === 'string' ? raw.content : raw.content === null || raw.content === undefined ? '' : JSON.stringify(raw.content);
    return [{ ...raw, role: raw.role as string, id, content } as Message];
  });
}

export function nextCursor(current: number, returned: unknown, eventIds: number[]): number {
  const cursor = typeof returned === 'number' && Number.isSafeInteger(returned) && returned >= 0 ? returned : current;
  return Math.max(current, cursor, ...eventIds.filter(Number.isSafeInteger));
}

export type StreamState = { cursor: number; live: Record<string, string>; finished: string[] };
export const emptyStream = (): StreamState => ({ cursor: 0, live: {}, finished: [] });

/** Event delivery may overlap after a reconnect. Each ID contributes at most once. */
export function applyEventPage(previous: StreamState, events: Event[], cursor: unknown): { state: StreamState; changed: boolean } {
  const live = { ...previous.live };
  const finished = new Set(previous.finished);
  let after = previous.cursor;
  let changed = false;
  for (const event of [...events].sort((a, b) => a.id - b.id)) {
    if (!Number.isSafeInteger(event.id) || event.id <= after) continue;
    after = event.id;
    const run = typeof event.data.run_id === 'string' ? event.data.run_id : undefined;
    if (event.kind === 'run.delta' && run) {
      if (finished.has(run)) continue;
      const delta = typeof event.data.delta === 'string' ? event.data.delta : '';
      live[run] = ((live[run] ?? '') + delta).slice(-200000);
      // The server allows at most 16 concurrent runs. Bound orphaned streams too.
      if (Object.keys(live).length > 16) delete live[Object.keys(live)[0]];
    } else {
      changed = true;
      if (run && (event.kind === 'run.finished' || /completed|failed|cancel|interrupt|unknown/.test(event.kind ?? ''))) {
        delete live[run];
        finished.add(run);
      }
      if (event.kind === 'resource.deleted' && event.data.kind === 'runs' && event.resource_id) {
        delete live[event.resource_id];
        finished.add(event.resource_id);
      }
    }
  }
  return { state: { cursor: nextCursor(after, cursor, []), live, finished: [...finished].slice(-256) }, changed };
}

/** An acknowledgement owns only the draft revision that it submitted. */
export function acknowledgeDraft(current: { text: string; revision: number }, submittedRevision: number): string {
  return current.revision === submittedRevision ? '' : current.text;
}

/** Offset pages can overlap when another user inserts a resource between reads. */
export function mergeResourcePages<T extends { id: string; version: number }>(pages: { items: T[] }[]): T[] {
  const seen = new Map<string, T>();
  for (const page of pages) for (const resource of page.items) {
    const previous = seen.get(resource.id);
    if (!previous || resource.version > previous.version) seen.set(resource.id, resource);
  }
  return [...seen.values()];
}

export const RESOURCE_PAGE_SIZE = 100;
export function nextResourceOffset(lastPage: { items: unknown[] }, lastOffset: number): number | undefined {
  return lastPage.items.length === RESOURCE_PAGE_SIZE ? lastOffset + RESOURCE_PAGE_SIZE : undefined;
}
