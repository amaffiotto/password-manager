import React, { useState, useEffect } from 'react';
import api from '../api';
import { validateMasterPassword } from '../utils/validators';

export default function Login({ onUnlock, theme, toggleTheme }) {
  const [password, setPassword] = useState('');
  const [confirmPassword, setConfirmPassword] = useState('');
  const [isFirstTime, setIsFirstTime] = useState(null);
  const [error, setError] = useState('');
  const [errors, setErrors] = useState([]);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    api.hasMasterPassword().then((has) => {
      setIsFirstTime(!has);
      setLoading(false);
    }).catch((err) => {
      console.error('Failed to check master password:', err);
      setIsFirstTime(true);
      setLoading(false);
    });
  }, []);

  const handleUnlock = async (e) => {
    e.preventDefault();
    setError('');
    setErrors([]);

    if (isFirstTime) {
      const validation = validateMasterPassword(password);
      if (!validation.valid) {
        setErrors(validation.errors);
        return;
      }
      if (password !== confirmPassword) {
        setError('Passwords do not match');
        return;
      }
      try {
        await api.setMasterPassword(password);
        onUnlock();
      } catch (err) {
        setError(typeof err === 'string' ? err : err.message);
      }
    } else {
      try {
        const valid = await api.verifyMasterPassword(password);
        if (valid) {
          onUnlock();
        } else {
          setError('Incorrect master password');
        }
      } catch (err) {
        setError(typeof err === 'string' ? err : err.message);
      }
    }
  };

  if (loading) {
    return (
      <div className="login-container">
        <p style={{ color: 'var(--text-secondary)' }}>Loading...</p>
      </div>
    );
  }

  return (
    <div className="login-container">
      <button
        className="theme-toggle"
        onClick={toggleTheme}
        style={{ position: 'absolute', top: 16, right: 16 }}
        title="Toggle theme"
      >
        {theme === 'dark' ? '\u2600' : '\u263E'}
      </button>

      <h1>Password Manager</h1>
      <p>{isFirstTime ? 'Create your master password' : 'Enter your master password'}</p>

      <form className="login-form" onSubmit={handleUnlock}>
        <input
          type="password"
          placeholder="Master password"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          autoFocus
        />

        {isFirstTime && (
          <input
            type="password"
            placeholder="Confirm master password"
            value={confirmPassword}
            onChange={(e) => setConfirmPassword(e.target.value)}
          />
        )}

        {error && <span className="error">{error}</span>}
        {errors.length > 0 && (
          <ul className="error-list">
            {errors.map((err, i) => (
              <li key={i}>{err}</li>
            ))}
          </ul>
        )}

        <button type="submit" className="btn btn-primary" style={{ width: '100%' }}>
          {isFirstTime ? 'Create' : 'Unlock'}
        </button>
      </form>
    </div>
  );
}
