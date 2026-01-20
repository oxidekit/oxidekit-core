//! Attestation service for processing uploads and generating reports.

use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::capabilities::PermissionManifest;
use crate::error::{PermissionError, PermissionResult};

use super::report::*;
use super::scanner::{BinaryScanner, ScannerConfig};

/// Attestation service configuration.
#[derive(Debug, Clone)]
pub struct AttestationServiceConfig {
    /// Scanner configuration.
    pub scanner_config: ScannerConfig,
    /// Directory for storing reports.
    pub reports_dir: PathBuf,
    /// Directory for storing attestation bundles.
    pub bundles_dir: PathBuf,
    /// Maximum upload size in bytes.
    pub max_upload_size: u64,
    /// Whether to store uploaded binaries.
    pub store_binaries: bool,
}

impl Default for AttestationServiceConfig {
    fn default() -> Self {
        Self {
            scanner_config: ScannerConfig::default(),
            reports_dir: PathBuf::from("./attestation/reports"),
            bundles_dir: PathBuf::from("./attestation/bundles"),
            max_upload_size: 500 * 1024 * 1024, // 500 MB
            store_binaries: false,
        }
    }
}

/// Attestation service for processing binary uploads.
pub struct AttestationService {
    config: AttestationServiceConfig,
    scanner: BinaryScanner,
}

impl AttestationService {
    /// Create a new attestation service.
    pub fn new(config: AttestationServiceConfig) -> Self {
        let scanner = BinaryScanner::with_config(config.scanner_config.clone());
        Self { config, scanner }
    }

    /// Create with default configuration.
    pub fn default_service() -> Self {
        Self::new(AttestationServiceConfig::default())
    }

    /// Process a binary upload and generate an attestation report.
    pub fn upload<P: AsRef<Path>>(&self, binary_path: P) -> PermissionResult<AttestationResult> {
        let binary_path = binary_path.as_ref();

        // Check file size
        let metadata = std::fs::metadata(binary_path)?;
        if metadata.len() > self.config.max_upload_size {
            return Err(PermissionError::BinaryAnalysisFailed(format!(
                "File too large: {} bytes (max: {})",
                metadata.len(),
                self.config.max_upload_size
            )));
        }

        // Scan the binary
        let report = self.scanner.scan(binary_path)?;

        // Generate attestation ID
        let attestation_id = format!(
            "{}-{}-{}",
            report.app.name.replace(' ', "-").to_lowercase(),
            report.app.version,
            Utc::now().timestamp()
        );

        // Save report
        let report_path = self.save_report(&attestation_id, &report)?;

        // Generate bundle if verified
        let bundle_path = if report.is_verified() {
            Some(self.generate_bundle(&attestation_id, &report)?)
        } else {
            None
        };

        Ok(AttestationResult {
            attestation_id,
            report,
            report_path,
            bundle_path,
            generated_at: Utc::now(),
        })
    }

    /// Process a binary with a provided manifest.
    pub fn upload_with_manifest<P: AsRef<Path>>(
        &self,
        binary_path: P,
        manifest: &PermissionManifest,
    ) -> PermissionResult<AttestationResult> {
        let binary_path = binary_path.as_ref();

        // Check file size
        let metadata = std::fs::metadata(binary_path)?;
        if metadata.len() > self.config.max_upload_size {
            return Err(PermissionError::BinaryAnalysisFailed(format!(
                "File too large: {} bytes (max: {})",
                metadata.len(),
                self.config.max_upload_size
            )));
        }

        // Scan with manifest
        let report = self.scanner.scan_with_manifest(binary_path, manifest)?;

        // Generate attestation ID
        let attestation_id = format!(
            "{}-{}-{}",
            report.app.name.replace(' ', "-").to_lowercase(),
            report.app.version,
            Utc::now().timestamp()
        );

        // Save report
        let report_path = self.save_report(&attestation_id, &report)?;

        // Generate bundle if verified
        let bundle_path = if report.is_verified() {
            Some(self.generate_bundle(&attestation_id, &report)?)
        } else {
            None
        };

        Ok(AttestationResult {
            attestation_id,
            report,
            report_path,
            bundle_path,
            generated_at: Utc::now(),
        })
    }

