//! Compliance report generation
//!
//! Generate comprehensive license compliance reports in various formats.

use std::path::Path;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use crate::error::LegalResult;
use crate::scanner::{ScanResult, DependencyLicense};
use crate::policy::{LicensePolicy, PolicyValidationResult};

/// Compliance report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Plain text
    Text,
    /// Markdown
    Markdown,
    /// JSON
    Json,
    /// HTML
    Html,
    /// CSV (for spreadsheet import)
    Csv,
}

/// Compliance status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    /// Fully compliant
    Compliant,
    /// Compliant with warnings
    CompliantWithWarnings,
    /// Non-compliant
    NonCompliant,
    /// Requires manual review
    RequiresReview,
}

/// Comprehensive compliance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Report title
    pub title: String,
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Overall compliance status
    pub status: ComplianceStatus,
    /// Policy used
    pub policy_name: String,
    /// Scan results
    pub scan: ScanResult,
    /// Policy validation results
    pub validation: PolicyValidationResult,
    /// Attribution section (for NOTICE files)
    pub attributions: Vec<Attribution>,
    /// Recommendations
    pub recommendations: Vec<String>,
}

impl ComplianceReport {
    /// Generate a compliance report from scan results
    pub fn generate(scan: ScanResult, policy: &LicensePolicy) -> Self {
        let validation = policy.validate(&scan);

        let status = if !validation.passed {
            ComplianceStatus::NonCompliant
        } else if !validation.warnings.is_empty() {
            ComplianceStatus::CompliantWithWarnings
        } else if scan.summary.unknown_count > 0 {
            ComplianceStatus::RequiresReview
        } else {
            ComplianceStatus::Compliant
        };

        let attributions = scan
            .dependencies
            .iter()
            .filter(|d| d.license.requires_attribution())
            .map(Attribution::from_dependency)
            .collect();

        let mut recommendations = Vec::new();

        // Add recommendations based on scan results
        if scan.summary.unknown_count > 0 {
            recommendations.push(format!(
                "Review {} dependencies with unknown licenses",
                scan.summary.unknown_count
            ));
        }

        if scan.summary.copyleft_count > 0 {
            recommendations.push(format!(
                "Verify compliance with {} copyleft dependencies",
                scan.summary.copyleft_count
            ));
        }

        if !validation.warnings.is_empty() {
            recommendations.push("Address policy warnings before release".to_string());
        }

        Self {
            title: format!("{} License Compliance Report", scan.project_name),
            project_name: scan.project_name.clone(),
            project_version: scan.project_version.clone(),
            generated_at: Utc::now(),
            status,
            policy_name: policy.name.clone(),
            scan,
            validation,
            attributions,
            recommendations,
        }
    }

    /// Export report to specified format
    pub fn export(&self, format: ReportFormat) -> LegalResult<String> {
        match format {
            ReportFormat::Text => self.to_text(),
            ReportFormat::Markdown => self.to_markdown(),
            ReportFormat::Json => self.to_json(),
            ReportFormat::Html => self.to_html(),
            ReportFormat::Csv => self.to_csv(),
        }
    }

    /// Export report to a file
    pub fn export_to_file(&self, path: impl AsRef<Path>, format: ReportFormat) -> LegalResult<()> {
        let content = self.export(format)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Export as plain text
    fn to_text(&self) -> LegalResult<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!("{}\n", "=".repeat(60)));
        output.push_str(&format!("{}\n", self.title));
        output.push_str(&format!("{}\n\n", "=".repeat(60)));

