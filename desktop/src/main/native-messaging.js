#!/usr/bin/env node

/**
 * native-messaging.js — Chrome Native Messaging host.
 *
 * Chrome launches this script as a child process.
 * It reads length-prefixed JSON from stdin and writes responses to stdout.
 * Each request is forwarded to the Electron app's local HTTP server.
 */

const http = require('http');
const path = require('path');
const fs = require('fs');
const os = require('os');

// --- Port discovery ---

/**
 * Find the port the desktop app's local server is listening on.
 * The Electron app writes this to a known file in its userData folder.
 */
function getServerPort() {
  // Electron's userData path by platform
  const appName = 'password-manager';
  let userDataDir;

  switch (process.platform) {
    case 'darwin':
      userDataDir = path.join(os.homedir(), 'Library', 'Application Support', appName);
      break;
    case 'win32':
      userDataDir = path.join(process.env.APPDATA || '', appName);
      break;
    default: // linux
      userDataDir = path.join(os.homedir(), '.config', appName);
      break;
  }

  const portFile = path.join(userDataDir, '.server-port');
  try {
    const port = parseInt(fs.readFileSync(portFile, 'utf-8').trim(), 10);
    if (isNaN(port)) throw new Error('Invalid port');
    return port;
  } catch {
    return null;
  }
}

// --- Native messaging protocol (Chrome) ---

/**
 * Read a single native messaging message from stdin.
 * Format: 4-byte little-endian length prefix + JSON payload.
 */
function readMessage() {
  return new Promise((resolve, reject) => {
    // Read 4-byte length header
    const headerBuf = Buffer.alloc(4);
    let headerRead = 0;

    function onReadable() {
      while (headerRead < 4) {
        const chunk = process.stdin.read(4 - headerRead);
        if (chunk === null) return; // wait for more data
        chunk.copy(headerBuf, headerRead);
        headerRead += chunk.length;
      }

      // Got the length
      const messageLength = headerBuf.readUInt32LE(0);
      if (messageLength === 0 || messageLength > 1024 * 1024) {
        reject(new Error(`Invalid message length: ${messageLength}`));
        return;
      }

      // Read the JSON body
      let bodyRead = 0;
      const bodyBuf = Buffer.alloc(messageLength);

      function readBody() {
        while (bodyRead < messageLength) {
          const chunk = process.stdin.read(messageLength - bodyRead);
          if (chunk === null) return; // wait for more data
          chunk.copy(bodyBuf, bodyRead);
          bodyRead += chunk.length;
        }

        process.stdin.removeListener('readable', readBody);
        try {
          const message = JSON.parse(bodyBuf.toString('utf-8'));
          resolve(message);
        } catch (err) {
          reject(err);
        }
      }

      process.stdin.removeListener('readable', onReadable);
      process.stdin.on('readable', readBody);
      readBody();
    }

    process.stdin.on('readable', onReadable);
    onReadable();
  });
}

/**
 * Write a native messaging response to stdout.
 */
function writeMessage(message) {
  const json = JSON.stringify(message);
  const buf = Buffer.from(json, 'utf-8');
  const header = Buffer.alloc(4);
  header.writeUInt32LE(buf.length, 0);
  process.stdout.write(header);
  process.stdout.write(buf);
}

// --- HTTP client to local server ---

function forwardToServer(port, payload) {
  return new Promise((resolve, reject) => {
    const data = JSON.stringify(payload);

    const req = http.request(
      {
        hostname: '127.0.0.1',
        port,
        path: '/api',
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          'Content-Length': Buffer.byteLength(data),
        },
        timeout: 5000,
      },
      (res) => {
        let body = '';
        res.on('data', (chunk) => (body += chunk));
        res.on('end', () => {
          try {
            resolve(JSON.parse(body));
          } catch {
            resolve({ error: 'Invalid response from desktop app' });
          }
        });
      }
    );

    req.on('error', () => {
      resolve({ error: 'Cannot connect to desktop app' });
    });

    req.on('timeout', () => {
      req.destroy();
      resolve({ error: 'Request timed out' });
    });

    req.write(data);
    req.end();
  });
}

// --- Main loop ---

async function main() {
  process.stdin.resume();

  while (true) {
    let message;
    try {
      message = await readMessage();
    } catch {
      // stdin closed or invalid data — exit
      process.exit(0);
    }

    const requestId = message._requestId;

    const port = getServerPort();
    let response;

    if (!port) {
      response = { error: 'Desktop app is not running' };
    } else {
      // Remove internal field before forwarding
      const payload = { ...message };
      delete payload._requestId;
      response = await forwardToServer(port, payload);
    }

    // Attach the request ID so the extension can match responses
    if (requestId != null) {
      response._requestId = requestId;
    }

    writeMessage(response);
  }
}

main();
