//! Trust levels and verification policies.
//!
//! OxideKit uses a tiered trust system to prevent "npm malware 2.0":
//!
//! - **Official**: Maintained by OxideKit org, full native access
//! - **Verified**: Identity verified, signed releases, capability review
//! - **Community**: Sandbox by default, clear warnings
//!
//! # Execution Lanes
//!
//! - **Lane A (Native)**: Compiled Rust, only for Official/Verified
//! - **Lane B (Sandbox)**: WASM execution for Community plugins

use serde::{Deserialize, Serialize};

/// Trust level assigned to plugins and publishers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrustLevel {
    /// Community plugins - sandbox by default, clear warnings.
    Community,
    /// Verified publishers - identity verified, signed releases.
    Verified,
    /// Official plugins - maintained by OxideKit org.
    Official,
}

impl TrustLevel {
    /// Get a human-readable description.
    pub fn description(&self) -> &'static str {
        match self {
            TrustLevel::Community => "Community plugin - runs in sandbox",
            TrustLevel::Verified => "Verified publisher - signed releases",
            TrustLevel::Official => "Official OxideKit plugin",
        }
    }

    /// Check if native execution is allowed.
    pub fn allows_native_execution(&self) -> bool {
        matches!(self, TrustLevel::Verified | TrustLevel::Official)
    }

    /// Check if this trust level can access all capabilities.
    pub fn has_full_capability_access(&self) -> bool {
        matches!(self, TrustLevel::Official)
    }

    /// Get the execution lane for this trust level.
    pub fn execution_lane(&self) -> ExecutionLane {
        match self {
            TrustLevel::Community => ExecutionLane::Sandbox,
            TrustLevel::Verified => ExecutionLane::Native,
            TrustLevel::Official => ExecutionLane::Native,
        }
    }

    /// Get the display color (for CLI output).
    pub fn color(&self) -> &'static str {
        match self {
            TrustLevel::Community => "yellow",
            TrustLevel::Verified => "blue",
            TrustLevel::Official => "green",
        }
    }

    /// Get the badge text for marketplace display.
    pub fn badge(&self) -> &'static str {
        match self {
            TrustLevel::Community => "Community",
            TrustLevel::Verified => "Verified",
            TrustLevel::Official => "Official",
        }
    }
}

impl Default for TrustLevel {
    fn default() -> Self {
        TrustLevel::Community
    }
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TrustLevel::Community => write!(f, "community"),
            TrustLevel::Verified => write!(f, "verified"),
            TrustLevel::Official => write!(f, "official"),
        }
    }
}

/// Execution lane for plugin code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExecutionLane {
    /// Native execution - compiled Rust dynamic libraries.
    Native,
    /// Sandboxed execution - WASM with restricted capabilities.
    Sandbox,
}

impl ExecutionLane {
    /// Get a description of this execution lane.
    pub fn description(&self) -> &'static str {
        match self {
            ExecutionLane::Native => "Native Rust code with full system access",
            ExecutionLane::Sandbox => "WASM sandbox with restricted capabilities",
        }
    }
}

impl std::fmt::Display for ExecutionLane {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionLane::Native => write!(f, "native"),
            ExecutionLane::Sandbox => write!(f, "sandbox"),
        }
    }
}

/// Policy for determining trust levels and execution behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustPolicy {
    /// Whether to allow native execution for verified plugins.
    pub allow_verified_native: bool,
    /// Whether to require signatures for verified plugins.
    pub require_signatures: bool,
    /// Whether to allow community plugins at all.
    pub allow_community: bool,
    /// Whether to show warnings for community plugins.
    pub show_community_warnings: bool,
    /// Minimum trust level for plugins with dangerous capabilities.
    pub dangerous_capability_minimum: TrustLevel,
    /// Custom trusted publishers (treated as Verified).
    pub trusted_publishers: Vec<String>,
    /// Blocked publishers.
    pub blocked_publishers: Vec<String>,
}

impl Default for TrustPolicy {
    fn default() -> Self {
        Self {
            allow_verified_native: true,
            require_signatures: true,
            allow_community: true,
            show_community_warnings: true,
            dangerous_capability_minimum: TrustLevel::Verified,
            trusted_publishers: Vec::new(),
            blocked_publishers: Vec::new(),
        }
    }
}

impl TrustPolicy {
    /// Create a strict policy that only allows Official plugins.
    pub fn strict() -> Self {
        Self {
            allow_verified_native: false,
            require_signatures: true,
            allow_community: false,
            show_community_warnings: true,
            dangerous_capability_minimum: TrustLevel::Official,
            trusted_publishers: Vec::new(),
            blocked_publishers: Vec::new(),
        }
    }

