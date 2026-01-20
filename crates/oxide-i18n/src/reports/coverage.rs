//! Translation coverage reporting
//!
//! Analyzes translation files to determine coverage percentages
//! and identify missing or incomplete translations.

use crate::formats::{TranslationFile, TranslationState};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Coverage statistics for a single locale
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleCoverage {
    /// Locale code
    pub locale: String,
    /// Total number of keys
    pub total_keys: usize,
    /// Number of translated keys
    pub translated_keys: usize,
    /// Number of approved keys
    pub approved_keys: usize,
    /// Number of keys needing review
    pub needs_review_keys: usize,
    /// Number of outdated keys
    pub outdated_keys: usize,
    /// Translation percentage (0-100)
    pub translation_percentage: f64,
    /// Approval percentage (0-100)
    pub approval_percentage: f64,
    /// List of missing keys
    pub missing_keys: Vec<String>,
    /// List of keys needing review
    pub review_keys: Vec<String>,
    /// List of outdated keys
    pub outdated_key_list: Vec<String>,
}

impl LocaleCoverage {
    /// Create coverage from a translation file
    pub fn from_file(file: &TranslationFile) -> Self {
        let total_keys = file.entries.len();
        let mut translated_keys = 0;
        let mut approved_keys = 0;
        let mut needs_review_keys = 0;
        let mut outdated_keys = 0;
        let mut missing_keys = Vec::new();
        let mut review_keys = Vec::new();
        let mut outdated_key_list = Vec::new();

        for entry in &file.entries {
            if entry.target.is_some() {
                translated_keys += 1;
            } else {
                missing_keys.push(entry.key.clone());
            }

            match entry.state {
                TranslationState::Approved | TranslationState::Final => {
                    approved_keys += 1;
                }
                TranslationState::NeedsReview => {
                    needs_review_keys += 1;
                    review_keys.push(entry.key.clone());
                }
                TranslationState::Outdated => {
                    outdated_keys += 1;
                    outdated_key_list.push(entry.key.clone());
                }
                _ => {}
            }
        }

        let translation_percentage = if total_keys > 0 {
            (translated_keys as f64 / total_keys as f64) * 100.0
        } else {
            100.0
        };

        let approval_percentage = if total_keys > 0 {
            (approved_keys as f64 / total_keys as f64) * 100.0
        } else {
            100.0
        };

        LocaleCoverage {
            locale: file.target_locale.clone(),
            total_keys,
            translated_keys,
            approved_keys,
            needs_review_keys,
            outdated_keys,
            translation_percentage,
            approval_percentage,
            missing_keys,
            review_keys,
            outdated_key_list,
        }
    }

    /// Check if this locale is fully translated
    pub fn is_complete(&self) -> bool {
        self.translation_percentage >= 100.0
    }

    /// Check if this locale is release-ready
    pub fn is_release_ready(&self) -> bool {
        self.translation_percentage >= 100.0
            && self.needs_review_keys == 0
            && self.outdated_keys == 0
    }

    /// Get a status emoji for quick visualization
    pub fn status_emoji(&self) -> &'static str {
        if self.is_release_ready() {
            "complete"
        } else if self.translation_percentage >= 100.0 {
            "needs-review"
        } else if self.translation_percentage >= 75.0 {
            "good"
        } else if self.translation_percentage >= 50.0 {
            "warning"
        } else {
            "critical"
        }
    }
}

/// Coverage for a specific key across all locales
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyCoverage {
    /// The translation key
    pub key: String,
    /// Locales where this key is translated
    pub translated_locales: Vec<String>,
    /// Locales where this key is missing
    pub missing_locales: Vec<String>,
    /// Locales where this key needs review
    pub review_locales: Vec<String>,
    /// Coverage percentage across locales
    pub coverage_percentage: f64,
}

impl KeyCoverage {
    /// Check if this key is fully translated
    pub fn is_complete(&self) -> bool {
        self.missing_locales.is_empty()
    }
}

/// Comprehensive coverage report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageReport {
    /// Project name
    pub project: Option<String>,
    /// Report generation time
    pub generated_at: DateTime<Utc>,
    /// Source locale
    pub source_locale: String,
    /// Coverage by locale
    pub by_locale: Vec<LocaleCoverage>,
    /// Coverage by key (for keys with issues)
    pub by_key: Vec<KeyCoverage>,
    /// Overall statistics
    pub overall: OverallStats,
}

