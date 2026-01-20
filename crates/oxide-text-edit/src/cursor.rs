//! Cursor handling for text editing
//!
//! Provides cursor positioning, movement, and blinking functionality.

use crate::selection::{SelectionHelper, SelectionRange, TextPosition};
use instant::Instant;
use unicode_segmentation::UnicodeSegmentation;

/// Cursor state for text editing
#[derive(Debug, Clone)]
pub struct Cursor {
    /// Current cursor position
    position: TextPosition,
    /// Whether the cursor is currently visible (for blinking)
    visible: bool,
    /// Last time the cursor visibility was toggled
    last_blink: Instant,
    /// Blink rate in milliseconds
    blink_rate_ms: u64,
    /// Preferred column for vertical movement (in graphemes)
    /// This preserves the column when moving up/down through lines of different lengths
    preferred_column: Option<usize>,
    /// Whether the cursor should blink
    blink_enabled: bool,
}

impl Default for Cursor {
    fn default() -> Self {
        Self::new()
    }
}

impl Cursor {
    /// Create a new cursor at position 0
    pub fn new() -> Self {
        Self {
            position: TextPosition::default(),
            visible: true,
            last_blink: Instant::now(),
            blink_rate_ms: 530, // Standard cursor blink rate
            preferred_column: None,
            blink_enabled: true,
        }
    }

    /// Create a cursor at a specific position
    pub fn at(position: TextPosition) -> Self {
        Self {
            position,
            ..Self::new()
        }
    }

    /// Create a cursor at a byte offset
    pub fn at_offset(offset: usize) -> Self {
        Self::at(TextPosition::from_offset(offset))
    }

    /// Get the current cursor position
    pub fn position(&self) -> TextPosition {
        self.position
    }

    /// Get the byte offset of the cursor
    pub fn offset(&self) -> usize {
        self.position.offset
    }

    /// Set the cursor position
    pub fn set_position(&mut self, position: TextPosition) {
        self.position = position;
        self.reset_blink();
        // Clear preferred column when explicitly setting position
        self.preferred_column = None;
    }

    /// Set the cursor position from a byte offset
    pub fn set_offset(&mut self, offset: usize) {
        self.position = TextPosition::from_offset(offset);
        self.reset_blink();
        self.preferred_column = None;
    }

    /// Set the cursor position from text and byte offset
    pub fn set_from_text(&mut self, text: &str, offset: usize) {
        self.position = TextPosition::from_text_offset(text, offset);
        self.reset_blink();
        self.preferred_column = None;
    }

    /// Update cursor position and preserve preferred column
    fn set_position_preserve_column(&mut self, position: TextPosition) {
        self.position = position;
        self.reset_blink();
        // Don't clear preferred_column - preserve it for vertical movement
    }

    /// Get the blink rate in milliseconds
    pub fn blink_rate_ms(&self) -> u64 {
        self.blink_rate_ms
    }

    /// Set the blink rate in milliseconds
    pub fn set_blink_rate(&mut self, rate_ms: u64) {
        self.blink_rate_ms = rate_ms;
    }

    /// Enable or disable cursor blinking
    pub fn set_blink_enabled(&mut self, enabled: bool) {
        self.blink_enabled = enabled;
        if !enabled {
            self.visible = true;
        }
    }

    /// Check if cursor blinking is enabled
    pub fn is_blink_enabled(&self) -> bool {
        self.blink_enabled
    }

    /// Update the cursor blink state
    /// Returns true if the visibility changed
    pub fn update_blink(&mut self) -> bool {
        if !self.blink_enabled {
            return false;
        }

        let now = Instant::now();
        let elapsed = now.duration_since(self.last_blink).as_millis() as u64;

        if elapsed >= self.blink_rate_ms {
            self.visible = !self.visible;
            self.last_blink = now;
            true
        } else {
            false
        }
    }

    /// Reset the blink timer (called when cursor moves)
    pub fn reset_blink(&mut self) {
        self.visible = true;
        self.last_blink = Instant::now();
    }

