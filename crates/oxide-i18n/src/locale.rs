//! Locale handling and registry
//!
//! Provides locale parsing, validation, and fallback chain computation.

use crate::error::{I18nError, I18nResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// A parsed locale identifier following BCP 47 conventions
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Locale {
    /// Language code (e.g., "en", "nl", "zh")
    pub language: String,

    /// Optional region code (e.g., "US", "NL", "TW")
    pub region: Option<String>,

    /// Optional script code (e.g., "Hans", "Hant")
    pub script: Option<String>,
}

impl Locale {
    /// Parse a locale string (e.g., "en-US", "zh-Hant-TW")
    pub fn parse(locale: &str) -> I18nResult<Self> {
        let locale = locale.trim();

        if locale.is_empty() {
            return Err(I18nError::InvalidLocale {
                locale: locale.to_string(),
            });
        }

        // Split by hyphen or underscore
        let parts: Vec<&str> = locale.split(|c| c == '-' || c == '_').collect();

        if parts.is_empty() || parts[0].is_empty() {
            return Err(I18nError::InvalidLocale {
                locale: locale.to_string(),
            });
        }

        let language = parts[0].to_lowercase();

        // Validate language code (2-3 characters)
        if language.len() < 2 || language.len() > 3 || !language.chars().all(|c| c.is_ascii_alphabetic()) {
            return Err(I18nError::InvalidLocale {
                locale: locale.to_string(),
            });
        }

        let mut script = None;
        let mut region = None;

        for part in parts.iter().skip(1) {
            let part_len = part.len();

            if part_len == 4 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                // Script code (4 letters)
                script = Some(part[0..1].to_uppercase() + &part[1..].to_lowercase());
            } else if part_len == 2 && part.chars().all(|c| c.is_ascii_alphabetic()) {
                // Region code (2 letters)
                region = Some(part.to_uppercase());
            } else if part_len == 3 && part.chars().all(|c| c.is_ascii_digit()) {
                // Numeric region code (3 digits) - treat as region
                region = Some(part.to_string());
            }
        }

        Ok(Self {
            language,
            region,
            script,
        })
    }

    /// Create a new locale from components
    pub fn new(language: impl Into<String>) -> Self {
        Self {
            language: language.into().to_lowercase(),
            region: None,
            script: None,
        }
    }

    /// Add a region to this locale
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into().to_uppercase());
        self
    }

    /// Add a script to this locale
    pub fn with_script(mut self, script: impl Into<String>) -> Self {
        let s = script.into();
        self.script = Some(s[0..1].to_uppercase() + &s[1..].to_lowercase());
        self
    }

    /// Get the normalized string representation
    pub fn to_string_normalized(&self) -> String {
        let mut result = self.language.clone();

        if let Some(ref script) = self.script {
            result.push('-');
            result.push_str(script);
        }

        if let Some(ref region) = self.region {
            result.push('-');
            result.push_str(region);
        }

        result
    }

    /// Compute the fallback chain for this locale
    ///
    /// For "nl-NL", returns ["nl-NL", "nl"]
    /// For "zh-Hant-TW", returns ["zh-Hant-TW", "zh-Hant", "zh"]
    pub fn fallback_chain(&self) -> Vec<String> {
        let mut chain = Vec::new();

        // Full locale
        chain.push(self.to_string_normalized());

        // Language + Script (if both present)
        if self.script.is_some() && self.region.is_some() {
            let mut partial = self.language.clone();
            if let Some(ref script) = self.script {
                partial.push('-');
                partial.push_str(script);
            }
            chain.push(partial);
        }

        // Language only
        if self.region.is_some() || self.script.is_some() {
            chain.push(self.language.clone());
        }

        chain
    }

    /// Check if this locale is RTL
    pub fn is_rtl(&self) -> bool {
        matches!(
            self.language.as_str(),
            "ar" | "he" | "fa" | "ur" | "yi" | "ps" | "sd" | "ug"
        )
    }
}

impl fmt::Display for Locale {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string_normalized())
    }
}

impl Serialize for Locale {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string_normalized())
    }
}

impl<'de> Deserialize<'de> for Locale {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Locale::parse(&s).map_err(serde::de::Error::custom)
    }
}

/// Registry of available locales and their metadata
#[derive(Debug, Clone, Default)]
pub struct LocaleRegistry {
    /// Available locales mapped to their file paths
    locales: HashMap<String, LocaleInfo>,

    /// Default locale (fallback of last resort)
    default_locale: Option<String>,

    /// In-progress locales (warn instead of fail)
    in_progress: Vec<String>,
}

/// Information about a loaded locale
#[derive(Debug, Clone)]
pub struct LocaleInfo {
    /// The parsed locale
    pub locale: Locale,

    /// Path to the translation file
    pub path: std::path::PathBuf,

    /// Number of translation keys
    pub key_count: usize,

    /// Completion percentage (compared to default locale)
    pub completion: f32,
}

