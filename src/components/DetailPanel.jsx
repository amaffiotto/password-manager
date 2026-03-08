import React, { useState, useEffect } from 'react';
import api from '../api';
import TotpDisplay from './TotpDisplay';

const ICON_COLORS = ['icon-blue', 'icon-green', 'icon-orange', 'icon-purple', 'icon-teal', 'icon-pink', 'icon-red', 'icon-indigo'];

function getIconColor(name) {
  let hash = 0;
  for (let i = 0; i < name.length; i++) hash = name.charCodeAt(i) + ((hash << 5) - hash);
  return ICON_COLORS[Math.abs(hash) % ICON_COLORS.length];
}

function formatDate(dateStr) {
  if (!dateStr) return '';
  try {
    const d = new Date(dateStr);
    return d.toLocaleDateString(undefined, {
      day: '2-digit',
      month: '2-digit',
      year: 'numeric',
    }) + ', ' + d.toLocaleTimeString(undefined, {
      hour: '2-digit',
      minute: '2-digit',
      second: '2-digit',
    });
  } catch {
    return dateStr;
  }
}

export default function DetailPanel({ entry, isFavourite, onToggleFavourite, onEdit, onDelete, onSetupTotp, onRemoveTotp }) {
  const [showPassword, setShowPassword] = useState(false);
  const [decryptedPassword, setDecryptedPassword] = useState('');
  const [passwordStrength, setPasswordStrength] = useState(null);
  const [copied, setCopied] = useState('');

  useEffect(() => {
    setShowPassword(false);
    setDecryptedPassword('');
    setPasswordStrength(null);
    setCopied('');

    if (entry) {
      api.getDecryptedPassword(entry.id).then((pw) => {
        setDecryptedPassword(pw);
        setPasswordStrength(calculateStrength(pw));
      }).catch(() => {});
    }
  }, [entry?.id]);

  const handleCopy = async (text, field) => {
    await api.copyToClipboard(text);
    setCopied(field);
    setTimeout(() => setCopied(''), 2000);
  };

  const handleCopyPassword = async () => {
    if (decryptedPassword) {
      await handleCopy(decryptedPassword, 'password');
    }
  };

  if (!entry) {
    return (
      <div className="detail-panel">
        <div className="detail-empty">Select an entry to view details</div>
      </div>
    );
  }

  const iconColor = getIconColor(entry.site_name);
  const initial = entry.site_name.charAt(0);

  return (
    <div className="detail-panel">
      <div className="detail-header">
        <div className={`detail-icon ${iconColor}`}>{initial}</div>
        <div>
          <div className="detail-title">{entry.site_name}</div>
        </div>
      </div>

      <div className="detail-body">
        <div className="detail-field">
          <span className="detail-field-label">Username</span>
          <div className="detail-field-value">
            {entry.username}
            <button
              className="btn-icon"
              onClick={() => handleCopy(entry.username, 'username')}
              title="Copy username"
            >
              {copied === 'username' ? '\u2713' : '\u2398'}
            </button>
          </div>
        </div>

        <div className="detail-field">
          <span className="detail-field-label">Password</span>
          <div className="detail-password-row">
            <span className={showPassword ? 'detail-password-revealed' : 'detail-password-dots'}>
              {showPassword ? decryptedPassword : '\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022\u2022'}
            </span>
            <div className="detail-password-actions">
              <button
                className="btn-icon"
                onClick={() => setShowPassword(!showPassword)}
                title={showPassword ? 'Hide password' : 'Show password'}
              >
                {showPassword ? '\u25C9' : '\u25CE'}
              </button>
              <button
                className="btn-icon"
                onClick={handleCopyPassword}
                title="Copy password"
              >
                {copied === 'password' ? '\u2713' : '\u2398'}
              </button>
            </div>
          </div>
          {passwordStrength && (
            <div className="detail-strength">
              <div className="strength-bar">
                <div
                  className="strength-fill"
                  style={{
                    width: `${(passwordStrength.score / passwordStrength.maxScore) * 100}%`,
                    background: passwordStrength.color,
                  }}
                />
              </div>
              <span className="strength-label" style={{ color: passwordStrength.color }}>
                {passwordStrength.label}
                {decryptedPassword && ` (${decryptedPassword.length} / ${passwordStrength.bits} bits)`}
              </span>
            </div>
          )}
        </div>

        {entry.has_totp && (
          <div className="detail-field">
            <span className="detail-field-label">2FA Code</span>
            <TotpDisplay entryId={entry.id} />
          </div>
        )}

        {entry.url && (
          <div className="detail-field">
            <span className="detail-field-label">URL</span>
            <div className="detail-field-value">
              <a href={entry.url.startsWith('http') ? entry.url : `https://${entry.url}`} target="_blank" rel="noreferrer">
                {entry.url}
              </a>
            </div>
          </div>
        )}

        <div className="detail-field">
          <span className="detail-field-label">Tags</span>
          <div className="detail-tags">
            {isFavourite && <span className="tag-badge tag-badge-blue">Favourite</span>}
          </div>
        </div>
      </div>

      <div className="detail-meta">
        <div className="detail-meta-title">Metadata</div>
        <div className="detail-meta-row">
          <span className="detail-meta-label">ID</span>
          <span className="detail-meta-value">{entry.id}</span>
        </div>
        <div className="detail-meta-row">
          <span className="detail-meta-label">Created</span>
          <span className="detail-meta-value">{formatDate(entry.created_at)}</span>
        </div>
        <div className="detail-meta-row">
          <span className="detail-meta-label">Modified</span>
          <span className="detail-meta-value">{formatDate(entry.updated_at)}</span>
        </div>
      </div>

      <div className="detail-actions">
        <button className="btn btn-secondary btn-small" onClick={() => onToggleFavourite(entry.id)}>
          {isFavourite ? '\u2605 Unfavourite' : '\u2606 Favourite'}
        </button>
        {entry.has_totp ? (
          <button className="btn btn-secondary btn-small" onClick={() => onRemoveTotp(entry)}>
            Disable 2FA
          </button>
        ) : (
          <button className="btn btn-secondary btn-small" onClick={() => onSetupTotp(entry)}>
            Enable 2FA
          </button>
        )}
        <button className="btn btn-secondary btn-small" onClick={() => onEdit(entry)}>
          Edit
        </button>
        <button className="btn btn-danger btn-small" onClick={() => onDelete(entry.id)}>
          Delete
        </button>
      </div>
    </div>
  );
}

function calculateStrength(password) {
  if (!password) return null;

  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (password.length >= 16) score++;
  if (/[a-z]/.test(password)) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^a-zA-Z0-9]/.test(password)) score++;

  const charsetSize =
    (/[a-z]/.test(password) ? 26 : 0) +
    (/[A-Z]/.test(password) ? 26 : 0) +
    (/[0-9]/.test(password) ? 10 : 0) +
    (/[^a-zA-Z0-9]/.test(password) ? 33 : 0);
  const bits = Math.round(password.length * Math.log2(charsetSize || 1) * 10) / 10;

  const levels = [
    { min: 0, label: '', color: 'var(--text-tertiary)' },
    { min: 1, label: 'Very Weak', color: 'var(--strength-weak)' },
    { min: 3, label: 'Weak', color: 'var(--strength-fair)' },
    { min: 5, label: 'Good', color: 'var(--strength-good)' },
    { min: 6, label: 'Strong', color: 'var(--strength-strong)' },
    { min: 7, label: 'Very Strong', color: 'var(--strength-excellent)' },
  ];

  const level = levels.filter((l) => score >= l.min).pop();
  return { score, maxScore: 7, bits, ...level };
}
