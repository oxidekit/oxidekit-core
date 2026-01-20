//! Translation quality reporting
//!
//! Analyzes translations for quality issues like:
//! - Missing placeholders
//! - Length violations
//! - Consistency issues
//! - Terminology problems

use crate::formats::{TranslationEntry, TranslationFile, TranslationValue};
use crate::memory::extract_placeholders;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Severity level of a quality issue
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QualityLevel {
    /// Informational
    Info,
    /// Warning - should be addressed
    Warning,
    /// Error - must be fixed
    Error,
}

/// Types of quality issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    /// Missing placeholder in translation
    MissingPlaceholder,
    /// Extra placeholder not in source
    ExtraPlaceholder,
    /// Translation too long
    LengthExceeded,
    /// Translation too short (suspicious)
    LengthTooShort,
    /// Same as source (possibly untranslated)
    IdenticalToSource,
    /// Inconsistent capitalization
    InconsistentCapitalization,
    /// Missing punctuation
    MissingPunctuation,
    /// Different punctuation
    DifferentPunctuation,
    /// Leading/trailing whitespace
    WhitespaceIssue,
    /// Terminology inconsistency
    TerminologyIssue,
    /// Empty translation
    EmptyTranslation,
}

impl IssueType {
    /// Get default severity for this issue type
    pub fn default_severity(&self) -> QualityLevel {
        match self {
            IssueType::MissingPlaceholder => QualityLevel::Error,
            IssueType::ExtraPlaceholder => QualityLevel::Error,
            IssueType::LengthExceeded => QualityLevel::Warning,
            IssueType::LengthTooShort => QualityLevel::Info,
            IssueType::IdenticalToSource => QualityLevel::Warning,
            IssueType::InconsistentCapitalization => QualityLevel::Info,
            IssueType::MissingPunctuation => QualityLevel::Info,
            IssueType::DifferentPunctuation => QualityLevel::Info,
            IssueType::WhitespaceIssue => QualityLevel::Warning,
            IssueType::TerminologyIssue => QualityLevel::Warning,
            IssueType::EmptyTranslation => QualityLevel::Error,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            IssueType::MissingPlaceholder => "Translation is missing a placeholder from source",
            IssueType::ExtraPlaceholder => "Translation has placeholder not in source",
            IssueType::LengthExceeded => "Translation exceeds maximum allowed length",
            IssueType::LengthTooShort => "Translation is suspiciously short",
            IssueType::IdenticalToSource => "Translation is identical to source text",
            IssueType::InconsistentCapitalization => "Capitalization differs from source pattern",
            IssueType::MissingPunctuation => "Translation is missing ending punctuation",
            IssueType::DifferentPunctuation => "Translation has different punctuation",
            IssueType::WhitespaceIssue => "Translation has leading/trailing whitespace",
            IssueType::TerminologyIssue => "Translation uses inconsistent terminology",
            IssueType::EmptyTranslation => "Translation is empty",
        }
    }
}

/// A quality issue found in a translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityIssue {
    /// Translation key
    pub key: String,
    /// Locale
    pub locale: String,
    /// Issue type
    pub issue_type: IssueType,
    /// Severity level
    pub level: QualityLevel,
    /// Detailed message
    pub message: String,
    /// Source text
    pub source: String,
    /// Target text (translation)
    pub target: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

impl QualityIssue {
    /// Create a new quality issue
    pub fn new(
        key: impl Into<String>,
        locale: impl Into<String>,
        issue_type: IssueType,
        source: impl Into<String>,
        target: impl Into<String>,
    ) -> Self {
        Self {
            key: key.into(),
            locale: locale.into(),
            issue_type,
            level: issue_type.default_severity(),
            message: issue_type.description().to_string(),
            source: source.into(),
            target: target.into(),
            suggestion: None,
        }
    }

    /// Set a custom message
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = message.into();
        self
    }

    /// Set a suggestion
    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    /// Override severity
    pub fn with_level(mut self, level: QualityLevel) -> Self {
        self.level = level;
        self
    }
}

