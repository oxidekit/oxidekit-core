//! Secret handling standards
//!
//! This module provides functionality for managing secrets securely across
//! different environments: local development, CI/CD, and production deployments.

use crate::error::{DeployResult, ValidationCategory, ValidationError, ValidationErrors};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Secret storage backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SecretBackend {
    /// Environment variables (local .env file)
    EnvFile,
    /// OS keychain (macOS Keychain, Windows Credential Manager, Linux Secret Service)
    OsKeychain,
    /// GitHub Actions secrets
    GitHubSecrets,
    /// Railway secrets
    Railway,
    /// Fly.io secrets
    FlyIo,
    /// Render secrets
    Render,
    /// AWS Secrets Manager
    AwsSecretsManager,
    /// HashiCorp Vault
    Vault,
    /// Custom/other
    Custom,
}

impl std::fmt::Display for SecretBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvFile => write!(f, ".env file"),
            Self::OsKeychain => write!(f, "OS keychain"),
            Self::GitHubSecrets => write!(f, "GitHub Actions secrets"),
            Self::Railway => write!(f, "Railway secrets"),
            Self::FlyIo => write!(f, "Fly.io secrets"),
            Self::Render => write!(f, "Render secrets"),
            Self::AwsSecretsManager => write!(f, "AWS Secrets Manager"),
            Self::Vault => write!(f, "HashiCorp Vault"),
            Self::Custom => write!(f, "Custom backend"),
        }
    }
}

/// A secret definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecretDefinition {
    /// Secret name
    pub name: String,
    /// Description
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this secret is required
    #[serde(default = "default_true")]
    pub required: bool,
    /// Recommended backends for this secret
    #[serde(default)]
    pub recommended_backends: Vec<SecretBackend>,
    /// Rotation period in days (0 = no rotation)
    #[serde(default)]
    pub rotation_days: u32,
    /// Pattern to match secret values (for validation)
    #[serde(default)]
    pub pattern: Option<String>,
}

fn default_true() -> bool {
    true
}

impl SecretDefinition {
    /// Create a new secret definition
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            required: true,
            recommended_backends: vec![SecretBackend::EnvFile],
            rotation_days: 0,
            pattern: None,
        }
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Set recommended backends
    pub fn with_backends(mut self, backends: Vec<SecretBackend>) -> Self {
        self.recommended_backends = backends;
        self
    }

    /// Set rotation period
    pub fn with_rotation(mut self, days: u32) -> Self {
        self.rotation_days = days;
        self
    }

    /// Set validation pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }
}

/// Secret configuration for an application
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecretConfig {
    /// Secret definitions
    #[serde(default)]
    pub secrets: Vec<SecretDefinition>,
    /// Files to check for committed secrets
    #[serde(default)]
    pub scan_patterns: Vec<String>,
    /// Patterns to exclude from scanning
    #[serde(default)]
    pub exclude_patterns: Vec<String>,
}

impl SecretConfig {
    /// Create a new empty secret configuration
    pub fn new() -> Self {
        Self {
            secrets: Vec::new(),
            scan_patterns: vec![
                "*.env".to_string(),
                "*.env.*".to_string(),
                "*.pem".to_string(),
                "*.key".to_string(),
                "*credentials*".to_string(),
                "*secret*".to_string(),
            ],
            exclude_patterns: vec![
                "*.example".to_string(),
                "*.sample".to_string(),
                "*.template".to_string(),
                ".env.example".to_string(),
            ],
        }
    }

    /// Add a secret definition
    pub fn add(&mut self, secret: SecretDefinition) {
        self.secrets.push(secret);
    }

    /// Get a secret by name
    pub fn get(&self, name: &str) -> Option<&SecretDefinition> {
        self.secrets.iter().find(|s| s.name == name)
    }

    /// Get all required secrets
    pub fn required_secrets(&self) -> Vec<&SecretDefinition> {
        self.secrets.iter().filter(|s| s.required).collect()
    }

    /// Load configuration from TOML
    pub fn from_toml(content: &str) -> DeployResult<Self> {
        Ok(toml::from_str(content)?)
    }

