//! Error types for the branding system

use thiserror::Error;

/// Result type alias for branding operations
pub type BrandingResult<T> = Result<T, BrandingError>;

/// Branding system errors
#[derive(Debug, Error)]
pub enum BrandingError {
    /// Brand pack validation failed
    #[error("Brand validation failed: {0}")]
    ValidationError(String),

    /// Required asset is missing
    #[error("Missing required asset: {0}")]
    MissingAsset(String),

    /// Asset integrity check failed
    #[error("Asset integrity check failed for '{path}': expected {expected}, got {actual}")]
    IntegrityError {
        path: String,
        expected: String,
        actual: String,
    },

    /// Token is locked and cannot be overridden
    #[error("Token '{token}' is locked at {level:?} level and cannot be overridden")]
    TokenLocked { token: String, level: LockLevel },

    /// Compliance violation
    #[error("Brand compliance violation: {0}")]
    ComplianceViolation(String),

    /// Invalid brand configuration
    #[error("Invalid brand configuration: {0}")]
    InvalidConfig(String),

    /// White-label configuration error
    #[error("White-label configuration error: {0}")]
    WhiteLabelError(String),

    /// Export error
    #[error("Export error: {0}")]
    ExportError(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Image processing error
    #[cfg(feature = "image-processing")]
    #[error("Image processing error: {0}")]
    ImageError(#[from] image::ImageError),

    /// Theme inheritance error
    #[error("Theme inheritance error: {0}")]
    InheritanceError(String),

    /// Asset pipeline error
    #[error("Asset pipeline error: {0}")]
    PipelineError(String),
}

/// Lock level for governance errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LockLevel {
    /// Brand-level lock (highest priority)
    Brand,
    /// Organization-level lock
    Organization,
    /// App-level lock
    App,
    /// No lock
    #[default]
    None,
}

impl From<toml::de::Error> for BrandingError {
    fn from(err: toml::de::Error) -> Self {
        BrandingError::SerializationError(err.to_string())
    }
}

impl From<toml::ser::Error> for BrandingError {
    fn from(err: toml::ser::Error) -> Self {
        BrandingError::SerializationError(err.to_string())
    }
}

impl From<serde_json::Error> for BrandingError {
    fn from(err: serde_json::Error) -> Self {
        BrandingError::SerializationError(err.to_string())
    }
}
