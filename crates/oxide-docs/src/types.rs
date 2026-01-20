//! Core types for the documentation system

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// A documentation page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocPage {
    /// Unique identifier for the page
    pub id: String,
    /// Page title
    pub title: String,
    /// Path relative to docs root
    pub path: PathBuf,
    /// Page content (markdown or HTML)
    pub content: String,
    /// Content format
    pub format: ContentFormat,
    /// Page category
    pub category: DocCategory,
    /// Tags for searchability
    pub tags: Vec<String>,
    /// Parent page ID (for hierarchical navigation)
    pub parent: Option<String>,
    /// Child page IDs
    pub children: Vec<String>,
    /// Last modified timestamp
    pub modified: DateTime<Utc>,
    /// Page metadata
    pub metadata: HashMap<String, String>,
}

/// Content format for documentation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ContentFormat {
    /// Markdown format
    Markdown,
    /// Pre-rendered HTML
    Html,
    /// Plain text
    Text,
}

impl Default for ContentFormat {
    fn default() -> Self {
        Self::Markdown
    }
}

/// Documentation category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocCategory {
    /// Getting started guides
    GettingStarted,
    /// Core concepts
    Concepts,
    /// API reference
    ApiReference,
    /// Component documentation
    Components,
    /// Tutorial content
    Tutorials,
    /// How-to guides
    Guides,
    /// Example projects
    Examples,
    /// CLI reference
    Cli,
    /// Architecture documentation
    Architecture,
    /// Contributing guidelines
    Contributing,
    /// FAQ
    Faq,
    /// Changelog/release notes
    Changelog,
    /// Other/uncategorized
    Other,
}

impl Default for DocCategory {
    fn default() -> Self {
        Self::Other
    }
}

impl DocCategory {
    /// Get the display name for this category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::GettingStarted => "Getting Started",
            Self::Concepts => "Core Concepts",
            Self::ApiReference => "API Reference",
            Self::Components => "Components",
            Self::Tutorials => "Tutorials",
            Self::Guides => "Guides",
            Self::Examples => "Examples",
            Self::Cli => "CLI Reference",
            Self::Architecture => "Architecture",
            Self::Contributing => "Contributing",
            Self::Faq => "FAQ",
            Self::Changelog => "Changelog",
            Self::Other => "Other",
        }
    }

    /// Get the URL slug for this category
    pub fn slug(&self) -> &'static str {
        match self {
            Self::GettingStarted => "getting-started",
            Self::Concepts => "concepts",
            Self::ApiReference => "api",
            Self::Components => "components",
            Self::Tutorials => "tutorials",
            Self::Guides => "guides",
            Self::Examples => "examples",
            Self::Cli => "cli",
            Self::Architecture => "architecture",
            Self::Contributing => "contributing",
            Self::Faq => "faq",
            Self::Changelog => "changelog",
            Self::Other => "other",
        }
    }
}

/// Navigation item in the documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavItem {
    /// Item title
    pub title: String,
    /// Link path
    pub path: String,
    /// Child items
    pub children: Vec<NavItem>,
    /// Whether this item is expanded by default
    pub expanded: bool,
    /// Icon name (optional)
    pub icon: Option<String>,
}

/// Table of contents for a page
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    /// TOC entries
    pub entries: Vec<TocEntry>,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry title
    pub title: String,
    /// Anchor ID
    pub anchor: String,
    /// Heading level (1-6)
    pub level: u8,
    /// Child entries
    pub children: Vec<TocEntry>,
}

/// An example project in the documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Example {
    /// Example identifier
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: ExampleCategory,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Path to the example project
    pub path: PathBuf,
    /// Whether this example can run offline
    pub offline_capable: bool,
    /// Required features
    pub features: Vec<String>,
    /// Tags
    pub tags: Vec<String>,
}

/// Example category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ExampleCategory {
    /// Basic examples
    Basic,
    /// UI component examples
    Components,
    /// Layout examples
    Layout,
    /// State management
    State,
    /// Styling examples
    Styling,
    /// Integration examples
    Integration,
    /// Full applications
    Applications,
}

/// Difficulty level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    /// Beginner-friendly
    Beginner,
    /// Some experience needed
    Intermediate,
    /// Expert level
    Advanced,
}

impl Difficulty {
    /// Get display name
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Beginner => "Beginner",
            Self::Intermediate => "Intermediate",
            Self::Advanced => "Advanced",
        }
    }
}

/// Code snippet for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeSnippet {
    /// Snippet content
    pub code: String,
    /// Programming language
    pub language: String,
    /// Optional filename
    pub filename: Option<String>,
    /// Line numbers to highlight
    pub highlight_lines: Vec<usize>,
    /// Whether this snippet is runnable
    pub runnable: bool,
}

/// Documentation version info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string
    pub version: String,
    /// Whether this is the latest version
    pub is_latest: bool,
    /// Release date
    pub release_date: DateTime<Utc>,
    /// Minimum OxideKit core version
    pub min_core_version: String,
}
