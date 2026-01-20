//! Attestation report format.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use chrono::{DateTime, Utc};

use crate::capabilities::RiskLevel;

/// Complete attestation report for an OxideKit application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationReport {
    /// Report format version.
    pub version: String,
    /// Report generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Application information.
    pub app: AttestationAppInfo,
    /// Binary information.
    pub binary: BinaryInfo,
    /// Signature verification status.
    pub signature: SignatureStatus,
    /// SBOM (Software Bill of Materials) summary.
    pub sbom: SbomSummary,
    /// Declared permissions summary.
    pub permissions: PermissionsSummary,
    /// Network policy summary.
    pub network: NetworkSummary,
    /// Privacy settings summary.
    pub privacy: PrivacySummary,
    /// Verification checks.
    pub checks: Vec<AttestationCheck>,
    /// Trust classification.
    pub trust_classification: TrustClassification,
    /// Badges earned.
    pub badges: Vec<AttestationBadge>,
    /// Overall attestation status.
    pub status: AttestationStatus,
    /// Additional metadata.
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Application information for attestation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationAppInfo {
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// Publisher/developer.
    pub publisher: Option<String>,
    /// Application identifier.
    pub app_id: Option<String>,
    /// Website URL.
    pub website: Option<String>,
    /// Repository URL.
    pub repository: Option<String>,
}

/// Binary information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryInfo {
    /// Binary file name.
    pub filename: String,
    /// Binary size in bytes.
    pub size_bytes: u64,
    /// SHA-256 hash of the binary.
    pub sha256: String,
    /// Target platform.
    pub target: String,
    /// Build timestamp (if available).
    pub build_time: Option<DateTime<Utc>>,
    /// OxideKit version used.
    pub oxidekit_version: Option<String>,
    /// Rust version used.
    pub rust_version: Option<String>,
}

/// Signature verification status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignatureStatus {
    /// Whether the binary is signed.
    pub is_signed: bool,
    /// Signature algorithm used.
    pub algorithm: Option<String>,
    /// Signer identity.
    pub signer: Option<String>,
    /// Signature timestamp.
    pub signed_at: Option<DateTime<Utc>>,
    /// Whether signature is valid.
    pub is_valid: bool,
    /// Verification error (if any).
    pub error: Option<String>,
}

impl Default for SignatureStatus {
    fn default() -> Self {
        Self {
            is_signed: false,
            algorithm: None,
            signer: None,
            signed_at: None,
            is_valid: false,
            error: None,
        }
    }
}

/// SBOM summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomSummary {
    /// Total number of dependencies.
    pub total_dependencies: usize,
    /// Direct dependencies.
    pub direct_dependencies: usize,
    /// Transitive dependencies.
    pub transitive_dependencies: usize,
    /// Dependencies with known vulnerabilities.
    pub vulnerable_dependencies: usize,
    /// License summary.
    pub licenses: HashMap<String, usize>,
    /// High-risk dependencies flagged.
    pub flagged_dependencies: Vec<FlaggedDependency>,
}

impl Default for SbomSummary {
    fn default() -> Self {
        Self {
            total_dependencies: 0,
            direct_dependencies: 0,
            transitive_dependencies: 0,
            vulnerable_dependencies: 0,
            licenses: HashMap::new(),
            flagged_dependencies: Vec::new(),
        }
    }
}

/// A flagged dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlaggedDependency {
    /// Dependency name.
    pub name: String,
    /// Version.
    pub version: String,
    /// Reason for flagging.
    pub reason: String,
    /// Severity.
    pub severity: String,
}

/// Permissions summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionsSummary {
    /// Total capabilities declared.
    pub total_capabilities: usize,
    /// Capabilities by category.
    pub by_category: HashMap<String, Vec<String>>,
    /// Highest risk level.
    pub max_risk_level: RiskLevel,
    /// High-risk capabilities.
    pub high_risk_capabilities: Vec<String>,
    /// Whether all capabilities have reasons.
    pub all_have_reasons: bool,
}

impl Default for PermissionsSummary {
    fn default() -> Self {
        Self {
            total_capabilities: 0,
            by_category: HashMap::new(),
            max_risk_level: RiskLevel::Low,
            high_risk_capabilities: Vec::new(),
            all_have_reasons: false,
        }
    }
}

/// Network policy summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummary {
    /// Whether network is used.
    pub uses_network: bool,
    /// Whether allowlist is enforced.
    pub allowlist_enforced: bool,
    /// Enforcement status text.
    pub enforcement_status: String,
    /// Allowed domains (if enforced).
    pub allowed_domains: Vec<String>,
    /// Whether private ranges are blocked.
    pub blocks_private_ranges: bool,
    /// Whether HTTPS is required.
    pub requires_https: bool,
}

