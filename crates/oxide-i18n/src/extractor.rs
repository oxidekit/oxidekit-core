//! Translation key extraction from source files
//!
//! Scans .oui and .rs files to find all translation keys in use.

use crate::error::{I18nError, I18nResult};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

/// Extracted key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedKey {
    /// The translation key (e.g., "auth.login.title")
    pub key: String,

    /// Locations where this key is used
    pub locations: Vec<KeyLocation>,

    /// Parameters used with this key (e.g., ["name", "count"])
    pub params: HashSet<String>,

    /// Whether this key uses pluralization
    pub is_plural: bool,
}

/// Location of a key usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLocation {
    /// File path
    pub file: PathBuf,

    /// Line number (1-based)
    pub line: usize,

    /// Column number (1-based)
    pub column: usize,

    /// The source line containing the key
    pub context: String,
}

/// Key extraction report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionReport {
    /// All extracted keys
    pub keys: HashMap<String, ExtractedKey>,

    /// Files scanned
    pub files_scanned: Vec<PathBuf>,

    /// Errors encountered during extraction
    pub errors: Vec<ExtractionError>,

    /// Statistics
    pub stats: ExtractionStats,
}

/// Extraction statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExtractionStats {
    /// Total files scanned
    pub total_files: usize,

    /// .oui files scanned
    pub oui_files: usize,

    /// .rs files scanned
    pub rs_files: usize,

    /// Total unique keys found
    pub unique_keys: usize,

    /// Total key usages
    pub total_usages: usize,

    /// Keys with pluralization
    pub plural_keys: usize,
}

/// An error during extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionError {
    /// File where the error occurred
    pub file: PathBuf,

    /// Error message
    pub message: String,
}

/// Key extractor configuration
#[derive(Debug, Clone)]
pub struct ExtractorConfig {
    /// File extensions to scan
    pub extensions: Vec<String>,

    /// Directories to exclude
    pub exclude_dirs: Vec<String>,

    /// Custom patterns to match (in addition to defaults)
    pub custom_patterns: Vec<String>,
}

impl Default for ExtractorConfig {
    fn default() -> Self {
        Self {
            extensions: vec!["oui".to_string(), "rs".to_string()],
            exclude_dirs: vec![
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
            ],
            custom_patterns: Vec::new(),
        }
    }
}

/// Translation key extractor
pub struct KeyExtractor {
    config: ExtractorConfig,
    patterns: Vec<(Regex, PatternType)>,
}

/// Type of pattern match
#[derive(Debug, Clone, Copy)]
enum PatternType {
    /// t!("key") or t!("key", ...)
    TMacro,
    /// t_with!(i18n, "key") or t_with!(i18n, "key", ...)
    TMacroWith,
    /// t("key") in OUI files
    TFunction,
    /// value: t("key") in OUI files
    TFunctionValue,
}

impl KeyExtractor {
    /// Create a new extractor with default configuration
    pub fn new() -> I18nResult<Self> {
        Self::with_config(ExtractorConfig::default())
    }

