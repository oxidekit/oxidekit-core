//! Translation diff and change tracking
//!
//! Tools for comparing translation files and tracking changes:
//! - Diff between versions
//! - Change detection
//! - Impact analysis

use crate::formats::{TranslationEntry, TranslationFile, TranslationState, TranslationValue};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::{HashMap, HashSet};

/// Type of change detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    /// Key was added
    Added,
    /// Key was removed
    Removed,
    /// Source text was modified
    SourceModified,
    /// Translation was added
    TranslationAdded,
    /// Translation was modified
    TranslationModified,
    /// Translation was removed
    TranslationRemoved,
    /// State changed
    StateChanged,
    /// Metadata changed
    MetadataChanged,
}

impl ChangeType {
    /// Check if this change affects translations
    pub fn affects_translations(&self) -> bool {
        matches!(
            self,
            ChangeType::Added
                | ChangeType::SourceModified
                | ChangeType::TranslationAdded
                | ChangeType::TranslationModified
                | ChangeType::TranslationRemoved
        )
    }

    /// Get severity for release impact
    pub fn severity(&self) -> ChangeSeverity {
        match self {
            ChangeType::Added => ChangeSeverity::Medium,
            ChangeType::Removed => ChangeSeverity::High,
            ChangeType::SourceModified => ChangeSeverity::High,
            ChangeType::TranslationAdded => ChangeSeverity::Low,
            ChangeType::TranslationModified => ChangeSeverity::Medium,
            ChangeType::TranslationRemoved => ChangeSeverity::High,
            ChangeType::StateChanged => ChangeSeverity::Low,
            ChangeType::MetadataChanged => ChangeSeverity::Low,
        }
    }
}

/// Severity of a change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeSeverity {
    /// Low impact
    Low,
    /// Medium impact
    Medium,
    /// High impact
    High,
}

/// A single change entry in the diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffEntry {
    /// The translation key
    pub key: String,
    /// Type of change
    pub change_type: ChangeType,
    /// Old value (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_value: Option<String>,
    /// New value (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_value: Option<String>,
    /// Textual diff (for text changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text_diff: Option<String>,
    /// Locale affected (for translation changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
}

impl DiffEntry {
    /// Create a new diff entry
    pub fn new(key: impl Into<String>, change_type: ChangeType) -> Self {
        Self {
            key: key.into(),
            change_type,
            old_value: None,
            new_value: None,
            text_diff: None,
            locale: None,
        }
    }

    /// Add old and new values
    pub fn with_values(
        mut self,
        old: Option<impl Into<String>>,
        new: Option<impl Into<String>>,
    ) -> Self {
        self.old_value = old.map(|v| v.into());
        self.new_value = new.map(|v| v.into());
        self
    }

    /// Add locale information
    pub fn for_locale(mut self, locale: impl Into<String>) -> Self {
        self.locale = Some(locale.into());
        self
    }

    /// Generate text diff
    pub fn with_text_diff(mut self) -> Self {
        if let (Some(old), Some(new)) = (&self.old_value, &self.new_value) {
            let diff = TextDiff::from_lines(old, new);
            let mut output = String::new();

            for change in diff.iter_all_changes() {
                let sign = match change.tag() {
                    ChangeTag::Delete => "-",
                    ChangeTag::Insert => "+",
                    ChangeTag::Equal => " ",
                };
                output.push_str(&format!("{}{}", sign, change));
            }

            if !output.is_empty() {
                self.text_diff = Some(output);
            }
        }
        self
    }
}

/// Diff between two translation files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationDiff {
    /// Source locale
    pub source_locale: String,
    /// Target locale (for translation diffs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_locale: Option<String>,
    /// Version of the old file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old_version: Option<String>,
    /// Version of the new file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new_version: Option<String>,
    /// When the diff was generated
    pub generated_at: DateTime<Utc>,
    /// Individual changes
    pub changes: Vec<DiffEntry>,
    /// Summary statistics
    pub summary: DiffSummary,
}

/// Summary statistics for a diff
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffSummary {
    /// Number of keys added
    pub added: usize,
    /// Number of keys removed
    pub removed: usize,
    /// Number of source modifications
    pub source_modified: usize,
    /// Number of translation changes
    pub translations_changed: usize,
    /// Total keys in old file
    pub total_old: usize,
    /// Total keys in new file
    pub total_new: usize,
}

