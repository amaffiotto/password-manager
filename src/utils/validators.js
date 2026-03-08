export const MIN_MASTER_PASSWORD_LENGTH = 8;

export function validateMasterPassword(password) {
  const errors = [];
  if (!password || password.length < MIN_MASTER_PASSWORD_LENGTH) {
    errors.push(`Password must be at least ${MIN_MASTER_PASSWORD_LENGTH} characters`);
  }
  if (!/[A-Z]/.test(password)) {
    errors.push('Must contain at least one uppercase letter');
  }
  if (!/[a-z]/.test(password)) {
    errors.push('Must contain at least one lowercase letter');
  }
  if (!/[0-9]/.test(password)) {
    errors.push('Must contain at least one number');
  }
  return { valid: errors.length === 0, errors };
}

export function validateUrl(url) {
  if (!url) return true;
  try {
    new URL(url.startsWith('http') ? url : `https://${url}`);
    return true;
  } catch {
    return false;
  }
}

export function validateEntry(siteName, username) {
  const errors = [];
  if (!siteName || siteName.trim().length === 0) errors.push('Site name is required');
  if (!username || username.trim().length === 0) errors.push('Username is required');
  return { valid: errors.length === 0, errors };
}
