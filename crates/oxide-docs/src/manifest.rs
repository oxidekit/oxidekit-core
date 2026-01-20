//! Documentation bundle manifest

use crate::types::{DocCategory, Example, NavItem, VersionInfo};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Manifest for a documentation bundle
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocsManifest {
    /// Manifest format version
    pub manifest_version: u32,
    /// Documentation version info
    pub version: VersionInfo,
    /// Bundle creation timestamp
    pub created_at: DateTime<Utc>,
    /// Bundle ID (for tracking/updates)
    pub bundle_id: String,
    /// Navigation structure
    pub navigation: Vec<NavItem>,
    /// Page index (id -> relative path)
    pub pages: HashMap<String, PageEntry>,
    /// Examples included in the bundle
    pub examples: Vec<Example>,
    /// Tutorial IDs included
    pub tutorials: Vec<String>,
    /// Categories and their page counts
    pub categories: HashMap<DocCategory, usize>,
    /// Search index path (relative to bundle root)
    pub search_index_path: Option<PathBuf>,
    /// Total size of bundle in bytes
    pub bundle_size: u64,
    /// Checksum for integrity verification
    pub checksum: String,
}

/// Entry for a page in the manifest
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageEntry {
    /// Page title
    pub title: String,
    /// Relative path within bundle
    pub path: PathBuf,
    /// Category
    pub category: DocCategory,
    /// Whether this page has been indexed for search
    pub indexed: bool,
}

impl DocsManifest {
    /// Create a new empty manifest
    pub fn new(version: VersionInfo) -> Self {
        Self {
            manifest_version: 1,
            version,
            created_at: Utc::now(),
            bundle_id: uuid::Uuid::new_v4().to_string(),
            navigation: Vec::new(),
            pages: HashMap::new(),
            examples: Vec::new(),
            tutorials: Vec::new(),
            categories: HashMap::new(),
            search_index_path: None,
            bundle_size: 0,
            checksum: String::new(),
        }
    }

    /// Add a page to the manifest
    pub fn add_page(&mut self, id: String, entry: PageEntry) {
        *self.categories.entry(entry.category).or_insert(0) += 1;
        self.pages.insert(id, entry);
    }

    /// Get a page entry by ID
    pub fn get_page(&self, id: &str) -> Option<&PageEntry> {
        self.pages.get(id)
    }

    /// Get all pages in a category
    pub fn pages_in_category(&self, category: DocCategory) -> Vec<(&String, &PageEntry)> {
        self.pages
            .iter()
            .filter(|(_, entry)| entry.category == category)
            .collect()
    }

    /// Load manifest from a file
    pub fn load(path: &std::path::Path) -> crate::DocsResult<Self> {
        let content = std::fs::read_to_string(path)?;
        let manifest: Self = serde_json::from_str(&content)?;
        Ok(manifest)
    }

    /// Save manifest to a file
    pub fn save(&self, path: &std::path::Path) -> crate::DocsResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Verify bundle integrity using checksum
    pub fn verify_checksum(&self, computed_checksum: &str) -> bool {
        self.checksum == computed_checksum
    }

    /// Get the total number of pages
    pub fn page_count(&self) -> usize {
        self.pages.len()
    }

    /// Check if the bundle is compatible with the current version
    pub fn is_compatible(&self, current_version: &str) -> bool {
        // Simple semver major version check
        let bundle_major = self.version.version.split('.').next().unwrap_or("0");
        let current_major = current_version.split('.').next().unwrap_or("0");
        bundle_major == current_major
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let version = VersionInfo {
            version: "0.1.0".to_string(),
            is_latest: true,
            release_date: Utc::now(),
            min_core_version: "0.1.0".to_string(),
        };

        let manifest = DocsManifest::new(version);
        assert_eq!(manifest.manifest_version, 1);
        assert!(manifest.pages.is_empty());
    }

    #[test]
    fn test_add_page() {
        let version = VersionInfo {
            version: "0.1.0".to_string(),
            is_latest: true,
            release_date: Utc::now(),
            min_core_version: "0.1.0".to_string(),
        };

        let mut manifest = DocsManifest::new(version);
        manifest.add_page(
            "getting-started".to_string(),
            PageEntry {
                title: "Getting Started".to_string(),
                path: PathBuf::from("getting-started/index.html"),
                category: DocCategory::GettingStarted,
                indexed: true,
            },
        );

        assert_eq!(manifest.page_count(), 1);
        assert_eq!(manifest.categories.get(&DocCategory::GettingStarted), Some(&1));
    }
}
