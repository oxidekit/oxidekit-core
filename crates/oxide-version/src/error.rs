//! Error types for versioning operations

use std::fmt;
use thiserror::Error;

/// Errors that can occur during version operations
#[derive(Debug, Error)]
pub enum VersionError {
    /// Invalid version format
    #[error("Invalid version format: {0}")]
    InvalidFormat(String),

    /// Version constraint parse error
    #[error("Invalid version constraint: {0}")]
    InvalidConstraint(String),

    /// No compatible version found
    #[error("No compatible version found: {0}")]
    NoCompatibleVersion(String),

    /// Incompatible components
    #[error("Incompatible components: {0}")]
    IncompatibleComponents(String),

    /// Manifest parse error
    #[error("Failed to parse manifest: {0}")]
    ManifestParse(String),

    /// Lockfile error
    #[error("Lockfile error: {0}")]
    Lockfile(String),

    /// Migration error
    #[error("Migration error: {0}")]
    Migration(String),

    /// Breaking change detected
    #[error("Breaking change detected: {0}")]
    BreakingChange(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

/// Result type for version operations
pub type VersionResult<T> = Result<T, VersionError>;

/// An incompatibility that prevents an operation
#[derive(Debug, Clone)]
pub struct Incompatibility {
    /// What is incompatible
    pub subject: String,
    /// The expected/required version
    pub expected: String,
    /// The actual version found
    pub actual: String,
    /// Explanation
    pub reason: String,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl fmt::Display for Incompatibility {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: expected {}, found {} - {}",
            self.subject, self.expected, self.actual, self.reason
        )?;

        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n  Suggestion: {}", suggestion)?;
        }

        Ok(())
    }
}

/// A collection of incompatibilities
#[derive(Debug, Default)]
pub struct IncompatibilityReport {
    /// List of incompatibilities
    pub items: Vec<Incompatibility>,
}

impl IncompatibilityReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self { items: Vec::new() }
    }

    /// Add an incompatibility
    pub fn add(&mut self, item: Incompatibility) {
        self.items.push(item);
    }

    /// Check if the report has any incompatibilities
    pub fn has_issues(&self) -> bool {
        !self.items.is_empty()
    }

    /// Get the number of issues
    pub fn count(&self) -> usize {
        self.items.len()
    }

    /// Convert to an error if there are issues
    pub fn into_error(self) -> Option<VersionError> {
        if self.has_issues() {
            let messages: Vec<String> = self.items.iter().map(|i| i.to_string()).collect();
            Some(VersionError::IncompatibleComponents(messages.join("\n")))
        } else {
            None
        }
    }

    /// Format as a human-readable report
    pub fn to_report(&self) -> String {
        if self.items.is_empty() {
            return "No incompatibilities found.".to_string();
        }

        let mut output = format!("Found {} incompatibility issue(s):\n\n", self.items.len());

        for (i, item) in self.items.iter().enumerate() {
            output.push_str(&format!("{}. {}\n", i + 1, item));
            output.push('\n');
        }

        output
    }
}

impl fmt::Display for IncompatibilityReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_report())
    }
}

/// Install validation result
#[derive(Debug)]
pub enum InstallValidation {
    /// Installation can proceed
    Allowed,
    /// Installation blocked due to incompatibilities
    Blocked(IncompatibilityReport),
    /// Installation can proceed with warnings
    AllowedWithWarnings(Vec<String>),
}

impl InstallValidation {
    /// Check if installation is allowed
    pub fn is_allowed(&self) -> bool {
        matches!(self, InstallValidation::Allowed | InstallValidation::AllowedWithWarnings(_))
    }

    /// Check if installation is blocked
    pub fn is_blocked(&self) -> bool {
        matches!(self, InstallValidation::Blocked(_))
    }

    /// Get warnings if any
    pub fn warnings(&self) -> Vec<String> {
        match self {
            InstallValidation::AllowedWithWarnings(w) => w.clone(),
            _ => Vec::new(),
        }
    }

    /// Get the incompatibility report if blocked
    pub fn incompatibilities(&self) -> Option<&IncompatibilityReport> {
        match self {
            InstallValidation::Blocked(report) => Some(report),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_incompatibility_display() {
        let incompat = Incompatibility {
            subject: "plugin-x".to_string(),
            expected: ">=1.0.0".to_string(),
            actual: "0.9.0".to_string(),
            reason: "Version too old".to_string(),
            suggestion: Some("Upgrade to 1.0.0".to_string()),
        };

        let display = incompat.to_string();
        assert!(display.contains("plugin-x"));
        assert!(display.contains(">=1.0.0"));
        assert!(display.contains("0.9.0"));
    }

    #[test]
    fn test_report() {
        let mut report = IncompatibilityReport::new();
        assert!(!report.has_issues());

        report.add(Incompatibility {
            subject: "test".to_string(),
            expected: "1.0".to_string(),
            actual: "0.9".to_string(),
            reason: "Too old".to_string(),
            suggestion: None,
        });

        assert!(report.has_issues());
        assert_eq!(report.count(), 1);
    }

    #[test]
    fn test_install_validation() {
        let allowed = InstallValidation::Allowed;
        assert!(allowed.is_allowed());
        assert!(!allowed.is_blocked());

        let warnings = InstallValidation::AllowedWithWarnings(vec!["Warning".to_string()]);
        assert!(warnings.is_allowed());
        assert_eq!(warnings.warnings().len(), 1);

        let blocked = InstallValidation::Blocked(IncompatibilityReport::new());
        assert!(blocked.is_blocked());
        assert!(!blocked.is_allowed());
    }
}
