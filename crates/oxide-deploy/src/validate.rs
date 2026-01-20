//! Configuration validation
//!
//! This module provides comprehensive validation for deployment configurations,
//! including environment variables, ports, secrets, and overall deployment readiness.

use crate::env_schema::EnvSchema;
use crate::error::{DeployResult, ValidationCategory, ValidationErrors};
use crate::ports::PortConfig;
use crate::secrets::{SecretConfig, SecretScanner};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Overall deployment configuration to validate
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Application name
    pub app_name: String,
    /// Environment schema
    #[serde(default)]
    pub env_schema: Option<EnvSchema>,
    /// Port configuration
    #[serde(default)]
    pub port_config: Option<PortConfig>,
    /// Secret configuration
    #[serde(default)]
    pub secret_config: Option<SecretConfig>,
    /// Project root directory
    #[serde(skip)]
    pub project_root: Option<PathBuf>,
}

impl DeployConfig {
    /// Create a new deployment configuration
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            ..Default::default()
        }
    }

    /// Set the environment schema
    pub fn with_env_schema(mut self, schema: EnvSchema) -> Self {
        self.env_schema = Some(schema);
        self
    }

    /// Set the port configuration
    pub fn with_port_config(mut self, config: PortConfig) -> Self {
        self.port_config = Some(config);
        self
    }

    /// Set the secret configuration
    pub fn with_secret_config(mut self, config: SecretConfig) -> Self {
        self.secret_config = Some(config);
        self
    }

    /// Set the project root directory
    pub fn with_project_root(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_root = Some(path.into());
        self
    }

    /// Load configuration from a project directory
    pub fn from_project(project_dir: impl AsRef<Path>) -> DeployResult<Self> {
        let dir = project_dir.as_ref();
        let mut config = Self::new("app");
        config.project_root = Some(dir.to_path_buf());

        // Try to load oxide.env.schema.toml
        let env_schema_path = dir.join("oxide.env.schema.toml");
        if env_schema_path.exists() {
            config.env_schema = Some(EnvSchema::from_file(&env_schema_path)?);
        }

        // Try to load oxide.ports.toml
        let ports_path = dir.join("oxide.ports.toml");
        if ports_path.exists() {
            config.port_config = Some(PortConfig::from_file(&ports_path)?);
        }

        Ok(config)
    }
}

/// Validation severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Severity {
    /// Informational notice
    Info,
    /// Warning - might cause issues
    Warning,
    /// Error - will cause deployment failure
    Error,
}

impl std::fmt::Display for Severity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Info => write!(f, "INFO"),
            Self::Warning => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
        }
    }
}

/// A validation finding with severity
#[derive(Debug, Clone)]
pub struct ValidationFinding {
    /// Severity level
    pub severity: Severity,
    /// Category
    pub category: ValidationCategory,
    /// Message
    pub message: String,
    /// Optional file path
    pub file: Option<PathBuf>,
    /// Optional line number
    pub line: Option<usize>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

impl ValidationFinding {
    /// Create a new finding
    pub fn new(severity: Severity, category: ValidationCategory, message: impl Into<String>) -> Self {
        Self {
            severity,
            category,
            message: message.into(),
            file: None,
            line: None,
            suggestion: None,
        }
    }

    /// Create an error finding
    pub fn error(category: ValidationCategory, message: impl Into<String>) -> Self {
        Self::new(Severity::Error, category, message)
    }

    /// Create a warning finding
    pub fn warning(category: ValidationCategory, message: impl Into<String>) -> Self {
        Self::new(Severity::Warning, category, message)
    }

    /// Create an info finding
    pub fn info(category: ValidationCategory, message: impl Into<String>) -> Self {
        Self::new(Severity::Info, category, message)
    }

    /// Add file context
    pub fn with_file(mut self, file: impl Into<PathBuf>) -> Self {
        self.file = Some(file.into());
        self
    }

    /// Add line context
    pub fn with_line(mut self, line: usize) -> Self {
        self.line = Some(line);
        self
    }

    /// Add a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }
}

impl std::fmt::Display for ValidationFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] [{}] {}", self.severity, self.category, self.message)?;
        if let Some(ref file) = self.file {
            write!(f, "\n       File: {}", file.display())?;
        }
        if let Some(line) = self.line {
            write!(f, " (line {})", line)?;
        }
        if let Some(ref suggestion) = self.suggestion {
            write!(f, "\n       Suggestion: {}", suggestion)?;
        }
        Ok(())
    }
}

