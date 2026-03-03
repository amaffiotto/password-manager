/* service-worker.js — Background service worker for native messaging bridge */

const NATIVE_HOST = 'com.passwordmanager.app';

let port = null;
let pendingRequests = new Map();
let requestId = 0;

/**
 * Connect (or reconnect) to the native messaging host.
 */
function connectNative() {
  if (port) return port;

  port = chrome.runtime.connectNative(NATIVE_HOST);

  port.onMessage.addListener((message) => {
    // Route response back to the pending request
    if (message._requestId != null && pendingRequests.has(message._requestId)) {
      const { resolve } = pendingRequests.get(message._requestId);
      pendingRequests.delete(message._requestId);
      delete message._requestId;
      resolve(message);
    }
  });

  port.onDisconnect.addListener(() => {
    const error = chrome.runtime.lastError
      ? chrome.runtime.lastError.message
      : 'Native host disconnected';

    // Reject all pending requests
    for (const [, { reject }] of pendingRequests) {
      reject(new Error(error));
    }
    pendingRequests.clear();
    port = null;
  });

  return port;
}

/**
 * Send a message to the native host and wait for a response.
 */
function sendToNative(payload) {
  return new Promise((resolve, reject) => {
    try {
      const p = connectNative();
      const id = ++requestId;

      pendingRequests.set(id, { resolve, reject });

      p.postMessage({ ...payload, _requestId: id });

      // Timeout after 10 seconds
      setTimeout(() => {
        if (pendingRequests.has(id)) {
          pendingRequests.delete(id);
          reject(new Error('Request timed out'));
        }
      }, 10000);
    } catch (err) {
      reject(err);
    }
  });
}

/**
 * Listen for messages from popup or content scripts.
 */
chrome.runtime.onMessage.addListener((message, _sender, sendResponse) => {
  if (message.type === 'native-request') {
    sendToNative(message.payload)
      .then((response) => sendResponse(response))
      .catch((err) => sendResponse({ error: err.message }));

    // Return true to indicate async sendResponse
    return true;
  }
});
