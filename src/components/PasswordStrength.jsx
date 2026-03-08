import React from 'react';

function calculateStrength(password) {
  if (!password) return { score: 0, label: '', color: '' };

  let score = 0;
  if (password.length >= 8) score++;
  if (password.length >= 12) score++;
  if (password.length >= 16) score++;
  if (/[a-z]/.test(password)) score++;
  if (/[A-Z]/.test(password)) score++;
  if (/[0-9]/.test(password)) score++;
  if (/[^a-zA-Z0-9]/.test(password)) score++;

  const levels = [
    { min: 0, label: '', color: '#333' },
    { min: 1, label: 'Very Weak', color: '#e94560' },
    { min: 3, label: 'Weak', color: '#ff8c00' },
    { min: 5, label: 'Good', color: '#ffd700' },
    { min: 6, label: 'Strong', color: '#4ecca3' },
    { min: 7, label: 'Very Strong', color: '#00cc66' },
  ];

  const level = levels.filter((l) => score >= l.min).pop();
  return { score, maxScore: 7, ...level };
}

export default function PasswordStrength({ password }) {
  const { score, maxScore, label, color } = calculateStrength(password);
  if (!password) return null;

  const percentage = (score / maxScore) * 100;

  return (
    <div className="password-strength">
      <div className="strength-bar">
        <div
          className="strength-fill"
          style={{ width: `${percentage}%`, background: color }}
        />
      </div>
      <span className="strength-label" style={{ color }}>
        {label}
      </span>
    </div>
  );
}
