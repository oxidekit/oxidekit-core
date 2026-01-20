//! Permission manifest schema for oxide.toml.
//!
//! Defines the `[permissions]`, `[capabilities]`, and `[network]` sections
//! that applications declare in their oxide.toml manifest.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::types::{Capability, CapabilityCategory, RiskLevel};
use crate::error::{PermissionError, PermissionResult};
use crate::network::NetworkPolicy;

/// The complete permissions manifest for an OxideKit application.
///
/// This is typically parsed from the `oxide.toml` file in the project root.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionManifest {
    /// Declared permissions grouped by plugin namespace.
    ///
    /// Example:
    /// ```toml
    /// [permissions]
    /// "native.filesystem" = ["filesystem.read"]
    /// "native.keychain" = ["keychain.access"]
    /// ```
    #[serde(default)]
    pub permissions: HashMap<String, Vec<String>>,

    /// Direct capability declarations with optional explanations.
    ///
    /// Example:
    /// ```toml
    /// [capabilities]
    /// filesystem.read = { reason = "Save user preferences" }
    /// network.http = { reason = "Fetch API data", domains = ["api.example.com"] }
    /// ```
    #[serde(default)]
    pub capabilities: HashMap<String, CapabilityDeclaration>,

    /// Network policy configuration.
    #[serde(default)]
    pub network: Option<NetworkPolicy>,

    /// Privacy and diagnostics settings.
    #[serde(default)]
    pub privacy: Option<PrivacySettings>,
}

/// A capability declaration with optional metadata.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityDeclaration {
    /// Human-readable reason for needing this capability.
    #[serde(default)]
    pub reason: Option<String>,

    /// Whether this capability is required or optional.
    #[serde(default)]
    pub required: bool,

    /// For network capabilities, the specific domains allowed.
    #[serde(default)]
    pub domains: Vec<String>,

    /// For filesystem capabilities, the specific paths allowed.
    #[serde(default)]
    pub paths: Vec<String>,

    /// Additional metadata for custom capabilities.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Privacy and diagnostics settings.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PrivacySettings {
    /// Whether automatic crash reporting is enabled.
    #[serde(default)]
    pub auto_crash_reporting: bool,

    /// Whether analytics collection is enabled.
    #[serde(default)]
    pub analytics_enabled: bool,

    /// Whether manual diagnostics export is allowed.
    #[serde(default = "default_true")]
    pub manual_export_allowed: bool,

    /// Whether telemetry requires user consent.
    #[serde(default = "default_true")]
    pub telemetry_requires_consent: bool,

    /// Data retention period in days (0 = no retention).
    #[serde(default)]
    pub data_retention_days: u32,
}

fn default_true() -> bool {
    true
}

impl PermissionManifest {
    /// Create a new empty permission manifest.
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a permission manifest from an oxide.toml file.
    pub fn from_file<P: AsRef<Path>>(path: P) -> PermissionResult<Self> {
        let content = std::fs::read_to_string(path.as_ref()).map_err(|e| {
            if e.kind() == std::io::ErrorKind::NotFound {
                PermissionError::FileNotFound(path.as_ref().to_path_buf())
            } else {
                PermissionError::IoError(e)
            }
        })?;

        Self::from_str(&content)
    }

    /// Parse a permission manifest from a TOML string.
    pub fn from_str(content: &str) -> PermissionResult<Self> {
        // Parse the TOML content
        let value: toml::Value = toml::from_str(content)?;

        // Extract the permissions-related sections
        let mut manifest = Self::new();

        if let Some(permissions) = value.get("permissions") {
            // Deserialize directly from the TOML Value to avoid re-parsing issues
            manifest.permissions = permissions.clone().try_into().map_err(|e| {
                crate::error::PermissionError::InvalidManifest(format!(
                    "Invalid permissions section: {}",
                    e
                ))
            })?;
        }

        if let Some(capabilities) = value.get("capabilities") {
            manifest.capabilities = capabilities.clone().try_into().map_err(|e| {
                crate::error::PermissionError::InvalidManifest(format!(
                    "Invalid capabilities section: {}",
                    e
                ))
            })?;
        }

        if let Some(network) = value.get("network") {
            manifest.network = Some(network.clone().try_into().map_err(|e| {
                crate::error::PermissionError::InvalidManifest(format!(
                    "Invalid network section: {}",
                    e
                ))
            })?);
        }

        if let Some(privacy) = value.get("privacy") {
            manifest.privacy = Some(privacy.clone().try_into().map_err(|e| {
                crate::error::PermissionError::InvalidManifest(format!(
                    "Invalid privacy section: {}",
                    e
                ))
            })?);
        }

        Ok(manifest)
    }

    /// Get all declared capabilities as a flat set.
    pub fn all_capabilities(&self) -> HashSet<Capability> {
        let mut caps = HashSet::new();

        // Add capabilities from permissions map
        for capabilities in self.permissions.values() {
            for cap in capabilities {
                caps.insert(Capability::new(cap.clone()));
            }
        }

        // Add capabilities from direct declarations
        for cap in self.capabilities.keys() {
            caps.insert(Capability::new(cap.clone()));
        }

        caps
    }

    /// Get capabilities grouped by category.
    pub fn capabilities_by_category(&self) -> HashMap<CapabilityCategory, Vec<Capability>> {
        let mut grouped: HashMap<CapabilityCategory, Vec<Capability>> = HashMap::new();

        for cap in self.all_capabilities() {
            let category = CapabilityCategory::from_capability(&cap);
            grouped.entry(category).or_default().push(cap);
        }

        grouped
    }

