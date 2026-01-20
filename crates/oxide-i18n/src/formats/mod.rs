//! Translation file formats for team collaboration
//!
//! Supports multiple formats for different workflows:
//! - TOML: Internal development format
//! - JSON: API and tool integration
//! - XLIFF: Professional translator exchange format
//! - PO: GNU gettext compatibility

pub mod json;
pub mod toml_format;
pub mod xliff;
pub mod po;

pub use json::{JsonFormat, JsonTranslationFile};
pub use toml_format::{TomlFormat, TomlTranslationFile};
pub use xliff::{XliffFormat, XliffFile, TranslationUnit};
pub use po::{PoFormat, PoFile, PoEntry};

use crate::error::{I18nError, I18nResult};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// A translation value that can be simple text or pluralized
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TranslationValue {
    /// Simple text value
    Simple(String),
    /// Pluralized value with forms
    Plural(PluralValue),
    /// Array of values (for lists)
    Array(Vec<String>),
}

impl TranslationValue {
    /// Get the simple string value, or the "other" form for plurals
    pub fn as_string(&self) -> Option<&str> {
        match self {
            TranslationValue::Simple(s) => Some(s),
            TranslationValue::Plural(p) => p.other.as_deref(),
            TranslationValue::Array(_) => None,
        }
    }

    /// Check if this value has any content
    pub fn is_empty(&self) -> bool {
        match self {
            TranslationValue::Simple(s) => s.is_empty(),
            TranslationValue::Plural(p) => {
                p.one.as_ref().map_or(true, |s| s.is_empty())
                    && p.other.as_ref().map_or(true, |s| s.is_empty())
            }
            TranslationValue::Array(a) => a.is_empty(),
        }
    }
}

/// Plural forms for a translation
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PluralValue {
    /// Zero form (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero: Option<String>,
    /// Singular form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one: Option<String>,
    /// Two form (for dual languages)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two: Option<String>,
    /// Few form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub few: Option<String>,
    /// Many form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub many: Option<String>,
    /// Other/default form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
}

impl Default for PluralValue {
    fn default() -> Self {
        Self {
            zero: None,
            one: None,
            two: None,
            few: None,
            many: None,
            other: None,
        }
    }
}

/// Metadata for a translation entry
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationMetadata {
    /// Context for translators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Maximum character length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Developer notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Screenshot or reference image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    /// Tags for categorization
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    /// Who last modified this entry
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified_by: Option<String>,
}

/// A single translation entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationEntry {
    /// The translation key
    pub key: String,
    /// Source text (usually English)
    pub source: TranslationValue,
    /// Translated text
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TranslationValue>,
    /// Translation state
    #[serde(default)]
    pub state: TranslationState,
    /// Metadata for this entry
    #[serde(default, skip_serializing_if = "TranslationMetadata::is_default")]
    pub metadata: TranslationMetadata,
}

impl TranslationMetadata {
    fn is_default(&self) -> bool {
        self.context.is_none()
            && self.max_length.is_none()
            && self.notes.is_none()
            && self.screenshot.is_none()
            && self.tags.is_empty()
            && self.modified.is_none()
            && self.modified_by.is_none()
    }
}

/// State of a translation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TranslationState {
    /// Not yet translated
    #[default]
    New,
    /// Translation in progress
    InProgress,
    /// Translated but needs review
    NeedsReview,
    /// Reviewed and approved
    Approved,
    /// Final/published
    Final,
    /// Source text changed, needs update
    Outdated,
}

impl TranslationState {
    /// Check if the translation is complete
    pub fn is_complete(&self) -> bool {
        matches!(self, TranslationState::Approved | TranslationState::Final)
    }

    /// Check if the translation needs work
    pub fn needs_work(&self) -> bool {
        matches!(
            self,
            TranslationState::New
                | TranslationState::InProgress
                | TranslationState::NeedsReview
                | TranslationState::Outdated
        )
    }
}

/// Unified translation file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationFile {
    /// Source locale (e.g., "en")
    pub source_locale: String,
    /// Target locale (e.g., "de")
    pub target_locale: String,
    /// File version
    #[serde(default = "default_version")]
    pub version: String,
    /// Translation entries
    pub entries: Vec<TranslationEntry>,
    /// File-level metadata
    #[serde(default)]
    pub metadata: FileMetadata,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// File-level metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileMetadata {
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<chrono::DateTime<chrono::Utc>>,
    /// File notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl TranslationFile {
    /// Create a new translation file
    pub fn new(source_locale: impl Into<String>, target_locale: impl Into<String>) -> Self {
        Self {
            source_locale: source_locale.into(),
            target_locale: target_locale.into(),
            version: default_version(),
            entries: Vec::new(),
            metadata: FileMetadata::default(),
        }
    }

    /// Add a translation entry
    pub fn add_entry(&mut self, entry: TranslationEntry) {
        self.entries.push(entry);
    }

    /// Get an entry by key
    pub fn get_entry(&self, key: &str) -> Option<&TranslationEntry> {
        self.entries.iter().find(|e| e.key == key)
    }

    /// Get a mutable entry by key
    pub fn get_entry_mut(&mut self, key: &str) -> Option<&mut TranslationEntry> {
        self.entries.iter_mut().find(|e| e.key == key)
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.iter().map(|e| e.key.as_str())
    }

    /// Get translation coverage statistics
    pub fn coverage(&self) -> CoverageStats {
        let total = self.entries.len();
        let translated = self.entries.iter().filter(|e| e.target.is_some()).count();
        let approved = self
            .entries
            .iter()
            .filter(|e| e.state.is_complete())
            .count();
        let needs_review = self
            .entries
            .iter()
            .filter(|e| e.state == TranslationState::NeedsReview)
            .count();
        let outdated = self
            .entries
            .iter()
            .filter(|e| e.state == TranslationState::Outdated)
            .count();

        CoverageStats {
            total,
            translated,
            approved,
            needs_review,
            outdated,
        }
    }
}

