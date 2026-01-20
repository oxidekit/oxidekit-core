//! Portability checker for compile-time and runtime validation.
//!
//! Analyzes code, plugins, and projects for portability issues.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::PortabilityResult;
use crate::plugin::PortabilityManifest;
use crate::target::Target;

/// Portability checker that analyzes code for cross-platform compatibility.
#[derive(Debug)]
pub struct PortabilityChecker {
    /// Target to check against
    target: Target,
    /// Known desktop-only APIs
    desktop_only_apis: Vec<DesktopOnlyApiPattern>,
    /// Known web-only APIs
    web_only_apis: Vec<WebOnlyApiPattern>,
    /// Configuration
    config: CheckerConfig,
}

/// Configuration for the portability checker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckerConfig {
    /// Treat warnings as errors
    #[serde(default)]
    pub strict: bool,
    /// Ignore specific patterns
    #[serde(default)]
    pub ignore_patterns: Vec<String>,
    /// Additional target checks
    #[serde(default)]
    pub additional_targets: Vec<String>,
    /// Check dependencies
    #[serde(default = "default_true")]
    pub check_dependencies: bool,
}

fn default_true() -> bool {
    true
}

impl Default for CheckerConfig {
    fn default() -> Self {
        Self {
            strict: false,
            ignore_patterns: Vec::new(),
            additional_targets: Vec::new(),
            check_dependencies: true,
        }
    }
}

/// Pattern for identifying desktop-only APIs.
#[derive(Debug, Clone)]
pub struct DesktopOnlyApiPattern {
    /// Pattern name
    pub name: String,
    /// Module path patterns to match
    pub module_patterns: Vec<String>,
    /// Function name patterns
    pub function_patterns: Vec<String>,
    /// Reason this is desktop-only
    pub reason: String,
    /// Suggested alternative
    pub alternative: Option<String>,
}

/// Pattern for identifying web-only APIs.
#[derive(Debug, Clone)]
pub struct WebOnlyApiPattern {
    /// Pattern name
    pub name: String,
    /// Module path patterns to match
    pub module_patterns: Vec<String>,
    /// Reason this is web-only
    pub reason: String,
}

impl PortabilityChecker {
    /// Create a new checker for the current target.
    pub fn new() -> Self {
        Self::for_target(Target::current())
    }

    /// Create a checker for a specific target.
    pub fn for_target(target: Target) -> Self {
        Self {
            target,
            desktop_only_apis: Self::default_desktop_only_patterns(),
            web_only_apis: Self::default_web_only_patterns(),
            config: CheckerConfig::default(),
        }
    }

    /// Configure the checker.
    pub fn with_config(mut self, config: CheckerConfig) -> Self {
        self.config = config;
        self
    }

    /// Get default patterns for desktop-only APIs.
    fn default_desktop_only_patterns() -> Vec<DesktopOnlyApiPattern> {
        vec![
            DesktopOnlyApiPattern {
                name: "std::fs".to_string(),
                module_patterns: vec!["std::fs".to_string()],
                function_patterns: vec![],
                reason: "File system operations are not available on web".to_string(),
                alternative: Some("Use IndexedDB or Storage API on web".to_string()),
            },
            DesktopOnlyApiPattern {
                name: "std::process".to_string(),
                module_patterns: vec!["std::process".to_string()],
                function_patterns: vec![],
                reason: "Process spawning is not available on web or mobile".to_string(),
                alternative: None,
            },
            DesktopOnlyApiPattern {
                name: "std::env".to_string(),
                module_patterns: vec!["std::env".to_string()],
                function_patterns: vec!["var".to_string(), "vars".to_string(), "current_dir".to_string(), "home_dir".to_string()],
                reason: "Environment variables are not available on web".to_string(),
                alternative: Some("Use configuration or local storage".to_string()),
            },
            DesktopOnlyApiPattern {
                name: "native_window".to_string(),
                module_patterns: vec!["winit".to_string(), "tao".to_string()],
                function_patterns: vec![],
                reason: "Native window APIs are desktop-only".to_string(),
                alternative: Some("Use oxide-web for web targets".to_string()),
            },
            DesktopOnlyApiPattern {
                name: "system_tray".to_string(),
                module_patterns: vec!["tray_icon".to_string(), "system-tray".to_string()],
                function_patterns: vec![],
                reason: "System tray is only available on desktop".to_string(),
                alternative: None,
            },
        ]
    }

