//! OxideKit Client SDK Generation
//!
//! Generates typed client SDKs from OpenAPI specifications:
//! - Rust clients for OxideKit apps
//! - TypeScript clients for web apps
//! - Contract linting and breaking change detection
//!
//! # Philosophy
//!
//! - Clients are generated from OpenAPI specs, never hand-written
//! - Version-pinned and typed
//! - No manual editing allowed
//! - Breaking changes are detected and blocked
//!
//! # Example
//!
//! ```ignore
//! use oxide_client_gen::{RustClientGenerator, TypeScriptClientGenerator};
//!
//! // Generate Rust client
//! let rust_gen = RustClientGenerator::new("my-api-client");
//! rust_gen.generate_from_file("openapi.json", "generated/")?;
//!
//! // Generate TypeScript client
//! let ts_gen = TypeScriptClientGenerator::new("my-api-client");
//! ts_gen.generate_from_file("openapi.json", "generated/")?;
//! ```

pub mod contract_lint;
pub mod rust_client;
pub mod typescript_client;

pub use contract_lint::{BreakingChange, ContractDiff, ContractLinter, LintResult, LintRule};
pub use rust_client::RustClientGenerator;
pub use typescript_client::TypeScriptClientGenerator;

/// Error types for client generation
#[derive(Debug, thiserror::Error)]
pub enum ClientGenError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// OpenAPI parsing error
    #[error("OpenAPI parsing error: {0}")]
    OpenApiParse(String),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// Code generation error
    #[error("Code generation error: {0}")]
    CodeGen(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// Breaking change detected
    #[error("Breaking change detected: {0}")]
    BreakingChange(String),
}

/// Result type alias for client generation operations
pub type Result<T> = std::result::Result<T, ClientGenError>;

/// Common configuration for client generators
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientConfig {
    /// Name of the generated client package
    pub package_name: String,
    /// Version of the generated client
    pub version: String,
    /// Description
    pub description: String,
    /// Base URL for the API (can be overridden at runtime)
    pub base_url: Option<String>,
    /// Generate async methods (where applicable)
    pub async_methods: bool,
    /// Include request/response logging
    pub include_logging: bool,
}

impl ClientConfig {
    /// Create a new client configuration
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            package_name: package_name.into(),
            version: "0.1.0".to_string(),
            description: String::new(),
            base_url: None,
            async_methods: true,
            include_logging: false,
        }
    }

    /// Set the version
    pub fn with_version(mut self, version: impl Into<String>) -> Self {
        self.version = version.into();
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the base URL
    pub fn with_base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }
}

/// Trait for client generators
pub trait ClientGenerator {
    /// Generate client code from an OpenAPI spec
    fn generate(&self, spec: &openapiv3::OpenAPI) -> Result<GeneratedClient>;

    /// Generate client from a JSON file
    fn generate_from_file(&self, spec_path: impl AsRef<std::path::Path>) -> Result<GeneratedClient> {
        let content = std::fs::read_to_string(spec_path.as_ref())?;
        let spec: openapiv3::OpenAPI =
            serde_json::from_str(&content).map_err(|e| ClientGenError::OpenApiParse(e.to_string()))?;
        self.generate(&spec)
    }

    /// Write generated client to a directory
    fn write_to_dir(
        &self,
        client: &GeneratedClient,
        output_dir: impl AsRef<std::path::Path>,
    ) -> Result<()> {
        let output = output_dir.as_ref();
        std::fs::create_dir_all(output)?;

        for file in &client.files {
            let path = output.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &file.content)?;
        }

        Ok(())
    }
}

/// A generated client package
#[derive(Debug, Clone)]
pub struct GeneratedClient {
    /// Package name
    pub name: String,
    /// Generated files
    pub files: Vec<GeneratedFile>,
    /// Target language
    pub language: ClientLanguage,
}

/// A generated source file
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// File path relative to output directory
    pub path: String,
    /// File content
    pub content: String,
}

/// Supported client languages
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClientLanguage {
    /// Rust client
    Rust,
    /// TypeScript client
    TypeScript,
    /// Swift client (future)
    Swift,
    /// Kotlin client (future)
    Kotlin,
}

impl ClientLanguage {
    /// Get the file extension for this language
    pub fn extension(&self) -> &'static str {
        match self {
            ClientLanguage::Rust => "rs",
            ClientLanguage::TypeScript => "ts",
            ClientLanguage::Swift => "swift",
            ClientLanguage::Kotlin => "kt",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_creation() {
        let config = ClientConfig::new("my-api-client");
        assert_eq!(config.package_name, "my-api-client");
        assert_eq!(config.version, "0.1.0");
        assert!(config.async_methods);
    }

    #[test]
    fn test_client_config_builder() {
        let config = ClientConfig::new("my-api-client")
            .with_version("1.0.0")
            .with_description("My API client")
            .with_base_url("https://api.example.com");

        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.description, "My API client");
        assert_eq!(config.base_url, Some("https://api.example.com".to_string()));
    }

    #[test]
    fn test_client_language_extension() {
        assert_eq!(ClientLanguage::Rust.extension(), "rs");
        assert_eq!(ClientLanguage::TypeScript.extension(), "ts");
    }
}
