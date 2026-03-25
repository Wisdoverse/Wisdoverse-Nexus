//! Key derivation using Argon2id.

use argon2::{Algorithm, Argon2, Params, Version};
use rand::RngCore;

/// Key derivation helper.
pub struct KeyDerivation;

impl KeyDerivation {
    /// Derive a 32-byte key from a password and salt using Argon2id.
    pub fn derive_key(password: &str, salt: &[u8]) -> [u8; 32] {
        let params = Params::new(64 * 1024, 3, 2, Some(32)).expect("Argon2 params should be valid");
        let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut key = [0u8; 32];
        argon2
            .hash_password_into(password.as_bytes(), salt, &mut key)
            .expect("Argon2 hashing should succeed");
        key
    }

    /// Generate a random 8-byte salt.
    pub fn generate_salt() -> [u8; 8] {
        let mut salt = [0u8; 8];
        rand::thread_rng().fill_bytes(&mut salt);
        salt
    }

    /// Derive a key from a password, generating a fresh salt.
    /// Returns `(key, salt)`.
    pub fn derive_key_with_new_salt(password: &str) -> ([u8; 32], [u8; 8]) {
        let salt = Self::generate_salt();
        let key = Self::derive_key(password, &salt);
        (key, salt)
    }
}
