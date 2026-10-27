import { useInfiniteQuery, useQuery, useQueryClient } from '@tanstack/react-query';
import type { InfiniteData, QueryClient, QueryKey } from '@tanstack/react-query';
import { api, isResourceUnavailable, items, unwrap } from './api';
import type { Message, Resource } from './api';
import { mergeResourcePages, nextResourceOffset, RESOURCE_PAGE_SIZE } from './conversation';

export const workspacePath = (workspace: string) => `/workspaces/${encodeURIComponent(workspace)}`;
export const resourcePath = (workspace: string, id: string) => `${workspacePath(workspace)}/resource/${encodeURIComponent(id)}`;
export const collectionPath = (workspace: string, kind: string) => `${workspacePath(workspace)}/resources/${encodeURIComponent(kind)}`;
type ResourcePage = { items: Resource[] };
type MessagePage = { items: Message[]; next_before: number | null; total: number };

async function authorizedRead<T>(client: QueryClient, key: QueryKey, signal: AbortSignal, read: () => Promise<T>): Promise<T> {
  try { return await read(); }
  catch (error) {
    // Failed refetches retain data. Clear the exact cache on a definite denial,
    // so a later network error or a new observer cannot revive the old content.
    // Only a successful server read replaces this null access barrier.
    if (!signal.aborted && isResourceUnavailable(error)) client.setQueryData(key, null);
    throw error;
  }
}

export function useResources(workspace: string, kind: string, interval?: number) {
  const client = useQueryClient();
  const key = [workspace, 'resources', kind];
  const query = useInfiniteQuery<ResourcePage, Error, InfiniteData<ResourcePage> | null, QueryKey, number>({
    queryKey: key, enabled: !!workspace,
    initialPageParam: 0,
    queryFn: ({ pageParam, signal }) => authorizedRead(client, key, signal, async () => ({ items: items(await api<ResourcePage>(`${collectionPath(workspace, kind)}?limit=${RESOURCE_PAGE_SIZE}&offset=${pageParam}`, { signal })) })),
    getNextPageParam: (lastPage, _pages, lastOffset) => nextResourceOffset(lastPage, lastOffset),
    refetchInterval: interval,
  });
  return { ...query, data: query.data ? mergeResourcePages(query.data.pages) : undefined, unavailable: query.data === null || isResourceUnavailable(query.error) };
}
export function useResource(workspace: string, id: string | null, interval?: number, metadataOnly = false) {
  const client = useQueryClient();
  const key = [workspace, 'resource', id];
  const query = useQuery<Resource | null>({
    queryKey: key, enabled: !!workspace && !!id,
    queryFn: ({ signal }) => authorizedRead(client, key, signal, async () => unwrap(await api<Resource>(`${resourcePath(workspace, id!)}${metadataOnly ? '?include_messages=false' : ''}`, { signal }))),
    refetchInterval: interval,
  });
  return { ...query, data: query.data ?? undefined, unavailable: query.data === null || isResourceUnavailable(query.error) };
}

export function useMessages(workspace: string, id: string | null) {
  const client = useQueryClient();
  const key = [workspace, 'messages', id];
  const query = useInfiniteQuery<MessagePage, Error, InfiniteData<MessagePage> | null, QueryKey, number | undefined>({
    queryKey: key, enabled: !!workspace && !!id,
    initialPageParam: undefined,
    queryFn: ({ pageParam, signal }) => authorizedRead(client, key, signal, () => api<MessagePage>(`${workspacePath(workspace)}/conversations/${encodeURIComponent(id!)}/messages?limit=50${pageParam === undefined ? '' : `&before=${pageParam}`}`, { signal })),
    getNextPageParam: page => page.next_before ?? undefined,
  });
  return { ...query, data: query.data ?? undefined, unavailable: query.data === null || isResourceUnavailable(query.error) };
}

export function safePreference(key: string, fallback = '') {
  try { return localStorage.getItem(`rstrbot:${key}`) ?? fallback; } catch { return fallback; }
}
export function savePreference(key: string, value: string) {
  try { localStorage.setItem(`rstrbot:${key}`, value); } catch { /* Private browsing can disable storage. The current session still works. */ }
}
