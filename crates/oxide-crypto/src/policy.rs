//! Security guardrails and policy enforcement.
//!
//! This module provides types for:
//! - No-log enforcement
//! - Human-readable signing confirmation
//! - Transaction simulation hooks
//! - Explicit user confirmation
//! - Attestation integration
//!
//! # Verified Wallet Rules
//!
//! - Keychain required for key storage
//! - Network allowlist enforced
//! - Screenshots disabled by default

use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use chrono::{DateTime, Utc};
use uuid::Uuid;

// ============================================================================
// Signing Policy
// ============================================================================

/// Signing policy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningPolicy {
    /// Require user confirmation for all signing operations
    pub require_confirmation: bool,
    /// Require human-readable transaction display
    pub require_human_readable: bool,
    /// Require transaction simulation before signing
    pub require_simulation: bool,
    /// Maximum value that can be signed without additional verification
    pub max_value_without_review: Option<u64>,
    /// Allowed contract addresses (None = all allowed)
    pub allowed_contracts: Option<HashSet<String>>,
    /// Blocked contract addresses
    pub blocked_contracts: HashSet<String>,
    /// Allowed methods (for contract calls)
    pub allowed_methods: Option<HashSet<String>>,
    /// Time lock between signing requests (seconds)
    pub signing_cooldown_seconds: Option<u32>,
    /// Maximum signing requests per time period
    pub rate_limit: Option<SigningRateLimit>,
}

impl SigningPolicy {
    /// Create a new default policy.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a strict policy for high-value operations.
    pub fn strict() -> Self {
        Self {
            require_confirmation: true,
            require_human_readable: true,
            require_simulation: true,
            max_value_without_review: Some(0), // Always review
            allowed_contracts: None,
            blocked_contracts: HashSet::new(),
            allowed_methods: None,
            signing_cooldown_seconds: Some(5),
            rate_limit: Some(SigningRateLimit {
                max_requests: 10,
                period_seconds: 60,
            }),
        }
    }

    /// Create a permissive policy for development.
    pub fn development() -> Self {
        Self {
            require_confirmation: false,
            require_human_readable: false,
            require_simulation: false,
            max_value_without_review: None,
            allowed_contracts: None,
            blocked_contracts: HashSet::new(),
            allowed_methods: None,
            signing_cooldown_seconds: None,
            rate_limit: None,
        }
    }

    /// Set require_confirmation flag.
    pub fn require_confirmation(mut self, require: bool) -> Self {
        self.require_confirmation = require;
        self
    }

    /// Set require_human_readable flag.
    pub fn require_human_readable(mut self, require: bool) -> Self {
        self.require_human_readable = require;
        self
    }

    /// Set require_simulation flag.
    pub fn require_simulation(mut self, require: bool) -> Self {
        self.require_simulation = require;
        self
    }

    /// Add a blocked contract.
    pub fn block_contract(mut self, address: &str) -> Self {
        self.blocked_contracts.insert(address.to_lowercase());
        self
    }

    /// Check if a contract is allowed.
    pub fn is_contract_allowed(&self, address: &str) -> bool {
        let addr = address.to_lowercase();

        if self.blocked_contracts.contains(&addr) {
            return false;
        }

        if let Some(allowed) = &self.allowed_contracts {
            allowed.contains(&addr)
        } else {
            true
        }
    }
}

impl Default for SigningPolicy {
    fn default() -> Self {
        Self {
            require_confirmation: true,
            require_human_readable: true,
            require_simulation: false,
            max_value_without_review: None,
            allowed_contracts: None,
            blocked_contracts: HashSet::new(),
            allowed_methods: None,
            signing_cooldown_seconds: None,
            rate_limit: None,
        }
    }
}

/// Rate limit for signing requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRateLimit {
    /// Maximum requests in the period
    pub max_requests: u32,
    /// Period in seconds
    pub period_seconds: u32,
}

// ============================================================================
// Signing Request
// ============================================================================

/// A request to sign something.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningRequest {
    /// Unique request ID
    pub id: Uuid,
    /// Request type
    pub request_type: SigningRequestType,
    /// When the request was created
    pub created_at: DateTime<Utc>,
    /// Expiration time
    pub expires_at: Option<DateTime<Utc>>,
    /// Human-readable description
    pub description: String,
    /// Detailed information for display
    pub details: SigningDetails,
    /// Risk assessment
    pub risk: RiskAssessment,
    /// Current status
    pub status: SigningRequestStatus,
}

impl SigningRequest {
    /// Create a new signing request.
    pub fn new(request_type: SigningRequestType, description: String, details: SigningDetails) -> Self {
        Self {
            id: Uuid::new_v4(),
            request_type,
            created_at: Utc::now(),
            expires_at: Some(Utc::now() + chrono::Duration::minutes(5)),
            description,
            details,
            risk: RiskAssessment::default(),
            status: SigningRequestStatus::Pending,
        }
    }

