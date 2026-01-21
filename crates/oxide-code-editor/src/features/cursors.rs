//! Cursor and selection management.

use crate::Position;
use serde::{Deserialize, Serialize};

/// Cursor shape
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CursorShape {
    /// Line cursor
    #[default]
    Line,
    /// Block cursor
    Block,
    /// Underline cursor
    Underline,
}

/// Cursor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum CursorMode {
    /// Normal editing mode
    #[default]
    Normal,
    /// Insert mode
    Insert,
    /// Visual/selection mode
    Visual,
}

/// A text selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    /// Start of selection (anchor point)
    pub start: Position,
    /// End of selection (head/cursor position)
    pub end: Position,
    /// Anchor position (where selection started)
    pub anchor: Position,
    /// Head position (where cursor is)
    pub head: Position,
}

impl Selection {
    /// Create a new selection
    pub fn new(start: Position, end: Position) -> Self {
        Self { start, end, anchor: start, head: end }
    }

    /// Check if selection is empty
    pub fn is_empty(&self) -> bool {
        self.start == self.end
    }

    /// Get normalized selection (start before end)
    pub fn normalized(&self) -> Self {
        if self.end.is_before(&self.start) {
            Self {
                start: self.end,
                end: self.start,
                anchor: self.anchor,
                head: self.head,
            }
        } else {
            *self
        }
    }

    /// Convert to range
    pub fn to_range(&self) -> crate::Range {
        crate::Range::new(self.start, self.end).normalize()
    }
}

/// A cursor in the editor
#[derive(Debug, Clone, Default)]
pub struct Cursor {
    /// Position in the document
    pub position: Position,
    /// Selection (if any)
    pub selection: Option<Selection>,
    /// Shape of the cursor
    pub shape: CursorShape,
}

impl Cursor {
    /// Create a new cursor at position
    pub fn new(position: Position) -> Self {
        Self {
            position,
            selection: None,
            shape: CursorShape::Line,
        }
    }

    /// Move to position
    pub fn move_to(&mut self, position: Position) {
        self.position = position;
    }

    /// Start selection
    pub fn start_selection(&mut self) {
        self.selection = Some(Selection::new(self.position, self.position));
    }

    /// Extend selection to position
    pub fn extend_selection(&mut self, position: Position) {
        if let Some(ref mut sel) = self.selection {
            sel.end = position;
        }
        self.position = position;
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selection = None;
    }

    /// Check if has selection
    pub fn has_selection(&self) -> bool {
        self.selection.map_or(false, |s| !s.is_empty())
    }

    /// Set selection
    pub fn set_selection(&mut self, selection: Selection) {
        self.selection = Some(selection);
        self.position = selection.head;
    }
}

/// Multi-cursor support
#[derive(Debug, Clone, Default)]
pub struct MultiCursor {
    /// All cursors
    pub cursors: Vec<Cursor>,
}

impl MultiCursor {
    /// Create new with default cursor at origin
    pub fn new() -> Self {
        Self {
            cursors: vec![Cursor::default()],
        }
    }

    /// Create with a single cursor
    pub fn single(cursor: Cursor) -> Self {
        Self {
            cursors: vec![cursor],
        }
    }

    /// Get all cursors
    pub fn all(&self) -> &[Cursor] {
        &self.cursors
    }

    /// Set primary cursor position
    pub fn set_primary_position(&mut self, position: Position) {
        if let Some(cursor) = self.cursors.first_mut() {
            cursor.position = position;
        }
    }

    /// Restore cursor positions
    pub fn restore_positions(&mut self, positions: &[Position]) {
        self.cursors.clear();
        for pos in positions {
            self.cursors.push(Cursor::new(*pos));
        }
        if self.cursors.is_empty() {
            self.cursors.push(Cursor::default());
        }
    }

    /// Add a cursor
    pub fn add(&mut self, cursor: Cursor) {
        self.cursors.push(cursor);
    }

    /// Remove cursor at index
    pub fn remove(&mut self, index: usize) {
        if self.cursors.len() > 1 && index < self.cursors.len() {
            self.cursors.remove(index);
        }
    }

    /// Get primary cursor
    /// # Panics
    /// Panics if there are no cursors (should never happen as we always maintain at least one)
    pub fn primary(&self) -> &Cursor {
        self.cursors.first().expect("MultiCursor should always have at least one cursor")
    }

    /// Get mutable primary cursor
    /// # Panics
    /// Panics if there are no cursors (should never happen as we always maintain at least one)
    pub fn primary_mut(&mut self) -> &mut Cursor {
        self.cursors.first_mut().expect("MultiCursor should always have at least one cursor")
    }

    /// Reset to single cursor
    pub fn reset(&mut self) {
        if let Some(first) = self.cursors.first().cloned() {
            self.cursors = vec![first];
        }
    }
}
