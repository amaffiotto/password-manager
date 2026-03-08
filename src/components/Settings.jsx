import React, { useState, useEffect } from 'react';
import api from '../api';
import { useTheme } from '../ThemeContext';

export default function Settings({ onLock }) {
  const { appearance, setAppearance, resetAppearance } = useTheme();
  const [autoLockMinutes, setAutoLockMinutes] = useState(5);
  const [defaultPasswordLength, setDefaultPasswordLength] = useState(16);
  const [clipboardClearSeconds, setClipboardClearSeconds] = useState(30);
  const [saved, setSaved] = useState(false);
  const [exportFormat, setExportFormat] = useState('json');
  const [importData, setImportData] = useState('');
  const [importFormat, setImportFormat] = useState('json');
  const [importResult, setImportResult] = useState('');
  const [extensionId, setExtensionId] = useState('');
  const [extensionSaved, setExtensionSaved] = useState(false);

  useEffect(() => {
    api.getSettings().then((settings) => {
      setAutoLockMinutes(settings.auto_lock_minutes);
      setDefaultPasswordLength(settings.default_password_length);
      setClipboardClearSeconds(settings.clipboard_clear_seconds);
    });
  }, []);

  const handleSave = async () => {
    await api.updateSettings({
      auto_lock_minutes: autoLockMinutes,
      default_password_length: defaultPasswordLength,
      clipboard_clear_seconds: clipboardClearSeconds,
    });
    setSaved(true);
    setTimeout(() => setSaved(false), 2000);
  };

  const handleExport = async () => {
    try {
      const data = await api.exportVault(exportFormat);
      await api.copyToClipboard(data);
      alert(`Exported ${exportFormat.toUpperCase()} copied to clipboard.`);
    } catch (err) {
      alert('Export failed: ' + (typeof err === 'string' ? err : err.message));
    }
  };

  const handleImport = async () => {
    if (!importData.trim()) {
      setImportResult('Please paste your import data');
      return;
    }
    try {
      const count = await api.importVault(importFormat, importData);
      setImportResult(`Successfully imported ${count} entries`);
      setImportData('');
    } catch (err) {
      setImportResult('Import failed: ' + (typeof err === 'string' ? err : err.message));
    }
  };

  return (
    <div className="settings-container">
      <h2 style={{ marginBottom: 16, fontSize: 16, fontWeight: 600 }}>Settings</h2>

      <div className="settings-section">
        <h3>Security</h3>
        <div className="setting-row">
          <label>Auto-lock after (minutes)</label>
          <input
            type="number"
            min="1"
            max="60"
            value={autoLockMinutes}
            onChange={(e) => setAutoLockMinutes(Number(e.target.value))}
          />
        </div>
        <div className="setting-row">
          <label>Clipboard auto-clear (seconds)</label>
          <input
            type="number"
            min="10"
            max="300"
            value={clipboardClearSeconds}
            onChange={(e) => setClipboardClearSeconds(Number(e.target.value))}
          />
        </div>
        <div className="setting-row">
          <label>Default password length</label>
          <input
            type="number"
            min="8"
            max="128"
            value={defaultPasswordLength}
            onChange={(e) => setDefaultPasswordLength(Number(e.target.value))}
          />
        </div>
        <button className="btn btn-primary btn-small" onClick={handleSave}>
          {saved ? 'Saved!' : 'Save Settings'}
        </button>
      </div>

      <div className="settings-section">
        <h3>Appearance</h3>
        <div className="setting-row">
          <label>Accent color</label>
          <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
            <input
              type="color"
              value={appearance.accentColor || '#4da3ff'}
              onChange={(e) => setAppearance({ accentColor: e.target.value })}
              style={{ width: 36, height: 28, padding: 0, border: 'none', cursor: 'pointer' }}
            />
            {appearance.accentColor && (
              <button className="btn btn-ghost btn-small" onClick={() => setAppearance({ accentColor: '' })}>Reset</button>
            )}
          </div>
        </div>
        <div className="setting-row">
          <label>Background color</label>
          <div style={{ display: 'flex', gap: 6, alignItems: 'center' }}>
            <input
              type="color"
              value={appearance.bgColor || '#1e1e1e'}
              onChange={(e) => setAppearance({ bgColor: e.target.value })}
              style={{ width: 36, height: 28, padding: 0, border: 'none', cursor: 'pointer' }}
            />
            {appearance.bgColor && (
              <button className="btn btn-ghost btn-small" onClick={() => setAppearance({ bgColor: '' })}>Reset</button>
            )}
          </div>
        </div>
        <div className="setting-row">
          <label>UI opacity</label>
          <div style={{ display: 'flex', gap: 8, alignItems: 'center', width: 160 }}>
            <input
              type="range"
              min="0.5"
              max="1"
              step="0.05"
              value={appearance.uiOpacity}
              onChange={(e) => setAppearance({ uiOpacity: parseFloat(e.target.value) })}
              style={{ flex: 1 }}
            />
            <span style={{ fontSize: 12, color: 'var(--text-secondary)', minWidth: 32, textAlign: 'right' }}>
              {Math.round(appearance.uiOpacity * 100)}%
            </span>
          </div>
        </div>
        <button className="btn btn-secondary btn-small" onClick={resetAppearance}>
          Reset All to Default
        </button>
      </div>

      <div className="settings-section">
        <h3>Export Vault</h3>
        <div className="setting-row">
          <label>Format</label>
          <select value={exportFormat} onChange={(e) => setExportFormat(e.target.value)}>
            <option value="json">JSON</option>
            <option value="csv">CSV</option>
          </select>
        </div>
        <button className="btn btn-secondary btn-small" onClick={handleExport}>
          Export to Clipboard
        </button>
      </div>

      <div className="settings-section">
        <h3>Import Vault</h3>
        <div className="setting-row">
          <label>Format</label>
          <select value={importFormat} onChange={(e) => setImportFormat(e.target.value)}>
            <option value="json">JSON</option>
            <option value="csv">CSV</option>
          </select>
        </div>
        <textarea
          className="import-textarea"
          placeholder={`Paste your ${importFormat.toUpperCase()} data here...`}
          value={importData}
          onChange={(e) => setImportData(e.target.value)}
          rows={6}
        />
        <button className="btn btn-secondary btn-small" onClick={handleImport}>
          Import
        </button>
        {importResult && (
          <span className={importResult.startsWith('Success') ? 'success' : 'error'} style={{ marginLeft: 8 }}>
            {importResult}
          </span>
        )}
      </div>

      <div className="settings-section">
        <h3>Browser Extension</h3>
        <p style={{ fontSize: 12, color: 'var(--text-secondary)', marginBottom: 8 }}>
          The native messaging host is automatically installed on app startup.
          If your extension uses a different ID (e.g. from loading unpacked), enter it here.
        </p>
        <div className="setting-row">
          <label>Extension ID (optional)</label>
          <input
            type="text"
            value={extensionId}
            onChange={(e) => setExtensionId(e.target.value.trim())}
            placeholder="e.g. abcdefghij..."
            style={{ width: 220, fontFamily: 'var(--font-mono)', fontSize: 11 }}
          />
        </div>
        <div style={{ display: 'flex', gap: 8 }}>
          <button
            className="btn btn-primary btn-small"
            onClick={async () => {
              try {
                if (extensionId) {
                  await api.setExtensionId(extensionId);
                } else {
                  await api.reinstallNativeHost();
                }
                setExtensionSaved(true);
                setTimeout(() => setExtensionSaved(false), 2000);
              } catch (err) {
                alert('Failed: ' + (typeof err === 'string' ? err : err.message));
              }
            }}
          >
            {extensionSaved ? 'Installed!' : extensionId ? 'Save & Reinstall' : 'Reinstall Native Host'}
          </button>
        </div>
      </div>
    </div>
  );
}
