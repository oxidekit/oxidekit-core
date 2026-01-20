//! Translation file format definitions and parsing
//!
//! Supports TOML-based translation files with:
//! - Nested namespaces (e.g., `auth.login.title`)
//! - Metadata support (notes for translators, context)
//! - Plural forms
//! - Optional "locked" keys

use crate::error::{I18nError, I18nResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A translation file containing all translations for a single locale
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationFile {
    /// Metadata about this translation file
    #[serde(default, skip_serializing_if = "TranslationMetadata::is_empty")]
    pub _meta: TranslationMetadata,

    /// The actual translations, stored as nested values
    #[serde(flatten)]
    pub translations: HashMap<String, TranslationNode>,
}

/// Metadata for a translation file
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TranslationMetadata {
    /// Locale identifier (e.g., "en-US")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,

    /// Language name in its own language (e.g., "English")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_name: Option<String>,

    /// Version of the translation file
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    /// Last updated timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_updated: Option<String>,

    /// Translator credits
    #[serde(skip_serializing_if = "Option::is_none")]
    pub translator: Option<String>,

    /// Text direction (ltr or rtl)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
}

impl TranslationMetadata {
    /// Check if metadata is empty (for serialization skip)
    pub fn is_empty(&self) -> bool {
        self.locale.is_none()
            && self.language_name.is_none()
            && self.version.is_none()
            && self.last_updated.is_none()
            && self.translator.is_none()
            && self.direction.is_none()
    }
}

/// A node in the translation tree (can be a value or nested namespace)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum TranslationNode {
    /// A simple string value
    Value(String),

    /// A translation with plural forms
    Plural(PluralValue),

    /// A translation with metadata
    WithMeta(TranslationWithMeta),

    /// A nested namespace
    Namespace(HashMap<String, TranslationNode>),
}

/// Plural forms for a translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluralValue {
    /// Zero form (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub zero: Option<String>,

    /// One/singular form
    #[serde(skip_serializing_if = "Option::is_none")]
    pub one: Option<String>,

    /// Two form (for languages like Arabic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub two: Option<String>,

    /// Few form (for languages like Polish)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub few: Option<String>,

    /// Many form (for languages like Arabic)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub many: Option<String>,

    /// Other/default form (required)
    pub other: String,
}

impl PluralValue {
    /// Get the appropriate plural form for a count
    pub fn get_form(&self, category: &str) -> Option<&str> {
        match category {
            "zero" => self.zero.as_deref().or(Some(&self.other)),
            "one" => self.one.as_deref().or(Some(&self.other)),
            "two" => self.two.as_deref().or(Some(&self.other)),
            "few" => self.few.as_deref().or(Some(&self.other)),
            "many" => self.many.as_deref().or(Some(&self.other)),
            "other" | _ => Some(&self.other),
        }
    }
}

/// A translation value with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationWithMeta {
    /// The actual translation value
    pub value: String,

    /// Note for translators
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,

    /// Context about where this translation is used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,

    /// Whether this key is locked (requires review to change)
    #[serde(default, skip_serializing_if = "is_false")]
    pub locked: bool,

    /// Maximum character length (for UI constraints)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// A resolved translation value
#[derive(Debug, Clone)]
pub enum TranslationValue {
    /// Simple string
    Simple(String),

    /// Plural forms
    Plural(PluralValue),
}

impl TranslationValue {
    /// Get the string value, optionally with plural category
    pub fn get(&self, plural_category: Option<&str>) -> Option<&str> {
        match self {
            Self::Simple(s) => Some(s.as_str()),
            Self::Plural(p) => p.get_form(plural_category.unwrap_or("other")),
        }
    }

    /// Get an owned copy of the string value
    pub fn get_owned(&self, plural_category: Option<&str>) -> Option<String> {
        self.get(plural_category).map(|s| s.to_string())
    }
}

impl TranslationFile {
    /// Load a translation file from disk
    pub fn load(path: impl AsRef<Path>) -> I18nResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(I18nError::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        let content = fs::read_to_string(path)?;
        Self::parse(&content, path)
    }

    /// Parse translation content from a string
    pub fn parse(content: &str, path: impl AsRef<Path>) -> I18nResult<Self> {
        let path = path.as_ref();

        toml::from_str(content).map_err(|e| I18nError::ParseError {
            path: path.to_path_buf(),
            message: e.to_string(),
        })
    }

    /// Get a translation value by dotted key path
    pub fn get(&self, key: &str) -> Option<TranslationValue> {
        let parts: Vec<&str> = key.split('.').collect();
        self.get_nested(&parts, &self.translations)
    }

