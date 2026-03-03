/**
 * local-server.js — Local HTTP server for native messaging bridge.
 *
 * The Electron app runs this server on 127.0.0.1 so the browser extension
 * (via the native messaging host) can query the vault without needing
 * direct database access or the master password.
 */

const http = require('http');
const { app } = require('electron');
const path = require('path');
const fs = require('fs');

let server = null;
let serverPort = null;

// Reference to the functions we need — set from outside via init()
let handlers = null;

/**
 * Initialize the local server with the IPC-like handler functions.
 * @param {object} opts
 * @param {Function} opts.getAllEntries
 * @param {Function} opts.searchEntries
 * @param {Function} opts.getDecryptedPassword
 * @param {Function} opts.isUnlocked - returns true if vault is unlocked
 * @param {Function} opts.getEntryUsername - returns username for entry id
 */
function initLocalServer(opts) {
  handlers = opts;
}

/**
 * Start the local HTTP server on a random port, bound to localhost only.
 * Writes the port number to a file so the native messaging host can find it.
 */
function startLocalServer() {
  if (server) return;

  server = http.createServer(async (req, res) => {
    // Only accept POST on /api
    if (req.method !== 'POST' || req.url !== '/api') {
      res.writeHead(404);
      res.end();
      return;
    }

    let body = '';
    req.on('data', (chunk) => (body += chunk));
    req.on('end', () => {
      try {
        const message = JSON.parse(body);
        const result = handleMessage(message);
        res.writeHead(200, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify(result));
      } catch (err) {
        res.writeHead(500, { 'Content-Type': 'application/json' });
        res.end(JSON.stringify({ error: err.message }));
      }
    });
  });

  // Bind to localhost only — never expose to the network
  server.listen(0, '127.0.0.1', () => {
    serverPort = server.address().port;
    writePortFile(serverPort);
    console.log(`Local server listening on 127.0.0.1:${serverPort}`);
  });

  server.on('error', (err) => {
    console.error('Local server error:', err);
  });
}

/**
 * Handle an incoming message from the native messaging host.
 */
function handleMessage(message) {
  if (!handlers) {
    return { error: 'Server not initialized' };
  }

  const { action } = message;

  switch (action) {
    case 'status':
      return { unlocked: handlers.isUnlocked() };

    case 'search-entries': {
      if (!handlers.isUnlocked()) return { error: 'Vault is locked' };
      const entries = handlers.searchEntries(message.query || '');
      return { entries };
    }

    case 'get-all-entries': {
      if (!handlers.isUnlocked()) return { error: 'Vault is locked' };
      const entries = handlers.getAllEntries();
      return { entries };
    }

    case 'get-decrypted-password': {
      if (!handlers.isUnlocked()) return { error: 'Vault is locked' };
      const password = handlers.getDecryptedPassword(message.entryId);
      const username = handlers.getEntryUsername(message.entryId);
      return { password, username };
    }

    default:
      return { error: `Unknown action: ${action}` };
  }
}

/**
 * Write the port number to a known file path so the native messaging
 * host can discover it.
 */
function writePortFile(port) {
  const portFilePath = getPortFilePath();
  fs.mkdirSync(path.dirname(portFilePath), { recursive: true });
  fs.writeFileSync(portFilePath, String(port), 'utf-8');
}

/**
 * Returns the path to the port file.
 * Stored in the app's userData folder.
 */
function getPortFilePath() {
  return path.join(app.getPath('userData'), '.server-port');
}

/**
 * Stop the local server and clean up the port file.
 */
function stopLocalServer() {
  if (server) {
    server.close();
    server = null;
    serverPort = null;
  }
  try {
    fs.unlinkSync(getPortFilePath());
  } catch {
    // File may not exist
  }
}

module.exports = { initLocalServer, startLocalServer, stopLocalServer };
