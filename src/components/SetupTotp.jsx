import React, { useState, useEffect } from 'react';
import api from '../api';

export default function SetupTotp({ entry, onClose, onSaved }) {
  const [tab, setTab] = useState('manual'); // 'manual' | 'uri'
  const [secret, setSecret] = useState('');
  const [algorithm, setAlgorithm] = useState('SHA1');
  const [digits, setDigits] = useState(6);
  const [period, setPeriod] = useState(30);
  const [uri, setUri] = useState('');
  const [qrSvg, setQrSvg] = useState('');
  const [verifyCode, setVerifyCode] = useState('');
  const [step, setStep] = useState('setup'); // 'setup' | 'verify' | 'done'
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (tab === 'manual' && !secret) {
      api.generateTotpSecret().then(setSecret);
    }
  }, [tab]);

  const handleSetupManual = async () => {
    if (!secret.trim()) {
      setError('Secret is required');
      return;
    }
    setLoading(true);
    setError('');
    try {
      await api.setupTotp(entry.id, secret, algorithm, digits, period);
      const svg = await api.getTotpQrSvg(entry.id);
      setQrSvg(svg);
      setStep('verify');
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleSetupUri = async () => {
    if (!uri.trim()) {
      setError('otpauth:// URI is required');
      return;
    }
    setLoading(true);
    setError('');
    try {
      await api.setupTotpFromUri(entry.id, uri);
      const svg = await api.getTotpQrSvg(entry.id);
      setQrSvg(svg);
      setStep('verify');
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleVerify = async () => {
    setError('');
    try {
      const result = await api.generateTotpCode(entry.id);
      if (result.code === verifyCode.replace(/\s/g, '')) {
        setStep('done');
        setTimeout(() => onSaved(), 1500);
      } else {
        setError('Code does not match. Check your authenticator app and try again.');
      }
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    }
  };

  const handleSkipVerify = () => {
    onSaved();
  };

  return (
    <div className="form-overlay" onClick={onClose}>
      <div className="form-modal" onClick={(e) => e.stopPropagation()} style={{ maxWidth: 480 }}>
        <h2>Setup 2FA — {entry.site_name}</h2>

        {step === 'setup' && (
          <>
            <div className="pgp-action-tabs" style={{ marginBottom: 12 }}>
              <button
                className={`btn btn-small ${tab === 'manual' ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setTab('manual')}
              >
                Manual
              </button>
              <button
                className={`btn btn-small ${tab === 'uri' ? 'btn-primary' : 'btn-secondary'}`}
                onClick={() => setTab('uri')}
              >
                Paste URI
              </button>
            </div>

            {tab === 'manual' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                <label style={{ fontSize: 12, color: 'var(--text-secondary)' }}>Secret (Base32)</label>
                <input
                  type="text"
                  value={secret}
                  onChange={(e) => setSecret(e.target.value.toUpperCase())}
                  placeholder="e.g. JBSWY3DPEHPK3PXP"
                  style={{ fontFamily: 'var(--font-mono)', fontSize: 13 }}
                />
                <div style={{ display: 'flex', gap: 10 }}>
                  <div style={{ flex: 1 }}>
                    <label style={{ fontSize: 12, color: 'var(--text-secondary)' }}>Algorithm</label>
                    <select value={algorithm} onChange={(e) => setAlgorithm(e.target.value)}>
                      <option value="SHA1">SHA1</option>
                      <option value="SHA256">SHA256</option>
                      <option value="SHA512">SHA512</option>
                    </select>
                  </div>
                  <div style={{ flex: 1 }}>
                    <label style={{ fontSize: 12, color: 'var(--text-secondary)' }}>Digits</label>
                    <select value={digits} onChange={(e) => setDigits(Number(e.target.value))}>
                      <option value={6}>6</option>
                      <option value={8}>8</option>
                    </select>
                  </div>
                  <div style={{ flex: 1 }}>
                    <label style={{ fontSize: 12, color: 'var(--text-secondary)' }}>Period</label>
                    <select value={period} onChange={(e) => setPeriod(Number(e.target.value))}>
                      <option value={30}>30s</option>
                      <option value={60}>60s</option>
                    </select>
                  </div>
                </div>
              </div>
            )}

            {tab === 'uri' && (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
                <label style={{ fontSize: 12, color: 'var(--text-secondary)' }}>otpauth:// URI</label>
                <textarea
                  className="pgp-textarea"
                  value={uri}
                  onChange={(e) => setUri(e.target.value)}
                  placeholder="otpauth://totp/Example:user@example.com?secret=..."
                  rows={3}
                />
              </div>
            )}

            {error && <span className="error">{error}</span>}

            <div className="form-actions" style={{ marginTop: 12 }}>
              <button
                className="btn btn-primary"
                onClick={tab === 'manual' ? handleSetupManual : handleSetupUri}
                disabled={loading}
              >
                {loading ? 'Setting up...' : 'Enable 2FA'}
              </button>
              <button className="btn btn-secondary" onClick={onClose}>
                Cancel
              </button>
            </div>
          </>
        )}

        {step === 'verify' && (
          <>
            <p style={{ marginBottom: 12, color: 'var(--text-secondary)' }}>
              Scan this QR code with your authenticator app, then enter the code to verify.
            </p>

            {qrSvg && (
              <div
                className="totp-qr-container"
                dangerouslySetInnerHTML={{ __html: qrSvg }}
              />
            )}

            <label style={{ fontSize: 12, color: 'var(--text-secondary)', marginTop: 12 }}>
              Verification Code
            </label>
            <input
              type="text"
              value={verifyCode}
              onChange={(e) => setVerifyCode(e.target.value)}
              placeholder="Enter 6-digit code"
              maxLength={8}
              style={{ fontFamily: 'var(--font-mono)', fontSize: 18, textAlign: 'center', letterSpacing: 6 }}
              autoFocus
            />

            {error && <span className="error">{error}</span>}

            <div className="form-actions" style={{ marginTop: 12 }}>
              <button className="btn btn-primary" onClick={handleVerify}>
                Verify
              </button>
              <button className="btn btn-secondary" onClick={handleSkipVerify}>
                Skip Verification
              </button>
            </div>
          </>
        )}

        {step === 'done' && (
          <div style={{ textAlign: 'center', padding: 24 }}>
            <div style={{ fontSize: 32, marginBottom: 8 }}>{'\u2705'}</div>
            <p className="success" style={{ fontSize: 14 }}>2FA enabled successfully!</p>
          </div>
        )}
      </div>
    </div>
  );
}
