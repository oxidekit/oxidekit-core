//! Error types for the documentation system

use std::path::PathBuf;
use thiserror::Error;

/// Result type for documentation operations
pub type DocsResult<T> = Result<T, DocsError>;

/// Documentation system errors
#[derive(Error, Debug)]
pub enum DocsError {
    /// IO error occurred
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Document parsing error
    #[error("Parse error in {path}: {message}")]
    Parse {
        path: PathBuf,
        message: String,
    },

    /// Bundle not found
    #[error("Documentation bundle not found at {0}")]
    BundleNotFound(PathBuf),

    /// Invalid bundle format
    #[error("Invalid bundle format: {0}")]
    InvalidBundle(String),

    /// Search index error
    #[error("Search index error: {0}")]
    SearchIndex(String),

    /// Tutorial execution error
    #[error("Tutorial error: {0}")]
    Tutorial(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// Version mismatch
    #[error("Version mismatch: bundle is for {bundle_version}, but running {current_version}")]
    VersionMismatch {
        bundle_version: String,
        current_version: String,
    },

    /// Missing required content
    #[error("Missing required content: {0}")]
    MissingContent(String),

    /// Server error
    #[error("Server error: {0}")]
    Server(String),

    /// Syntax highlighting error
    #[error("Syntax highlighting error: {0}")]
    Highlight(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<serde_json::Error> for DocsError {
    fn from(err: serde_json::Error) -> Self {
        DocsError::Serialization(err.to_string())
    }
}

impl From<toml::de::Error> for DocsError {
    fn from(err: toml::de::Error) -> Self {
        DocsError::Serialization(err.to_string())
    }
}

impl From<toml::ser::Error> for DocsError {
    fn from(err: toml::ser::Error) -> Self {
        DocsError::Serialization(err.to_string())
    }
}

impl From<tantivy::TantivyError> for DocsError {
    fn from(err: tantivy::TantivyError) -> Self {
        DocsError::SearchIndex(err.to_string())
    }
}

impl From<tantivy::query::QueryParserError> for DocsError {
    fn from(err: tantivy::query::QueryParserError) -> Self {
        DocsError::SearchIndex(err.to_string())
    }
}

impl From<minijinja::Error> for DocsError {
    fn from(err: minijinja::Error) -> Self {
        DocsError::Template(err.to_string())
    }
}

impl From<walkdir::Error> for DocsError {
    fn from(err: walkdir::Error) -> Self {
        DocsError::Io(std::io::Error::new(std::io::ErrorKind::Other, err.to_string()))
    }
}
