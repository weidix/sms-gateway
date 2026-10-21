/**
 * HTTP client for the gateway API.
 *
 * Requests are same-origin. Credentials live in sessionStorage (or
 * localStorage when "keep me signed in" is checked) as the base64 Basic token
 * the Rust server expects, so the client can authenticate every call,
 * including the SSE stream.
 */

const STORAGE_KEY = 'sms-gateway.auth';

export class ApiError extends Error {
  /**
   * @param {number} status
   * @param {string} message
   * @param {any} [data]
   */
  constructor(status, message, data = null) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.data = data;
  }
}

/** @returns {{ token: string, username: string } | null} */
export function readStoredAuth() {
  if (typeof window === 'undefined') return null;
  for (const storage of [window.sessionStorage, window.localStorage]) {
    try {
      const raw = storage.getItem(STORAGE_KEY);
      if (!raw) continue;
      const parsed = JSON.parse(raw);
      if (parsed?.token) return parsed;
    } catch {
      // Ignore malformed storage entries.
    }
  }
  return null;
}

/** @param {{ token: string, username: string }} entry @param {boolean} remember */
export function saveStoredAuth(entry, remember) {
  if (typeof window === 'undefined') return;
  const serialized = JSON.stringify(entry);
  window.sessionStorage.setItem(STORAGE_KEY, serialized);
  if (remember) {
    window.localStorage.setItem(STORAGE_KEY, serialized);
  } else {
    window.localStorage.removeItem(STORAGE_KEY);
  }
}

export function clearStoredAuth() {
  if (typeof window === 'undefined') return;
  window.sessionStorage.removeItem(STORAGE_KEY);
  window.localStorage.removeItem(STORAGE_KEY);
}

/** @param {Record<string, string | number | boolean | null | undefined>} params */
function queryString(params) {
  const pairs = Object.entries(params)
    .filter(([, value]) => value !== undefined && value !== null && value !== '')
    .map(([key, value]) => `${encodeURIComponent(key)}=${encodeURIComponent(String(value))}`);
  return pairs.length ? `?${pairs.join('&')}` : '';
}

/**
 * @param {string} path
 * @param {{
 *   method?: string,
 *   body?: any,
 *   query?: Record<string, any>,
 *   signal?: AbortSignal,
 *   token?: string | null,
 *   silent401?: boolean
 * }} [options]
 */
async function request(path, options = {}) {
  const { method = 'GET', body, query, signal, token, silent401 = false } = options;
  /** @type {Record<string, string>} */
  const headers = {};
  if (body !== undefined) headers['Content-Type'] = 'application/json';

  const authToken = token ?? readStoredAuth()?.token ?? '';
  if (authToken) headers.Authorization = `Basic ${authToken}`;

  let response;
  try {
    response = await fetch(path + (query ? queryString(query) : ''), {
      method,
      headers,
      body: body === undefined ? undefined : JSON.stringify(body),
      credentials: 'same-origin',
      signal
    });
  } catch (error) {
    if (error?.name === 'AbortError') throw error;
    throw new ApiError(0, 'The gateway did not respond. Check that the service is running.');
  }

  const text = await response.text();
  let data = null;
  if (text) {
    try {
      data = JSON.parse(text);
    } catch {
      data = text;
    }
  }

  if (!response.ok) {
    if (response.status === 401 && !silent401) {
      clearStoredAuth();
      if (window.location.pathname !== '/login') window.location.assign('/login');
    }
    const message =
      typeof data === 'string' && data.trim()
        ? data.trim()
        : `Request failed with status ${response.status}.`;
    throw new ApiError(response.status, message, data);
  }

  return data;
}

/** @param {string} data */
function parseSseEvent(data) {
  let event = 'message';
  const payload = [];
  for (const line of data.split('\n')) {
    if (line.startsWith(':')) continue;
    if (line.startsWith('event:')) event = line.slice(6).trim();
    else if (line.startsWith('data:')) payload.push(line.slice(5).trimStart());
  }
  return { event, data: payload.join('\n') };
}

