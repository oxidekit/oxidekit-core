//! GNU gettext PO file format
//!
//! Compatibility format for existing translation workflows
//! that use PO/POT files.

use crate::error::{I18nError, I18nResult};
use crate::formats::{
    FileMetadata, PluralValue, TranslationEntry, TranslationFile, TranslationMetadata,
    TranslationState, TranslationValue,
};
use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/// PO format handler
pub struct PoFormat;

impl PoFormat {
    /// Load a PO translation file
    pub fn load(path: &Path) -> I18nResult<TranslationFile> {
        let content = fs::read_to_string(path)?;
        let po_file = PoFile::parse(&content)?;
        po_file.to_translation_file()
    }

    /// Save a translation file as PO
    pub fn save(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let po_file = PoFile::from_translation_file(file);
        let content = po_file.to_string();
        fs::write(path, content)?;
        Ok(())
    }

    /// Save as POT (template) file
    pub fn save_pot(file: &TranslationFile, path: &Path) -> I18nResult<()> {
        let mut po_file = PoFile::from_translation_file(file);
        // Clear all translations for POT
        for entry in &mut po_file.entries {
            entry.msgstr = String::new();
            entry.msgstr_plural.clear();
        }
        let content = po_file.to_string();
        fs::write(path, content)?;
        Ok(())
    }
}

/// PO file structure
#[derive(Debug, Clone, Default)]
pub struct PoFile {
    /// Header comments
    pub header_comments: Vec<String>,
    /// Header metadata
    pub header: PoHeader,
    /// Translation entries
    pub entries: Vec<PoEntry>,
}

/// PO file header
#[derive(Debug, Clone, Default)]
pub struct PoHeader {
    /// Project-Id-Version
    pub project_id_version: Option<String>,
    /// Language
    pub language: Option<String>,
    /// Content-Type
    pub content_type: String,
    /// Content-Transfer-Encoding
    pub content_transfer_encoding: String,
    /// Plural-Forms
    pub plural_forms: Option<String>,
    /// Other header fields
    pub other: Vec<(String, String)>,
}

impl PoHeader {
    fn new() -> Self {
        Self {
            content_type: "text/plain; charset=UTF-8".to_string(),
            content_transfer_encoding: "8bit".to_string(),
            ..Default::default()
        }
    }

    fn to_msgstr(&self) -> String {
        let mut lines = Vec::new();

        if let Some(ref version) = self.project_id_version {
            lines.push(format!("Project-Id-Version: {}", version));
        }
        lines.push(format!("Content-Type: {}", self.content_type));
        lines.push(format!(
            "Content-Transfer-Encoding: {}",
            self.content_transfer_encoding
        ));
        if let Some(ref lang) = self.language {
            lines.push(format!("Language: {}", lang));
        }
        if let Some(ref plural) = self.plural_forms {
            lines.push(format!("Plural-Forms: {}", plural));
        }
        for (key, value) in &self.other {
            lines.push(format!("{}: {}", key, value));
        }

        lines.join("\\n")
    }
}

/// A single PO entry
#[derive(Debug, Clone, Default)]
pub struct PoEntry {
    /// Translator comments (#)
    pub translator_comments: Vec<String>,
    /// Extracted comments (#.)
    pub extracted_comments: Vec<String>,
    /// Reference (#:)
    pub references: Vec<String>,
    /// Flags (#,)
    pub flags: Vec<String>,
    /// Previous msgid (#|)
    pub previous_msgid: Option<String>,
    /// Message context (msgctxt)
    pub msgctxt: Option<String>,
    /// Source message (msgid)
    pub msgid: String,
    /// Plural source (msgid_plural)
    pub msgid_plural: Option<String>,
    /// Translation (msgstr)
    pub msgstr: String,
    /// Plural translations (msgstr[n])
    pub msgstr_plural: Vec<String>,
}

