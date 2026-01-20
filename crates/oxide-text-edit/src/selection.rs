//! Text selection handling
//!
//! Provides types and utilities for managing text selection ranges,
//! including word, line, and paragraph selection.

use unicode_segmentation::UnicodeSegmentation;

/// A position within text, representing both character offset and optional line/column info
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    /// Character offset from the start of the text (in bytes)
    pub offset: usize,
    /// Line number (0-indexed)
    pub line: usize,
    /// Column position within the line (in graphemes)
    pub column: usize,
}

impl TextPosition {
    /// Create a new text position
    pub fn new(offset: usize, line: usize, column: usize) -> Self {
        Self { offset, line, column }
    }

    /// Create a position from just an offset (line/column will be 0)
    pub fn from_offset(offset: usize) -> Self {
        Self {
            offset,
            line: 0,
            column: 0,
        }
    }

    /// Calculate full position from text and byte offset
    pub fn from_text_offset(text: &str, offset: usize) -> Self {
        let offset = offset.min(text.len());
        let text_before = &text[..offset];

        let line = text_before.chars().filter(|&c| c == '\n').count();
        let last_newline = text_before.rfind('\n').map(|i| i + 1).unwrap_or(0);
        let column = text_before[last_newline..].graphemes(true).count();

        Self { offset, line, column }
    }
}

impl PartialOrd for TextPosition {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TextPosition {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.offset.cmp(&other.offset)
    }
}

/// Selection range with anchor and focus positions
///
/// The anchor is where the selection started, and the focus is the current position.
/// These may be in any order - the selection can go forward or backward.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SelectionRange {
    /// The starting point of the selection (where the user began selecting)
    pub anchor: TextPosition,
    /// The end point of the selection (where the cursor currently is)
    pub focus: TextPosition,
}

impl SelectionRange {
    /// Create a new selection range
    pub fn new(anchor: TextPosition, focus: TextPosition) -> Self {
        Self { anchor, focus }
    }

    /// Create a collapsed selection (cursor with no selection) at the given position
    pub fn collapsed(position: TextPosition) -> Self {
        Self {
            anchor: position,
            focus: position,
        }
    }

    /// Create a collapsed selection at a byte offset
    pub fn collapsed_at(offset: usize) -> Self {
        Self::collapsed(TextPosition::from_offset(offset))
    }

    /// Check if the selection is collapsed (cursor with no selection)
    pub fn is_collapsed(&self) -> bool {
        self.anchor.offset == self.focus.offset
    }

    /// Check if the selection is empty (same as collapsed)
    pub fn is_empty(&self) -> bool {
        self.is_collapsed()
    }

    /// Get the start position (minimum of anchor and focus)
    pub fn start(&self) -> TextPosition {
        if self.anchor.offset <= self.focus.offset {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Get the end position (maximum of anchor and focus)
    pub fn end(&self) -> TextPosition {
        if self.anchor.offset >= self.focus.offset {
            self.anchor
        } else {
            self.focus
        }
    }

    /// Get the start offset
    pub fn start_offset(&self) -> usize {
        self.start().offset
    }

    /// Get the end offset
    pub fn end_offset(&self) -> usize {
        self.end().offset
    }

    /// Get the length of the selection in bytes
    pub fn len(&self) -> usize {
        self.end_offset() - self.start_offset()
    }

    /// Check if the selection direction is forward (anchor before focus)
    pub fn is_forward(&self) -> bool {
        self.anchor.offset <= self.focus.offset
    }

    /// Check if the selection direction is backward (focus before anchor)
    pub fn is_backward(&self) -> bool {
        self.anchor.offset > self.focus.offset
    }

    /// Collapse the selection to the start
    pub fn collapse_to_start(&mut self) {
        let start = self.start();
        self.anchor = start;
        self.focus = start;
    }

    /// Collapse the selection to the end
    pub fn collapse_to_end(&mut self) {
        let end = self.end();
        self.anchor = end;
        self.focus = end;
    }

    /// Extend the selection to a new focus position
    pub fn extend_to(&mut self, new_focus: TextPosition) {
        self.focus = new_focus;
    }

    /// Set both anchor and focus to a new position (collapse)
    pub fn set_position(&mut self, position: TextPosition) {
        self.anchor = position;
        self.focus = position;
    }

    /// Extract the selected text from the source string
    pub fn extract_text<'a>(&self, text: &'a str) -> &'a str {
        let start = self.start_offset().min(text.len());
        let end = self.end_offset().min(text.len());
        &text[start..end]
    }

    /// Check if a position is within this selection
    pub fn contains(&self, position: usize) -> bool {
        position >= self.start_offset() && position < self.end_offset()
    }

    /// Check if this selection overlaps with another
    pub fn overlaps(&self, other: &SelectionRange) -> bool {
        self.start_offset() < other.end_offset() && other.start_offset() < self.end_offset()
    }
}