export const api = {
  /** Public authentication mode probe. */
  authStatus: () => request('/api/auth/status', { silent401: true }),

  /**
   * Validates a Basic token against the protected API.
   * @param {string} [token]
   */
  async check(token) {
    try {
      await request('/api/check', { token, silent401: true });
      return true;
    } catch (error) {
      if (error instanceof ApiError && error.status === 401) return false;
      throw error;
    }
  },

  /** @param {AbortSignal} [signal] */
  conversations: (signal) => request('/api/conversation', { signal }),

  /**
   * @param {{ page?: number, perPage?: number, contactId?: string | null, signal?: AbortSignal }} [options]
   */
  smsPage: (options = {}) => {
    const { page = 1, perPage = 30, contactId = null, signal } = options;
    return request('/api/sms', {
      query: { page, per_page: perPage, contact_id: contactId },
      signal
    });
  },

  /**
   * @param {{ simId: string, contact: { id: string, name: string }, message: string, isNew: boolean }} payload
   */
  sendSms: ({ simId, contact, message, isNew }) =>
    request('/api/sms', {
      method: 'POST',
      body: { sim_id: simId, contact, message, new: isNew }
    }),

  /** Deletes a conversation and its message history from the database. */
  deleteConversation: (contactId) =>
    request(`/api/contacts/${encodeURIComponent(contactId)}`, { method: 'DELETE' }),

  /** @param {AbortSignal} [signal] */
  simCards: (signal) => request('/api/sim-cards', { signal }),

  /** @param {AbortSignal} [signal] */
  simsInfo: (signal) => request('/api/sims/info', { signal }),

  /** @param {string} simId @param {AbortSignal} [signal] */
  simInfo: (simId, signal) => request(`/api/sims/${encodeURIComponent(simId)}/info`, { signal }),

  /** Imports and cleans up device SMS for one SIM. */
  refreshSim: (simId) => request(`/api/sims/${encodeURIComponent(simId)}/refresh`),

  /** @param {string} simId @param {string | null} alias */
  updateAlias: (simId, alias) =>
    request(`/api/sim-cards/${encodeURIComponent(simId)}/alias`, {
      method: 'PUT',
      body: { alias }
    }),

  /** @param {string} simId @param {string | null} phoneNumber */
  updatePhone: (simId, phoneNumber) =>
    request(`/api/sim-cards/${encodeURIComponent(simId)}/phone`, {
      method: 'PUT',
      body: { phone_number: phoneNumber }
    }),

  /** @param {string} simId @param {AbortSignal} [signal] */
  storageStatus: (simId, signal) =>
    request(`/api/sims/${encodeURIComponent(simId)}/storage`, { signal }),

  /** @param {string} simId @param {'SIM' | 'ME' | 'MT'} storage */
  setStorage: (simId, storage) =>
    request(`/api/sims/${encodeURIComponent(simId)}/storage`, {
      method: 'PUT',
      body: { storage }
    }),

  /** @param {string} simId @param {string} command */
  runAt: (simId, command) =>
    request(`/api/sims/${encodeURIComponent(simId)}/at`, {
      method: 'POST',
      body: { command }
    })
};

/**
 * Subscribes to the conversation SSE stream with the authenticated fetch API
 * (native EventSource cannot carry an Authorization header).
 *
 * @param {{
 *   onConversations?: (conversations: any[]) => void,
 *   onStatus?: (status: 'open' | 'closed' | 'error') => void
 * }} handlers
 * @returns {() => void} unsubscribe
 */
export function openConversationStream(handlers = {}) {
  const controller = new AbortController();
  let closed = false;

  const run = async () => {
    while (!closed) {
      try {
        const token = readStoredAuth()?.token ?? '';
        const response = await fetch('/api/sms/sse', {
          headers: token ? { Authorization: `Basic ${token}` } : {},
          credentials: 'same-origin',
          signal: controller.signal
        });
        if (!response.ok || !response.body) {
          throw new ApiError(response.status, 'Live updates are unavailable.');
        }

        handlers.onStatus?.('open');
        const reader = response.body.getReader();
        const decoder = new TextDecoder();
        let buffer = '';

        while (!closed) {
          const { value, done } = await reader.read();
          if (done) break;
          buffer += decoder.decode(value, { stream: true });

          let boundary = buffer.indexOf('\n\n');
          while (boundary !== -1) {
            const chunk = buffer.slice(0, boundary);
            buffer = buffer.slice(boundary + 2);
            const event = parseSseEvent(chunk);
            if (event.event === 'conversations' && event.data) {
              try {
                handlers.onConversations?.(JSON.parse(event.data));
              } catch {
                // Ignore malformed frames; the stream stays alive.
              }
            }
            boundary = buffer.indexOf('\n\n');
          }
        }
      } catch (error) {
        if (closed || error?.name === 'AbortError') break;
        handlers.onStatus?.('error');
      }

      if (closed) break;
      await new Promise((resolve) => setTimeout(resolve, 5000));
    }
    if (!closed) handlers.onStatus?.('closed');
  };

  run();

  return () => {
    closed = true;
    controller.abort();
  };
}
