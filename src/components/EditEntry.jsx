import React, { useState } from 'react';
import api from '../api';
import PasswordStrength from './PasswordStrength';
import { validateEntry } from '../utils/validators';

export default function EditEntry({ entry, onClose, onSaved }) {
  const [siteName, setSiteName] = useState(entry.site_name);
  const [url, setUrl] = useState(entry.url || '');
  const [username, setUsername] = useState(entry.username);
  const [password, setPassword] = useState('');
  const [passwordLength, setPasswordLength] = useState(16);
  const [error, setError] = useState('');

  const handleGenerate = async () => {
    const generated = await api.generatePassword(passwordLength);
    setPassword(generated);
  };

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');

    const validation = validateEntry(siteName, username);
    if (!validation.valid) {
      setError(validation.errors.join('. '));
      return;
    }

    try {
      // If password is empty, keep the current one
      let finalPassword = password;
      if (!finalPassword) {
        finalPassword = await api.getDecryptedPassword(entry.id);
      }
      await api.updateEntry(entry.id, siteName, url, username, finalPassword);
      onSaved();
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    }
  };

  return (
    <div className="form-overlay" onClick={onClose}>
      <div className="form-modal" onClick={(e) => e.stopPropagation()}>
        <h2>Edit Password</h2>
        <form onSubmit={handleSubmit}>
          <label>Site Name</label>
          <input
            type="text"
            placeholder="e.g. Google"
            value={siteName}
            onChange={(e) => setSiteName(e.target.value)}
            autoFocus
          />

          <label>URL (optional)</label>
          <input
            type="text"
            placeholder="e.g. google.com"
            value={url}
            onChange={(e) => setUrl(e.target.value)}
          />

          <label>Username / Email</label>
          <input
            type="text"
            placeholder="e.g. user@email.com"
            value={username}
            onChange={(e) => setUsername(e.target.value)}
          />

          <label>Password (leave empty to keep current)</label>
          <div className="password-field">
            <input
              type="text"
              placeholder="Enter new password or leave empty"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
            />
            <input
              type="number"
              min="8"
              max="128"
              value={passwordLength}
              onChange={(e) => setPasswordLength(Number(e.target.value))}
              className="password-length-input"
              title="Password length"
            />
            <button
              type="button"
              className="btn btn-secondary btn-small"
              onClick={handleGenerate}
            >
              Generate
            </button>
          </div>
          {password && <PasswordStrength password={password} />}

          {error && <span className="error">{error}</span>}

          <div className="form-actions">
            <button type="submit" className="btn btn-primary">
              Save
            </button>
            <button type="button" className="btn btn-secondary" onClick={onClose}>
              Cancel
            </button>
          </div>
        </form>
      </div>
    </div>
  );
}