    /// Check if the cursor is currently visible
    pub fn is_visible(&self) -> bool {
        self.visible || !self.blink_enabled
    }

    /// Force cursor visibility
    pub fn show(&mut self) {
        self.visible = true;
        self.last_blink = Instant::now();
    }

    /// Hide the cursor
    pub fn hide(&mut self) {
        self.visible = false;
    }

    // Movement operations

    /// Move cursor to the start of the text
    pub fn move_to_start(&mut self, _text: &str) {
        self.set_position(TextPosition::new(0, 0, 0));
    }

    /// Move cursor to the end of the text
    pub fn move_to_end(&mut self, text: &str) {
        let pos = TextPosition::from_text_offset(text, text.len());
        self.set_position(pos);
    }

    /// Move cursor left by one grapheme
    pub fn move_left(&mut self, text: &str) {
        if self.position.offset == 0 {
            return;
        }

        let before = &text[..self.position.offset];
        let graphemes: Vec<&str> = before.graphemes(true).collect();
        if let Some(last) = graphemes.last() {
            let new_offset = self.position.offset - last.len();
            self.set_from_text(text, new_offset);
        }
    }

    /// Move cursor right by one grapheme
    pub fn move_right(&mut self, text: &str) {
        if self.position.offset >= text.len() {
            return;
        }

        let after = &text[self.position.offset..];
        if let Some(grapheme) = after.graphemes(true).next() {
            let new_offset = self.position.offset + grapheme.len();
            self.set_from_text(text, new_offset);
        }
    }

    /// Move cursor left by one word
    pub fn move_word_left(&mut self, text: &str) {
        let new_offset = SelectionHelper::prev_word_boundary(text, self.position.offset);
        self.set_from_text(text, new_offset);
    }

    /// Move cursor right by one word
    pub fn move_word_right(&mut self, text: &str) {
        let new_offset = SelectionHelper::next_word_boundary(text, self.position.offset);
        self.set_from_text(text, new_offset);
    }

    /// Move cursor up one line, maintaining column position
    pub fn move_up(&mut self, text: &str) {
        if self.position.line == 0 {
            // Already at the first line, move to start
            self.move_to_line_start(text);
            return;
        }

        // Store the preferred column if not already set
        if self.preferred_column.is_none() {
            self.preferred_column = Some(self.position.column);
        }

        let target_column = self.preferred_column.unwrap_or(self.position.column);
        let target_line = self.position.line - 1;

        let new_offset = self.offset_for_line_column(text, target_line, target_column);
        let new_pos = TextPosition::from_text_offset(text, new_offset);
        self.set_position_preserve_column(new_pos);
    }

    /// Move cursor down one line, maintaining column position
    pub fn move_down(&mut self, text: &str) {
        let line_count = text.chars().filter(|&c| c == '\n').count() + 1;
        if self.position.line >= line_count - 1 {
            // Already at the last line, move to end
            self.move_to_line_end(text);
            return;
        }

        // Store the preferred column if not already set
        if self.preferred_column.is_none() {
            self.preferred_column = Some(self.position.column);
        }

        let target_column = self.preferred_column.unwrap_or(self.position.column);
        let target_line = self.position.line + 1;

        let new_offset = self.offset_for_line_column(text, target_line, target_column);
        let new_pos = TextPosition::from_text_offset(text, new_offset);
        self.set_position_preserve_column(new_pos);
    }

    /// Move cursor to the start of the current line
    pub fn move_to_line_start(&mut self, text: &str) {
        let (start, _) = SelectionHelper::line_at(text, self.position.offset);
        self.set_from_text(text, start);
    }

    /// Move cursor to the end of the current line
    pub fn move_to_line_end(&mut self, text: &str) {
        let (_, end) = SelectionHelper::line_at(text, self.position.offset);
        self.set_from_text(text, end);
    }