    /// Recursively get a nested value
    fn get_nested(
        &self,
        parts: &[&str],
        current: &HashMap<String, TranslationNode>,
    ) -> Option<TranslationValue> {
        if parts.is_empty() {
            return None;
        }

        let key = parts[0];
        let node = current.get(key)?;

        if parts.len() == 1 {
            // Final key - return the value
            match node {
                TranslationNode::Value(s) => Some(TranslationValue::Simple(s.clone())),
                TranslationNode::Plural(p) => Some(TranslationValue::Plural(p.clone())),
                TranslationNode::WithMeta(m) => Some(TranslationValue::Simple(m.value.clone())),
                TranslationNode::Namespace(_) => None,
            }
        } else {
            // Continue traversing
            match node {
                TranslationNode::Namespace(nested) => self.get_nested(&parts[1..], nested),
                _ => None,
            }
        }
    }

    /// Get all keys in this file (flattened)
    pub fn keys(&self) -> Vec<String> {
        let mut keys = Vec::new();
        self.collect_keys(&self.translations, String::new(), &mut keys);
        keys
    }

    /// Recursively collect all keys
    fn collect_keys(
        &self,
        current: &HashMap<String, TranslationNode>,
        prefix: String,
        keys: &mut Vec<String>,
    ) {
        for (key, node) in current {
            let full_key = if prefix.is_empty() {
                key.clone()
            } else {
                format!("{}.{}", prefix, key)
            };

            match node {
                TranslationNode::Namespace(nested) => {
                    self.collect_keys(nested, full_key, keys);
                }
                _ => {
                    keys.push(full_key);
                }
            }
        }
    }

    /// Save the translation file to disk
    pub fn save(&self, path: impl AsRef<Path>) -> I18nResult<()> {
        let content = toml::to_string_pretty(self).map_err(|e| {
            I18nError::with_context("Failed to serialize translation file", e.to_string())
        })?;

        fs::write(path, content)?;
        Ok(())
    }

    /// Create a new empty translation file
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a translation value at a key path
    pub fn set(&mut self, key: &str, value: impl Into<String>) {
        let parts: Vec<&str> = key.split('.').collect();
        self.set_nested(&parts, TranslationNode::Value(value.into()));
    }

    /// Set a nested value
    fn set_nested(&mut self, parts: &[&str], value: TranslationNode) {
        if parts.is_empty() {
            return;
        }

        let mut current = &mut self.translations;

        for (i, key) in parts.iter().enumerate() {
            if i == parts.len() - 1 {
                // Final key - set the value
                current.insert((*key).to_string(), value);
                return;
            }

            // Ensure intermediate namespace exists
            current
                .entry((*key).to_string())
                .or_insert_with(|| TranslationNode::Namespace(HashMap::new()));

            if let TranslationNode::Namespace(ref mut nested) = current.get_mut(*key).unwrap() {
                current = nested;
            } else {
                // Can't traverse into non-namespace node
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let content = r#"
[auth]
login = "Sign in"
logout = "Sign out"

[auth.errors]
invalid = "Invalid credentials"
"#;

        let file = TranslationFile::parse(content, "test.toml").unwrap();

        assert_eq!(
            file.get("auth.login").and_then(|v| v.get_owned(None)),
            Some("Sign in".to_string())
        );
        assert_eq!(
            file.get("auth.logout").and_then(|v| v.get_owned(None)),
            Some("Sign out".to_string())
        );
        assert_eq!(
            file.get("auth.errors.invalid").and_then(|v| v.get_owned(None)),
            Some("Invalid credentials".to_string())
        );
    }

    #[test]
    fn test_parse_plural() {
        let content = r#"
[cart]
items = { one = "{count} item", other = "{count} items" }
"#;

        let file = TranslationFile::parse(content, "test.toml").unwrap();

        let value = file.get("cart.items").unwrap();
        assert_eq!(value.get(Some("one")), Some("{count} item"));
        assert_eq!(value.get(Some("other")), Some("{count} items"));
    }

    #[test]
    fn test_keys() {
        let content = r#"
[auth]
login = "Sign in"
logout = "Sign out"

[auth.errors]
invalid = "Invalid credentials"

[home]
title = "Home"
"#;

        let file = TranslationFile::parse(content, "test.toml").unwrap();
        let keys = file.keys();

        assert!(keys.contains(&"auth.login".to_string()));
        assert!(keys.contains(&"auth.logout".to_string()));
        assert!(keys.contains(&"auth.errors.invalid".to_string()));
        assert!(keys.contains(&"home.title".to_string()));
    }

    #[test]
    fn test_set_value() {
        let mut file = TranslationFile::new();
        file.set("auth.login.title", "Sign In");
        file.set("auth.login.button", "Continue");

        assert_eq!(
            file.get("auth.login.title").and_then(|v| v.get_owned(None)),
            Some("Sign In".to_string())
        );
        assert_eq!(
            file.get("auth.login.button").and_then(|v| v.get_owned(None)),
            Some("Continue".to_string())
        );
    }
}
