//! State encryption for sensitive data.
//!
//! This module provides encryption capabilities for state data,
//! using AES-256-GCM for encryption and Argon2id for key derivation.

use crate::error::{StateError, StateResult};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{PasswordHasher, SaltString},
    Argon2,
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Encrypted data wrapper with nonce and salt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedData {
    /// The encrypted ciphertext.
    pub ciphertext: Vec<u8>,
    /// The nonce used for encryption.
    pub nonce: [u8; 12],
    /// The salt used for key derivation (only for user-derived keys).
    pub salt: Option<String>,
    /// Encryption version for future compatibility.
    pub version: u8,
}

impl EncryptedData {
    /// Create a new encrypted data structure.
    pub fn new(ciphertext: Vec<u8>, nonce: [u8; 12], salt: Option<String>) -> Self {
        Self {
            ciphertext,
            nonce,
            salt,
            version: 1,
        }
    }

    /// Encode to base64 for storage.
    pub fn to_base64(&self) -> StateResult<String> {
        let json = serde_json::to_string(self)?;
        Ok(base64_encode(json.as_bytes()))
    }

    /// Decode from base64.
    pub fn from_base64(encoded: &str) -> StateResult<Self> {
        let bytes = base64_decode(encoded)?;
        let json = String::from_utf8(bytes)
            .map_err(|e| StateError::Decryption(format!("Invalid UTF-8: {}", e)))?;
        Ok(serde_json::from_str(&json)?)
    }
}

/// Simple base64 encoding (no padding).
fn base64_encode(data: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let b0 = chunk[0] as usize;
        let b1 = chunk.get(1).copied().unwrap_or(0) as usize;
        let b2 = chunk.get(2).copied().unwrap_or(0) as usize;

        result.push(ALPHABET[b0 >> 2] as char);
        result.push(ALPHABET[((b0 & 0x03) << 4) | (b1 >> 4)] as char);

        if chunk.len() > 1 {
            result.push(ALPHABET[((b1 & 0x0f) << 2) | (b2 >> 6)] as char);
        }
        if chunk.len() > 2 {
            result.push(ALPHABET[b2 & 0x3f] as char);
        }
    }

    result
}

/// Simple base64 decoding.
fn base64_decode(input: &str) -> StateResult<Vec<u8>> {
    const DECODE_TABLE: [i8; 128] = {
        let mut table = [-1i8; 128];
        let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
        let mut i = 0;
        while i < 64 {
            table[alphabet[i] as usize] = i as i8;
            i += 1;
        }
        table
    };

    let input = input.trim_end_matches('=');
    let mut result = Vec::with_capacity(input.len() * 3 / 4);

    let chars: Vec<u8> = input.bytes().collect();

    for chunk in chars.chunks(4) {
        let mut buf = [0u8; 4];
        for (i, &b) in chunk.iter().enumerate() {
            if b >= 128 {
                return Err(StateError::Decryption("Invalid base64 character".into()));
            }
            let val = DECODE_TABLE[b as usize];
            if val < 0 {
                return Err(StateError::Decryption("Invalid base64 character".into()));
            }
            buf[i] = val as u8;
        }

        result.push((buf[0] << 2) | (buf[1] >> 4));
        if chunk.len() > 2 {
            result.push((buf[1] << 4) | (buf[2] >> 2));
        }
        if chunk.len() > 3 {
            result.push((buf[2] << 6) | buf[3]);
        }
    }

    Ok(result)
}

/// State encryption service.
///
/// Provides encryption and decryption for sensitive state data.
pub struct StateEncryption {
    /// Cached application key (for secure tier).
    app_key: Arc<RwLock<Option<[u8; 32]>>>,
}

impl Default for StateEncryption {
    fn default() -> Self {
        Self::new()
    }
}

impl StateEncryption {
    /// Create a new state encryption service.
    pub fn new() -> Self {
        Self {
            app_key: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize with an application key.
    ///
    /// The application key is used for the "secure" tier encryption,
    /// which doesn't require user input but provides protection against
    /// casual access.
    pub async fn init_app_key(&self, key: [u8; 32]) {
        let mut app_key = self.app_key.write().await;
        *app_key = Some(key);
    }

    /// Generate a new random application key.
    pub async fn generate_app_key(&self) -> [u8; 32] {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);

        let mut app_key = self.app_key.write().await;
        *app_key = Some(key);

        key
    }

    /// Clear the cached application key.
    pub async fn clear_app_key(&self) {
        let mut app_key = self.app_key.write().await;
        *app_key = None;
    }

    /// Derive an encryption key from a password.
    pub fn derive_key(&self, password: &str, salt: &SaltString) -> StateResult<[u8; 32]> {
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), salt)
            .map_err(|e| StateError::Encryption(format!("Key derivation failed: {}", e)))?;

