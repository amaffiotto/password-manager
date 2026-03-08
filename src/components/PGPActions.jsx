import React, { useState } from 'react';
import api from '../api';

export default function PGPActions({ keyRecord, onClose }) {
  const [action, setAction] = useState('encrypt'); // 'encrypt' | 'decrypt' | 'sign' | 'verify'
  const [input, setInput] = useState('');
  const [passphrase, setPassphrase] = useState('');
  const [output, setOutput] = useState('');
  const [error, setError] = useState('');
  const [loading, setLoading] = useState(false);

  const handleExecute = async () => {
    setError('');
    setOutput('');
    setLoading(true);

    try {
      switch (action) {
        case 'encrypt': {
          const encrypted = await api.pgpEncrypt(input, keyRecord.id);
          setOutput(encrypted);
          break;
        }
        case 'decrypt': {
          if (!passphrase) {
            setError('Passphrase is required for decryption');
            setLoading(false);
            return;
          }
          const decrypted = await api.pgpDecrypt(input, keyRecord.id, passphrase);
          setOutput(decrypted);
          break;
        }
        case 'sign': {
          if (!passphrase) {
            setError('Passphrase is required for signing');
            setLoading(false);
            return;
          }
          const signed = await api.pgpSign(input, keyRecord.id, passphrase);
          setOutput(signed);
          break;
        }
        case 'verify': {
          const [verified, content] = await api.pgpVerify(input, keyRecord.id);
          setOutput(
            verified
              ? `Signature VALID\n\nMessage content:\n${content}`
              : 'Signature INVALID'
          );
          break;
        }
      }
    } catch (err) {
      setError(typeof err === 'string' ? err : err.message);
    } finally {
      setLoading(false);
    }
  };

  const handleCopyOutput = async () => {
    if (output) {
      await api.copyToClipboard(output);
    }
  };

  const needsPassphrase = action === 'decrypt' || action === 'sign';

  return (
    <div className="form-overlay" onClick={onClose}>
      <div className="form-modal pgp-actions-modal" onClick={(e) => e.stopPropagation()}>
        <h2>PGP Actions — {keyRecord.name}</h2>

        <div className="pgp-action-tabs">
          {['encrypt', 'decrypt', 'sign', 'verify'].map((a) => (
            <button
              key={a}
              className={`btn btn-small ${action === a ? 'btn-primary' : 'btn-secondary'}`}
              onClick={() => {
                setAction(a);
                setOutput('');
                setError('');
              }}
            >
              {a.charAt(0).toUpperCase() + a.slice(1)}
            </button>
          ))}
        </div>

        <label>
          {action === 'encrypt' && 'Plaintext to encrypt'}
          {action === 'decrypt' && 'PGP encrypted message'}
          {action === 'sign' && 'Message to sign'}
          {action === 'verify' && 'Signed PGP message'}
        </label>
        <textarea
          className="pgp-textarea"
          placeholder="Enter text here..."
          value={input}
          onChange={(e) => setInput(e.target.value)}
          rows={6}
        />

        {needsPassphrase && (
          <>
            <label>Private Key Passphrase</label>
            <input
              type="password"
              placeholder="Enter passphrase"
              value={passphrase}
              onChange={(e) => setPassphrase(e.target.value)}
            />
          </>
        )}

        {error && <span className="error">{error}</span>}

        <div className="form-actions">
          <button
            className="btn btn-primary"
            onClick={handleExecute}
            disabled={loading || !input}
          >
            {loading ? 'Processing...' : action.charAt(0).toUpperCase() + action.slice(1)}
          </button>
          <button className="btn btn-secondary" onClick={onClose}>
            Close
          </button>
        </div>

        {output && (
          <div className="pgp-output">
            <div className="pgp-output-header">
              <label>Result</label>
              <button className="btn btn-secondary btn-small" onClick={handleCopyOutput}>
                Copy
              </button>
            </div>
            <textarea className="pgp-textarea" value={output} readOnly rows={8} />
          </div>
        )}
      </div>
    </div>
  );
}
