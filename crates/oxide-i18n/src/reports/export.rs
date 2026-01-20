//! Report export functionality
//!
//! Export reports in various formats for CI/CD integration.

use super::coverage::CoverageReport;
use super::quality::QualityReport;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// Report output format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// JSON format
    Json,
    /// Markdown format
    Markdown,
    /// HTML format
    Html,
    /// Plain text
    Text,
    /// JUnit XML (for CI)
    JUnit,
    /// GitHub Actions annotations
    GithubActions,
}

impl ReportFormat {
    /// Get file extension for format
    pub fn extension(&self) -> &'static str {
        match self {
            ReportFormat::Json => "json",
            ReportFormat::Markdown => "md",
            ReportFormat::Html => "html",
            ReportFormat::Text => "txt",
            ReportFormat::JUnit => "xml",
            ReportFormat::GithubActions => "txt",
        }
    }
}

/// Report exporter utility
pub struct ReportExporter;

impl ReportExporter {
    /// Export coverage report
    pub fn export_coverage(
        report: &CoverageReport,
        path: &Path,
        format: ReportFormat,
    ) -> std::io::Result<()> {
        let content = match format {
            ReportFormat::Json => report.to_json(),
            ReportFormat::Markdown => report.to_markdown(),
            ReportFormat::Html => Self::coverage_to_html(report),
            ReportFormat::Text => Self::coverage_to_text(report),
            ReportFormat::JUnit => Self::coverage_to_junit(report),
            ReportFormat::GithubActions => Self::coverage_to_github_actions(report),
        };

        fs::write(path, content)
    }

    /// Export quality report
    pub fn export_quality(
        report: &QualityReport,
        path: &Path,
        format: ReportFormat,
    ) -> std::io::Result<()> {
        let content = match format {
            ReportFormat::Json => serde_json::to_string_pretty(report).unwrap_or_default(),
            ReportFormat::Markdown => report.to_markdown(),
            ReportFormat::Html => Self::quality_to_html(report),
            ReportFormat::Text => Self::quality_to_text(report),
            ReportFormat::JUnit => Self::quality_to_junit(report),
            ReportFormat::GithubActions => Self::quality_to_github_actions(report),
        };

        fs::write(path, content)
    }

