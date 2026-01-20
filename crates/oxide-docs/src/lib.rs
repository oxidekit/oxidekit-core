//! OxideKit Offline Documentation & Learning System
//!
//! This crate provides comprehensive offline documentation support for OxideKit,
//! enabling teams to onboard without internet dependency.
//!
//! # Features
//!
//! - **Documentation Bundler**: Package docs into distributable offline bundles
//! - **Offline Viewer**: Browse documentation locally with a built-in server
//! - **Search Index**: Full-text search over all documentation
//! - **Interactive Tutorials**: Step-by-step learning guides with runnable examples
//! - **Code Documentation**: Generate docs from Rust code and .oxide files
//!
//! # Example
//!
//! ```no_run
//! use oxide_docs::{DocBundle, DocsConfig};
//!
//! // Load or create a documentation bundle
//! let config = DocsConfig::default();
//! let bundle = DocBundle::build(&config)?;
//!
//! // Start the offline viewer
//! bundle.serve(3030)?;
//! # Ok::<(), oxide_docs::DocsError>(())
//! ```

pub mod bundler;
pub mod codegen;
pub mod search;
pub mod tutorials;
#[cfg(feature = "viewer")]
pub mod viewer;

mod error;
mod manifest;
mod types;

pub use bundler::{DocBundle, DocBundler};
pub use codegen::{CodeDocGenerator, DocComment};
pub use error::{DocsError, DocsResult};
pub use manifest::DocsManifest;
pub use search::{DocIndex, SearchResult};
pub use tutorials::{Tutorial, TutorialRunner, TutorialStep};
pub use types::*;

/// Documentation configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DocsConfig {
    /// OxideKit version this docs bundle is for
    pub version: String,
    /// Root directory containing documentation sources
    pub source_dir: std::path::PathBuf,
    /// Output directory for the built bundle
    pub output_dir: std::path::PathBuf,
    /// Whether to include API reference docs
    pub include_api_docs: bool,
    /// Whether to include tutorials
    pub include_tutorials: bool,
    /// Whether to include example projects
    pub include_examples: bool,
    /// Custom CSS/styles to include
    pub custom_styles: Option<String>,
    /// Base URL for the documentation (for relative links)
    pub base_url: String,
}

impl Default for DocsConfig {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            source_dir: std::path::PathBuf::from("docs"),
            output_dir: std::path::PathBuf::from("target/docs"),
            include_api_docs: true,
            include_tutorials: true,
            include_examples: true,
            custom_styles: None,
            base_url: "/".to_string(),
        }
    }
}

impl DocsConfig {
    /// Create a new configuration with the specified version
    pub fn with_version(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            ..Default::default()
        }
    }

    /// Set the source directory
    pub fn source_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.source_dir = dir.into();
        self
    }

    /// Set the output directory
    pub fn output_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.output_dir = dir.into();
        self
    }

    /// Include or exclude API docs
    pub fn with_api_docs(mut self, include: bool) -> Self {
        self.include_api_docs = include;
        self
    }

    /// Include or exclude tutorials
    pub fn with_tutorials(mut self, include: bool) -> Self {
        self.include_tutorials = include;
        self
    }

    /// Include or exclude examples
    pub fn with_examples(mut self, include: bool) -> Self {
        self.include_examples = include;
        self
    }
}

/// Get the default docs directory based on platform
pub fn default_docs_dir() -> std::path::PathBuf {
    if let Some(data_dir) = dirs_data_dir() {
        data_dir.join("oxidekit").join("docs")
    } else {
        std::path::PathBuf::from(".oxidekit").join("docs")
    }
}

/// Get the user's data directory
fn dirs_data_dir() -> Option<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        std::env::var_os("HOME")
            .map(std::path::PathBuf::from)
            .map(|h| h.join("Library").join("Application Support"))
    }
    #[cfg(target_os = "linux")]
    {
        std::env::var_os("XDG_DATA_HOME")
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .map(std::path::PathBuf::from)
                    .map(|h| h.join(".local").join("share"))
            })
    }
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA").map(std::path::PathBuf::from)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = DocsConfig::default();
        assert!(config.include_api_docs);
        assert!(config.include_tutorials);
        assert!(config.include_examples);
    }

    #[test]
    fn test_config_builder() {
        let config = DocsConfig::with_version("0.2.0")
            .source_dir("/custom/docs")
            .output_dir("/custom/output")
            .with_api_docs(false);

        assert_eq!(config.version, "0.2.0");
        assert!(!config.include_api_docs);
    }
}
