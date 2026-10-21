let sequence = 0;
let items = $state([]);
const timers = new Map();

/** @typedef {{ id: number, tone: 'ok' | 'warn' | 'danger' | 'neutral', title: string, message: string }} Toast */

export const toast = {
  get items() {
    return items;
  },

  /**
   * @param {string} message
   * @param {{ tone?: 'ok' | 'warn' | 'danger' | 'neutral', title?: string }} [options]
   */
  push(message, options = {}) {
    const id = ++sequence;
    /** @type {Toast} */
    const entry = {
      id,
      tone: options.tone ?? 'neutral',
      title: options.title ?? '',
      message
    };
    items = [...items, entry];
    timers.set(
      id,
      setTimeout(() => toast.dismiss(id), 4800)
    );
    return id;
  },

  /** @param {number} id */
  dismiss(id) {
    const timer = timers.get(id);
    if (timer) {
      clearTimeout(timer);
      timers.delete(id);
    }
    items = items.filter((item) => item.id !== id);
  }
};

/** @param {string} message */
export const toastOk = (message) => toast.push(message, { tone: 'ok', title: 'Done' });
/** @param {string} message */
export const toastWarn = (message) => toast.push(message, { tone: 'warn', title: 'Check' });
/** @param {string} message */
export const toastError = (message) => toast.push(message, { tone: 'danger', title: 'Failed' });