    /// Get the highest risk level among all declared capabilities.
    pub fn max_risk_level(&self) -> RiskLevel {
        self.all_capabilities()
            .iter()
            .map(|cap| CapabilityCategory::from_capability(cap).risk_level())
            .max()
            .unwrap_or(RiskLevel::Low)
    }

    /// Check if a specific capability is declared.
    pub fn has_capability(&self, capability: &str) -> bool {
        let target = Capability::new(capability);

        // Check direct capability declarations
        if self.capabilities.contains_key(capability) {
            return true;
        }

        // Check permissions map
        for caps in self.permissions.values() {
            if caps.iter().any(|c| {
                let cap = Capability::new(c.clone());
                cap == target || cap.matches(&target)
            }) {
                return true;
            }
        }

        false
    }

    /// Check if network access is allowed.
    pub fn allows_network(&self) -> bool {
        self.has_capability(Capability::NETWORK_HTTP)
            || self.has_capability(Capability::NETWORK_WEBSOCKET)
            || self.has_capability(Capability::NETWORK_FULL)
    }

    /// Get the reason for a capability if declared.
    pub fn capability_reason(&self, capability: &str) -> Option<&str> {
        self.capabilities
            .get(capability)
            .and_then(|decl| decl.reason.as_deref())
    }

    /// Check if the manifest declares network allowlist enforcement.
    pub fn has_network_allowlist(&self) -> bool {
        self.network
            .as_ref()
            .is_some_and(|n| n.mode == crate::network::NetworkMode::Allowlist)
    }

    /// Validate the manifest for consistency and correctness.
    pub fn validate(&self) -> PermissionResult<ValidationReport> {
        let mut report = ValidationReport::new();

        // Validate capability names
        for cap_name in self.capabilities.keys() {
            if let Err(e) = cap_name.parse::<Capability>() {
                report.add_error(format!("Invalid capability '{}': {}", cap_name, e));
            }
        }

        for caps in self.permissions.values() {
            for cap_name in caps {
                if let Err(e) = cap_name.parse::<Capability>() {
                    report.add_error(format!("Invalid capability '{}': {}", cap_name, e));
                }
            }
        }

        // Validate network policy if present
        if let Some(network) = &self.network {
            if network.mode == crate::network::NetworkMode::Allowlist && network.allow.is_empty() {
                report.add_warning(
                    "Network mode is 'allowlist' but no domains are allowed".to_string(),
                );
            }

            // Check for network capability when network policy is declared
            if !self.allows_network() {
                report.add_warning(
                    "Network policy declared but no network capability requested".to_string(),
                );
            }
        }

        // Check for high-risk capabilities without reasons
        for (cap_name, decl) in &self.capabilities {
            let cap = Capability::new(cap_name.clone());
            let category = CapabilityCategory::from_capability(&cap);
            if category.risk_level() >= RiskLevel::High && decl.reason.is_none() {
                report.add_warning(format!(
                    "High-risk capability '{}' has no declared reason",
                    cap_name
                ));
            }
        }

        Ok(report)
    }

    /// Serialize the manifest to TOML.
    pub fn to_toml(&self) -> PermissionResult<String> {
        toml::to_string_pretty(self).map_err(Into::into)
    }
}

/// Report from manifest validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationReport {
    /// Validation errors (manifest is invalid)
    pub errors: Vec<String>,
    /// Validation warnings (manifest is valid but has issues)
    pub warnings: Vec<String>,
}

impl ValidationReport {
    /// Create a new validation report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if validation passed without any issues.
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE_MANIFEST: &str = r#"
[permissions]
"native.filesystem" = ["filesystem.read"]
"native.keychain" = ["keychain.access"]
"native.network" = ["network.http"]

[capabilities]
"filesystem.read" = { reason = "Save user preferences" }
"network.http" = { reason = "Fetch API data", domains = ["api.example.com"] }

[network]
mode = "allowlist"
allow = ["api.example.com", "cdn.example.com"]
deny_private_ranges = true

[privacy]
auto_crash_reporting = false
analytics_enabled = false
manual_export_allowed = true
"#;

    #[test]
    fn test_parse_manifest() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();

        assert!(manifest.has_capability("filesystem.read"));
        assert!(manifest.has_capability("keychain.access"));
        assert!(manifest.has_capability("network.http"));
        assert!(!manifest.has_capability("camera.capture"));
    }

    #[test]
    fn test_all_capabilities() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();
        let caps = manifest.all_capabilities();

        assert!(caps.contains(&Capability::new("filesystem.read")));
        assert!(caps.contains(&Capability::new("keychain.access")));
        assert!(caps.contains(&Capability::new("network.http")));
    }

    #[test]
    fn test_network_allowlist() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();
        assert!(manifest.has_network_allowlist());

        let network = manifest.network.as_ref().unwrap();
        assert!(network.allow.contains(&"api.example.com".to_string()));
        assert!(network.deny_private_ranges);
    }

    #[test]
    fn test_privacy_settings() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();
        let privacy = manifest.privacy.as_ref().unwrap();

        assert!(!privacy.auto_crash_reporting);
        assert!(!privacy.analytics_enabled);
        assert!(privacy.manual_export_allowed);
    }

    #[test]
    fn test_validation() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();
        let report = manifest.validate().unwrap();
        assert!(report.is_valid());
    }

    #[test]
    fn test_capability_reason() {
        let manifest = PermissionManifest::from_str(SAMPLE_MANIFEST).unwrap();
        assert_eq!(
            manifest.capability_reason("filesystem.read"),
            Some("Save user preferences")
        );
    }
}
