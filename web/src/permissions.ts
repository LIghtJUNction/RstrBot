import type { Resource, User } from './api';

// UI affordances mirror the server's membership rules. The API remains the
// authority; instance administration does not grant workspace ownership.
const rank: Record<string, number> = { viewer: 0, member: 1, operator: 2, admin: 3, owner: 4 };
export function roleAllows(role: string, minimum: string): boolean {
  return (rank[role] ?? -1) >= (rank[minimum] ?? Infinity);
}

export function canCreateResource(role: string, kind: string, user: User): boolean {
  const minimum = ['models', 'plugins', 'routes', 'agents'].includes(kind) ? 'admin' : 'member';
  return roleAllows(role, minimum) && (kind !== 'routes' || user.instance_admin === true);
}

export function canWriteResource(role: string, user: User, resource: Resource): boolean {
  // Existing routes can be moderated by a workspace administrator. Approving
  // a new outbound destination remains a separate instance-admin operation.
  const minimum = ['models', 'plugins', 'routes', 'agents'].includes(resource.kind) ? 'admin' : 'member';
  if (!roleAllows(role, minimum)) return false;
  if (resource.visibility === 'private' && resource.owner_id !== user.id) return false;
  return resource.owner_id === user.id || roleAllows(role, 'operator');
}
