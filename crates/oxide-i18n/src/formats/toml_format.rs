//! TOML translation file format
//!
//! The internal format used for development. Supports nested namespaces
//! and is easy to edit by developers.

use crate::error::{I18nError, I18nResult};
use crate::formats::{
    FileMetadata, PluralValue, TranslationEntry, TranslationFile, TranslationMetadata,
    TranslationState, TranslationValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// TOML format handler
pub struct TomlFormat;

impl TomlFormat {
    /// Load a TOML translation file
    pub fn load(path: &Path) -> I18nResult<TranslationFile> {
        let content = fs::read_to_string(path)?;
        let toml_file: TomlTranslationFile =
            toml::from_str(&content).map_err(|e| I18nError::parse_error(path, e.to_string()))?;

        toml_file.to_translation_file()
    }

    /// Save a translation file as TOML
    pub fn save(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let toml_file = TomlTranslationFile::from_translation_file(file);
        let content = toml::to_string_pretty(&toml_file)
            .map_err(|e| I18nError::with_context("TOML serialization", e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// TOML file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlTranslationFile {
    /// File metadata section
    #[serde(default)]
    pub meta: TomlMeta,

    /// Translation entries organized by namespace
    #[serde(flatten)]
    pub namespaces: HashMap<String, TomlNamespace>,
}

/// Metadata section in TOML
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TomlMeta {
    /// Source locale
    #[serde(default = "default_source")]
    pub source: String,
    /// Target locale
    #[serde(default)]
    pub target: String,
    /// File version
    #[serde(default = "default_version")]
    pub version: String,
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

fn default_source() -> String {
    "en".to_string()
}

fn default_version() -> String {
    "1.0".to_string()
}

/// A namespace containing translation entries
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TomlNamespace {
    /// Nested namespace
    Nested(HashMap<String, TomlNamespace>),
    /// Simple string value
    Simple(String),
    /// Plural value
    Plural(TomlPluralValue),
    /// Entry with metadata
    WithMeta(TomlEntryWithMeta),
}

/// Plural value in TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlPluralValue {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub few: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub many: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub other: Option<String>,
}

impl From<TomlPluralValue> for PluralValue {
    fn from(v: TomlPluralValue) -> Self {
        PluralValue {
            zero: v.zero,
            one: v.one,
            two: v.two,
            few: v.few,
            many: v.many,
            other: v.other,
        }
    }
}

impl From<&PluralValue> for TomlPluralValue {
    fn from(v: &PluralValue) -> Self {
        TomlPluralValue {
            zero: v.zero.clone(),
            one: v.one.clone(),
            two: v.two.clone(),
            few: v.few.clone(),
            many: v.many.clone(),
            other: v.other.clone(),
        }
    }
}

/// Entry with metadata in TOML
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TomlEntryWithMeta {
    /// The translation value
    pub value: String,
    /// Context for translators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Maximum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Translation state
    #[serde(default, skip_serializing_if = "is_default_state")]
    pub state: TranslationState,
}

fn is_default_state(state: &TranslationState) -> bool {
    *state == TranslationState::default()
}

impl TomlTranslationFile {
    /// Convert to unified TranslationFile
    pub fn to_translation_file(&self) -> I18nResult<TranslationFile> {
        let mut file = TranslationFile::new(&self.meta.source, &self.meta.target);
        file.version = self.meta.version.clone();
        file.metadata = FileMetadata {
            project: self.meta.project.clone(),
            modified: None,
            notes: self.meta.notes.clone(),
        };

        // Flatten namespaces into entries
        for (key, namespace) in &self.namespaces {
            if key == "meta" {
                continue; // Skip metadata section
            }
            self.flatten_namespace(key, namespace, &mut file.entries);
        }

        Ok(file)
    }

    /// Recursively flatten namespaces
    fn flatten_namespace(&self, prefix: &str, namespace: &TomlNamespace, entries: &mut Vec<TranslationEntry>) {
        match namespace {
            TomlNamespace::Simple(value) => {
                entries.push(TranslationEntry {
                    key: prefix.to_string(),
                    source: TranslationValue::Simple(value.clone()),
                    target: None,
                    state: TranslationState::New,
                    metadata: TranslationMetadata::default(),
                });
            }
            TomlNamespace::Plural(plural) => {
                entries.push(TranslationEntry {
                    key: prefix.to_string(),
                    source: TranslationValue::Plural(plural.clone().into()),
                    target: None,
                    state: TranslationState::New,
                    metadata: TranslationMetadata::default(),
                });
            }
            TomlNamespace::WithMeta(entry) => {
                entries.push(TranslationEntry {
                    key: prefix.to_string(),
                    source: TranslationValue::Simple(entry.value.clone()),
                    target: None,
                    state: entry.state,
                    metadata: TranslationMetadata {
                        context: entry.context.clone(),
                        max_length: entry.max_length,
                        notes: entry.notes.clone(),
                        tags: entry.tags.clone(),
                        screenshot: None,
                        modified: None,
                        modified_by: None,
                    },
                });
            }
            TomlNamespace::Nested(map) => {
                for (key, child) in map {
                    let new_prefix = format!("{}.{}", prefix, key);
                    self.flatten_namespace(&new_prefix, child, entries);
                }
            }
        }
    }

    /// Create from unified TranslationFile
    pub fn from_translation_file(file: &TranslationFile) -> Self {
        let mut namespaces: HashMap<String, TomlNamespace> = HashMap::new();

        for entry in &file.entries {
            let parts: Vec<&str> = entry.key.split('.').collect();
            Self::insert_entry(&mut namespaces, &parts, entry);
        }

        TomlTranslationFile {
            meta: TomlMeta {
                source: file.source_locale.clone(),
                target: file.target_locale.clone(),
                version: file.version.clone(),
                project: file.metadata.project.clone(),
                notes: file.metadata.notes.clone(),
            },
            namespaces,
        }
    }

    /// Insert an entry into the namespace tree
    fn insert_entry(
        namespaces: &mut HashMap<String, TomlNamespace>,
        parts: &[&str],
        entry: &TranslationEntry,
    ) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Leaf node - insert the value
            let value = match &entry.source {
                TranslationValue::Simple(s) => {
                    if entry.metadata.context.is_some()
                        || entry.metadata.max_length.is_some()
                        || entry.metadata.notes.is_some()
                        || !entry.metadata.tags.is_empty()
                    {
                        TomlNamespace::WithMeta(TomlEntryWithMeta {
                            value: s.clone(),
                            context: entry.metadata.context.clone(),
                            max_length: entry.metadata.max_length,
                            notes: entry.metadata.notes.clone(),
                            tags: entry.metadata.tags.clone(),
                            state: entry.state,
                        })
                    } else {
                        TomlNamespace::Simple(s.clone())
                    }
                }
                TranslationValue::Plural(p) => TomlNamespace::Plural(p.into()),
                TranslationValue::Array(a) => {
                    // For arrays, join with newlines
                    TomlNamespace::Simple(a.join("\n"))
                }
            };
            namespaces.insert(parts[0].to_string(), value);
        } else {
            // Intermediate node - ensure nested map exists
            let child = namespaces
                .entry(parts[0].to_string())
                .or_insert_with(|| TomlNamespace::Nested(HashMap::new()));

            if let TomlNamespace::Nested(map) = child {
                Self::insert_entry(map, &parts[1..], entry);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_toml_roundtrip() {
        let mut file = TranslationFile::new("en", "de");
        file.add_entry(TranslationEntry {
            key: "common.greeting".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata::default(),
        });
        file.add_entry(TranslationEntry {
            key: "common.farewell".to_string(),
            source: TranslationValue::Simple("Goodbye".to_string()),
            target: None,
            state: TranslationState::New,
            metadata: TranslationMetadata {
                context: Some("Used when user leaves".to_string()),
                ..Default::default()
            },
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.toml");

        TomlFormat::save(&file, &path).unwrap();
        let loaded = TomlFormat::load(&path).unwrap();

        assert_eq!(loaded.source_locale, "en");
        assert_eq!(loaded.target_locale, "de");
        assert!(loaded.entries.len() >= 2);
    }
}
