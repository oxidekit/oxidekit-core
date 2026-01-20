//! Binary scanner for attestation.

use std::io::Read;
use std::path::Path;

use crate::capabilities::PermissionManifest;
use crate::error::{PermissionError, PermissionResult};
use crate::verified_build::{VerifiedBuildChecker, VerifiedBuildProfile};

use super::report::*;

/// Binary scanner configuration.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Build profile to use for verification.
    pub build_profile: VerifiedBuildProfile,
    /// Whether to extract embedded manifest.
    pub extract_manifest: bool,
    /// Whether to analyze dependencies.
    pub analyze_dependencies: bool,
    /// Whether to check signatures.
    pub check_signatures: bool,
    /// Maximum file size to scan (bytes).
    pub max_file_size: u64,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            build_profile: VerifiedBuildProfile::standard(),
            extract_manifest: true,
            analyze_dependencies: true,
            check_signatures: true,
            max_file_size: 500 * 1024 * 1024, // 500 MB
        }
    }
}

/// Binary scanner for attestation.
pub struct BinaryScanner {
    config: ScannerConfig,
}

impl BinaryScanner {
    /// Create a new binary scanner with default configuration.
    pub fn new() -> Self {
        Self {
            config: ScannerConfig::default(),
        }
    }

    /// Create a scanner with custom configuration.
    pub fn with_config(config: ScannerConfig) -> Self {
        Self { config }
    }