/// Validation report containing all findings
#[derive(Debug, Default)]
pub struct ValidationReport {
    /// All findings
    pub findings: Vec<ValidationFinding>,
}

impl ValidationReport {
    /// Create a new empty report
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a finding
    pub fn add(&mut self, finding: ValidationFinding) {
        self.findings.push(finding);
    }

    /// Get all errors
    pub fn errors(&self) -> Vec<&ValidationFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Error)
            .collect()
    }

    /// Get all warnings
    pub fn warnings(&self) -> Vec<&ValidationFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == Severity::Warning)
            .collect()
    }

    /// Check if there are any errors
    pub fn has_errors(&self) -> bool {
        self.findings.iter().any(|f| f.severity == Severity::Error)
    }

    /// Check if there are any warnings
    pub fn has_warnings(&self) -> bool {
        self.findings
            .iter()
            .any(|f| f.severity == Severity::Warning)
    }

    /// Get error count
    pub fn error_count(&self) -> usize {
        self.errors().len()
    }

    /// Get warning count
    pub fn warning_count(&self) -> usize {
        self.warnings().len()
    }

    /// Check if the validation passed (no errors)
    pub fn passed(&self) -> bool {
        !self.has_errors()
    }

    /// Get findings by category
    pub fn by_category(&self, category: ValidationCategory) -> Vec<&ValidationFinding> {
        self.findings
            .iter()
            .filter(|f| f.category == category)
            .collect()
    }

    /// Merge another report into this one
    pub fn merge(&mut self, other: ValidationReport) {
        self.findings.extend(other.findings);
    }
}

impl std::fmt::Display for ValidationReport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.findings.is_empty() {
            writeln!(f, "Validation passed with no issues.")?;
            return Ok(());
        }

        writeln!(
            f,
            "Validation completed: {} error(s), {} warning(s)",
            self.error_count(),
            self.warning_count()
        )?;
        writeln!(f)?;

        for finding in &self.findings {
            writeln!(f, "{}", finding)?;
        }

        Ok(())
    }
}

/// Validator for deployment configurations
pub struct Validator {
    /// Configuration to validate
    config: DeployConfig,
    /// Current environment
    environment: HashMap<String, String>,
}

impl Validator {
    /// Create a new validator
    pub fn new(config: DeployConfig) -> Self {
        Self {
            config,
            environment: std::env::vars().collect(),
        }
    }

    /// Create a validator with custom environment
    pub fn with_environment(config: DeployConfig, env: HashMap<String, String>) -> Self {
        Self {
            config,
            environment: env,
        }
    }

    /// Run all validations
    pub fn validate(&self) -> ValidationReport {
        let mut report = ValidationReport::new();

        // Validate environment schema
        if let Some(ref schema) = self.config.env_schema {
            self.validate_env_schema(schema, &mut report);
        }

        // Validate port configuration
        if let Some(ref ports) = self.config.port_config {
            self.validate_ports(ports, &mut report);
        }

        // Validate secrets
        if let Some(ref secrets) = self.config.secret_config {
            self.validate_secrets(secrets, &mut report);
        }

        // Check for committed secrets in project
        if let Some(ref root) = self.config.project_root {
            self.validate_no_committed_secrets(root, &mut report);
        }

        report
    }

    /// Validate environment schema
    fn validate_env_schema(&self, schema: &EnvSchema, report: &mut ValidationReport) {
        for var in &schema.variables {
            let value = self.environment.get(&var.name);

            // Check required variables
            if var.required && value.is_none() && var.default.is_none() {
                report.add(
                    ValidationFinding::error(
                        ValidationCategory::EnvVar,
                        format!("Required environment variable '{}' is not set", var.name),
                    )
                    .with_suggestion(format!("Set {}=<value> in your environment", var.name)),
                );
                continue;
            }

            // Validate value type if present
            if let Some(val) = value {
                if let Err(e) = var.validate(Some(val)) {
                    report.add(ValidationFinding::error(ValidationCategory::EnvVar, e.to_string()));
                }
            }

            // Warn about secrets in environment
            if var.secret && value.is_some() {
                report.add(ValidationFinding::info(
                    ValidationCategory::Secret,
                    format!(
                        "Secret '{}' is set in environment (ensure it's not committed)",
                        var.name
                    ),
                ));
            }

            // Check for insecure defaults
            if var.secret && var.default.is_some() {
                report.add(
                    ValidationFinding::warning(
                        ValidationCategory::Secret,
                        format!(
                            "Secret '{}' has a default value, which may be insecure",
                            var.name
                        ),
                    )
                    .with_suggestion("Remove default value for secrets"),
                );
            }
        }
    }

