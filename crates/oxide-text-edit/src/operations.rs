//! Text editing operations
//!
//! Provides atomic text manipulation operations for insert, delete, and replace.

use crate::selection::{SelectionRange, TextPosition};

/// Result of a text operation, containing the new text and cursor position
#[derive(Debug, Clone)]
pub struct OperationResult {
    /// The resulting text after the operation
    pub text: String,
    /// The new selection/cursor position after the operation
    pub selection: SelectionRange,
    /// The inverse operation that can undo this change
    pub inverse: Option<TextOperation>,
}

/// Atomic text editing operation
#[derive(Debug, Clone, PartialEq)]
pub enum TextOperation {
    /// Insert text at a position
    Insert {
        /// Position to insert at (byte offset)
        offset: usize,
        /// Text to insert
        text: String,
    },
    /// Delete a range of text
    Delete {
        /// Start position (byte offset)
        start: usize,
        /// End position (byte offset)
        end: usize,
        /// The deleted text (for undo)
        deleted_text: String,
    },
    /// Replace a range with new text
    Replace {
        /// Start position (byte offset)
        start: usize,
        /// End position (byte offset)
        end: usize,
        /// The old text (for undo)
        old_text: String,
        /// The new text
        new_text: String,
    },
    /// Composite operation (multiple operations as one)
    Composite {
        /// Child operations
        operations: Vec<TextOperation>,
    },
}

impl TextOperation {
    /// Create an insert operation
    pub fn insert(offset: usize, text: impl Into<String>) -> Self {
        Self::Insert {
            offset,
            text: text.into(),
        }
    }

    /// Create a delete operation
    pub fn delete(start: usize, end: usize, deleted_text: impl Into<String>) -> Self {
        Self::Delete {
            start,
            end,
            deleted_text: deleted_text.into(),
        }
    }

    /// Create a replace operation
    pub fn replace(
        start: usize,
        end: usize,
        old_text: impl Into<String>,
        new_text: impl Into<String>,
    ) -> Self {
        Self::Replace {
            start,
            end,
            old_text: old_text.into(),
            new_text: new_text.into(),
        }
    }

    /// Create a composite operation from multiple operations
    pub fn composite(operations: Vec<TextOperation>) -> Self {
        Self::Composite { operations }
    }

    /// Get the inverse operation for undo
    pub fn inverse(&self) -> Self {
        match self {
            Self::Insert { offset, text } => Self::Delete {
                start: *offset,
                end: offset + text.len(),
                deleted_text: text.clone(),
            },
            Self::Delete {
                start,
                deleted_text,
                ..
            } => Self::Insert {
                offset: *start,
                text: deleted_text.clone(),
            },
            Self::Replace {
                start,
                old_text,
                new_text,
                ..
            } => Self::Replace {
                start: *start,
                end: start + new_text.len(),
                old_text: new_text.clone(),
                new_text: old_text.clone(),
            },
            Self::Composite { operations } => Self::Composite {
                operations: operations.iter().rev().map(|op| op.inverse()).collect(),
            },
        }
    }

    /// Apply this operation to text and return the result
    pub fn apply(&self, text: &str) -> OperationResult {
        match self {
            Self::Insert { offset, text: ins } => {
                let offset = (*offset).min(text.len());
                let mut result = String::with_capacity(text.len() + ins.len());
                result.push_str(&text[..offset]);
                result.push_str(ins);
                result.push_str(&text[offset..]);

                let new_offset = offset + ins.len();
                OperationResult {
                    text: result,
                    selection: SelectionRange::collapsed_at(new_offset),
                    inverse: Some(self.inverse()),
                }
            }
            Self::Delete { start, end, .. } => {
                let start = (*start).min(text.len());
                let end = (*end).min(text.len()).max(start);
                let deleted = text[start..end].to_string();

                let mut result = String::with_capacity(text.len() - (end - start));
                result.push_str(&text[..start]);
                result.push_str(&text[end..]);

                OperationResult {
                    text: result,
                    selection: SelectionRange::collapsed_at(start),
                    inverse: Some(Self::Insert {
                        offset: start,
                        text: deleted,
                    }),
                }
            }
            Self::Replace {
                start,
                end,
                new_text,
                ..
            } => {
                let start = (*start).min(text.len());
                let end = (*end).min(text.len()).max(start);
                let old = text[start..end].to_string();

                let mut result = String::with_capacity(text.len() - (end - start) + new_text.len());
                result.push_str(&text[..start]);
                result.push_str(new_text);
                result.push_str(&text[end..]);

                let new_offset = start + new_text.len();
                OperationResult {
                    text: result,
                    selection: SelectionRange::collapsed_at(new_offset),
                    inverse: Some(Self::Replace {
                        start,
                        end: start + new_text.len(),
                        old_text: new_text.clone(),
                        new_text: old,
                    }),
                }
            }
            Self::Composite { operations } => {
                let mut current_text = text.to_string();
                let mut final_selection = SelectionRange::collapsed_at(0);
                let mut inverse_ops = Vec::with_capacity(operations.len());

                for op in operations {
                    let result = op.apply(&current_text);
                    current_text = result.text;
                    final_selection = result.selection;
                    if let Some(inv) = result.inverse {
                        inverse_ops.push(inv);
                    }
                }

                inverse_ops.reverse();
                OperationResult {
                    text: current_text,
                    selection: final_selection,
                    inverse: Some(Self::Composite {
                        operations: inverse_ops,
                    }),
                }
            }
        }
    }
}

