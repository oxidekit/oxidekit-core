//! Quality report generation
//!
//! Structured reporting for quality gate results.

use crate::lint::LintReport;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Overall quality report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Project path
    pub project: PathBuf,
    /// Overall status
    pub status: QualityStatus,
    /// Report sections
    pub sections: Vec<QualitySection>,
    /// Summary statistics
    pub summary: ReportSummary,
    /// Timestamp
    pub timestamp: String,
    /// Total execution time in milliseconds
    pub total_duration_ms: u64,
}

impl QualityReport {
    pub fn new(project_path: &Path) -> Self {
        Self {
            project: project_path.to_path_buf(),
            status: QualityStatus::Pending,
            sections: Vec::new(),
            summary: ReportSummary::default(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            total_duration_ms: 0,
        }
    }

    /// Add a section to the report
    pub fn add_section(&mut self, section: QualitySection) {
        self.sections.push(section);
    }

    /// Finalize the report and compute status
    pub fn finalize(&mut self) {
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut any_failed = false;

        for section in &self.sections {
            match section {
                QualitySection::Lint(report) => {
                    total_errors += report.by_severity.errors;
                    total_warnings += report.by_severity.warnings;
                    if !report.passed {
                        any_failed = true;
                    }
                    self.total_duration_ms += report.duration_ms;
                }
                #[cfg(feature = "a11y")]
                QualitySection::Accessibility(report) => {
                    total_errors += report.errors;
                    total_warnings += report.warnings;
                    if !report.passed {
                        any_failed = true;
                    }
                    self.total_duration_ms += report.duration_ms;
                }
                #[cfg(feature = "perf")]
                QualitySection::Performance(report) => {
                    total_errors += report.violations.iter().filter(|v| v.is_error).count();
                    total_warnings += report.violations.iter().filter(|v| !v.is_error).count();
                    if !report.passed {
                        any_failed = true;
                    }
                    self.total_duration_ms += report.duration_ms;
                }
                #[cfg(feature = "security")]
                QualitySection::Security(report) => {
                    total_errors += report.critical + report.high;
                    total_warnings += report.medium + report.low;
                    if !report.passed {
                        any_failed = true;
                    }
                    self.total_duration_ms += report.duration_ms;
                }
                #[cfg(feature = "bundle")]
                QualitySection::Bundle(report) => {
                    if report.exceeded_size {
                        total_errors += 1;
                    }
                    total_warnings += report.warnings.len();
                    if !report.passed {
                        any_failed = true;
                    }
                    self.total_duration_ms += report.duration_ms;
                }
            }
        }

        self.summary = ReportSummary {
            total_errors,
            total_warnings,
            sections_passed: self.sections.iter().filter(|s| s.passed()).count(),
            sections_failed: self.sections.iter().filter(|s| !s.passed()).count(),
        };

        self.status = if any_failed {
            QualityStatus::Failed
        } else if total_warnings > 0 {
            QualityStatus::PassedWithWarnings
        } else {
            QualityStatus::Passed
        };
    }

    /// Check if all quality gates passed
    pub fn passed(&self) -> bool {
        matches!(self.status, QualityStatus::Passed | QualityStatus::PassedWithWarnings)
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Export to Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str("# Quality Report\n\n");
        md.push_str(&format!("**Project:** `{}`\n", self.project.display()));
        md.push_str(&format!("**Status:** {}\n", self.status.emoji()));
        md.push_str(&format!("**Generated:** {}\n\n", self.timestamp));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- **Errors:** {}\n", self.summary.total_errors));
        md.push_str(&format!("- **Warnings:** {}\n", self.summary.total_warnings));
        md.push_str(&format!("- **Duration:** {}ms\n\n", self.total_duration_ms));

        // Sections
        for section in &self.sections {
            md.push_str(&section.to_markdown());
            md.push('\n');
        }

        md
    }

    /// Export to plain text
    pub fn to_text(&self) -> String {
        let mut text = String::new();

        text.push_str(&format!("Quality Report for {}\n", self.project.display()));
        text.push_str(&"=".repeat(60));
        text.push('\n');
        text.push_str(&format!("Status: {:?}\n", self.status));
        text.push_str(&format!("Errors: {}, Warnings: {}\n", self.summary.total_errors, self.summary.total_warnings));
        text.push_str(&format!("Duration: {}ms\n\n", self.total_duration_ms));

        for section in &self.sections {
            text.push_str(&section.to_text());
            text.push('\n');
        }

        text
    }
}

/// Quality check status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QualityStatus {
    /// Not yet run
    Pending,
    /// All checks passed
    Passed,
    /// Passed but with warnings
    PassedWithWarnings,
    /// One or more checks failed
    Failed,
}

impl QualityStatus {
    /// Get emoji representation
    pub fn emoji(&self) -> &'static str {
        match self {
            QualityStatus::Pending => "Pending",
            QualityStatus::Passed => "Passed",
            QualityStatus::PassedWithWarnings => "Passed (with warnings)",
            QualityStatus::Failed => "Failed",
        }
    }
}

/// Report summary statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReportSummary {
    pub total_errors: usize,
    pub total_warnings: usize,
    pub sections_passed: usize,
    pub sections_failed: usize,
}

/// Quality report sections
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum QualitySection {
    Lint(LintReport),
    #[cfg(feature = "a11y")]
    Accessibility(crate::A11yReport),
    #[cfg(feature = "perf")]
    Performance(crate::PerfReport),
    #[cfg(feature = "security")]
    Security(crate::SecurityReport),
    #[cfg(feature = "bundle")]
    Bundle(crate::BundleReport),
}

