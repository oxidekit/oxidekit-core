//! Quality gate error types

use thiserror::Error;

/// Quality gate errors
#[derive(Debug, Error)]
pub enum QualityError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error in {file}: {message}")]
    Parse {
        file: String,
        message: String,
    },

    #[error("Lint error: {0}")]
    Lint(String),

    #[error("Accessibility error: {0}")]
    Accessibility(String),

    #[error("Performance budget exceeded: {0}")]
    Performance(String),

    #[error("Security violation: {0}")]
    Security(String),

    #[error("Bundle size exceeded: {0}")]
    Bundle(String),

    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid rule: {0}")]
    InvalidRule(String),
}

/// Result type for quality operations
pub type QualityResult<T> = Result<T, QualityError>;

/// Error code for machine-readable output
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ErrorCode {
    /// Domain (lint, a11y, perf, security, bundle)
    pub domain: ErrorDomain,
    /// Numeric code within domain
    pub code: u32,
}

impl ErrorCode {
    pub fn new(domain: ErrorDomain, code: u32) -> Self {
        Self { domain, code }
    }

    pub fn lint(code: u32) -> Self {
        Self::new(ErrorDomain::Lint, code)
    }

    pub fn a11y(code: u32) -> Self {
        Self::new(ErrorDomain::Accessibility, code)
    }

    pub fn perf(code: u32) -> Self {
        Self::new(ErrorDomain::Performance, code)
    }

    pub fn security(code: u32) -> Self {
        Self::new(ErrorDomain::Security, code)
    }

    pub fn bundle(code: u32) -> Self {
        Self::new(ErrorDomain::Bundle, code)
    }
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let prefix = match self.domain {
            ErrorDomain::Lint => "L",
            ErrorDomain::Accessibility => "A",
            ErrorDomain::Performance => "P",
            ErrorDomain::Security => "S",
            ErrorDomain::Bundle => "B",
        };
        write!(f, "{}{:04}", prefix, self.code)
    }
}

/// Error domain categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ErrorDomain {
    Lint,
    Accessibility,
    Performance,
    Security,
    Bundle,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_code_display() {
        assert_eq!(ErrorCode::lint(1).to_string(), "L0001");
        assert_eq!(ErrorCode::a11y(42).to_string(), "A0042");
        assert_eq!(ErrorCode::perf(100).to_string(), "P0100");
    }
}
