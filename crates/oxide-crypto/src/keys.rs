//! Key management types for crypto operations.
//!
//! This module provides types for:
//! - BIP-39 mnemonic generation/import
//! - BIP-32/BIP-44 derivation paths
//! - Encrypted local vault management
//! - Integration with native keychain
//! - Key lifecycle (create, lock, unlock, rotate)
//!
//! # Security
//!
//! - Keys are stored in OS keychain or encrypted vault only
//! - Memory zeroization after use
//! - No plaintext export by default
//! - No logging of secrets

use crate::{CryptoError, CryptoResult};
use crate::core::RedactedString;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;
use chrono::{DateTime, Utc};

// ============================================================================
// BIP-39 Mnemonic
// ============================================================================

/// Word count for BIP-39 mnemonics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MnemonicWordCount {
    /// 12 words (128 bits of entropy)
    Words12 = 12,
    /// 15 words (160 bits of entropy)
    Words15 = 15,
    /// 18 words (192 bits of entropy)
    Words18 = 18,
    /// 21 words (224 bits of entropy)
    Words21 = 21,
    /// 24 words (256 bits of entropy)
    Words24 = 24,
}

impl MnemonicWordCount {
    /// Get the entropy bits for this word count.
    pub fn entropy_bits(&self) -> usize {
        match self {
            Self::Words12 => 128,
            Self::Words15 => 160,
            Self::Words18 => 192,
            Self::Words21 => 224,
            Self::Words24 => 256,
        }
    }

    /// Get the checksum bits for this word count.
    pub fn checksum_bits(&self) -> usize {
        self.entropy_bits() / 32
    }
}

impl Default for MnemonicWordCount {
    fn default() -> Self {
        Self::Words24
    }
}

/// Configuration for generating a new mnemonic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicConfig {
    /// Number of words
    pub word_count: MnemonicWordCount,
    /// Language (default: English)
    pub language: MnemonicLanguage,
    /// Optional passphrase for additional security
    pub passphrase: Option<RedactedString>,
}

impl Default for MnemonicConfig {
    fn default() -> Self {
        Self {
            word_count: MnemonicWordCount::Words24,
            language: MnemonicLanguage::English,
            passphrase: None,
        }
    }
}

/// Supported mnemonic languages.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MnemonicLanguage {
    #[default]
    /// English wordlist
    English,
    /// Japanese wordlist
    Japanese,
    /// Korean wordlist
    Korean,
    /// Spanish wordlist
    Spanish,
    /// Chinese Simplified wordlist
    ChineseSimplified,
    /// Chinese Traditional wordlist
    ChineseTraditional,
    /// French wordlist
    French,
    /// Italian wordlist
    Italian,
    /// Czech wordlist
    Czech,
    /// Portuguese wordlist
    Portuguese,
}

/// Metadata about a mnemonic (without the actual words).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MnemonicInfo {
    /// Word count
    pub word_count: MnemonicWordCount,
    /// Language
    pub language: MnemonicLanguage,
    /// Whether a passphrase is used
    pub has_passphrase: bool,
    /// When the mnemonic was created/imported
    pub created_at: DateTime<Utc>,
}

// ============================================================================
// BIP-32/44 Derivation Paths
// ============================================================================

/// A BIP-32 derivation path component.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DerivationComponent {
    /// Normal (non-hardened) derivation
    Normal(u32),
    /// Hardened derivation (index + 0x80000000)
    Hardened(u32),
}

impl DerivationComponent {
    /// Get the raw index (without hardening flag).
    pub fn index(&self) -> u32 {
        match self {
            Self::Normal(i) | Self::Hardened(i) => *i,
        }
    }

    /// Check if this is a hardened component.
    pub fn is_hardened(&self) -> bool {
        matches!(self, Self::Hardened(_))
    }

    /// Get the full index (with hardening flag if applicable).
    pub fn full_index(&self) -> u32 {
        match self {
            Self::Normal(i) => *i,
            Self::Hardened(i) => i | 0x80000000,
        }
    }
}

impl fmt::Display for DerivationComponent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Normal(i) => write!(f, "{}", i),
            Self::Hardened(i) => write!(f, "{}'", i),
        }
    }
}

/// A BIP-32 derivation path.
///
/// Supports standard BIP-44 paths like `m/44'/60'/0'/0/0` for Ethereum.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DerivationPath {
    /// Path components after the master key
    components: Vec<DerivationComponent>,
}

impl DerivationPath {
    /// Create a new empty derivation path (master key).
    pub fn master() -> Self {
        Self { components: vec![] }
    }

    /// Create a derivation path from components.
    pub fn new(components: Vec<DerivationComponent>) -> Self {
        Self { components }
    }

