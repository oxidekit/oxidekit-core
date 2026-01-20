//! OxideKit Markdown Rendering and Editing Component
//!
//! A comprehensive markdown component for OxideKit applications, providing:
//!
//! - CommonMark and GitHub Flavored Markdown (GFM) support
//! - Syntax highlighting for code blocks
//! - Live preview markdown editor with split view
//! - Table of contents generation
//! - Extensions: footnotes, math, diagrams, emoji shortcodes
//! - Customizable themes (GitHub-like, dark/light)
//!
//! # Markdown Renderer
//!
//! ```rust,ignore
//! use oxide_markdown::prelude::*;
//!
//! // Render markdown content
//! let view = Markdown::new(&markdown_content)
//!     .theme(MarkdownTheme::GitHub)
//!     .highlight_code(true)
//!     .toc(true)
//!     .render();
//!
//! // With extensions
//! let view = Markdown::new(&content)
//!     .extension(Extension::Footnotes)
//!     .extension(Extension::Math)
//!     .extension(Extension::Mermaid)
//!     .render();
//! ```
//!
//! # Markdown Editor
//!
//! ```rust,ignore
//! use oxide_markdown::prelude::*;
//!
//! // Full-featured editor with live preview
//! let editor = MarkdownEditor::new()
//!     .value(&draft)
//!     .split_view(true)
//!     .toolbar(true)
//!     .auto_save(Duration::from_secs(30))
//!     .on_change(|content| save_draft(content));
//!
//! // Editor with custom toolbar
//! let editor = MarkdownEditor::new()
//!     .value(&content)
//!     .toolbar_items(&[
//!         ToolbarItem::Bold,
//!         ToolbarItem::Italic,
//!         ToolbarItem::Separator,
//!         ToolbarItem::Link,
//!         ToolbarItem::Image,
//!         ToolbarItem::Code,
//!     ])
//!     .on_change(|content| handle_change(content));
//! ```
//!
//! # Code Blocks
//!
//! ```rust,ignore
//! use oxide_markdown::prelude::*;
//!
//! // Standalone code block with syntax highlighting
//! let block = CodeBlockView::new()
//!     .language("rust")
//!     .code(&source_code)
//!     .line_numbers(true)
//!     .copy_button(true)
//!     .filename("main.rs");
//! ```

pub mod editor;
pub mod extensions;
pub mod highlight;
pub mod parser;
pub mod renderer;
pub mod theme;
pub mod toc;
pub mod toolbar;

// Re-export main types
pub use editor::{
    AutoSaveConfig, EditorMode, EditorState, KeyboardShortcut, MarkdownEditor, MarkdownEditorConfig,
};
pub use extensions::{
    Abbreviation, DiagramRenderer, DiagramType, Extension, ExtensionRegistry, Footnote,
    FootnoteDefinition, MathBlock, MathRenderer,
};
pub use highlight::{
    CodeBlockView, HighlightTheme, LanguageDefinition, LanguageRegistry, SyntaxHighlighter,
    TokenStyle, TokenType,
};
pub use parser::{
    BlockElement, InlineElement, ListItem, ListType, MarkdownDocument, MarkdownParser, ParseError,
    TableAlignment, TableCell, TableRow,
};
pub use renderer::{
    Markdown, MarkdownRenderer, RenderedBlock, RenderedElement, RenderedInline, RenderOptions,
};
pub use theme::{MarkdownTheme, ThemeColors, ThemeConfig, TypographyTokens};
pub use toc::{TocConfig, TocEntry, TocRenderer, TableOfContents};
pub use toolbar::{Toolbar, ToolbarConfig, ToolbarItem, ToolbarAction};

/// Errors that can occur in markdown processing
#[derive(Debug, thiserror::Error)]
pub enum MarkdownError {
    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid markdown syntax
    #[error("Invalid markdown syntax at line {line}: {message}")]
    InvalidSyntax { line: usize, message: String },

    /// Unknown language for syntax highlighting
    #[error("Unknown language: {0}")]
    UnknownLanguage(String),

    /// Extension error
    #[error("Extension error ({extension}): {message}")]
    ExtensionError { extension: String, message: String },

    /// Math rendering error
    #[error("Math rendering error: {0}")]
    MathError(String),

    /// Diagram rendering error
    #[error("Diagram rendering error: {0}")]
    DiagramError(String),

    /// Theme error
    #[error("Theme error: {0}")]
    ThemeError(String),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),

    /// Render error
    #[error("Render error: {0}")]
    RenderError(String),
}

/// Result type for markdown operations
pub type MarkdownResult<T> = Result<T, MarkdownError>;