/// Text editor operations helper
pub struct TextOperations;

impl TextOperations {
    /// Insert text at the cursor position
    pub fn insert_at_cursor(text: &str, cursor_offset: usize, insert_text: &str) -> OperationResult {
        TextOperation::insert(cursor_offset, insert_text).apply(text)
    }

    /// Insert text, replacing any selected text
    pub fn insert_at_selection(
        text: &str,
        selection: &SelectionRange,
        insert_text: &str,
    ) -> OperationResult {
        if selection.is_collapsed() {
            Self::insert_at_cursor(text, selection.focus.offset, insert_text)
        } else {
            let start = selection.start_offset();
            let end = selection.end_offset();
            let old_text = text[start..end].to_string();
            TextOperation::replace(start, end, old_text, insert_text).apply(text)
        }
    }

    /// Delete one character before the cursor (backspace)
    pub fn backspace(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            // Delete selection
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        if offset == 0 {
            return None;
        }

        // Find the previous grapheme boundary
        use unicode_segmentation::UnicodeSegmentation;
        let before = &text[..offset];
        let graphemes: Vec<&str> = before.graphemes(true).collect();
        if let Some(last) = graphemes.last() {
            let start = offset - last.len();
            let deleted = text[start..offset].to_string();
            Some(TextOperation::delete(start, offset, deleted).apply(text))
        } else {
            None
        }
    }

    /// Delete one character after the cursor (delete forward)
    pub fn delete_forward(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            // Delete selection
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        if offset >= text.len() {
            return None;
        }

        // Find the next grapheme boundary
        use unicode_segmentation::UnicodeSegmentation;
        let after = &text[offset..];
        if let Some(grapheme) = after.graphemes(true).next() {
            let end = offset + grapheme.len();
            let deleted = text[offset..end].to_string();
            Some(TextOperation::delete(offset, end, deleted).apply(text))
        } else {
            None
        }
    }

    /// Delete the selected text
    pub fn delete_selection(text: &str, selection: &SelectionRange) -> OperationResult {
        let start = selection.start_offset();
        let end = selection.end_offset();
        let deleted = text[start..end].to_string();
        TextOperation::delete(start, end, deleted).apply(text)
    }

    /// Delete a word before the cursor
    pub fn delete_word_backward(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        if offset == 0 {
            return None;
        }

        use crate::selection::SelectionHelper;
        let start = SelectionHelper::prev_word_boundary(text, offset);
        let deleted = text[start..offset].to_string();
        Some(TextOperation::delete(start, offset, deleted).apply(text))
    }

    /// Delete a word after the cursor
    pub fn delete_word_forward(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        if offset >= text.len() {
            return None;
        }

        use crate::selection::SelectionHelper;
        let end = SelectionHelper::next_word_boundary(text, offset);
        let deleted = text[offset..end].to_string();
        Some(TextOperation::delete(offset, end, deleted).apply(text))
    }

    /// Delete to the start of the line
    pub fn delete_to_line_start(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        use crate::selection::SelectionHelper;
        let (line_start, _) = SelectionHelper::line_at(text, offset);

        if line_start == offset {
            return None;
        }

        let deleted = text[line_start..offset].to_string();
        Some(TextOperation::delete(line_start, offset, deleted).apply(text))
    }

