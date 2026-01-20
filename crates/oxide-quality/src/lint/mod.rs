//! OUI file linting
//!
//! Static analysis for .oui files with comprehensive lint rules.

mod rules;
mod analyzer;

pub use rules::*;
pub use analyzer::*;

use crate::{LintConfig, ErrorCode, ErrorDomain};
use oxide_compiler::{compile, ComponentIR};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Lint check results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintReport {
    /// Files analyzed
    pub files_analyzed: usize,
    /// Total violations found
    pub total_violations: usize,
    /// Violations by severity
    pub by_severity: SeverityCounts,
    /// All violations
    pub violations: Vec<LintViolation>,
    /// Whether lint passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl LintReport {
    pub fn new() -> Self {
        Self {
            files_analyzed: 0,
            total_violations: 0,
            by_severity: SeverityCounts::default(),
            violations: Vec::new(),
            passed: true,
            duration_ms: 0,
        }
    }

    pub fn add_violation(&mut self, violation: LintViolation) {
        match violation.severity {
            LintSeverity::Error => {
                self.by_severity.errors += 1;
                self.passed = false;
            }
            LintSeverity::Warning => self.by_severity.warnings += 1,
            LintSeverity::Info => self.by_severity.info += 1,
        }
        self.total_violations += 1;
        self.violations.push(violation);
    }
}

impl Default for LintReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Violation counts by severity
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeverityCounts {
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

/// A lint violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintViolation {
    /// Error code
    pub code: ErrorCode,
    /// Rule that was violated
    pub rule: String,
    /// Severity
    pub severity: LintSeverity,
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: usize,
    /// Column number (1-indexed)
    pub column: usize,
    /// Violation message
    pub message: String,
    /// Suggested fix
    pub fix: Option<String>,
    /// Source code snippet
    pub snippet: Option<String>,
}

impl LintViolation {
    pub fn new(
        code: ErrorCode,
        rule: &str,
        severity: LintSeverity,
        file: &str,
        line: usize,
        column: usize,
        message: &str,
    ) -> Self {
        Self {
            code,
            rule: rule.to_string(),
            severity,
            file: file.to_string(),
            line,
            column,
            message: message.to_string(),
            fix: None,
            snippet: None,
        }
    }

    pub fn with_fix(mut self, fix: &str) -> Self {
        self.fix = Some(fix.to_string());
        self
    }

    pub fn with_snippet(mut self, snippet: &str) -> Self {
        self.snippet = Some(snippet.to_string());
        self
    }
}

/// Lint severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LintSeverity {
    Error,
    Warning,
    Info,
}

/// A lint rule definition
#[derive(Debug, Clone)]
pub struct LintRule {
    /// Rule ID (e.g., "no-hardcoded-colors")
    pub id: String,
    /// Rule description
    pub description: String,
    /// Default severity
    pub default_severity: LintSeverity,
    /// Error code
    pub code: ErrorCode,
    /// Whether rule is enabled by default
    pub enabled_by_default: bool,
    /// Rule category
    pub category: RuleCategory,
}

/// Rule categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuleCategory {
    /// Style and formatting
    Style,
    /// Best practices
    BestPractice,
    /// Accessibility
    Accessibility,
    /// Performance
    Performance,
    /// Security
    Security,
    /// Correctness
    Correctness,
}