    /// Parse a derivation path string.
    ///
    /// Supports formats like:
    /// - `m/44'/60'/0'/0/0`
    /// - `44'/60'/0'/0/0`
    /// - `m/44h/60h/0h/0/0` (h for hardened)
    pub fn parse(s: &str) -> CryptoResult<Self> {
        let s = s.trim();

        // Remove optional "m/" prefix
        let path_str = s.strip_prefix("m/").unwrap_or(s);

        if path_str.is_empty() || path_str == "m" {
            return Ok(Self::master());
        }

        let mut components = Vec::new();

        for part in path_str.split('/') {
            let part = part.trim();
            if part.is_empty() {
                continue;
            }

            let (index_str, hardened) = if let Some(s) = part.strip_suffix('\'') {
                (s, true)
            } else if let Some(s) = part.strip_suffix('h') {
                (s, true)
            } else if let Some(s) = part.strip_suffix('H') {
                (s, true)
            } else {
                (part, false)
            };

            let index: u32 = index_str.parse().map_err(|_| {
                CryptoError::InvalidDerivationPath(format!("invalid index: {}", part))
            })?;

            if index >= 0x80000000 {
                return Err(CryptoError::InvalidDerivationPath(
                    "index too large".to_string(),
                ));
            }

            components.push(if hardened {
                DerivationComponent::Hardened(index)
            } else {
                DerivationComponent::Normal(index)
            });
        }

        Ok(Self { components })
    }

    /// Get the path components.
    pub fn components(&self) -> &[DerivationComponent] {
        &self.components
    }

    /// Get the depth (number of components).
    pub fn depth(&self) -> usize {
        self.components.len()
    }

    /// Check if this is the master path (empty).
    pub fn is_master(&self) -> bool {
        self.components.is_empty()
    }

    /// Append a normal component.
    pub fn child(&self, index: u32) -> Self {
        let mut components = self.components.clone();
        components.push(DerivationComponent::Normal(index));
        Self { components }
    }

    /// Append a hardened component.
    pub fn hardened_child(&self, index: u32) -> Self {
        let mut components = self.components.clone();
        components.push(DerivationComponent::Hardened(index));
        Self { components }
    }

    /// Create a BIP-44 path for a specific coin and account.
    ///
    /// Format: m/44'/coin'/account'/change/index
    pub fn bip44(coin: u32, account: u32, change: u32, index: u32) -> Self {
        Self {
            components: vec![
                DerivationComponent::Hardened(44),
                DerivationComponent::Hardened(coin),
                DerivationComponent::Hardened(account),
                DerivationComponent::Normal(change),
                DerivationComponent::Normal(index),
            ],
        }
    }

    /// Create a BIP-44 Ethereum path.
    ///
    /// Format: m/44'/60'/account'/0/index
    pub fn ethereum(account: u32, index: u32) -> Self {
        Self::bip44(60, account, 0, index)
    }

    /// Create a BIP-44 Bitcoin path.
    ///
    /// Format: m/44'/0'/account'/change/index
    pub fn bitcoin(account: u32, change: u32, index: u32) -> Self {
        Self::bip44(0, account, change, index)
    }

    /// Create a BIP-84 path for native SegWit.
    ///
    /// Format: m/84'/0'/account'/change/index
    pub fn bip84(account: u32, change: u32, index: u32) -> Self {
        Self {
            components: vec![
                DerivationComponent::Hardened(84),
                DerivationComponent::Hardened(0),
                DerivationComponent::Hardened(account),
                DerivationComponent::Normal(change),
                DerivationComponent::Normal(index),
            ],
        }
    }
}

impl fmt::Display for DerivationPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "m")?;
        for component in &self.components {
            write!(f, "/{}", component)?;
        }
        Ok(())
    }
}

impl FromStr for DerivationPath {
    type Err = CryptoError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}

// ============================================================================
// Key Lifecycle
// ============================================================================

/// The current status of a key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStatus {
    /// Key is locked and requires unlock
    Locked,
    /// Key is unlocked and ready for use
    Unlocked,
    /// Key is being used for a signing operation
    InUse,
    /// Key has been rotated and is no longer the primary
    Rotated,
    /// Key has been destroyed
    Destroyed,
}

impl fmt::Display for KeyStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Locked => write!(f, "locked"),
            Self::Unlocked => write!(f, "unlocked"),
            Self::InUse => write!(f, "in_use"),
            Self::Rotated => write!(f, "rotated"),
            Self::Destroyed => write!(f, "destroyed"),
        }
    }
}

/// Key lifecycle management.
///
/// Handles the state transitions of keys through their lifecycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLifecycle {
    /// Current status
    pub status: KeyStatus,
    /// When the key was created
    pub created_at: DateTime<Utc>,
    /// When the key was last accessed
    pub last_accessed: Option<DateTime<Utc>>,
    /// When the key was unlocked (if currently unlocked)
    pub unlocked_at: Option<DateTime<Utc>>,
    /// Auto-lock timeout in seconds (None = never)
    pub auto_lock_seconds: Option<u32>,
    /// Number of times the key has been used
    pub use_count: u64,
}