    /// Delete to the end of the line
    pub fn delete_to_line_end(text: &str, selection: &SelectionRange) -> Option<OperationResult> {
        if !selection.is_collapsed() {
            return Some(Self::delete_selection(text, selection));
        }

        let offset = selection.focus.offset;
        use crate::selection::SelectionHelper;
        let (_, line_end) = SelectionHelper::line_at(text, offset);

        if line_end == offset {
            return None;
        }

        let deleted = text[offset..line_end].to_string();
        Some(TextOperation::delete(offset, line_end, deleted).apply(text))
    }

    /// Replace all text
    pub fn replace_all(text: &str, new_text: &str) -> OperationResult {
        TextOperation::replace(0, text.len(), text, new_text).apply(text)
    }

    /// Insert a newline at the cursor
    pub fn insert_newline(text: &str, selection: &SelectionRange) -> OperationResult {
        Self::insert_at_selection(text, selection, "\n")
    }

    /// Insert a tab at the cursor
    pub fn insert_tab(text: &str, selection: &SelectionRange, use_spaces: bool, tab_size: usize) -> OperationResult {
        let insert_text = if use_spaces {
            " ".repeat(tab_size)
        } else {
            "\t".to_string()
        };
        Self::insert_at_selection(text, selection, &insert_text)
    }
}

/// Describes what kind of change an operation represents
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeKind {
    /// Character insertion
    Insert,
    /// Character deletion
    Delete,
    /// Text replacement
    Replace,
    /// Multiple operations
    Composite,
}

impl TextOperation {
    /// Get the kind of change this operation represents
    pub fn kind(&self) -> ChangeKind {
        match self {
            Self::Insert { .. } => ChangeKind::Insert,
            Self::Delete { .. } => ChangeKind::Delete,
            Self::Replace { .. } => ChangeKind::Replace,
            Self::Composite { .. } => ChangeKind::Composite,
        }
    }

    /// Check if this operation can be merged with another (for undo grouping)
    pub fn can_merge_with(&self, other: &Self) -> bool {
        match (self, other) {
            // Consecutive character insertions can be merged
            (
                Self::Insert { offset: o1, text: t1 },
                Self::Insert { offset: o2, .. },
            ) => *o2 == o1 + t1.len(),
            // Consecutive backspace deletions can be merged
            (
                Self::Delete { start: s1, .. },
                Self::Delete { end: e2, .. },
            ) => *s1 == *e2,
            _ => false,
        }
    }

    /// Merge this operation with another
    pub fn merge(self, other: Self) -> Option<Self> {
        if !self.can_merge_with(&other) {
            return None;
        }

        match (self, other) {
            (
                Self::Insert { offset, text: t1 },
                Self::Insert { text: t2, .. },
            ) => Some(Self::Insert {
                offset,
                text: t1 + &t2,
            }),
            (
                Self::Delete {
                    end: e1,
                    deleted_text: d1,
                    ..
                },
                Self::Delete {
                    start: s2,
                    deleted_text: d2,
                    ..
                },
            ) => Some(Self::Delete {
                start: s2,
                end: e1,
                deleted_text: d2 + &d1,
            }),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_at_cursor() {
        let text = "Hello World";
        let result = TextOperations::insert_at_cursor(text, 5, " Beautiful");
        assert_eq!(result.text, "Hello Beautiful World");
        assert_eq!(result.selection.focus.offset, 15);
    }

    #[test]
    fn test_insert_at_start() {
        let text = "World";
        let result = TextOperations::insert_at_cursor(text, 0, "Hello ");
        assert_eq!(result.text, "Hello World");
        assert_eq!(result.selection.focus.offset, 6);
    }

    #[test]
    fn test_insert_at_end() {
        let text = "Hello";
        let result = TextOperations::insert_at_cursor(text, 5, " World");
        assert_eq!(result.text, "Hello World");
        assert_eq!(result.selection.focus.offset, 11);
    }

    #[test]
    fn test_backspace() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(5);
        let result = TextOperations::backspace(text, &selection).unwrap();
        assert_eq!(result.text, "Hell");
        assert_eq!(result.selection.focus.offset, 4);
    }

    #[test]
    fn test_backspace_at_start() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(0);
        let result = TextOperations::backspace(text, &selection);
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_forward() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(0);
        let result = TextOperations::delete_forward(text, &selection).unwrap();
        assert_eq!(result.text, "ello");
        assert_eq!(result.selection.focus.offset, 0);
    }