/// Convenient re-exports for common usage
pub mod prelude {
    pub use crate::editor::{
        AutoSaveConfig, EditorMode, EditorState, KeyboardShortcut, MarkdownEditor,
        MarkdownEditorConfig,
    };
    pub use crate::extensions::{
        Abbreviation, DiagramRenderer, DiagramType, Extension, ExtensionRegistry, Footnote,
        FootnoteDefinition, MathBlock, MathRenderer,
    };
    pub use crate::highlight::{
        CodeBlockView, HighlightTheme, LanguageDefinition, LanguageRegistry, SyntaxHighlighter,
        TokenStyle, TokenType,
    };
    pub use crate::parser::{
        BlockElement, InlineElement, ListItem, ListType, MarkdownDocument, MarkdownParser,
        ParseError, TableAlignment, TableCell, TableRow,
    };
    pub use crate::renderer::{
        Markdown, MarkdownRenderer, RenderedBlock, RenderedElement, RenderedInline, RenderOptions,
    };
    pub use crate::theme::{MarkdownTheme, ThemeColors, ThemeConfig, TypographyTokens};
    pub use crate::toc::{TocConfig, TocEntry, TocRenderer, TableOfContents};
    pub use crate::toolbar::{Toolbar, ToolbarAction, ToolbarConfig, ToolbarItem};
    pub use crate::{MarkdownError, MarkdownResult};
}

/// Position in markdown content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed)
    pub column: usize,
}

impl Position {
    /// Create a new position
    pub fn new(line: usize, column: usize) -> Self {
        Self { line, column }
    }

    /// Create a position at the start of a line
    pub fn line_start(line: usize) -> Self {
        Self { line, column: 0 }
    }

    /// Check if this position is before another
    pub fn is_before(&self, other: &Position) -> bool {
        self.line < other.line || (self.line == other.line && self.column < other.column)
    }

    /// Check if this position is after another
    pub fn is_after(&self, other: &Position) -> bool {
        other.is_before(self)
    }
}

impl PartialOrd for Position {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Position {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.line.cmp(&other.line) {
            std::cmp::Ordering::Equal => self.column.cmp(&other.column),
            ord => ord,
        }
    }
}

/// A range in markdown content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Range {
    /// Start position (inclusive)
    pub start: Position,
    /// End position (exclusive)
    pub end: Position,
}

impl Range {
    /// Create a new range
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end }
    }

    /// Create a range for a single line
    pub fn single_line(line: usize, start_col: usize, end_col: usize) -> Self {
        Self {
            start: Position::new(line, start_col),
            end: Position::new(line, end_col),
        }
    }

    /// Check if this range contains a position
    pub fn contains(&self, pos: &Position) -> bool {
        !pos.is_before(&self.start) && pos.is_before(&self.end)
    }

    /// Check if this range is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

/// Heading level (1-6)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HeadingLevel {
    H1 = 1,
    H2 = 2,
    H3 = 3,
    H4 = 4,
    H5 = 5,
    H6 = 6,
}

impl HeadingLevel {
    /// Create from a numeric level (clamped to 1-6)
    pub fn from_level(level: usize) -> Self {
        match level {
            1 => HeadingLevel::H1,
            2 => HeadingLevel::H2,
            3 => HeadingLevel::H3,
            4 => HeadingLevel::H4,
            5 => HeadingLevel::H5,
            _ => HeadingLevel::H6,
        }
    }

    /// Get the numeric level
    pub fn level(&self) -> usize {
        *self as usize
    }
}

impl From<usize> for HeadingLevel {
    fn from(level: usize) -> Self {
        HeadingLevel::from_level(level)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_position_ordering() {
        let p1 = Position::new(0, 0);
        let p2 = Position::new(0, 5);
        let p3 = Position::new(1, 0);

        assert!(p1.is_before(&p2));
        assert!(p2.is_before(&p3));
        assert!(!p3.is_before(&p1));
        assert!(p3.is_after(&p1));
    }

    #[test]
    fn test_range_contains() {
        let range = Range::new(Position::new(1, 5), Position::new(3, 10));

        assert!(range.contains(&Position::new(1, 5)));
        assert!(range.contains(&Position::new(2, 0)));
        assert!(!range.contains(&Position::new(3, 10)));
        assert!(!range.contains(&Position::new(0, 0)));
    }

    #[test]
    fn test_heading_level() {
        assert_eq!(HeadingLevel::from_level(1), HeadingLevel::H1);
        assert_eq!(HeadingLevel::from_level(6), HeadingLevel::H6);
        assert_eq!(HeadingLevel::from_level(7), HeadingLevel::H6);
        assert_eq!(HeadingLevel::H3.level(), 3);
    }
}