    /// Get default patterns for web-only APIs.
    fn default_web_only_patterns() -> Vec<WebOnlyApiPattern> {
        vec![
            WebOnlyApiPattern {
                name: "web_sys".to_string(),
                module_patterns: vec!["web_sys".to_string()],
                reason: "Web APIs are only available in browser".to_string(),
            },
            WebOnlyApiPattern {
                name: "wasm_bindgen".to_string(),
                module_patterns: vec!["wasm_bindgen".to_string()],
                reason: "WASM bindings are only available in browser".to_string(),
            },
        ]
    }

    /// Check a plugin manifest for portability issues.
    pub fn check_plugin(&self, manifest: &PortabilityManifest) -> PortabilityReport {
        let mut report = PortabilityReport::new(&manifest.id);

        // Check if plugin supports the target
        if !manifest.supports(&self.target) {
            if let Some(reason) = manifest.portability.unsupported_reason(&self.target) {
                report.add_issue(PortabilityIssue {
                    severity: IssueSeverity::Error,
                    location: IssueLocation::Manifest,
                    message: format!("Plugin does not support target {}: {}", self.target.triple(), reason),
                    suggestion: None,
                    api_name: None,
                });
            }
        }

        // Check individual APIs
        for (api_name, api_portability) in &manifest.apis {
            if !api_portability.supports(&self.target) {
                if let Some(reason) = api_portability.unsupported_reason(&self.target) {
                    report.add_issue(PortabilityIssue {
                        severity: IssueSeverity::Warning,
                        location: IssueLocation::Api(api_name.clone()),
                        message: format!("API '{}' does not support target {}: {}", api_name, self.target.triple(), reason),
                        suggestion: Some("Consider providing an alternative implementation".to_string()),
                        api_name: Some(api_name.clone()),
                    });
                }
            }
        }

        // Check dependencies
        if self.config.check_dependencies {
            for dep in &manifest.dependencies {
                if dep.required && !dep.platforms.is_empty() && !dep.platforms.contains(&self.target.platform()) {
                    report.add_issue(PortabilityIssue {
                        severity: IssueSeverity::Warning,
                        location: IssueLocation::Dependency(dep.name.clone()),
                        message: format!("Dependency '{}' is not used on target {}", dep.name, self.target.triple()),
                        suggestion: Some("This dependency may be unnecessary for this target".to_string()),
                        api_name: None,
                    });
                }
            }
        }

        report.finalize();
        report
    }

    /// Check a project directory for portability issues.
    pub fn check_project(&self, project_path: &Path) -> PortabilityResult<PortabilityReport> {
        let project_name = project_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");

        let mut report = PortabilityReport::new(project_name);

        // Look for plugin manifest
        let manifest_path = project_path.join("oxide-plugin.toml");
        if manifest_path.exists() {
            let manifest = PortabilityManifest::load(&manifest_path)?;
            let plugin_report = self.check_plugin(&manifest);
            report.merge(plugin_report);
        }

        // Check Cargo.toml for problematic dependencies
        let cargo_path = project_path.join("Cargo.toml");
        if cargo_path.exists() {
            self.check_cargo_toml(&cargo_path, &mut report)?;
        }

        // Scan source files
        let src_path = project_path.join("src");
        if src_path.exists() {
            self.scan_source_dir(&src_path, &mut report)?;
        }

        report.finalize();
        Ok(report)
    }

    /// Check Cargo.toml for portability issues.
    fn check_cargo_toml(&self, path: &Path, report: &mut PortabilityReport) -> PortabilityResult<()> {
        let content = std::fs::read_to_string(path)?;

        // Check for desktop-only dependencies
        for pattern in &self.desktop_only_apis {
            for module in &pattern.module_patterns {
                if content.contains(module) && self.target.is_web() {
                    report.add_issue(PortabilityIssue {
                        severity: IssueSeverity::Warning,
                        location: IssueLocation::Dependency(module.clone()),
                        message: format!("Dependency '{}' is desktop-only: {}", module, pattern.reason),
                        suggestion: pattern.alternative.clone(),
                        api_name: None,
                    });
                }
            }
        }

        // Check for web-only dependencies on non-web targets
        for pattern in &self.web_only_apis {
            for module in &pattern.module_patterns {
                if content.contains(module) && !self.target.is_web() {
                    report.add_issue(PortabilityIssue {
                        severity: IssueSeverity::Warning,
                        location: IssueLocation::Dependency(module.clone()),
                        message: format!("Dependency '{}' is web-only: {}", module, pattern.reason),
                        suggestion: None,
                        api_name: None,
                    });
                }
            }
        }

        Ok(())
    }

