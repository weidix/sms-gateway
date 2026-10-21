/**
 * Presentation helpers. Timestamps from the API are naive strings produced by
 * the modem's local clock, so they are parsed as local time.
 */

/** @param {string | number | Date | null | undefined} value */
export function parseTimestamp(value) {
  if (!value) return null;
  if (value instanceof Date) return value;
  const date = new Date(value);
  return Number.isNaN(date.getTime()) ? null : date;
}

/** @param {string | number | Date | null | undefined} value */
export function formatTime(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  return date.toLocaleTimeString('en-GB', { hour: '2-digit', minute: '2-digit' });
}

/** @param {string | number | Date | null | undefined} value */
export function formatDate(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  return date.toLocaleDateString('en-GB', { day: '2-digit', month: 'short', year: 'numeric' });
}

/** @param {string | number | Date | null | undefined} value */
export function formatDateTime(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  return `${formatDate(date)}, ${formatTime(date)}`;
}

/** @param {string | number | Date | null | undefined} value */
export function formatShortDate(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  const now = new Date();
  if (isSameDay(date, now)) return formatTime(date);
  if (isSameDay(date, addDays(now, -1))) return 'Yesterday';
  if (Math.abs(now.getTime() - date.getTime()) < 6 * 86400000) {
    return date.toLocaleDateString('en-GB', { weekday: 'short' });
  }
  return date.toLocaleDateString('en-GB', { day: '2-digit', month: 'short' });
}

/** @param {string | number | Date | null | undefined} value */
export function formatDayLabel(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  const now = new Date();
  if (isSameDay(date, now)) return 'Today';
  if (isSameDay(date, addDays(now, -1))) return 'Yesterday';
  const sameYear = date.getFullYear() === now.getFullYear();
  return date.toLocaleDateString('en-GB', {
    weekday: 'long',
    day: '2-digit',
    month: 'short',
    ...(sameYear ? {} : { year: 'numeric' })
  });
}

/** @param {string | number | Date | null | undefined} value */
export function formatRelative(value) {
  const date = parseTimestamp(value);
  if (!date) return '—';
  const diff = Date.now() - date.getTime();
  if (diff < 45000) return 'just now';
  if (diff < 3600000) return `${Math.round(diff / 60000)}m ago`;
  if (diff < 86400000) return `${Math.round(diff / 3600000)}h ago`;
  if (diff < 7 * 86400000) return `${Math.round(diff / 86400000)}d ago`;
  return formatShortDate(date);
}

/** @param {Date} a @param {Date} b */
export function isSameDay(a, b) {
  return (
    a.getFullYear() === b.getFullYear() &&
    a.getMonth() === b.getMonth() &&
    a.getDate() === b.getDate()
  );
}

/** @param {Date} date @param {number} days */
function addDays(date, days) {
  const next = new Date(date);
  next.setDate(next.getDate() + days);
  return next;
}

/**
 * Groups the oldest-first message list into day sections.
 * @param {Array<Record<string, any>>} messages
 */
export function groupMessagesByDay(messages) {
  /** @type {Array<{ key: string, label: string, items: Array<Record<string, any>> }>} */
  const groups = [];
  for (const message of messages) {
    const date = parseTimestamp(message.timestamp);
    const key = date ? `${date.getFullYear()}-${date.getMonth()}-${date.getDate()}` : 'unknown';
    const last = groups.at(-1);
    if (last && last.key === key) {
      last.items.push(message);
    } else {
      groups.push({ key, label: formatDayLabel(date ?? message.timestamp), items: [message] });
    }
  }
  return groups;
}

/** @param {any} conversation */
export function conversationTimestamp(conversation) {
  return conversation?.sms_preview?.timestamp ?? null;
}


/** @param {number | null | undefined} rssi */
export function signalLabel(rssi) {
  if (!Number.isFinite(rssi) || rssi < 0 || rssi > 31) return 'Unknown';
  if (rssi >= 20) return 'Excellent';
  if (rssi >= 15) return 'Good';
  if (rssi >= 10) return 'Fair';
  if (rssi >= 1) return 'Weak';
  return 'No signal';
}

/** @param {number} value @param {number} total */
export function ratioPercent(value, total) {
  if (!Number.isFinite(value) || !Number.isFinite(total) || total <= 0) return 0;
  return Math.max(0, Math.min(100, Math.round((value / total) * 100)));
}

/**
 * Accepts the two shapes the backend can return for memory/storage status:
 * "Read: 5/100, Write: 2/50, Receive: 7/150" or a raw "+CPMS: ..." line.
 * @param {any} status
 */
export function parseStorageStatus(status) {
  if (!status) return null;
  const text = typeof status === 'string' ? status : '';
  if (!text) return null;

  const named = text.match(
    /Read:\s*(\d+)\/(\d+),\s*Write:\s*(\d+)\/(\d+),\s*Receive:\s*(\d+)\/(\d+)/i
  );
  if (named) {
    return {
      read: { used: Number(named[1]), total: Number(named[2]) },
      write: { used: Number(named[3]), total: Number(named[4]) },
      receive: { used: Number(named[5]), total: Number(named[6]) }
    };
  }

  const cpms = text.match(/"([^"]+)",(\d+),(\d+),"([^"]+)",(\d+),(\d+),"([^"]+)",(\d+),(\d+)/);
  if (cpms) {
    return {
      read: { used: Number(cpms[2]), total: Number(cpms[3]), storage: cpms[1] },
      write: { used: Number(cpms[5]), total: Number(cpms[6]), storage: cpms[4] },
      receive: { used: Number(cpms[8]), total: Number(cpms[9]), storage: cpms[7] }
    };
  }

  return null;
}

/** @param {string | null | undefined} value */
export function displaySimId(value) {
  const text = String(value ?? '');
  if (!text) return '—';
  return text.length > 14 ? `…${text.slice(-12)}` : text;
}

/**
 * @param {{ alias?: string | null, phone_number?: string | null, id?: string } | null | undefined} simCard
 * @param {string | null | undefined} fallbackId
 */
export function simDisplayName(simCard, fallbackId = '') {
  if (simCard?.alias?.trim()) return simCard.alias.trim();
  if (simCard?.phone_number?.trim()) return simCard.phone_number.trim();
  const id = simCard?.id ?? fallbackId ?? '';
  return id ? `SIM-${id.slice(-4)}` : 'SIM';
}
