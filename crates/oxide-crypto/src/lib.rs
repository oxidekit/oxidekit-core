//! # OxideKit Crypto Platform
//!
//! First-class crypto support for OxideKit that enables building:
//! - Desktop/mobile crypto wallets
//! - Admin dashboards for crypto services
//! - Node/operator tooling
//! - Validator dashboards
//!
//! ## Core Principles
//!
//! - Crypto functionality lives in **`crypto.*` plugins**, never hardcoded into core
//! - Key material is **never exposed by default** and must use OS-backed secure storage
//! - Network access must respect **allowlists and attestation rules**
//! - All signing must be **human-readable and reviewable**
//! - Node tooling is **tooling**, not runtime UI magic
//!
//! ## Plugin Namespace
//!
//! All crypto plugins use the `crypto.*` namespace:
//!
//! - `crypto.core` - Shared primitives (encoding, serialization, errors)
//! - `crypto.keys` - Key management with secure storage
//! - `crypto.eth` - Ethereum support (EIP-1559, EIP-712, ABI)
//! - `crypto.btc` - Bitcoin support (UTXO, PSBT, address formats)
//! - `crypto.rpc` - Provider management with failover
//! - `crypto.policy` - Security guardrails and attestation
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use oxide_crypto::{
//!     core::{HexString, Base58String},
//!     keys::{DerivationPath, KeyLifecycle},
//!     ethereum::{ChainConfig, Eip1559Transaction},
//!     rpc::{ProviderConfig, ProviderPool},
//!     policy::{SigningPolicy, AttestationConfig},
//! };
//!
//! // Configure Ethereum mainnet
//! let chain = ChainConfig::ethereum_mainnet();
//!
//! // Set up provider pool with failover
//! let pool = ProviderPool::new()
//!     .add_provider(ProviderConfig::new("https://eth.llamarpc.com"))
//!     .add_provider(ProviderConfig::new("https://cloudflare-eth.com"))
//!     .with_rate_limit(10, Duration::from_secs(1));
//!
//! // Create signing policy
//! let policy = SigningPolicy::new()
//!     .require_confirmation(true)
//!     .require_human_readable(true);
//! ```
//!
//! ## Security Model
//!
//! OxideKit crypto follows a strict security model:
//!
//! 1. **No plaintext key export** - Keys never leave secure storage in plaintext
//! 2. **OS keychain integration** - Uses `native.keychain` for secure storage
//! 3. **Memory zeroization** - Sensitive data is zeroed after use
//! 4. **Attestation support** - Builds can be verified and audited
//! 5. **Network allowlists** - Only whitelisted domains can be accessed
//! 6. **Screenshot protection** - Optional screenshot blocking for sensitive UIs

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod core;
pub mod keys;
#[cfg(feature = "ethereum")]
pub mod ethereum;
#[cfg(feature = "bitcoin")]
pub mod bitcoin;
pub mod rpc;
pub mod policy;
pub mod nodeops;
pub mod starters;

mod error;

pub use error::{CryptoError, CryptoResult};

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::core::{
        HexString, Base58String, Bech32String,
        BigUint, Encoding, RedactedString,
    };
    pub use crate::keys::{
        DerivationPath, KeyLifecycle, KeyStatus, SecureKeyStore,
    };
    #[cfg(feature = "ethereum")]
    pub use crate::ethereum::{
        ChainConfig, Eip1559Transaction, TypedDataSigning,
    };
    #[cfg(feature = "bitcoin")]
    pub use crate::bitcoin::{
        BitcoinAddress, AddressFormat, UtxoSet, Psbt,
    };
    pub use crate::rpc::{
        ProviderConfig, ProviderPool, RpcClient,
    };
    pub use crate::policy::{
        SigningPolicy, AttestationConfig, PolicyEnforcer,
    };
    pub use crate::CryptoResult;
}

/// Plugin namespace for crypto functionality
pub const CRYPTO_NAMESPACE: &str = "crypto";

/// Plugin IDs for core crypto plugins
pub mod plugin_ids {
    /// Core primitives plugin
    pub const CORE: &str = "crypto.core";
    /// Key management plugin
    pub const KEYS: &str = "crypto.keys";
    /// Ethereum support plugin
    pub const ETH: &str = "crypto.eth";
    /// Bitcoin support plugin
    pub const BTC: &str = "crypto.btc";
    /// RPC provider plugin
    pub const RPC: &str = "crypto.rpc";
    /// Security policy plugin
    pub const POLICY: &str = "crypto.policy";
    /// Node operations plugin
    pub const NODEOPS: &str = "tool.nodeops";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_constant() {
        assert_eq!(CRYPTO_NAMESPACE, "crypto");
    }

    #[test]
    fn test_plugin_ids() {
        assert!(plugin_ids::CORE.starts_with("crypto."));
        assert!(plugin_ids::KEYS.starts_with("crypto."));
        assert!(plugin_ids::RPC.starts_with("crypto."));
        assert!(plugin_ids::POLICY.starts_with("crypto."));
    }
}
