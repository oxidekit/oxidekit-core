//! Error types for oxide-deploy
//!
//! This module defines all error types used throughout the deployment configuration system.

use std::path::PathBuf;

/// Errors that can occur during deployment configuration operations
#[derive(Debug, thiserror::Error)]
pub enum DeployError {
    /// Configuration file not found
    #[error("Configuration file not found: {0}")]
    ConfigNotFound(PathBuf),

    /// Invalid configuration format
    #[error("Invalid configuration format: {0}")]
    InvalidConfig(String),

    /// TOML parsing error
    #[error("Failed to parse TOML: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("Failed to serialize TOML: {0}")]
    TomlSerialize(#[from] toml::ser::Error),

    /// JSON parsing error
    #[error("Failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Environment variable error
    #[error("Environment variable error: {0}")]
    EnvVar(String),

    /// Missing required environment variable
    #[error("Missing required environment variable: {0}")]
    MissingEnvVar(String),

    /// Invalid environment variable type
    #[error("Invalid type for environment variable '{name}': expected {expected}, got '{value}'")]
    InvalidEnvVarType {
        /// Variable name
        name: String,
        /// Expected type
        expected: String,
        /// Actual value
        value: String,
    },

    /// Port collision detected
    #[error("Port collision detected: port {port} is used by services: {services:?}")]
    PortCollision {
        /// Colliding port
        port: u16,
        /// Services using the port
        services: Vec<String>,
    },

    /// Invalid port number
    #[error("Invalid port number: {0}")]
    InvalidPort(u16),

    /// Port already in use
    #[error("Port {0} is already in use by another process")]
    PortInUse(u16),

    /// Secret handling error
    #[error("Secret handling error: {0}")]
    Secret(String),

    /// Secret committed to repository
    #[error("Secret '{name}' appears to be committed to repository in file: {file}")]
    SecretCommitted {
        /// Secret name
        name: String,
        /// File containing secret
        file: PathBuf,
    },

    /// Template generation error
    #[error("Template generation error: {0}")]
    Template(String),

    /// Unsupported deploy target
    #[error("Unsupported deploy target: {0}")]
    UnsupportedTarget(String),

    /// CI workflow error
    #[error("CI workflow error: {0}")]
    CiWorkflow(String),

    /// Validation error with multiple issues
    #[error("Validation failed with {0} error(s)")]
    Validation(usize),

    /// Validation error detail
    #[error("Validation error: {0}")]
    ValidationDetail(String),

    /// Walkdir error
    #[error("Directory walk error: {0}")]
    WalkDir(#[from] walkdir::Error),

    /// Regex error
    #[error("Regex error: {0}")]
    Regex(#[from] regex::Error),
}

/// Result type alias for deploy operations
pub type DeployResult<T> = Result<T, DeployError>;

/// A collection of validation errors
#[derive(Debug, Default)]
pub struct ValidationErrors {
    /// List of validation errors
    pub errors: Vec<ValidationError>,
}

impl ValidationErrors {
    /// Create a new empty validation errors collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a validation error
    pub fn push(&mut self, error: ValidationError) {
        self.errors.push(error);
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the number of errors
    pub fn len(&self) -> usize {
        self.errors.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.errors.is_empty()
    }

    /// Convert to a DeployError if there are errors
    pub fn into_result(self) -> DeployResult<()> {
        if self.has_errors() {
            Err(DeployError::Validation(self.len()))
        } else {
            Ok(())
        }
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Validation failed with {} error(s):", self.errors.len())?;
        for (i, error) in self.errors.iter().enumerate() {
            writeln!(f, "  {}. {}", i + 1, error)?;
        }
        Ok(())
    }
}

impl std::error::Error for ValidationErrors {}

/// A single validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error category
    pub category: ValidationCategory,
    /// Error message
    pub message: String,
    /// Optional file path
    pub file: Option<PathBuf>,
    /// Optional line number
    pub line: Option<usize>,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(category: ValidationCategory, message: impl Into<String>) -> Self {
        Self {
            category,
            message: message.into(),
            file: None,
            line: None,
        }
    }

    /// Set the file path
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Set the line number
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.category, self.message)?;
        if let Some(ref file) = self.file {
            write!(f, " (file: {})", file.display())?;
        }
        if let Some(line) = self.line {
            write!(f, " at line {}", line)?;
        }
        Ok(())
    }
}

/// Categories for validation errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationCategory {
    /// Environment variable related
    EnvVar,
    /// Port related
    Port,
    /// Secret related
    Secret,
    /// Configuration related
    Config,
    /// Template related
    Template,
    /// CI workflow related
    Ci,
}

impl std::fmt::Display for ValidationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvVar => write!(f, "ENV"),
            Self::Port => write!(f, "PORT"),
            Self::Secret => write!(f, "SECRET"),
            Self::Config => write!(f, "CONFIG"),
            Self::Template => write!(f, "TEMPLATE"),
            Self::Ci => write!(f, "CI"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_errors_collection() {
        let mut errors = ValidationErrors::new();
        assert!(errors.is_empty());
        assert!(!errors.has_errors());

        errors.push(ValidationError::new(
            ValidationCategory::EnvVar,
            "Missing DATABASE_URL",
        ));
        assert!(!errors.is_empty());
        assert!(errors.has_errors());
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn test_validation_error_display() {
        let error = ValidationError::new(ValidationCategory::Secret, "Secret committed")
            .with_file(".env")
            .with_line(5);

        let display = format!("{}", error);
        assert!(display.contains("[SECRET]"));
        assert!(display.contains("Secret committed"));
        assert!(display.contains(".env"));
        assert!(display.contains("line 5"));
    }

    #[test]
    fn test_deploy_error_display() {
        let error = DeployError::PortCollision {
            port: 8080,
            services: vec!["api".to_string(), "web".to_string()],
        };
        let display = format!("{}", error);
        assert!(display.contains("8080"));
        assert!(display.contains("api"));
        assert!(display.contains("web"));
    }
}
