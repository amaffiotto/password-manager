import React, { useState } from 'react';
import api from '../api';

export default function GeneratePGPKey({ onClose, onGenerated }) {
  const [name, setName] = useState('');
  const [email, setEmail] = useState('');
  const [keyType, setKeyType] = useState('ed25519');
  const [passphrase, setPassphrase] = useState('');
  const [confirmPassphrase, setConfirmPassphrase] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleSubmit = async (e) => {
    e.preventDefault();
    setError('');

    if (!name.trim()) {
      setError('Name is required');
      return;
    }
    if (!email.trim() || !email.includes('@')) {
      setError('Valid email is required');
      return;
    }
    if (!passphrase) {
      setError('Passphrase is required');
      return;
    }
    if (passphrase !== confirmPassphrase) {
      setError('Passphrases do not match');
      return;
    }

    setLoading(true);
    try {
      await api.generatePgpKey(name, email, keyType, passphrase);
      onGenerated();
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="form-overlay" onClick={onClose}>
      <div className="form-modal" onClick={(e) => e.stopPropagation()}>
        <h2>Generate PGP Key</h2>
        <form onSubmit={handleSubmit}>
          <label>Name</label>
          <input
            type="text"
            placeholder="e.g. John Doe"
            value={name}
            onChange={(e) => setName(e.target.value)}
            autoFocus
          />

          <label>Email</label>
          <input
            type="text"
            placeholder="e.g. john@example.com"
            value={email}
            onChange={(e) => setEmail(e.target.value)}
          />

          <label>Key Type</label>
          <select value={keyType} onChange={(e) => setKeyType(e.target.value)}>
            <option value="ed25519">Ed25519 (recommended, fast)</option>
            <option value="rsa4096">RSA 4096 (compatible, slower)</option>
          </select>

          <label>Passphrase</label>
          <input
            type="password"
            placeholder="Passphrase for the private key"
            value={passphrase}
            onChange={(e) => setPassphrase(e.target.value)}
          />

          <label>Confirm Passphrase</label>
          <input
            type="password"
            placeholder="Confirm passphrase"
            value={confirmPassphrase}
            onChange={(e) => setConfirmPassphrase(e.target.value)}
          />

          {error && <span className="error">{error}</span>}

          <div className="form-actions">
            <button type="submit" className="btn btn-primary" disabled={loading}>
              {loading ? 'Generating...' : 'Generate'}
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
