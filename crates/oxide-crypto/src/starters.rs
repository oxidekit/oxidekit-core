//! Crypto starter definitions.
//!
//! This module provides starter templates for crypto applications:
//!
//! - `starter.crypto-wallet` - Desktop/mobile cryptocurrency wallet
//! - `starter.node-dashboard` - Node operator dashboard
//!
//! Starters provide production-ready scaffolding including:
//! - UI layouts and components
//! - Plugin configurations
//! - Security policies
//! - Network configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ============================================================================
// Starter Specifications
// ============================================================================

/// A crypto starter specification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoStarter {
    /// Starter identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Starter category
    pub category: StarterCategory,
    /// Required plugins
    pub plugins: Vec<PluginRequirement>,
    /// Permission presets
    pub permissions: Vec<PermissionPreset>,
    /// Default network configuration
    pub networks: Vec<NetworkPreset>,
    /// Security policy
    pub security: SecurityPreset,
    /// UI features
    pub features: Vec<StarterFeature>,
    /// Generated files
    pub files: Vec<GeneratedFile>,
}

impl CryptoStarter {
    /// Create the crypto wallet starter.
    pub fn crypto_wallet() -> Self {
        Self {
            id: "crypto-wallet".to_string(),
            name: "Crypto Wallet".to_string(),
            description: "A secure desktop/mobile cryptocurrency wallet with multi-chain support".to_string(),
            category: StarterCategory::Wallet,
            plugins: vec![
                PluginRequirement::required("crypto.core"),
                PluginRequirement::required("crypto.keys"),
                PluginRequirement::required("crypto.rpc"),
                PluginRequirement::required("crypto.policy"),
                PluginRequirement::optional("crypto.eth"),
                PluginRequirement::optional("crypto.btc"),
                PluginRequirement::required("native.keychain"),
                PluginRequirement::required("ui.tables"),
                PluginRequirement::optional("ui.charts"),
            ],
            permissions: vec![
                PermissionPreset::new("keychain")
                    .with_description("Secure key storage"),
                PermissionPreset::new("network")
                    .with_description("RPC network access")
                    .with_domains(vec![
                        "*.infura.io",
                        "*.alchemy.com",
                        "*.llamarpc.com",
                        "cloudflare-eth.com",
                        "mempool.space",
                    ]),
                PermissionPreset::new("clipboard")
                    .with_description("Copy addresses and transaction IDs"),
            ],
            networks: vec![
                NetworkPreset::ethereum_mainnet(),
                NetworkPreset::ethereum_sepolia(),
                NetworkPreset::bitcoin_mainnet(),
            ],
            security: SecurityPreset::wallet_default(),
            features: vec![
                StarterFeature::AssetList,
                StarterFeature::TransactionHistory,
                StarterFeature::SendReceive,
                StarterFeature::Settings,
                StarterFeature::DiagnosticsExport,
            ],
            files: vec![
                GeneratedFile::page("src/pages/assets.oxi", "Asset list and portfolio view"),
                GeneratedFile::page("src/pages/send.oxi", "Send transaction flow"),
                GeneratedFile::page("src/pages/receive.oxi", "Receive address display"),
                GeneratedFile::page("src/pages/history.oxi", "Transaction history"),
                GeneratedFile::page("src/pages/settings.oxi", "Wallet settings"),
                GeneratedFile::component("src/components/asset-row.oxi", "Asset list item"),
                GeneratedFile::component("src/components/tx-row.oxi", "Transaction list item"),
                GeneratedFile::component("src/components/address-qr.oxi", "QR code for addresses"),
                GeneratedFile::component("src/components/amount-input.oxi", "Crypto amount input"),
                GeneratedFile::config("oxide.toml", "Project configuration"),
                GeneratedFile::config("networks.toml", "Network configuration"),
                GeneratedFile::config("security.toml", "Security policy"),
            ],
        }
    }