        output.push_str(&format!("Project: {} v{}\n", self.project_name, self.project_version));
        output.push_str(&format!("Generated: {}\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("Policy: {}\n", self.policy_name));
        output.push_str(&format!("Status: {:?}\n\n", self.status));

        // Summary
        output.push_str("SUMMARY\n");
        output.push_str(&format!("{}\n", "-".repeat(40)));
        output.push_str(&format!("Total Dependencies: {}\n", self.scan.summary.total_dependencies));
        output.push_str(&format!("  Permissive: {}\n", self.scan.summary.permissive_count));
        output.push_str(&format!("  Weak Copyleft: {}\n", self.scan.summary.weak_copyleft_count));
        output.push_str(&format!("  Strong Copyleft: {}\n", self.scan.summary.copyleft_count));
        output.push_str(&format!("  Proprietary: {}\n", self.scan.summary.proprietary_count));
        output.push_str(&format!("  Unknown: {}\n\n", self.scan.summary.unknown_count));

        // Violations
        if !self.validation.violations.is_empty() {
            output.push_str("VIOLATIONS\n");
            output.push_str(&format!("{}\n", "-".repeat(40)));
            for v in &self.validation.violations {
                output.push_str(&format!("  [!] {} v{}: {} ({})\n", v.package, v.version, v.license, v.message));
            }
            output.push('\n');
        }

        // Warnings
        if !self.validation.warnings.is_empty() {
            output.push_str("WARNINGS\n");
            output.push_str(&format!("{}\n", "-".repeat(40)));
            for w in &self.validation.warnings {
                output.push_str(&format!("  [~] {} v{}: {} ({})\n", w.package, w.version, w.license, w.message));
            }
            output.push('\n');
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            output.push_str("RECOMMENDATIONS\n");
            output.push_str(&format!("{}\n", "-".repeat(40)));
            for r in &self.recommendations {
                output.push_str(&format!("  - {}\n", r));
            }
            output.push('\n');
        }

        // Dependencies
        output.push_str("DEPENDENCIES\n");
        output.push_str(&format!("{}\n", "-".repeat(40)));
        for dep in &self.scan.dependencies {
            output.push_str(&format!(
                "  {} v{}: {} ({})\n",
                dep.name,
                dep.version,
                dep.license.spdx_id,
                dep.license.category.description()
            ));
        }

        Ok(output)
    }

    /// Export as Markdown
    fn to_markdown(&self) -> LegalResult<String> {
        let mut output = String::new();

        // Header
        output.push_str(&format!("# {}\n\n", self.title));
        output.push_str(&format!("**Project:** {} v{}  \n", self.project_name, self.project_version));
        output.push_str(&format!("**Generated:** {}  \n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        output.push_str(&format!("**Policy:** {}  \n", self.policy_name));
        output.push_str(&format!("**Status:** {:?}\n\n", self.status));

        // Summary
        output.push_str("## Summary\n\n");
        output.push_str("| Category | Count |\n");
        output.push_str("|----------|-------|\n");
        output.push_str(&format!("| Total Dependencies | {} |\n", self.scan.summary.total_dependencies));
        output.push_str(&format!("| Permissive | {} |\n", self.scan.summary.permissive_count));
        output.push_str(&format!("| Weak Copyleft | {} |\n", self.scan.summary.weak_copyleft_count));
        output.push_str(&format!("| Strong Copyleft | {} |\n", self.scan.summary.copyleft_count));
        output.push_str(&format!("| Proprietary | {} |\n", self.scan.summary.proprietary_count));
        output.push_str(&format!("| Unknown | {} |\n\n", self.scan.summary.unknown_count));

        // Violations
        if !self.validation.violations.is_empty() {
            output.push_str("## Violations\n\n");
            output.push_str("| Package | Version | License | Issue |\n");
            output.push_str("|---------|---------|---------|-------|\n");
            for v in &self.validation.violations {
                output.push_str(&format!("| {} | {} | {} | {} |\n", v.package, v.version, v.license, v.message));
            }
            output.push('\n');
        }

        // Warnings
        if !self.validation.warnings.is_empty() {
            output.push_str("## Warnings\n\n");
            output.push_str("| Package | Version | License | Issue |\n");
            output.push_str("|---------|---------|---------|-------|\n");
            for w in &self.validation.warnings {
                output.push_str(&format!("| {} | {} | {} | {} |\n", w.package, w.version, w.license, w.message));
            }
            output.push('\n');
        }

        // Recommendations
        if !self.recommendations.is_empty() {
            output.push_str("## Recommendations\n\n");
            for r in &self.recommendations {
                output.push_str(&format!("- {}\n", r));
            }
            output.push('\n');
        }

        // Dependencies table
        output.push_str("## Dependencies\n\n");
        output.push_str("| Package | Version | License | Category |\n");
        output.push_str("|---------|---------|---------|----------|\n");
        for dep in &self.scan.dependencies {
            output.push_str(&format!(
                "| {} | {} | {} | {:?} |\n",
                dep.name,
                dep.version,
                dep.license.spdx_id,
                dep.license.category
            ));
        }

        Ok(output)
    }

    /// Export as JSON
    fn to_json(&self) -> LegalResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Export as HTML
    fn to_html(&self) -> LegalResult<String> {
        let mut output = String::new();

        output.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        output.push_str(&format!("<title>{}</title>\n", self.title));
        output.push_str(r#"<style>
body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 40px; }
h1 { color: #333; }
table { border-collapse: collapse; width: 100%; margin: 20px 0; }
th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }
th { background-color: #4CAF50; color: white; }
tr:nth-child(even) { background-color: #f2f2f2; }
.violation { color: red; }
.warning { color: orange; }
.compliant { color: green; }
.status-badge { padding: 5px 10px; border-radius: 4px; font-weight: bold; }
.status-compliant { background-color: #4CAF50; color: white; }
.status-warning { background-color: #ff9800; color: white; }
.status-noncompliant { background-color: #f44336; color: white; }
</style>
"#);
        output.push_str("</head>\n<body>\n");

        // Header
        output.push_str(&format!("<h1>{}</h1>\n", self.title));

        let status_class = match self.status {
            ComplianceStatus::Compliant => "status-compliant",
            ComplianceStatus::CompliantWithWarnings => "status-warning",
            ComplianceStatus::NonCompliant => "status-noncompliant",
            ComplianceStatus::RequiresReview => "status-warning",
        };
        output.push_str(&format!(
            "<p><strong>Project:</strong> {} v{}</p>\n",
            self.project_name, self.project_version
        ));
        output.push_str(&format!(
            "<p><strong>Generated:</strong> {}</p>\n",
            self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        output.push_str(&format!(
            "<p><strong>Status:</strong> <span class=\"status-badge {}\">{:?}</span></p>\n",
            status_class, self.status
        ));

        // Summary
        output.push_str("<h2>Summary</h2>\n<table>\n");
        output.push_str("<tr><th>Category</th><th>Count</th></tr>\n");
        output.push_str(&format!("<tr><td>Total Dependencies</td><td>{}</td></tr>\n", self.scan.summary.total_dependencies));
        output.push_str(&format!("<tr><td>Permissive</td><td>{}</td></tr>\n", self.scan.summary.permissive_count));
        output.push_str(&format!("<tr><td>Weak Copyleft</td><td>{}</td></tr>\n", self.scan.summary.weak_copyleft_count));
        output.push_str(&format!("<tr><td>Strong Copyleft</td><td>{}</td></tr>\n", self.scan.summary.copyleft_count));
        output.push_str(&format!("<tr><td>Unknown</td><td>{}</td></tr>\n", self.scan.summary.unknown_count));
        output.push_str("</table>\n");

        // Violations
        if !self.validation.violations.is_empty() {
            output.push_str("<h2 class=\"violation\">Violations</h2>\n<table>\n");
            output.push_str("<tr><th>Package</th><th>Version</th><th>License</th><th>Issue</th></tr>\n");
            for v in &self.validation.violations {
                output.push_str(&format!(
                    "<tr class=\"violation\"><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>\n",
                    v.package, v.version, v.license, v.message
                ));
            }
            output.push_str("</table>\n");
        }

        // Dependencies
        output.push_str("<h2>Dependencies</h2>\n<table>\n");
        output.push_str("<tr><th>Package</th><th>Version</th><th>License</th><th>Category</th></tr>\n");
        for dep in &self.scan.dependencies {
            output.push_str(&format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{:?}</td></tr>\n",
                dep.name, dep.version, dep.license.spdx_id, dep.license.category
            ));
        }
        output.push_str("</table>\n");

        output.push_str("</body>\n</html>");
        Ok(output)
    }

    /// Export as CSV
    fn to_csv(&self) -> LegalResult<String> {
        let mut output = String::new();

        output.push_str("Package,Version,License,Category,OSI Approved,Requires Attribution\n");
        for dep in &self.scan.dependencies {
            output.push_str(&format!(
                "{},{},{},{:?},{},{}\n",
                dep.name,
                dep.version,
                dep.license.spdx_id,
                dep.license.category,
                dep.license.osi_approved,
                dep.license.requires_attribution()
            ));
        }

        Ok(output)
    }

    /// Generate a NOTICE file for attribution
    pub fn generate_notice(&self) -> String {
        let mut notice = String::new();

        notice.push_str(&format!("{} Third-Party Notices\n", self.project_name));
        notice.push_str(&format!("{}\n\n", "=".repeat(50)));

        notice.push_str("This project includes software from the following third parties:\n\n");

        for attr in &self.attributions {
            notice.push_str(&format!("{}\n", "-".repeat(40)));
            notice.push_str(&format!("{} v{}\n", attr.package, attr.version));
            notice.push_str(&format!("License: {}\n", attr.license));

            if !attr.authors.is_empty() {
                notice.push_str(&format!("Authors: {}\n", attr.authors.join(", ")));
            }

            if let Some(ref repo) = attr.repository {
                notice.push_str(&format!("Repository: {}\n", repo));
            }

            notice.push('\n');
        }

        notice
    }
}

/// Attribution information for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Attribution {
    /// Package name
    pub package: String,
    /// Package version
    pub version: String,
    /// License identifier
    pub license: String,
    /// Authors
    pub authors: Vec<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// License text (if available)
    pub license_text: Option<String>,
}

impl Attribution {
    /// Create attribution from a dependency
    pub fn from_dependency(dep: &DependencyLicense) -> Self {
        Self {
            package: dep.name.clone(),
            version: dep.version.clone(),
            license: dep.license.spdx_id.clone(),
            authors: dep.authors.clone(),
            repository: dep.repository.clone(),
            license_text: None, // Could be populated from license file
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScanSummary;

    #[test]
    fn test_report_format_text() {
        let scan = ScanResult {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            project_license: Some("MIT".to_string()),
            dependencies: vec![],
            summary: ScanSummary::default(),
        };

        let policy = LicensePolicy::permissive();
        let report = ComplianceReport::generate(scan, &policy);

        let text = report.export(ReportFormat::Text).unwrap();
        assert!(text.contains("test-project"));
        assert!(text.contains("License Compliance Report"));
    }
}
