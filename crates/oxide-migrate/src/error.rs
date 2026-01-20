//! Error types for the oxide-migrate crate.
//!
//! Provides comprehensive error handling for all migration operations including
//! file parsing, framework detection, token extraction, and output generation.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Result type alias for migration operations
pub type MigrateResult<T> = Result<T, MigrateError>;

/// Main error type for migration operations
#[derive(Debug, thiserror::Error)]
pub enum MigrateError {
    /// I/O errors (file reading, writing, etc.)
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Path-related errors
    #[error("Invalid path: {path} - {reason}")]
    InvalidPath { path: PathBuf, reason: String },

    /// Archive/zip file errors
    #[error("Archive error: {0}")]
    Archive(String),

    /// HTML parsing errors
    #[error("HTML parsing error: {0}")]
    HtmlParse(String),

    /// CSS parsing errors
    #[error("CSS parsing error in {file}: {message}")]
    CssParse { file: String, message: String },

    /// Framework detection failed
    #[error("Could not detect framework: {0}")]
    FrameworkDetection(String),

    /// Token extraction errors
    #[error("Token extraction error: {0}")]
    TokenExtraction(String),

    /// Component mapping errors
    #[error("Component mapping error: {0}")]
    ComponentMapping(String),

    /// Output generation errors
    #[error("Output generation error: {0}")]
    OutputGeneration(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Unsupported feature or format
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Missing required file or asset
    #[error("Missing required: {0}")]
    MissingRequired(String),

    /// Invalid color format
    #[error("Invalid color format: {value} - {reason}")]
    InvalidColor { value: String, reason: String },

    /// Invalid CSS value
    #[error("Invalid CSS value: {property}: {value}")]
    InvalidCssValue { property: String, value: String },
}

impl From<toml::ser::Error> for MigrateError {
    fn from(err: toml::ser::Error) -> Self {
        MigrateError::Serialization(format!("TOML serialization: {}", err))
    }
}

impl From<toml::de::Error> for MigrateError {
    fn from(err: toml::de::Error) -> Self {
        MigrateError::Serialization(format!("TOML deserialization: {}", err))
    }
}

impl From<serde_json::Error> for MigrateError {
    fn from(err: serde_json::Error) -> Self {
        MigrateError::Serialization(format!("JSON: {}", err))
    }
}

impl From<zip::result::ZipError> for MigrateError {
    fn from(err: zip::result::ZipError) -> Self {
        MigrateError::Archive(err.to_string())
    }
}

impl From<regex::Error> for MigrateError {
    fn from(err: regex::Error) -> Self {
        MigrateError::CssParse {
            file: "regex".into(),
            message: err.to_string(),
        }
    }
}

/// Validation issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum Severity {
    /// Informational message
    Info,
    /// Warning - migration can proceed but may need manual review
    Warning,
    /// Error - migration blocked or component cannot be mapped
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Severity::Info => write!(f, "INFO"),
            Severity::Warning => write!(f, "WARN"),
            Severity::Error => write!(f, "ERROR"),
        }
    }
}

/// A validation or migration issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationIssue {
    /// Severity of the issue
    pub severity: Severity,
    /// Category of the issue
    pub category: IssueCategory,
    /// Descriptive message
    pub message: String,
    /// Source file (if applicable)
    pub source_file: Option<String>,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

impl MigrationIssue {
    /// Create a new info issue
    pub fn info(category: IssueCategory, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Info,
            category,
            message: message.into(),
            source_file: None,
            line: None,
            suggestion: None,
        }
    }

    /// Create a new warning issue
    pub fn warning(category: IssueCategory, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            category,
            message: message.into(),
            source_file: None,
            line: None,
            suggestion: None,
        }
    }

    /// Create a new error issue
    pub fn error(category: IssueCategory, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            category,
            message: message.into(),
            source_file: None,
            line: None,
            suggestion: None,
        }
    }

    /// Set the source file
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.source_file = Some(file.into());
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Set a suggested fix
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for MigrationIssue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}: {}", self.severity, self.category, self.message)?;
        if let Some(ref file) = self.source_file {
            write!(f, " (in {})", file)?;
            if let Some(line) = self.line {
                write!(f, " at line {}", line)?;
            }
        }
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// Categories of migration issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueCategory {
    /// Framework detection issues
    FrameworkDetection,
    /// Color token issues
    ColorToken,
    /// Typography issues
    Typography,
    /// Spacing issues
    Spacing,
    /// Component mapping issues
    ComponentMapping,
    /// Layout issues
    Layout,
    /// Asset issues
    Asset,
    /// Compatibility issues
    Compatibility,
    /// General/other issues
    General,
}

impl std::fmt::Display for IssueCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IssueCategory::FrameworkDetection => write!(f, "Framework"),
            IssueCategory::ColorToken => write!(f, "Color"),
            IssueCategory::Typography => write!(f, "Typography"),
            IssueCategory::Spacing => write!(f, "Spacing"),
            IssueCategory::ComponentMapping => write!(f, "Component"),
            IssueCategory::Layout => write!(f, "Layout"),
            IssueCategory::Asset => write!(f, "Asset"),
            IssueCategory::Compatibility => write!(f, "Compat"),
            IssueCategory::General => write!(f, "General"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrate_error_display() {
        let err = MigrateError::InvalidPath {
            path: PathBuf::from("/test/path"),
            reason: "not a directory".into(),
        };
        assert!(err.to_string().contains("/test/path"));
        assert!(err.to_string().contains("not a directory"));
    }

    #[test]
    fn test_migration_issue_display() {
        let issue = MigrationIssue::warning(
            IssueCategory::ColorToken,
            "Could not parse color value",
        )
        .with_file("theme.css")
        .with_line(42)
        .with_suggestion("Use hex format like #FF0000");

        let display = issue.to_string();
        assert!(display.contains("WARN"));
        assert!(display.contains("Color"));
        assert!(display.contains("theme.css"));
        assert!(display.contains("42"));
        assert!(display.contains("hex format"));
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }

    #[test]
    fn test_issue_category_display() {
        assert_eq!(IssueCategory::ColorToken.to_string(), "Color");
        assert_eq!(IssueCategory::ComponentMapping.to_string(), "Component");
    }
}