impl KeyLifecycle {
    /// Create a new key lifecycle in locked state.
    pub fn new() -> Self {
        Self {
            status: KeyStatus::Locked,
            created_at: Utc::now(),
            last_accessed: None,
            unlocked_at: None,
            auto_lock_seconds: Some(300), // 5 minutes default
            use_count: 0,
        }
    }

    /// Check if the key should auto-lock.
    pub fn should_auto_lock(&self) -> bool {
        if self.status != KeyStatus::Unlocked {
            return false;
        }

        if let (Some(timeout), Some(unlocked_at)) = (self.auto_lock_seconds, self.unlocked_at) {
            let elapsed = Utc::now().signed_duration_since(unlocked_at);
            elapsed.num_seconds() > timeout as i64
        } else {
            false
        }
    }

    /// Transition to unlocked state.
    pub fn unlock(&mut self) -> CryptoResult<()> {
        match self.status {
            KeyStatus::Locked => {
                self.status = KeyStatus::Unlocked;
                self.unlocked_at = Some(Utc::now());
                Ok(())
            }
            KeyStatus::Unlocked => Ok(()), // Already unlocked
            _ => Err(CryptoError::KeyNotFound {
                key_id: "cannot unlock key in current state".to_string(),
            }),
        }
    }

    /// Transition to locked state.
    pub fn lock(&mut self) {
        if self.status == KeyStatus::Unlocked {
            self.status = KeyStatus::Locked;
            self.unlocked_at = None;
        }
    }

    /// Record a key use.
    pub fn record_use(&mut self) {
        self.use_count += 1;
        self.last_accessed = Some(Utc::now());
    }

    /// Mark the key as rotated.
    pub fn rotate(&mut self) {
        self.status = KeyStatus::Rotated;
    }

    /// Mark the key as destroyed.
    pub fn destroy(&mut self) {
        self.status = KeyStatus::Destroyed;
    }
}

impl Default for KeyLifecycle {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Key Storage
// ============================================================================

/// Identifier for a stored key.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyId(Uuid);

impl KeyId {
    /// Generate a new random key ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Parse from string.
    pub fn parse(s: &str) -> CryptoResult<Self> {
        Uuid::parse_str(s)
            .map(Self)
            .map_err(|_| CryptoError::KeyNotFound {
                key_id: "invalid key ID format".to_string(),
            })
    }
}

impl Default for KeyId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for KeyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The type of key storage backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStorageType {
    /// OS-level keychain (Keychain on macOS, Credential Manager on Windows)
    OsKeychain,
    /// Encrypted file-based vault
    EncryptedVault,
    /// Hardware security module
    Hsm,
    /// Memory only (volatile, for testing)
    Memory,
}

impl fmt::Display for KeyStorageType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OsKeychain => write!(f, "os_keychain"),
            Self::EncryptedVault => write!(f, "encrypted_vault"),
            Self::Hsm => write!(f, "hsm"),
            Self::Memory => write!(f, "memory"),
        }
    }
}

/// Configuration for secure key storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecureKeyStore {
    /// Storage type
    pub storage_type: KeyStorageType,
    /// Service name for keychain (e.g., "com.oxidekit.crypto")
    pub service_name: String,
    /// Whether biometric unlock is enabled
    pub biometric_enabled: bool,
    /// Auto-lock timeout in seconds
    pub auto_lock_seconds: Option<u32>,
}

impl Default for SecureKeyStore {
    fn default() -> Self {
        Self {
            storage_type: KeyStorageType::OsKeychain,
            service_name: "com.oxidekit.crypto".to_string(),
            biometric_enabled: false,
            auto_lock_seconds: Some(300),
        }
    }
}

/// Metadata about a stored key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyMetadata {
    /// Unique identifier
    pub id: KeyId,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Key type
    pub key_type: KeyType,
    /// Storage type
    pub storage_type: KeyStorageType,
    /// Lifecycle information
    pub lifecycle: KeyLifecycle,
    /// Associated derivation path (if derived)
    pub derivation_path: Option<DerivationPath>,
    /// Tags for organization
    pub tags: Vec<String>,
}

/// The type of cryptographic key.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyType {
    /// Secp256k1 (Bitcoin, Ethereum)
    Secp256k1,
    /// Ed25519 (Solana, etc.)
    Ed25519,
    /// Sr25519 (Polkadot)
    Sr25519,
    /// BLS12-381 (Ethereum 2.0)
    Bls12381,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Secp256k1 => write!(f, "secp256k1"),
            Self::Ed25519 => write!(f, "ed25519"),
            Self::Sr25519 => write!(f, "sr25519"),
            Self::Bls12381 => write!(f, "bls12-381"),
        }
    }
}

