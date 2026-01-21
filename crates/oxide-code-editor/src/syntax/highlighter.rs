//! Syntax highlighter implementation.

use serde::{Deserialize, Serialize};
use super::languages::Language;
use super::themes::Theme;

/// Type of syntax token
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// Keyword
    Keyword,
    /// Identifier
    Identifier,
    /// String literal
    String,
    /// Number literal
    Number,
    /// Comment
    Comment,
    /// Operator
    Operator,
    /// Punctuation
    Punctuation,
    /// Type name
    Type,
    /// Function name
    Function,
    /// Variable
    Variable,
    /// Constant
    Constant,
    /// Attribute/annotation
    Attribute,
    /// Tag (XML/HTML)
    Tag,
    /// Plain text
    Text,
}

/// A highlighted span
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightSpan {
    /// Start offset
    pub start: usize,
    /// End offset
    pub end: usize,
    /// Token type
    pub token_type: TokenType,
}

impl HighlightSpan {
    /// Create a new highlight span
    pub fn new(start: usize, end: usize, token_type: TokenType) -> Self {
        Self {
            start,
            end,
            token_type,
        }
    }

    /// Length of the span
    pub fn len(&self) -> usize {
        self.end.saturating_sub(self.start)
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// A highlight (alias for compatibility)
pub type Highlight = HighlightSpan;

/// Syntax highlighter
#[derive(Debug, Clone)]
pub struct SyntaxHighlighter {
    /// Current language
    language: Option<Language>,
    /// Current theme
    theme: Theme,
    /// Cached highlights per line
    highlights: Vec<Vec<HighlightSpan>>,
}

impl SyntaxHighlighter {
    /// Create a new highlighter
    pub fn new() -> Self {
        Self {
            language: None,
            theme: Theme::default(),
            highlights: Vec::new(),
        }
    }

    /// Set language (builder style)
    pub fn language(mut self, language: Language) -> Self {
        self.language = Some(language);
        self
    }

    /// Set language (mutating)
    pub fn set_language(&mut self, language: Language) {
        self.language = Some(language);
    }

    /// Set theme
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Highlight a line
    pub fn highlight_line(&self, line: &str) -> Vec<HighlightSpan> {
        // Basic implementation - return single text span
        if line.is_empty() {
            return Vec::new();
        }
        vec![HighlightSpan::new(0, line.len(), TokenType::Text)]
    }

    /// Highlight full content
    pub fn highlight(&self, content: &str) -> Vec<Vec<HighlightSpan>> {
        content.lines().map(|line| self.highlight_line(line)).collect()
    }

    /// Highlight all content from a rope document
    pub fn highlight_all(&mut self, document: &ropey::Rope) {
        self.highlights.clear();
        for line_idx in 0..document.len_lines() {
            let line = document.line(line_idx).to_string();
            self.highlights.push(self.highlight_line(&line));
        }
    }

    /// Highlight a range of lines
    pub fn highlight_range(&mut self, document: &ropey::Rope, start_line: usize, end_line: usize) {
        let total_lines = document.len_lines();

        // Ensure highlights vec is large enough
        while self.highlights.len() < total_lines {
            self.highlights.push(Vec::new());
        }

        // Highlight the range
        for line_idx in start_line..end_line.min(total_lines) {
            let line = document.line(line_idx).to_string();
            if line_idx < self.highlights.len() {
                self.highlights[line_idx] = self.highlight_line(&line);
            }
        }
    }

    /// Get cached highlights for a line
    pub fn get_line_highlights(&self, line: usize) -> Option<&[HighlightSpan]> {
        self.highlights.get(line).map(|v| v.as_slice())
    }
}

impl Default for SyntaxHighlighter {
    fn default() -> Self {
        Self::new()
    }
}
