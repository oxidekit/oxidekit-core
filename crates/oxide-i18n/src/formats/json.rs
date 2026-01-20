//! JSON translation file format
//!
//! Used for API integration and external tools. Provides a flat
//! or nested structure that's easy to parse programmatically.

use crate::error::{I18nError, I18nResult};
use crate::formats::{
    FileMetadata, PluralValue, TranslationEntry, TranslationFile, TranslationMetadata,
    TranslationState, TranslationValue,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// JSON format handler
pub struct JsonFormat;

impl JsonFormat {
    /// Load a JSON translation file
    pub fn load(path: &Path) -> I18nResult<TranslationFile> {
        let content = fs::read_to_string(path)?;
        let json_file: JsonTranslationFile =
            serde_json::from_str(&content).map_err(|e| I18nError::parse_error(path, e.to_string()))?;

        json_file.to_translation_file()
    }

    /// Save a translation file as JSON
    pub fn save(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let json_file = JsonTranslationFile::from_translation_file(file);
        let content = serde_json::to_string_pretty(&json_file)
            .map_err(|e| I18nError::with_context("JSON serialization", e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Save as a simple key-value JSON (for runtime use)
    pub fn save_simple(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let mut map: HashMap<String, serde_json::Value> = HashMap::new();

        for entry in &file.entries {
            let value = if let Some(target) = &entry.target {
                value_to_json(target)
            } else {
                value_to_json(&entry.source)
            };
            map.insert(entry.key.clone(), value);
        }

        let content = serde_json::to_string_pretty(&map)
            .map_err(|e| I18nError::with_context("JSON serialization", e.to_string()))?;
        fs::write(path, content)?;
        Ok(())
    }
}

fn value_to_json(value: &TranslationValue) -> serde_json::Value {
    match value {
        TranslationValue::Simple(s) => serde_json::Value::String(s.clone()),
        TranslationValue::Plural(p) => {
            let mut map = serde_json::Map::new();
            if let Some(v) = &p.zero {
                map.insert("zero".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(v) = &p.one {
                map.insert("one".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(v) = &p.two {
                map.insert("two".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(v) = &p.few {
                map.insert("few".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(v) = &p.many {
                map.insert("many".to_string(), serde_json::Value::String(v.clone()));
            }
            if let Some(v) = &p.other {
                map.insert("other".to_string(), serde_json::Value::String(v.clone()));
            }
            serde_json::Value::Object(map)
        }
        TranslationValue::Array(a) => {
            serde_json::Value::Array(a.iter().map(|s| serde_json::Value::String(s.clone())).collect())
        }
    }
}

/// JSON file structure for full translation workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonTranslationFile {
    /// File format version
    #[serde(default = "default_version")]
    pub version: String,
    /// Source locale
    pub source_locale: String,
    /// Target locale
    pub target_locale: String,
    /// File metadata
    #[serde(default, skip_serializing_if = "JsonMetadata::is_empty")]
    pub metadata: JsonMetadata,
    /// Translation entries
    pub entries: Vec<JsonEntry>,
}

fn default_version() -> String {
    "1.0".to_string()
}

/// File metadata in JSON format
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsonMetadata {
    /// Project name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Last modified timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modified: Option<String>,
    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl JsonMetadata {
    fn is_empty(&self) -> bool {
        self.project.is_none() && self.modified.is_none() && self.notes.is_none()
    }
}

/// A single translation entry in JSON format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonEntry {
    /// Translation key
    pub key: String,
    /// Source text
    pub source: JsonValue,
    /// Target text (translated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<JsonValue>,
    /// Translation state
    #[serde(default)]
    pub state: TranslationState,
    /// Entry metadata
    #[serde(default, skip_serializing_if = "JsonEntryMeta::is_empty")]
    pub metadata: JsonEntryMeta,
}

/// Translation value in JSON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonValue {
    /// Simple string
    Simple(String),
    /// Plural forms
    Plural {
        #[serde(skip_serializing_if = "Option::is_none")]
        zero: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        one: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        two: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        few: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        many: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        other: Option<String>,
    },
    /// Array of values
    Array(Vec<String>),
}

impl From<&TranslationValue> for JsonValue {
    fn from(v: &TranslationValue) -> Self {
        match v {
            TranslationValue::Simple(s) => JsonValue::Simple(s.clone()),
            TranslationValue::Plural(p) => JsonValue::Plural {
                zero: p.zero.clone(),
                one: p.one.clone(),
                two: p.two.clone(),
                few: p.few.clone(),
                many: p.many.clone(),
                other: p.other.clone(),
            },
            TranslationValue::Array(a) => JsonValue::Array(a.clone()),
        }
    }
}

impl From<JsonValue> for TranslationValue {
    fn from(v: JsonValue) -> Self {
        match v {
            JsonValue::Simple(s) => TranslationValue::Simple(s),
            JsonValue::Plural {
                zero,
                one,
                two,
                few,
                many,
                other,
            } => TranslationValue::Plural(PluralValue {
                zero,
                one,
                two,
                few,
                many,
                other,
            }),
            JsonValue::Array(a) => TranslationValue::Array(a),
        }
    }
}

/// Entry metadata in JSON
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct JsonEntryMeta {
    /// Context for translators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Maximum length
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    /// Notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Screenshot URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub screenshot: Option<String>,
    /// Tags
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
}

impl JsonEntryMeta {
    fn is_empty(&self) -> bool {
        self.context.is_none()
            && self.max_length.is_none()
            && self.notes.is_none()
            && self.screenshot.is_none()
            && self.tags.is_empty()
    }
}

impl JsonTranslationFile {
    /// Convert to unified TranslationFile
    pub fn to_translation_file(&self) -> I18nResult<TranslationFile> {
        let entries: Vec<TranslationEntry> = self
            .entries
            .iter()
            .map(|e| TranslationEntry {
                key: e.key.clone(),
                source: e.source.clone().into(),
                target: e.target.clone().map(|v| v.into()),
                state: e.state,
                metadata: TranslationMetadata {
                    context: e.metadata.context.clone(),
                    max_length: e.metadata.max_length,
                    notes: e.metadata.notes.clone(),
                    screenshot: e.metadata.screenshot.clone(),
                    tags: e.metadata.tags.clone(),
                    modified: None,
                    modified_by: None,
                },
            })
            .collect();

        Ok(TranslationFile {
            source_locale: self.source_locale.clone(),
            target_locale: self.target_locale.clone(),
            version: self.version.clone(),
            entries,
            metadata: FileMetadata {
                project: self.metadata.project.clone(),
                modified: None,
                notes: self.metadata.notes.clone(),
            },
        })
    }

    /// Create from unified TranslationFile
    pub fn from_translation_file(file: &TranslationFile) -> Self {
        let entries: Vec<JsonEntry> = file
            .entries
            .iter()
            .map(|e| JsonEntry {
                key: e.key.clone(),
                source: (&e.source).into(),
                target: e.target.as_ref().map(|v| v.into()),
                state: e.state,
                metadata: JsonEntryMeta {
                    context: e.metadata.context.clone(),
                    max_length: e.metadata.max_length,
                    notes: e.metadata.notes.clone(),
                    screenshot: e.metadata.screenshot.clone(),
                    tags: e.metadata.tags.clone(),
                },
            })
            .collect();

        JsonTranslationFile {
            version: file.version.clone(),
            source_locale: file.source_locale.clone(),
            target_locale: file.target_locale.clone(),
            metadata: JsonMetadata {
                project: file.metadata.project.clone(),
                modified: file.metadata.modified.map(|d| d.to_rfc3339()),
                notes: file.metadata.notes.clone(),
            },
            entries,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_json_roundtrip() {
        let mut file = TranslationFile::new("en", "fr");
        file.add_entry(TranslationEntry {
            key: "hello".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: Some(TranslationValue::Simple("Bonjour".to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata::default(),
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.json");

        JsonFormat::save(&file, &path).unwrap();
        let loaded = JsonFormat::load(&path).unwrap();

        assert_eq!(loaded.source_locale, "en");
        assert_eq!(loaded.target_locale, "fr");
        assert_eq!(loaded.entries.len(), 1);
        assert_eq!(loaded.entries[0].state, TranslationState::Approved);
    }

    #[test]
    fn test_json_simple_export() {
        let mut file = TranslationFile::new("en", "de");
        file.add_entry(TranslationEntry {
            key: "greeting".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: Some(TranslationValue::Simple("Hallo".to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata::default(),
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("simple.json");

        JsonFormat::save_simple(&file, &path).unwrap();
        let content = fs::read_to_string(&path).unwrap();
        let parsed: HashMap<String, String> = serde_json::from_str(&content).unwrap();

        assert_eq!(parsed.get("greeting"), Some(&"Hallo".to_string()));
    }
}
