// src/utils/crypto.ts

// ─── Simple Password Obfuscation ─────────────────────────────────────────────
// Uses a fixed key to XOR-encode passwords before storage.
// Not military-grade crypto, but prevents plaintext passwords on disk.
const ENCRYPT_KEY = 'TableView2026!SecretKey';

const xorCipher = (input: string, key: string): string => {
  let result = '';
  for (let i = 0; i < input.length; i++) {
    result += String.fromCharCode(input.charCodeAt(i) ^ key.charCodeAt(i % key.length));
  }
  return result;
};

export const encryptPassword = (password: string): string => {
  if (!password) return '';
  return btoa(xorCipher(password, ENCRYPT_KEY));
};

export const decryptPassword = (encrypted: string): string => {
  if (!encrypted) return '';
  try {
    return xorCipher(atob(encrypted), ENCRYPT_KEY);
  } catch {
    return encrypted; // If it fails, assume it's already plaintext (migration)
  }
};
