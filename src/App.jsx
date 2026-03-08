import React, { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useTheme } from './ThemeContext';
import Login from './components/Login';
import Sidebar from './components/Sidebar';
import Vault from './components/Vault';
import DetailPanel from './components/DetailPanel';
import PGPKeys from './components/PGPKeys';
import Settings from './components/Settings';
import AddEntry from './components/AddEntry';
import EditEntry from './components/EditEntry';
import SetupTotp from './components/SetupTotp';
import api from './api';
import './styles.css';

export default function App() {
  const { theme, toggleTheme } = useTheme();
  const [unlocked, setUnlocked] = useState(false);
  const [view, setView] = useState('vault');
  const [entries, setEntries] = useState([]);
  const [selectedEntry, setSelectedEntry] = useState(null);
  const [search, setSearch] = useState('');
  const [showAdd, setShowAdd] = useState(false);
  const [editingEntry, setEditingEntry] = useState(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState(null);
  const [totpEntry, setTotpEntry] = useState(null);
  const [showHelp, setShowHelp] = useState(false);
  const [favourites, setFavourites] = useState(() => {
    try {
      const saved = localStorage.getItem('pm-favourites');
      return new Set(saved ? JSON.parse(saved) : []);
    } catch {
      return new Set();
    }
  });

  useEffect(() => {
    const unlisten = listen('vault-locked', () => {
      setUnlocked(false);
      setView('vault');
      setEntries([]);
      setSelectedEntry(null);
    });
    return () => {
      unlisten.then((fn) => fn());
    };
  }, []);

  useEffect(() => {
    try {
      localStorage.setItem('pm-favourites', JSON.stringify([...favourites]));
    } catch {}
  }, [favourites]);

  const loadEntries = useCallback(async () => {
    try {
      const data = await api.getEntries();
      setEntries(data);
    } catch (err) {
      console.error('Failed to load entries:', err);
    }
  }, []);

  useEffect(() => {
    if (unlocked) loadEntries();
  }, [unlocked, loadEntries]);

  const handleUnlock = () => {
    setUnlocked(true);
  };

  const handleLock = async () => {
    await api.lockVault();
    setUnlocked(false);
    setEntries([]);
    setSelectedEntry(null);
  };

  const handleToggleFavourite = (id) => {
    setFavourites((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleEntryAdded = () => {
    setShowAdd(false);
    loadEntries();
  };

  const handleEntryEdited = () => {
    setEditingEntry(null);
    loadEntries();
  };

  const handleDeleteConfirm = async () => {
    await api.deleteEntry(deleteConfirmId);
    if (selectedEntry && selectedEntry.id === deleteConfirmId) setSelectedEntry(null);
    setDeleteConfirmId(null);
    loadEntries();
  };

  const handleTotpSaved = () => {
    setTotpEntry(null);
    loadEntries();
  };

  const handleRemoveTotp = async (entry) => {
    if (confirm(`Disable 2FA for ${entry.site_name}?`)) {
      try {
        await api.removeTotp(entry.id);
        loadEntries();
      } catch (err) {
        alert('Failed to remove TOTP: ' + (typeof err === 'string' ? err : err.message));
      }
    }
  };

  const handleViewChange = (newView) => {
    setView(newView);
    setSelectedEntry(null);
    if (newView.startsWith('fav-')) {
      const id = parseInt(newView.replace('fav-', ''), 10);
      const entry = entries.find((e) => e.id === id);
      if (entry) {
        setView('vault');
        setSelectedEntry(entry);
      }
    }
  };

  if (!unlocked) {
    return <Login onUnlock={handleUnlock} theme={theme} toggleTheme={toggleTheme} />;
  }

  const getFilteredEntries = () => {
    let filtered = entries;

    if (view === 'favourites') {
      filtered = filtered.filter((e) => favourites.has(e.id));
    }

    if (search) {
      const q = search.toLowerCase();
      filtered = filtered.filter(
        (e) =>
          e.site_name.toLowerCase().includes(q) ||
          (e.url && e.url.toLowerCase().includes(q)) ||
          e.username.toLowerCase().includes(q)
      );
    }

    return filtered;
  };

  const showTable = view === 'vault' || view === 'all' || view === 'favourites';
  const filteredEntries = getFilteredEntries();

  return (
    <div className="app">
      {/* Toolbar */}
      <div className="toolbar">
        <div className="toolbar-left">
          <div>
            <div className="toolbar-title">Vault</div>
          </div>
          <button className="toolbar-btn" onClick={() => setShowHelp(true)} title="Info & Shortcuts">
            <svg width="16" height="16" viewBox="0 0 16 16" fill="currentColor">
              <path d="M8 1a7 7 0 1 0 0 14A7 7 0 0 0 8 1zm0 12.5a5.5 5.5 0 1 1 0-11 5.5 5.5 0 0 1 0 11zM7.25 5a.75.75 0 1 1 1.5 0 .75.75 0 0 1-1.5 0zM7.25 7h1.5v4h-1.5V7z"/>
            </svg>
          </button>
        </div>

        <div className="toolbar-center">
          <div className="toolbar-search">
            <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
              <path d="M11.742 10.344a6.5 6.5 0 1 0-1.397 1.398h-.001l3.85 3.85a1 1 0 0 0 1.415-1.414l-3.85-3.85zm-5.242 1.16a5 5 0 1 1 0-10 5 5 0 0 1 0 10z"/>
            </svg>
            <input
              type="text"
              placeholder="Search..."
              value={search}
              onChange={(e) => setSearch(e.target.value)}
            />
          </div>
        </div>

        <div className="toolbar-right">
          <button className="toolbar-btn" onClick={() => setShowAdd(true)} title="Add Entry">
            +
          </button>
          <button className="theme-toggle" onClick={toggleTheme} title="Toggle theme">
            {theme === 'dark' ? '\u2600' : '\u263E'}
          </button>
          <button className="toolbar-btn" onClick={handleLock} title="Lock Vault">
            &#128274;
          </button>
        </div>
      </div>

      {/* Main Layout */}
      <div className="app-layout">
        <Sidebar
          entries={entries}
          view={view}
          onViewChange={handleViewChange}
          favourites={favourites}
        />

        {/* Center content */}
        {showTable ? (
          <Vault
            entries={filteredEntries}
            selectedEntry={selectedEntry}
            onSelect={setSelectedEntry}
            favourites={favourites}
          />
        ) : view === 'pgp' ? (
          <PGPKeys />
        ) : view === 'settings' ? (
          <Settings onLock={handleLock} />
        ) : (
          <Vault
            entries={filteredEntries}
            selectedEntry={selectedEntry}
            onSelect={setSelectedEntry}
            favourites={favourites}
          />
        )}

        {/* Detail panel - only show when viewing entries */}
        {showTable && (
          <DetailPanel
            entry={selectedEntry}
            isFavourite={selectedEntry ? favourites.has(selectedEntry.id) : false}
            onToggleFavourite={handleToggleFavourite}
            onEdit={(entry) => setEditingEntry(entry)}
            onDelete={(id) => setDeleteConfirmId(id)}
            onSetupTotp={(entry) => setTotpEntry(entry)}
            onRemoveTotp={handleRemoveTotp}
          />
        )}
      </div>

      {/* Modals */}
      {showAdd && <AddEntry onClose={() => setShowAdd(false)} onSaved={handleEntryAdded} />}
      {editingEntry && (
        <EditEntry
          entry={editingEntry}
          onClose={() => setEditingEntry(null)}
          onSaved={handleEntryEdited}
        />
      )}
      {deleteConfirmId !== null && (
        <div className="form-overlay" onClick={() => setDeleteConfirmId(null)}>
          <div className="form-modal" onClick={(e) => e.stopPropagation()}>
            <h2>Confirm Delete</h2>
            <p>Are you sure you want to delete this entry? This cannot be undone.</p>
            <div className="form-actions">
              <button className="btn btn-danger" onClick={handleDeleteConfirm}>Delete</button>
              <button className="btn btn-secondary" onClick={() => setDeleteConfirmId(null)}>Cancel</button>
            </div>
          </div>
        </div>
      )}
      {totpEntry && (
        <SetupTotp
          entry={totpEntry}
          onClose={() => setTotpEntry(null)}
          onSaved={handleTotpSaved}
        />
      )}
      {showHelp && (
        <div className="form-overlay" onClick={() => setShowHelp(false)}>
          <div className="form-modal help-modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 520 }}>
            <h2>Commands & Shortcuts</h2>
            <div className="help-section">
              <h3>Vault</h3>
              <div className="help-row"><span className="help-key">+</span><span>Add new entry</span></div>
              <div className="help-row"><span className="help-key">Search bar</span><span>Search entries by name, URL, or username</span></div>
              <div className="help-row"><span className="help-key">Click entry</span><span>View entry details</span></div>
            </div>
            <div className="help-section">
              <h3>Entry Actions</h3>
              <div className="help-row"><span className="help-key">Edit</span><span>Modify entry fields</span></div>
              <div className="help-row"><span className="help-key">Delete</span><span>Permanently remove entry</span></div>
              <div className="help-row"><span className="help-key">Favourite</span><span>Star/unstar entry</span></div>
              <div className="help-row"><span className="help-key">Enable 2FA</span><span>Set up TOTP for an entry</span></div>
              <div className="help-row"><span className="help-key">Disable 2FA</span><span>Remove TOTP from an entry</span></div>
            </div>
            <div className="help-section">
              <h3>Password</h3>
              <div className="help-row"><span className="help-key">Eye icon</span><span>Show/hide password</span></div>
              <div className="help-row"><span className="help-key">Copy icon</span><span>Copy username or password</span></div>
              <div className="help-row"><span className="help-key">Generate</span><span>Generate random password when adding/editing</span></div>
            </div>
            <div className="help-section">
              <h3>Sidebar</h3>
              <div className="help-row"><span className="help-key">Database</span><span>View all entries</span></div>
              <div className="help-row"><span className="help-key">Favourites</span><span>View starred entries</span></div>
              <div className="help-row"><span className="help-key">PGP Keys</span><span>Manage PGP encryption keys</span></div>
              <div className="help-row"><span className="help-key">Settings</span><span>Configure auto-lock, clipboard, export/import</span></div>
            </div>
            <div className="help-section">
              <h3>Toolbar</h3>
              <div className="help-row"><span className="help-key">Sun/Moon</span><span>Toggle light/dark theme</span></div>
              <div className="help-row"><span className="help-key">Lock</span><span>Lock the vault</span></div>
              <div className="help-row"><span className="help-key">Info (i)</span><span>Show this help panel</span></div>
            </div>
            <div className="help-section">
              <h3>Appearance (Settings)</h3>
              <div className="help-row"><span className="help-key">Accent color</span><span>Customize the accent / highlight color</span></div>
              <div className="help-row"><span className="help-key">Background</span><span>Set a custom background color</span></div>
              <div className="help-row"><span className="help-key">UI opacity</span><span>Adjust overall UI transparency (50–100%)</span></div>
            </div>
            <div className="form-actions" style={{ marginTop: 16 }}>
              <button className="btn btn-secondary" onClick={() => setShowHelp(false)}>Close</button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