    /// Serialize to TOML
    pub fn to_toml(&self) -> DeployResult<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

/// Patterns that indicate secret values
const SECRET_PATTERNS: &[&str] = &[
    // API keys and tokens
    r#"(?i)(api[_-]?key|api[_-]?secret|access[_-]?token|auth[_-]?token)\s*[=:]\s*['"]?[a-zA-Z0-9_-]{20,}['"]?"#,
    // AWS credentials
    r#"(?i)(aws[_-]?access[_-]?key[_-]?id|aws[_-]?secret[_-]?access[_-]?key)\s*[=:]\s*['"]?[A-Z0-9]{16,}['"]?"#,
    // Private keys
    r"-----BEGIN (RSA |EC |DSA |OPENSSH )?PRIVATE KEY-----",
    // Database URLs with credentials
    r"(?i)(postgres|mysql|mongodb|redis)://[^:]+:[^@]+@",
    // Bearer tokens
    r"(?i)bearer\s+[a-zA-Z0-9_.=-]{20,}",
    // JWT tokens
    r"eyJ[a-zA-Z0-9_-]*\.eyJ[a-zA-Z0-9_-]*\.[a-zA-Z0-9_-]*",
    // GitHub tokens
    r"ghp_[a-zA-Z0-9]{36}",
    r"gho_[a-zA-Z0-9]{36}",
    r"ghu_[a-zA-Z0-9]{36}",
    // Slack tokens
    r"xox[baprs]-[a-zA-Z0-9-]+",
    // Stripe keys
    r"sk_live_[a-zA-Z0-9]{24,}",
    r"rk_live_[a-zA-Z0-9]{24,}",
    // Generic secrets in env format
    r#"(?i)(password|passwd|secret|private[_-]?key)\s*[=:]\s*['"]?[^\s'"]{8,}['"]?"#,
];

/// Scanner for detecting committed secrets
pub struct SecretScanner {
    /// Patterns to match secret files
    file_patterns: Vec<glob::Pattern>,
    /// Patterns to exclude
    exclude_patterns: Vec<glob::Pattern>,
    /// Content patterns for secrets
    content_patterns: Vec<regex::Regex>,
    /// Additional file extensions to scan
    scan_extensions: HashSet<String>,
}

impl SecretScanner {
    /// Create a new secret scanner with default patterns
    pub fn new() -> Self {
        let file_patterns = vec![
            glob::Pattern::new("*.env").unwrap(),
            glob::Pattern::new(".env*").unwrap(),
            glob::Pattern::new("*.pem").unwrap(),
            glob::Pattern::new("*.key").unwrap(),
        ];

        let exclude_patterns = vec![
            glob::Pattern::new("*.example").unwrap(),
            glob::Pattern::new("*.sample").unwrap(),
            glob::Pattern::new("*.template").unwrap(),
            glob::Pattern::new(".env.example").unwrap(),
        ];

        let content_patterns = SECRET_PATTERNS
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();

        let mut scan_extensions = HashSet::new();
        scan_extensions.insert("env".to_string());
        scan_extensions.insert("yaml".to_string());
        scan_extensions.insert("yml".to_string());
        scan_extensions.insert("json".to_string());
        scan_extensions.insert("toml".to_string());
        scan_extensions.insert("conf".to_string());
        scan_extensions.insert("config".to_string());
        scan_extensions.insert("properties".to_string());

        Self {
            file_patterns,
            exclude_patterns,
            content_patterns,
            scan_extensions,
        }
    }

    /// Create scanner from configuration
    pub fn from_config(config: &SecretConfig) -> DeployResult<Self> {
        let file_patterns = config
            .scan_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let exclude_patterns = config
            .exclude_patterns
            .iter()
            .filter_map(|p| glob::Pattern::new(p).ok())
            .collect();

        let content_patterns = SECRET_PATTERNS
            .iter()
            .filter_map(|p| regex::Regex::new(p).ok())
            .collect();

        Ok(Self {
            file_patterns,
            exclude_patterns,
            content_patterns,
            scan_extensions: HashSet::new(),
        })
    }

    /// Check if a file should be scanned based on patterns
    fn should_scan_file(&self, path: &Path) -> bool {
        let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

        // Check exclude patterns first
        for pattern in &self.exclude_patterns {
            if pattern.matches(filename) {
                return false;
            }
        }

        // Check file patterns
        for pattern in &self.file_patterns {
            if pattern.matches(filename) {
                return true;
            }
        }

        // Check extensions
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            if self.scan_extensions.contains(ext) {
                return true;
            }
        }

        false
    }

