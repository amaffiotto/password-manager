import React, { useState, useEffect } from 'react';
import api from '../api';
import GeneratePGPKey from './GeneratePGPKey';
import PGPActions from './PGPActions';

export default function PGPKeys() {
  const [keys, setKeys] = useState([]);
  const [showGenerate, setShowGenerate] = useState(false);
  const [selectedKey, setSelectedKey] = useState(null);
  const [deleteConfirmId, setDeleteConfirmId] = useState(null);
  const [copiedId, setCopiedId] = useState(null);

  const loadKeys = async () => {
    const data = await api.getPgpKeys();
    setKeys(data);
  };

  useEffect(() => {
    loadKeys();
  }, []);

  const handleExportPublic = async (keyId) => {
    const pubKey = await api.exportPgpPublicKey(keyId);
    await api.copyToClipboard(pubKey);
    setCopiedId(keyId);
    setTimeout(() => setCopiedId(null), 2000);
  };

  const handleDeleteConfirm = async () => {
    await api.deletePgpKey(deleteConfirmId);
    setDeleteConfirmId(null);
    if (selectedKey && selectedKey.id === deleteConfirmId) {
      setSelectedKey(null);
    }
    loadKeys();
  };

  const handleGenerated = () => {
    setShowGenerate(false);
    loadKeys();
  };

  return (
    <div className="vault-container">
      <div className="vault-header">
        <h1>PGP Keys</h1>
        <div className="header-actions">
          <button
            className="btn btn-primary btn-small"
            onClick={() => setShowGenerate(true)}
          >
            + Generate Key
          </button>
        </div>
      </div>

      <div className="entry-list">
        {keys.length === 0 && (
          <div className="empty-state">
            No PGP keys yet. Click "+ Generate Key" to create one.
          </div>
        )}

        {keys.map((key) => (
          <div key={key.id} className="entry-card">
            <div className="entry-info">
              <h3>{key.name}</h3>
              <p>{key.email}</p>
              <p className="fingerprint">
                {key.key_type.toUpperCase()} &middot;{' '}
                {key.fingerprint.substring(0, 16).toUpperCase()}...
              </p>
            </div>
            <div className="entry-actions">
              <button
                className="btn btn-secondary btn-small"
                onClick={() => handleExportPublic(key.id)}
              >
                {copiedId === key.id ? 'Copied!' : 'Copy Public'}
              </button>
              <button
                className="btn btn-secondary btn-small"
                onClick={() => setSelectedKey(key)}
              >
                Actions
              </button>
              <button
                className="btn btn-danger btn-small"
                onClick={() => setDeleteConfirmId(key.id)}
              >
                Delete
              </button>
            </div>
          </div>
        ))}
      </div>

      {showGenerate && (
        <GeneratePGPKey
          onClose={() => setShowGenerate(false)}
          onGenerated={handleGenerated}
        />
      )}

      {selectedKey && (
        <PGPActions keyRecord={selectedKey} onClose={() => setSelectedKey(null)} />
      )}

      {deleteConfirmId !== null && (
        <div className="form-overlay" onClick={() => setDeleteConfirmId(null)}>
          <div className="form-modal" onClick={(e) => e.stopPropagation()}>
            <h2>Confirm Delete</h2>
            <p>
              Are you sure you want to delete this PGP key pair? This action cannot be
              undone.
            </p>
            <div className="form-actions">
              <button className="btn btn-danger" onClick={handleDeleteConfirm}>
                Delete
              </button>
              <button
                className="btn btn-secondary"
                onClick={() => setDeleteConfirmId(null)}
              >
                Cancel
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
}