    /// Create the node dashboard starter.
    pub fn node_dashboard() -> Self {
        Self {
            id: "node-dashboard".to_string(),
            name: "Node Dashboard".to_string(),
            description: "A monitoring dashboard for blockchain node operators".to_string(),
            category: StarterCategory::NodeOperator,
            plugins: vec![
                PluginRequirement::required("crypto.core"),
                PluginRequirement::required("crypto.rpc"),
                PluginRequirement::required("tool.nodeops"),
                PluginRequirement::required("ui.tables"),
                PluginRequirement::required("ui.charts"),
                PluginRequirement::optional("native.notifications"),
            ],
            permissions: vec![
                PermissionPreset::new("network")
                    .with_description("Node RPC access")
                    .with_domains(vec!["localhost", "127.0.0.1"]),
                PermissionPreset::new("notifications")
                    .with_description("Alert notifications"),
            ],
            networks: vec![
                NetworkPreset::local_ethereum(),
                NetworkPreset::local_bitcoin(),
            ],
            security: SecurityPreset::dashboard_default(),
            features: vec![
                StarterFeature::NodeHealth,
                StarterFeature::LogViewer,
                StarterFeature::SyncStatus,
                StarterFeature::Alerts,
                StarterFeature::PeerInfo,
            ],
            files: vec![
                GeneratedFile::page("src/pages/dashboard.oxi", "Main dashboard view"),
                GeneratedFile::page("src/pages/logs.oxi", "Log viewer"),
                GeneratedFile::page("src/pages/peers.oxi", "Peer list"),
                GeneratedFile::page("src/pages/alerts.oxi", "Alert configuration"),
                GeneratedFile::page("src/pages/settings.oxi", "Dashboard settings"),
                GeneratedFile::component("src/components/health-card.oxi", "Node health card"),
                GeneratedFile::component("src/components/sync-progress.oxi", "Sync progress bar"),
                GeneratedFile::component("src/components/log-line.oxi", "Log line display"),
                GeneratedFile::component("src/components/alert-rule.oxi", "Alert rule editor"),
                GeneratedFile::config("oxide.toml", "Project configuration"),
                GeneratedFile::config("nodes.toml", "Node configuration"),
                GeneratedFile::config("alerts.toml", "Alert rules"),
            ],
        }
    }

    /// Get all available crypto starters.
    pub fn all() -> Vec<Self> {
        vec![Self::crypto_wallet(), Self::node_dashboard()]
    }
}

/// Starter category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarterCategory {
    /// Cryptocurrency wallet
    Wallet,
    /// Node operator tools
    NodeOperator,
    /// DeFi application
    DeFi,
    /// NFT application
    Nft,
    /// Analytics dashboard
    Analytics,
}

impl std::fmt::Display for StarterCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Wallet => write!(f, "Wallet"),
            Self::NodeOperator => write!(f, "Node Operator"),
            Self::DeFi => write!(f, "DeFi"),
            Self::Nft => write!(f, "NFT"),
            Self::Analytics => write!(f, "Analytics"),
        }
    }
}

// ============================================================================
// Plugin Requirements
// ============================================================================

/// A required or optional plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginRequirement {
    /// Plugin ID
    pub plugin_id: String,
    /// Whether the plugin is required
    pub required: bool,
    /// Minimum version (if any)
    pub min_version: Option<String>,
    /// Reason for requiring this plugin
    pub reason: Option<String>,
}

impl PluginRequirement {
    /// Create a required plugin.
    pub fn required(plugin_id: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            required: true,
            min_version: None,
            reason: None,
        }
    }

    /// Create an optional plugin.
    pub fn optional(plugin_id: &str) -> Self {
        Self {
            plugin_id: plugin_id.to_string(),
            required: false,
            min_version: None,
            reason: None,
        }
    }

    /// Set minimum version.
    pub fn with_version(mut self, version: &str) -> Self {
        self.min_version = Some(version.to_string());
        self
    }

    /// Set reason.
    pub fn with_reason(mut self, reason: &str) -> Self {
        self.reason = Some(reason.to_string());
        self
    }
}

// ============================================================================
// Permission Presets
// ============================================================================

/// A permission preset for a starter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPreset {
    /// Permission name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// Allowed domains (for network permissions)
    pub allowed_domains: Vec<String>,
    /// Specific capabilities
    pub capabilities: Vec<String>,
}