/// Selection granularity for click/drag operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionGranularity {
    /// Select individual characters/graphemes
    #[default]
    Character,
    /// Select whole words
    Word,
    /// Select whole lines
    Line,
    /// Select whole paragraphs
    Paragraph,
}

/// Helper functions for text selection operations
pub struct SelectionHelper;

impl SelectionHelper {
    /// Find word boundaries around a position
    ///
    /// Returns (start, end) offsets of the word containing the given position
    pub fn word_at(text: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(text.len());

        // Find the start of the word
        let mut start = offset;
        for (idx, _) in text[..offset].char_indices().rev() {
            let c = text[idx..].chars().next().unwrap();
            if Self::is_word_boundary(c) {
                start = idx + c.len_utf8();
                break;
            }
            start = idx;
        }

        // Find the end of the word
        let mut end = offset;
        for (idx, c) in text[offset..].char_indices() {
            if Self::is_word_boundary(c) {
                end = offset + idx;
                break;
            }
            end = offset + idx + c.len_utf8();
        }

        (start, end)
    }

    /// Check if a character is a word boundary
    pub fn is_word_boundary(c: char) -> bool {
        c.is_whitespace() || c.is_ascii_punctuation()
    }

    /// Find line boundaries around a position
    ///
    /// Returns (start, end) offsets of the line containing the given position
    pub fn line_at(text: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(text.len());

        // Find start of line
        let start = text[..offset].rfind('\n').map(|i| i + 1).unwrap_or(0);

        // Find end of line
        let end = text[offset..].find('\n')
            .map(|i| offset + i)
            .unwrap_or(text.len());

        (start, end)
    }

    /// Find paragraph boundaries around a position
    ///
    /// A paragraph is delimited by blank lines (two or more newlines)
    /// Returns (start, end) offsets of the paragraph
    pub fn paragraph_at(text: &str, offset: usize) -> (usize, usize) {
        let offset = offset.min(text.len());

        // Find start of paragraph (look for double newline or start of text)
        let mut start = 0;
        let text_before = &text[..offset];
        if let Some(pos) = text_before.rfind("\n\n") {
            start = pos + 2;
        } else if let Some(pos) = text_before.rfind("\r\n\r\n") {
            start = pos + 4;
        }

        // Find end of paragraph
        let text_after = &text[offset..];
        let end = if let Some(pos) = text_after.find("\n\n") {
            offset + pos
        } else if let Some(pos) = text_after.find("\r\n\r\n") {
            offset + pos
        } else {
            text.len()
        };

        (start, end)
    }

    /// Find the next word boundary after a position
    pub fn next_word_boundary(text: &str, offset: usize) -> usize {
        let offset = offset.min(text.len());
        let mut chars = text[offset..].char_indices();

        // Skip current word characters
        let mut in_word = false;
        for (idx, c) in chars.by_ref() {
            if Self::is_word_boundary(c) {
                if in_word {
                    return offset + idx;
                }
            } else {
                in_word = true;
            }
        }

        text.len()
    }

    /// Find the previous word boundary before a position
    pub fn prev_word_boundary(text: &str, offset: usize) -> usize {
        let offset = offset.min(text.len());
        if offset == 0 {
            return 0;
        }

        let mut in_word = false;
        let chars: Vec<(usize, char)> = text[..offset].char_indices().collect();

        for (idx, c) in chars.into_iter().rev() {
            if Self::is_word_boundary(c) {
                if in_word {
                    return idx + c.len_utf8();
                }
            } else {
                in_word = true;
            }
        }

        0
    }

    /// Get the grapheme at a given byte offset
    pub fn grapheme_at(text: &str, offset: usize) -> Option<&str> {
        let offset = offset.min(text.len());
        text[offset..].graphemes(true).next()
    }

    /// Convert a byte offset to a grapheme index
    pub fn byte_to_grapheme(text: &str, byte_offset: usize) -> usize {
        text[..byte_offset.min(text.len())].graphemes(true).count()
    }

    /// Convert a grapheme index to a byte offset
    pub fn grapheme_to_byte(text: &str, grapheme_idx: usize) -> usize {
        let mut offset = 0;
        for (idx, grapheme) in text.grapheme_indices(true) {
            if idx >= grapheme_idx {
                return offset;
            }
            offset = idx + grapheme.len();
        }
        text.len()
    }
}