impl LocaleRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a locale
    pub fn register(&mut self, locale: Locale, path: std::path::PathBuf, key_count: usize) {
        let key = locale.to_string_normalized();
        self.locales.insert(
            key,
            LocaleInfo {
                locale,
                path,
                key_count,
                completion: 100.0,
            },
        );
    }

    /// Set the default locale
    pub fn set_default(&mut self, locale: &str) -> I18nResult<()> {
        let parsed = Locale::parse(locale)?;
        self.default_locale = Some(parsed.to_string_normalized());
        Ok(())
    }

    /// Get the default locale
    pub fn default_locale(&self) -> Option<&str> {
        self.default_locale.as_deref()
    }

    /// Mark a locale as in-progress
    pub fn mark_in_progress(&mut self, locale: &str) {
        if !self.in_progress.contains(&locale.to_string()) {
            self.in_progress.push(locale.to_string());
        }
    }

    /// Check if a locale is in-progress
    pub fn is_in_progress(&self, locale: &str) -> bool {
        self.in_progress.contains(&locale.to_string())
    }

    /// Get all registered locales
    pub fn locales(&self) -> impl Iterator<Item = &LocaleInfo> {
        self.locales.values()
    }

    /// Get info for a specific locale
    pub fn get(&self, locale: &str) -> Option<&LocaleInfo> {
        self.locales.get(locale)
    }

    /// Check if a locale is available
    pub fn has_locale(&self, locale: &str) -> bool {
        self.locales.contains_key(locale)
    }

    /// Update completion percentages based on default locale
    pub fn update_completion(&mut self) {
        let default_count = self
            .default_locale
            .as_ref()
            .and_then(|d| self.locales.get(d))
            .map(|l| l.key_count)
            .unwrap_or(0);

        if default_count == 0 {
            return;
        }

        for info in self.locales.values_mut() {
            info.completion = (info.key_count as f32 / default_count as f32) * 100.0;
        }
    }

    /// Find the best matching locale for a requested locale
    pub fn resolve(&self, requested: &str) -> Option<String> {
        let locale = Locale::parse(requested).ok()?;

        for candidate in locale.fallback_chain() {
            if self.locales.contains_key(&candidate) {
                return Some(candidate);
            }
        }

        // Fall back to default locale
        self.default_locale.clone()
    }
}

/// Detect the system locale
pub fn detect_system_locale() -> Option<Locale> {
    // Try environment variables in order of preference
    for var in ["LC_ALL", "LC_MESSAGES", "LANG", "LANGUAGE"] {
        if let Ok(value) = std::env::var(var) {
            // Strip encoding suffix (e.g., .UTF-8)
            let locale_str = value.split('.').next().unwrap_or(&value);
            if let Ok(locale) = Locale::parse(locale_str) {
                return Some(locale);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple() {
        let locale = Locale::parse("en").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, None);
        assert_eq!(locale.script, None);
    }

    #[test]
    fn test_parse_with_region() {
        let locale = Locale::parse("en-US").unwrap();
        assert_eq!(locale.language, "en");
        assert_eq!(locale.region, Some("US".to_string()));

        let locale = Locale::parse("nl_NL").unwrap();
        assert_eq!(locale.language, "nl");
        assert_eq!(locale.region, Some("NL".to_string()));
    }

    #[test]
    fn test_parse_with_script() {
        let locale = Locale::parse("zh-Hant-TW").unwrap();
        assert_eq!(locale.language, "zh");
        assert_eq!(locale.script, Some("Hant".to_string()));
        assert_eq!(locale.region, Some("TW".to_string()));
    }

    #[test]
    fn test_fallback_chain() {
        let locale = Locale::parse("zh-Hant-TW").unwrap();
        let chain = locale.fallback_chain();
        assert_eq!(chain, vec!["zh-Hant-TW", "zh-Hant", "zh"]);

        let locale = Locale::parse("nl-NL").unwrap();
        let chain = locale.fallback_chain();
        assert_eq!(chain, vec!["nl-NL", "nl"]);

        let locale = Locale::parse("en").unwrap();
        let chain = locale.fallback_chain();
        assert_eq!(chain, vec!["en"]);
    }

    #[test]
    fn test_is_rtl() {
        assert!(Locale::parse("ar").unwrap().is_rtl());
        assert!(Locale::parse("he-IL").unwrap().is_rtl());
        assert!(!Locale::parse("en-US").unwrap().is_rtl());
        assert!(!Locale::parse("zh-CN").unwrap().is_rtl());
    }

    #[test]
    fn test_registry_resolve() {
        let mut registry = LocaleRegistry::new();
        registry.register(
            Locale::parse("en").unwrap(),
            "i18n/en.toml".into(),
            100,
        );
        registry.register(
            Locale::parse("nl").unwrap(),
            "i18n/nl.toml".into(),
            90,
        );
        registry.set_default("en").unwrap();

        // Exact match
        assert_eq!(registry.resolve("en"), Some("en".to_string()));
        assert_eq!(registry.resolve("nl"), Some("nl".to_string()));

        // Fallback from region-specific to language
        assert_eq!(registry.resolve("nl-NL"), Some("nl".to_string()));
        assert_eq!(registry.resolve("en-US"), Some("en".to_string()));

        // Fallback to default
        assert_eq!(registry.resolve("fr"), Some("en".to_string()));
    }
}
