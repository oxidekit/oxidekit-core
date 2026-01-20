//! I18n runtime - the main entry point for translations
//!
//! Provides locale management, translation lookup, and interpolation.

use crate::error::{I18nError, I18nResult};
use crate::format::{TranslationFile, TranslationValue};
use crate::locale::{detect_system_locale, Locale, LocaleRegistry};
use crate::pluralization::{get_plural_rules, select_plural, PluralRules};
use crate::rtl::{Direction, RtlSupport};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

/// Global i18n instance for the `t!` macro
pub static GLOBAL_I18N: Lazy<RwLock<Option<Arc<I18n>>>> = Lazy::new(|| RwLock::new(None));

/// Initialize the global i18n instance
pub fn init_global(i18n: I18n) {
    let mut global = GLOBAL_I18N.write().unwrap();
    *global = Some(Arc::new(i18n));
}

/// Get the global i18n instance
pub fn global() -> Option<Arc<I18n>> {
    GLOBAL_I18N.read().unwrap().clone()
}

/// The main i18n instance
pub struct I18n {
    /// Loaded translations per locale
    translations: HashMap<String, TranslationFile>,

    /// Locale registry
    registry: LocaleRegistry,

    /// Currently active locale
    current_locale: RwLock<String>,

    /// Plural rules cache (not Debug because trait objects don't impl Debug)
    plural_rules: RwLock<HashMap<String, Box<dyn PluralRules + Send + Sync>>>,

    /// RTL support
    rtl: RwLock<RtlSupport>,

    /// Configuration
    config: I18nConfig,

    /// Directory containing translation files
    translations_dir: PathBuf,
}

impl std::fmt::Debug for I18n {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("I18n")
            .field("translations", &self.translations)
            .field("registry", &self.registry)
            .field("current_locale", &self.current_locale)
            .field("plural_rules", &"<plural rules cache>")
            .field("rtl", &self.rtl)
            .field("config", &self.config)
            .field("translations_dir", &self.translations_dir)
            .finish()
    }
}

/// I18n configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I18nConfig {
    /// Default locale (fallback of last resort)
    pub default_locale: String,

    /// Whether to show missing key warnings in dev mode
    #[serde(default = "default_true")]
    pub warn_missing_keys: bool,

    /// Whether to return key as fallback for missing translations
    #[serde(default = "default_true")]
    pub key_as_fallback: bool,

    /// In-progress locales (warn instead of fail)
    #[serde(default)]
    pub in_progress_locales: Vec<String>,

    /// Required locales (fail if missing keys)
    #[serde(default)]
    pub required_locales: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for I18nConfig {
    fn default() -> Self {
        Self {
            default_locale: "en".to_string(),
            warn_missing_keys: true,
            key_as_fallback: true,
            in_progress_locales: Vec::new(),
            required_locales: Vec::new(),
        }
    }
}

impl I18n {
    /// Create a new i18n instance and load translations from a directory
    pub fn load(path: impl AsRef<Path>) -> I18nResult<Self> {
        Self::load_with_config(path, I18nConfig::default())
    }

