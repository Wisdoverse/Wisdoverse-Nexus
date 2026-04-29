//! AES-256-GCM data encryption for Wisdoverse Nexus Gateway.
//!
//! Provides optional encryption of sensitive data at rest.
//! Enabled via `NEXIS_ENCRYPTION_KEY` environment variable.

use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use rand::Rng;

/// Error type for cryptographic operations.
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    #[error("Invalid key length: expected 32 bytes")]
    InvalidKeyLength,
    #[error("Invalid nonce")]
    InvalidNonce,
    #[error("Invalid base64: {0}")]
    InvalidBase64(String),
    #[error("Invalid UTF-8: {0}")]
    InvalidUtf8(String),
}

/// Nonce size in bytes (96 bits for AES-GCM).
const NONCE_SIZE: usize = 12;

/// AES-256-GCM data encryption wrapper.
///
/// Ciphertext format: `[nonce (12 bytes)] [ciphertext + tag]`
/// When using string methods, the full ciphertext is base64-encoded.
#[derive(Clone)]
pub struct DataEncryption {
    cipher: Aes256Gcm,
}

impl DataEncryption {
    /// Create a new encryption instance from a 32-byte key.
    pub fn new(key: &[u8; 32]) -> Self {
        let cipher =
            Aes256Gcm::new_from_slice(key).expect("Aes256Gcm accepts exactly 32-byte keys");
        Self { cipher }
    }

    /// Try to create from environment variable `NEXIS_ENCRYPTION_KEY`.
    ///
    /// The env var must contain a hex-encoded 32-byte key (64 hex chars).
    /// Returns `None` if the env var is not set or invalid.
    pub fn from_env() -> Option<Self> {
        let hex_key = std::env::var("NEXIS_ENCRYPTION_KEY").ok()?;
        let key = hex::decode(&hex_key).ok()?;
        if key.len() != 32 {
            return None;
        }
        let key_array: [u8; 32] = key.try_into().ok()?;
        Some(Self::new(&key_array))
    }

    /// Generate a random 32-byte key, hex-encoded.
    pub fn generate_key_hex() -> String {
        let mut key = [0u8; 32];
        rand::rng().fill_bytes(&mut key);
        hex::encode(key)
    }

    /// Encrypt data.
    ///
    /// Returns: `[nonce (12 bytes)] [ciphertext + auth tag]`
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        rand::rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

        let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
        output.extend_from_slice(&nonce_bytes);
        output.extend_from_slice(&ciphertext);
        Ok(output)
    }

    /// Decrypt data.
    ///
    /// Expects input format: `[nonce (12 bytes)] [ciphertext + auth tag]`
    pub fn decrypt(&self, ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
        if ciphertext.len() < NONCE_SIZE {
            return Err(CryptoError::InvalidNonce);
        }
        let nonce = Nonce::from_slice(&ciphertext[..NONCE_SIZE]);
        let payload = &ciphertext[NONCE_SIZE..];

        self.cipher
            .decrypt(nonce, payload)
            .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }

    /// Encrypt a string, returning base64-encoded ciphertext.
    pub fn encrypt_string(&self, plaintext: &str) -> Result<String, CryptoError> {
        let ciphertext = self.encrypt(plaintext.as_bytes())?;
        Ok(BASE64.encode(&ciphertext))
    }

    /// Decrypt a base64-encoded ciphertext back to a string.
    pub fn decrypt_string(&self, ciphertext: &str) -> Result<String, CryptoError> {
        let raw = BASE64
            .decode(ciphertext)
            .map_err(|e| CryptoError::InvalidBase64(e.to_string()))?;
        let plaintext = self.decrypt(&raw)?;
        String::from_utf8(plaintext).map_err(|e| CryptoError::InvalidUtf8(e.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_decrypt() {
        let key = [0u8; 32];
        let enc = DataEncryption::new(&key);
        let plaintext = b"hello world";
        let ciphertext = enc.encrypt(plaintext).unwrap();
        let decrypted = enc.decrypt(&ciphertext).unwrap();
        assert_eq!(plaintext.as_slice(), decrypted.as_slice());
    }

    #[test]
    fn test_encrypt_decrypt_string() {
        let key = [0u8; 32];
        let enc = DataEncryption::new(&key);
        let plaintext = "hello world";
        let ciphertext = enc.encrypt_string(plaintext).unwrap();
        let decrypted = enc.decrypt_string(&ciphertext).unwrap();
        assert_eq!(plaintext, decrypted);
    }

    #[test]
    fn test_different_nonces() {
        let key = [0u8; 32];
        let enc = DataEncryption::new(&key);
        let plaintext = b"hello world";
        let c1 = enc.encrypt(plaintext).unwrap();
        let c2 = enc.encrypt(plaintext).unwrap();
        assert_ne!(c1, c2); // Different nonces produce different ciphertexts
    }

    #[test]
    fn test_decrypt_invalid_nonce() {
        let key = [0u8; 32];
        let enc = DataEncryption::new(&key);
        let result = enc.decrypt(b"short");
        assert!(matches!(result, Err(CryptoError::InvalidNonce)));
    }
}