impl Default for NetworkSummary {
    fn default() -> Self {
        Self {
            uses_network: false,
            allowlist_enforced: false,
            enforcement_status: "unknown".to_string(),
            allowed_domains: Vec::new(),
            blocks_private_ranges: false,
            requires_https: false,
        }
    }
}

/// Privacy settings summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySummary {
    /// Auto crash reporting enabled.
    pub auto_crash_reporting: bool,
    /// Analytics enabled.
    pub analytics_enabled: bool,
    /// Manual export allowed.
    pub manual_export_allowed: bool,
    /// Telemetry requires consent.
    pub telemetry_requires_consent: bool,
    /// Data retention period.
    pub data_retention: Option<String>,
    /// Privacy score (0-100).
    pub privacy_score: u8,
}

impl Default for PrivacySummary {
    fn default() -> Self {
        Self {
            auto_crash_reporting: false,
            analytics_enabled: false,
            manual_export_allowed: true,
            telemetry_requires_consent: true,
            data_retention: None,
            privacy_score: 100,
        }
    }
}

/// An attestation check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationCheck {
    /// Check name.
    pub name: String,
    /// Check description.
    pub description: String,
    /// Whether the check passed.
    pub passed: bool,
    /// Check result details.
    pub details: Option<String>,
    /// Check severity (for failures).
    pub severity: CheckSeverity,
}

/// Severity of a check failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckSeverity {
    /// Informational only.
    Info,
    /// Warning - doesn't fail attestation.
    Warning,
    /// Error - fails attestation.
    Error,
    /// Critical - fails attestation and prevents distribution.
    Critical,
}

/// Trust classification result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustClassification {
    /// Trust level.
    pub level: TrustLevel,
    /// Classification reasons.
    pub reasons: Vec<String>,
    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
}

/// Trust level for attestation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrustLevel {
    /// Untrusted - significant issues found.
    Untrusted,
    /// Unknown - insufficient information.
    Unknown,
    /// Basic - meets minimum requirements.
    Basic,
    /// Verified - meets all verification requirements.
    Verified,
    /// Official - from OxideKit organization.
    Official,
}

impl std::fmt::Display for TrustLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Untrusted => write!(f, "Untrusted"),
            Self::Unknown => write!(f, "Unknown"),
            Self::Basic => write!(f, "Basic"),
            Self::Verified => write!(f, "Verified"),
            Self::Official => write!(f, "Official"),
        }
    }
}

/// An attestation badge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttestationBadge {
    /// Badge identifier.
    pub id: String,
    /// Badge name.
    pub name: String,
    /// Badge description.
    pub description: String,
    /// Whether badge is earned.
    pub earned: bool,
    /// Badge icon.
    pub icon: String,
    /// Badge category.
    pub category: String,
}

/// Overall attestation status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttestationStatus {
    /// Attestation passed all checks.
    Passed,
    /// Attestation passed with warnings.
    PassedWithWarnings,
    /// Attestation failed.
    Failed,
    /// Attestation could not be completed.
    Incomplete,
}

impl AttestationReport {
    /// Create a new attestation report.
    pub fn new(app_name: impl Into<String>, app_version: impl Into<String>) -> Self {
        Self {
            version: "1.0".to_string(),
            generated_at: Utc::now(),
            app: AttestationAppInfo {
                name: app_name.into(),
                version: app_version.into(),
                publisher: None,
                app_id: None,
                website: None,
                repository: None,
            },
            binary: BinaryInfo {
                filename: String::new(),
                size_bytes: 0,
                sha256: String::new(),
                target: String::new(),
                build_time: None,
                oxidekit_version: None,
                rust_version: None,
            },
            signature: SignatureStatus::default(),
            sbom: SbomSummary::default(),
            permissions: PermissionsSummary::default(),
            network: NetworkSummary::default(),
            privacy: PrivacySummary::default(),
            checks: Vec::new(),
            trust_classification: TrustClassification {
                level: TrustLevel::Unknown,
                reasons: Vec::new(),
                recommendations: Vec::new(),
            },
            badges: Vec::new(),
            status: AttestationStatus::Incomplete,
            metadata: std::collections::HashMap::new(),
        }
    }

    /// Set binary information.
    pub fn with_binary(mut self, binary: BinaryInfo) -> Self {
        self.binary = binary;
        self
    }

    /// Add a check result.
    pub fn add_check(&mut self, check: AttestationCheck) {
        self.checks.push(check);
    }