// ============================================================================
// Key Operations (Types Only)
// ============================================================================

/// Request to create a new key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateKeyRequest {
    /// Human-readable name
    pub name: String,
    /// Key type
    pub key_type: KeyType,
    /// Mnemonic configuration (for HD keys)
    pub mnemonic_config: Option<MnemonicConfig>,
    /// Derivation path (for derived keys)
    pub derivation_path: Option<DerivationPath>,
    /// Storage configuration
    pub storage: SecureKeyStore,
    /// Optional tags
    pub tags: Vec<String>,
}

/// Request to import a key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportKeyRequest {
    /// Human-readable name
    pub name: String,
    /// Key type
    pub key_type: KeyType,
    /// Import source
    pub source: KeyImportSource,
    /// Derivation path (for derived keys)
    pub derivation_path: Option<DerivationPath>,
    /// Storage configuration
    pub storage: SecureKeyStore,
    /// Optional tags
    pub tags: Vec<String>,
}

/// Source for key import.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyImportSource {
    /// BIP-39 mnemonic phrase
    Mnemonic {
        /// Word count to validate
        word_count: MnemonicWordCount,
        /// Language
        language: MnemonicLanguage,
    },
    /// Raw private key (hex encoded)
    PrivateKey,
    /// Keystore JSON file
    KeystoreJson,
    /// WIF format (Bitcoin)
    Wif,
}

/// Request to export a key's public information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportPublicKeyRequest {
    /// Key ID
    pub key_id: KeyId,
    /// Output format
    pub format: PublicKeyFormat,
}

/// Format for public key export.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PublicKeyFormat {
    /// Raw bytes (hex)
    Hex,
    /// Compressed public key
    Compressed,
    /// Uncompressed public key
    Uncompressed,
    /// Ethereum address
    EthereumAddress,
    /// Bitcoin address (legacy, P2PKH)
    BitcoinLegacy,
    /// Bitcoin address (SegWit, P2WPKH)
    BitcoinSegwit,
}

/// Result of a key operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyOperationResult {
    /// Whether the operation succeeded
    pub success: bool,
    /// Key ID (if applicable)
    pub key_id: Option<KeyId>,
    /// Error message (if failed)
    pub error: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_derivation_path_parsing() {
        let path = DerivationPath::parse("m/44'/60'/0'/0/0").unwrap();
        assert_eq!(path.depth(), 5);
        assert_eq!(path.to_string(), "m/44'/60'/0'/0/0");

        // Test without "m/" prefix
        let path2 = DerivationPath::parse("44'/60'/0'/0/0").unwrap();
        assert_eq!(path2, path);

        // Test with 'h' for hardened
        let path3 = DerivationPath::parse("m/44h/60h/0h/0/0").unwrap();
        assert_eq!(path3, path);
    }

    #[test]
    fn test_derivation_path_ethereum() {
        let path = DerivationPath::ethereum(0, 0);
        assert_eq!(path.to_string(), "m/44'/60'/0'/0/0");

        let path2 = DerivationPath::ethereum(1, 5);
        assert_eq!(path2.to_string(), "m/44'/60'/1'/0/5");
    }

    #[test]
    fn test_derivation_path_bitcoin() {
        let path = DerivationPath::bitcoin(0, 0, 0);
        assert_eq!(path.to_string(), "m/44'/0'/0'/0/0");

        let path_change = DerivationPath::bitcoin(0, 1, 0);
        assert_eq!(path_change.to_string(), "m/44'/0'/0'/1/0");
    }

    #[test]
    fn test_derivation_path_bip84() {
        let path = DerivationPath::bip84(0, 0, 0);
        assert_eq!(path.to_string(), "m/84'/0'/0'/0/0");
    }

    #[test]
    fn test_key_lifecycle() {
        let mut lifecycle = KeyLifecycle::new();
        assert_eq!(lifecycle.status, KeyStatus::Locked);

        lifecycle.unlock().unwrap();
        assert_eq!(lifecycle.status, KeyStatus::Unlocked);

        lifecycle.record_use();
        assert_eq!(lifecycle.use_count, 1);

        lifecycle.lock();
        assert_eq!(lifecycle.status, KeyStatus::Locked);
    }

    #[test]
    fn test_key_id() {
        let id = KeyId::new();
        let parsed = KeyId::parse(&id.to_string()).unwrap();
        assert_eq!(id, parsed);
    }

    #[test]
    fn test_mnemonic_word_count() {
        assert_eq!(MnemonicWordCount::Words12.entropy_bits(), 128);
        assert_eq!(MnemonicWordCount::Words24.entropy_bits(), 256);
        assert_eq!(MnemonicWordCount::Words12.checksum_bits(), 4);
        assert_eq!(MnemonicWordCount::Words24.checksum_bits(), 8);
    }
}