    /// Convert coverage report to HTML
    fn coverage_to_html(report: &CoverageReport) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Translation Coverage Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("th { background-color: #f5f5f5; }\n");
        html.push_str(".complete { color: #28a745; }\n");
        html.push_str(".good { color: #17a2b8; }\n");
        html.push_str(".warning { color: #ffc107; }\n");
        html.push_str(".critical { color: #dc3545; }\n");
        html.push_str(".progress { background: #e9ecef; border-radius: 4px; height: 20px; overflow: hidden; }\n");
        html.push_str(".progress-bar { background: #28a745; height: 100%; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str("<h1>Translation Coverage Report</h1>\n");
        html.push_str(&format!("<p>Generated: {}</p>\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary
        html.push_str("<h2>Summary</h2>\n");
        html.push_str("<table>\n");
        html.push_str(&format!("<tr><td>Total Locales</td><td>{}</td></tr>\n", report.overall.total_locales));
        html.push_str(&format!("<tr><td>Complete Locales</td><td>{}</td></tr>\n", report.overall.complete_locales));
        html.push_str(&format!("<tr><td>Release Ready</td><td>{}</td></tr>\n", report.overall.release_ready_locales));
        html.push_str(&format!("<tr><td>Total Keys</td><td>{}</td></tr>\n", report.overall.total_keys));
        html.push_str(&format!("<tr><td>Avg Translation</td><td>{:.1}%</td></tr>\n", report.overall.average_translation_percentage));
        html.push_str("</table>\n");

        // Locale table
        html.push_str("<h2>Coverage by Locale</h2>\n");
        html.push_str("<table>\n");
        html.push_str("<tr><th>Locale</th><th>Progress</th><th>Translated</th><th>Missing</th><th>Status</th></tr>\n");

        for locale in &report.by_locale {
            let status_class = locale.status_emoji();
            html.push_str("<tr>\n");
            html.push_str(&format!("<td>{}</td>\n", locale.locale));
            html.push_str(&format!(
                "<td><div class=\"progress\"><div class=\"progress-bar\" style=\"width: {:.1}%\"></div></div></td>\n",
                locale.translation_percentage
            ));
            html.push_str(&format!("<td>{:.1}%</td>\n", locale.translation_percentage));
            html.push_str(&format!("<td>{}</td>\n", locale.missing_keys.len()));
            html.push_str(&format!("<td class=\"{}\">{}</td>\n", status_class, status_class.to_uppercase()));
            html.push_str("</tr>\n");
        }

        html.push_str("</table>\n");
        html.push_str("</body>\n</html>");

        html
    }

    /// Convert coverage report to plain text
    fn coverage_to_text(report: &CoverageReport) -> String {
        let mut text = String::new();

        text.push_str("TRANSLATION COVERAGE REPORT\n");
        text.push_str(&"=".repeat(60));
        text.push('\n');
        text.push_str(&format!("Generated: {}\n\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        text.push_str("SUMMARY\n");
        text.push_str(&"-".repeat(40));
        text.push('\n');
        text.push_str(&format!("Total Locales:     {}\n", report.overall.total_locales));
        text.push_str(&format!("Complete Locales:  {}\n", report.overall.complete_locales));
        text.push_str(&format!("Total Keys:        {}\n", report.overall.total_keys));
        text.push_str(&format!("Avg Translation:   {:.1}%\n", report.overall.average_translation_percentage));
        text.push_str(&format!("Total Missing:     {}\n\n", report.overall.total_missing));

        text.push_str("BY LOCALE\n");
        text.push_str(&"-".repeat(40));
        text.push('\n');

        for locale in &report.by_locale {
            let bar_len = (locale.translation_percentage / 5.0) as usize;
            let bar = format!("[{}{}]", "#".repeat(bar_len), " ".repeat(20 - bar_len));
            text.push_str(&format!(
                "{:8} {} {:5.1}% ({} missing)\n",
                locale.locale,
                bar,
                locale.translation_percentage,
                locale.missing_keys.len()
            ));
        }

        text
    }

    /// Convert coverage report to JUnit XML
    fn coverage_to_junit(report: &CoverageReport) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let total_tests = report.by_locale.len();
        let failures = report.by_locale.iter().filter(|l| !l.is_complete()).count();

        xml.push_str(&format!(
            "<testsuite name=\"i18n-coverage\" tests=\"{}\" failures=\"{}\">\n",
            total_tests, failures
        ));

        for locale in &report.by_locale {
            xml.push_str(&format!("  <testcase name=\"{}\"", locale.locale));
            if locale.is_complete() {
                xml.push_str("/>\n");
            } else {
                xml.push_str(">\n");
                xml.push_str(&format!(
                    "    <failure message=\"Missing {} translations ({:.1}% complete)\">\n",
                    locale.missing_keys.len(),
                    locale.translation_percentage
                ));
                for key in locale.missing_keys.iter().take(10) {
                    xml.push_str(&format!("      - {}\n", key));
                }
                if locale.missing_keys.len() > 10 {
                    xml.push_str(&format!("      ... and {} more\n", locale.missing_keys.len() - 10));
                }
                xml.push_str("    </failure>\n");
                xml.push_str("  </testcase>\n");
            }
        }

        xml.push_str("</testsuite>\n");
        xml
    }

    /// Convert coverage report to GitHub Actions annotations
    fn coverage_to_github_actions(report: &CoverageReport) -> String {
        let mut output = String::new();

        for locale in &report.by_locale {
            if !locale.is_complete() {
                output.push_str(&format!(
                    "::warning::Locale '{}' is {:.1}% translated ({} keys missing)\n",
                    locale.locale,
                    locale.translation_percentage,
                    locale.missing_keys.len()
                ));
            }
        }

        if report.is_release_ready() {
            output.push_str("::notice::All locales are release ready\n");
        } else {
            output.push_str(&format!(
                "::error::Only {}/{} locales are release ready\n",
                report.overall.release_ready_locales,
                report.overall.total_locales
            ));
        }

        output
    }

    /// Convert quality report to HTML
    fn quality_to_html(report: &QualityReport) -> String {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("<title>Translation Quality Report</title>\n");
        html.push_str("<style>\n");
        html.push_str("body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; }\n");
        html.push_str("table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("th { background-color: #f5f5f5; }\n");
        html.push_str(".error { color: #dc3545; background: #f8d7da; }\n");
        html.push_str(".warning { color: #856404; background: #fff3cd; }\n");
        html.push_str(".info { color: #004085; background: #cce5ff; }\n");
        html.push_str(".score { font-size: 48px; font-weight: bold; text-align: center; }\n");
        html.push_str(".passing { color: #28a745; }\n");
        html.push_str(".failing { color: #dc3545; }\n");
        html.push_str("</style>\n</head>\n<body>\n");

        html.push_str("<h1>Translation Quality Report</h1>\n");
        html.push_str(&format!("<p>Locale: {} | Generated: {}</p>\n",
            report.locale,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));

        // Score
        let score_class = if report.is_passing() { "passing" } else { "failing" };
        html.push_str(&format!(
            "<div class=\"score {}\">{:.0}%</div>\n",
            score_class,
            report.summary.quality_score
        ));

        // Summary
        html.push_str("<h2>Summary</h2>\n");
        html.push_str("<table>\n");
        html.push_str(&format!("<tr><td>Entries Checked</td><td>{}</td></tr>\n", report.summary.total_checked));
        html.push_str(&format!("<tr><td>Errors</td><td class=\"error\">{}</td></tr>\n", report.summary.error_count));
        html.push_str(&format!("<tr><td>Warnings</td><td class=\"warning\">{}</td></tr>\n", report.summary.warning_count));
        html.push_str(&format!("<tr><td>Info</td><td class=\"info\">{}</td></tr>\n", report.summary.info_count));
        html.push_str("</table>\n");

        // Issues
        if !report.issues.is_empty() {
            html.push_str("<h2>Issues</h2>\n");
            html.push_str("<table>\n");
            html.push_str("<tr><th>Level</th><th>Key</th><th>Type</th><th>Message</th></tr>\n");

            for issue in &report.issues {
                let level_class = match issue.level {
                    super::quality::QualityLevel::Error => "error",
                    super::quality::QualityLevel::Warning => "warning",
                    super::quality::QualityLevel::Info => "info",
                };
                html.push_str(&format!(
                    "<tr class=\"{}\"><td>{:?}</td><td>{}</td><td>{:?}</td><td>{}</td></tr>\n",
                    level_class, issue.level, issue.key, issue.issue_type, issue.message
                ));
            }

            html.push_str("</table>\n");
        }

        html.push_str("</body>\n</html>");
        html
    }

    /// Convert quality report to plain text
    fn quality_to_text(report: &QualityReport) -> String {
        let mut text = String::new();

        text.push_str("TRANSLATION QUALITY REPORT\n");
        text.push_str(&"=".repeat(60));
        text.push('\n');
        text.push_str(&format!("Locale: {}\n", report.locale));
        text.push_str(&format!("Generated: {}\n\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        text.push_str(&format!("Quality Score: {:.0}%\n", report.summary.quality_score));
        text.push_str(&format!("Status: {}\n\n", if report.is_passing() { "PASSING" } else { "FAILING" }));

        text.push_str("SUMMARY\n");
        text.push_str(&"-".repeat(40));
        text.push('\n');
        text.push_str(&format!("Entries Checked: {}\n", report.summary.total_checked));
        text.push_str(&format!("Errors:          {}\n", report.summary.error_count));
        text.push_str(&format!("Warnings:        {}\n", report.summary.warning_count));
        text.push_str(&format!("Info:            {}\n\n", report.summary.info_count));

        if !report.issues.is_empty() {
            text.push_str("ISSUES\n");
            text.push_str(&"-".repeat(40));
            text.push('\n');

            for issue in &report.issues {
                text.push_str(&format!(
                    "[{:?}] {} - {:?}: {}\n",
                    issue.level, issue.key, issue.issue_type, issue.message
                ));
            }
        }

        text
    }

    /// Convert quality report to JUnit XML
    fn quality_to_junit(report: &QualityReport) -> String {
        let mut xml = String::new();
        xml.push_str("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n");

        let failures = report.summary.error_count;

        xml.push_str(&format!(
            "<testsuite name=\"i18n-quality-{}\" tests=\"{}\" failures=\"{}\">\n",
            report.locale,
            report.summary.total_checked,
            failures
        ));

        // Group issues by key
        let mut issues_by_key: std::collections::HashMap<&str, Vec<_>> = std::collections::HashMap::new();
        for issue in &report.issues {
            issues_by_key.entry(&issue.key).or_default().push(issue);
        }

        for (key, issues) in issues_by_key {
            let has_errors = issues.iter().any(|i| matches!(i.level, super::quality::QualityLevel::Error));

            xml.push_str(&format!("  <testcase name=\"{}\"", key));
            if has_errors {
                xml.push_str(">\n");
                for issue in issues {
                    if matches!(issue.level, super::quality::QualityLevel::Error) {
                        xml.push_str(&format!(
                            "    <failure type=\"{:?}\" message=\"{}\"/>\n",
                            issue.issue_type, issue.message
                        ));
                    }
                }
                xml.push_str("  </testcase>\n");
            } else {
                xml.push_str("/>\n");
            }
        }

        xml.push_str("</testsuite>\n");
        xml
    }

    /// Convert quality report to GitHub Actions annotations
    fn quality_to_github_actions(report: &QualityReport) -> String {
        let mut output = String::new();

        for issue in &report.issues {
            let level = match issue.level {
                super::quality::QualityLevel::Error => "error",
                super::quality::QualityLevel::Warning => "warning",
                super::quality::QualityLevel::Info => "notice",
            };

            output.push_str(&format!(
                "::{}::{}[{}]: {}\n",
                level, issue.key, issue.issue_type.description(), issue.message
            ));
        }

        if report.is_passing() {
            output.push_str(&format!(
                "::notice::Quality check passed with score {:.0}%\n",
                report.summary.quality_score
            ));
        } else {
            output.push_str(&format!(
                "::error::Quality check failed with {} errors (score: {:.0}%)\n",
                report.summary.error_count,
                report.summary.quality_score
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_extensions() {
        assert_eq!(ReportFormat::Json.extension(), "json");
        assert_eq!(ReportFormat::Markdown.extension(), "md");
        assert_eq!(ReportFormat::Html.extension(), "html");
        assert_eq!(ReportFormat::JUnit.extension(), "xml");
    }
}