    /// Validate port configuration
    fn validate_ports(&self, ports: &PortConfig, report: &mut ValidationReport) {
        // Check for collisions
        let collisions = ports.detect_collisions();
        for collision in collisions {
            report.add(
                ValidationFinding::error(
                    ValidationCategory::Port,
                    format!(
                        "Port {} is used by multiple services: {}",
                        collision.port,
                        collision.services.join(", ")
                    ),
                )
                .with_suggestion("Assign unique ports to each service"),
            );
        }

        // Check for well-known ports
        for mapping in &ports.ports {
            let port = mapping.effective_port();
            if port < 1024 {
                report.add(
                    ValidationFinding::warning(
                        ValidationCategory::Port,
                        format!(
                            "Service '{}' uses privileged port {} (requires root/admin)",
                            mapping.service, port
                        ),
                    )
                    .with_suggestion("Use a port >= 1024 unless root access is guaranteed"),
                );
            }

            // Check if port is in use (only for external ports)
            if mapping.external {
                #[cfg(not(target_arch = "wasm32"))]
                if crate::ports::is_port_in_use(port) {
                    report.add(ValidationFinding::error(
                        ValidationCategory::Port,
                        format!("Port {} is already in use on this system", port),
                    ));
                }
            }
        }
    }

    /// Validate secrets configuration
    fn validate_secrets(&self, secrets: &SecretConfig, report: &mut ValidationReport) {
        for secret in secrets.required_secrets() {
            if self.environment.get(&secret.name).is_none() {
                report.add(
                    ValidationFinding::error(
                        ValidationCategory::Secret,
                        format!("Required secret '{}' is not set", secret.name),
                    )
                    .with_suggestion(format!("Set {} in your environment or secret store", secret.name)),
                );
            }
        }
    }

    /// Check for committed secrets in project
    fn validate_no_committed_secrets(&self, project_root: &Path, report: &mut ValidationReport) {
        let scanner = SecretScanner::default();

        if let Ok(findings) = scanner.scan_directory(project_root) {
            for finding in findings {
                report.add(
                    ValidationFinding::error(
                        ValidationCategory::Secret,
                        finding.description,
                    )
                    .with_file(finding.file)
                    .with_suggestion("Remove this file from version control and add to .gitignore"),
                );
            }
        }
    }
}

/// Quick validation function for a project directory
pub fn validate_project(project_dir: impl AsRef<Path>) -> DeployResult<ValidationReport> {
    let config = DeployConfig::from_project(project_dir)?;
    let validator = Validator::new(config);
    Ok(validator.validate())
}

/// Validation check that can be run as part of a doctor command
pub fn doctor_check(project_dir: impl AsRef<Path>) -> DeployResult<Vec<String>> {
    let report = validate_project(project_dir)?;
    let mut messages = Vec::new();

    if report.passed() {
        messages.push("All deployment configuration checks passed.".to_string());
    } else {
        messages.push(format!(
            "Deployment configuration has {} error(s) and {} warning(s):",
            report.error_count(),
            report.warning_count()
        ));

        for finding in report.errors() {
            messages.push(format!("  ERROR: {}", finding.message));
        }

        for finding in report.warnings() {
            messages.push(format!("  WARN: {}", finding.message));
        }
    }

    Ok(messages)
}