/// Overall statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OverallStats {
    /// Total number of locales
    pub total_locales: usize,
    /// Number of complete locales
    pub complete_locales: usize,
    /// Number of release-ready locales
    pub release_ready_locales: usize,
    /// Total number of keys
    pub total_keys: usize,
    /// Average translation percentage
    pub average_translation_percentage: f64,
    /// Average approval percentage
    pub average_approval_percentage: f64,
    /// Total missing translations
    pub total_missing: usize,
    /// Total needing review
    pub total_needs_review: usize,
}

impl CoverageReport {
    /// Create a new coverage report
    pub fn new(source_locale: impl Into<String>) -> Self {
        Self {
            project: None,
            generated_at: Utc::now(),
            source_locale: source_locale.into(),
            by_locale: Vec::new(),
            by_key: Vec::new(),
            overall: OverallStats::default(),
        }
    }

    /// Set project name
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Add locale coverage
    pub fn add_locale(&mut self, coverage: LocaleCoverage) {
        self.by_locale.push(coverage);
    }

    /// Generate from multiple translation files
    pub fn from_files(source_locale: &str, files: &[TranslationFile]) -> Self {
        let mut report = CoverageReport::new(source_locale);

        // Collect all keys
        let mut all_keys: HashSet<String> = HashSet::new();
        let mut key_translations: HashMap<String, HashSet<String>> = HashMap::new();
        let mut key_reviews: HashMap<String, Vec<String>> = HashMap::new();

        for file in files {
            let coverage = LocaleCoverage::from_file(file);

            for entry in &file.entries {
                all_keys.insert(entry.key.clone());

                if entry.target.is_some() {
                    key_translations
                        .entry(entry.key.clone())
                        .or_default()
                        .insert(file.target_locale.clone());
                }

                if entry.state == TranslationState::NeedsReview {
                    key_reviews
                        .entry(entry.key.clone())
                        .or_default()
                        .push(file.target_locale.clone());
                }
            }

            report.add_locale(coverage);
        }

        // Calculate key coverage
        let locales: HashSet<String> = files.iter().map(|f| f.target_locale.clone()).collect();
        let total_locales = locales.len();

        for key in &all_keys {
            let translated: Vec<String> = key_translations
                .get(key)
                .map(|s| s.iter().cloned().collect())
                .unwrap_or_default();

            let missing: Vec<String> = locales
                .iter()
                .filter(|l| !translated.contains(l))
                .cloned()
                .collect();

            let review = key_reviews.get(key).cloned().unwrap_or_default();

            // Only add keys with issues
            if !missing.is_empty() || !review.is_empty() {
                let coverage_percentage = if total_locales > 0 {
                    (translated.len() as f64 / total_locales as f64) * 100.0
                } else {
                    100.0
                };

                report.by_key.push(KeyCoverage {
                    key: key.clone(),
                    translated_locales: translated,
                    missing_locales: missing,
                    review_locales: review,
                    coverage_percentage,
                });
            }
        }

        // Sort keys by coverage (worst first)
        report.by_key.sort_by(|a, b| {
            a.coverage_percentage
                .partial_cmp(&b.coverage_percentage)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Calculate overall stats
        report.calculate_overall();

        report
    }

    /// Calculate overall statistics
    fn calculate_overall(&mut self) {
        let total_locales = self.by_locale.len();
        if total_locales == 0 {
            return;
        }

        let complete_locales = self.by_locale.iter().filter(|c| c.is_complete()).count();
        let release_ready_locales = self.by_locale.iter().filter(|c| c.is_release_ready()).count();

        let total_keys = self.by_locale.first().map(|c| c.total_keys).unwrap_or(0);

        let avg_translation = self.by_locale.iter().map(|c| c.translation_percentage).sum::<f64>()
            / total_locales as f64;

        let avg_approval = self.by_locale.iter().map(|c| c.approval_percentage).sum::<f64>()
            / total_locales as f64;

        let total_missing: usize = self.by_locale.iter().map(|c| c.missing_keys.len()).sum();
        let total_needs_review: usize = self.by_locale.iter().map(|c| c.needs_review_keys).sum();

        self.overall = OverallStats {
            total_locales,
            complete_locales,
            release_ready_locales,
            total_keys,
            average_translation_percentage: avg_translation,
            average_approval_percentage: avg_approval,
            total_missing,
            total_needs_review,
        };
    }

    /// Format as markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Translation Coverage Report\n\n");
        md.push_str(&format!("Generated: {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        if let Some(ref project) = self.project {
            md.push_str(&format!("Project: {}\n\n", project));
        }

        // Overall summary
        md.push_str("## Summary\n\n");
        md.push_str("| Metric | Value |\n");
        md.push_str("|--------|-------|\n");
        md.push_str(&format!("| Total Locales | {} |\n", self.overall.total_locales));
        md.push_str(&format!("| Complete Locales | {} |\n", self.overall.complete_locales));
        md.push_str(&format!("| Release Ready | {} |\n", self.overall.release_ready_locales));
        md.push_str(&format!("| Total Keys | {} |\n", self.overall.total_keys));
        md.push_str(&format!("| Avg Translation | {:.1}% |\n", self.overall.average_translation_percentage));
        md.push_str(&format!("| Total Missing | {} |\n", self.overall.total_missing));
        md.push_str(&format!("| Needs Review | {} |\n", self.overall.total_needs_review));

        // Locale details
        md.push_str("\n## Coverage by Locale\n\n");
        md.push_str("| Locale | Translated | Approved | Missing | Review | Status |\n");
        md.push_str("|--------|------------|----------|---------|--------|--------|\n");

        for locale in &self.by_locale {
            md.push_str(&format!(
                "| {} | {:.1}% | {:.1}% | {} | {} | {} |\n",
                locale.locale,
                locale.translation_percentage,
                locale.approval_percentage,
                locale.missing_keys.len(),
                locale.needs_review_keys,
                locale.status_emoji()
            ));
        }

        // Keys with issues
        if !self.by_key.is_empty() {
            md.push_str("\n## Keys with Issues\n\n");
            md.push_str("| Key | Coverage | Missing Locales | Review Locales |\n");
            md.push_str("|-----|----------|-----------------|----------------|\n");

            for key in self.by_key.iter().take(20) {
                md.push_str(&format!(
                    "| `{}` | {:.1}% | {} | {} |\n",
                    key.key,
                    key.coverage_percentage,
                    key.missing_locales.join(", "),
                    key.review_locales.join(", ")
                ));
            }

            if self.by_key.len() > 20 {
                md.push_str(&format!("\n*... and {} more keys with issues*\n", self.by_key.len() - 20));
            }
        }

        md
    }

    /// Format as JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_default()
    }

    /// Check if ready for release
    pub fn is_release_ready(&self) -> bool {
        self.overall.release_ready_locales == self.overall.total_locales
    }

    /// Get locales not ready for release
    pub fn blockers(&self) -> Vec<&LocaleCoverage> {
        self.by_locale.iter().filter(|c| !c.is_release_ready()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::{TranslationEntry, TranslationMetadata, TranslationValue};

    fn create_test_file(locale: &str, translated: usize, total: usize) -> TranslationFile {
        let mut file = TranslationFile::new("en", locale);

        for i in 0..total {
            let has_translation = i < translated;
            file.add_entry(TranslationEntry {
                key: format!("key.{}", i),
                source: TranslationValue::Simple(format!("Source {}", i)),
                target: if has_translation {
                    Some(TranslationValue::Simple(format!("Translation {}", i)))
                } else {
                    None
                },
                state: if has_translation {
                    TranslationState::Approved
                } else {
                    TranslationState::New
                },
                metadata: TranslationMetadata::default(),
            });
        }

        file
    }

    #[test]
    fn test_locale_coverage() {
        let file = create_test_file("de", 8, 10);
        let coverage = LocaleCoverage::from_file(&file);

        assert_eq!(coverage.locale, "de");
        assert_eq!(coverage.total_keys, 10);
        assert_eq!(coverage.translated_keys, 8);
        assert_eq!(coverage.translation_percentage, 80.0);
        assert_eq!(coverage.missing_keys.len(), 2);
    }

    #[test]
    fn test_coverage_report() {
        let files = vec![
            create_test_file("de", 10, 10),
            create_test_file("fr", 8, 10),
            create_test_file("es", 5, 10),
        ];

        let report = CoverageReport::from_files("en", &files);

        assert_eq!(report.overall.total_locales, 3);
        assert_eq!(report.overall.complete_locales, 1);
        assert_eq!(report.overall.total_keys, 10);
    }

    #[test]
    fn test_release_ready() {
        let file = create_test_file("de", 10, 10);
        let coverage = LocaleCoverage::from_file(&file);
        assert!(coverage.is_release_ready());

        let file = create_test_file("fr", 8, 10);
        let coverage = LocaleCoverage::from_file(&file);
        assert!(!coverage.is_release_ready());
    }
}
