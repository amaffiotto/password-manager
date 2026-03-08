import React, { useState, useEffect, useRef } from 'react';
import api from '../api';

export default function TotpDisplay({ entryId }) {
  const [code, setCode] = useState('');
  const [remaining, setRemaining] = useState(0);
  const [period, setPeriod] = useState(30);
  const [copied, setCopied] = useState(false);
  const [error, setError] = useState('');
  const intervalRef = useRef(null);

  const fetchCode = async () => {
    try {
      const result = await api.generateTotpCode(entryId);
      setCode(result.code);
      setRemaining(result.remaining);
      setPeriod(result.period);
      setError('');
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    }
  };

  useEffect(() => {
    fetchCode();
    intervalRef.current = setInterval(() => {
      setRemaining((prev) => {
        if (prev <= 1) {
          fetchCode();
          return period;
        }
        return prev - 1;
      });
    }, 1000);

    return () => clearInterval(intervalRef.current);
  }, [entryId]);

  const handleCopy = async () => {
    if (code) {
      await api.copyToClipboard(code);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  };

  if (error) return null;

  const progress = period > 0 ? (remaining / period) * 100 : 0;
  const isUrgent = remaining <= 5;

  // Format code with a space in the middle (e.g., "123 456")
  const half = Math.floor(code.length / 2);
  const displayCode = code.slice(0, half) + ' ' + code.slice(half);

  return (
    <div className="totp-display">
      <div className="totp-code-row">
        <span className={`totp-code ${isUrgent ? 'totp-urgent' : ''}`}>{displayCode}</span>
        <button className="btn-icon" onClick={handleCopy} title="Copy code">
          {copied ? '\u2713' : '\u2398'}
        </button>
      </div>
      <div className="totp-timer">
        <div className="totp-timer-bar">
          <div
            className="totp-timer-fill"
            style={{
              width: `${progress}%`,
              background: isUrgent ? 'var(--danger)' : 'var(--accent)',
            }}
          />
        </div>
        <span className="totp-timer-text">{remaining}s</span>
      </div>
    </div>
  );
}