    /// Add a badge.
    pub fn add_badge(&mut self, badge: AttestationBadge) {
        self.badges.push(badge);
    }

    /// Finalize the report and compute status.
    pub fn finalize(&mut self) {
        // Compute status based on checks
        let has_critical = self
            .checks
            .iter()
            .any(|c| !c.passed && c.severity == CheckSeverity::Critical);
        let has_error = self
            .checks
            .iter()
            .any(|c| !c.passed && c.severity == CheckSeverity::Error);
        let has_warning = self
            .checks
            .iter()
            .any(|c| !c.passed && c.severity == CheckSeverity::Warning);

        self.status = if has_critical || has_error {
            AttestationStatus::Failed
        } else if has_warning {
            AttestationStatus::PassedWithWarnings
        } else {
            AttestationStatus::Passed
        };

        // Compute trust level
        self.trust_classification.level = if has_critical {
            TrustLevel::Untrusted
        } else if has_error {
            TrustLevel::Unknown
        } else if self.signature.is_valid && self.network.allowlist_enforced {
            TrustLevel::Verified
        } else if !has_warning {
            TrustLevel::Basic
        } else {
            TrustLevel::Unknown
        };
    }

    /// Generate a human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!(
            "Attestation Report: {} v{}",
            self.app.name, self.app.version
        ));
        lines.push(format!("Generated: {}", self.generated_at));
        lines.push(format!("Status: {:?}", self.status));
        lines.push(format!(
            "Trust Level: {}",
            self.trust_classification.level
        ));
        lines.push(String::new());

        // Permissions
        lines.push("Permissions:".to_string());
        for (category, caps) in &self.permissions.by_category {
            for cap in caps {
                lines.push(format!("  - {}: {}", category, cap));
            }
        }
        lines.push(String::new());

        // Network
        lines.push("Network:".to_string());
        if self.network.allowlist_enforced {
            lines.push(format!(
                "  Allowlist enforced: {}",
                self.network.allowed_domains.join(", ")
            ));
        } else {
            lines.push("  Domains: unknown (policy not enforced)".to_string());
        }
        lines.push(String::new());

        // Privacy
        lines.push("Privacy:".to_string());
        lines.push(format!(
            "  Crash reporting: {}",
            if self.privacy.auto_crash_reporting {
                "auto"
            } else {
                "disabled"
            }
        ));
        lines.push(format!(
            "  Analytics: {}",
            if self.privacy.analytics_enabled {
                "enabled"
            } else {
                "disabled"
            }
        ));
        lines.push(String::new());

        // Badges
        lines.push("Badges:".to_string());
        for badge in &self.badges {
            let status = if badge.earned { "[x]" } else { "[ ]" };
            lines.push(format!("  {} {}: {}", status, badge.name, badge.description));
        }

        lines.join("\n")
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }

    /// Check if the report indicates a verified build.
    pub fn is_verified(&self) -> bool {
        matches!(
            self.status,
            AttestationStatus::Passed | AttestationStatus::PassedWithWarnings
        ) && self.trust_classification.level >= TrustLevel::Verified
    }

    /// Get failed checks.
    pub fn failed_checks(&self) -> Vec<&AttestationCheck> {
        self.checks.iter().filter(|c| !c.passed).collect()
    }

    /// Get earned badges.
    pub fn earned_badges(&self) -> Vec<&AttestationBadge> {
        self.badges.iter().filter(|b| b.earned).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attestation_report() {
        let mut report = AttestationReport::new("TestApp", "1.0.0");

        report.add_check(AttestationCheck {
            name: "manifest".to_string(),
            description: "Check manifest".to_string(),
            passed: true,
            details: None,
            severity: CheckSeverity::Error,
        });

        report.network.allowlist_enforced = true;
        report.network.allowed_domains = vec!["api.example.com".to_string()];

        report.finalize();

        assert_eq!(report.status, AttestationStatus::Passed);
    }

    #[test]
    fn test_trust_classification() {
        let mut report = AttestationReport::new("TestApp", "1.0.0");

        report.signature.is_valid = true;
        report.network.allowlist_enforced = true;

        report.finalize();

        assert_eq!(report.trust_classification.level, TrustLevel::Verified);
    }

    #[test]
    fn test_failed_attestation() {
        let mut report = AttestationReport::new("TestApp", "1.0.0");

        report.add_check(AttestationCheck {
            name: "forbidden_api".to_string(),
            description: "Check for forbidden APIs".to_string(),
            passed: false,
            details: Some("socket2 crate detected".to_string()),
            severity: CheckSeverity::Error,
        });

        report.finalize();

        assert_eq!(report.status, AttestationStatus::Failed);
        assert!(!report.is_verified());
    }
}