/// Run lint checks on a project
pub fn check(project_path: &Path, config: &LintConfig) -> LintReport {
    let start = std::time::Instant::now();
    let mut report = LintReport::new();

    if !config.enabled {
        tracing::debug!("Linting disabled");
        return report;
    }

    // Find all .oui files
    let files = find_oui_files(project_path, config);
    report.files_analyzed = files.len();

    tracing::info!("Linting {} OUI files", files.len());

    // Get enabled rules
    let rules = get_enabled_rules(config);

    // Analyze each file
    for file in &files {
        if let Err(e) = analyze_file(file, &rules, config, &mut report) {
            tracing::warn!("Failed to analyze {:?}: {}", file, e);
        }
    }

    // Check max warnings
    if let Some(max) = config.max_warnings {
        if report.by_severity.warnings > max {
            report.passed = false;
        }
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    report
}

/// Find all .oui files in a project
fn find_oui_files(project_path: &Path, config: &LintConfig) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    for pattern in &config.include {
        let glob_pattern = project_path.join(pattern).to_string_lossy().to_string();
        if let Ok(paths) = glob::glob(&glob_pattern) {
            for entry in paths.flatten() {
                // Check exclusions
                let excluded = config.exclude.iter().any(|ex| {
                    let ex_pattern = project_path.join(ex).to_string_lossy().to_string();
                    glob::Pattern::new(&ex_pattern)
                        .map(|p| p.matches_path(&entry))
                        .unwrap_or(false)
                });

                if !excluded {
                    files.push(entry);
                }
            }
        }
    }

    // Also search with walkdir for nested structures
    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map(|e| e == "oui").unwrap_or(false) {
            if !files.contains(&path.to_path_buf()) {
                files.push(path.to_path_buf());
            }
        }
    }

    files
}

/// Analyze a single .oui file
fn analyze_file(
    file: &Path,
    rules: &[LintRule],
    config: &LintConfig,
    report: &mut LintReport,
) -> Result<(), crate::QualityError> {
    let source = std::fs::read_to_string(file)?;
    let file_str = file.to_string_lossy().to_string();

    // Try to compile the file
    match compile(&source) {
        Ok(ir) => {
            // Run structural analysis
            analyze_component(&ir, &source, &file_str, rules, config, report);
        }
        Err(e) => {
            // Report parse error
            let violation = LintViolation::new(
                ErrorCode::lint(1),
                "parse-error",
                LintSeverity::Error,
                &file_str,
                1,
                1,
                &format!("Failed to parse: {}", e),
            );
            report.add_violation(violation);
        }
    }

    // Run source-level analysis
    analyze_source(&source, &file_str, rules, config, report);

    Ok(())
}

/// Analyze a compiled component IR
fn analyze_component(
    ir: &ComponentIR,
    source: &str,
    file: &str,
    rules: &[LintRule],
    config: &LintConfig,
    report: &mut LintReport,
) {
    let analyzer = ComponentAnalyzer::new(source, file);

    for rule in rules {
        let violations = analyzer.check_rule(&rule.id, ir, config);
        for mut violation in violations {
            // Apply severity override if configured
            if let Some(severity_str) = config.severity.get(&rule.id) {
                violation.severity = match severity_str.as_str() {
                    "error" => LintSeverity::Error,
                    "warning" => LintSeverity::Warning,
                    "info" => LintSeverity::Info,
                    _ => violation.severity,
                };
            }
            report.add_violation(violation);
        }
    }

    // Recursively check children
    for child in &ir.children {
        analyze_component(child, source, file, rules, config, report);
    }
}

/// Analyze source code directly (before compilation)
fn analyze_source(
    source: &str,
    file: &str,
    rules: &[LintRule],
    config: &LintConfig,
    report: &mut LintReport,
) {
    let analyzer = SourceAnalyzer::new(source, file);

    for rule in rules {
        let violations = analyzer.check_rule(&rule.id, config);
        for mut violation in violations {
            if let Some(severity_str) = config.severity.get(&rule.id) {
                violation.severity = match severity_str.as_str() {
                    "error" => LintSeverity::Error,
                    "warning" => LintSeverity::Warning,
                    "info" => LintSeverity::Info,
                    _ => violation.severity,
                };
            }
            report.add_violation(violation);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lint_report() {
        let mut report = LintReport::new();
        assert!(report.passed);

        report.add_violation(LintViolation::new(
            ErrorCode::lint(1),
            "test-rule",
            LintSeverity::Warning,
            "test.oui",
            1,
            1,
            "Test warning",
        ));

        assert!(report.passed);
        assert_eq!(report.by_severity.warnings, 1);

        report.add_violation(LintViolation::new(
            ErrorCode::lint(2),
            "test-rule",
            LintSeverity::Error,
            "test.oui",
            2,
            1,
            "Test error",
        ));

        assert!(!report.passed);
        assert_eq!(report.by_severity.errors, 1);
    }
}
