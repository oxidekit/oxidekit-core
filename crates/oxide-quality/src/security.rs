//! Security audit system
//!
//! Analyzes code for security vulnerabilities and enforces security policies.

use crate::{ErrorCode, SecurityConfig};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Security audit report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityReport {
    /// Critical vulnerabilities count
    pub critical: usize,
    /// High severity vulnerabilities count
    pub high: usize,
    /// Medium severity vulnerabilities count
    pub medium: usize,
    /// Low severity vulnerabilities count
    pub low: usize,
    /// Vulnerability details
    pub vulnerabilities: Vec<SecurityVulnerability>,
    /// Unsafe code usage
    pub unsafe_usage: Vec<UnsafeUsage>,
    /// Forbidden API usage
    pub forbidden_apis: Vec<ForbiddenApiUsage>,
    /// Permission issues
    pub permission_issues: Vec<PermissionIssue>,
    /// Whether security check passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl SecurityReport {
    pub fn new() -> Self {
        Self {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
            vulnerabilities: Vec::new(),
            unsafe_usage: Vec::new(),
            forbidden_apis: Vec::new(),
            permission_issues: Vec::new(),
            passed: true,
            duration_ms: 0,
        }
    }

    pub fn add_vulnerability(&mut self, vuln: SecurityVulnerability) {
        match vuln.severity {
            VulnerabilitySeverity::Critical => self.critical += 1,
            VulnerabilitySeverity::High => self.high += 1,
            VulnerabilitySeverity::Medium => self.medium += 1,
            VulnerabilitySeverity::Low => self.low += 1,
        }
        self.vulnerabilities.push(vuln);
    }
}

impl Default for SecurityReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Security violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityViolation {
    /// Error code
    pub code: ErrorCode,
    /// Category
    pub category: SecurityCategory,
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Message
    pub message: String,
    /// Severity
    pub severity: VulnerabilitySeverity,
    /// Suggested fix
    pub fix: Option<String>,
}

/// Vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityVulnerability {
    /// Advisory ID (e.g., RUSTSEC-2024-0001)
    pub id: String,
    /// Title
    pub title: String,
    /// Description
    pub description: String,
    /// Affected package
    pub package: String,
    /// Affected version
    pub version: String,
    /// Patched version (if available)
    pub patched_version: Option<String>,
    /// Severity
    pub severity: VulnerabilitySeverity,
    /// CVSS score (if available)
    pub cvss: Option<f64>,
    /// URL for more information
    pub url: Option<String>,
}

/// Vulnerability severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VulnerabilitySeverity {
    Critical,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for VulnerabilitySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VulnerabilitySeverity::Critical => write!(f, "CRITICAL"),
            VulnerabilitySeverity::High => write!(f, "HIGH"),
            VulnerabilitySeverity::Medium => write!(f, "MEDIUM"),
            VulnerabilitySeverity::Low => write!(f, "LOW"),
        }
    }
}

/// Security check category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SecurityCategory {
    /// Dependency vulnerability
    Dependency,
    /// Unsafe code usage
    UnsafeCode,
    /// Forbidden API usage
    ForbiddenApi,
    /// Permission violation
    Permission,
    /// Secret exposure
    SecretExposure,
}

/// Unsafe code usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnsafeUsage {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Code snippet
    pub snippet: String,
    /// Reason for unsafe usage
    pub reason: Option<String>,
    /// Whether this is allowed
    pub allowed: bool,
}

/// Forbidden API usage record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForbiddenApiUsage {
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// API name
    pub api: String,
    /// Usage context
    pub context: String,
}

/// Permission issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionIssue {
    /// Required permission
    pub permission: String,
    /// Issue description
    pub description: String,
    /// Severity
    pub severity: VulnerabilitySeverity,
}

