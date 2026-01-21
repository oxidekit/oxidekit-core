//! Markdown rendering.

use serde::{Deserialize, Serialize};
use crate::parser::MarkdownDocument;
use crate::theme::MarkdownTheme;

/// Render options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RenderOptions {
    /// Enable syntax highlighting
    pub highlight_code: bool,
    /// Generate table of contents
    pub toc: bool,
    /// Enable link previews
    pub link_previews: bool,
    /// Sanitize HTML
    pub sanitize: bool,
}

impl RenderOptions {
    /// Create new options
    pub fn new() -> Self {
        Self {
            highlight_code: true,
            toc: false,
            link_previews: false,
            sanitize: true,
        }
    }
}

/// A rendered block element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderedBlock {
    /// Paragraph
    Paragraph(Vec<RenderedInline>),
    /// Heading
    Heading {
        level: u8,
        content: Vec<RenderedInline>,
        id: Option<String>,
    },
    /// Code block
    CodeBlock {
        language: Option<String>,
        code: String,
        highlighted: Option<String>,
    },
    /// Block quote
    BlockQuote(Vec<RenderedBlock>),
    /// List
    List {
        ordered: bool,
        items: Vec<Vec<RenderedBlock>>,
    },
    /// Horizontal rule
    HorizontalRule,
    /// Table
    Table {
        headers: Vec<Vec<RenderedInline>>,
        rows: Vec<Vec<Vec<RenderedInline>>>,
    },
}

/// A rendered inline element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RenderedInline {
    /// Plain text
    Text(String),
    /// Emphasis
    Emphasis(Vec<RenderedInline>),
    /// Strong emphasis
    Strong(Vec<RenderedInline>),
    /// Strikethrough
    Strikethrough(Vec<RenderedInline>),
    /// Code span
    Code(String),
    /// Link
    Link {
        url: String,
        title: Option<String>,
        content: Vec<RenderedInline>,
    },
    /// Image
    Image {
        url: String,
        alt: String,
        title: Option<String>,
    },
    /// Line break
    LineBreak,
}

/// A rendered element (alias)
pub type RenderedElement = RenderedBlock;

/// Markdown renderer
#[derive(Debug, Clone)]
pub struct MarkdownRenderer {
    /// Theme
    pub theme: MarkdownTheme,
    /// Options
    pub options: RenderOptions,
}

impl MarkdownRenderer {
    /// Create new renderer
    pub fn new() -> Self {
        Self {
            theme: MarkdownTheme::default(),
            options: RenderOptions::new(),
        }
    }

    /// Set theme
    pub fn theme(mut self, theme: MarkdownTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set options
    pub fn options(mut self, options: RenderOptions) -> Self {
        self.options = options;
        self
    }

    /// Render document
    pub fn render(&self, _doc: &MarkdownDocument) -> Vec<RenderedBlock> {
        // Basic implementation
        Vec::new()
    }
}

impl Default for MarkdownRenderer {
    fn default() -> Self {
        Self::new()
    }
}

/// Markdown component (high-level API)
#[derive(Debug, Clone)]
pub struct Markdown {
    /// Content
    content: String,
    /// Theme
    theme: MarkdownTheme,
    /// Options
    options: RenderOptions,
}

impl Markdown {
    /// Create new markdown component
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            theme: MarkdownTheme::default(),
            options: RenderOptions::new(),
        }
    }

    /// Set theme
    pub fn theme(mut self, theme: MarkdownTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Enable code highlighting
    pub fn highlight_code(mut self, enable: bool) -> Self {
        self.options.highlight_code = enable;
        self
    }

    /// Enable TOC
    pub fn toc(mut self, enable: bool) -> Self {
        self.options.toc = enable;
        self
    }

    /// Render to blocks
    pub fn render(&self) -> Vec<RenderedBlock> {
        let renderer = MarkdownRenderer::new()
            .theme(self.theme.clone())
            .options(self.options.clone());
        let doc = MarkdownDocument::default();
        renderer.render(&doc)
    }

    /// Get content
    pub fn content(&self) -> &str {
        &self.content
    }
}
