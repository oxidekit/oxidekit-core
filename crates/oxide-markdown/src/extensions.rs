//! Markdown extensions (footnotes, math, diagrams).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Available extensions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Extension {
    /// Footnotes
    Footnotes,
    /// Math (LaTeX)
    Math,
    /// Mermaid diagrams
    Mermaid,
    /// Emoji shortcodes
    Emoji,
    /// Table of contents
    Toc,
    /// Abbreviations
    Abbreviations,
    /// Task lists
    TaskLists,
}

/// A footnote reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Footnote {
    /// Footnote identifier
    pub id: String,
    /// Reference number
    pub number: usize,
}

impl Footnote {
    /// Create a new footnote
    pub fn new(id: impl Into<String>, number: usize) -> Self {
        Self {
            id: id.into(),
            number,
        }
    }
}

/// Footnote definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FootnoteDefinition {
    /// Footnote identifier
    pub id: String,
    /// Content
    pub content: String,
}

impl FootnoteDefinition {
    /// Create a new definition
    pub fn new(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            content: content.into(),
        }
    }
}

/// Math block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MathBlock {
    /// LaTeX content
    pub content: String,
    /// Display mode (block vs inline)
    pub display: bool,
}

impl MathBlock {
    /// Create inline math
    pub fn inline(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            display: false,
        }
    }

    /// Create display math
    pub fn display(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            display: true,
        }
    }
}

/// Math renderer trait
pub trait MathRenderer: Send + Sync {
    /// Render LaTeX to HTML/SVG
    fn render(&self, latex: &str, display: bool) -> Result<String, String>;
}

/// Diagram type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiagramType {
    /// Mermaid
    Mermaid,
    /// PlantUML
    PlantUml,
    /// GraphViz DOT
    Graphviz,
}

/// Diagram renderer trait
pub trait DiagramRenderer: Send + Sync {
    /// Render diagram to SVG
    fn render(&self, source: &str, diagram_type: DiagramType) -> Result<String, String>;
}

/// An abbreviation definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Abbreviation {
    /// Short form
    pub short: String,
    /// Full form
    pub full: String,
}

impl Abbreviation {
    /// Create a new abbreviation
    pub fn new(short: impl Into<String>, full: impl Into<String>) -> Self {
        Self {
            short: short.into(),
            full: full.into(),
        }
    }
}

/// Extension registry
#[derive(Debug, Clone, Default)]
pub struct ExtensionRegistry {
    /// Enabled extensions
    enabled: HashMap<Extension, bool>,
    /// Abbreviations
    pub abbreviations: Vec<Abbreviation>,
    /// Footnotes
    pub footnotes: Vec<FootnoteDefinition>,
}

impl ExtensionRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable an extension
    pub fn enable(&mut self, ext: Extension) {
        self.enabled.insert(ext, true);
    }

    /// Disable an extension
    pub fn disable(&mut self, ext: Extension) {
        self.enabled.insert(ext, false);
    }

    /// Check if extension is enabled
    pub fn is_enabled(&self, ext: Extension) -> bool {
        self.enabled.get(&ext).copied().unwrap_or(false)
    }

    /// Add abbreviation
    pub fn add_abbreviation(&mut self, abbr: Abbreviation) {
        self.abbreviations.push(abbr);
    }

    /// Add footnote
    pub fn add_footnote(&mut self, footnote: FootnoteDefinition) {
        self.footnotes.push(footnote);
    }
}
