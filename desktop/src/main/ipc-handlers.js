const { clipboard } = require('electron');
const {
  hasMasterPassword,
  setMasterPassword,
  verifyMasterPassword,
  addEntry,
  getAllEntries,
  getDecryptedPassword,
  updateEntry,
  deleteEntry,
  searchEntries,
} = require('./database');
const { generatePassword } = require('./crypto');
const { CLIPBOARD_CLEAR_MS } = require('../../../shared/constants');

// The master password is held in memory only while the app is unlocked
let currentMasterPassword = null;

let clipboardTimer = null;

function registerIpcHandlers(ipcMain) {
  // --- Master password ---

  ipcMain.handle('has-master-password', () => {
    return hasMasterPassword();
  });

  ipcMain.handle('set-master-password', (_event, password) => {
    setMasterPassword(password);
    currentMasterPassword = password;
    return true;
  });

  ipcMain.handle('verify-master-password', (_event, password) => {
    const valid = verifyMasterPassword(password);
    if (valid) {
      currentMasterPassword = password;
    }
    return valid;
  });

  // --- Vault entries ---

  ipcMain.handle('get-entries', () => {
    return getAllEntries();
  });

  ipcMain.handle('add-entry', (_event, siteName, url, username, password) => {
    if (!currentMasterPassword) throw new Error('Vault is locked');
    return addEntry(siteName, url, username, password, currentMasterPassword);
  });

  ipcMain.handle('update-entry', (_event, id, siteName, url, username, password) => {
    if (!currentMasterPassword) throw new Error('Vault is locked');
    return updateEntry(id, siteName, url, username, password, currentMasterPassword);
  });

  ipcMain.handle('delete-entry', (_event, id) => {
    return deleteEntry(id);
  });

  ipcMain.handle('get-decrypted-password', (_event, id) => {
    if (!currentMasterPassword) throw new Error('Vault is locked');
    return getDecryptedPassword(id, currentMasterPassword);
  });

  ipcMain.handle('search-entries', (_event, query) => {
    return searchEntries(query);
  });

  // --- Utilities ---

  ipcMain.handle('generate-password', (_event, length) => {
    return generatePassword(length || 16);
  });

  ipcMain.handle('copy-to-clipboard', (_event, text) => {
    clipboard.writeText(text);

    // Auto-clear clipboard after timeout
    if (clipboardTimer) clearTimeout(clipboardTimer);
    clipboardTimer = setTimeout(() => {
      clipboard.writeText('');
      clipboardTimer = null;
    }, CLIPBOARD_CLEAR_MS);

    return true;
  });
}

function isUnlocked() {
  return currentMasterPassword !== null;
}

function getCurrentMasterPassword() {
  return currentMasterPassword;
}

module.exports = { registerIpcHandlers, isUnlocked, getCurrentMasterPassword };
