//! Local Figma file loading
//!
//! Supports loading Figma files from local JSON exports.
//!
//! # Usage
//!
//! ```no_run
//! use oxide_figma::local::LocalLoader;
//!
//! # fn example() -> anyhow::Result<()> {
//! // Load from exported JSON
//! let file = LocalLoader::from_json_file("design.json")?;
//!
//! // Or from a JSON string
//! let json = std::fs::read_to_string("design.json")?;
//! let file = LocalLoader::from_json_str(&json)?;
//!
//! // Use with translator
//! use oxide_figma::Translator;
//! let translator = Translator::new();
//! let result = translator.translate(&file)?;
//! # Ok(())
//! # }
//! ```

use crate::error::{FigmaError, Result};
use crate::types::FigmaFile;
use std::path::Path;

/// Local Figma file loader
///
/// Provides methods to load Figma files from local sources:
/// - JSON files (exported from Figma API)
/// - JSON strings
pub struct LocalLoader;

impl LocalLoader {
    /// Load a Figma file from a JSON file
    ///
    /// The JSON should be in the format returned by the Figma API.
    /// You can obtain this by:
    /// 1. Using the Figma API to fetch a file
    /// 2. Saving the response JSON to a file
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxide_figma::local::LocalLoader;
    ///
    /// let file = LocalLoader::from_json_file("my-design.json").unwrap();
    /// println!("Loaded: {}", file.name);
    /// ```
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<FigmaFile> {
        let path = path.as_ref();
        let content = std::fs::read_to_string(path)?;

        Self::from_json_str(&content)
    }

    /// Load a Figma file from a JSON string
    ///
    /// # Example
    ///
    /// ```no_run
    /// use oxide_figma::local::LocalLoader;
    ///
    /// let json = r#"{"name": "My Design", "document": {...}}"#;
    /// let file = LocalLoader::from_json_str(json).unwrap();
    /// ```
    pub fn from_json_str(json: &str) -> Result<FigmaFile> {
        serde_json::from_str(json).map_err(|e| FigmaError::ParseError(e.to_string()))
    }

    /// Load a Figma file from a JSON value
    pub fn from_json_value(value: serde_json::Value) -> Result<FigmaFile> {
        serde_json::from_value(value).map_err(|e| FigmaError::ParseError(e.to_string()))
    }

    /// Check if a file path looks like a supported local Figma export
    ///
    /// Currently supports: `.json`
    pub fn is_supported_format<P: AsRef<Path>>(path: P) -> bool {
        let path = path.as_ref();
        if let Some(ext) = path.extension() {
            matches!(ext.to_str(), Some("json"))
        } else {
            false
        }
    }
}

/// Extension trait for FigmaFile to add local loading methods
impl FigmaFile {
    /// Load from a JSON file
    pub fn from_json_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        LocalLoader::from_json_file(path)
    }

    /// Load from a JSON string
    pub fn from_json_str(json: &str) -> Result<Self> {
        LocalLoader::from_json_str(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_supported_format() {
        assert!(LocalLoader::is_supported_format("design.json"));
        assert!(LocalLoader::is_supported_format("/path/to/design.json"));
        assert!(!LocalLoader::is_supported_format("design.fig"));
        assert!(!LocalLoader::is_supported_format("design.txt"));
    }

    #[test]
    fn test_parse_minimal_json() {
        let json = r#"{
            "name": "Test File",
            "document": {
                "id": "0:0",
                "name": "Document",
                "type": "DOCUMENT",
                "children": []
            },
            "components": {},
            "styles": {},
            "schemaVersion": 0
        }"#;

        let file = LocalLoader::from_json_str(json).unwrap();
        assert_eq!(file.name, "Test File");
        assert_eq!(file.document.name, "Document");
    }

    #[test]
    fn test_parse_error() {
        let invalid_json = "{ not valid json }";
        let result = LocalLoader::from_json_str(invalid_json);
        assert!(result.is_err());
    }
}