/// Run security checks
pub fn check(project_path: &Path, config: &SecurityConfig) -> SecurityReport {
    let start = std::time::Instant::now();
    let mut report = SecurityReport::new();

    if !config.enabled {
        tracing::debug!("Security checks disabled");
        return report;
    }

    tracing::info!("Running security audit");

    // Audit dependencies
    if config.audit_deps {
        audit_dependencies(project_path, config, &mut report);
    }

    // Check unsafe code
    if config.check_unsafe {
        check_unsafe_code(project_path, config, &mut report);
    }

    // Check forbidden APIs
    check_forbidden_apis(project_path, config, &mut report);

    // Check permissions
    if config.check_permissions {
        check_permissions(project_path, config, &mut report);
    }

    // Check for exposed secrets
    check_secret_exposure(project_path, &mut report);

    // Determine if passed
    let limits = &config.max_vulnerabilities;
    report.passed = report.critical <= limits.critical
        && report.high <= limits.high
        && report.medium <= limits.medium
        && report.low <= limits.low
        && report.forbidden_apis.is_empty()
        && report.permission_issues.iter().all(|p| p.severity != VulnerabilitySeverity::Critical);

    report.duration_ms = start.elapsed().as_millis() as u64;
    report
}

/// Audit dependencies for known vulnerabilities
fn audit_dependencies(project_path: &Path, _config: &SecurityConfig, report: &mut SecurityReport) {
    let cargo_lock_path = project_path.join("Cargo.lock");

    if !cargo_lock_path.exists() {
        tracing::warn!("Cargo.lock not found, skipping dependency audit");
        return;
    }

    // Parse Cargo.lock
    let content = match std::fs::read_to_string(&cargo_lock_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to read Cargo.lock: {}", e);
            return;
        }
    };

    let lock_file: toml::Value = match toml::from_str(&content) {
        Ok(v) => v,
        Err(e) => {
            tracing::warn!("Failed to parse Cargo.lock: {}", e);
            return;
        }
    };

    // Check for known vulnerable packages
    // In production, this would check against a vulnerability database
    let known_vulnerabilities = get_known_vulnerabilities();

    if let Some(packages) = lock_file.get("package").and_then(|p| p.as_array()) {
        for package in packages {
            let name = package.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let version = package.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0");

            // Check if package has known vulnerabilities
            for vuln in &known_vulnerabilities {
                if vuln.package == name && is_version_affected(version, &vuln.affected_versions) {
                    report.add_vulnerability(SecurityVulnerability {
                        id: vuln.id.clone(),
                        title: vuln.title.clone(),
                        description: vuln.description.clone(),
                        package: name.to_string(),
                        version: version.to_string(),
                        patched_version: vuln.patched_version.clone(),
                        severity: vuln.severity,
                        cvss: vuln.cvss,
                        url: vuln.url.clone(),
                    });
                }
            }
        }
    }
}

/// Known vulnerability entry (would be populated from RustSec or similar)
struct KnownVulnerability {
    id: String,
    title: String,
    description: String,
    package: String,
    affected_versions: String,
    patched_version: Option<String>,
    severity: VulnerabilitySeverity,
    cvss: Option<f64>,
    url: Option<String>,
}

/// Get known vulnerabilities database
fn get_known_vulnerabilities() -> Vec<KnownVulnerability> {
    // In production, this would query RustSec advisory database
    // For now, return empty (no false positives)
    vec![]
}

/// Check if a version is affected by vulnerability
fn is_version_affected(_version: &str, _affected: &str) -> bool {
    // In production, this would do proper semver comparison
    false
}

/// Check for unsafe code usage
fn check_unsafe_code(project_path: &Path, config: &SecurityConfig, report: &mut SecurityReport) {
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only check Rust source files
        if path.extension().map(|e| e != "rs").unwrap_or(true) {
            continue;
        }

        // Skip target directory
        if path.to_string_lossy().contains("target/") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_str = path.to_string_lossy().to_string();

        // Find unsafe blocks
        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check for unsafe keyword
            if trimmed.contains("unsafe") && !trimmed.starts_with("//") && !trimmed.starts_with("/*") {
                // Check if this is in an allowed crate
                let crate_name = extract_crate_name(&file_str);
                let allowed = config.allow_unsafe_crates.iter()
                    .any(|c| crate_name.contains(c));

                // Extract context
                let has_safety_comment = line_num > 0 && content.lines()
                    .nth(line_num.saturating_sub(1))
                    .map(|l| l.contains("SAFETY:") || l.contains("Safety:"))
                    .unwrap_or(false);

                report.unsafe_usage.push(UnsafeUsage {
                    file: file_str.clone(),
                    line: line_num + 1,
                    snippet: trimmed.to_string(),
                    reason: if has_safety_comment {
                        content.lines()
                            .nth(line_num.saturating_sub(1))
                            .map(|l| l.to_string())
                    } else {
                        None
                    },
                    allowed,
                });

                if !allowed && !has_safety_comment {
                    report.add_vulnerability(SecurityVulnerability {
                        id: format!("UNSAFE-{}", line_num + 1),
                        title: "Unsafe code without SAFETY comment".to_string(),
                        description: format!(
                            "Unsafe code found in {} at line {} without documented safety justification",
                            file_str, line_num + 1
                        ),
                        package: crate_name,
                        version: "0.0.0".to_string(),
                        patched_version: None,
                        severity: VulnerabilitySeverity::Medium,
                        cvss: None,
                        url: None,
                    });
                }
            }
        }
    }
}