    /// Check if the request has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Approve the request.
    pub fn approve(&mut self) -> CryptoResult<()> {
        if self.is_expired() {
            return Err(CryptoError::SigningFailed);
        }

        if self.status != SigningRequestStatus::Pending {
            return Err(CryptoError::SigningFailed);
        }

        self.status = SigningRequestStatus::Approved;
        Ok(())
    }

    /// Reject the request.
    pub fn reject(&mut self) {
        self.status = SigningRequestStatus::Rejected;
    }
}

/// Type of signing request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningRequestType {
    /// Sign a transaction
    Transaction,
    /// Sign a message
    Message,
    /// Sign typed data (EIP-712)
    TypedData,
    /// Sign a PSBT
    Psbt,
    /// Sign raw data
    Raw,
}

impl fmt::Display for SigningRequestType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transaction => write!(f, "Transaction"),
            Self::Message => write!(f, "Message"),
            Self::TypedData => write!(f, "Typed Data"),
            Self::Psbt => write!(f, "PSBT"),
            Self::Raw => write!(f, "Raw Data"),
        }
    }
}

/// Status of a signing request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SigningRequestStatus {
    /// Waiting for user action
    Pending,
    /// User approved
    Approved,
    /// User rejected
    Rejected,
    /// Request expired
    Expired,
    /// Signing completed
    Completed,
    /// Signing failed
    Failed,
}

/// Detailed information for signing request display.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SigningDetails {
    /// Ethereum transaction details
    EthereumTransaction {
        /// Chain name
        chain: String,
        /// From address
        from: String,
        /// To address
        to: Option<String>,
        /// Value in native currency
        value: String,
        /// Value in USD (if available)
        value_usd: Option<String>,
        /// Gas info
        gas: String,
        /// Function being called (if contract)
        function: Option<String>,
        /// Decoded parameters
        parameters: Option<Vec<ParameterDisplay>>,
    },
    /// Bitcoin transaction details
    BitcoinTransaction {
        /// Inputs
        inputs: Vec<BtcInputDisplay>,
        /// Outputs
        outputs: Vec<BtcOutputDisplay>,
        /// Fee
        fee: String,
        /// Fee rate
        fee_rate: String,
    },
    /// Message signing details
    Message {
        /// The message content
        content: String,
        /// Message hash
        hash: String,
    },
    /// EIP-712 typed data details
    TypedData {
        /// Domain name
        domain: String,
        /// Primary type
        primary_type: String,
        /// Formatted message
        formatted: String,
    },
    /// Raw data details
    Raw {
        /// Data hash
        hash: String,
        /// Data length
        length: usize,
    },
}

/// Display format for a parameter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDisplay {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Formatted value
    pub value: String,
}

/// Display format for Bitcoin input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcInputDisplay {
    /// Address
    pub address: String,
    /// Value
    pub value: String,
}

/// Display format for Bitcoin output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BtcOutputDisplay {
    /// Address
    pub address: String,
    /// Value
    pub value: String,
    /// Whether this is change
    pub is_change: bool,
}

// ============================================================================
// Risk Assessment
// ============================================================================

/// Risk assessment for a signing request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk level
    pub level: RiskLevel,
    /// Risk score (0-100)
    pub score: u8,
    /// Individual risk factors
    pub factors: Vec<RiskFactor>,
    /// Warnings to display
    pub warnings: Vec<String>,
    /// Whether the operation is blocked
    pub blocked: bool,
    /// Block reason (if blocked)
    pub block_reason: Option<String>,
}

impl RiskAssessment {
    /// Create a low-risk assessment.
    pub fn low() -> Self {
        Self {
            level: RiskLevel::Low,
            score: 10,
            factors: vec![],
            warnings: vec![],
            blocked: false,
            block_reason: None,
        }
    }

    /// Add a risk factor.
    pub fn add_factor(&mut self, factor: RiskFactor) {
        self.score = self.score.saturating_add(factor.score);
        self.factors.push(factor);
        self.update_level();
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Block the operation.
    pub fn block(&mut self, reason: String) {
        self.blocked = true;
        self.block_reason = Some(reason);
    }

    /// Update the risk level based on score.
    fn update_level(&mut self) {
        self.level = match self.score {
            0..=20 => RiskLevel::Low,
            21..=50 => RiskLevel::Medium,
            51..=80 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };
    }
}

impl Default for RiskAssessment {
    fn default() -> Self {
        Self::low()
    }
}

/// Risk level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// A risk factor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Factor identifier
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Description
    pub description: String,
    /// Score contribution
    pub score: u8,
}

