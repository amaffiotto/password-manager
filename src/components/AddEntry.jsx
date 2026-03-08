import React, { useState } from 'react';
import api from '../api';
import PasswordStrength from './PasswordStrength';
import { validateEntry } from '../utils/validators';

export default function AddEntry({ onClose, onSaved }) {
  const [siteName, setSiteName] = useState('');
  const [url, setUrl] = useState('');
  const [username, setUsername] = useState('');
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
    if (!password) {
      setError('Password is required');
      return;
    }

    try {
      await api.addEntry(siteName, url, username, password);
      onSaved();
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    }
  };

  return (
    <div className="form-overlay" onClick={onClose}>
      <div className="form-modal" onClick={(e) => e.stopPropagation()}>
        <h2>Add Password</h2>
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

          <label>Password</label>
          <div className="password-field">
            <input
              type="text"
              placeholder="Enter or generate"
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
          <PasswordStrength password={password} />

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