    /// Helper to find byte offset for a given line and column
    fn offset_for_line_column(&self, text: &str, line: usize, column: usize) -> usize {
        let mut current_line = 0;
        let mut line_start = 0;

        for (idx, c) in text.char_indices() {
            if current_line == line {
                line_start = idx;
                break;
            }
            if c == '\n' {
                current_line += 1;
                line_start = idx + 1;
            }
        }

        if current_line < line {
            return text.len();
        }

        // Find the end of this line
        let line_end = text[line_start..]
            .find('\n')
            .map(|i| line_start + i)
            .unwrap_or(text.len());

        // Get the text of this line
        let line_text = &text[line_start..line_end];

        // Move to the target column (or end of line if shorter)
        let mut current_column = 0;
        for (idx, _) in line_text.grapheme_indices(true) {
            if current_column >= column {
                return line_start + idx;
            }
            current_column += 1;
        }

        line_end
    }

    /// Clear the preferred column (call after horizontal movement)
    pub fn clear_preferred_column(&mut self) {
        self.preferred_column = None;
    }

    /// Create a selection from the cursor position
    pub fn to_selection(&self) -> SelectionRange {
        SelectionRange::collapsed(self.position)
    }
}

/// Direction for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorDirection {
    Left,
    Right,
    Up,
    Down,
}

/// Unit for cursor movement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorUnit {
    /// Move by grapheme (single character/emoji)
    Grapheme,
    /// Move by word
    Word,
    /// Move by line
    Line,
    /// Move to document boundary
    Document,
}