    /// Create a new extractor with custom configuration
    pub fn with_config(config: ExtractorConfig) -> I18nResult<Self> {
        let mut patterns = Vec::new();

        // Rust t! macro: t!("key") or t!("key", name = value, ...)
        patterns.push((
            Regex::new(r#"t!\s*\(\s*"([^"]+)"(?:\s*,\s*([^)]+))?\s*\)"#)?,
            PatternType::TMacro,
        ));

        // Rust t_with! macro: t_with!(i18n, "key", ...)
        patterns.push((
            Regex::new(r#"t_with!\s*\(\s*[^,]+,\s*"([^"]+)"(?:\s*,\s*([^)]+))?\s*\)"#)?,
            PatternType::TMacroWith,
        ));

        // OUI t("key") function call
        patterns.push((
            Regex::new(r#"t\s*\(\s*"([^"]+)"(?:\s*,\s*\{([^}]*)\})?\s*\)"#)?,
            PatternType::TFunction,
        ));

        // OUI value: t("key") attribute
        patterns.push((
            Regex::new(r#"(?:text|content|value|label|placeholder|title)\s*:\s*t\s*\(\s*"([^"]+)"(?:\s*,\s*\{([^}]*)\})?\s*\)"#)?,
            PatternType::TFunctionValue,
        ));

        // Add custom patterns
        for pattern in &config.custom_patterns {
            patterns.push((Regex::new(pattern)?, PatternType::TMacro));
        }

        Ok(Self { config, patterns })
    }

    /// Extract keys from a directory
    pub fn extract(&self, dir: impl AsRef<Path>) -> I18nResult<ExtractionReport> {
        let dir = dir.as_ref();
        let mut report = ExtractionReport::default();

        if !dir.exists() {
            return Err(I18nError::FileNotFound {
                path: dir.to_path_buf(),
            });
        }

        for entry in WalkDir::new(dir)
            .follow_links(true)
            .into_iter()
            .filter_entry(|e| !self.should_exclude(e.path()))
        {
            let entry = match entry {
                Ok(e) => e,
                Err(e) => {
                    report.errors.push(ExtractionError {
                        file: dir.to_path_buf(),
                        message: e.to_string(),
                    });
                    continue;
                }
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let path = entry.path();
            let extension = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or_default();

            if !self.config.extensions.contains(&extension.to_string()) {
                continue;
            }

            // Extract from this file
            match self.extract_from_file(path) {
                Ok(keys) => {
                    report.files_scanned.push(path.to_path_buf());

                    match extension {
                        "oui" => report.stats.oui_files += 1,
                        "rs" => report.stats.rs_files += 1,
                        _ => {}
                    }

                    for extracted in keys {
                        report.stats.total_usages += extracted.locations.len();

                        if extracted.is_plural {
                            report.stats.plural_keys += 1;
                        }

                        // Merge with existing key info
                        report
                            .keys
                            .entry(extracted.key.clone())
                            .and_modify(|existing| {
                                existing.locations.extend(extracted.locations.clone());
                                existing.params.extend(extracted.params.clone());
                                existing.is_plural = existing.is_plural || extracted.is_plural;
                            })
                            .or_insert(extracted);
                    }
                }
                Err(e) => {
                    report.errors.push(ExtractionError {
                        file: path.to_path_buf(),
                        message: e.to_string(),
                    });
                }
            }
        }

        report.stats.total_files = report.files_scanned.len();
        report.stats.unique_keys = report.keys.len();

        Ok(report)
    }

    /// Extract keys from a single file
    pub fn extract_from_file(&self, path: impl AsRef<Path>) -> I18nResult<Vec<ExtractedKey>> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)?;
        self.extract_from_string(&content, path)
    }

    /// Extract keys from a string
    pub fn extract_from_string(
        &self,
        content: &str,
        source_path: &Path,
    ) -> I18nResult<Vec<ExtractedKey>> {
        let mut keys: HashMap<String, ExtractedKey> = HashMap::new();

        for (line_num, line) in content.lines().enumerate() {
            for (pattern, _pattern_type) in &self.patterns {
                for cap in pattern.captures_iter(line) {
                    let key = cap.get(1).map(|m| m.as_str()).unwrap_or_default();

                    if key.is_empty() {
                        continue;
                    }

                    // Extract parameters if present
                    let params_str = cap.get(2).map(|m| m.as_str()).unwrap_or_default();
                    let (params, is_plural) = self.parse_params(params_str);

                    let column = cap.get(1).map(|m| m.start() + 1).unwrap_or(1);

                    let location = KeyLocation {
                        file: source_path.to_path_buf(),
                        line: line_num + 1,
                        column,
                        context: line.trim().to_string(),
                    };

                    keys.entry(key.to_string())
                        .and_modify(|existing| {
                            existing.locations.push(location.clone());
                            existing.params.extend(params.clone());
                            existing.is_plural = existing.is_plural || is_plural;
                        })
                        .or_insert(ExtractedKey {
                            key: key.to_string(),
                            locations: vec![location],
                            params,
                            is_plural,
                        });
                }
            }
        }

        Ok(keys.into_values().collect())
    }

    /// Parse parameters from a capture group
    fn parse_params(&self, params_str: &str) -> (HashSet<String>, bool) {
        let mut params = HashSet::new();
        let mut is_plural = false;

        if params_str.is_empty() {
            return (params, is_plural);
        }

        // Match parameter names: name = value
        let param_regex = Regex::new(r"(\w+)\s*[=:]").unwrap();

        for cap in param_regex.captures_iter(params_str) {
            if let Some(name) = cap.get(1) {
                let name = name.as_str();
                params.insert(name.to_string());

                if name == "count" {
                    is_plural = true;
                }
            }
        }

        (params, is_plural)
    }

    /// Check if a path should be excluded
    fn should_exclude(&self, path: &Path) -> bool {
        for component in path.components() {
            if let std::path::Component::Normal(name) = component {
                let name = name.to_string_lossy();
                if self.config.exclude_dirs.iter().any(|d| d == name.as_ref()) {
                    return true;
                }
            }
        }
        false
    }
}

impl Default for KeyExtractor {
    fn default() -> Self {
        Self::new().expect("Default extractor creation should not fail")
    }
}

impl ExtractionReport {
    /// Get all unique keys sorted alphabetically
    pub fn sorted_keys(&self) -> Vec<&str> {
        let mut keys: Vec<_> = self.keys.keys().map(|s| s.as_str()).collect();
        keys.sort();
        keys
    }

    /// Save the report as JSON
    pub fn save_json(&self, path: impl AsRef<Path>) -> I18nResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// Generate a canonical keys file (for tracking)
    pub fn generate_keys_file(&self) -> String {
        let mut output = String::new();
        output.push_str("# Translation Keys\n");
        output.push_str("# Auto-generated by oxide i18n extract\n\n");

        for key in self.sorted_keys() {
            if let Some(info) = self.keys.get(key) {
                output.push_str(&format!("# Used {} time(s)\n", info.locations.len()));

                if !info.params.is_empty() {
                    let params: Vec<&str> = info.params.iter().map(|s| s.as_str()).collect();
                    output.push_str(&format!("# Parameters: {}\n", params.join(", ")));
                }

                if info.is_plural {
                    output.push_str("# Plural: yes\n");
                }

                output.push_str(&format!("{}\n\n", key));
            }
        }

        output
    }

    /// Convert to a JSON schema for AI/tooling
    pub fn to_ai_schema(&self) -> serde_json::Value {
        let keys_schema: Vec<_> = self
            .sorted_keys()
            .iter()
            .map(|key| {
                let info = self.keys.get(*key).unwrap();
                serde_json::json!({
                    "key": key,
                    "usages": info.locations.len(),
                    "params": info.params.iter().collect::<Vec<_>>(),
                    "plural": info.is_plural,
                    "locations": info.locations.iter().map(|l| {
                        serde_json::json!({
                            "file": l.file.to_string_lossy(),
                            "line": l.line,
                        })
                    }).collect::<Vec<_>>(),
                })
            })
            .collect();

        serde_json::json!({
            "version": "1.0",
            "total_keys": self.stats.unique_keys,
            "total_usages": self.stats.total_usages,
            "files_scanned": self.stats.total_files,
            "keys": keys_schema,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    #[test]
    fn test_extract_rust_t_macro() {
        let extractor = KeyExtractor::new().unwrap();

        let content = r#"
fn main() {
    let text = t!("auth.login.title");
    let welcome = t!("welcome.message", name = "Alice");
    let items = t!("cart.items", count = 5);
}
"#;

        let keys = extractor
            .extract_from_string(content, Path::new("test.rs"))
            .unwrap();

        assert_eq!(keys.len(), 3);

        let key_names: HashSet<_> = keys.iter().map(|k| k.key.as_str()).collect();
        assert!(key_names.contains("auth.login.title"));
        assert!(key_names.contains("welcome.message"));
        assert!(key_names.contains("cart.items"));

        // Check pluralization detection
        let items_key = keys.iter().find(|k| k.key == "cart.items").unwrap();
        assert!(items_key.is_plural);
        assert!(items_key.params.contains("count"));
    }

    #[test]
    fn test_extract_oui_t_function() {
        let extractor = KeyExtractor::new().unwrap();

        let content = r#"
app MyApp {
    Column {
        Text {
            content: t("auth.login.title")
        }
        Button {
            text: t("auth.login.button", { name: user.name })
        }
    }
}
"#;

        let keys = extractor
            .extract_from_string(content, Path::new("app.oui"))
            .unwrap();

        assert_eq!(keys.len(), 2);

        let key_names: HashSet<_> = keys.iter().map(|k| k.key.as_str()).collect();
        assert!(key_names.contains("auth.login.title"));
        assert!(key_names.contains("auth.login.button"));
    }

    #[test]
    fn test_extract_directory() {
        let dir = TempDir::new().unwrap();

        // Create a Rust file
        let rs_content = r#"
let text = t!("test.key1");
let text2 = t!("test.key2", name = "Test");
"#;
        let mut rs_file = fs::File::create(dir.path().join("test.rs")).unwrap();
        rs_file.write_all(rs_content.as_bytes()).unwrap();

        // Create an OUI file
        let oui_content = r#"
Text { content: t("test.key3") }
"#;
        let mut oui_file = fs::File::create(dir.path().join("test.oui")).unwrap();
        oui_file.write_all(oui_content.as_bytes()).unwrap();

        let extractor = KeyExtractor::new().unwrap();
        let report = extractor.extract(dir.path()).unwrap();

        assert_eq!(report.stats.unique_keys, 3);
        assert_eq!(report.stats.rs_files, 1);
        assert_eq!(report.stats.oui_files, 1);
    }

    #[test]
    fn test_exclude_dirs() {
        let dir = TempDir::new().unwrap();

        // Create a file in a target directory (should be excluded)
        fs::create_dir(dir.path().join("target")).unwrap();
        let rs_content = r#"let text = t!("excluded.key");"#;
        let mut rs_file = fs::File::create(dir.path().join("target/test.rs")).unwrap();
        rs_file.write_all(rs_content.as_bytes()).unwrap();

        // Create a file in the root (should be included)
        let main_content = r#"let text = t!("included.key");"#;
        let mut main_file = fs::File::create(dir.path().join("main.rs")).unwrap();
        main_file.write_all(main_content.as_bytes()).unwrap();

        let extractor = KeyExtractor::new().unwrap();
        let report = extractor.extract(dir.path()).unwrap();

        assert_eq!(report.stats.unique_keys, 1);
        assert!(report.keys.contains_key("included.key"));
        assert!(!report.keys.contains_key("excluded.key"));
    }
}
