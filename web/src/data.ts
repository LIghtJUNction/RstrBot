import { useInfiniteQuery, useQuery } from '@tanstack/react-query';
import { api, items, unwrap } from './api';
import type { Message, Resource } from './api';
import { mergeResourcePages, nextResourceOffset, RESOURCE_PAGE_SIZE } from './conversation';

export const workspacePath = (workspace: string) => `/workspaces/${encodeURIComponent(workspace)}`;
export const resourcePath = (workspace: string, id: string) => `${workspacePath(workspace)}/resource/${encodeURIComponent(id)}`;
export const collectionPath = (workspace: string, kind: string) => `${workspacePath(workspace)}/resources/${encodeURIComponent(kind)}`;
export function useResources(workspace: string, kind: string, interval?: number) {
  const query = useInfiniteQuery({
    queryKey: [workspace, 'resources', kind], enabled: !!workspace,
    initialPageParam: 0,
    queryFn: async ({ pageParam, signal }) => ({ items: items(await api<{ items: Resource[] }>(`${collectionPath(workspace, kind)}?limit=${RESOURCE_PAGE_SIZE}&offset=${pageParam}`, { signal })) }),
    getNextPageParam: (lastPage, _pages, lastOffset) => nextResourceOffset(lastPage, lastOffset),
    refetchInterval: interval,
  });
  return { ...query, data: query.data ? mergeResourcePages(query.data.pages) : undefined };
}
export const useResource = (workspace: string, id: string | null, interval?: number, metadataOnly = false) => useQuery({
  queryKey: [workspace, 'resource', id], enabled: !!workspace && !!id,
  queryFn: async ({ signal }) => unwrap(await api<Resource>(`${resourcePath(workspace, id!)}${metadataOnly ? '?include_messages=false' : ''}`, { signal })),
  refetchInterval: interval,
});

export const useMessages = (workspace: string, id: string | null) => useInfiniteQuery({
  queryKey: [workspace, 'messages', id], enabled: !!workspace && !!id,
  initialPageParam: undefined as number | undefined,
  queryFn: ({ pageParam, signal }) => api<{ items: Message[]; next_before: number | null; total: number }>(`${workspacePath(workspace)}/conversations/${encodeURIComponent(id!)}/messages?limit=50${pageParam === undefined ? '' : `&before=${pageParam}`}`, { signal }),
  getNextPageParam: page => page.next_before ?? undefined,
});

export function safePreference(key: string, fallback = '') {
  try { return localStorage.getItem(`rstrbot:${key}`) ?? fallback; } catch { return fallback; }
}
export function savePreference(key: string, value: string) {
  try { localStorage.setItem(`rstrbot:${key}`, value); } catch { /* Private browsing can disable storage. The current session still works. */ }
}
