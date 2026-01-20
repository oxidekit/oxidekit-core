//! Error types for the i18n system

use std::path::PathBuf;
use thiserror::Error;

/// Result type alias for i18n operations
pub type I18nResult<T> = Result<T, I18nError>;

/// Errors that can occur during i18n operations
#[derive(Debug, Error)]
pub enum I18nError {
    /// Translation file not found
    #[error("Translation file not found: {path}")]
    FileNotFound { path: PathBuf },

    /// Failed to parse translation file
    #[error("Failed to parse translation file '{path}': {message}")]
    ParseError { path: PathBuf, message: String },

    /// Invalid locale identifier
    #[error("Invalid locale identifier: {locale}")]
    InvalidLocale { locale: String },

    /// Locale not loaded
    #[error("Locale not loaded: {locale}")]
    LocaleNotLoaded { locale: String },

    /// Translation key not found
    #[error("Translation key not found: {key}")]
    KeyNotFound { key: String },

    /// Missing placeholder in translation
    #[error("Missing placeholder '{placeholder}' for key '{key}'")]
    MissingPlaceholder { key: String, placeholder: String },

    /// Invalid plural form
    #[error("Invalid plural form for key '{key}': expected one of {expected:?}")]
    InvalidPluralForm { key: String, expected: Vec<String> },

    /// Invalid translation value type
    #[error("Invalid translation value type for key '{key}': expected {expected}, got {actual}")]
    InvalidValueType {
        key: String,
        expected: String,
        actual: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Walkdir error
    #[error("Directory traversal error: {0}")]
    Walkdir(#[from] walkdir::Error),

    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),

    /// Generic error with context
    #[error("{context}: {message}")]
    WithContext { context: String, message: String },
}

impl I18nError {
    /// Create an error with additional context
    pub fn with_context(context: impl Into<String>, message: impl Into<String>) -> Self {
        Self::WithContext {
            context: context.into(),
            message: message.into(),
        }
    }

    /// Create a parse error
    pub fn parse_error(path: impl Into<PathBuf>, message: impl Into<String>) -> Self {
        Self::ParseError {
            path: path.into(),
            message: message.into(),
        }
    }

    /// Get a stable error code for this error type
    pub fn error_code(&self) -> &'static str {
        match self {
            Self::FileNotFound { .. } => "I18N_E001",
            Self::ParseError { .. } => "I18N_E002",
            Self::InvalidLocale { .. } => "I18N_E003",
            Self::LocaleNotLoaded { .. } => "I18N_E004",
            Self::KeyNotFound { .. } => "I18N_E005",
            Self::MissingPlaceholder { .. } => "I18N_E006",
            Self::InvalidPluralForm { .. } => "I18N_E007",
            Self::InvalidValueType { .. } => "I18N_E008",
            Self::Io(_) => "I18N_E009",
            Self::Toml(_) => "I18N_E010",
            Self::Json(_) => "I18N_E011",
            Self::Walkdir(_) => "I18N_E012",
            Self::Regex(_) => "I18N_E013",
            Self::WithContext { .. } => "I18N_E099",
        }
    }

    /// Get severity level for CI integration
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::KeyNotFound { .. } => ErrorSeverity::Error,
            Self::MissingPlaceholder { .. } => ErrorSeverity::Error,
            Self::InvalidPluralForm { .. } => ErrorSeverity::Error,
            Self::FileNotFound { .. } => ErrorSeverity::Error,
            Self::ParseError { .. } => ErrorSeverity::Error,
            Self::InvalidLocale { .. } => ErrorSeverity::Warning,
            Self::LocaleNotLoaded { .. } => ErrorSeverity::Warning,
            Self::InvalidValueType { .. } => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

/// Severity level for errors
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorSeverity {
    /// Warnings that don't fail the build
    Warning,
    /// Errors that fail the build in strict mode
    Error,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        let err = I18nError::KeyNotFound {
            key: "test.key".to_string(),
        };
        assert_eq!(err.error_code(), "I18N_E005");

        let err = I18nError::FileNotFound {
            path: PathBuf::from("test.toml"),
        };
        assert_eq!(err.error_code(), "I18N_E001");
    }

    #[test]
    fn test_error_severity() {
        let err = I18nError::KeyNotFound {
            key: "test".to_string(),
        };
        assert_eq!(err.severity(), ErrorSeverity::Error);

        let err = I18nError::InvalidLocale {
            locale: "bad".to_string(),
        };
        assert_eq!(err.severity(), ErrorSeverity::Warning);
    }
}