/// Convert ValidationErrors to ValidationReport
impl From<ValidationErrors> for ValidationReport {
    fn from(errors: ValidationErrors) -> Self {
        let mut report = Self::new();
        for error in errors.errors {
            let mut finding = ValidationFinding::error(error.category, error.message);
            if let Some(file) = error.file {
                finding = finding.with_file(file);
            }
            if let Some(line) = error.line {
                finding = finding.with_line(line);
            }
            report.add(finding);
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::env_schema::EnvSchemaBuilder;
    use crate::error::ValidationError;
    use crate::ports::PortConfigBuilder;

    #[test]
    fn test_validation_finding_creation() {
        let finding = ValidationFinding::error(ValidationCategory::EnvVar, "Missing variable")
            .with_file("/app/.env")
            .with_line(10)
            .with_suggestion("Add the variable");

        assert_eq!(finding.severity, Severity::Error);
        assert!(finding.file.is_some());
        assert_eq!(finding.line, Some(10));
        assert!(finding.suggestion.is_some());
    }

    #[test]
    fn test_validation_report() {
        let mut report = ValidationReport::new();
        report.add(ValidationFinding::error(
            ValidationCategory::EnvVar,
            "Error 1",
        ));
        report.add(ValidationFinding::warning(
            ValidationCategory::Port,
            "Warning 1",
        ));
        report.add(ValidationFinding::info(
            ValidationCategory::Config,
            "Info 1",
        ));

        assert!(!report.passed());
        assert_eq!(report.error_count(), 1);
        assert_eq!(report.warning_count(), 1);
        assert!(report.has_errors());
        assert!(report.has_warnings());
    }

    #[test]
    fn test_validate_missing_env_var() {
        let schema = EnvSchemaBuilder::new()
            .string("REQUIRED_VAR")
            .build();

        let config = DeployConfig::new("test-app").with_env_schema(schema);
        let validator = Validator::with_environment(config, HashMap::new());
        let report = validator.validate();

        assert!(report.has_errors());
        assert!(report.errors().iter().any(|f| f.message.contains("REQUIRED_VAR")));
    }

    #[test]
    fn test_validate_env_var_with_default() {
        let schema = EnvSchemaBuilder::new()
            .string_optional("OPTIONAL_VAR", "default")
            .build();

        let config = DeployConfig::new("test-app").with_env_schema(schema);
        let validator = Validator::with_environment(config, HashMap::new());
        let report = validator.validate();

        assert!(report.passed());
    }

    #[test]
    fn test_validate_port_collision() {
        let ports = PortConfigBuilder::new()
            .custom(crate::ports::PortMapping::new("service1", 8080))
            .custom(crate::ports::PortMapping::new("service2", 8080))
            .build();

        let config = DeployConfig::new("test-app").with_port_config(ports);
        let validator = Validator::new(config);
        let report = validator.validate();

        assert!(report.has_errors());
        assert!(report
            .by_category(ValidationCategory::Port)
            .iter()
            .any(|f| f.message.contains("collision") || f.message.contains("multiple services")));
    }

    #[test]
    fn test_validate_privileged_port_warning() {
        let ports = PortConfigBuilder::new()
            .custom(crate::ports::PortMapping::new("http", 80))
            .build();

        let config = DeployConfig::new("test-app").with_port_config(ports);
        let validator = Validator::new(config);
        let report = validator.validate();

        assert!(report.has_warnings());
        assert!(report.warnings().iter().any(|f| f.message.contains("privileged")));
    }

    #[test]
    fn test_validation_report_display() {
        let mut report = ValidationReport::new();
        report.add(ValidationFinding::error(
            ValidationCategory::EnvVar,
            "Missing DATABASE_URL",
        ));

        let display = format!("{}", report);
        assert!(display.contains("1 error(s)"));
        assert!(display.contains("DATABASE_URL"));
    }

    #[test]
    fn test_validation_report_by_category() {
        let mut report = ValidationReport::new();
        report.add(ValidationFinding::error(ValidationCategory::EnvVar, "Env error"));
        report.add(ValidationFinding::error(ValidationCategory::Port, "Port error"));
        report.add(ValidationFinding::warning(ValidationCategory::EnvVar, "Env warning"));

        let env_findings = report.by_category(ValidationCategory::EnvVar);
        assert_eq!(env_findings.len(), 2);

        let port_findings = report.by_category(ValidationCategory::Port);
        assert_eq!(port_findings.len(), 1);
    }

    #[test]
    fn test_secret_with_default_warning() {
        use crate::env_schema::EnvVarDefinition;

        let mut schema = EnvSchema::new();
        schema.add_variable(
            EnvVarDefinition::new("API_KEY")
                .with_default("insecure_default")
                .secret(),
        );

        let config = DeployConfig::new("test-app").with_env_schema(schema);
        let validator = Validator::new(config);
        let report = validator.validate();

        assert!(report.has_warnings());
        assert!(report.warnings().iter().any(|f| f.message.contains("default value")));
    }

    #[test]
    fn test_validation_errors_to_report() {
        let mut errors = ValidationErrors::new();
        errors.push(ValidationError::new(
            ValidationCategory::EnvVar,
            "Test error",
        ));

        let report: ValidationReport = errors.into();
        assert_eq!(report.error_count(), 1);
    }

    #[test]
    fn test_severity_ordering() {
        assert!(Severity::Info < Severity::Warning);
        assert!(Severity::Warning < Severity::Error);
    }
}
