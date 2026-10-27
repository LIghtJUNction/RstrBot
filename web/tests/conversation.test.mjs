import test from 'node:test';
import assert from 'node:assert/strict';
import { normalizeMessages, nextCursor, acknowledgeDraft, applyEventPage, emptyStream, mergeResourcePages, nextResourceOffset, RESOURCE_PAGE_SIZE } from '../src/conversation.ts';

test('malformed records cannot prevent valid history from rendering', () => {
  const messages = normalizeMessages([null, false, {}, { id: '1', role: 'user', content: 'hello' }, { id: '2', role: 'assistant', content: [{ type: 'text', text: 'world' }] }]);
  assert.equal(messages.length, 2);
  assert.equal(messages[0].content, 'hello');
  assert.match(messages[1].content, /world/);
});

test('duplicate stable message IDs are rendered only once', () => {
  assert.equal(normalizeMessages([{ id: '1', role: 'user', content: 'a' }, { id: '1', role: 'user', content: 'a' }]).length, 1);
});

test('raw model text remains text, including HTML payloads', () => {
  const content = '<img src=x onerror=alert(1)>';
  assert.equal(normalizeMessages([{ role: 'assistant', content }])[0].content, content);
});

test('a delayed or invalid cursor never rolls event replay backwards', () => {
  assert.equal(nextCursor(100, 30, [45, 90]), 100);
  assert.equal(nextCursor(100, undefined, [101, 102]), 102);
  assert.equal(nextCursor(100, NaN, [101]), 101);
  assert.equal(nextCursor(100, 120, [110]), 120);
});

test('reconnect overlap and terminal receipts do not repeat or retain generated text', () => {
  const delta = (id, run, text) => ({ id, kind: 'run.delta', data: { run_id: run, delta: text } });
  let { state } = applyEventPage(emptyStream(), [delta(1, 'a', 'first'), delta(2, 'a', ' second')], 2);
  assert.equal(state.live.a, 'first second');
  // A reconnect overlaps the last event; a terminal receipt arrives in the same page.
  state = applyEventPage(state, [delta(2, 'a', ' second'), { id: 3, kind: 'run.finished', data: { run_id: 'a', status: 'completed' } }], 3).state;
  assert.deepEqual(state.live, {});
  // A late producer must not resurrect a finished run even with a newer event ID.
  state = applyEventPage(state, [delta(4, 'a', 'late')], 4).state;
  assert.deepEqual(state.live, {});
  assert.equal(state.cursor, 4);
});

test('historical replay releases terminal streams and bounds missing-terminal buffers', () => {
  let state = emptyStream();
  for (let i = 0; i < 1000; i++) {
    state = applyEventPage(state, [
      { id: i * 2 + 1, kind: 'run.delta', data: { run_id: String(i), delta: 'content' } },
      { id: i * 2 + 2, kind: 'run.finished', data: { run_id: String(i), status: i % 2 ? 'failed' : 'completed' } },
    ], i * 2 + 2).state;
  }
  assert.deepEqual(state.live, {});
  assert.ok(state.finished.length <= 256);
  state = applyEventPage(state, Array.from({ length: 30 }, (_, i) => ({ id: 2001 + i, kind: 'run.delta', data: { run_id: `orphan-${i}`, delta: 'x'.repeat(300000) } })), 2030).state;
  assert.ok(Object.keys(state.live).length <= 16);
  assert.ok(Object.values(state.live).every(value => value.length <= 200000));
});

test('unordered duplicate delivery is consumed in ID order and cannot revive terminal text', () => {
  const result = applyEventPage(emptyStream(), [
    { id: 3, kind: 'run.finished', data: { run_id: 'a', status: 'completed' } },
    { id: 2, kind: 'run.delta', data: { run_id: 'a', delta: 'two' } },
    { id: 1, kind: 'run.delta', data: { run_id: 'a', delta: 'one' } },
    { id: 2, kind: 'run.delta', data: { run_id: 'a', delta: 'two' } },
  ], 1);
  assert.equal(result.state.cursor, 3);
  assert.deepEqual(result.state.live, {});
  assert.equal(result.changed, true);
});

test('a delayed send acknowledgement preserves text entered while the request was pending', async () => {
  let acknowledge;
  const network = new Promise(resolve => { acknowledge = resolve; });
  let draft = { text: 'first message', revision: 1 };
  const submitted = draft.revision;
  const request = network.then(() => acknowledgeDraft(draft, submitted));
  draft = { text: 'next message', revision: 2 };
  acknowledge();
  assert.equal(await request, 'next message');
  assert.equal(acknowledgeDraft(draft, 2), '');
  // A user may intentionally replace the content with the same text; revision still owns it.
  assert.equal(acknowledgeDraft({ text: 'first message', revision: 3 }, submitted), 'first message');
});

test('pagination reaches resources beyond the old 200-item window despite page overlap', () => {
  const rows = Array.from({ length: 350 }, (_, i) => ({ id: String(349 - i), version: 1 }));
  const pages = [];
  let offset = 0;
  while (offset !== undefined) {
    const page = { items: rows.slice(offset, offset + RESOURCE_PAGE_SIZE) };
    pages.push(page);
    if (offset === 0) rows.unshift({ id: 'new', version: 1 });
    offset = nextResourceOffset(page, offset);
  }
  const resources = mergeResourcePages(pages);
  assert.equal(resources.length, 350);
  assert.equal(new Set(resources.map(resource => resource.id)).size, resources.length);
  assert.equal(resources.at(-1).id, '0');
});

test('an overlapping resource page keeps the newest resource revision', () => {
  const records = mergeResourcePages([{ items: [{ id: 'a', version: 2 }, { id: 'b', version: 1 }] }, { items: [{ id: 'a', version: 3 }, { id: 'b', version: 0 }] }]);
  assert.deepEqual(records, [{ id: 'a', version: 3 }, { id: 'b', version: 1 }]);
});