impl Cursor {
    /// Move the cursor in a direction by a unit
    pub fn move_by(&mut self, text: &str, direction: CursorDirection, unit: CursorUnit) {
        match (direction, unit) {
            (CursorDirection::Left, CursorUnit::Grapheme) => self.move_left(text),
            (CursorDirection::Right, CursorUnit::Grapheme) => self.move_right(text),
            (CursorDirection::Left, CursorUnit::Word) => self.move_word_left(text),
            (CursorDirection::Right, CursorUnit::Word) => self.move_word_right(text),
            (CursorDirection::Up, CursorUnit::Grapheme | CursorUnit::Line | CursorUnit::Word) => {
                self.move_up(text)
            }
            (CursorDirection::Down, CursorUnit::Grapheme | CursorUnit::Line | CursorUnit::Word) => {
                self.move_down(text)
            }
            (CursorDirection::Left, CursorUnit::Line) => self.move_to_line_start(text),
            (CursorDirection::Right, CursorUnit::Line) => self.move_to_line_end(text),
            (CursorDirection::Left | CursorDirection::Up, CursorUnit::Document) => {
                self.move_to_start(text)
            }
            (CursorDirection::Right | CursorDirection::Down, CursorUnit::Document) => {
                self.move_to_end(text)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_new() {
        let cursor = Cursor::new();
        assert_eq!(cursor.offset(), 0);
        assert!(cursor.is_visible());
    }

    #[test]
    fn test_cursor_at_offset() {
        let cursor = Cursor::at_offset(10);
        assert_eq!(cursor.offset(), 10);
    }

    #[test]
    fn test_cursor_set_position() {
        let mut cursor = Cursor::new();
        cursor.set_offset(5);
        assert_eq!(cursor.offset(), 5);
    }

    #[test]
    fn test_cursor_move_left_right() {
        let text = "Hello";
        let mut cursor = Cursor::at_offset(2);

        cursor.move_left(text);
        assert_eq!(cursor.offset(), 1);

        cursor.move_right(text);
        assert_eq!(cursor.offset(), 2);
    }

    #[test]
    fn test_cursor_move_left_at_start() {
        let text = "Hello";
        let mut cursor = Cursor::at_offset(0);
        cursor.move_left(text);
        assert_eq!(cursor.offset(), 0);
    }

    #[test]
    fn test_cursor_move_right_at_end() {
        let text = "Hello";
        let mut cursor = Cursor::at_offset(5);
        cursor.move_right(text);
        assert_eq!(cursor.offset(), 5);
    }

    #[test]
    fn test_cursor_move_to_start_end() {
        let text = "Hello World";
        let mut cursor = Cursor::at_offset(5);

        cursor.move_to_start(text);
        assert_eq!(cursor.offset(), 0);

        cursor.move_to_end(text);
        assert_eq!(cursor.offset(), 11);
    }

    #[test]
    fn test_cursor_move_word() {
        let text = "Hello World Test";
        let mut cursor = Cursor::at_offset(0);

        cursor.move_word_right(text);
        assert_eq!(cursor.offset(), 5);

        cursor.move_word_right(text);
        assert_eq!(cursor.offset(), 11);

        cursor.move_word_left(text);
        assert_eq!(cursor.offset(), 6);
    }

    #[test]
    fn test_cursor_move_up_down() {
        let text = "Line 1\nLine 2\nLine 3";
        let mut cursor = Cursor::new();
        cursor.set_from_text(text, 8); // In "Line 2"

        cursor.move_up(text);
        assert!(cursor.offset() < 7); // Should be in "Line 1"

        cursor.set_from_text(text, 8);
        cursor.move_down(text);
        assert!(cursor.offset() >= 14); // Should be in "Line 3"
    }

    #[test]
    fn test_cursor_preferred_column() {
        let text = "Long line here\nShort\nAnother long line";
        let mut cursor = Cursor::new();
        cursor.set_from_text(text, 10); // Column 10 in first line

        cursor.move_down(text);
        // Should be at end of "Short" (column 5) but remember column 10
        assert_eq!(cursor.position().line, 1);

        cursor.move_down(text);
        // Should be back at column 10 in the third line
        assert_eq!(cursor.position().line, 2);
        assert!(cursor.position().column >= 5);
    }

    #[test]
    fn test_cursor_line_start_end() {
        let text = "Hello\nWorld\nTest";
        let mut cursor = Cursor::new();
        cursor.set_from_text(text, 8); // Middle of "World"

        cursor.move_to_line_start(text);
        assert_eq!(cursor.offset(), 6); // Start of "World"

        cursor.move_to_line_end(text);
        assert_eq!(cursor.offset(), 11); // End of "World"
    }

    #[test]
    fn test_cursor_blink() {
        let mut cursor = Cursor::new();
        cursor.set_blink_rate(10); // Very fast for testing
        assert!(cursor.is_visible());

        // Wait a bit (can't actually wait in unit tests, but we can simulate)
        // Just verify the blink_enabled flag works
        cursor.set_blink_enabled(false);
        assert!(!cursor.is_blink_enabled());
        assert!(cursor.is_visible()); // Should stay visible when blink disabled
    }

    #[test]
    fn test_cursor_move_by() {
        let text = "Hello World";
        let mut cursor = Cursor::at_offset(0);

        cursor.move_by(text, CursorDirection::Right, CursorUnit::Word);
        assert_eq!(cursor.offset(), 5);

        cursor.move_by(text, CursorDirection::Right, CursorUnit::Document);
        assert_eq!(cursor.offset(), 11);

        cursor.move_by(text, CursorDirection::Left, CursorUnit::Document);
        assert_eq!(cursor.offset(), 0);
    }

    #[test]
    fn test_cursor_with_unicode() {
        let text = "Hello ";
        let mut cursor = Cursor::at_offset(0);

        // Move past "Hello "
        for _ in 0..6 {
            cursor.move_right(text);
        }
        assert_eq!(cursor.offset(), 6);

        // Move into emoji (4 bytes for the emoji)
        cursor.move_right(text);
        assert_eq!(cursor.offset(), 10); // After the emoji

        cursor.move_left(text);
        assert_eq!(cursor.offset(), 6); // Back before the emoji
    }

    #[test]
    fn test_cursor_to_selection() {
        let cursor = Cursor::at_offset(5);
        let selection = cursor.to_selection();
        assert!(selection.is_collapsed());
        assert_eq!(selection.anchor.offset, 5);
        assert_eq!(selection.focus.offset, 5);
    }
}