    /// Get a report by attestation ID.
    pub fn get_report(&self, attestation_id: &str) -> PermissionResult<AttestationReport> {
        let report_path = self.config.reports_dir.join(format!("{}.json", attestation_id));

        if !report_path.exists() {
            return Err(PermissionError::FileNotFound(report_path));
        }

        let content = std::fs::read_to_string(&report_path)?;
        AttestationReport::from_json(&content)
            .map_err(|e| PermissionError::SerializationError(e.to_string()))
    }

    /// Save a report to disk.
    fn save_report(
        &self,
        attestation_id: &str,
        report: &AttestationReport,
    ) -> PermissionResult<PathBuf> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.config.reports_dir)?;

        let report_path = self.config.reports_dir.join(format!("{}.json", attestation_id));

        let json = report
            .to_json()
            .map_err(|e| PermissionError::SerializationError(e.to_string()))?;

        std::fs::write(&report_path, json)?;

        Ok(report_path)
    }

    /// Generate an attestation bundle.
    fn generate_bundle(
        &self,
        attestation_id: &str,
        report: &AttestationReport,
    ) -> PermissionResult<PathBuf> {
        // Ensure directory exists
        std::fs::create_dir_all(&self.config.bundles_dir)?;

        let bundle_path = self
            .config
            .bundles_dir
            .join(format!("{}.attestation", attestation_id));

        // Create bundle content
        let bundle = AttestationBundle {
            version: "1.0".to_string(),
            attestation_id: attestation_id.to_string(),
            generated_at: Utc::now(),
            app_name: report.app.name.clone(),
            app_version: report.app.version.clone(),
            binary_hash: report.binary.sha256.clone(),
            trust_level: report.trust_classification.level,
            badges: report
                .badges
                .iter()
                .filter(|b| b.earned)
                .map(|b| b.id.clone())
                .collect(),
            report_hash: self.hash_report(report),
        };

        let bundle_json = serde_json::to_string_pretty(&bundle)
            .map_err(|e| PermissionError::SerializationError(e.to_string()))?;

        std::fs::write(&bundle_path, bundle_json)?;

        Ok(bundle_path)
    }

    /// Hash a report for integrity checking.
    fn hash_report(&self, report: &AttestationReport) -> String {
        #[cfg(feature = "attestation")]
        {
            use sha2::{Digest, Sha256};
            let json = report.to_json().unwrap_or_default();
            let mut hasher = Sha256::new();
            hasher.update(json.as_bytes());
            hex::encode(hasher.finalize())
        }

        #[cfg(not(feature = "attestation"))]
        {
            let _ = report;
            "hash-not-available".to_string()
        }
    }

    /// Verify an attestation bundle against a binary.
    pub fn verify_bundle<P: AsRef<Path>>(
        &self,
        bundle_path: P,
        binary_path: P,
    ) -> PermissionResult<BundleVerification> {
        let bundle_path = bundle_path.as_ref();
        let binary_path = binary_path.as_ref();

        // Load bundle
        let bundle_content = std::fs::read_to_string(bundle_path)?;
        let bundle: AttestationBundle = serde_json::from_str(&bundle_content)
            .map_err(|e| PermissionError::SerializationError(e.to_string()))?;

        // Scan binary to get hash
        let report = self.scanner.scan(binary_path)?;

        // Verify hash match
        let hash_matches = bundle.binary_hash == report.binary.sha256;

        Ok(BundleVerification {
            bundle,
            hash_matches,
            verified_at: Utc::now(),
        })
    }
}

impl Default for AttestationService {
    fn default() -> Self {
        Self::default_service()
    }
}

/// Result of an attestation upload.
#[derive(Debug)]
pub struct AttestationResult {
    /// Unique attestation ID.
    pub attestation_id: String,
    /// The generated report.
    pub report: AttestationReport,
    /// Path to saved report.
    pub report_path: PathBuf,
    /// Path to attestation bundle (if verified).
    pub bundle_path: Option<PathBuf>,
    /// When the attestation was generated.
    pub generated_at: DateTime<Utc>,
}