    /// Scan a directory for committed secrets
    pub fn scan_directory(&self, dir: impl AsRef<Path>) -> DeployResult<Vec<SecretFinding>> {
        let mut findings = Vec::new();

        for entry in WalkDir::new(dir.as_ref())
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // Skip hidden directories and common non-source directories
                let name = e.file_name().to_str().unwrap_or("");
                !name.starts_with('.') || name == ".env"
            })
        {
            let entry = entry?;
            let path = entry.path();

            if !path.is_file() {
                continue;
            }

            // Check if this file might contain secrets based on name
            if self.should_scan_file(path) {
                findings.push(SecretFinding {
                    file: path.to_path_buf(),
                    line: None,
                    finding_type: FindingType::SensitiveFile,
                    description: format!(
                        "Potentially sensitive file: {}",
                        path.file_name().unwrap_or_default().to_string_lossy()
                    ),
                    snippet: None,
                });
            }

            // Scan file contents for secret patterns
            if let Ok(content) = std::fs::read_to_string(path) {
                for (line_num, line) in content.lines().enumerate() {
                    for pattern in &self.content_patterns {
                        if pattern.is_match(line) {
                            findings.push(SecretFinding {
                                file: path.to_path_buf(),
                                line: Some(line_num + 1),
                                finding_type: FindingType::SecretPattern,
                                description: "Potential secret detected in content".to_string(),
                                snippet: Some(redact_secret(line)),
                            });
                            break; // Only report once per line
                        }
                    }
                }
            }
        }

        Ok(findings)
    }

    /// Scan a single file for secrets
    pub fn scan_file(&self, path: impl AsRef<Path>) -> DeployResult<Vec<SecretFinding>> {
        let path = path.as_ref();
        let mut findings = Vec::new();

        if !path.exists() {
            return Ok(findings);
        }

        if self.should_scan_file(path) {
            findings.push(SecretFinding {
                file: path.to_path_buf(),
                line: None,
                finding_type: FindingType::SensitiveFile,
                description: format!(
                    "Potentially sensitive file: {}",
                    path.file_name().unwrap_or_default().to_string_lossy()
                ),
                snippet: None,
            });
        }

        if let Ok(content) = std::fs::read_to_string(path) {
            for (line_num, line) in content.lines().enumerate() {
                for pattern in &self.content_patterns {
                    if pattern.is_match(line) {
                        findings.push(SecretFinding {
                            file: path.to_path_buf(),
                            line: Some(line_num + 1),
                            finding_type: FindingType::SecretPattern,
                            description: "Potential secret detected".to_string(),
                            snippet: Some(redact_secret(line)),
                        });
                        break;
                    }
                }
            }
        }

        Ok(findings)
    }
}

impl Default for SecretScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// A secret finding from scanning
#[derive(Debug, Clone)]
pub struct SecretFinding {
    /// File containing the finding
    pub file: PathBuf,
    /// Line number (if applicable)
    pub line: Option<usize>,
    /// Type of finding
    pub finding_type: FindingType,
    /// Description of the finding
    pub description: String,
    /// Redacted snippet
    pub snippet: Option<String>,
}

impl std::fmt::Display for SecretFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.finding_type, self.description)?;
        write!(f, " in {}", self.file.display())?;
        if let Some(line) = self.line {
            write!(f, " at line {}", line)?;
        }
        if let Some(ref snippet) = self.snippet {
            write!(f, "\n  > {}", snippet)?;
        }
        Ok(())
    }
}

/// Type of secret finding
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FindingType {
    /// Sensitive file detected
    SensitiveFile,
    /// Secret pattern in content
    SecretPattern,
    /// Hardcoded credential
    HardcodedCredential,
}

impl std::fmt::Display for FindingType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SensitiveFile => write!(f, "SENSITIVE_FILE"),
            Self::SecretPattern => write!(f, "SECRET_PATTERN"),
            Self::HardcodedCredential => write!(f, "HARDCODED"),
        }
    }
}

