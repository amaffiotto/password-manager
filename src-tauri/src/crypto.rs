use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use sha2::Sha512;
use zeroize::Zeroize;

const PBKDF2_ITERATIONS: u32 = 100_000;
const KEY_LENGTH: usize = 32;
const SALT_LENGTH: usize = 16;
const IV_LENGTH: usize = 12;
const TAG_LENGTH: usize = 16;

const PASSWORD_CHARS: &[u8] =
    b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789!@#$%^&*()_+-=";

/// Derive a 256-bit key from master password + salt using PBKDF2-HMAC-SHA512.
fn derive_key(master_password: &str, salt: &[u8]) -> [u8; KEY_LENGTH] {
    let mut key = [0u8; KEY_LENGTH];
    pbkdf2::<Hmac<Sha512>>(
        master_password.as_bytes(),
        salt,
        PBKDF2_ITERATIONS,
        &mut key,
    )
    .expect("PBKDF2 derivation failed");
    key
}

/// Encrypt plaintext with AES-256-GCM.
/// Returns "salt:iv:authTag:ciphertext" (all hex-encoded).
/// Compatible with the Node.js crypto module format.
pub fn encrypt(plain_text: &str, master_password: &str) -> Result<String, String> {
    let mut salt = [0u8; SALT_LENGTH];
    let mut iv = [0u8; IV_LENGTH];
    rand::thread_rng().fill_bytes(&mut salt);
    rand::thread_rng().fill_bytes(&mut iv);

    let mut key = derive_key(master_password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    key.zeroize();

    let nonce = Nonce::from_slice(&iv);
    let ciphertext_with_tag = cipher
        .encrypt(nonce, plain_text.as_bytes())
        .map_err(|e| e.to_string())?;

    // aes-gcm appends the 16-byte auth tag to ciphertext
    let ct_len = ciphertext_with_tag.len() - TAG_LENGTH;
    let ciphertext = &ciphertext_with_tag[..ct_len];
    let auth_tag = &ciphertext_with_tag[ct_len..];

    Ok(format!(
        "{}:{}:{}:{}",
        hex::encode(salt),
        hex::encode(iv),
        hex::encode(auth_tag),
        hex::encode(ciphertext)
    ))
}

/// Decrypt from "salt:iv:authTag:ciphertext" format.
/// Compatible with the Node.js crypto module format.
pub fn decrypt(encrypted_data: &str, master_password: &str) -> Result<String, String> {
    let parts: Vec<&str> = encrypted_data.split(':').collect();
    if parts.len() != 4 {
        return Err("Invalid encrypted data format".to_string());
    }

    let salt = hex::decode(parts[0]).map_err(|e| e.to_string())?;
    let iv = hex::decode(parts[1]).map_err(|e| e.to_string())?;
    let auth_tag = hex::decode(parts[2]).map_err(|e| e.to_string())?;
    let ciphertext = hex::decode(parts[3]).map_err(|e| e.to_string())?;

    let mut key = derive_key(master_password, &salt);
    let cipher = Aes256Gcm::new_from_slice(&key).map_err(|e| e.to_string())?;
    key.zeroize();

    let nonce = Nonce::from_slice(&iv);

    // Reconstruct ciphertext||tag for aes-gcm
    let mut combined = ciphertext;
    combined.extend_from_slice(&auth_tag);

    let plaintext = cipher
        .decrypt(nonce, combined.as_ref())
        .map_err(|_| "Decryption failed: invalid password or corrupted data".to_string())?;

    String::from_utf8(plaintext).map_err(|e| e.to_string())
}

/// Generate a cryptographically secure random password.
/// Uses rejection sampling to eliminate modulo bias.
pub fn generate_password(length: usize) -> String {
    let charset_len = PASSWORD_CHARS.len();
    // Largest multiple of charset_len that fits in u8 range (0..256).
    // Values >= threshold are rejected to eliminate bias.
    let threshold = 256 - (256 % charset_len);
    let mut result = String::with_capacity(length);
    let mut rng = rand::thread_rng();

    while result.len() < length {
        let mut byte = [0u8; 1];
        rng.fill_bytes(&mut byte);
        let b = byte[0] as usize;
        if b < threshold {
            result.push(PASSWORD_CHARS[b % charset_len] as char);
        }
        // else: reject and retry (removes modulo bias)
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let password = "test-master-password";
        let plaintext = "my-secret-password-123!@#";

        let encrypted = encrypt(plaintext, password).unwrap();
        let decrypted = decrypt(&encrypted, password).unwrap();

        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_encrypted_format() {
        let encrypted = encrypt("test", "password").unwrap();
        let parts: Vec<&str> = encrypted.split(':').collect();
        assert_eq!(parts.len(), 4);
        assert_eq!(parts[0].len(), SALT_LENGTH * 2); // hex
        assert_eq!(parts[1].len(), IV_LENGTH * 2);
        assert_eq!(parts[2].len(), TAG_LENGTH * 2);
    }

    #[test]
    fn test_wrong_password_fails() {
        let encrypted = encrypt("secret", "correct-password").unwrap();
        assert!(decrypt(&encrypted, "wrong-password").is_err());
    }

    #[test]
    fn test_generate_password_length() {
        for len in [8, 16, 32, 64, 128] {
            let pwd = generate_password(len);
            assert_eq!(pwd.len(), len);
        }
    }

    #[test]
    fn test_generate_password_characters() {
        let pwd = generate_password(1000);
        let chars = std::str::from_utf8(PASSWORD_CHARS).unwrap();
        for c in pwd.chars() {
            assert!(chars.contains(c), "Unexpected character: {}", c);
        }
    }
}