    /// Scan source directory for portability issues.
    fn scan_source_dir(&self, dir: &Path, report: &mut PortabilityReport) -> PortabilityResult<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                self.scan_source_dir(&path, report)?;
            } else if path.extension().map_or(false, |e| e == "rs") {
                self.scan_source_file(&path, report)?;
            }
        }

        Ok(())
    }

    /// Scan a single source file for portability issues.
    fn scan_source_file(&self, path: &Path, report: &mut PortabilityReport) -> PortabilityResult<()> {
        let content = std::fs::read_to_string(path)?;

        // Check for desktop-only API usage
        for pattern in &self.desktop_only_apis {
            if self.target.is_web() {
                for module in &pattern.module_patterns {
                    if content.contains(&format!("use {}", module))
                        || content.contains(&format!("{}::", module))
                    {
                        // Check if it's cfg-gated
                        if !self.is_cfg_gated(&content, module) {
                            report.add_issue(PortabilityIssue {
                                severity: IssueSeverity::Warning,
                                location: IssueLocation::SourceFile(path.to_path_buf()),
                                message: format!("Use of '{}' is not portable to web: {}", module, pattern.reason),
                                suggestion: pattern.alternative.clone(),
                                api_name: Some(pattern.name.clone()),
                            });
                        }
                    }
                }
            }
        }

        // Check for web-only API usage
        for pattern in &self.web_only_apis {
            if !self.target.is_web() {
                for module in &pattern.module_patterns {
                    if content.contains(&format!("use {}", module))
                        || content.contains(&format!("{}::", module))
                    {
                        if !self.is_cfg_gated(&content, module) {
                            report.add_issue(PortabilityIssue {
                                severity: IssueSeverity::Warning,
                                location: IssueLocation::SourceFile(path.to_path_buf()),
                                message: format!("Use of '{}' is not portable to native: {}", module, pattern.reason),
                                suggestion: None,
                                api_name: Some(pattern.name.clone()),
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if a module usage is properly cfg-gated.
    fn is_cfg_gated(&self, content: &str, module: &str) -> bool {
        // Simple heuristic: check if there's a #[cfg(...)] near the usage
        // A real implementation would use syn to parse the AST

        // Look for patterns like:
        // #[cfg(not(target_arch = "wasm32"))]
        // #[cfg(target_os = "...")]
        let lines: Vec<&str> = content.lines().collect();

        for (i, line) in lines.iter().enumerate() {
            if line.contains(module) {
                // Check previous lines for cfg attributes
                for j in (0..i).rev().take(5) {
                    if lines[j].contains("#[cfg(") {
                        return true;
                    }
                    // Stop if we hit a non-attribute, non-whitespace line
                    let trimmed = lines[j].trim();
                    if !trimmed.is_empty() && !trimmed.starts_with("#[") && !trimmed.starts_with("//") {
                        break;
                    }
                }
            }
        }

        false
    }

    /// Check multiple targets at once.
    pub fn check_for_targets(&self, project_path: &Path, targets: &[Target]) -> PortabilityResult<HashMap<String, PortabilityReport>> {
        let mut reports = HashMap::new();

        for target in targets {
            let checker = PortabilityChecker::for_target(target.clone())
                .with_config(self.config.clone());
            let report = checker.check_project(project_path)?;
            reports.insert(target.triple().to_string(), report);
        }

        Ok(reports)
    }
}

impl Default for PortabilityChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Report from portability checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityReport {
    /// Name of the checked item
    pub name: String,
    /// Overall status
    pub status: ReportStatus,
    /// Issues found
    pub issues: Vec<PortabilityIssue>,
    /// Summary statistics
    pub summary: ReportSummary,
}

impl PortabilityReport {
    /// Create a new report.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            status: ReportStatus::Passed,
            issues: Vec::new(),
            summary: ReportSummary::default(),
        }
    }

    /// Add an issue.
    pub fn add_issue(&mut self, issue: PortabilityIssue) {
        self.issues.push(issue);
    }

    /// Merge another report into this one.
    pub fn merge(&mut self, other: PortabilityReport) {
        self.issues.extend(other.issues);
    }

    /// Finalize the report, computing status and summary.
    pub fn finalize(&mut self) {
        self.summary = ReportSummary {
            total_issues: self.issues.len(),
            errors: self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).count(),
            warnings: self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).count(),
            info: self.issues.iter().filter(|i| i.severity == IssueSeverity::Info).count(),
        };

        self.status = if self.summary.errors > 0 {
            ReportStatus::Failed
        } else if self.summary.warnings > 0 {
            ReportStatus::PassedWithWarnings
        } else {
            ReportStatus::Passed
        };
    }

    /// Check if the report passed (no errors).
    pub fn passed(&self) -> bool {
        self.status != ReportStatus::Failed
    }

    /// Get error issues.
    pub fn errors(&self) -> Vec<&PortabilityIssue> {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Error).collect()
    }

    /// Get warning issues.
    pub fn warnings(&self) -> Vec<&PortabilityIssue> {
        self.issues.iter().filter(|i| i.severity == IssueSeverity::Warning).collect()
    }

    /// Format as a human-readable string.
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Portability Report: {}\n", self.name));
        output.push_str(&format!("Status: {:?}\n", self.status));
        output.push_str(&format!(
            "Summary: {} errors, {} warnings, {} info\n\n",
            self.summary.errors, self.summary.warnings, self.summary.info
        ));

        for issue in &self.issues {
            let icon = match issue.severity {
                IssueSeverity::Error => "ERROR",
                IssueSeverity::Warning => "WARN ",
                IssueSeverity::Info => "INFO ",
            };

            output.push_str(&format!("[{}] {}\n", icon, issue.message));
            output.push_str(&format!("  Location: {:?}\n", issue.location));

            if let Some(suggestion) = &issue.suggestion {
                output.push_str(&format!("  Suggestion: {}\n", suggestion));
            }
            output.push('\n');
        }

        output
    }
}

/// Status of a portability report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportStatus {
    /// All checks passed
    Passed,
    /// Passed but with warnings
    PassedWithWarnings,
    /// Failed due to errors
    Failed,
}