impl RiskFactor {
    /// Create a new risk factor.
    pub fn new(id: &str, name: &str, description: &str, score: u8) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: description.to_string(),
            score,
        }
    }

    // Common risk factors

    /// First interaction with contract.
    pub fn first_interaction() -> Self {
        Self::new(
            "first_interaction",
            "First Interaction",
            "This is your first time interacting with this contract",
            15,
        )
    }

    /// Unverified contract.
    pub fn unverified_contract() -> Self {
        Self::new(
            "unverified_contract",
            "Unverified Contract",
            "The contract source code is not verified",
            25,
        )
    }

    /// Large value transfer.
    pub fn large_value() -> Self {
        Self::new(
            "large_value",
            "Large Value",
            "This transaction involves a large value",
            20,
        )
    }

    /// Token approval.
    pub fn token_approval() -> Self {
        Self::new(
            "token_approval",
            "Token Approval",
            "This grants permission to spend your tokens",
            15,
        )
    }

    /// Unlimited approval.
    pub fn unlimited_approval() -> Self {
        Self::new(
            "unlimited_approval",
            "Unlimited Approval",
            "This grants unlimited spending permission",
            40,
        )
    }

    /// Known phishing address.
    pub fn known_phishing() -> Self {
        Self::new(
            "known_phishing",
            "Known Phishing Address",
            "This address has been flagged as a phishing address",
            100,
        )
    }
}

// ============================================================================
// Attestation
// ============================================================================

/// Attestation configuration for a crypto application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationConfig {
    /// Application identifier
    pub app_id: String,
    /// Version
    pub version: String,
    /// Supported chains
    pub supported_chains: Vec<String>,
    /// Allowed network domains
    pub allowed_domains: Vec<String>,
    /// Key storage method
    pub key_storage: KeyStorageAttestation,
    /// Screenshot capability
    pub screenshot_capability: ScreenshotCapability,
    /// Build verification status
    pub build_verified: bool,
    /// Build hash
    pub build_hash: Option<String>,
    /// Signing timestamp
    pub signed_at: Option<DateTime<Utc>>,
}

impl AttestationConfig {
    /// Create a new attestation config.
    pub fn new(app_id: &str, version: &str) -> Self {
        Self {
            app_id: app_id.to_string(),
            version: version.to_string(),
            supported_chains: vec![],
            allowed_domains: vec![],
            key_storage: KeyStorageAttestation::OsKeychain,
            screenshot_capability: ScreenshotCapability::Disabled,
            build_verified: false,
            build_hash: None,
            signed_at: None,
        }
    }

    /// Add a supported chain.
    pub fn with_chain(mut self, chain: &str) -> Self {
        self.supported_chains.push(chain.to_string());
        self
    }

    /// Add an allowed domain.
    pub fn with_domain(mut self, domain: &str) -> Self {
        self.allowed_domains.push(domain.to_string());
        self
    }

    /// Generate attestation report.
    pub fn generate_report(&self) -> AttestationReport {
        AttestationReport {
            app_id: self.app_id.clone(),
            version: self.version.clone(),
            generated_at: Utc::now(),
            chains: self.supported_chains.clone(),
            allowed_domains: self.allowed_domains.clone(),
            key_storage: self.key_storage,
            screenshots_enabled: self.screenshot_capability != ScreenshotCapability::Disabled,
            build_verified: self.build_verified,
            build_hash: self.build_hash.clone(),
        }
    }
}

/// Key storage attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KeyStorageAttestation {
    /// OS-level keychain
    OsKeychain,
    /// Encrypted vault
    EncryptedVault,
    /// Hardware security module
    Hsm,
    /// Unknown/insecure
    Unknown,
}

impl fmt::Display for KeyStorageAttestation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OsKeychain => write!(f, "OS Keychain"),
            Self::EncryptedVault => write!(f, "Encrypted Vault"),
            Self::Hsm => write!(f, "HSM"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Screenshot capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScreenshotCapability {
    /// Screenshots are disabled
    Disabled,
    /// Screenshots are allowed
    Enabled,
    /// Screenshots are disabled for sensitive screens only
    SensitiveOnly,
}

/// Attestation report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReport {
    /// Application ID
    pub app_id: String,
    /// Application version
    pub version: String,
    /// Report generation time
    pub generated_at: DateTime<Utc>,
    /// Supported chains
    pub chains: Vec<String>,
    /// Allowed network domains
    pub allowed_domains: Vec<String>,
    /// Key storage method
    pub key_storage: KeyStorageAttestation,
    /// Whether screenshots are enabled
    pub screenshots_enabled: bool,
    /// Whether build is verified
    pub build_verified: bool,
    /// Build hash
    pub build_hash: Option<String>,
}

// ============================================================================
// Policy Enforcer
// ============================================================================

