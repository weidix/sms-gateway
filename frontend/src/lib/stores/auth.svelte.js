import {
  api,
  clearStoredAuth,
  readStoredAuth,
  saveStoredAuth
} from '../api.js';

let token = $state('');
let username = $state('');
let remember = $state(false);
let loginRequired = $state(false);
let ready = $state(false);
let busy = $state(false);
let sessionValid = $state(false);

/** @param {string} user @param {string} password */
export function basicToken(user, password) {
  const bytes = new TextEncoder().encode(`${user}:${password}`);
  let binary = '';
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

export const auth = {
  get ready() {
    return ready;
  },
  get busy() {
    return busy;
  },
  get loginRequired() {
    return loginRequired;
  },
  get sessionValid() {
    return sessionValid;
  },
  get username() {
    return username;
  },
  get token() {
    return token;
  },

  /** Restores the session once per page load. */
  async init() {
    if (ready) return;
    try {
      const status = await api.authStatus();
      loginRequired = Boolean(status?.login_required);
    } catch {
      loginRequired = true;
    }

    const stored = readStoredAuth();
    if (!loginRequired) {
      token = stored?.token ?? '';
      username = stored?.username ?? '';
      sessionValid = true;
      ready = true;
      return;
    }

    if (stored?.token) {
      const valid = await api.check(stored.token).catch(() => false);
      if (valid) {
        token = stored.token;
        username = stored.username ?? '';
        remember = Boolean(localStorage.getItem('sms-gateway.auth'));
        sessionValid = true;
      } else {
        clearStoredAuth();
      }
    }

    ready = true;
  },

  /**
   * @param {string} user
   * @param {string} password
   * @param {boolean} keepSignedIn
   */
  async signIn(user, password, keepSignedIn) {
    busy = true;
    try {
      const candidate = basicToken(user, password);
      const valid = await api.check(candidate);
      if (!valid) throw new Error('Incorrect username or password.');
      token = candidate;
      username = user;
      remember = keepSignedIn;
      saveStoredAuth({ token: candidate, username: user }, keepSignedIn);
      sessionValid = true;
    } finally {
      busy = false;
    }
  },

  signOut() {
    token = '';
    username = '';
    sessionValid = false;
    clearStoredAuth();
    window.location.assign('/login');
  }
};
