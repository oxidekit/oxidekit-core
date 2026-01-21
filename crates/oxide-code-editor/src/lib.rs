//! OxideKit Code Editor Component
//!
//! A full-featured code editor component for OxideKit applications, providing:
//!
//! - Syntax highlighting with support for multiple languages
//! - Line numbers and gutter with fold markers
//! - Code folding for blocks and regions
//! - Multiple cursor support
//! - Bracket matching and auto-closing
//! - Find and replace with regex support
//! - Basic code intelligence (autocomplete, hover, diagnostics)
//! - Minimap for navigation
//! - Diff view for comparing files
//! - Theme support (dark/light and custom)
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_code_editor::prelude::*;
//!
//! // Full-featured code editor
//! let editor = CodeEditor::new()
//!     .language(Language::Rust)
//!     .theme(Theme::Dark)
//!     .value(&source_code)
//!     .line_numbers(true)
//!     .folding(true)
//!     .on_change(|code| save_code(code));
//!
//! // Read-only code block
//! let block = CodeBlock::new()
//!     .language(Language::Json)
//!     .code(&json_string)
//!     .copy_button(true)
//!     .highlight_lines(&[3, 4, 5]);
//! ```

pub mod diff;
pub mod editor;
pub mod features;
pub mod syntax;
pub mod view;

// Re-export main types
pub use diff::{DiffChange, DiffLine, DiffView, DiffViewConfig};
pub use editor::{CodeBlock, CodeEditor, EditorConfig, EditorEvent, EditorState};
pub use features::complete::{
    AutoComplete, CompletionItem, CompletionKind, CompletionProvider, CompletionTrigger,
};
pub use features::cursors::{Cursor, CursorMode, CursorShape, MultiCursor, Selection};
pub use features::folding::{FoldRange, FoldState, FoldingProvider, FoldingRange};
pub use features::search::{FindOptions, FindResult, SearchMatch, SearchReplace};
pub use syntax::highlighter::{Highlight, HighlightSpan, SyntaxHighlighter, TokenType};
pub use syntax::languages::{Language, LanguageConfig, LanguageDefinition, LanguageRegistry};
pub use syntax::themes::{Theme, ThemeColors, TokenColors};
pub use view::gutter::{GutterConfig, GutterItem, GutterMarker, GutterView};
pub use view::line_numbers::{LineNumberConfig, LineNumberView};
pub use view::minimap::{Minimap, MinimapConfig};

/// Errors that can occur in the code editor
#[derive(Debug, thiserror::Error)]
pub enum EditorError {
    /// Language not found in registry
    #[error("Unknown language: {0}")]
    UnknownLanguage(String),

    /// Invalid line number
    #[error("Line {0} out of range (max: {1})")]
    LineOutOfRange(usize, usize),

    /// Invalid column number
    #[error("Column {0} out of range at line {1} (max: {2})")]
    ColumnOutOfRange(usize, usize, usize),

    /// Invalid selection range
    #[error("Invalid selection range")]
    InvalidSelection,

    /// Regex pattern error
    #[error("Invalid regex pattern: {0}")]
    InvalidRegex(String),

    /// Theme error
    #[error("Theme error: {0}")]
    ThemeError(String),

    /// Fold error
    #[error("Cannot fold at line {0}")]
    CannotFold(usize),

    /// I/O error
    #[error("I/O error: {0}")]
    IoError(String),
}

/// Result type for editor operations
pub type EditorResult<T> = Result<T, EditorError>;

/// Convenient re-exports for common usage
pub mod prelude {
    pub use crate::diff::{DiffChange, DiffLine, DiffView, DiffViewConfig};
    pub use crate::editor::{CodeBlock, CodeEditor, EditorConfig, EditorEvent, EditorState};
    pub use crate::features::complete::{
        AutoComplete, CompletionItem, CompletionKind, CompletionProvider, CompletionTrigger,
    };
    pub use crate::features::cursors::{Cursor, CursorMode, CursorShape, MultiCursor, Selection};
    pub use crate::features::folding::{FoldRange, FoldState, FoldingProvider, FoldingRange};
    pub use crate::features::search::{FindOptions, FindResult, SearchMatch, SearchReplace};
    pub use crate::syntax::highlighter::{Highlight, HighlightSpan, SyntaxHighlighter, TokenType};
    pub use crate::syntax::languages::{Language, LanguageConfig, LanguageDefinition, LanguageRegistry};
    pub use crate::syntax::themes::{Theme, ThemeColors, TokenColors};
    pub use crate::view::gutter::{GutterConfig, GutterItem, GutterMarker, GutterView};
    pub use crate::view::line_numbers::{LineNumberConfig, LineNumberView};
    pub use crate::view::minimap::{Minimap, MinimapConfig};
    pub use crate::{EditorError, EditorResult};
}