/// Configuration for quality checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Check for placeholder issues
    pub check_placeholders: bool,
    /// Check for length issues
    pub check_length: bool,
    /// Maximum length multiplier (translation length vs source)
    pub max_length_multiplier: f64,
    /// Minimum length ratio (translation vs source)
    pub min_length_ratio: f64,
    /// Check for identical translations
    pub check_identical: bool,
    /// Check punctuation consistency
    pub check_punctuation: bool,
    /// Check whitespace issues
    pub check_whitespace: bool,
    /// Terminology glossary for consistency checks
    #[serde(default)]
    pub terminology: HashMap<String, String>,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            check_placeholders: true,
            check_length: true,
            max_length_multiplier: 2.0,
            min_length_ratio: 0.3,
            check_identical: true,
            check_punctuation: true,
            check_whitespace: true,
            terminology: HashMap::new(),
        }
    }
}

/// Quality report for translation files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityReport {
    /// Report generation time
    pub generated_at: DateTime<Utc>,
    /// Project name
    pub project: Option<String>,
    /// Locale analyzed
    pub locale: String,
    /// All issues found
    pub issues: Vec<QualityIssue>,
    /// Summary statistics
    pub summary: QualitySummary,
}

/// Summary of quality issues
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QualitySummary {
    /// Total entries checked
    pub total_checked: usize,
    /// Entries with errors
    pub entries_with_errors: usize,
    /// Entries with warnings
    pub entries_with_warnings: usize,
    /// Total errors
    pub error_count: usize,
    /// Total warnings
    pub warning_count: usize,
    /// Total info issues
    pub info_count: usize,
    /// Quality score (0-100)
    pub quality_score: f64,
}

impl QualityReport {
    /// Create a new report
    pub fn new(locale: impl Into<String>) -> Self {
        Self {
            generated_at: Utc::now(),
            project: None,
            locale: locale.into(),
            issues: Vec::new(),
            summary: QualitySummary::default(),
        }
    }

    /// Generate quality report from a translation file
    pub fn from_file(file: &TranslationFile, config: &QualityConfig) -> Self {
        let mut report = QualityReport::new(&file.target_locale);

        for entry in &file.entries {
            let entry_issues = check_entry(entry, &file.target_locale, config);
            report.issues.extend(entry_issues);
        }

        report.calculate_summary(file.entries.len());
        report
    }

    /// Add an issue
    pub fn add_issue(&mut self, issue: QualityIssue) {
        self.issues.push(issue);
    }

    /// Calculate summary statistics
    fn calculate_summary(&mut self, total_checked: usize) {
        let mut entries_with_errors = std::collections::HashSet::new();
        let mut entries_with_warnings = std::collections::HashSet::new();
        let mut error_count = 0;
        let mut warning_count = 0;
        let mut info_count = 0;

        for issue in &self.issues {
            match issue.level {
                QualityLevel::Error => {
                    error_count += 1;
                    entries_with_errors.insert(&issue.key);
                }
                QualityLevel::Warning => {
                    warning_count += 1;
                    entries_with_warnings.insert(&issue.key);
                }
                QualityLevel::Info => {
                    info_count += 1;
                }
            }
        }

        // Calculate quality score
        // Start at 100, deduct for issues
        let error_penalty = error_count as f64 * 5.0;
        let warning_penalty = warning_count as f64 * 1.0;
        let quality_score = (100.0 - error_penalty - warning_penalty).max(0.0);

        self.summary = QualitySummary {
            total_checked,
            entries_with_errors: entries_with_errors.len(),
            entries_with_warnings: entries_with_warnings.len(),
            error_count,
            warning_count,
            info_count,
            quality_score,
        };
    }

    /// Get issues by level
    pub fn issues_by_level(&self, level: QualityLevel) -> Vec<&QualityIssue> {
        self.issues.iter().filter(|i| i.level == level).collect()
    }

    /// Get issues by type
    pub fn issues_by_type(&self, issue_type: IssueType) -> Vec<&QualityIssue> {
        self.issues.iter().filter(|i| i.issue_type == issue_type).collect()
    }

    /// Check if passing (no errors)
    pub fn is_passing(&self) -> bool {
        self.summary.error_count == 0
    }

