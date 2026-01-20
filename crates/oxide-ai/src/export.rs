//! AI Schema Export
//!
//! Export OxideKit schemas for AI consumption.

use serde::{Deserialize, Serialize};
use std::path::Path;

/// AI schema exporter
#[derive(Debug, Clone)]
pub struct AiExporter {
    options: ExportOptions,
}

impl AiExporter {
    /// Create a new exporter with default options
    pub fn new() -> Self {
        Self {
            options: ExportOptions::default(),
        }
    }

    /// Create with custom options
    pub fn with_options(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Export schema to JSON string
    pub fn export_json<T: Serialize>(&self, data: &T) -> Result<String, serde_json::Error> {
        if self.options.pretty {
            serde_json::to_string_pretty(data)
        } else {
            serde_json::to_string(data)
        }
    }

    /// Export schema to file
    pub fn export_to_file<T: Serialize>(&self, data: &T, path: &Path) -> Result<(), ExportError> {
        let json = self.export_json(data).map_err(ExportError::Serialization)?;
        std::fs::write(path, json).map_err(ExportError::Io)?;
        Ok(())
    }

    /// Export as oxide.ai.json
    pub fn export_ai_schema<T: Serialize>(&self, data: &T, project_root: &Path) -> Result<(), ExportError> {
        let path = project_root.join("oxide.ai.json");
        self.export_to_file(data, &path)
    }
}

impl Default for AiExporter {
    fn default() -> Self {
        Self::new()
    }
}

/// Export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Pretty-print JSON
    pub pretty: bool,
    /// Include internal components
    pub include_internal: bool,
    /// Include deprecated items
    pub include_deprecated: bool,
    /// Include examples
    pub include_examples: bool,
    /// Schema version to export
    pub schema_version: String,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            include_internal: false,
            include_deprecated: false,
            include_examples: true,
            schema_version: "1.0".to_string(),
        }
    }
}

/// Export error
#[derive(Debug)]
pub enum ExportError {
    Serialization(serde_json::Error),
    Io(std::io::Error),
}

impl std::fmt::Display for ExportError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Serialization(e) => write!(f, "Serialization error: {}", e),
            Self::Io(e) => write!(f, "IO error: {}", e),
        }
    }
}

impl std::error::Error for ExportError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_json() {
        let exporter = AiExporter::new();
        let data = serde_json::json!({"test": true});
        let json = exporter.export_json(&data).unwrap();
        assert!(json.contains("test"));
    }
}
