/* popup.js — Popup logic for the Pvault extension */

const NATIVE_HOST = 'com.passwordmanager.app';

let passwordsVisible = false;
let currentEntries = [];

const views = {
  loading: document.getElementById('view-loading'),
  disconnected: document.getElementById('view-disconnected'),
  locked: document.getElementById('view-locked'),
  empty: document.getElementById('view-empty'),
  credentials: document.getElementById('view-credentials'),
};

function showView(name) {
  Object.values(views).forEach((v) => v.classList.add('hidden'));
  views[name].classList.remove('hidden');
}

// --- Native messaging helpers ---

function sendNativeMessage(message) {
  return new Promise((resolve, reject) => {
    chrome.runtime.sendMessage(
      { type: 'native-request', payload: message },
      (response) => {
        if (chrome.runtime.lastError) {
          reject(new Error(chrome.runtime.lastError.message));
        } else if (response && response.error) {
          reject(new Error(response.error));
        } else {
          resolve(response);
        }
      }
    );
  });
}

// --- Extract hostname from the active tab ---

async function getCurrentTabHostname() {
  const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
  if (!tab || !tab.url) return null;
  try {
    const url = new URL(tab.url);
    return url.hostname;
  } catch {
    return null;
  }
}

// --- Autofill: send credentials to content script ---

async function autofill(entryId) {
  try {
    const result = await sendNativeMessage({
      action: 'get-decrypted-password',
      entryId,
    });

    const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
    if (!tab) return;

    chrome.tabs.sendMessage(tab.id, {
      type: 'autofill',
      username: result.username,
      password: result.password,
    });

    window.close();
  } catch (err) {
    console.error('Autofill failed:', err);
  }
}

// --- Show/hide all passwords ---

async function togglePasswords() {
  const btn = document.getElementById('btn-show-passwords');

  if (passwordsVisible) {
    passwordsVisible = false;
    btn.textContent = 'Show All';
    document.querySelectorAll('.credential-password').forEach((el) => {
      el.className = 'credential-password-dots';
      el.textContent = '\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022';
    });
    return;
  }

  btn.textContent = 'Loading...';

  try {
    for (const entry of currentEntries) {
      try {
        const result = await sendNativeMessage({
          action: 'get-decrypted-password',
          entryId: entry.id,
        });
        const el = document.querySelector(`[data-pw-id="${entry.id}"]`);
        if (el) {
          el.className = 'credential-password';
          el.textContent = result.password;
        }
      } catch {
        const el = document.querySelector(`[data-pw-id="${entry.id}"]`);
        if (el) {
          el.className = 'credential-password';
          el.textContent = '(failed to decrypt)';
        }
      }
    }
    passwordsVisible = true;
    btn.textContent = 'Hide All';
  } catch {
    btn.textContent = 'Show All';
  }
}

// --- Build the credentials list UI ---

function renderCredentials(entries, hostname) {
  currentEntries = entries;
  const list = document.getElementById('credentials-list');
  list.innerHTML = '';

  entries.forEach((entry) => {
    const li = document.createElement('li');
    li.className = 'credential-item';
    li.innerHTML = `
      <div class="credential-info">
        <div class="credential-site">${escapeHtml(entry.site_name)}</div>
        <div class="credential-user">${escapeHtml(entry.username)}</div>
        <div class="credential-password-dots" data-pw-id="${entry.id}">\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022</div>
      </div>
      <div class="credential-actions">
        <button class="btn btn-fill" data-id="${entry.id}">Autofill</button>
      </div>
    `;
    list.appendChild(li);
  });

  list.querySelectorAll('[data-id]').forEach((btn) => {
    btn.addEventListener('click', () => autofill(Number(btn.dataset.id)));
  });

  document.getElementById('site-name').textContent = hostname;
  showView('credentials');
}

function escapeHtml(str) {
  const div = document.createElement('div');
  div.textContent = str;
  return div.innerHTML;
}

// --- Main init ---

async function init() {
  showView('loading');
  passwordsVisible = false;

  const hostname = await getCurrentTabHostname();

  try {
    const status = await sendNativeMessage({ action: 'status' });

    if (!status.unlocked) {
      showView('locked');
      return;
    }

    if (!hostname) {
      showView('empty');
      return;
    }

    const result = await sendNativeMessage({
      action: 'search-entries',
      query: hostname,
    });

    const entries = result.entries || [];

    if (entries.length === 0) {
      document.getElementById('current-site-hint').textContent = hostname;
      showView('empty');
    } else {
      renderCredentials(entries, hostname);
    }
  } catch (err) {
    console.error('Connection error:', err);
    showView('disconnected');
  }
}

// Event listeners
document.getElementById('btn-retry').addEventListener('click', init);
document.getElementById('btn-show-passwords').addEventListener('click', togglePasswords);

// Start
init();