impl PermissionPreset {
    /// Create a new permission preset.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            description: None,
            allowed_domains: vec![],
            capabilities: vec![],
        }
    }

    /// Set description.
    pub fn with_description(mut self, description: &str) -> Self {
        self.description = Some(description.to_string());
        self
    }

    /// Set allowed domains.
    pub fn with_domains(mut self, domains: Vec<&str>) -> Self {
        self.allowed_domains = domains.into_iter().map(String::from).collect();
        self
    }

    /// Add a capability.
    pub fn with_capability(mut self, capability: &str) -> Self {
        self.capabilities.push(capability.to_string());
        self
    }
}

// ============================================================================
// Network Presets
// ============================================================================

/// A network configuration preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPreset {
    /// Network identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Chain type
    pub chain_type: ChainType,
    /// Chain ID (for EVM chains)
    pub chain_id: Option<u64>,
    /// Default RPC endpoints
    pub rpc_urls: Vec<String>,
    /// Block explorer URL
    pub explorer_url: Option<String>,
    /// Whether this is a testnet
    pub testnet: bool,
    /// Native currency symbol
    pub native_currency: String,
}

impl NetworkPreset {
    /// Ethereum mainnet preset.
    pub fn ethereum_mainnet() -> Self {
        Self {
            id: "ethereum-mainnet".to_string(),
            name: "Ethereum Mainnet".to_string(),
            chain_type: ChainType::Ethereum,
            chain_id: Some(1),
            rpc_urls: vec![
                "https://eth.llamarpc.com".to_string(),
                "https://cloudflare-eth.com".to_string(),
            ],
            explorer_url: Some("https://etherscan.io".to_string()),
            testnet: false,
            native_currency: "ETH".to_string(),
        }
    }

    /// Ethereum Sepolia testnet preset.
    pub fn ethereum_sepolia() -> Self {
        Self {
            id: "ethereum-sepolia".to_string(),
            name: "Sepolia Testnet".to_string(),
            chain_type: ChainType::Ethereum,
            chain_id: Some(11155111),
            rpc_urls: vec!["https://rpc.sepolia.org".to_string()],
            explorer_url: Some("https://sepolia.etherscan.io".to_string()),
            testnet: true,
            native_currency: "SEP".to_string(),
        }
    }

    /// Bitcoin mainnet preset.
    pub fn bitcoin_mainnet() -> Self {
        Self {
            id: "bitcoin-mainnet".to_string(),
            name: "Bitcoin Mainnet".to_string(),
            chain_type: ChainType::Bitcoin,
            chain_id: None,
            rpc_urls: vec!["https://mempool.space/api".to_string()],
            explorer_url: Some("https://mempool.space".to_string()),
            testnet: false,
            native_currency: "BTC".to_string(),
        }
    }

    /// Local Ethereum node preset.
    pub fn local_ethereum() -> Self {
        Self {
            id: "ethereum-local".to_string(),
            name: "Local Ethereum Node".to_string(),
            chain_type: ChainType::Ethereum,
            chain_id: Some(1),
            rpc_urls: vec!["http://localhost:8545".to_string()],
            explorer_url: None,
            testnet: false,
            native_currency: "ETH".to_string(),
        }
    }

    /// Local Bitcoin node preset.
    pub fn local_bitcoin() -> Self {
        Self {
            id: "bitcoin-local".to_string(),
            name: "Local Bitcoin Node".to_string(),
            chain_type: ChainType::Bitcoin,
            chain_id: None,
            rpc_urls: vec!["http://localhost:8332".to_string()],
            explorer_url: None,
            testnet: false,
            native_currency: "BTC".to_string(),
        }
    }
}

/// Chain type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChainType {
    /// Ethereum and EVM-compatible chains
    Ethereum,
    /// Bitcoin
    Bitcoin,
    /// Solana
    Solana,
    /// Cosmos
    Cosmos,
    /// Other
    Other,
}

// ============================================================================
// Security Presets
// ============================================================================

/// Security configuration preset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityPreset {
    /// Preset name
    pub name: String,
    /// Require confirmation for signing
    pub require_signing_confirmation: bool,
    /// Require human-readable transaction display
    pub require_human_readable: bool,
    /// Require transaction simulation
    pub require_simulation: bool,
    /// Enable screenshot protection
    pub screenshot_protection: bool,
    /// Use OS keychain
    pub use_os_keychain: bool,
    /// Auto-lock timeout (seconds)
    pub auto_lock_seconds: Option<u32>,
    /// Allowed networks only
    pub strict_network_allowlist: bool,
}

