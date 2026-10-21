/** Quick AT presets and transcript helpers for the line console. */

/** Read-only presets that run as soon as they are clicked. */
export const QUICK_COMMANDS = [
  { command: 'AT', hint: 'Modem presence' },
  { command: 'AT+CSQ', hint: 'Signal quality' },
  { command: 'AT+COPS?', hint: 'Registered operator' },
  { command: 'AT+CPIN?', hint: 'SIM PIN state' },
  { command: 'AT+CREG?', hint: 'Network registration' },
  { command: 'AT+QTEMP?', hint: 'Modem temperature' },
  { command: 'AT+CGSN', hint: 'IMEI serial' },
  { command: 'AT+CGMM', hint: 'Modem model' }
];

/** @type {Record<string, { label: string, tone: 'ok' | 'warn' | 'danger' | 'idle' }>} */
const STATUS_VIEW = {
  ok: { label: 'OK', tone: 'ok' },
  error: { label: 'Error', tone: 'danger' },
  unknown: { label: 'No verdict', tone: 'idle' },
  transport_error: { label: 'Transport', tone: 'warn' }
};

/** @param {string | null | undefined} status */
export function atStatusView(status) {
  return STATUS_VIEW[status] ?? { label: status ? String(status) : 'Unknown', tone: 'idle' };
}

/** @param {number | null | undefined} milliseconds */
export function formatDuration(milliseconds) {
  if (!Number.isFinite(milliseconds)) return '—';
  if (milliseconds < 1000) return `${Math.round(milliseconds)} ms`;
  return `${(milliseconds / 1000).toFixed(2)} s`;
}

/**
 * Transport failures arrive as a non-2xx response whose body still carries an
 * execution record. Treat such a payload as a real transcript entry.
 * @param {any} value
 */
export function isAtExecution(value) {
  return Boolean(
    value &&
      typeof value === 'object' &&
      typeof value.command === 'string' &&
      typeof value.output === 'string' &&
      typeof value.status === 'string' &&
      typeof value.duration_ms === 'number'
  );
}