/// Summary statistics for a report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    /// Total number of issues
    pub total_issues: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of info messages
    pub info: usize,
}

/// A portability issue found during checking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityIssue {
    /// Severity of the issue
    pub severity: IssueSeverity,
    /// Location of the issue
    pub location: IssueLocation,
    /// Description of the issue
    pub message: String,
    /// Suggested fix
    pub suggestion: Option<String>,
    /// Name of the problematic API
    pub api_name: Option<String>,
}

/// Severity of a portability issue.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical issue that must be fixed
    Error,
    /// Issue that should be addressed
    Warning,
    /// Informational message
    Info,
}

/// Location of a portability issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IssueLocation {
    /// In the manifest file
    Manifest,
    /// In a specific API
    Api(String),
    /// In a dependency
    Dependency(String),
    /// In a source file
    SourceFile(PathBuf),
    /// Unknown location
    Unknown,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::targets;

    #[test]
    fn test_checker_creation() {
        let checker = PortabilityChecker::new();
        assert!(!checker.desktop_only_apis.is_empty());
    }

    #[test]
    fn test_plugin_check() {
        let checker = PortabilityChecker::for_target(targets::web_wasm32());

        let manifest = PortabilityManifest::new("test", "1.0.0")
            .with_portability(crate::plugin::PluginPortability::desktop_only());

        let report = checker.check_plugin(&manifest);
        assert!(!report.passed());
        assert!(!report.errors().is_empty());
    }

    #[test]
    fn test_portable_plugin_check() {
        let checker = PortabilityChecker::for_target(targets::web_wasm32());

        let manifest = PortabilityManifest::new("test", "1.0.0")
            .with_portability(crate::plugin::PluginPortability::portable());

        let report = checker.check_plugin(&manifest);
        assert!(report.passed());
    }

    #[test]
    fn test_report_formatting() {
        let mut report = PortabilityReport::new("test-plugin");
        report.add_issue(PortabilityIssue {
            severity: IssueSeverity::Warning,
            location: IssueLocation::Manifest,
            message: "Test warning".to_string(),
            suggestion: Some("Fix it".to_string()),
            api_name: None,
        });
        report.finalize();

        let formatted = report.format();
        assert!(formatted.contains("Test warning"));
        assert!(formatted.contains("Fix it"));
    }
}