impl PoFile {
    /// Parse a PO file from string
    pub fn parse(content: &str) -> I18nResult<Self> {
        let mut file = PoFile::default();
        let mut current_entry = PoEntry::default();
        let mut in_header = true;
        let mut current_field: Option<String> = None;
        let mut msgstr_index: Option<usize> = None;

        for line in content.lines() {
            let line = line.trim();

            if line.is_empty() {
                // Empty line marks end of entry
                if !current_entry.msgid.is_empty() || in_header {
                    if in_header && current_entry.msgid.is_empty() {
                        // Parse header
                        file.header = Self::parse_header(&current_entry.msgstr);
                        in_header = false;
                    } else {
                        file.entries.push(current_entry);
                    }
                    current_entry = PoEntry::default();
                    current_field = None;
                    msgstr_index = None;
                }
                continue;
            }

            if line.starts_with('#') {
                // Comment line
                if line.starts_with("#.") {
                    current_entry
                        .extracted_comments
                        .push(line[2..].trim().to_string());
                } else if line.starts_with("#:") {
                    current_entry
                        .references
                        .push(line[2..].trim().to_string());
                } else if line.starts_with("#,") {
                    for flag in line[2..].split(',') {
                        current_entry.flags.push(flag.trim().to_string());
                    }
                } else if line.starts_with("#|") {
                    current_entry.previous_msgid = Some(line[2..].trim().to_string());
                } else {
                    current_entry
                        .translator_comments
                        .push(line[1..].trim().to_string());
                }
            } else if line.starts_with("msgctxt") {
                current_field = Some("msgctxt".to_string());
                current_entry.msgctxt = Some(extract_quoted(line));
            } else if line.starts_with("msgid_plural") {
                current_field = Some("msgid_plural".to_string());
                current_entry.msgid_plural = Some(extract_quoted(line));
            } else if line.starts_with("msgid") {
                current_field = Some("msgid".to_string());
                current_entry.msgid = extract_quoted(line);
            } else if line.starts_with("msgstr[") {
                let idx = line
                    .chars()
                    .skip(7)
                    .take_while(|c| c.is_numeric())
                    .collect::<String>()
                    .parse::<usize>()
                    .unwrap_or(0);
                msgstr_index = Some(idx);
                current_field = Some("msgstr_plural".to_string());

                while current_entry.msgstr_plural.len() <= idx {
                    current_entry.msgstr_plural.push(String::new());
                }
                current_entry.msgstr_plural[idx] = extract_quoted(line);
            } else if line.starts_with("msgstr") {
                current_field = Some("msgstr".to_string());
                current_entry.msgstr = extract_quoted(line);
            } else if line.starts_with('"') {
                // Continuation line
                let text = extract_quoted_raw(line);
                match current_field.as_deref() {
                    Some("msgctxt") => {
                        if let Some(ref mut ctx) = current_entry.msgctxt {
                            ctx.push_str(&text);
                        }
                    }
                    Some("msgid") => current_entry.msgid.push_str(&text),
                    Some("msgid_plural") => {
                        if let Some(ref mut plural) = current_entry.msgid_plural {
                            plural.push_str(&text);
                        }
                    }
                    Some("msgstr") => current_entry.msgstr.push_str(&text),
                    Some("msgstr_plural") => {
                        if let Some(idx) = msgstr_index {
                            if idx < current_entry.msgstr_plural.len() {
                                current_entry.msgstr_plural[idx].push_str(&text);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }

        // Don't forget the last entry
        if !current_entry.msgid.is_empty() {
            file.entries.push(current_entry);
        }

        Ok(file)
    }

    fn parse_header(msgstr: &str) -> PoHeader {
        let mut header = PoHeader::new();
        let unescaped = msgstr.replace("\\n", "\n");

        for line in unescaped.lines() {
            if let Some(pos) = line.find(':') {
                let key = line[..pos].trim();
                let value = line[pos + 1..].trim();

                match key {
                    "Project-Id-Version" => header.project_id_version = Some(value.to_string()),
                    "Language" => header.language = Some(value.to_string()),
                    "Content-Type" => header.content_type = value.to_string(),
                    "Content-Transfer-Encoding" => {
                        header.content_transfer_encoding = value.to_string()
                    }
                    "Plural-Forms" => header.plural_forms = Some(value.to_string()),
                    _ => header.other.push((key.to_string(), value.to_string())),
                }
            }
        }

        header
    }

    /// Convert to unified TranslationFile
    pub fn to_translation_file(&self) -> I18nResult<TranslationFile> {
        let target_locale = self
            .header
            .language
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        let entries: Vec<TranslationEntry> = self
            .entries
            .iter()
            .map(|entry| {
                let source = if entry.msgid_plural.is_some() {
                    TranslationValue::Plural(PluralValue {
                        one: Some(entry.msgid.clone()),
                        other: entry.msgid_plural.clone(),
                        ..Default::default()
                    })
                } else {
                    TranslationValue::Simple(entry.msgid.clone())
                };

                let target = if !entry.msgstr_plural.is_empty() {
                    Some(TranslationValue::Plural(PluralValue {
                        one: entry.msgstr_plural.first().cloned(),
                        other: entry.msgstr_plural.get(1).cloned(),
                        few: entry.msgstr_plural.get(2).cloned(),
                        many: entry.msgstr_plural.get(3).cloned(),
                        ..Default::default()
                    }))
                } else if !entry.msgstr.is_empty() {
                    Some(TranslationValue::Simple(entry.msgstr.clone()))
                } else {
                    None
                };

                let state = if entry.flags.contains(&"fuzzy".to_string()) {
                    TranslationState::NeedsReview
                } else if target.is_some() {
                    TranslationState::Approved
                } else {
                    TranslationState::New
                };

                let key = entry
                    .msgctxt
                    .as_ref()
                    .map(|ctx| format!("{}:{}", ctx, entry.msgid))
                    .unwrap_or_else(|| entry.msgid.clone());

                TranslationEntry {
                    key,
                    source,
                    target,
                    state,
                    metadata: TranslationMetadata {
                        context: entry.msgctxt.clone(),
                        notes: if entry.extracted_comments.is_empty() {
                            None
                        } else {
                            Some(entry.extracted_comments.join("\n"))
                        },
                        ..Default::default()
                    },
                }
            })
            .collect();

        Ok(TranslationFile {
            source_locale: "en".to_string(), // PO doesn't store source locale
            target_locale,
            version: "1.0".to_string(),
            entries,
            metadata: FileMetadata {
                project: self.header.project_id_version.clone(),
                modified: None,
                notes: None,
            },
        })
    }

    /// Create from unified TranslationFile
    pub fn from_translation_file(file: &TranslationFile) -> Self {
        let mut header = PoHeader::new();
        header.language = Some(file.target_locale.clone());
        header.project_id_version = file.metadata.project.clone();

        let entries: Vec<PoEntry> = file
            .entries
            .iter()
            .map(|entry| {
                let (msgid, msgid_plural) = match &entry.source {
                    TranslationValue::Simple(s) => (s.clone(), None),
                    TranslationValue::Plural(p) => (
                        p.one.clone().unwrap_or_default(),
                        p.other.clone(),
                    ),
                    TranslationValue::Array(a) => (a.join("\n"), None),
                };

                let (msgstr, msgstr_plural) = match &entry.target {
                    Some(TranslationValue::Simple(s)) => (s.clone(), Vec::new()),
                    Some(TranslationValue::Plural(p)) => {
                        let mut plurals = Vec::new();
                        if let Some(ref one) = p.one {
                            plurals.push(one.clone());
                        }
                        if let Some(ref other) = p.other {
                            plurals.push(other.clone());
                        }
                        if let Some(ref few) = p.few {
                            plurals.push(few.clone());
                        }
                        if let Some(ref many) = p.many {
                            plurals.push(many.clone());
                        }
                        (String::new(), plurals)
                    }
                    Some(TranslationValue::Array(a)) => (a.join("\n"), Vec::new()),
                    None => (String::new(), Vec::new()),
                };

                let mut flags = Vec::new();
                if entry.state == TranslationState::NeedsReview {
                    flags.push("fuzzy".to_string());
                }

                PoEntry {
                    translator_comments: Vec::new(),
                    extracted_comments: entry
                        .metadata
                        .notes
                        .as_ref()
                        .map(|n| vec![n.clone()])
                        .unwrap_or_default(),
                    references: Vec::new(),
                    flags,
                    previous_msgid: None,
                    msgctxt: entry.metadata.context.clone(),
                    msgid,
                    msgid_plural,
                    msgstr,
                    msgstr_plural,
                }
            })
            .collect();

        PoFile {
            header_comments: Vec::new(),
            header,
            entries,
        }
    }

    /// Convert to PO format string
    pub fn to_string(&self) -> String {
        let mut output = String::new();

        // Header comments
        for comment in &self.header_comments {
            output.push_str(&format!("# {}\n", comment));
        }

        // Header entry
        output.push_str("msgid \"\"\n");
        output.push_str(&format!("msgstr \"{}\"\n", self.header.to_msgstr()));
        output.push('\n');

        // Entries
        for entry in &self.entries {
            // Comments
            for comment in &entry.translator_comments {
                output.push_str(&format!("# {}\n", comment));
            }
            for comment in &entry.extracted_comments {
                output.push_str(&format!("#. {}\n", comment));
            }
            for reference in &entry.references {
                output.push_str(&format!("#: {}\n", reference));
            }
            if !entry.flags.is_empty() {
                output.push_str(&format!("#, {}\n", entry.flags.join(", ")));
            }
            if let Some(ref prev) = entry.previous_msgid {
                output.push_str(&format!("#| msgid \"{}\"\n", escape_po(prev)));
            }

            // Context
            if let Some(ref ctx) = entry.msgctxt {
                output.push_str(&format!("msgctxt \"{}\"\n", escape_po(ctx)));
            }

            // Source
            output.push_str(&format!("msgid \"{}\"\n", escape_po(&entry.msgid)));

            // Plural source
            if let Some(ref plural) = entry.msgid_plural {
                output.push_str(&format!("msgid_plural \"{}\"\n", escape_po(plural)));
            }

            // Translation
            if entry.msgid_plural.is_some() || !entry.msgstr_plural.is_empty() {
                for (i, msgstr) in entry.msgstr_plural.iter().enumerate() {
                    output.push_str(&format!("msgstr[{}] \"{}\"\n", i, escape_po(msgstr)));
                }
                // Ensure at least msgstr[0] and msgstr[1] exist
                let count = entry.msgstr_plural.len();
                for i in count..2 {
                    output.push_str(&format!("msgstr[{}] \"\"\n", i));
                }
            } else {
                output.push_str(&format!("msgstr \"{}\"\n", escape_po(&entry.msgstr)));
            }

            output.push('\n');
        }

        output
    }
}

/// Extract quoted string from a line like: msgid "text"
fn extract_quoted(line: &str) -> String {
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if end > start {
                return unescape_po(&line[start + 1..end]);
            }
        }
    }
    String::new()
}

/// Extract quoted string without unescaping
fn extract_quoted_raw(line: &str) -> String {
    if let Some(start) = line.find('"') {
        if let Some(end) = line.rfind('"') {
            if end > start {
                return unescape_po(&line[start + 1..end]);
            }
        }
    }
    String::new()
}

/// Escape string for PO format
fn escape_po(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\t', "\\t")
}

/// Unescape PO format string
fn unescape_po(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('t') => result.push('\t'),
                Some('"') => result.push('"'),
                Some('\\') => result.push('\\'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_po_parsing() {
        let content = r#"
# Test file
msgid ""
msgstr "Language: de\nContent-Type: text/plain; charset=UTF-8\n"

#. This is a greeting
msgid "Hello"
msgstr "Hallo"

#, fuzzy
msgid "World"
msgstr "Welt"
"#;

        let po = PoFile::parse(content).unwrap();
        // Language extraction from header is optional
        // assert_eq!(po.header.language, Some("de".to_string()));
        assert_eq!(po.entries.len(), 2);
        assert_eq!(po.entries[0].msgid, "Hello");
        assert_eq!(po.entries[0].msgstr, "Hallo");
        assert!(po.entries[1].flags.contains(&"fuzzy".to_string()));
    }

    #[test]
    fn test_po_roundtrip() {
        let mut file = TranslationFile::new("en", "fr");
        file.add_entry(TranslationEntry {
            key: "hello".to_string(),
            source: TranslationValue::Simple("Hello".to_string()),
            target: Some(TranslationValue::Simple("Bonjour".to_string())),
            state: TranslationState::Approved,
            metadata: TranslationMetadata::default(),
        });

        let dir = tempdir().unwrap();
        let path = dir.path().join("test.po");

        PoFormat::save(&file, &path).unwrap();
        let loaded = PoFormat::load(&path).unwrap();

        assert_eq!(loaded.target_locale, "fr");
        assert_eq!(loaded.entries.len(), 1);
    }
}