impl QualitySection {
    /// Check if this section passed
    pub fn passed(&self) -> bool {
        match self {
            QualitySection::Lint(r) => r.passed,
            #[cfg(feature = "a11y")]
            QualitySection::Accessibility(r) => r.passed,
            #[cfg(feature = "perf")]
            QualitySection::Performance(r) => r.passed,
            #[cfg(feature = "security")]
            QualitySection::Security(r) => r.passed,
            #[cfg(feature = "bundle")]
            QualitySection::Bundle(r) => r.passed,
        }
    }

    /// Get section name
    pub fn name(&self) -> &'static str {
        match self {
            QualitySection::Lint(_) => "Lint",
            #[cfg(feature = "a11y")]
            QualitySection::Accessibility(_) => "Accessibility",
            #[cfg(feature = "perf")]
            QualitySection::Performance(_) => "Performance",
            #[cfg(feature = "security")]
            QualitySection::Security(_) => "Security",
            #[cfg(feature = "bundle")]
            QualitySection::Bundle(_) => "Bundle",
        }
    }

    /// Convert to Markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();
        let status = if self.passed() { "Passed" } else { "Failed" };

        md.push_str(&format!("### {} {}\n\n", self.name(), status));

        match self {
            QualitySection::Lint(report) => {
                md.push_str(&format!("Files analyzed: {}\n", report.files_analyzed));
                md.push_str(&format!("Violations: {} errors, {} warnings\n\n", report.by_severity.errors, report.by_severity.warnings));

                if !report.violations.is_empty() {
                    md.push_str("| File | Line | Rule | Message |\n");
                    md.push_str("|------|------|------|--------|\n");

                    for v in report.violations.iter().take(20) {
                        md.push_str(&format!("| {} | {} | {} | {} |\n", v.file, v.line, v.rule, v.message));
                    }

                    if report.violations.len() > 20 {
                        md.push_str(&format!("\n*...and {} more violations*\n", report.violations.len() - 20));
                    }
                }
            }
            #[cfg(feature = "a11y")]
            QualitySection::Accessibility(report) => {
                md.push_str(&format!("WCAG Level: {}\n", report.wcag_level));
                md.push_str(&format!("Violations: {} errors, {} warnings\n\n", report.errors, report.warnings));

                if !report.violations.is_empty() {
                    for v in report.violations.iter().take(10) {
                        md.push_str(&format!("- **{}** ({}): {}\n", v.code, v.rule, v.message));
                    }
                }
            }
            #[cfg(feature = "perf")]
            QualitySection::Performance(report) => {
                md.push_str(&format!("Frame budget: {}ms\n", report.frame_budget_ms));
                md.push_str(&format!("Violations: {}\n\n", report.violations.len()));

                for v in &report.violations {
                    md.push_str(&format!("- {}: {} (budget: {}, actual: {})\n", v.category, v.message, v.budget, v.actual));
                }
            }
            #[cfg(feature = "security")]
            QualitySection::Security(report) => {
                md.push_str(&format!("Critical: {}, High: {}, Medium: {}, Low: {}\n\n", report.critical, report.high, report.medium, report.low));

                for v in report.vulnerabilities.iter().take(10) {
                    md.push_str(&format!("- **{}**: {} ({})\n", v.severity, v.title, v.package));
                }
            }
            #[cfg(feature = "bundle")]
            QualitySection::Bundle(report) => {
                md.push_str(&format!("Total size: {}\n", report.total_size_formatted));
                md.push_str(&format!("Size limit: {}\n", report.size_limit_formatted));

                if report.exceeded_size {
                    md.push_str("\n**Bundle size exceeded!**\n");
                }
            }
        }

        md
    }

    /// Convert to plain text
    pub fn to_text(&self) -> String {
        let mut text = String::new();
        let status = if self.passed() { "PASSED" } else { "FAILED" };

        text.push_str(&format!("{} - {}\n", self.name(), status));
        text.push_str(&"-".repeat(40));
        text.push('\n');

        match self {
            QualitySection::Lint(report) => {
                text.push_str(&format!("Files: {}, Errors: {}, Warnings: {}\n", report.files_analyzed, report.by_severity.errors, report.by_severity.warnings));

                for v in report.violations.iter().take(10) {
                    text.push_str(&format!("  {}:{}: [{}] {}\n", v.file, v.line, v.rule, v.message));
                }
            }
            #[cfg(feature = "a11y")]
            QualitySection::Accessibility(report) => {
                text.push_str(&format!("WCAG {}: {} errors, {} warnings\n", report.wcag_level, report.errors, report.warnings));
            }
            #[cfg(feature = "perf")]
            QualitySection::Performance(report) => {
                text.push_str(&format!("Frame budget: {}ms, Violations: {}\n", report.frame_budget_ms, report.violations.len()));
            }
            #[cfg(feature = "security")]
            QualitySection::Security(report) => {
                text.push_str(&format!("Critical: {}, High: {}, Medium: {}, Low: {}\n", report.critical, report.high, report.medium, report.low));
            }
            #[cfg(feature = "bundle")]
            QualitySection::Bundle(report) => {
                text.push_str(&format!("Size: {} / {}\n", report.total_size_formatted, report.size_limit_formatted));
            }
        }

        text
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_report() {
        let mut report = QualityReport::new(Path::new("/test/project"));
        assert_eq!(report.status, QualityStatus::Pending);

        let lint_report = LintReport::new();
        report.add_section(QualitySection::Lint(lint_report));
        report.finalize();

        assert_eq!(report.status, QualityStatus::Passed);
    }

    #[test]
    fn test_report_markdown() {
        let mut report = QualityReport::new(Path::new("/test/project"));
        report.add_section(QualitySection::Lint(LintReport::new()));
        report.finalize();

        let md = report.to_markdown();
        assert!(md.contains("# Quality Report"));
        assert!(md.contains("Lint"));
    }
}
