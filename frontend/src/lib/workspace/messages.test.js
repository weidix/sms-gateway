import assert from 'node:assert/strict';
import test from 'node:test';
import { mergeConversations } from './messages.js';

/** @param {string} id @param {string} name @param {string} timestamp */
function conversation(id, name, timestamp) {
  return {
    contact: { id, name },
    sms_preview: { message: `${name} preview`, timestamp, status: 1, sim_id: 'sim-1' }
  };
}

test('a partial SSE update keeps every other sidebar conversation', () => {
  const existing = [
    conversation('a', 'Alice', '2026-10-21T10:00:00'),
    conversation('b', 'Bob', '2026-10-21T11:00:00'),
    conversation('c', 'Carol', '2026-10-21T09:00:00')
  ];
  const incoming = [conversation('a', 'Alice', '2026-10-21T12:00:00')];

  const merged = mergeConversations(existing, incoming);

  assert.deepEqual(
    merged.map((item) => item.contact.name),
    ['Alice', 'Bob', 'Carol']
  );
  assert.equal(merged[0].sms_preview.timestamp, '2026-10-21T12:00:00');
});

test('a partial SSE update adds a brand-new sender once', () => {
  const existing = [conversation('a', 'Alice', '2026-10-21T10:00:00')];
  const incoming = [
    conversation('d', 'Dave', '2026-10-21T13:00:00'),
    conversation('d', 'Dave', '2026-10-21T13:00:00')
  ];

  const merged = mergeConversations(existing, incoming);

  assert.equal(merged.length, 2);
  assert.deepEqual(
    merged.map((item) => item.contact.name),
    ['Dave', 'Alice']
  );
});

test('an empty update leaves the list untouched', () => {
  const existing = [conversation('a', 'Alice', '2026-10-21T10:00:00')];
  assert.equal(mergeConversations(existing, []).length, 1);
});