impl AttestationResult {
    /// Check if the attestation is verified.
    pub fn is_verified(&self) -> bool {
        self.report.is_verified()
    }

    /// Get the trust level.
    pub fn trust_level(&self) -> TrustLevel {
        self.report.trust_classification.level
    }

    /// Get earned badges.
    pub fn earned_badges(&self) -> Vec<&AttestationBadge> {
        self.report.earned_badges()
    }
}

/// Attestation bundle for shipping with binaries.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AttestationBundle {
    /// Bundle format version.
    pub version: String,
    /// Attestation ID.
    pub attestation_id: String,
    /// Generation timestamp.
    pub generated_at: DateTime<Utc>,
    /// Application name.
    pub app_name: String,
    /// Application version.
    pub app_version: String,
    /// SHA-256 hash of the attested binary.
    pub binary_hash: String,
    /// Trust level assigned.
    pub trust_level: TrustLevel,
    /// Badges earned.
    pub badges: Vec<String>,
    /// Hash of the full report.
    pub report_hash: String,
}

/// Result of bundle verification.
#[derive(Debug)]
pub struct BundleVerification {
    /// The bundle that was verified.
    pub bundle: AttestationBundle,
    /// Whether the binary hash matches.
    pub hash_matches: bool,
    /// When verification was performed.
    pub verified_at: DateTime<Utc>,
}

impl BundleVerification {
    /// Check if verification passed.
    pub fn is_valid(&self) -> bool {
        self.hash_matches
    }
}

/// CLI interface for attestation commands.
pub mod cli {
    use super::*;

    /// Upload a binary for attestation.
    pub fn upload_command(binary_path: &str) -> PermissionResult<String> {
        let service = AttestationService::default_service();
        let result = service.upload(binary_path)?;

        Ok(format!(
            "Attestation complete!\n\
             ID: {}\n\
             Status: {:?}\n\
             Trust Level: {}\n\
             Report: {}\n\
             Bundle: {}",
            result.attestation_id,
            result.report.status,
            result.trust_level(),
            result.report_path.display(),
            result
                .bundle_path
                .map(|p| p.display().to_string())
                .unwrap_or_else(|| "Not generated (verification failed)".to_string())
        ))
    }

    /// Get a report for an attestation.
    pub fn report_command(attestation_id: &str) -> PermissionResult<String> {
        let service = AttestationService::default_service();
        let report = service.get_report(attestation_id)?;

        Ok(report.summary())
    }

    /// Verify a bundle against a binary.
    pub fn verify_command(bundle_path: &str, binary_path: &str) -> PermissionResult<String> {
        let service = AttestationService::default_service();
        let verification = service.verify_bundle(bundle_path, binary_path)?;

        if verification.is_valid() {
            Ok(format!(
                "Verification PASSED\n\
                 App: {} v{}\n\
                 Trust Level: {}\n\
                 Badges: {}",
                verification.bundle.app_name,
                verification.bundle.app_version,
                verification.bundle.trust_level,
                verification.bundle.badges.join(", ")
            ))
        } else {
            Ok("Verification FAILED: Binary hash does not match attestation".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_config() {
        let config = AttestationServiceConfig::default();
        assert!(!config.store_binaries);
    }

    #[test]
    fn test_attestation_bundle_serialization() {
        let bundle = AttestationBundle {
            version: "1.0".to_string(),
            attestation_id: "test-1234".to_string(),
            generated_at: Utc::now(),
            app_name: "TestApp".to_string(),
            app_version: "1.0.0".to_string(),
            binary_hash: "abc123".to_string(),
            trust_level: TrustLevel::Verified,
            badges: vec!["network_allowlist".to_string()],
            report_hash: "def456".to_string(),
        };

        let json = serde_json::to_string(&bundle).unwrap();
        let parsed: AttestationBundle = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.app_name, "TestApp");
        assert_eq!(parsed.trust_level, TrustLevel::Verified);
    }
}
