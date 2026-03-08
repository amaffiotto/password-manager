import { invoke } from '@tauri-apps/api/core';

const api = {
  // Master password
  hasMasterPassword: () => invoke('has_master_password'),
  setMasterPassword: (password) => invoke('set_master_password', { password }),
  verifyMasterPassword: (password) => invoke('verify_master_password', { password }),

  // Vault entries
  getEntries: () => invoke('get_entries'),
  addEntry: (siteName, url, username, password) =>
    invoke('add_entry', { siteName, url, username, password }),
  updateEntry: (id, siteName, url, username, password) =>
    invoke('update_entry', { id, siteName, url, username, password }),
  deleteEntry: (id) => invoke('delete_entry', { id }),
  getDecryptedPassword: (id) => invoke('get_decrypted_password', { id }),
  searchEntries: (query) => invoke('search_entries', { query }),

  // Utilities
  generatePassword: (length) => invoke('generate_password', { length }),
  copyToClipboard: (text) => invoke('copy_to_clipboard', { text }),
  lockVault: () => invoke('lock_vault'),

  // Settings
  getSettings: () => invoke('get_settings'),
  updateSettings: (settings) => invoke('update_settings', { settings }),

  // Export / Import
  exportVault: (format) => invoke('export_vault', { format }),
  importVault: (format, data) => invoke('import_vault', { format, data }),

  // PGP
  generatePgpKey: (name, email, keyType, passphrase) =>
    invoke('generate_pgp_key', { name, email, keyType, passphrase }),
  getPgpKeys: () => invoke('get_pgp_keys'),
  getPgpKey: (keyId) => invoke('get_pgp_key', { keyId }),
  deletePgpKey: (keyId) => invoke('delete_pgp_key', { keyId }),
  exportPgpPublicKey: (keyId) => invoke('export_pgp_public_key', { keyId }),
  exportPgpPrivateKey: (keyId) => invoke('export_pgp_private_key', { keyId }),
  pgpEncrypt: (plaintext, keyId) => invoke('pgp_encrypt', { plaintext, keyId }),
  pgpDecrypt: (encrypted, keyId, passphrase) =>
    invoke('pgp_decrypt', { encrypted, keyId, passphrase }),
  pgpSign: (message, keyId, passphrase) =>
    invoke('pgp_sign', { message, keyId, passphrase }),
  pgpVerify: (signedMessage, keyId) =>
    invoke('pgp_verify', { signedMessage, keyId }),

  // TOTP
  generateTotpSecret: () => invoke('generate_totp_secret'),
  setupTotp: (entryId, secret, algorithm, digits, period) =>
    invoke('setup_totp', { entryId, secret, algorithm, digits, period }),
  setupTotpFromUri: (entryId, uri) =>
    invoke('setup_totp_from_uri', { entryId, uri }),
  generateTotpCode: (entryId) => invoke('generate_totp_code', { entryId }),
  getTotpQrSvg: (entryId) => invoke('get_totp_qr_svg', { entryId }),
  removeTotp: (entryId) => invoke('remove_totp', { entryId }),

  // Extension
  setExtensionId: (id) => invoke('set_extension_id', { id }),
  reinstallNativeHost: () => invoke('reinstall_native_host'),
};

export default api;