/// Translation coverage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoverageStats {
    /// Total number of keys
    pub total: usize,
    /// Number of translated keys
    pub translated: usize,
    /// Number of approved translations
    pub approved: usize,
    /// Number of translations needing review
    pub needs_review: usize,
    /// Number of outdated translations
    pub outdated: usize,
}

impl CoverageStats {
    /// Get translation percentage
    pub fn translation_percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.translated as f64 / self.total as f64) * 100.0
    }

    /// Get approval percentage
    pub fn approval_percentage(&self) -> f64 {
        if self.total == 0 {
            return 100.0;
        }
        (self.approved as f64 / self.total as f64) * 100.0
    }

    /// Check if fully translated
    pub fn is_fully_translated(&self) -> bool {
        self.translated == self.total
    }

    /// Check if fully approved
    pub fn is_fully_approved(&self) -> bool {
        self.approved == self.total
    }
}

/// Format detection and conversion utilities
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Format {
    /// TOML format (internal)
    Toml,
    /// JSON format (API/tools)
    Json,
    /// XLIFF 2.0 format (professional)
    Xliff,
    /// PO format (GNU gettext)
    Po,
}

impl Format {
    /// Detect format from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "toml" => Some(Format::Toml),
            "json" => Some(Format::Json),
            "xliff" | "xlf" => Some(Format::Xliff),
            "po" => Some(Format::Po),
            _ => None,
        }
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Format::Toml => "toml",
            Format::Json => "json",
            Format::Xliff => "xliff",
            Format::Po => "po",
        }
    }
}

/// Load a translation file, auto-detecting format
pub fn load_file(path: &Path) -> I18nResult<TranslationFile> {
    let format = Format::from_extension(path).ok_or_else(|| {
        I18nError::with_context("format detection", format!("Unknown format for {:?}", path))
    })?;

    match format {
        Format::Toml => TomlFormat::load(path),
        Format::Json => JsonFormat::load(path),
        Format::Xliff => XliffFormat::load(path),
        Format::Po => PoFormat::load(path),
    }
}

/// Save a translation file in the specified format
pub fn save_file(file: &TranslationFile, path: &Path, format: Format) -> I18nResult<()> {
    match format {
        Format::Toml => TomlFormat::save(file, path),
        Format::Json => JsonFormat::save(file, path),
        Format::Xliff => XliffFormat::save(file, path),
        Format::Po => PoFormat::save(file, path),
    }
}

/// Convert between formats
pub fn convert(
    input_path: &Path,
    output_path: &Path,
    output_format: Option<Format>,
) -> I18nResult<()> {
    let file = load_file(input_path)?;
    let format = output_format.unwrap_or_else(|| {
        Format::from_extension(output_path).unwrap_or(Format::Json)
    });
    save_file(&file, output_path, format)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_detection() {
        assert_eq!(
            Format::from_extension(Path::new("test.toml")),
            Some(Format::Toml)
        );
        assert_eq!(
            Format::from_extension(Path::new("test.json")),
            Some(Format::Json)
        );
        assert_eq!(
            Format::from_extension(Path::new("test.xliff")),
            Some(Format::Xliff)
        );
        assert_eq!(
            Format::from_extension(Path::new("test.po")),
            Some(Format::Po)
        );
        assert_eq!(Format::from_extension(Path::new("test.txt")), None);
    }

    #[test]
    fn test_coverage_stats() {
        let mut file = TranslationFile::new("en", "de");
        file.add_entry(TranslationEntry {
            key: "key1".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: Some(TranslationValue::Simple("Hallo".to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata::default(),
        });
        file.add_entry(TranslationEntry {
            key: "key2".to_string(),
            source: TranslationValue::Simple("World".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });

        let stats = file.coverage();
        assert_eq!(stats.total, 2);
        assert_eq!(stats.translated, 1);
        assert_eq!(stats.approved, 1);
        assert_eq!(stats.translation_percentage(), 50.0);
    }
}