impl TranslationDiff {
    /// Create an empty diff
    pub fn new(source_locale: impl Into<String>) -> Self {
        Self {
            source_locale: source_locale.into(),
            target_locale: None,
            old_version: None,
            new_version: None,
            generated_at: Utc::now(),
            changes: Vec::new(),
            summary: DiffSummary::default(),
        }
    }

    /// Compare two translation files
    pub fn compare(old: &TranslationFile, new: &TranslationFile) -> Self {
        let mut diff = Self::new(&new.source_locale);
        diff.target_locale = Some(new.target_locale.clone());
        diff.old_version = Some(old.version.clone());
        diff.new_version = Some(new.version.clone());

        let old_keys: HashSet<&str> = old.entries.iter().map(|e| e.key.as_str()).collect();
        let new_keys: HashSet<&str> = new.entries.iter().map(|e| e.key.as_str()).collect();

        let old_map: HashMap<&str, &TranslationEntry> =
            old.entries.iter().map(|e| (e.key.as_str(), e)).collect();
        let new_map: HashMap<&str, &TranslationEntry> =
            new.entries.iter().map(|e| (e.key.as_str(), e)).collect();

        // Find added keys
        for key in new_keys.difference(&old_keys) {
            let entry = new_map[key];
            let mut change = DiffEntry::new(*key, ChangeType::Added)
                .with_values(None::<String>, entry.source.as_string());

            if new.target_locale != new.source_locale {
                change = change.for_locale(&new.target_locale);
            }

            diff.changes.push(change);
            diff.summary.added += 1;
        }

        // Find removed keys
        for key in old_keys.difference(&new_keys) {
            let entry = old_map[key];
            let mut change = DiffEntry::new(*key, ChangeType::Removed)
                .with_values(entry.source.as_string(), None::<String>);

            if old.target_locale != old.source_locale {
                change = change.for_locale(&old.target_locale);
            }

            diff.changes.push(change);
            diff.summary.removed += 1;
        }

        // Find modified keys
        for key in old_keys.intersection(&new_keys) {
            let old_entry = old_map[key];
            let new_entry = new_map[key];

            // Check source text changes
            if old_entry.source != new_entry.source {
                let change = DiffEntry::new(*key, ChangeType::SourceModified)
                    .with_values(old_entry.source.as_string(), new_entry.source.as_string())
                    .with_text_diff();

                diff.changes.push(change);
                diff.summary.source_modified += 1;
            }

            // Check translation changes
            match (&old_entry.target, &new_entry.target) {
                (None, Some(new_target)) => {
                    let change = DiffEntry::new(*key, ChangeType::TranslationAdded)
                        .with_values(None::<String>, new_target.as_string())
                        .for_locale(&new.target_locale);

                    diff.changes.push(change);
                    diff.summary.translations_changed += 1;
                }
                (Some(old_target), None) => {
                    let change = DiffEntry::new(*key, ChangeType::TranslationRemoved)
                        .with_values(old_target.as_string(), None::<String>)
                        .for_locale(&new.target_locale);

                    diff.changes.push(change);
                    diff.summary.translations_changed += 1;
                }
                (Some(old_target), Some(new_target)) if old_target != new_target => {
                    let change = DiffEntry::new(*key, ChangeType::TranslationModified)
                        .with_values(old_target.as_string(), new_target.as_string())
                        .for_locale(&new.target_locale)
                        .with_text_diff();

                    diff.changes.push(change);
                    diff.summary.translations_changed += 1;
                }
                _ => {}
            }

            // Check state changes
            if old_entry.state != new_entry.state {
                let change = DiffEntry::new(*key, ChangeType::StateChanged)
                    .with_values(
                        Some(format!("{:?}", old_entry.state)),
                        Some(format!("{:?}", new_entry.state)),
                    );

                diff.changes.push(change);
            }
        }

        diff.summary.total_old = old.entries.len();
        diff.summary.total_new = new.entries.len();

        diff
    }

