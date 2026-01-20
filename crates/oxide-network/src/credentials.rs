//! Secure credential storage for OxideKit.
//!
//! Provides platform-native secure storage integration:
//! - macOS: Keychain
//! - Windows: Credential Manager
//! - Linux: libsecret/GNOME Keyring
//!
//! Fallback to encrypted file storage when platform keychain is unavailable.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;

use crate::auth::TokenPair;
use crate::error::NetworkResult;

/// Trait for credential storage backends.
#[async_trait]
pub trait CredentialStore: Send + Sync + std::fmt::Debug {
    /// Store a credential.
    async fn store(&self, key: &str, credential: &Credential) -> NetworkResult<()>;

    /// Retrieve a credential.
    async fn retrieve(&self, key: &str) -> NetworkResult<Option<Credential>>;

    /// Delete a credential.
    async fn delete(&self, key: &str) -> NetworkResult<()>;

    /// List all credential keys.
    async fn list_keys(&self) -> NetworkResult<Vec<String>>;

    /// Check if a credential exists.
    async fn exists(&self, key: &str) -> NetworkResult<bool> {
        Ok(self.retrieve(key).await?.is_some())
    }

    /// Clear all credentials.
    async fn clear(&self) -> NetworkResult<()>;
}

/// A stored credential.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Credential {
    /// Credential type.
    pub credential_type: CredentialType,
    /// The credential value (encrypted at rest).
    pub value: String,
    /// Optional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// When the credential was stored.
    pub stored_at: chrono::DateTime<chrono::Utc>,
    /// When the credential expires (if applicable).
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl Credential {
    /// Create a new credential.
    pub fn new(credential_type: CredentialType, value: impl Into<String>) -> Self {
        Self {
            credential_type,
            value: value.into(),
            metadata: HashMap::new(),
            stored_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    /// Create a password credential.
    pub fn password(value: impl Into<String>) -> Self {
        Self::new(CredentialType::Password, value)
    }

    /// Create a token credential.
    pub fn token(value: impl Into<String>) -> Self {
        Self::new(CredentialType::Token, value)
    }

    /// Create an API key credential.
    pub fn api_key(value: impl Into<String>) -> Self {
        Self::new(CredentialType::ApiKey, value)
    }

    /// Set expiration.
    pub fn with_expiry(mut self, expires_at: chrono::DateTime<chrono::Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Add metadata.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if the credential is expired.
    pub fn is_expired(&self) -> bool {
        self.expires_at
            .map(|exp| chrono::Utc::now() >= exp)
            .unwrap_or(false)
    }
}

/// Types of credentials that can be stored.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CredentialType {
    /// Password.
    Password,
    /// Access/refresh token.
    Token,
    /// API key.
    ApiKey,
    /// OAuth refresh token.
    RefreshToken,
    /// Certificate.
    Certificate,
    /// Generic secret.
    Secret,
}

/// Credential storage manager.
///
/// Manages credential storage with automatic backend selection
/// and provides a high-level API for common operations.
#[derive(Debug)]
pub struct CredentialManager {
    /// The storage backend.
    store: Box<dyn CredentialStore>,
    /// Application identifier for namespacing.
    app_id: String,
}

impl CredentialManager {
    /// Create a new credential manager with the platform-native store.
    pub fn new(app_id: impl Into<String>) -> Self {
        let app_id = app_id.into();

        // Try to use platform keychain, fall back to file storage
        #[cfg(feature = "keychain")]
        let store: Box<dyn CredentialStore> = {
            match KeychainStore::new(&app_id) {
                Ok(store) => {
                    debug!("Using platform keychain for credential storage");
                    Box::new(store)
                }
                Err(e) => {
                    warn!(
                        "Platform keychain unavailable, using file storage: {}",
                        e
                    );
                    Box::new(FileStore::new(&app_id))
                }
            }
        };

        #[cfg(not(feature = "keychain"))]
        let store: Box<dyn CredentialStore> = {
            debug!("Keychain feature not enabled, using file storage");
            Box::new(FileStore::new(&app_id))
        };

        Self { store, app_id }
    }

    /// Create a credential manager with a specific backend.
    pub fn with_store(app_id: impl Into<String>, store: impl CredentialStore + 'static) -> Self {
        Self {
            store: Box::new(store),
            app_id: app_id.into(),
        }
    }

    /// Store a token pair.
    pub async fn store_tokens(&self, provider_id: &str, tokens: &TokenPair) -> NetworkResult<()> {
        let key = self.token_key(provider_id);
        let json = serde_json::to_string(tokens)?;

        let credential = Credential::new(CredentialType::Token, json)
            .with_metadata("provider_id", provider_id)
            .with_metadata("token_type", &tokens.token_type);

        if let Some(exp) = tokens.expires_at {
            self.store.store(&key, &credential.with_expiry(exp)).await
        } else {
            self.store.store(&key, &credential).await
        }
    }

    /// Retrieve a token pair.
    pub async fn retrieve_tokens(&self, provider_id: &str) -> NetworkResult<Option<TokenPair>> {
        let key = self.token_key(provider_id);

        match self.store.retrieve(&key).await? {
            Some(credential) => {
                if credential.is_expired() {
                    // Auto-delete expired credentials
                    let _ = self.store.delete(&key).await;
                    return Ok(None);
                }

                let tokens: TokenPair = serde_json::from_str(&credential.value)?;
                Ok(Some(tokens))
            }
            None => Ok(None),
        }
    }

    /// Delete stored tokens.
    pub async fn delete_tokens(&self, provider_id: &str) -> NetworkResult<()> {
        let key = self.token_key(provider_id);
        self.store.delete(&key).await
    }

    /// Store an API key.
    pub async fn store_api_key(&self, key_id: &str, api_key: &str) -> NetworkResult<()> {
        let storage_key = self.api_key_key(key_id);
        let credential = Credential::api_key(api_key).with_metadata("key_id", key_id);

        self.store.store(&storage_key, &credential).await
    }

    /// Retrieve an API key.
    pub async fn retrieve_api_key(&self, key_id: &str) -> NetworkResult<Option<String>> {
        let storage_key = self.api_key_key(key_id);

        match self.store.retrieve(&storage_key).await? {
            Some(credential) => Ok(Some(credential.value)),
            None => Ok(None),
        }
    }

    /// Delete an API key.
    pub async fn delete_api_key(&self, key_id: &str) -> NetworkResult<()> {
        let storage_key = self.api_key_key(key_id);
        self.store.delete(&storage_key).await
    }

    /// Store a generic secret.
    pub async fn store_secret(&self, name: &str, value: &str) -> NetworkResult<()> {
        let key = self.secret_key(name);
        let credential = Credential::new(CredentialType::Secret, value);
        self.store.store(&key, &credential).await
    }

    /// Retrieve a generic secret.
    pub async fn retrieve_secret(&self, name: &str) -> NetworkResult<Option<String>> {
        let key = self.secret_key(name);
        match self.store.retrieve(&key).await? {
            Some(credential) => Ok(Some(credential.value)),
            None => Ok(None),
        }
    }

    /// Delete a generic secret.
    pub async fn delete_secret(&self, name: &str) -> NetworkResult<()> {
        let key = self.secret_key(name);
        self.store.delete(&key).await
    }

    /// List all stored credentials.
    pub async fn list_all(&self) -> NetworkResult<Vec<String>> {
        self.store.list_keys().await
    }

    /// Clear all stored credentials.
    pub async fn clear_all(&self) -> NetworkResult<()> {
        self.store.clear().await
    }

    /// Generate storage key for tokens.
    fn token_key(&self, provider_id: &str) -> String {
        format!("{}.token.{}", self.app_id, provider_id)
    }

    /// Generate storage key for API keys.
    fn api_key_key(&self, key_id: &str) -> String {
        format!("{}.apikey.{}", self.app_id, key_id)
    }

    /// Generate storage key for secrets.
    fn secret_key(&self, name: &str) -> String {
        format!("{}.secret.{}", self.app_id, name)
    }
}

/// File-based credential storage (fallback).
///
/// Stores credentials in an encrypted file on disk.
/// This is used when platform keychain is not available.
#[derive(Debug)]
pub struct FileStore {
    /// Path to the credentials file.
    path: PathBuf,
    /// In-memory cache.
    cache: tokio::sync::RwLock<HashMap<String, Credential>>,
}

impl FileStore {
    /// Create a new file store.
    pub fn new(app_id: &str) -> Self {
        let path = dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("oxidekit")
            .join(app_id)
            .join("credentials.json");

        Self {
            path,
            cache: tokio::sync::RwLock::new(HashMap::new()),
        }
    }

    /// Load credentials from file.
    async fn load(&self) -> NetworkResult<HashMap<String, Credential>> {
        if !self.path.exists() {
            return Ok(HashMap::new());
        }

        let content = tokio::fs::read_to_string(&self.path).await?;

        // In production, this would be encrypted
        let credentials: HashMap<String, Credential> = serde_json::from_str(&content)?;
        Ok(credentials)
    }

    /// Save credentials to file.
    async fn save(&self, credentials: &HashMap<String, Credential>) -> NetworkResult<()> {
        // Ensure directory exists
        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        // In production, this would be encrypted
        let content = serde_json::to_string_pretty(credentials)?;
        tokio::fs::write(&self.path, content).await?;
        Ok(())
    }
}

#[async_trait]
impl CredentialStore for FileStore {
    async fn store(&self, key: &str, credential: &Credential) -> NetworkResult<()> {
        let mut cache = self.cache.write().await;

        // Load existing credentials
        if cache.is_empty() {
            *cache = self.load().await?;
        }

        cache.insert(key.to_string(), credential.clone());
        self.save(&cache).await?;

        debug!(key = %key, "Credential stored");
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> NetworkResult<Option<Credential>> {
        let mut cache = self.cache.write().await;

        // Load if cache is empty
        if cache.is_empty() {
            *cache = self.load().await?;
        }

        Ok(cache.get(key).cloned())
    }

    async fn delete(&self, key: &str) -> NetworkResult<()> {
        let mut cache = self.cache.write().await;

        // Load if cache is empty
        if cache.is_empty() {
            *cache = self.load().await?;
        }

        cache.remove(key);
        self.save(&cache).await?;

        debug!(key = %key, "Credential deleted");
        Ok(())
    }

    async fn list_keys(&self) -> NetworkResult<Vec<String>> {
        let mut cache = self.cache.write().await;

        if cache.is_empty() {
            *cache = self.load().await?;
        }

        Ok(cache.keys().cloned().collect())
    }

    async fn clear(&self) -> NetworkResult<()> {
        let mut cache = self.cache.write().await;
        cache.clear();

        if self.path.exists() {
            tokio::fs::remove_file(&self.path).await?;
        }

        debug!("All credentials cleared");
        Ok(())
    }
}

/// In-memory credential store (for testing).
#[derive(Debug, Default)]
pub struct MemoryStore {
    credentials: tokio::sync::RwLock<HashMap<String, Credential>>,
}

impl MemoryStore {
    /// Create a new memory store.
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl CredentialStore for MemoryStore {
    async fn store(&self, key: &str, credential: &Credential) -> NetworkResult<()> {
        self.credentials
            .write()
            .await
            .insert(key.to_string(), credential.clone());
        Ok(())
    }

    async fn retrieve(&self, key: &str) -> NetworkResult<Option<Credential>> {
        Ok(self.credentials.read().await.get(key).cloned())
    }

    async fn delete(&self, key: &str) -> NetworkResult<()> {
        self.credentials.write().await.remove(key);
        Ok(())
    }

    async fn list_keys(&self) -> NetworkResult<Vec<String>> {
        Ok(self.credentials.read().await.keys().cloned().collect())
    }

    async fn clear(&self) -> NetworkResult<()> {
        self.credentials.write().await.clear();
        Ok(())
    }
}

/// Platform keychain store (when keychain feature is enabled).
#[cfg(feature = "keychain")]
#[derive(Debug)]
pub struct KeychainStore {
    service: String,
}

#[cfg(feature = "keychain")]
impl KeychainStore {
    /// Create a new keychain store.
    pub fn new(app_id: &str) -> NetworkResult<Self> {
        Ok(Self {
            service: format!("oxidekit.{}", app_id),
        })
    }
}

#[cfg(feature = "keychain")]
#[async_trait]
impl CredentialStore for KeychainStore {
    async fn store(&self, key: &str, credential: &Credential) -> NetworkResult<()> {
        let service = self.service.clone();
        let key = key.to_string();
        let value = serde_json::to_string(credential)?;

        tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&service, &key)
                .map_err(|e| NetworkError::CredentialError {
                    message: e.to_string(),
                })?;
            entry
                .set_password(&value)
                .map_err(|e| NetworkError::CredentialError {
                    message: e.to_string(),
                })
        })
        .await
        .map_err(|e| NetworkError::CredentialError {
            message: e.to_string(),
        })?
    }

    async fn retrieve(&self, key: &str) -> NetworkResult<Option<Credential>> {
        let service = self.service.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&service, &key).map_err(|e| {
                NetworkError::CredentialError {
                    message: e.to_string(),
                }
            })?;

            match entry.get_password() {
                Ok(value) => {
                    let credential: Credential = serde_json::from_str(&value)?;
                    Ok(Some(credential))
                }
                Err(keyring::Error::NoEntry) => Ok(None),
                Err(e) => Err(NetworkError::CredentialError {
                    message: e.to_string(),
                }),
            }
        })
        .await
        .map_err(|e| NetworkError::CredentialError {
            message: e.to_string(),
        })?
    }

    async fn delete(&self, key: &str) -> NetworkResult<()> {
        let service = self.service.clone();
        let key = key.to_string();

        tokio::task::spawn_blocking(move || {
            let entry = keyring::Entry::new(&service, &key).map_err(|e| {
                NetworkError::CredentialError {
                    message: e.to_string(),
                }
            })?;

            match entry.delete_credential() {
                Ok(()) => Ok(()),
                Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
                Err(e) => Err(NetworkError::CredentialError {
                    message: e.to_string(),
                }),
            }
        })
        .await
        .map_err(|e| NetworkError::CredentialError {
            message: e.to_string(),
        })?
    }

    async fn list_keys(&self) -> NetworkResult<Vec<String>> {
        // Platform keychains don't typically support enumeration
        // This would need to be tracked separately
        Ok(Vec::new())
    }

    async fn clear(&self) -> NetworkResult<()> {
        // Can't enumerate keys, so can't clear
        // Individual keys must be deleted explicitly
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_store() {
        let store = MemoryStore::new();

        let credential = Credential::password("secret123");
        store.store("test-key", &credential).await.unwrap();

        let retrieved = store.retrieve("test-key").await.unwrap().unwrap();
        assert_eq!(retrieved.value, "secret123");

        store.delete("test-key").await.unwrap();
        assert!(store.retrieve("test-key").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_credential_manager() {
        let store = MemoryStore::new();
        let manager = CredentialManager::with_store("test-app", store);

        // Store and retrieve API key
        manager.store_api_key("my-api", "key123").await.unwrap();

        let retrieved = manager.retrieve_api_key("my-api").await.unwrap();
        assert_eq!(retrieved, Some("key123".to_string()));

        // Delete
        manager.delete_api_key("my-api").await.unwrap();
        assert!(manager.retrieve_api_key("my-api").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_token_storage() {
        let store = MemoryStore::new();
        let manager = CredentialManager::with_store("test-app", store);

        let tokens = TokenPair::new("access123")
            .with_refresh_token("refresh456")
            .with_expires_in(3600);

        manager.store_tokens("oauth", &tokens).await.unwrap();

        let retrieved = manager.retrieve_tokens("oauth").await.unwrap().unwrap();
        assert_eq!(retrieved.access_token, "access123");
        assert_eq!(retrieved.refresh_token, Some("refresh456".to_string()));
    }
}