/// Position in the editor (line and column are 0-indexed)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Position {
    /// Line number (0-indexed)
    pub line: usize,
    /// Column number (0-indexed, in characters not bytes)
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

    /// Get the minimum of two positions
    pub fn min(self, other: Position) -> Position {
        if self.is_before(&other) {
            self
        } else {
            other
        }
    }

    /// Get the maximum of two positions
    pub fn max(self, other: Position) -> Position {
        if self.is_after(&other) {
            self
        } else {
            other
        }
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

/// A range in the editor
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

    /// Create a range from line numbers (full lines)
    pub fn from_lines(start_line: usize, end_line: usize) -> Self {
        Self {
            start: Position::line_start(start_line),
            end: Position::line_start(end_line + 1),
        }
    }

    /// Check if this range contains a position
    pub fn contains(&self, pos: &Position) -> bool {
        !pos.is_before(&self.start) && pos.is_before(&self.end)
    }

    /// Check if this range overlaps with another
    pub fn overlaps(&self, other: &Range) -> bool {
        self.start.is_before(&other.end) && other.start.is_before(&self.end)
    }

    /// Check if this range is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Normalize the range so start is before end
    pub fn normalize(self) -> Self {
        if self.end.is_before(&self.start) {
            Self {
                start: self.end,
                end: self.start,
            }
        } else {
            self
        }
    }

    /// Extend this range to include another position
    pub fn extend_to(&self, pos: Position) -> Self {
        Self {
            start: self.start.min(pos),
            end: self.end.max(pos),
        }
    }

    /// Check if this range spans multiple lines
    pub fn is_multiline(&self) -> bool {
        self.start.line != self.end.line
    }

    /// Get the number of lines spanned
    pub fn line_count(&self) -> usize {
        self.end.line - self.start.line + 1
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
        let p4 = Position::new(1, 3);

        assert!(p1.is_before(&p2));
        assert!(p2.is_before(&p3));
        assert!(p3.is_before(&p4));
        assert!(!p4.is_before(&p1));
        assert!(p4.is_after(&p1));
    }

    #[test]
    fn test_position_min_max() {
        let p1 = Position::new(5, 10);
        let p2 = Position::new(3, 15);

        assert_eq!(p1.min(p2), p2);
        assert_eq!(p1.max(p2), p1);
    }

    #[test]
    fn test_range_contains() {
        let range = Range::new(Position::new(1, 5), Position::new(3, 10));

        assert!(range.contains(&Position::new(1, 5)));
        assert!(range.contains(&Position::new(2, 0)));
        assert!(range.contains(&Position::new(3, 9)));
        assert!(!range.contains(&Position::new(3, 10)));
        assert!(!range.contains(&Position::new(0, 0)));
        assert!(!range.contains(&Position::new(4, 0)));
    }

    #[test]
    fn test_range_overlaps() {
        let r1 = Range::new(Position::new(0, 0), Position::new(2, 5));
        let r2 = Range::new(Position::new(1, 0), Position::new(3, 0));
        let r3 = Range::new(Position::new(3, 0), Position::new(4, 0));
        let r4 = Range::new(Position::new(2, 0), Position::new(4, 0)); // actually overlaps with r2

        assert!(r1.overlaps(&r2));
        assert!(r2.overlaps(&r1));
        assert!(!r1.overlaps(&r3));
        // r2 ends at (3,0) and r3 starts at (3,0) - they share a boundary but don't overlap
        assert!(!r2.overlaps(&r3));
        // r2 and r4 actually overlap (r2: 1-3, r4: 2-4)
        assert!(r2.overlaps(&r4));
    }

    #[test]
    fn test_range_normalize() {
        let r1 = Range::new(Position::new(3, 5), Position::new(1, 0));
        let normalized = r1.normalize();

        assert_eq!(normalized.start, Position::new(1, 0));
        assert_eq!(normalized.end, Position::new(3, 5));
    }

    #[test]
    fn test_range_multiline() {
        let single = Range::single_line(5, 0, 10);
        let multi = Range::from_lines(1, 5);

        assert!(!single.is_multiline());
        assert!(multi.is_multiline());
        assert_eq!(single.line_count(), 1);
        assert_eq!(multi.line_count(), 6);
    }
}
