//! Error types for the legal module

use thiserror::Error;

/// Result type for legal operations
pub type LegalResult<T> = Result<T, LegalError>;

/// Errors that can occur in legal operations
#[derive(Error, Debug)]
pub enum LegalError {
    /// Failed to parse license identifier
    #[error("invalid license identifier: {0}")]
    InvalidLicense(String),

    /// Failed to parse cargo metadata
    #[error("failed to parse cargo metadata: {0}")]
    CargoMetadataError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// TOML deserialization error
    #[error("TOML deserialization error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// License policy violation
    #[error("license policy violation: {license} in {package} - {reason}")]
    PolicyViolation {
        package: String,
        license: String,
        reason: String,
    },

    /// Multiple policy violations
    #[error("multiple license policy violations: {count} violations found")]
    MultiplePolicyViolations { count: usize },

    /// Unknown license type
    #[error("unknown license type: {0}")]
    UnknownLicense(String),

    /// License incompatibility
    #[error("license incompatibility: {source_license} is not compatible with {target_license}")]
    LicenseIncompatibility { source_license: String, target_license: String },

    /// Missing license file
    #[error("missing license file in {0}")]
    MissingLicenseFile(String),

    /// CLA not signed
    #[error("CLA not signed by contributor: {0}")]
    ClaNotSigned(String),

    /// Export control restriction
    #[error("export control restriction: {0}")]
    ExportRestriction(String),

    /// SBOM generation failed
    #[error("SBOM generation failed: {0}")]
    SbomGenerationFailed(String),

    /// Network error (for CLA checks)
    #[error("network error: {0}")]
    NetworkError(String),

    /// Configuration error
    #[error("configuration error: {0}")]
    ConfigError(String),
}

impl From<cargo_metadata::Error> for LegalError {
    fn from(e: cargo_metadata::Error) -> Self {
        LegalError::CargoMetadataError(e.to_string())
    }
}
