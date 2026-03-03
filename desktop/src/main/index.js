const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const { initDatabase, closeDatabase } = require('./database');
const { registerIpcHandlers, isUnlocked, getCurrentMasterPassword } = require('./ipc-handlers');
const { initLocalServer, startLocalServer, stopLocalServer } = require('./local-server');
const { DB_FILENAME } = require('../../../shared/constants');

let mainWindow = null;

function createWindow() {
  mainWindow = new BrowserWindow({
    width: 900,
    height: 650,
    minWidth: 600,
    minHeight: 500,
    title: 'Password Manager',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false,
    },
  });

  // In development, load from Vite dev server; in production, load the built file
  if (process.env.NODE_ENV === 'development') {
    mainWindow.loadURL('http://localhost:5173');
    mainWindow.webContents.openDevTools();
  } else {
    mainWindow.loadFile(path.join(__dirname, '../renderer/dist/index.html'));
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
  });
}

app.whenReady().then(() => {
  // Initialize the database in the app's user data folder
  const dbPath = path.join(app.getPath('userData'), DB_FILENAME);
  initDatabase(dbPath);

  // Register all IPC handlers so the renderer can talk to the backend
  registerIpcHandlers(ipcMain);

  // Start the local HTTP server for the browser extension bridge
  const { getAllEntries, searchEntries, getDecryptedPassword } = require('./database');
  initLocalServer({
    isUnlocked,
    getAllEntries,
    searchEntries,
    getDecryptedPassword: (entryId) => {
      const masterPassword = getCurrentMasterPassword();
      if (!masterPassword) throw new Error('Vault is locked');
      return getDecryptedPassword(entryId, masterPassword);
    },
    getEntryUsername: (entryId) => {
      const entries = getAllEntries();
      const entry = entries.find((e) => e.id === entryId);
      return entry ? entry.username : null;
    },
  });
  startLocalServer();

  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  stopLocalServer();
  closeDatabase();
  if (process.platform !== 'darwin') {
    app.quit();
  }
});