    /// Get changes by type
    pub fn changes_by_type(&self, change_type: ChangeType) -> Vec<&DiffEntry> {
        self.changes
            .iter()
            .filter(|c| c.change_type == change_type)
            .collect()
    }

    /// Check if there are any breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.changes
            .iter()
            .any(|c| c.change_type.severity() == ChangeSeverity::High)
    }

    /// Get changes that require translation updates
    pub fn translation_impacting_changes(&self) -> Vec<&DiffEntry> {
        self.changes
            .iter()
            .filter(|c| c.change_type.affects_translations())
            .collect()
    }

    /// Format as markdown report
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        md.push_str("# Translation Diff Report\n\n");
        md.push_str(&format!("Generated: {}\n\n", self.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));

        if let (Some(old_v), Some(new_v)) = (&self.old_version, &self.new_version) {
            md.push_str(&format!("Comparing version {} to {}\n\n", old_v, new_v));
        }

        md.push_str("## Summary\n\n");
        md.push_str(&format!("| Metric | Count |\n"));
        md.push_str(&format!("|--------|-------|\n"));
        md.push_str(&format!("| Keys Added | {} |\n", self.summary.added));
        md.push_str(&format!("| Keys Removed | {} |\n", self.summary.removed));
        md.push_str(&format!("| Source Modified | {} |\n", self.summary.source_modified));
        md.push_str(&format!("| Translations Changed | {} |\n", self.summary.translations_changed));

        if !self.changes.is_empty() {
            md.push_str("\n## Changes\n\n");

            // Group by change type
            let added: Vec<_> = self.changes_by_type(ChangeType::Added);
            if !added.is_empty() {
                md.push_str("### Added Keys\n\n");
                for change in added {
                    md.push_str(&format!("- `{}`", change.key));
                    if let Some(ref val) = change.new_value {
                        md.push_str(&format!(": {}", truncate(val, 60)));
                    }
                    md.push('\n');
                }
                md.push('\n');
            }

            let removed: Vec<_> = self.changes_by_type(ChangeType::Removed);
            if !removed.is_empty() {
                md.push_str("### Removed Keys\n\n");
                for change in removed {
                    md.push_str(&format!("- `{}`\n", change.key));
                }
                md.push('\n');
            }

            let modified: Vec<_> = self.changes_by_type(ChangeType::SourceModified);
            if !modified.is_empty() {
                md.push_str("### Modified Source\n\n");
                for change in modified {
                    md.push_str(&format!("#### `{}`\n\n", change.key));
                    if let Some(ref diff) = change.text_diff {
                        md.push_str("```diff\n");
                        md.push_str(diff);
                        md.push_str("```\n\n");
                    }
                }
            }
        }

        md
    }
}

/// Truncate a string for display
fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formats::TranslationMetadata;

    #[test]
    fn test_diff_added_keys() {
        let old = TranslationFile::new("en", "de");
        let mut new = TranslationFile::new("en", "de");
        new.add_entry(TranslationEntry {
            key: "new.key".to_string(),
            source: TranslationValue::Simple("New text".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });

        let diff = TranslationDiff::compare(&old, &new);
        assert_eq!(diff.summary.added, 1);
        assert_eq!(diff.changes.len(), 1);
        assert_eq!(diff.changes[0].change_type, ChangeType::Added);
    }

    #[test]
    fn test_diff_removed_keys() {
        let mut old = TranslationFile::new("en", "de");
        old.add_entry(TranslationEntry {
            key: "old.key".to_string(),
            source: TranslationValue::Simple("Old text".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });

        let new = TranslationFile::new("en", "de");

        let diff = TranslationDiff::compare(&old, &new);
        assert_eq!(diff.summary.removed, 1);
        assert!(diff.has_breaking_changes());
    }

    #[test]
    fn test_diff_source_modified() {
        let mut old = TranslationFile::new("en", "de");
        old.add_entry(TranslationEntry {
            key: "greeting".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });

        let mut new = TranslationFile::new("en", "de");
        new.add_entry(TranslationEntry {
            key: "greeting".to_string(),
            source: TranslationValue::Simple("Hello, World!".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });

        let diff = TranslationDiff::compare(&old, &new);
        assert_eq!(diff.summary.source_modified, 1);
    }
}
