//! XLIFF 2.0 translation file format
//!
//! Industry-standard XML format for professional translation services.
//! Supports translation units, notes, and metadata.

use crate::error::{I18nError, I18nResult};
use crate::formats::{
    FileMetadata, TranslationEntry, TranslationFile, TranslationMetadata, TranslationState,
    TranslationValue,
};
use quick_xml::de::from_str;
use quick_xml::se::to_string;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

/// XLIFF format handler
pub struct XliffFormat;

impl XliffFormat {
    /// Load an XLIFF translation file
    pub fn load(path: &Path) -> I18nResult<TranslationFile> {
        let content = fs::read_to_string(path)?;
        let xliff: XliffFile =
            from_str(&content).map_err(|e| I18nError::parse_error(path, e.to_string()))?;

        xliff.to_translation_file()
    }

    /// Save a translation file as XLIFF
    pub fn save(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let xliff = XliffFile::from_translation_file(file);
        let content = xliff.to_xml()?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// XLIFF 2.0 file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename = "xliff")]
pub struct XliffFile {
    /// XLIFF version (2.0)
    #[serde(rename = "@version")]
    pub version: String,

    /// XML namespace
    #[serde(rename = "@xmlns")]
    pub xmlns: String,

    /// Source language
    #[serde(rename = "@srcLang")]
    pub src_lang: String,

    /// Target language
    #[serde(rename = "@trgLang")]
    pub trg_lang: String,

    /// File units
    pub file: XliffFileElement,
}

/// File element in XLIFF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XliffFileElement {
    /// File ID
    #[serde(rename = "@id")]
    pub id: String,

    /// Original file name
    #[serde(rename = "@original", skip_serializing_if = "Option::is_none")]
    pub original: Option<String>,

    /// Translation units
    #[serde(rename = "unit", default)]
    pub units: Vec<TranslationUnit>,

    /// Notes
    #[serde(rename = "notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<XliffNotes>,
}

/// Notes container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XliffNotes {
    #[serde(rename = "note", default)]
    pub notes: Vec<XliffNote>,
}

/// A single note
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XliffNote {
    /// Note category
    #[serde(rename = "@category", skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Note content
    #[serde(rename = "$text")]
    pub content: String,
}

/// A translation unit in XLIFF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationUnit {
    /// Unit ID (the translation key)
    #[serde(rename = "@id")]
    pub id: String,

    /// Unit name (optional human-readable name)
    #[serde(rename = "@name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Translation state
    #[serde(rename = "@state", skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,

    /// Segment containing source and target
    pub segment: XliffSegment,

    /// Notes for this unit
    #[serde(rename = "notes", skip_serializing_if = "Option::is_none")]
    pub notes: Option<XliffNotes>,
}

/// Segment element containing source and target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XliffSegment {
    /// Source text
    pub source: String,

    /// Target text (translated)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
}

impl XliffFile {
    /// Create a new XLIFF file
    pub fn new(source_lang: &str, target_lang: &str) -> Self {
        Self {
            version: "2.0".to_string(),
            xmlns: "urn:oasis:names:tc:xliff:document:2.0".to_string(),
            src_lang: source_lang.to_string(),
            trg_lang: target_lang.to_string(),
            file: XliffFileElement {
                id: "f1".to_string(),
                original: None,
                units: Vec::new(),
                notes: None,
            },
        }
    }