/// Selection affinity determines which line the cursor appears on when at a line boundary
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SelectionAffinity {
    /// Cursor appears at the end of the previous line
    #[default]
    Upstream,
    /// Cursor appears at the start of the next line
    Downstream,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_position_from_offset() {
        let pos = TextPosition::from_offset(10);
        assert_eq!(pos.offset, 10);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_text_position_from_text() {
        let text = "Hello\nWorld\nTest";

        let pos = TextPosition::from_text_offset(text, 0);
        assert_eq!(pos.line, 0);
        assert_eq!(pos.column, 0);

        let pos = TextPosition::from_text_offset(text, 7);
        assert_eq!(pos.line, 1);
        assert_eq!(pos.column, 1);

        let pos = TextPosition::from_text_offset(text, 12);
        assert_eq!(pos.line, 2);
        assert_eq!(pos.column, 0);
    }

    #[test]
    fn test_selection_range_collapsed() {
        let sel = SelectionRange::collapsed_at(5);
        assert!(sel.is_collapsed());
        assert!(sel.is_empty());
        assert_eq!(sel.len(), 0);
    }

    #[test]
    fn test_selection_range_forward() {
        let sel = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(10),
        );
        assert!(!sel.is_collapsed());
        assert!(sel.is_forward());
        assert!(!sel.is_backward());
        assert_eq!(sel.start_offset(), 5);
        assert_eq!(sel.end_offset(), 10);
        assert_eq!(sel.len(), 5);
    }

    #[test]
    fn test_selection_range_backward() {
        let sel = SelectionRange::new(
            TextPosition::from_offset(10),
            TextPosition::from_offset(5),
        );
        assert!(!sel.is_collapsed());
        assert!(!sel.is_forward());
        assert!(sel.is_backward());
        assert_eq!(sel.start_offset(), 5);
        assert_eq!(sel.end_offset(), 10);
    }

    #[test]
    fn test_selection_extract_text() {
        let text = "Hello World";
        let sel = SelectionRange::new(
            TextPosition::from_offset(0),
            TextPosition::from_offset(5),
        );
        assert_eq!(sel.extract_text(text), "Hello");
    }

    #[test]
    fn test_selection_contains() {
        let sel = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(10),
        );
        assert!(sel.contains(5));
        assert!(sel.contains(7));
        assert!(!sel.contains(10));
        assert!(!sel.contains(4));
    }

    #[test]
    fn test_word_at() {
        let text = "Hello World Test";

        let (start, end) = SelectionHelper::word_at(text, 2);
        assert_eq!(&text[start..end], "Hello");

        let (start, end) = SelectionHelper::word_at(text, 7);
        assert_eq!(&text[start..end], "World");
    }

    #[test]
    fn test_line_at() {
        let text = "Line 1\nLine 2\nLine 3";

        let (start, end) = SelectionHelper::line_at(text, 0);
        assert_eq!(&text[start..end], "Line 1");

        let (start, end) = SelectionHelper::line_at(text, 8);
        assert_eq!(&text[start..end], "Line 2");
    }

    #[test]
    fn test_paragraph_at() {
        let text = "Para 1\n\nPara 2\n\nPara 3";

        let (start, end) = SelectionHelper::paragraph_at(text, 0);
        assert_eq!(&text[start..end], "Para 1");

        let (start, end) = SelectionHelper::paragraph_at(text, 9);
        assert_eq!(&text[start..end], "Para 2");
    }

    #[test]
    fn test_next_word_boundary() {
        let text = "Hello World Test";
        assert_eq!(SelectionHelper::next_word_boundary(text, 0), 5);
        assert_eq!(SelectionHelper::next_word_boundary(text, 6), 11);
    }

    #[test]
    fn test_prev_word_boundary() {
        let text = "Hello World Test";
        assert_eq!(SelectionHelper::prev_word_boundary(text, 11), 6);
        assert_eq!(SelectionHelper::prev_word_boundary(text, 5), 0);
    }

    #[test]
    fn test_selection_collapse_to_start() {
        let mut sel = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(10),
        );
        sel.collapse_to_start();
        assert!(sel.is_collapsed());
        assert_eq!(sel.focus.offset, 5);
    }

    #[test]
    fn test_selection_collapse_to_end() {
        let mut sel = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(10),
        );
        sel.collapse_to_end();
        assert!(sel.is_collapsed());
        assert_eq!(sel.focus.offset, 10);
    }

    #[test]
    fn test_selection_extend() {
        let mut sel = SelectionRange::collapsed_at(5);
        sel.extend_to(TextPosition::from_offset(10));
        assert!(!sel.is_collapsed());
        assert_eq!(sel.anchor.offset, 5);
        assert_eq!(sel.focus.offset, 10);
    }

    #[test]
    fn test_selection_overlaps() {
        let sel1 = SelectionRange::new(
            TextPosition::from_offset(0),
            TextPosition::from_offset(10),
        );
        let sel2 = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(15),
        );
        let sel3 = SelectionRange::new(
            TextPosition::from_offset(15),
            TextPosition::from_offset(20),
        );

        assert!(sel1.overlaps(&sel2));
        assert!(!sel1.overlaps(&sel3));
    }
}
