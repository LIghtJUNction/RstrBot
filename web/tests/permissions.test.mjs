import test from 'node:test';
import assert from 'node:assert/strict';
import { canCreateResource, canWriteResource, roleAllows } from '../src/permissions.ts';

const alice = { id: 'alice', username: 'Alice' };
const resource = (kind, owner_id = 'alice', visibility = 'workspace') => ({ kind, owner_id, visibility });

test('instance administration does not imply workspace write permission', () => {
  const admin = { ...alice, instance_admin: true };
  assert.equal(canCreateResource('member', 'models', admin), false);
  assert.equal(canCreateResource('viewer', 'knowledge', admin), false);
  assert.equal(canCreateResource('admin', 'models', admin), true);
  assert.equal(canCreateResource('member', 'routes', admin), false);
  assert.equal(canCreateResource('admin', 'routes', admin), true);
  assert.equal(canCreateResource('owner', 'routes', alice), false);
});

test('shared resources are writable by their member owner or an operator', () => {
  assert.equal(canWriteResource('member', alice, resource('conversations')), true);
  assert.equal(canWriteResource('member', alice, resource('conversations', 'bob')), false);
  assert.equal(canWriteResource('operator', alice, resource('conversations', 'bob')), true);
  assert.equal(canWriteResource('operator', alice, resource('knowledge', 'bob')), true);
  assert.equal(canWriteResource('operator', alice, resource('models', 'bob')), false);
});

test('private ownership and unknown roles fail closed', () => {
  assert.equal(canWriteResource('owner', alice, resource('memories', 'bob', 'private')), false);
  assert.equal(canWriteResource('unknown', alice, resource('memories')), false);
  assert.equal(roleAllows('owner', 'unknown'), false);
  assert.equal(canWriteResource('viewer', alice, resource('conversations')), false);
});

test('workspace administrators can moderate approved routes without approving new destinations', () => {
  assert.equal(canCreateResource('admin', 'routes', alice), false);
  assert.equal(canWriteResource('admin', alice, resource('routes', 'bob')), true);
  assert.equal(canWriteResource('operator', alice, resource('routes', 'bob')), false);
  assert.equal(canWriteResource('admin', alice, resource('routes', 'bob', 'private')), false);
});
