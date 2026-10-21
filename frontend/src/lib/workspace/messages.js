/**
 * Pure helpers for the conversation workspace. The API paginates threads
 * newest-first, so every page is normalised to chronological order before it
 * is merged into the rendered thread.
 */

import { parseTimestamp } from '../format.js';

export const PAGE_SIZE = 30;

/** @param {string | number | Date | null | undefined} value */
function messageTime(value) {
  return parseTimestamp(value)?.getTime() ?? 0;
}

/**
 * @param {Record<string, any>} a
 * @param {Record<string, any>} b
 */
function compareMessages(a, b) {
  const delta = messageTime(a?.timestamp) - messageTime(b?.timestamp);
  if (delta !== 0) return delta;
  const aId = Number(a?.id);
  const bId = Number(b?.id);
  if (Number.isFinite(aId) && Number.isFinite(bId)) return aId - bId;
  return 0;
}

/**
 * Normalises one `/api/sms` page into oldest-first order.
 * @param {{ data?: Array<Record<string, any>> } | null | undefined} payload
 */
export function normalizePage(payload) {
  const data = Array.isArray(payload?.data) ? payload.data : [];
  return data
    .map((message) => ({ ...message, status: Number(message?.status ?? 0) }))
    .sort(compareMessages);
}

/**
 * Prepends older messages that are not already loaded.
 * @param {Array<Record<string, any>>} existing
 * @param {Array<Record<string, any>>} older
 */
export function mergeOlderMessages(existing = [], older = []) {
  const known = new Set(existing.map((message) => String(message?.id)));
  const added = older.filter((message) => !known.has(String(message?.id)));
  return [...added, ...existing];
}

/**
 * Merges partial conversation updates (the SSE stream only carries the
 * conversations that just archived an SMS) into the full sidebar list.
 * @param {Array<Record<string, any>>} existing
 * @param {Array<Record<string, any>>} incoming
 */
export function mergeConversations(existing = [], incoming = []) {
  if (!Array.isArray(incoming)) return existing;
  const byContact = new Map();
  for (const conversation of existing) {
    if (conversation?.contact?.id) byContact.set(conversation.contact.id, conversation);
  }
  for (const conversation of incoming) {
    if (conversation?.contact?.id) byContact.set(conversation.contact.id, conversation);
  }
  return [...byContact.values()].sort(
    (a, b) => messageTime(b?.sms_preview?.timestamp) - messageTime(a?.sms_preview?.timestamp)
  );
}

/**
 * Replaces the newest page while keeping older loaded pages and in-flight
 * optimistic messages.
 * @param {Array<Record<string, any>>} existing
 * @param {Array<Record<string, any>>} newest chronological page 1
 */
export function mergeNewestMessages(existing = [], newest = []) {
  if (!newest.length) return existing;
  const known = new Set(newest.map((message) => String(message?.id)));
  const oldestIncoming = messageTime(newest[0]?.timestamp);
  const kept = existing.filter((message) => {
    if (known.has(String(message?.id))) return false;
    if (isLocalMessage(message)) return true;
    return messageTime(message?.timestamp) < oldestIncoming;
  });
  return [...kept, ...newest];
}

/** @param {Record<string, any> | null | undefined} message */
export function isLocalMessage(message) {
  return typeof message?.id === 'string' && message.id.startsWith('local-');
}

/**
 * Keeps the requested line when it still exists, otherwise falls back to the
 * first registered line (or the requested id when card data is unavailable).
 * @param {Array<{ id?: string }>} simCards
 * @param {string | null | undefined} preferred
 */
export function resolveSimId(simCards = [], preferred = '') {
  const ids = simCards.map((card) => card?.id).filter(Boolean);
  if (preferred && ids.includes(preferred)) return preferred;
  if (ids.length) return ids[0];
  return preferred || '';
}

/** @param {number} value */
function pad(value) {
  return String(value).padStart(2, '0');
}

/** @param {Date} [date] */
function naiveTimestamp(date = new Date()) {
  return [
    date.getFullYear(),
    '-',
    pad(date.getMonth() + 1),
    '-',
    pad(date.getDate()),
    'T',
    pad(date.getHours()),
    ':',
    pad(date.getMinutes()),
    ':',
    pad(date.getSeconds())
  ].join('');
}

/**
 * @param {{ contactId: string, simId: string, message: string }} payload
 */
export function createOutgoingMessage({ contactId, simId, message }) {
  return {
    id: `local-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
    contact_id: contactId,
    timestamp: naiveTimestamp(),
    message,
    sim_id: simId,
    send: true,
    status: 2
  };
}

/** Contact ids for threads the gateway has never seen. */
export function uniqueContactId() {
  if (typeof crypto !== 'undefined' && typeof crypto.randomUUID === 'function') {
    return crypto.randomUUID();
  }
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, (char) => {
    const random = Math.floor(Math.random() * 16);
    const value = char === 'x' ? random : (random & 0x3) | 0x8;
    return value.toString(16);
  });
}
