/* popup.js — Popup logic for the Password Manager extension */

const NATIVE_HOST = 'com.passwordmanager.app';

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

    // Close the popup after autofill
    window.close();
  } catch (err) {
    console.error('Autofill failed:', err);
  }
}

// --- Build the credentials list UI ---

function renderCredentials(entries, hostname) {
  const list = document.getElementById('credentials-list');
  list.innerHTML = '';

  entries.forEach((entry) => {
    const li = document.createElement('li');
    li.className = 'credential-item';
    li.innerHTML = `
      <div class="credential-info">
        <div class="credential-site">${escapeHtml(entry.site_name)}</div>
        <div class="credential-user">${escapeHtml(entry.username)}</div>
      </div>
      <div class="credential-actions">
        <button class="btn btn-fill" data-id="${entry.id}">Autofill</button>
      </div>
    `;
    list.appendChild(li);
  });

  // Attach click handlers
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

  const hostname = await getCurrentTabHostname();

  try {
    // Ping the desktop app to check connection & vault status
    const status = await sendNativeMessage({ action: 'status' });

    if (!status.unlocked) {
      showView('locked');
      return;
    }

    // Search for entries matching the current site
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

// Retry button
document.getElementById('btn-retry').addEventListener('click', init);

// Start
init();