impl SecurityPreset {
    /// Default security for wallet applications.
    pub fn wallet_default() -> Self {
        Self {
            name: "wallet-default".to_string(),
            require_signing_confirmation: true,
            require_human_readable: true,
            require_simulation: false,
            screenshot_protection: true,
            use_os_keychain: true,
            auto_lock_seconds: Some(300), // 5 minutes
            strict_network_allowlist: true,
        }
    }

    /// Strict security for high-value operations.
    pub fn wallet_strict() -> Self {
        Self {
            name: "wallet-strict".to_string(),
            require_signing_confirmation: true,
            require_human_readable: true,
            require_simulation: true,
            screenshot_protection: true,
            use_os_keychain: true,
            auto_lock_seconds: Some(60), // 1 minute
            strict_network_allowlist: true,
        }
    }

    /// Default security for dashboard applications.
    pub fn dashboard_default() -> Self {
        Self {
            name: "dashboard-default".to_string(),
            require_signing_confirmation: false,
            require_human_readable: false,
            require_simulation: false,
            screenshot_protection: false,
            use_os_keychain: false,
            auto_lock_seconds: None,
            strict_network_allowlist: false, // Allow localhost
        }
    }
}

// ============================================================================
// Starter Features
// ============================================================================

/// Features available in a starter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StarterFeature {
    // Wallet features
    /// Asset list and portfolio
    AssetList,
    /// Transaction history
    TransactionHistory,
    /// Send/receive flows
    SendReceive,
    /// Settings page
    Settings,
    /// Diagnostics export
    DiagnosticsExport,
    /// Token management
    TokenManagement,
    /// NFT gallery
    NftGallery,
    /// Address book
    AddressBook,

    // Node dashboard features
    /// Node health monitoring
    NodeHealth,
    /// Log viewer
    LogViewer,
    /// Sync status
    SyncStatus,
    /// Alert system
    Alerts,
    /// Peer information
    PeerInfo,
    /// Resource usage
    ResourceUsage,
}

impl std::fmt::Display for StarterFeature {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AssetList => write!(f, "Asset List"),
            Self::TransactionHistory => write!(f, "Transaction History"),
            Self::SendReceive => write!(f, "Send/Receive"),
            Self::Settings => write!(f, "Settings"),
            Self::DiagnosticsExport => write!(f, "Diagnostics Export"),
            Self::TokenManagement => write!(f, "Token Management"),
            Self::NftGallery => write!(f, "NFT Gallery"),
            Self::AddressBook => write!(f, "Address Book"),
            Self::NodeHealth => write!(f, "Node Health"),
            Self::LogViewer => write!(f, "Log Viewer"),
            Self::SyncStatus => write!(f, "Sync Status"),
            Self::Alerts => write!(f, "Alerts"),
            Self::PeerInfo => write!(f, "Peer Info"),
            Self::ResourceUsage => write!(f, "Resource Usage"),
        }
    }
}

// ============================================================================
// Generated Files
// ============================================================================

/// A file generated by a starter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// File path (relative to project root)
    pub path: String,
    /// File type
    pub file_type: GeneratedFileType,
    /// Description
    pub description: String,
    /// Template name (for looking up content)
    pub template: Option<String>,
}

impl GeneratedFile {
    /// Create a page file.
    pub fn page(path: &str, description: &str) -> Self {
        Self {
            path: path.to_string(),
            file_type: GeneratedFileType::Page,
            description: description.to_string(),
            template: None,
        }
    }

    /// Create a component file.
    pub fn component(path: &str, description: &str) -> Self {
        Self {
            path: path.to_string(),
            file_type: GeneratedFileType::Component,
            description: description.to_string(),
            template: None,
        }
    }

    /// Create a config file.
    pub fn config(path: &str, description: &str) -> Self {
        Self {
            path: path.to_string(),
            file_type: GeneratedFileType::Config,
            description: description.to_string(),
            template: None,
        }
    }

    /// Create a style file.
    pub fn style(path: &str, description: &str) -> Self {
        Self {
            path: path.to_string(),
            file_type: GeneratedFileType::Style,
            description: description.to_string(),
            template: None,
        }
    }
}