/// Redact a potential secret value in a string
fn redact_secret(line: &str) -> String {
    // Simple redaction - replace anything that looks like a secret value
    let re = regex::Regex::new(r#"[=:]\s*['"]?([a-zA-Z0-9_\-\.]{8,})['"]?"#).unwrap();
    re.replace_all(line, "=*****").to_string()
}

/// Validate secrets configuration
pub fn validate_secrets(config: &SecretConfig, project_dir: impl AsRef<Path>) -> ValidationErrors {
    let mut errors = ValidationErrors::new();

    // Check for committed sensitive files
    let scanner = SecretScanner::default();
    if let Ok(findings) = scanner.scan_directory(project_dir) {
        for finding in findings {
            errors.push(
                ValidationError::new(ValidationCategory::Secret, finding.description.clone())
                    .with_file(finding.file),
            );
        }
    }

    // Check that required secrets are defined in environment
    for secret in config.required_secrets() {
        if std::env::var(&secret.name).is_err() {
            errors.push(ValidationError::new(
                ValidationCategory::Secret,
                format!("Required secret '{}' is not set", secret.name),
            ));
        }
    }

    errors
}

/// Generate secret management documentation
pub fn generate_secret_docs(config: &SecretConfig) -> String {
    let mut lines = Vec::new();

    lines.push("# Secret Management".to_string());
    lines.push(String::new());
    lines.push("This document describes the secrets required for this application.".to_string());
    lines.push(String::new());

    lines.push("## Required Secrets".to_string());
    lines.push(String::new());

    for secret in config.required_secrets() {
        lines.push(format!("### {}", secret.name));
        if let Some(ref desc) = secret.description {
            lines.push(desc.clone());
        }
        if !secret.recommended_backends.is_empty() {
            let backends: Vec<_> = secret
                .recommended_backends
                .iter()
                .map(|b| format!("{}", b))
                .collect();
            lines.push(format!("**Recommended storage:** {}", backends.join(", ")));
        }
        if secret.rotation_days > 0 {
            lines.push(format!("**Rotation:** Every {} days", secret.rotation_days));
        }
        lines.push(String::new());
    }

    lines.push("## Environment Setup".to_string());
    lines.push(String::new());

    lines.push("### Local Development".to_string());
    lines.push("1. Copy `.env.example` to `.env`".to_string());
    lines.push("2. Fill in the secret values".to_string());
    lines.push("3. Never commit `.env` to version control".to_string());
    lines.push(String::new());

    lines.push("### CI/CD (GitHub Actions)".to_string());
    lines.push("Add secrets in: Settings > Secrets and variables > Actions".to_string());
    lines.push(String::new());

    for secret in config.required_secrets() {
        lines.push(format!("- `{}`", secret.name));
    }
    lines.push(String::new());

    lines.push("### Production Deployment".to_string());
    lines.push("Use your platform's secret management:".to_string());
    lines.push("- **Railway:** Project > Variables".to_string());
    lines.push("- **Fly.io:** `fly secrets set KEY=value`".to_string());
    lines.push("- **Render:** Dashboard > Environment".to_string());

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_secret_definition() {
        let secret = SecretDefinition::new("API_KEY")
            .with_description("External API key")
            .with_backends(vec![SecretBackend::GitHubSecrets, SecretBackend::EnvFile])
            .with_rotation(90);

        assert_eq!(secret.name, "API_KEY");
        assert!(secret.required);
        assert_eq!(secret.rotation_days, 90);
        assert_eq!(secret.recommended_backends.len(), 2);
    }

    #[test]
    fn test_secret_config() {
        let mut config = SecretConfig::new();
        config.add(SecretDefinition::new("DATABASE_URL"));
        config.add(SecretDefinition::new("API_KEY").optional());

        assert_eq!(config.secrets.len(), 2);
        assert_eq!(config.required_secrets().len(), 1);
        assert!(config.get("DATABASE_URL").is_some());
    }

    #[test]
    fn test_secret_scanner_patterns() {
        let scanner = SecretScanner::new();

        // Test file pattern matching
        assert!(scanner.should_scan_file(Path::new(".env")));
        assert!(scanner.should_scan_file(Path::new("secrets.env")));
        assert!(!scanner.should_scan_file(Path::new(".env.example")));
        assert!(!scanner.should_scan_file(Path::new("config.txt")));
    }

    #[test]
    fn test_secret_scanner_content() {
        let temp_dir = TempDir::new().unwrap();
        let secret_file = temp_dir.path().join("config.yaml");

        let mut file = std::fs::File::create(&secret_file).unwrap();
        writeln!(file, "api_key: sk_live_abcdefghijklmnopqrstuvwxyz").unwrap();
        writeln!(file, "name: test").unwrap();

        let scanner = SecretScanner::new();
        let findings = scanner.scan_file(&secret_file).unwrap();

        // Should find the Stripe key pattern
        assert!(findings.iter().any(|f| f.finding_type == FindingType::SecretPattern));
    }

    #[test]
    fn test_redact_secret() {
        let line = "API_KEY=sk_live_abcdefghijklmnop";
        let redacted = redact_secret(line);
        assert!(redacted.contains("*****"));
        assert!(!redacted.contains("sk_live_"));
    }

    #[test]
    fn test_secret_config_toml_roundtrip() {
        let mut config = SecretConfig::new();
        config.add(SecretDefinition::new("DATABASE_URL").with_description("Database connection"));
        config.add(SecretDefinition::new("API_KEY").optional());

        let toml = config.to_toml().unwrap();
        let parsed = SecretConfig::from_toml(&toml).unwrap();

        assert_eq!(parsed.secrets.len(), config.secrets.len());
    }

    #[test]
    fn test_generate_secret_docs() {
        let mut config = SecretConfig::new();
        config.add(
            SecretDefinition::new("DATABASE_URL")
                .with_description("PostgreSQL connection string")
                .with_backends(vec![SecretBackend::EnvFile]),
        );

        let docs = generate_secret_docs(&config);
        assert!(docs.contains("DATABASE_URL"));
        assert!(docs.contains("Secret Management"));
        assert!(docs.contains("GitHub Actions"));
    }

    #[test]
    fn test_secret_backend_display() {
        assert_eq!(format!("{}", SecretBackend::EnvFile), ".env file");
        assert_eq!(format!("{}", SecretBackend::GitHubSecrets), "GitHub Actions secrets");
        assert_eq!(format!("{}", SecretBackend::FlyIo), "Fly.io secrets");
    }
}