    #[test]
    fn test_delete_forward_at_end() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(5);
        let result = TextOperations::delete_forward(text, &selection);
        assert!(result.is_none());
    }

    #[test]
    fn test_delete_selection() {
        let text = "Hello World";
        let selection = SelectionRange::new(
            TextPosition::from_offset(5),
            TextPosition::from_offset(11),
        );
        let result = TextOperations::delete_selection(text, &selection);
        assert_eq!(result.text, "Hello");
        assert_eq!(result.selection.focus.offset, 5);
    }

    #[test]
    fn test_replace_selection() {
        let text = "Hello World";
        let selection = SelectionRange::new(
            TextPosition::from_offset(6),
            TextPosition::from_offset(11),
        );
        let result = TextOperations::insert_at_selection(text, &selection, "Universe");
        assert_eq!(result.text, "Hello Universe");
        assert_eq!(result.selection.focus.offset, 14);
    }

    #[test]
    fn test_delete_word_backward() {
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(11);
        let result = TextOperations::delete_word_backward(text, &selection).unwrap();
        assert_eq!(result.text, "Hello ");
    }

    #[test]
    fn test_delete_word_forward() {
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(0);
        let result = TextOperations::delete_word_forward(text, &selection).unwrap();
        assert_eq!(result.text, " World");
    }

    #[test]
    fn test_operation_inverse() {
        let text = "Hello";
        let op = TextOperation::insert(5, " World");
        let result = op.apply(text);
        assert_eq!(result.text, "Hello World");

        let inverse = result.inverse.unwrap();
        let restored = inverse.apply(&result.text);
        assert_eq!(restored.text, "Hello");
    }

    #[test]
    fn test_replace_all() {
        let text = "Old text";
        let result = TextOperations::replace_all(text, "New text");
        assert_eq!(result.text, "New text");
    }

    #[test]
    fn test_composite_operation() {
        let text = "Hello";
        let ops = vec![
            TextOperation::insert(5, " World"),
            TextOperation::insert(11, "!"),
        ];
        let composite = TextOperation::composite(ops);
        let result = composite.apply(text);
        assert_eq!(result.text, "Hello World!");
    }

    #[test]
    fn test_operation_can_merge() {
        let op1 = TextOperation::insert(0, "H");
        let op2 = TextOperation::insert(1, "e");
        assert!(op1.can_merge_with(&op2));

        let op3 = TextOperation::insert(5, "x");
        assert!(!op1.can_merge_with(&op3));
    }

    #[test]
    fn test_operation_merge() {
        let op1 = TextOperation::insert(0, "H");
        let op2 = TextOperation::insert(1, "ello");
        let merged = op1.merge(op2).unwrap();

        if let TextOperation::Insert { offset, text } = merged {
            assert_eq!(offset, 0);
            assert_eq!(text, "Hello");
        } else {
            panic!("Expected Insert operation");
        }
    }

    #[test]
    fn test_insert_newline() {
        let text = "Hello World";
        let selection = SelectionRange::collapsed_at(5);
        let result = TextOperations::insert_newline(text, &selection);
        assert_eq!(result.text, "Hello\n World");
    }

    #[test]
    fn test_insert_tab_spaces() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(5);
        let result = TextOperations::insert_tab(text, &selection, true, 4);
        assert_eq!(result.text, "Hello    ");
    }

    #[test]
    fn test_insert_tab_character() {
        let text = "Hello";
        let selection = SelectionRange::collapsed_at(5);
        let result = TextOperations::insert_tab(text, &selection, false, 4);
        assert_eq!(result.text, "Hello\t");
    }

    #[test]
    fn test_delete_to_line_start() {
        let text = "Hello\nWorld";
        let selection = SelectionRange::collapsed_at(9);
        let result = TextOperations::delete_to_line_start(text, &selection).unwrap();
        assert_eq!(result.text, "Hello\nld");
    }

    #[test]
    fn test_delete_to_line_end() {
        let text = "Hello\nWorld";
        let selection = SelectionRange::collapsed_at(7);
        let result = TextOperations::delete_to_line_end(text, &selection).unwrap();
        assert_eq!(result.text, "Hello\nW");
    }
}