/// Check for forbidden API usage
fn check_forbidden_apis(project_path: &Path, config: &SecurityConfig, report: &mut SecurityReport) {
    if config.forbidden_apis.is_empty() {
        return;
    }

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Only check Rust source files
        if path.extension().map(|e| e != "rs").unwrap_or(true) {
            continue;
        }

        // Skip target directory
        if path.to_string_lossy().contains("target/") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_str = path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("/*") {
                continue;
            }

            for forbidden_api in &config.forbidden_apis {
                // Check for use statements
                if line.contains(&format!("use {}", forbidden_api))
                    || line.contains(&format!("{}(", forbidden_api.split("::").last().unwrap_or(forbidden_api)))
                {
                    report.forbidden_apis.push(ForbiddenApiUsage {
                        file: file_str.clone(),
                        line: line_num + 1,
                        api: forbidden_api.clone(),
                        context: trimmed.to_string(),
                    });
                }
            }
        }
    }
}

/// Check permission declarations
fn check_permissions(project_path: &Path, config: &SecurityConfig, report: &mut SecurityReport) {
    // Check for oxide.toml or similar config
    let config_paths = [
        project_path.join("oxide.toml"),
        project_path.join("Oxide.toml"),
    ];

    let mut declared_permissions: Vec<String> = Vec::new();

    for config_path in &config_paths {
        if config_path.exists() {
            if let Ok(content) = std::fs::read_to_string(config_path) {
                if let Ok(config_value) = toml::from_str::<toml::Value>(&content) {
                    if let Some(perms) = config_value.get("permissions") {
                        if let Some(arr) = perms.as_array() {
                            for perm in arr {
                                if let Some(s) = perm.as_str() {
                                    declared_permissions.push(s.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check if required permissions are declared
    for required in &config.required_permissions {
        if !declared_permissions.contains(required) {
            report.permission_issues.push(PermissionIssue {
                permission: required.clone(),
                description: format!("Required permission '{}' is not declared", required),
                severity: VulnerabilitySeverity::High,
            });
        }
    }

    // Detect potential permission needs based on code analysis
    let detected_permissions = detect_permissions(project_path);

    for (permission, usage) in detected_permissions {
        if !declared_permissions.contains(&permission) {
            report.permission_issues.push(PermissionIssue {
                permission: permission.clone(),
                description: format!(
                    "Code uses {} API which may require '{}' permission",
                    usage, permission
                ),
                severity: VulnerabilitySeverity::Medium,
            });
        }
    }
}

/// Detect permissions that might be needed based on code usage
fn detect_permissions(project_path: &Path) -> Vec<(String, String)> {
    let mut permissions = Vec::new();

    let permission_indicators = [
        ("std::fs::", "filesystem", "Read/write files"),
        ("std::net::", "network", "Network access"),
        ("std::process::", "process", "Spawn processes"),
        ("std::env::", "environment", "Environment variables"),
        ("reqwest::", "network", "HTTP requests"),
        ("tokio::fs::", "filesystem", "Async file operations"),
        ("tokio::net::", "network", "Async network"),
    ];

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if path.extension().map(|e| e != "rs").unwrap_or(true) {
            continue;
        }

        if path.to_string_lossy().contains("target/") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for (indicator, permission, usage) in &permission_indicators {
            if content.contains(indicator) {
                let entry = (permission.to_string(), usage.to_string());
                if !permissions.contains(&entry) {
                    permissions.push(entry);
                }
            }
        }
    }

    permissions
}

/// Check for exposed secrets
fn check_secret_exposure(project_path: &Path, report: &mut SecurityReport) {
    let secret_patterns = [
        ("API_KEY", r"[A-Za-z0-9_]{20,}"),
        ("SECRET", r"[A-Za-z0-9_]{16,}"),
        ("PASSWORD", r".+"),
        ("TOKEN", r"[A-Za-z0-9_\-]{20,}"),
        ("PRIVATE_KEY", r".+"),
    ];

    let dangerous_files = [
        ".env",
        ".env.local",
        ".env.production",
        "credentials.json",
        "secrets.json",
        "secrets.toml",
    ];

    // Check for dangerous files that shouldn't be committed
    for dangerous in &dangerous_files {
        let path = project_path.join(dangerous);
        if path.exists() {
            // Check if it's in .gitignore
            let gitignore_path = project_path.join(".gitignore");
            let is_ignored = if gitignore_path.exists() {
                std::fs::read_to_string(&gitignore_path)
                    .map(|c| c.lines().any(|l| l.trim() == *dangerous || l.contains(dangerous)))
                    .unwrap_or(false)
            } else {
                false
            };

            if !is_ignored {
                report.add_vulnerability(SecurityVulnerability {
                    id: format!("SECRET-FILE-{}", dangerous.replace('.', "_").to_uppercase()),
                    title: format!("Sensitive file '{}' may be exposed", dangerous),
                    description: format!(
                        "File '{}' exists and may not be properly excluded from version control",
                        dangerous
                    ),
                    package: "project".to_string(),
                    version: "0.0.0".to_string(),
                    patched_version: None,
                    severity: VulnerabilitySeverity::High,
                    cvss: None,
                    url: None,
                });
            }
        }
    }

    // Check source files for hardcoded secrets
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let ext = path.extension().map(|e| e.to_string_lossy().to_string());
        let check_exts = ["rs", "toml", "json", "yaml", "yml", "oui"];

        if !ext.map(|e| check_exts.contains(&e.as_str())).unwrap_or(false) {
            continue;
        }

        if path.to_string_lossy().contains("target/") {
            continue;
        }

        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let file_str = path.to_string_lossy().to_string();

        for (line_num, line) in content.lines().enumerate() {
            let upper = line.to_uppercase();

            for (secret_type, _pattern) in &secret_patterns {
                // Look for assignment patterns
                if upper.contains(secret_type) && (line.contains('=') || line.contains(':')) {
                    // Skip if it's an env var reference
                    if line.contains("env::var") || line.contains("std::env") || line.contains("${") {
                        continue;
                    }

                    // Check if there's a literal value
                    if line.contains('"') || line.contains('\'') {
                        let has_placeholder = line.contains("your_") || line.contains("YOUR_")
                            || line.contains("xxx") || line.contains("XXX")
                            || line.contains("changeme") || line.contains("CHANGEME");

                        if !has_placeholder {
                            report.add_vulnerability(SecurityVulnerability {
                                id: format!("SECRET-EXPOSED-{}-{}", secret_type, line_num + 1),
                                title: format!("Potential {} exposed in source code", secret_type),
                                description: format!(
                                    "Found potential hardcoded {} in {} at line {}",
                                    secret_type, file_str, line_num + 1
                                ),
                                package: "project".to_string(),
                                version: "0.0.0".to_string(),
                                patched_version: None,
                                severity: VulnerabilitySeverity::High,
                                cvss: None,
                                url: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Extract crate name from file path
fn extract_crate_name(file_path: &str) -> String {
    if let Some(idx) = file_path.find("/src/") {
        let before_src = &file_path[..idx];
        if let Some(last_slash) = before_src.rfind('/') {
            return before_src[last_slash + 1..].to_string();
        }
    }
    "unknown".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_report() {
        let mut report = SecurityReport::new();
        assert!(report.passed);
        assert_eq!(report.critical, 0);

        report.add_vulnerability(SecurityVulnerability {
            id: "TEST-001".to_string(),
            title: "Test vulnerability".to_string(),
            description: "Test".to_string(),
            package: "test".to_string(),
            version: "0.0.0".to_string(),
            patched_version: None,
            severity: VulnerabilitySeverity::High,
            cvss: None,
            url: None,
        });

        assert_eq!(report.high, 1);
    }

    #[test]
    fn test_extract_crate_name() {
        assert_eq!(
            extract_crate_name("/home/user/project/my-crate/src/lib.rs"),
            "my-crate"
        );
    }

    #[test]
    fn test_severity_display() {
        assert_eq!(VulnerabilitySeverity::Critical.to_string(), "CRITICAL");
        assert_eq!(VulnerabilitySeverity::High.to_string(), "HIGH");
    }
}