    /// Scan a binary file and generate an attestation report.
    pub fn scan<P: AsRef<Path>>(&self, binary_path: P) -> PermissionResult<AttestationReport> {
        let binary_path = binary_path.as_ref();

        // Check file exists
        if !binary_path.exists() {
            return Err(PermissionError::FileNotFound(binary_path.to_path_buf()));
        }

        // Get file metadata
        let metadata = std::fs::metadata(binary_path)?;

        if metadata.len() > self.config.max_file_size {
            return Err(PermissionError::BinaryAnalysisFailed(format!(
                "Binary too large: {} bytes (max: {} bytes)",
                metadata.len(),
                self.config.max_file_size
            )));
        }

        // Read file for hashing
        let mut file = std::fs::File::open(binary_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        // Calculate hash
        let hash = self.calculate_sha256(&buffer);

        // Extract filename
        let filename = binary_path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        // Determine target from extension
        let target = self.detect_target(&filename);

        // Create report
        let mut report = AttestationReport::new("Unknown", "0.0.0");

        report.binary = BinaryInfo {
            filename,
            size_bytes: metadata.len(),
            sha256: hash,
            target,
            build_time: None,
            oxidekit_version: None,
            rust_version: None,
        };

        // Try to extract embedded manifest
        if self.config.extract_manifest {
            if let Some(manifest) = self.try_extract_manifest(&buffer) {
                self.populate_from_manifest(&mut report, &manifest);
            }
        }

        // Check for signature
        if self.config.check_signatures {
            report.signature = self.check_signature(&buffer);
        }

        // Run verification checks
        self.run_checks(&mut report);

        // Assign badges
        self.assign_badges(&mut report);

        // Finalize
        report.finalize();

        Ok(report)
    }

    /// Scan a binary with a provided manifest.
    pub fn scan_with_manifest<P: AsRef<Path>>(
        &self,
        binary_path: P,
        manifest: &PermissionManifest,
    ) -> PermissionResult<AttestationReport> {
        let mut report = self.scan(binary_path)?;
        self.populate_from_manifest(&mut report, manifest);
        self.run_checks(&mut report);
        self.assign_badges(&mut report);
        report.finalize();
        Ok(report)
    }

    /// Calculate SHA-256 hash of data.
    fn calculate_sha256(&self, data: &[u8]) -> String {
        #[cfg(feature = "attestation")]
        {
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(data);
            hex::encode(hasher.finalize())
        }

        #[cfg(not(feature = "attestation"))]
        {
            // Fallback without sha2 crate
            let _ = data;
            "sha256-not-available".to_string()
        }
    }

    /// Detect target platform from filename.
    fn detect_target(&self, filename: &str) -> String {
        if filename.ends_with(".exe") {
            "x86_64-pc-windows-msvc".to_string()
        } else if filename.ends_with(".app") || filename.contains("darwin") {
            "aarch64-apple-darwin".to_string()
        } else if filename.contains("linux") {
            "x86_64-unknown-linux-gnu".to_string()
        } else {
            "unknown".to_string()
        }
    }

    /// Try to extract embedded manifest from binary.
    fn try_extract_manifest(&self, _data: &[u8]) -> Option<PermissionManifest> {
        // In a real implementation, this would search for embedded
        // OxideKit manifest data in the binary (e.g., in a special section
        // or appended metadata).
        //
        // For now, return None to indicate no embedded manifest found.
        None
    }

    /// Check for code signature.
    fn check_signature(&self, _data: &[u8]) -> SignatureStatus {
        // In a real implementation, this would verify:
        // - Code signature on macOS (codesign)
        // - Authenticode on Windows
        // - GPG signature on Linux
        //
        // For now, return a default unsigned status.
        SignatureStatus {
            is_signed: false,
            algorithm: None,
            signer: None,
            signed_at: None,
            is_valid: false,
            error: Some("Signature verification not implemented".to_string()),
        }
    }

    /// Populate report from manifest.
    fn populate_from_manifest(
        &self,
        report: &mut AttestationReport,
        manifest: &PermissionManifest,
    ) {
        // Extract capabilities
        let all_caps = manifest.all_capabilities();
        let by_category = manifest.capabilities_by_category();

        let mut by_category_strings: std::collections::HashMap<String, Vec<String>> =
            std::collections::HashMap::new();
        for (cat, caps) in by_category {
            by_category_strings.insert(
                format!("{:?}", cat),
                caps.iter().map(|c| c.to_string()).collect(),
            );
        }

        let high_risk: Vec<String> = all_caps
            .iter()
            .filter(|c| {
                let cat = crate::capabilities::CapabilityCategory::from_capability(c);
                cat.risk_level() >= crate::capabilities::RiskLevel::High
            })
            .map(|c| c.to_string())
            .collect();

        report.permissions = PermissionsSummary {
            total_capabilities: all_caps.len(),
            by_category: by_category_strings,
            max_risk_level: manifest.max_risk_level(),
            high_risk_capabilities: high_risk,
            all_have_reasons: manifest
                .capabilities
                .values()
                .all(|d| d.reason.is_some()),
        };

        // Extract network policy
        if let Some(network) = &manifest.network {
            report.network = NetworkSummary {
                uses_network: manifest.allows_network(),
                allowlist_enforced: manifest.has_network_allowlist(),
                enforcement_status: if manifest.has_network_allowlist() {
                    "enforced".to_string()
                } else {
                    "not_enforced".to_string()
                },
                allowed_domains: network.allow.clone(),
                blocks_private_ranges: network.deny_private_ranges,
                requires_https: network.require_https,
            };
        } else {
            report.network.uses_network = manifest.allows_network();
            if manifest.allows_network() {
                report.network.enforcement_status = "unknown".to_string();
            }
        }

        // Extract privacy settings
        if let Some(privacy) = &manifest.privacy {
            let privacy_score = self.calculate_privacy_score(privacy);
            report.privacy = PrivacySummary {
                auto_crash_reporting: privacy.auto_crash_reporting,
                analytics_enabled: privacy.analytics_enabled,
                manual_export_allowed: privacy.manual_export_allowed,
                telemetry_requires_consent: privacy.telemetry_requires_consent,
                data_retention: if privacy.data_retention_days > 0 {
                    Some(format!("{} days", privacy.data_retention_days))
                } else {
                    Some("None".to_string())
                },
                privacy_score,
            };
        }
    }

    /// Calculate privacy score (0-100).
    fn calculate_privacy_score(&self, privacy: &crate::capabilities::PrivacySettings) -> u8 {
        let mut score: u8 = 100;

        if privacy.auto_crash_reporting {
            score = score.saturating_sub(20);
        }
        if privacy.analytics_enabled {
            score = score.saturating_sub(30);
        }
        if !privacy.telemetry_requires_consent {
            score = score.saturating_sub(20);
        }
        if privacy.data_retention_days > 30 {
            score = score.saturating_sub(10);
        }

        score
    }

    /// Run verification checks.
    fn run_checks(&self, report: &mut AttestationReport) {
        // Check: Manifest present
        report.add_check(AttestationCheck {
            name: "manifest_present".to_string(),
            description: "Application has a permission manifest".to_string(),
            passed: report.permissions.total_capabilities > 0,
            details: None,
            severity: CheckSeverity::Warning,
        });

        // Check: Network policy
        if report.network.uses_network {
            report.add_check(AttestationCheck {
                name: "network_policy".to_string(),
                description: "Network access has allowlist policy".to_string(),
                passed: report.network.allowlist_enforced,
                details: if report.network.allowlist_enforced {
                    Some(format!(
                        "Domains: {}",
                        report.network.allowed_domains.join(", ")
                    ))
                } else {
                    Some("Network domains unknown (policy not enforced)".to_string())
                },
                severity: CheckSeverity::Warning,
            });
        }

        // Check: High-risk permissions documented
        if !report.permissions.high_risk_capabilities.is_empty() {
            report.add_check(AttestationCheck {
                name: "high_risk_documented".to_string(),
                description: "High-risk permissions have documented reasons".to_string(),
                passed: report.permissions.all_have_reasons,
                details: Some(format!(
                    "{} high-risk capabilities",
                    report.permissions.high_risk_capabilities.len()
                )),
                severity: CheckSeverity::Warning,
            });
        }

        // Check: Signature
        report.add_check(AttestationCheck {
            name: "code_signature".to_string(),
            description: "Binary is signed with valid signature".to_string(),
            passed: report.signature.is_valid,
            details: report.signature.error.clone(),
            severity: CheckSeverity::Info,
        });

        // Check: Privacy score
        report.add_check(AttestationCheck {
            name: "privacy_score".to_string(),
            description: "Privacy score meets threshold".to_string(),
            passed: report.privacy.privacy_score >= 70,
            details: Some(format!("Score: {}/100", report.privacy.privacy_score)),
            severity: CheckSeverity::Info,
        });
    }

    /// Assign badges based on report.
    fn assign_badges(&self, report: &mut AttestationReport) {
        // Network Allowlist badge
        report.add_badge(AttestationBadge {
            id: "network_allowlist".to_string(),
            name: "Network Allowlist".to_string(),
            description: if report.network.allowlist_enforced {
                "Network connections restricted to declared domains".to_string()
            } else if report.network.uses_network {
                "Network policy not enforced".to_string()
            } else {
                "No network access".to_string()
            },
            earned: report.network.allowlist_enforced || !report.network.uses_network,
            icon: "shield".to_string(),
            category: "security".to_string(),
        });

        // Verified Build badge
        let all_checks_pass = report.checks.iter().filter(|c| c.severity == CheckSeverity::Error).all(|c| c.passed);
        report.add_badge(AttestationBadge {
            id: "verified_build".to_string(),
            name: "Verified Build".to_string(),
            description: if all_checks_pass {
                "Build passes all verification checks".to_string()
            } else {
                "Build has verification issues".to_string()
            },
            earned: all_checks_pass,
            icon: "check-circle".to_string(),
            category: "security".to_string(),
        });

        // Privacy Conscious badge
        report.add_badge(AttestationBadge {
            id: "privacy_conscious".to_string(),
            name: "Privacy Conscious".to_string(),
            description: format!("Privacy score: {}/100", report.privacy.privacy_score),
            earned: report.privacy.privacy_score >= 80,
            icon: "eye-off".to_string(),
            category: "privacy".to_string(),
        });

        // Signed badge
        report.add_badge(AttestationBadge {
            id: "signed".to_string(),
            name: "Signed".to_string(),
            description: if report.signature.is_valid {
                format!(
                    "Signed by: {}",
                    report.signature.signer.as_deref().unwrap_or("Unknown")
                )
            } else {
                "Not signed".to_string()
            },
            earned: report.signature.is_valid,
            icon: "pen-tool".to_string(),
            category: "security".to_string(),
        });

        // Documented badge
        let documented = report.permissions.all_have_reasons
            || report.permissions.total_capabilities == 0;
        report.add_badge(AttestationBadge {
            id: "documented".to_string(),
            name: "Well Documented".to_string(),
            description: if documented {
                "All permissions have documented reasons".to_string()
            } else {
                "Some permissions lack documentation".to_string()
            },
            earned: documented,
            icon: "book".to_string(),
            category: "quality".to_string(),
        });
    }
}

impl Default for BinaryScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_config() {
        let config = ScannerConfig::default();
        assert!(config.extract_manifest);
        assert!(config.check_signatures);
    }

    #[test]
    fn test_privacy_score() {
        let scanner = BinaryScanner::new();

        let good_privacy = crate::capabilities::PrivacySettings {
            auto_crash_reporting: false,
            analytics_enabled: false,
            manual_export_allowed: true,
            telemetry_requires_consent: true,
            data_retention_days: 0,
        };
        assert_eq!(scanner.calculate_privacy_score(&good_privacy), 100);

        let bad_privacy = crate::capabilities::PrivacySettings {
            auto_crash_reporting: true,
            analytics_enabled: true,
            manual_export_allowed: true,
            telemetry_requires_consent: false,
            data_retention_days: 90,
        };
        assert!(scanner.calculate_privacy_score(&bad_privacy) < 50);
    }
}
