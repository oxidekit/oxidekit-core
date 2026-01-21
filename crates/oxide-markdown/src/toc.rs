//! Table of contents generation.

use serde::{Deserialize, Serialize};
use crate::HeadingLevel;

/// A table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry ID (for linking)
    pub id: String,
    /// Display text
    pub text: String,
    /// Heading level
    pub level: HeadingLevel,
    /// Child entries
    pub children: Vec<TocEntry>,
}

impl TocEntry {
    /// Create a new entry
    pub fn new(id: impl Into<String>, text: impl Into<String>, level: HeadingLevel) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            level,
            children: Vec::new(),
        }
    }

    /// Add child entry
    pub fn add_child(&mut self, child: TocEntry) {
        self.children.push(child);
    }

    /// Check if has children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }
}

/// TOC configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocConfig {
    /// Minimum heading level
    pub min_level: u8,
    /// Maximum heading level
    pub max_level: u8,
    /// Generate IDs for headings
    pub generate_ids: bool,
    /// ID prefix
    pub id_prefix: String,
}

impl Default for TocConfig {
    fn default() -> Self {
        Self {
            min_level: 1,
            max_level: 3,
            generate_ids: true,
            id_prefix: "heading-".to_string(),
        }
    }
}

impl TocConfig {
    /// Create new config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set level range
    pub fn levels(mut self, min: u8, max: u8) -> Self {
        self.min_level = min;
        self.max_level = max;
        self
    }
}

/// Table of contents
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TableOfContents {
    /// Entries
    pub entries: Vec<TocEntry>,
}

impl TableOfContents {
    /// Create new TOC
    pub fn new() -> Self {
        Self::default()
    }

    /// Add entry
    pub fn add_entry(&mut self, entry: TocEntry) {
        self.entries.push(entry);
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get entry count (including nested)
    pub fn count(&self) -> usize {
        fn count_recursive(entries: &[TocEntry]) -> usize {
            entries.iter().map(|e| 1 + count_recursive(&e.children)).sum()
        }
        count_recursive(&self.entries)
    }
}

/// TOC renderer
#[derive(Debug, Clone, Default)]
pub struct TocRenderer {
    /// Configuration
    pub config: TocConfig,
}

impl TocRenderer {
    /// Create new renderer
    pub fn new() -> Self {
        Self::default()
    }

    /// Set configuration
    pub fn config(mut self, config: TocConfig) -> Self {
        self.config = config;
        self
    }

    /// Generate ID from text
    pub fn generate_id(&self, text: &str) -> String {
        let slug: String = text
            .to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect();
        format!("{}{}", self.config.id_prefix, slug.trim_matches('-'))
    }

    /// Extract TOC from content
    pub fn extract(&self, _content: &str) -> TableOfContents {
        // Basic implementation
        TableOfContents::new()
    }
}