    /// Convert to XML string
    pub fn to_xml(&self) -> I18nResult<String> {
        let mut xml = String::from(r#"<?xml version="1.0" encoding="UTF-8"?>"#);
        xml.push('\n');

        // Manual XML generation for better formatting
        xml.push_str(&format!(
            r#"<xliff version="{}" xmlns="{}" srcLang="{}" trgLang="{}">"#,
            self.version, self.xmlns, self.src_lang, self.trg_lang
        ));
        xml.push('\n');

        xml.push_str(&format!(r#"  <file id="{}">"#, self.file.id));
        if let Some(orig) = &self.file.original {
            xml = xml.replace(
                &format!(r#"<file id="{}">"#, self.file.id),
                &format!(r#"<file id="{}" original="{}">"#, self.file.id, orig),
            );
        }
        xml.push('\n');

        // Add file notes
        if let Some(notes) = &self.file.notes {
            xml.push_str("    <notes>\n");
            for note in &notes.notes {
                if let Some(cat) = &note.category {
                    xml.push_str(&format!(
                        r#"      <note category="{}">{}</note>"#,
                        cat,
                        escape_xml(&note.content)
                    ));
                } else {
                    xml.push_str(&format!(
                        "      <note>{}</note>",
                        escape_xml(&note.content)
                    ));
                }
                xml.push('\n');
            }
            xml.push_str("    </notes>\n");
        }

        // Add units
        for unit in &self.file.units {
            xml.push_str(&format!(r#"    <unit id="{}""#, unit.id));
            if let Some(name) = &unit.name {
                xml.push_str(&format!(r#" name="{}""#, name));
            }
            if let Some(state) = &unit.state {
                xml.push_str(&format!(r#" state="{}""#, state));
            }
            xml.push_str(">\n");

            // Add unit notes
            if let Some(notes) = &unit.notes {
                xml.push_str("      <notes>\n");
                for note in &notes.notes {
                    if let Some(cat) = &note.category {
                        xml.push_str(&format!(
                            r#"        <note category="{}">{}</note>"#,
                            cat,
                            escape_xml(&note.content)
                        ));
                    } else {
                        xml.push_str(&format!(
                            "        <note>{}</note>",
                            escape_xml(&note.content)
                        ));
                    }
                    xml.push('\n');
                }
                xml.push_str("      </notes>\n");
            }

            xml.push_str("      <segment>\n");
            xml.push_str(&format!(
                "        <source>{}</source>\n",
                escape_xml(&unit.segment.source)
            ));
            if let Some(target) = &unit.segment.target {
                xml.push_str(&format!(
                    "        <target>{}</target>\n",
                    escape_xml(target)
                ));
            }
            xml.push_str("      </segment>\n");
            xml.push_str("    </unit>\n");
        }

        xml.push_str("  </file>\n");
        xml.push_str("</xliff>\n");

        Ok(xml)
    }

    /// Convert to unified TranslationFile
    pub fn to_translation_file(&self) -> I18nResult<TranslationFile> {
        let entries: Vec<TranslationEntry> = self
            .file
            .units
            .iter()
            .map(|unit| {
                let state = unit
                    .state
                    .as_ref()
                    .map(|s| xliff_state_to_state(s))
                    .unwrap_or_default();

                let mut metadata = TranslationMetadata::default();
                if let Some(notes) = &unit.notes {
                    for note in &notes.notes {
                        match note.category.as_deref() {
                            Some("context") => metadata.context = Some(note.content.clone()),
                            Some("developer") => metadata.notes = Some(note.content.clone()),
                            _ => {
                                if metadata.notes.is_none() {
                                    metadata.notes = Some(note.content.clone());
                                }
                            }
                        }
                    }
                }

                TranslationEntry {
                    key: unit.id.clone(),
                    source: TranslationValue::Simple(unit.segment.source.clone()),
                    target: unit
                        .segment
                        .target
                        .clone()
                        .map(TranslationValue::Simple),
                    state,
                    metadata,
                }
            })
            .collect();

        Ok(TranslationFile {
            source_locale: self.src_lang.clone(),
            target_locale: self.trg_lang.clone(),
            version: self.version.clone(),
            entries,
            metadata: FileMetadata {
                project: self.file.original.clone(),
                modified: None,
                notes: None,
            },
        })
    }

    /// Create from unified TranslationFile
    pub fn from_translation_file(file: &TranslationFile) -> Self {
        let units: Vec<TranslationUnit> = file
            .entries
            .iter()
            .map(|entry| {
                let source_text = match &entry.source {
                    TranslationValue::Simple(s) => s.clone(),
                    TranslationValue::Plural(p) => p.other.clone().unwrap_or_default(),
                    TranslationValue::Array(a) => a.join("\n"),
                };

                let target_text = entry.target.as_ref().map(|t| match t {
                    TranslationValue::Simple(s) => s.clone(),
                    TranslationValue::Plural(p) => p.other.clone().unwrap_or_default(),
                    TranslationValue::Array(a) => a.join("\n"),
                });

                let mut notes = Vec::new();
                if let Some(ctx) = &entry.metadata.context {
                    notes.push(XliffNote {
                        category: Some("context".to_string()),
                        content: ctx.clone(),
                    });
                }
                if let Some(dev_notes) = &entry.metadata.notes {
                    notes.push(XliffNote {
                        category: Some("developer".to_string()),
                        content: dev_notes.clone(),
                    });
                }

                TranslationUnit {
                    id: entry.key.clone(),
                    name: None,
                    state: Some(state_to_xliff_state(entry.state)),
                    segment: XliffSegment {
                        source: source_text,
                        target: target_text,
                    },
                    notes: if notes.is_empty() {
                        None
                    } else {
                        Some(XliffNotes { notes })
                    },
                }
            })
            .collect();

        let mut xliff = XliffFile::new(&file.source_locale, &file.target_locale);
        xliff.file.units = units;
        xliff.file.original = file.metadata.project.clone();
        xliff
    }
}

/// Convert XLIFF state to internal state
fn xliff_state_to_state(xliff_state: &str) -> TranslationState {
    match xliff_state {
        "initial" => TranslationState::New,
        "translated" => TranslationState::NeedsReview,
        "reviewed" => TranslationState::Approved,
        "final" => TranslationState::Final,
        "needs-translation" => TranslationState::Outdated,
        _ => TranslationState::New,
    }
}

/// Convert internal state to XLIFF state
fn state_to_xliff_state(state: TranslationState) -> String {
    match state {
        TranslationState::New => "initial".to_string(),
        TranslationState::InProgress => "initial".to_string(),
        TranslationState::NeedsReview => "translated".to_string(),
        TranslationState::Approved => "reviewed".to_string(),
        TranslationState::Final => "final".to_string(),
        TranslationState::Outdated => "needs-translation".to_string(),
    }
}

/// Escape XML special characters
fn escape_xml(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_xliff_creation() {
        let mut xliff = XliffFile::new("en", "de");
        xliff.file.units.push(TranslationUnit {
            id: "greeting".to_string(),
            name: None,
            state: Some("initial".to_string()),
            segment: XliffSegment {
                source: "Hello".to_string(),
                target: None,
            },
            notes: None,
        });

        let xml = xliff.to_xml().unwrap();
        assert!(xml.contains("srcLang=\"en\""));
        assert!(xml.contains("trgLang=\"de\""));
        assert!(xml.contains("<source>Hello</source>"));
    }

    #[test]
    fn test_xliff_roundtrip() {
        let mut file = TranslationFile::new("en", "es");
        file.add_entry(TranslationEntry {
            key: "welcome".to_string(),
            source: TranslationValue::Simple("Welcome".to_string()),
            target: Some(TranslationValue::Simple("Bienvenido".to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata {
                context: Some("Homepage greeting".to_string()),
                ..Default::default()
            },
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.xliff");

        XliffFormat::save(&file, &path).unwrap();
        let loaded = XliffFormat::load(&path).unwrap();

        assert_eq!(loaded.source_locale, "en");
        assert_eq!(loaded.target_locale, "es");
        assert_eq!(loaded.entries.len(), 1);
    }
}