    /// Create a new i18n instance with custom configuration
    pub fn load_with_config(path: impl AsRef<Path>, config: I18nConfig) -> I18nResult<Self> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(I18nError::FileNotFound {
                path: path.to_path_buf(),
            });
        }

        let mut translations = HashMap::new();
        let mut registry = LocaleRegistry::new();

        // Find all .toml translation files
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if file_path.extension().map(|e| e == "toml").unwrap_or(false) {
                let file_name = file_path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or_default();

                // Parse locale from filename
                if let Ok(locale) = Locale::parse(file_name) {
                    tracing::debug!("Loading translation file: {:?}", file_path);

                    let file = TranslationFile::load(&file_path)?;
                    let key_count = file.keys().len();

                    let locale_key = locale.to_string_normalized();
                    registry.register(locale.clone(), file_path.clone(), key_count);
                    translations.insert(locale_key, file);
                }
            }
        }

        // Set default locale
        registry.set_default(&config.default_locale)?;

        // Mark in-progress locales
        for locale in &config.in_progress_locales {
            registry.mark_in_progress(locale);
        }

        // Update completion percentages
        registry.update_completion();

        // Determine initial locale
        let current_locale = detect_system_locale()
            .map(|l| l.to_string_normalized())
            .and_then(|l| registry.resolve(&l))
            .unwrap_or_else(|| config.default_locale.clone());

        let rtl = RtlSupport::for_language(&current_locale);

        Ok(Self {
            translations,
            registry,
            current_locale: RwLock::new(current_locale),
            plural_rules: RwLock::new(HashMap::new()),
            rtl: RwLock::new(rtl),
            config,
            translations_dir: path.to_path_buf(),
        })
    }

    /// Get the current locale
    pub fn locale(&self) -> String {
        self.current_locale.read().unwrap().clone()
    }

    /// Set the current locale
    pub fn set_locale(&self, locale: &str) -> I18nResult<()> {
        let resolved = self.registry.resolve(locale).ok_or_else(|| {
            I18nError::LocaleNotLoaded {
                locale: locale.to_string(),
            }
        })?;

        let mut current = self.current_locale.write().unwrap();
        *current = resolved.clone();

        // Update RTL support
        let mut rtl = self.rtl.write().unwrap();
        *rtl = RtlSupport::for_language(&resolved);

        Ok(())
    }

    /// Get available locales
    pub fn available_locales(&self) -> Vec<String> {
        self.registry.locales().map(|l| l.locale.to_string()).collect()
    }

    /// Get the text direction for the current locale
    pub fn direction(&self) -> Direction {
        self.rtl.read().unwrap().direction()
    }

    /// Check if current locale is RTL
    pub fn is_rtl(&self) -> bool {
        self.rtl.read().unwrap().is_rtl()
    }

    /// Get RTL support utilities
    pub fn rtl(&self) -> RtlSupport {
        self.rtl.read().unwrap().clone()
    }

    /// Translate a key with optional parameters
    pub fn t(&self, key: &str) -> String {
        self.translate(key, &HashMap::new(), None)
    }

    /// Translate a key with parameters
    pub fn t_with_params(&self, key: &str, params: &HashMap<String, String>) -> String {
        self.translate(key, params, None)
    }

    /// Translate a key with a count for pluralization
    pub fn t_plural(&self, key: &str, count: i64) -> String {
        let mut params = HashMap::new();
        params.insert("count".to_string(), count.to_string());
        self.translate(key, &params, Some(count))
    }

    /// Translate a key with parameters and pluralization
    pub fn t_plural_with_params(
        &self,
        key: &str,
        count: i64,
        params: &HashMap<String, String>,
    ) -> String {
        let mut full_params = params.clone();
        full_params.insert("count".to_string(), count.to_string());
        self.translate(key, &full_params, Some(count))
    }

    /// Internal translation lookup
    fn translate(
        &self,
        key: &str,
        params: &HashMap<String, String>,
        count: Option<i64>,
    ) -> String {
        let locale = self.current_locale.read().unwrap().clone();
        let locale_obj = Locale::parse(&locale).unwrap_or_else(|_| Locale::new("en"));

        // Try fallback chain
        for candidate in locale_obj.fallback_chain() {
            if let Some(file) = self.translations.get(&candidate) {
                if let Some(value) = file.get(key) {
                    let text = self.resolve_value(&value, &candidate, count);
                    return self.interpolate(&text, params);
                }
            }
        }

        // Try default locale
        if let Some(file) = self.translations.get(&self.config.default_locale) {
            if let Some(value) = file.get(key) {
                let text = self.resolve_value(&value, &self.config.default_locale, count);
                return self.interpolate(&text, params);
            }
        }

        // Key not found
        if self.config.warn_missing_keys {
            tracing::warn!("Missing translation key: {} for locale: {}", key, locale);
        }

        if self.config.key_as_fallback {
            key.to_string()
        } else {
            format!("[missing: {}]", key)
        }
    }

    /// Resolve a translation value, handling pluralization
    fn resolve_value(&self, value: &TranslationValue, locale: &str, count: Option<i64>) -> String {
        match value {
            TranslationValue::Simple(s) => s.clone(),
            TranslationValue::Plural(p) => {
                let category = count
                    .map(|n| {
                        let rules = self.get_plural_rules(locale);
                        select_plural(rules.as_ref(), n as f64).as_str()
                    })
                    .unwrap_or("other");

                p.get_form(category).unwrap_or(&p.other).to_string()
            }
        }
    }

    /// Get plural rules for a locale (cached)
    fn get_plural_rules(&self, locale: &str) -> Box<dyn PluralRules + Send + Sync> {
        let mut cache = self.plural_rules.write().unwrap();

        if let Some(rules) = cache.get(locale) {
            // Return a clone (we need to implement Clone for the trait objects)
            // For now, just create a new one
        }

        let lang = locale.split('-').next().unwrap_or(locale);
        let rules = get_plural_rules(lang);
        cache.insert(locale.to_string(), get_plural_rules(lang));
        rules
    }

    /// Interpolate parameters into a string
    fn interpolate(&self, text: &str, params: &HashMap<String, String>) -> String {
        let mut result = text.to_string();

        for (key, value) in params {
            let placeholder = format!("{{{}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Get the locale registry
    pub fn registry(&self) -> &LocaleRegistry {
        &self.registry
    }

    /// Get the configuration
    pub fn config(&self) -> &I18nConfig {
        &self.config
    }

    /// Get all translation keys across all loaded files
    pub fn all_keys(&self) -> Vec<String> {
        let mut keys = std::collections::HashSet::new();

        for file in self.translations.values() {
            for key in file.keys() {
                keys.insert(key);
            }
        }

        let mut result: Vec<_> = keys.into_iter().collect();
        result.sort();
        result
    }

    /// Get keys for a specific locale
    pub fn keys_for_locale(&self, locale: &str) -> Vec<String> {
        self.translations
            .get(locale)
            .map(|f| f.keys())
            .unwrap_or_default()
    }

    /// Check if a key exists in the default locale
    pub fn has_key(&self, key: &str) -> bool {
        self.translations
            .get(&self.config.default_locale)
            .map(|f| f.get(key).is_some())
            .unwrap_or(false)
    }

    /// Get the translations directory path
    pub fn translations_dir(&self) -> &Path {
        &self.translations_dir
    }

    /// Reload translations from disk
    pub fn reload(&mut self) -> I18nResult<()> {
        let new_i18n = Self::load_with_config(&self.translations_dir, self.config.clone())?;

        self.translations = new_i18n.translations;
        self.registry = new_i18n.registry;

        Ok(())
    }
}

/// Builder for constructing I18n instances
#[derive(Debug, Default)]
pub struct I18nBuilder {
    config: I18nConfig,
    translations_dir: Option<PathBuf>,
}

impl I18nBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the translations directory
    pub fn translations_dir(mut self, path: impl AsRef<Path>) -> Self {
        self.translations_dir = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set the default locale
    pub fn default_locale(mut self, locale: impl Into<String>) -> Self {
        self.config.default_locale = locale.into();
        self
    }

    /// Set whether to warn about missing keys
    pub fn warn_missing_keys(mut self, warn: bool) -> Self {
        self.config.warn_missing_keys = warn;
        self
    }

    /// Set whether to use key as fallback
    pub fn key_as_fallback(mut self, fallback: bool) -> Self {
        self.config.key_as_fallback = fallback;
        self
    }

    /// Add an in-progress locale
    pub fn in_progress_locale(mut self, locale: impl Into<String>) -> Self {
        self.config.in_progress_locales.push(locale.into());
        self
    }

    /// Add a required locale
    pub fn required_locale(mut self, locale: impl Into<String>) -> Self {
        self.config.required_locales.push(locale.into());
        self
    }

    /// Build the I18n instance
    pub fn build(self) -> I18nResult<I18n> {
        let dir = self.translations_dir.ok_or_else(|| {
            I18nError::with_context("I18nBuilder", "translations_dir is required")
        })?;

        I18n::load_with_config(dir, self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn setup_test_translations() -> (TempDir, PathBuf) {
        let dir = TempDir::new().unwrap();
        let i18n_dir = dir.path().join("i18n");
        fs::create_dir(&i18n_dir).unwrap();

        // Create English translations
        let en_content = r#"
[auth.login]
title = "Sign in"
button = "Continue"
welcome = "Welcome, {name}!"

[cart]
items = { one = "{count} item", other = "{count} items" }
empty = "Your cart is empty"
"#;
        let mut en_file = fs::File::create(i18n_dir.join("en.toml")).unwrap();
        en_file.write_all(en_content.as_bytes()).unwrap();

        // Create Dutch translations
        let nl_content = r#"
[auth.login]
title = "Inloggen"
button = "Doorgaan"
welcome = "Welkom, {name}!"

[cart]
items = { one = "{count} item", other = "{count} items" }
"#;
        let mut nl_file = fs::File::create(i18n_dir.join("nl.toml")).unwrap();
        nl_file.write_all(nl_content.as_bytes()).unwrap();

        (dir, i18n_dir)
    }

    #[test]
    fn test_load_translations() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        assert!(i18n.available_locales().contains(&"en".to_string()));
        assert!(i18n.available_locales().contains(&"nl".to_string()));
    }

    #[test]
    fn test_simple_translation() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        i18n.set_locale("en").unwrap();
        assert_eq!(i18n.t("auth.login.title"), "Sign in");

        i18n.set_locale("nl").unwrap();
        assert_eq!(i18n.t("auth.login.title"), "Inloggen");
    }

    #[test]
    fn test_interpolation() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        i18n.set_locale("en").unwrap();

        let mut params = HashMap::new();
        params.insert("name".to_string(), "Alice".to_string());

        assert_eq!(
            i18n.t_with_params("auth.login.welcome", &params),
            "Welcome, Alice!"
        );
    }

    #[test]
    fn test_pluralization() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        i18n.set_locale("en").unwrap();

        assert_eq!(i18n.t_plural("cart.items", 1), "1 item");
        assert_eq!(i18n.t_plural("cart.items", 5), "5 items");
        assert_eq!(i18n.t_plural("cart.items", 0), "0 items");
    }

    #[test]
    fn test_fallback() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        i18n.set_locale("nl").unwrap();

        // "cart.empty" is missing in Dutch, should fall back to English
        assert_eq!(i18n.t("cart.empty"), "Your cart is empty");
    }

    #[test]
    fn test_locale_fallback_chain() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        // nl-NL should fall back to nl
        i18n.set_locale("nl-NL").unwrap();
        assert_eq!(i18n.t("auth.login.title"), "Inloggen");
    }

    #[test]
    fn test_missing_key() {
        let (_dir, i18n_dir) = setup_test_translations();
        let i18n = I18n::load(&i18n_dir).unwrap();

        i18n.set_locale("en").unwrap();

        // Non-existent key returns the key itself (with default config)
        assert_eq!(i18n.t("nonexistent.key"), "nonexistent.key");
    }

    #[test]
    fn test_builder() {
        let (_dir, i18n_dir) = setup_test_translations();

        let i18n = I18nBuilder::new()
            .translations_dir(&i18n_dir)
            .default_locale("en")
            .warn_missing_keys(false)
            .build()
            .unwrap();

        assert_eq!(i18n.t("auth.login.title"), "Sign in");
    }
}