    /// Create a permissive policy for development.
    pub fn development() -> Self {
        Self {
            allow_verified_native: true,
            require_signatures: false,
            allow_community: true,
            show_community_warnings: false,
            dangerous_capability_minimum: TrustLevel::Community,
            trusted_publishers: Vec::new(),
            blocked_publishers: Vec::new(),
        }
    }

    /// Check if a publisher is trusted.
    pub fn is_publisher_trusted(&self, publisher: &str) -> bool {
        // Check if official
        if publisher.starts_with("oxidekit") {
            return true;
        }

        // Check trusted publishers list
        self.trusted_publishers.contains(&publisher.to_string())
    }

    /// Check if a publisher is blocked.
    pub fn is_publisher_blocked(&self, publisher: &str) -> bool {
        self.blocked_publishers.contains(&publisher.to_string())
    }

    /// Determine the effective trust level for a plugin.
    pub fn effective_trust_level(&self, publisher: &str, declared_level: TrustLevel) -> TrustLevel {
        // Check for blocked publishers
        if self.is_publisher_blocked(publisher) {
            return TrustLevel::Community;
        }

        // Official publishers
        if publisher.starts_with("oxidekit") {
            return TrustLevel::Official;
        }

        // Trusted publishers get Verified
        if self.is_publisher_trusted(publisher) {
            return TrustLevel::Verified;
        }

        // Use the lower of declared level and Verified (can't self-declare as Official)
        if declared_level == TrustLevel::Official {
            TrustLevel::Verified
        } else {
            declared_level
        }
    }

    /// Check if a plugin can use native execution.
    pub fn can_use_native(&self, trust_level: TrustLevel) -> bool {
        match trust_level {
            TrustLevel::Official => true,
            TrustLevel::Verified => self.allow_verified_native,
            TrustLevel::Community => false,
        }
    }

    /// Check if a plugin is allowed to install.
    pub fn can_install(&self, trust_level: TrustLevel) -> bool {
        match trust_level {
            TrustLevel::Official | TrustLevel::Verified => true,
            TrustLevel::Community => self.allow_community,
        }
    }
}

/// Information about a verified publisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedPublisher {
    /// Publisher name/ID.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Publisher website.
    pub website: Option<String>,
    /// Public key for signature verification.
    pub public_key: String,
    /// Date of verification.
    pub verified_at: chrono::DateTime<chrono::Utc>,
    /// Verification expiry date.
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl VerifiedPublisher {
    /// Check if the verification is still valid.
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expires) => chrono::Utc::now() < expires,
            None => true,
        }
    }
}

/// Signature for a plugin release.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseSignature {
    /// Plugin ID.
    pub plugin_id: String,
    /// Plugin version.
    pub version: String,
    /// Hash of the plugin package.
    pub package_hash: String,
    /// Signature bytes (base64 encoded).
    pub signature: String,
    /// Signing timestamp.
    pub signed_at: chrono::DateTime<chrono::Utc>,
    /// Publisher who signed.
    pub signed_by: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trust_level_ordering() {
        assert!(TrustLevel::Community < TrustLevel::Verified);
        assert!(TrustLevel::Verified < TrustLevel::Official);
    }

    #[test]
    fn test_execution_lanes() {
        assert_eq!(TrustLevel::Community.execution_lane(), ExecutionLane::Sandbox);
        assert_eq!(TrustLevel::Verified.execution_lane(), ExecutionLane::Native);
        assert_eq!(TrustLevel::Official.execution_lane(), ExecutionLane::Native);
    }

    #[test]
    fn test_trust_policy() {
        let policy = TrustPolicy::default();

        assert!(policy.can_use_native(TrustLevel::Official));
        assert!(policy.can_use_native(TrustLevel::Verified));
        assert!(!policy.can_use_native(TrustLevel::Community));

        let strict = TrustPolicy::strict();
        assert!(!strict.can_install(TrustLevel::Community));
    }

    #[test]
    fn test_effective_trust_level() {
        let policy = TrustPolicy::default();

        // Official publisher
        assert_eq!(
            policy.effective_trust_level("oxidekit", TrustLevel::Community),
            TrustLevel::Official
        );

        // Can't self-declare as Official
        assert_eq!(
            policy.effective_trust_level("random", TrustLevel::Official),
            TrustLevel::Verified
        );
    }
}
