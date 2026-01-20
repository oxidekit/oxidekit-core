//! AI Validation
//!
//! Validate UI files against component schemas.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// AI validator for .oui files
#[derive(Debug, Clone, Default)]
pub struct AiValidator {
    strict: bool,
}

impl AiValidator {
    /// Create a new validator
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict mode
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Validate a .oui file
    pub fn validate_file(&self, path: &Path) -> ValidationReport {
        let mut report = ValidationReport::new(path.to_string_lossy().to_string());

        match std::fs::read_to_string(path) {
            Ok(content) => {
                self.validate_content(&content, &mut report);
            }
            Err(e) => {
                report.add_error(ValidationIssue {
                    severity: Severity::Error,
                    code: "E001".to_string(),
                    message: format!("Failed to read file: {}", e),
                    location: None,
                    suggestion: Some("Check file permissions and path".to_string()),
                });
            }
        }

        report
    }

    /// Validate content string
    pub fn validate_content(&self, content: &str, report: &mut ValidationReport) {
        // Basic validation checks
        if content.trim().is_empty() {
            report.add_error(ValidationIssue {
                severity: Severity::Error,
                code: "E002".to_string(),
                message: "File is empty".to_string(),
                location: None,
                suggestion: Some("Add component definitions".to_string()),
            });
            return;
        }

        // Check for common issues
        if !content.contains("Component") && !content.contains("View") && !content.contains("Box") {
            report.add_warning(ValidationIssue {
                severity: Severity::Warning,
                code: "W001".to_string(),
                message: "No recognized root element found".to_string(),
                location: Some(Location { line: 1, column: 1 }),
                suggestion: Some("Start with Component, View, or Box".to_string()),
            });
        }

        // Check for balanced braces
        let open_braces = content.matches('{').count();
        let close_braces = content.matches('}').count();
        if open_braces != close_braces {
            report.add_error(ValidationIssue {
                severity: Severity::Error,
                code: "E003".to_string(),
                message: format!("Unbalanced braces: {} open, {} close", open_braces, close_braces),
                location: None,
                suggestion: Some("Check for missing or extra braces".to_string()),
            });
        }
    }
}

/// Validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    pub file: String,
    pub valid: bool,
    pub issues: Vec<ValidationIssue>,
    pub error_count: usize,
    pub warning_count: usize,
}

impl ValidationReport {
    /// Create a new report
    pub fn new(file: String) -> Self {
        Self {
            file,
            valid: true,
            issues: Vec::new(),
            error_count: 0,
            warning_count: 0,
        }
    }

    /// Add an error
    pub fn add_error(&mut self, issue: ValidationIssue) {
        self.valid = false;
        self.error_count += 1;
        self.issues.push(issue);
    }

    /// Add a warning
    pub fn add_warning(&mut self, issue: ValidationIssue) {
        self.warning_count += 1;
        self.issues.push(issue);
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        self.valid
    }
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub severity: Severity,
    pub code: String,
    pub message: String,
    pub location: Option<Location>,
    pub suggestion: Option<String>,
}

/// Issue severity
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
    Hint,
}

/// Source location
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Location {
    pub line: usize,
    pub column: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_empty() {
        let validator = AiValidator::new();
        let mut report = ValidationReport::new("test.oui".to_string());
        validator.validate_content("", &mut report);
        assert!(!report.is_valid());
    }

    #[test]
    fn test_validate_valid() {
        let validator = AiValidator::new();
        let mut report = ValidationReport::new("test.oui".to_string());
        validator.validate_content("Component { Box { } }", &mut report);
        assert!(report.is_valid());
    }
}