/// Policy enforcer that validates signing requests.
#[derive(Debug, Clone)]
pub struct PolicyEnforcer {
    /// Signing policy
    policy: SigningPolicy,
    /// Attestation config
    attestation: Option<AttestationConfig>,
}

impl PolicyEnforcer {
    /// Create a new policy enforcer.
    pub fn new(policy: SigningPolicy) -> Self {
        Self {
            policy,
            attestation: None,
        }
    }

    /// Set attestation config.
    pub fn with_attestation(mut self, attestation: AttestationConfig) -> Self {
        self.attestation = Some(attestation);
        self
    }

    /// Validate a signing request.
    pub fn validate(&self, request: &SigningRequest) -> CryptoResult<()> {
        // Check if expired
        if request.is_expired() {
            return Err(CryptoError::SigningFailed);
        }

        // Check risk assessment
        if request.risk.blocked {
            return Err(CryptoError::PolicyViolation {
                rule: request.risk.block_reason.clone().unwrap_or_default(),
            });
        }

        // Check if confirmation is required
        if self.policy.require_confirmation && request.status != SigningRequestStatus::Approved {
            return Err(CryptoError::SigningRequiresConfirmation);
        }

        Ok(())
    }

    /// Get the signing policy.
    pub fn policy(&self) -> &SigningPolicy {
        &self.policy
    }

    /// Get the attestation config.
    pub fn attestation(&self) -> Option<&AttestationConfig> {
        self.attestation.as_ref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_signing_policy_default() {
        let policy = SigningPolicy::default();
        assert!(policy.require_confirmation);
        assert!(policy.require_human_readable);
        assert!(!policy.require_simulation);
    }

    #[test]
    fn test_signing_policy_strict() {
        let policy = SigningPolicy::strict();
        assert!(policy.require_confirmation);
        assert!(policy.require_human_readable);
        assert!(policy.require_simulation);
        assert_eq!(policy.max_value_without_review, Some(0));
    }

    #[test]
    fn test_signing_policy_blocked_contracts() {
        let policy = SigningPolicy::new()
            .block_contract("0xbad");

        assert!(!policy.is_contract_allowed("0xbad"));
        assert!(!policy.is_contract_allowed("0xBAD")); // Case insensitive
        assert!(policy.is_contract_allowed("0xgood"));
    }

    #[test]
    fn test_signing_request_expiration() {
        let mut request = SigningRequest::new(
            SigningRequestType::Transaction,
            "Test".to_string(),
            SigningDetails::Message {
                content: "test".to_string(),
                hash: "0x".to_string(),
            },
        );

        assert!(!request.is_expired());

        // Expire it
        request.expires_at = Some(Utc::now() - chrono::Duration::minutes(1));
        assert!(request.is_expired());
    }

    #[test]
    fn test_risk_assessment() {
        let mut risk = RiskAssessment::low();
        assert_eq!(risk.level, RiskLevel::Low);

        risk.add_factor(RiskFactor::large_value());
        risk.add_factor(RiskFactor::first_interaction());
        assert!(risk.score > 30);
        assert!(matches!(risk.level, RiskLevel::Medium | RiskLevel::High));
    }

    #[test]
    fn test_risk_factor_phishing() {
        let mut risk = RiskAssessment::low();
        risk.add_factor(RiskFactor::known_phishing());
        assert_eq!(risk.level, RiskLevel::Critical);
    }

    #[test]
    fn test_attestation_config() {
        let config = AttestationConfig::new("my-wallet", "1.0.0")
            .with_chain("ethereum")
            .with_chain("bitcoin")
            .with_domain("*.infura.io");

        let report = config.generate_report();
        assert_eq!(report.chains.len(), 2);
        assert_eq!(report.allowed_domains.len(), 1);
        assert!(!report.screenshots_enabled);
    }

    #[test]
    fn test_policy_enforcer() {
        let policy = SigningPolicy::default();
        let enforcer = PolicyEnforcer::new(policy);

        let request = SigningRequest::new(
            SigningRequestType::Message,
            "Test".to_string(),
            SigningDetails::Message {
                content: "test".to_string(),
                hash: "0x".to_string(),
            },
        );

        // Should fail because confirmation is required
        let result = enforcer.validate(&request);
        assert!(matches!(result, Err(CryptoError::SigningRequiresConfirmation)));
    }

    #[test]
    fn test_policy_enforcer_approved() {
        let policy = SigningPolicy::default();
        let enforcer = PolicyEnforcer::new(policy);

        let mut request = SigningRequest::new(
            SigningRequestType::Message,
            "Test".to_string(),
            SigningDetails::Message {
                content: "test".to_string(),
                hash: "0x".to_string(),
            },
        );

        request.approve().unwrap();
        let result = enforcer.validate(&request);
        assert!(result.is_ok());
    }
}