    /// Format as markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Translation Quality Report\n\n");
        md.push_str(&format!("Locale: {}\n", self.locale));
        md.push_str(&format!("Generated: {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        // Summary
        md.push_str("## Summary\n\n");
        md.push_str(&format!("- Quality Score: **{:.1}%**\n", self.summary.quality_score));
        md.push_str(&format!("- Entries Checked: {}\n", self.summary.total_checked));
        md.push_str(&format!("- Errors: {} ({} entries)\n", self.summary.error_count, self.summary.entries_with_errors));
        md.push_str(&format!("- Warnings: {} ({} entries)\n", self.summary.warning_count, self.summary.entries_with_warnings));
        md.push_str(&format!("- Info: {}\n\n", self.summary.info_count));

        // Status
        if self.is_passing() {
            md.push_str("Status: **PASSING** (no errors)\n\n");
        } else {
            md.push_str("Status: **FAILING** (has errors)\n\n");
        }

        // Issues by severity
        let errors: Vec<_> = self.issues_by_level(QualityLevel::Error);
        if !errors.is_empty() {
            md.push_str("## Errors\n\n");
            for issue in errors {
                md.push_str(&format!("### `{}`\n", issue.key));
                md.push_str(&format!("- **Type**: {:?}\n", issue.issue_type));
                md.push_str(&format!("- **Message**: {}\n", issue.message));
                md.push_str(&format!("- **Source**: {}\n", issue.source));
                md.push_str(&format!("- **Translation**: {}\n", issue.target));
                if let Some(ref suggestion) = issue.suggestion {
                    md.push_str(&format!("- **Suggestion**: {}\n", suggestion));
                }
                md.push('\n');
            }
        }

        let warnings: Vec<_> = self.issues_by_level(QualityLevel::Warning);
        if !warnings.is_empty() {
            md.push_str("## Warnings\n\n");
            md.push_str("| Key | Type | Message |\n");
            md.push_str("|-----|------|----------|\n");
            for issue in warnings.iter().take(20) {
                md.push_str(&format!(
                    "| `{}` | {:?} | {} |\n",
                    issue.key, issue.issue_type, issue.message
                ));
            }
            if warnings.len() > 20 {
                md.push_str(&format!("\n*... and {} more warnings*\n", warnings.len() - 20));
            }
        }

        md
    }
}

/// Check a single entry for quality issues
fn check_entry(entry: &TranslationEntry, locale: &str, config: &QualityConfig) -> Vec<QualityIssue> {
    let mut issues = Vec::new();

    let source_text = match entry.source.as_string() {
        Some(s) => s,
        None => return issues, // Skip non-string entries
    };

    let target_text = match &entry.target {
        Some(TranslationValue::Simple(s)) => s.as_str(),
        Some(_) => return issues, // Skip non-simple values for now
        None => return issues,     // No translation to check
    };

    // Check for empty translation
    if target_text.is_empty() {
        issues.push(QualityIssue::new(
            &entry.key,
            locale,
            IssueType::EmptyTranslation,
            source_text,
            target_text,
        ));
        return issues;
    }

    // Check placeholders
    if config.check_placeholders {
        let source_placeholders: std::collections::HashSet<_> =
            extract_placeholders(source_text).into_iter().collect();
        let target_placeholders: std::collections::HashSet<_> =
            extract_placeholders(target_text).into_iter().collect();

        for placeholder in &source_placeholders {
            if !target_placeholders.contains(placeholder) {
                issues.push(
                    QualityIssue::new(
                        &entry.key,
                        locale,
                        IssueType::MissingPlaceholder,
                        source_text,
                        target_text,
                    )
                    .with_message(format!("Missing placeholder: {}", placeholder))
                    .with_suggestion(format!("Add {} to the translation", placeholder)),
                );
            }
        }

        for placeholder in &target_placeholders {
            if !source_placeholders.contains(placeholder) {
                issues.push(
                    QualityIssue::new(
                        &entry.key,
                        locale,
                        IssueType::ExtraPlaceholder,
                        source_text,
                        target_text,
                    )
                    .with_message(format!("Extra placeholder not in source: {}", placeholder)),
                );
            }
        }
    }

    // Check length
    if config.check_length {
        let source_len = source_text.len();
        let target_len = target_text.len();

        if let Some(max_len) = entry.metadata.max_length {
            if target_len > max_len {
                issues.push(
                    QualityIssue::new(
                        &entry.key,
                        locale,
                        IssueType::LengthExceeded,
                        source_text,
                        target_text,
                    )
                    .with_message(format!(
                        "Translation length {} exceeds maximum {}",
                        target_len, max_len
                    )),
                );
            }
        }

        if source_len > 0 {
            let ratio = target_len as f64 / source_len as f64;
            if ratio > config.max_length_multiplier {
                issues.push(
                    QualityIssue::new(
                        &entry.key,
                        locale,
                        IssueType::LengthExceeded,
                        source_text,
                        target_text,
                    )
                    .with_message(format!(
                        "Translation is {:.1}x longer than source",
                        ratio
                    ))
                    .with_level(QualityLevel::Warning),
                );
            }

            if ratio < config.min_length_ratio && source_len > 10 {
                issues.push(
                    QualityIssue::new(
                        &entry.key,
                        locale,
                        IssueType::LengthTooShort,
                        source_text,
                        target_text,
                    )
                    .with_message(format!(
                        "Translation is only {:.0}% of source length",
                        ratio * 100.0
                    )),
                );
            }
        }
    }

    // Check for identical text
    if config.check_identical && source_text == target_text && locale != "en" {
        issues.push(
            QualityIssue::new(
                &entry.key,
                locale,
                IssueType::IdenticalToSource,
                source_text,
                target_text,
            )
            .with_message("Translation is identical to source (may be untranslated)"),
        );
    }

    // Check whitespace
    if config.check_whitespace {
        if target_text != target_text.trim() {
            issues.push(
                QualityIssue::new(
                    &entry.key,
                    locale,
                    IssueType::WhitespaceIssue,
                    source_text,
                    target_text,
                )
                .with_message("Translation has leading or trailing whitespace")
                .with_suggestion(target_text.trim().to_string()),
            );
        }
    }

    // Check punctuation
    if config.check_punctuation && !source_text.is_empty() && !target_text.is_empty() {
        let source_ends_with_punct = source_text
            .chars()
            .last()
            .map(|c| c.is_ascii_punctuation())
            .unwrap_or(false);
        let target_ends_with_punct = target_text
            .chars()
            .last()
            .map(|c| c.is_ascii_punctuation())
            .unwrap_or(false);

        if source_ends_with_punct && !target_ends_with_punct {
            let punct = source_text.chars().last().unwrap();
            issues.push(
                QualityIssue::new(
                    &entry.key,
                    locale,
                    IssueType::MissingPunctuation,
                    source_text,
                    target_text,
                )
                .with_message(format!("Translation is missing ending punctuation '{}'", punct)),
            );
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{TranslationMetadata, TranslationState};

    fn create_entry(key: &str, source: &str, target: Option<&str>) -> TranslationEntry {
        TranslationEntry {
            key: key.to_string(),
            source: TranslationValue::Simple(source.to_string()),
            target: target.map(|t| TranslationValue::Simple(t.to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata::default(),
        }
    }

    #[test]
    fn test_missing_placeholder() {
        let entry = create_entry("test", "Hello {name}!", Some("Hallo!"));
        let config = QualityConfig::default();
        let issues = check_entry(&entry, "de", &config);

        assert!(!issues.is_empty());
        assert!(issues.iter().any(|i| i.issue_type == IssueType::MissingPlaceholder));
    }

    #[test]
    fn test_identical_translation() {
        let entry = create_entry("test", "Hello", Some("Hello"));
        let config = QualityConfig::default();
        let issues = check_entry(&entry, "de", &config);

        assert!(issues.iter().any(|i| i.issue_type == IssueType::IdenticalToSource));
    }

    #[test]
    fn test_whitespace_issue() {
        let entry = create_entry("test", "Hello", Some(" Hallo "));
        let config = QualityConfig::default();
        let issues = check_entry(&entry, "de", &config);

        assert!(issues.iter().any(|i| i.issue_type == IssueType::WhitespaceIssue));
    }

    #[test]
    fn test_quality_report() {
        let mut file = TranslationFile::new("en", "de");
        file.add_entry(create_entry("good", "Hello", Some("Hallo")));
        file.add_entry(create_entry("bad", "Hello {name}!", Some("Hallo!")));

        let config = QualityConfig::default();
        let report = QualityReport::from_file(&file, &config);

        assert_eq!(report.summary.total_checked, 2);
        assert!(report.summary.error_count > 0);
    }
}
