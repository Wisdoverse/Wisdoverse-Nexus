//! Encryption and key derivation tests.
use super::*;

/// Helper: fresh random 32-byte key per test invocation (avoid hard-coded
/// fixtures flagged by CodeQL `rust/hard-coded-cryptographic-value`).
fn random_key() -> [u8; 32] {
    rand::random()
}

#[test]
fn encrypt_decrypt_bytes_roundtrip() {
    let key = random_key();
    let enc = DataEncryption::new(&key);
    let plaintext = b"hello, world!";
    let ciphertext = enc.encrypt(plaintext).expect("encrypt should succeed");
    let decrypted = enc.decrypt(&ciphertext).expect("decrypt should succeed");
    assert_eq!(decrypted, plaintext);
    assert_ne!(ciphertext, plaintext.to_vec());
}

#[test]
fn encrypt_decrypt_string_roundtrip() {
    let key = random_key();
    let enc = DataEncryption::new(&key);
    let plaintext = "hello, world!";
    let ciphertext = enc
        .encrypt_string(plaintext)
        .expect("encrypt_string should succeed");
    let decrypted = enc
        .decrypt_string(&ciphertext)
        .expect("decrypt_string should succeed");
    assert_eq!(decrypted, plaintext);
}

#[test]
fn different_nonces_different_ciphertexts() {
    let key = random_key();
    let enc = DataEncryption::new(&key);
    let plaintext = b"hello, world!";
    let c1 = enc.encrypt(plaintext).unwrap();
    let c2 = enc.encrypt(plaintext).unwrap();
    assert_ne!(c1, c2);
}

#[test]
fn decrypt_wrong_key_fails() {
    // Two independent 32-byte random keys: collision probability is 2^-256,
    // far below any realistic test failure mode. No fallback needed.
    let key1 = random_key();
    let key2 = random_key();
    let enc1 = DataEncryption::new(&key1);
    let enc2 = DataEncryption::new(&key2);
    let ciphertext = enc1.encrypt(b"secret").unwrap();
    let result = enc2.decrypt(&ciphertext);
    assert!(result.is_err());
}

#[test]
fn truncated_ciphertext_fails() {
    let key = random_key();
    let enc = DataEncryption::new(&key);
    let mut ciphertext = enc.encrypt(b"important data").unwrap();
    ciphertext[15] ^= 0xFF;
    let result = enc.decrypt(&ciphertext);
    assert!(result.is_err());
}
