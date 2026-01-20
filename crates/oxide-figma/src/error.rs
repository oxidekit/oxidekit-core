//! Error types for the Figma translator

use thiserror::Error;

/// Result type alias for Figma operations
pub type Result<T> = std::result::Result<T, FigmaError>;

/// Errors that can occur during Figma translation
#[derive(Error, Debug)]
pub enum FigmaError {
    /// Authentication error
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    /// Missing Figma token
    #[error("Figma token not found. Set FIGMA_TOKEN environment variable or provide token in config")]
    MissingToken,

    /// API request failed
    #[error("Figma API request failed: {0}")]
    ApiError(String),

    /// Rate limited by Figma API
    #[error("Rate limited by Figma API. Retry after {retry_after} seconds")]
    RateLimited { retry_after: u64 },

    /// Invalid URL format
    #[error("Invalid Figma URL: {0}")]
    InvalidUrl(String),

    /// File not found
    #[error("Figma file not found: {0}")]
    FileNotFound(String),

    /// Node not found
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Parse error
    #[error("Failed to parse Figma response: {0}")]
    ParseError(String),

    /// Component mapping error
    #[error("Failed to map component '{component}': {reason}")]
    ComponentMappingFailed { component: String, reason: String },

    /// Token extraction error
    #[error("Failed to extract token '{token_type}': {reason}")]
    TokenExtractionFailed { token_type: String, reason: String },

    /// Layout translation error
    #[error("Failed to translate layout: {0}")]
    LayoutTranslationFailed(String),

    /// Asset download error
    #[error("Failed to download asset '{asset}': {reason}")]
    AssetDownloadFailed { asset: String, reason: String },

    /// File I/O error
    #[error("File I/O error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Validation error
    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    /// Unsupported feature
    #[error("Unsupported Figma feature: {0}")]
    UnsupportedFeature(String),

    /// Network error
    #[error("Network error: {0}")]
    NetworkError(String),

    /// HTTP client error
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// JSON parsing error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML serialization error
    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    /// TOML deserialization error
    #[error("TOML deserialization error: {0}")]
    TomlDeError(#[from] toml::de::Error),

    /// Sync conflict
    #[error("Sync conflict detected: {0}")]
    SyncConflict(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Export error
    #[error("Export error: {0}")]
    ExportError(String),
}

impl FigmaError {
    /// Check if error is retriable
    pub fn is_retriable(&self) -> bool {
        matches!(
            self,
            FigmaError::RateLimited { .. }
                | FigmaError::NetworkError(_)
                | FigmaError::HttpError(_)
        )
    }

    /// Get retry delay in seconds if applicable
    pub fn retry_delay(&self) -> Option<u64> {
        match self {
            FigmaError::RateLimited { retry_after } => Some(*retry_after),
            FigmaError::NetworkError(_) | FigmaError::HttpError(_) => Some(5),
            _ => None,
        }
    }
}
