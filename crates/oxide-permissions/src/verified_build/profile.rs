//! Verified build profile configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Verified build profile configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedBuildProfile {
    /// Profile name.
    pub name: String,
    /// Profile description.
    pub description: String,
    /// Whether manifest permissions are required.
    #[serde(default = "default_true")]
    pub require_manifest_permissions: bool,
    /// Whether network policy is required when network is used.
    #[serde(default = "default_true")]
    pub require_network_policy: bool,
    /// Whether to enforce plugin trust rules.
    #[serde(default = "default_true")]
    pub enforce_plugin_trust: bool,
    /// Whether to check for forbidden APIs.
    #[serde(default = "default_true")]
    pub check_forbidden_apis: bool,
    /// Whether to check for forbidden crates.
    #[serde(default = "default_true")]
    pub check_forbidden_crates: bool,
    /// Additional forbidden APIs (beyond defaults).
    #[serde(default)]
    pub additional_forbidden_apis: HashSet<String>,
    /// Additional forbidden crates (beyond defaults).
    #[serde(default)]
    pub additional_forbidden_crates: HashSet<String>,
    /// Allowed exceptions for specific APIs.
    #[serde(default)]
    pub api_exceptions: HashSet<String>,
    /// Allowed exceptions for specific crates.
    #[serde(default)]
    pub crate_exceptions: HashSet<String>,
    /// Minimum required OxideKit version.
    pub min_oxidekit_version: Option<String>,
    /// Required compiler features.
    #[serde(default)]
    pub required_features: HashSet<String>,
    /// Build flags that must be enabled.
    #[serde(default)]
    pub required_build_flags: HashSet<String>,
}

fn default_true() -> bool {
    true
}

impl Default for VerifiedBuildProfile {
    fn default() -> Self {
        Self::standard()
    }
}

impl VerifiedBuildProfile {
    /// Create a new empty profile.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            require_manifest_permissions: true,
            require_network_policy: true,
            enforce_plugin_trust: true,
            check_forbidden_apis: true,
            check_forbidden_crates: true,
            additional_forbidden_apis: HashSet::new(),
            additional_forbidden_crates: HashSet::new(),
            api_exceptions: HashSet::new(),
            crate_exceptions: HashSet::new(),
            min_oxidekit_version: None,
            required_features: HashSet::new(),
            required_build_flags: HashSet::new(),
        }
    }

    /// Create the standard verified build profile.
    pub fn standard() -> Self {
        Self {
            name: "standard".to_string(),
            description: "Standard verified build profile with recommended security settings"
                .to_string(),
            require_manifest_permissions: true,
            require_network_policy: true,
            enforce_plugin_trust: true,
            check_forbidden_apis: true,
            check_forbidden_crates: true,
            additional_forbidden_apis: HashSet::new(),
            additional_forbidden_crates: HashSet::new(),
            api_exceptions: HashSet::new(),
            crate_exceptions: HashSet::new(),
            min_oxidekit_version: Some("0.1.0".to_string()),
            required_features: HashSet::new(),
            required_build_flags: HashSet::new(),
        }
    }

    /// Create a strict profile with maximum security.
    pub fn strict() -> Self {
        let mut profile = Self::standard();
        profile.name = "strict".to_string();
        profile.description = "Strict verified build profile with maximum security".to_string();

        // Add additional forbidden crates
        profile.additional_forbidden_crates.extend([
            "unsafe_code".to_string(),
            "libc".to_string(), // Direct libc access
        ]);

        // Require additional security features
        profile.required_build_flags.extend([
            "-C overflow-checks=on".to_string(),
            "-C debug-assertions=on".to_string(),
        ]);

        profile
    }

    /// Create a permissive profile for development.
    pub fn permissive() -> Self {
        Self {
            name: "permissive".to_string(),
            description: "Permissive profile for development (not for production)".to_string(),
            require_manifest_permissions: false,
            require_network_policy: false,
            enforce_plugin_trust: false,
            check_forbidden_apis: false,
            check_forbidden_crates: false,
            additional_forbidden_apis: HashSet::new(),
            additional_forbidden_crates: HashSet::new(),
            api_exceptions: HashSet::new(),
            crate_exceptions: HashSet::new(),
            min_oxidekit_version: None,
            required_features: HashSet::new(),
            required_build_flags: HashSet::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a forbidden API.
    pub fn forbid_api(mut self, api: impl Into<String>) -> Self {
        self.additional_forbidden_apis.insert(api.into());
        self
    }

    /// Add a forbidden crate.
    pub fn forbid_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.additional_forbidden_crates.insert(crate_name.into());
        self
    }

    /// Add an API exception.
    pub fn allow_api(mut self, api: impl Into<String>) -> Self {
        self.api_exceptions.insert(api.into());
        self
    }

    /// Add a crate exception.
    pub fn allow_crate(mut self, crate_name: impl Into<String>) -> Self {
        self.crate_exceptions.insert(crate_name.into());
        self
    }

    /// Set minimum OxideKit version.
    pub fn min_version(mut self, version: impl Into<String>) -> Self {
        self.min_oxidekit_version = Some(version.into());
        self
    }

    /// Add a required feature.
    pub fn require_feature(mut self, feature: impl Into<String>) -> Self {
        self.required_features.insert(feature.into());
        self
    }

    /// Add a required build flag.
    pub fn require_build_flag(mut self, flag: impl Into<String>) -> Self {
        self.required_build_flags.insert(flag.into());
        self
    }

    /// Check if an API is forbidden.
    pub fn is_api_forbidden(&self, api: &str) -> bool {
        if !self.check_forbidden_apis {
            return false;
        }
        if self.api_exceptions.contains(api) {
            return false;
        }
        self.additional_forbidden_apis.contains(api)
            || super::forbidden::FORBIDDEN_APIS.contains(&api)
    }

    /// Check if a crate is forbidden.
    pub fn is_crate_forbidden(&self, crate_name: &str) -> bool {
        if !self.check_forbidden_crates {
            return false;
        }
        if self.crate_exceptions.contains(crate_name) {
            return false;
        }
        self.additional_forbidden_crates.contains(crate_name)
            || super::forbidden::FORBIDDEN_CRATES.contains(&crate_name)
    }

    /// Serialize to TOML.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    /// Deserialize from TOML.
    pub fn from_toml(content: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(content)
    }
}

