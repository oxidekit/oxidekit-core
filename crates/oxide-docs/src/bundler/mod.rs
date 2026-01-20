//! Documentation bundler system
//!
//! This module provides functionality to bundle documentation into
//! distributable offline packages.

mod archive;
mod builder;
mod renderer;

pub use archive::{create_archive, extract_archive};
pub use builder::DocBundler;
pub use renderer::MarkdownRenderer;

use crate::manifest::{DocsManifest, PageEntry};
use crate::search::DocIndex;
use crate::types::{DocCategory, NavItem, VersionInfo};
use crate::{DocsConfig, DocsError, DocsResult};
use std::path::{Path, PathBuf};

/// A built documentation bundle ready for distribution or serving
pub struct DocBundle {
    /// Bundle root directory
    pub root: PathBuf,
    /// Bundle manifest
    pub manifest: DocsManifest,
    /// Search index (loaded lazily)
    search_index: Option<DocIndex>,
}

impl DocBundle {
    /// Build a new documentation bundle from configuration
    pub fn build(config: &DocsConfig) -> DocsResult<Self> {
        let bundler = DocBundler::new(config.clone());
        bundler.build()
    }

    /// Load an existing bundle from disk
    pub fn load(path: impl AsRef<Path>) -> DocsResult<Self> {
        let root = path.as_ref().to_path_buf();
        let manifest_path = root.join("manifest.json");

        if !manifest_path.exists() {
            return Err(DocsError::BundleNotFound(root));
        }

        let manifest = DocsManifest::load(&manifest_path)?;

        Ok(Self {
            root,
            manifest,
            search_index: None,
        })
    }

    /// Load from a compressed archive (.tar.gz)
    pub fn load_archive(archive_path: impl AsRef<Path>, extract_to: impl AsRef<Path>) -> DocsResult<Self> {
        extract_archive(archive_path.as_ref(), extract_to.as_ref())?;
        Self::load(extract_to)
    }

    /// Get the bundle version
    pub fn version(&self) -> &str {
        &self.manifest.version.version
    }

    /// Check if the bundle is compatible with a version
    pub fn is_compatible(&self, version: &str) -> bool {
        self.manifest.is_compatible(version)
    }

    /// Get all pages in the bundle
    pub fn pages(&self) -> impl Iterator<Item = (&String, &PageEntry)> {
        self.manifest.pages.iter()
    }

    /// Get pages by category
    pub fn pages_by_category(&self, category: DocCategory) -> Vec<(&String, &PageEntry)> {
        self.manifest.pages_in_category(category)
    }

    /// Read a page's content from the bundle
    pub fn read_page(&self, page_id: &str) -> DocsResult<String> {
        let entry = self
            .manifest
            .get_page(page_id)
            .ok_or_else(|| DocsError::MissingContent(format!("Page not found: {}", page_id)))?;

        let page_path = self.root.join(&entry.path);
        let content = std::fs::read_to_string(&page_path)?;
        Ok(content)
    }

    /// Get navigation structure
    pub fn navigation(&self) -> &[NavItem] {
        &self.manifest.navigation
    }

    /// Get the search index, loading it if necessary
    pub fn search_index(&mut self) -> DocsResult<&DocIndex> {
        if self.search_index.is_none() {
            if let Some(index_path) = &self.manifest.search_index_path {
                let full_path = self.root.join(index_path);
                let index = DocIndex::load(&full_path)?;
                self.search_index = Some(index);
            } else {
                return Err(DocsError::SearchIndex("No search index in bundle".to_string()));
            }
        }

        self.search_index
            .as_ref()
            .ok_or_else(|| DocsError::SearchIndex("Failed to load search index".to_string()))
    }

    /// Search the documentation
    pub fn search(&mut self, query: &str, limit: usize) -> DocsResult<Vec<crate::SearchResult>> {
        let index = self.search_index()?;
        index.search(query, limit)
    }

    /// Export the bundle to a compressed archive
    pub fn export(&self, output_path: impl AsRef<Path>) -> DocsResult<PathBuf> {
        create_archive(&self.root, output_path.as_ref())
    }

    /// Start the offline documentation viewer server
    #[cfg(feature = "viewer")]
    pub fn serve(&self, port: u16) -> DocsResult<()> {
        crate::viewer::serve_bundle(self, port)
    }

    /// Get the root directory of the bundle
    pub fn root_dir(&self) -> &Path {
        &self.root
    }

    /// List all examples in the bundle
    pub fn examples(&self) -> &[crate::types::Example] {
        &self.manifest.examples
    }

    /// List all tutorial IDs in the bundle
    pub fn tutorial_ids(&self) -> &[String] {
        &self.manifest.tutorials
    }
}

/// Check if a documentation bundle exists at the default location
pub fn bundle_exists() -> bool {
    let default_path = crate::default_docs_dir();
    default_path.join("manifest.json").exists()
}

/// Get information about the installed bundle
pub fn installed_bundle_info() -> Option<VersionInfo> {
    let default_path = crate::default_docs_dir();
    let manifest_path = default_path.join("manifest.json");

    if manifest_path.exists() {
        if let Ok(manifest) = DocsManifest::load(&manifest_path) {
            return Some(manifest.version);
        }
    }
    None
}