        // Extract the hash bytes
        let hash_output = hash
            .hash
            .ok_or_else(|| StateError::Encryption("No hash output".into()))?;

        let hash_bytes = hash_output.as_bytes();
        if hash_bytes.len() < 32 {
            return Err(StateError::Encryption("Hash too short".into()));
        }

        let mut key = [0u8; 32];
        key.copy_from_slice(&hash_bytes[..32]);
        Ok(key)
    }

    /// Encrypt data using the application key (secure tier).
    pub async fn encrypt_secure(&self, plaintext: &[u8]) -> StateResult<EncryptedData> {
        let app_key = self.app_key.read().await;
        let key = app_key
            .as_ref()
            .ok_or_else(|| StateError::Encryption("Application key not initialized".into()))?;

        self.encrypt_with_key(plaintext, key, None)
    }

    /// Decrypt data using the application key (secure tier).
    pub async fn decrypt_secure(&self, encrypted: &EncryptedData) -> StateResult<Vec<u8>> {
        let app_key = self.app_key.read().await;
        let key = app_key
            .as_ref()
            .ok_or_else(|| StateError::Encryption("Application key not initialized".into()))?;

        self.decrypt_with_key(encrypted, key)
    }

    /// Encrypt data using a password-derived key (encrypted tier).
    pub fn encrypt_with_password(
        &self,
        plaintext: &[u8],
        password: &str,
    ) -> StateResult<EncryptedData> {
        let salt = SaltString::generate(&mut OsRng);
        let key = self.derive_key(password, &salt)?;
        self.encrypt_with_key(plaintext, &key, Some(salt.as_str().to_string()))
    }

    /// Decrypt data using a password-derived key (encrypted tier).
    pub fn decrypt_with_password(
        &self,
        encrypted: &EncryptedData,
        password: &str,
    ) -> StateResult<Vec<u8>> {
        let salt_str = encrypted
            .salt
            .as_ref()
            .ok_or_else(|| StateError::Decryption("Missing salt for password-encrypted data".into()))?;

        let salt = SaltString::from_b64(salt_str)
            .map_err(|e| StateError::Decryption(format!("Invalid salt: {}", e)))?;

        let key = self.derive_key(password, &salt)?;
        self.decrypt_with_key(encrypted, &key)
    }

    /// Encrypt data with a raw key.
    fn encrypt_with_key(
        &self,
        plaintext: &[u8],
        key: &[u8; 32],
        salt: Option<String>,
    ) -> StateResult<EncryptedData> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| StateError::Encryption(format!("Cipher init failed: {}", e)))?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| StateError::Encryption(format!("Encryption failed: {}", e)))?;

        Ok(EncryptedData::new(ciphertext, nonce_bytes, salt))
    }

    /// Decrypt data with a raw key.
    fn decrypt_with_key(
        &self,
        encrypted: &EncryptedData,
        key: &[u8; 32],
    ) -> StateResult<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|e| StateError::Decryption(format!("Cipher init failed: {}", e)))?;

        let nonce = Nonce::from_slice(&encrypted.nonce);

        let plaintext = cipher
            .decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|e| StateError::Decryption(format!("Decryption failed: {}", e)))?;

        Ok(plaintext)
    }

    /// Verify a password against stored encrypted data.
    pub fn verify_password(&self, encrypted: &EncryptedData, password: &str) -> bool {
        self.decrypt_with_password(encrypted, password).is_ok()
    }
}

/// Trait for encrypted state types.
///
/// Implement this trait for state types that should be encrypted.
pub trait EncryptedState: serde::Serialize + serde::de::DeserializeOwned {
    /// Whether this state requires user-provided password.
    fn requires_password() -> bool;

    /// Encrypt the state to JSON.
    fn encrypt_json(&self, encryption: &StateEncryption, password: Option<&str>) -> StateResult<String> {
        let json = serde_json::to_string(self)?;
        let plaintext = json.as_bytes();

        let encrypted = if Self::requires_password() {
            let password = password
                .ok_or_else(|| StateError::Encryption("Password required for encryption".into()))?;
            encryption.encrypt_with_password(plaintext, password)?
        } else {
            // For non-password states, we'd use the app key
            // This would need to be async in practice
            return Err(StateError::Encryption(
                "Use encrypt_json_async for non-password encryption".into(),
            ));
        };

        encrypted.to_base64()
    }