/// Type of generated file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GeneratedFileType {
    /// Page (full-page view)
    Page,
    /// Component
    Component,
    /// Layout
    Layout,
    /// Configuration
    Config,
    /// Style/theme
    Style,
    /// Asset (image, icon, etc.)
    Asset,
}

// ============================================================================
// Starter Registry
// ============================================================================

/// Registry of available crypto starters.
#[derive(Debug, Clone, Default)]
pub struct CryptoStarterRegistry {
    starters: HashMap<String, CryptoStarter>,
}

impl CryptoStarterRegistry {
    /// Create a new registry with built-in starters.
    pub fn with_builtin() -> Self {
        let mut registry = Self::default();
        for starter in CryptoStarter::all() {
            registry.register(starter);
        }
        registry
    }

    /// Register a starter.
    pub fn register(&mut self, starter: CryptoStarter) {
        self.starters.insert(starter.id.clone(), starter);
    }

    /// Get a starter by ID.
    pub fn get(&self, id: &str) -> Option<&CryptoStarter> {
        self.starters.get(id)
    }

    /// List all starters.
    pub fn list(&self) -> Vec<&CryptoStarter> {
        self.starters.values().collect()
    }

    /// List starters by category.
    pub fn by_category(&self, category: StarterCategory) -> Vec<&CryptoStarter> {
        self.starters
            .values()
            .filter(|s| s.category == category)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crypto_wallet_starter() {
        let starter = CryptoStarter::crypto_wallet();
        assert_eq!(starter.id, "crypto-wallet");
        assert_eq!(starter.category, StarterCategory::Wallet);
        assert!(!starter.plugins.is_empty());
        assert!(!starter.files.is_empty());
    }

    #[test]
    fn test_node_dashboard_starter() {
        let starter = CryptoStarter::node_dashboard();
        assert_eq!(starter.id, "node-dashboard");
        assert_eq!(starter.category, StarterCategory::NodeOperator);
    }

    #[test]
    fn test_plugin_requirement() {
        let required = PluginRequirement::required("crypto.core");
        assert!(required.required);

        let optional = PluginRequirement::optional("crypto.eth");
        assert!(!optional.required);
    }

    #[test]
    fn test_network_presets() {
        let eth_mainnet = NetworkPreset::ethereum_mainnet();
        assert_eq!(eth_mainnet.chain_id, Some(1));
        assert!(!eth_mainnet.testnet);

        let eth_sepolia = NetworkPreset::ethereum_sepolia();
        assert!(eth_sepolia.testnet);

        let btc_mainnet = NetworkPreset::bitcoin_mainnet();
        assert_eq!(btc_mainnet.chain_type, ChainType::Bitcoin);
        assert!(btc_mainnet.chain_id.is_none());
    }

    #[test]
    fn test_security_presets() {
        let wallet = SecurityPreset::wallet_default();
        assert!(wallet.require_signing_confirmation);
        assert!(wallet.use_os_keychain);
        assert!(wallet.screenshot_protection);

        let strict = SecurityPreset::wallet_strict();
        assert!(strict.require_simulation);
        assert_eq!(strict.auto_lock_seconds, Some(60));

        let dashboard = SecurityPreset::dashboard_default();
        assert!(!dashboard.require_signing_confirmation);
        assert!(!dashboard.screenshot_protection);
    }

    #[test]
    fn test_starter_registry() {
        let registry = CryptoStarterRegistry::with_builtin();

        assert!(registry.get("crypto-wallet").is_some());
        assert!(registry.get("node-dashboard").is_some());
        assert!(registry.get("nonexistent").is_none());

        let wallets = registry.by_category(StarterCategory::Wallet);
        assert_eq!(wallets.len(), 1);
    }

    #[test]
    fn test_generated_files() {
        let page = GeneratedFile::page("src/pages/home.oxi", "Home page");
        assert_eq!(page.file_type, GeneratedFileType::Page);

        let component = GeneratedFile::component("src/components/button.oxi", "Button");
        assert_eq!(component.file_type, GeneratedFileType::Component);

        let config = GeneratedFile::config("oxide.toml", "Config");
        assert_eq!(config.file_type, GeneratedFileType::Config);
    }
}
