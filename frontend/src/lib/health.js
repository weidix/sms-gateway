/** Health vocabulary shared by the fleet overview and SIM detail screens. */

/** @typedef {'ok' | 'warn' | 'danger' | 'idle'} Tone */

/** @type {Record<string, { label: string, tone: Tone }>} */
export const HEALTH_STATES = {
  healthy: { label: 'Healthy', tone: 'ok' },
  degraded: { label: 'Degraded', tone: 'warn' },
  recovering: { label: 'Recovering', tone: 'warn' },
  critical: { label: 'Critical', tone: 'danger' }
};

/** @type {Record<string, string>} */
export const FAILURE_REASONS = {
  at_unreachable: 'AT unreachable',
  sim_not_ready: 'SIM not ready',
  network_not_registered: 'Network not registered',
  sms_storage_unavailable: 'SMS storage unavailable',
  sms_storage_full: 'SMS storage full',
  read_sms_failed: 'Reading SMS failed'
};

/** @type {Record<string, string>} */
export const RECOVERY_ACTIONS = {
  reinitialize_modem: 'Reinitialised modem',
  reapply_sms_storage: 'Reapplied SMS storage',
  restart_modem: 'Restarted modem'
};

/**
 * @param {string | null | undefined} status
 * @returns {{ label: string, tone: Tone }}
 */
export function healthState(status) {
  if (!status) return { label: 'Not probed', tone: 'idle' };
  return HEALTH_STATES[status] ?? { label: status, tone: 'idle' };
}

/**
 * @param {string[] | null | undefined} reasons
 * @returns {string[]}
 */
export function failureReasonLabels(reasons) {
  if (!reasons?.length) return [];
  return reasons.map((reason) => FAILURE_REASONS[reason] ?? reason);
}

/**
 * @param {string | null | undefined} action
 */
export function recoveryActionLabel(action) {
  if (!action) return '—';
  return RECOVERY_ACTIONS[action] ?? action;
}