/// Build output metadata for attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildMetadata {
    /// Build timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Build profile used.
    pub profile: String,
    /// OxideKit version.
    pub oxidekit_version: String,
    /// Rust version.
    pub rust_version: String,
    /// Target triple.
    pub target: String,
    /// Build features enabled.
    pub features: Vec<String>,
    /// Build flags used.
    pub build_flags: Vec<String>,
    /// Hash of the output binary.
    pub binary_hash: Option<String>,
    /// Verification checks passed.
    pub checks_passed: Vec<String>,
    /// Warnings during verification.
    pub warnings: Vec<String>,
    /// Whether the build is verified.
    pub is_verified: bool,
}

impl BuildMetadata {
    /// Create new build metadata.
    pub fn new(profile: impl Into<String>, oxidekit_version: impl Into<String>) -> Self {
        Self {
            timestamp: chrono::Utc::now(),
            profile: profile.into(),
            oxidekit_version: oxidekit_version.into(),
            rust_version: String::new(),
            target: String::new(),
            features: Vec::new(),
            build_flags: Vec::new(),
            binary_hash: None,
            checks_passed: Vec::new(),
            warnings: Vec::new(),
            is_verified: false,
        }
    }

    /// Set the Rust version.
    pub fn with_rust_version(mut self, version: impl Into<String>) -> Self {
        self.rust_version = version.into();
        self
    }

    /// Set the target triple.
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = target.into();
        self
    }

    /// Add a feature.
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Add a build flag.
    pub fn with_build_flag(mut self, flag: impl Into<String>) -> Self {
        self.build_flags.push(flag.into());
        self
    }

    /// Set the binary hash.
    pub fn with_binary_hash(mut self, hash: impl Into<String>) -> Self {
        self.binary_hash = Some(hash.into());
        self
    }

    /// Record a passed check.
    pub fn check_passed(&mut self, check: impl Into<String>) {
        self.checks_passed.push(check.into());
    }

    /// Record a warning.
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Mark as verified.
    pub fn mark_verified(&mut self) {
        self.is_verified = true;
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(content: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_profile() {
        let profile = VerifiedBuildProfile::standard();
        assert!(profile.require_manifest_permissions);
        assert!(profile.check_forbidden_apis);
    }

    #[test]
    fn test_strict_profile() {
        let profile = VerifiedBuildProfile::strict();
        assert!(profile.is_crate_forbidden("libc"));
    }

    #[test]
    fn test_permissive_profile() {
        let profile = VerifiedBuildProfile::permissive();
        assert!(!profile.require_manifest_permissions);
        assert!(!profile.check_forbidden_apis);
    }

    #[test]
    fn test_custom_profile() {
        let profile = VerifiedBuildProfile::new("custom")
            .forbid_api("std::process::Command")
            .allow_crate("allowed_crate");

        assert!(profile.is_api_forbidden("std::process::Command"));
        assert!(!profile.is_crate_forbidden("allowed_crate"));
    }

    #[test]
    fn test_build_metadata() {
        let mut metadata = BuildMetadata::new("standard", "0.1.0")
            .with_rust_version("1.75.0")
            .with_target("x86_64-unknown-linux-gnu");

        metadata.check_passed("manifest_valid");
        metadata.mark_verified();

        assert!(metadata.is_verified);
        assert!(metadata.checks_passed.contains(&"manifest_valid".to_string()));
    }
}