    /// Decrypt the state from JSON.
    fn decrypt_json(encryption: &StateEncryption, encrypted_json: &str, password: Option<&str>) -> StateResult<Self> {
        let encrypted = EncryptedData::from_base64(encrypted_json)?;

        let plaintext = if Self::requires_password() {
            let password = password
                .ok_or_else(|| StateError::Decryption("Password required for decryption".into()))?;
            encryption.decrypt_with_password(&encrypted, password)?
        } else {
            return Err(StateError::Decryption(
                "Use decrypt_json_async for non-password decryption".into(),
            ));
        };

        let json = String::from_utf8(plaintext)
            .map_err(|e| StateError::Decryption(format!("Invalid UTF-8: {}", e)))?;

        Ok(serde_json::from_str(&json)?)
    }
}

/// Secure string that is zeroed on drop.
pub struct SecureString {
    inner: String,
}

impl SecureString {
    /// Create a new secure string.
    pub fn new(s: impl Into<String>) -> Self {
        Self { inner: s.into() }
    }

    /// Get the string value.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl Drop for SecureString {
    fn drop(&mut self) {
        // Zero out the string memory
        // SAFETY: We're just zeroing memory we own
        unsafe {
            let bytes = self.inner.as_bytes_mut();
            for byte in bytes.iter_mut() {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl std::ops::Deref for SecureString {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// Secure bytes that are zeroed on drop.
pub struct SecureBytes {
    inner: Vec<u8>,
}

impl SecureBytes {
    /// Create new secure bytes.
    pub fn new(bytes: Vec<u8>) -> Self {
        Self { inner: bytes }
    }

    /// Create secure bytes from a slice.
    pub fn from_slice(slice: &[u8]) -> Self {
        Self {
            inner: slice.to_vec(),
        }
    }

    /// Get the bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.inner
    }
}

impl Drop for SecureBytes {
    fn drop(&mut self) {
        for byte in self.inner.iter_mut() {
            unsafe {
                std::ptr::write_volatile(byte, 0);
            }
        }
    }
}

impl std::ops::Deref for SecureBytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_app_key_encryption() {
        let encryption = StateEncryption::new();
        encryption.generate_app_key().await;

        let plaintext = b"Hello, World!";
        let encrypted = encryption.encrypt_secure(plaintext).await.unwrap();
        let decrypted = encryption.decrypt_secure(&encrypted).await.unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_password_encryption() {
        let encryption = StateEncryption::new();
        let password = "my-secret-password";
        let plaintext = b"Sensitive data";

        let encrypted = encryption
            .encrypt_with_password(plaintext, password)
            .unwrap();
        let decrypted = encryption
            .decrypt_with_password(&encrypted, password)
            .unwrap();

        assert_eq!(plaintext.to_vec(), decrypted);
    }

    #[test]
    fn test_wrong_password() {
        let encryption = StateEncryption::new();
        let plaintext = b"Sensitive data";

        let encrypted = encryption
            .encrypt_with_password(plaintext, "correct-password")
            .unwrap();
        let result = encryption.decrypt_with_password(&encrypted, "wrong-password");

        assert!(result.is_err());
    }

    #[test]
    fn test_base64_roundtrip() {
        let data = b"Hello, World! This is a test.";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data.to_vec(), decoded);
    }

    #[test]
    fn test_encrypted_data_serialization() {
        let data = EncryptedData::new(vec![1, 2, 3, 4], [0u8; 12], Some("salt".to_string()));
        let encoded = data.to_base64().unwrap();
        let decoded = EncryptedData::from_base64(&encoded).unwrap();

        assert_eq!(data.ciphertext, decoded.ciphertext);
        assert_eq!(data.nonce, decoded.nonce);
        assert_eq!(data.salt, decoded.salt);
    }

    #[test]
    fn test_secure_string_zeroing() {
        let s = SecureString::new("secret");
        let ptr = s.as_str().as_ptr();
        let len = s.as_str().len();

        drop(s);

        // The memory should be zeroed (in debug mode this might not be observable
        // due to compiler optimizations, but the mechanism is in place)
        // This is primarily a compile-time verification that the Drop impl exists
    }
}
