//! Undo/Redo system for text editing
//!
//! Provides undo history management with configurable limits and operation grouping.

use crate::operations::TextOperation;
use crate::selection::SelectionRange;
use instant::Instant;

/// An entry in the undo history
#[derive(Debug, Clone)]
pub struct UndoEntry {
    /// The operation that was performed
    pub operation: TextOperation,
    /// The selection before the operation
    pub selection_before: SelectionRange,
    /// The selection after the operation
    pub selection_after: SelectionRange,
    /// Timestamp when this entry was created
    pub timestamp: Instant,
    /// Optional group ID for grouping related operations
    pub group_id: Option<u64>,
}

impl UndoEntry {
    /// Create a new undo entry
    pub fn new(
        operation: TextOperation,
        selection_before: SelectionRange,
        selection_after: SelectionRange,
    ) -> Self {
        Self {
            operation,
            selection_before,
            selection_after,
            timestamp: Instant::now(),
            group_id: None,
        }
    }

    /// Create an undo entry with a group ID
    pub fn with_group(mut self, group_id: u64) -> Self {
        self.group_id = Some(group_id);
        self
    }
}

/// Undo manager configuration
#[derive(Debug, Clone)]
pub struct UndoConfig {
    /// Maximum number of undo entries to keep
    pub max_history: usize,
    /// Time window for grouping consecutive operations (milliseconds)
    pub grouping_timeout_ms: u64,
    /// Whether to automatically group consecutive character insertions
    pub auto_group_typing: bool,
}

impl Default for UndoConfig {
    fn default() -> Self {
        Self {
            max_history: 1000,
            grouping_timeout_ms: 500,
            auto_group_typing: true,
        }
    }
}

/// Manages undo/redo history for text editing
#[derive(Debug)]
pub struct UndoManager {
    /// Undo stack
    undo_stack: Vec<UndoEntry>,
    /// Redo stack
    redo_stack: Vec<UndoEntry>,
    /// Configuration
    config: UndoConfig,
    /// Next group ID
    next_group_id: u64,
    /// Current group ID for grouping operations
    current_group_id: Option<u64>,
    /// Whether we're currently in a group
    in_group: bool,
    /// Last operation timestamp for auto-grouping
    last_operation_time: Option<Instant>,
}

impl UndoManager {
    /// Create a new undo manager with default configuration
    pub fn new() -> Self {
        Self::with_config(UndoConfig::default())
    }

    /// Create a new undo manager with custom configuration
    pub fn with_config(config: UndoConfig) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            config,
            next_group_id: 0,
            current_group_id: None,
            in_group: false,
            last_operation_time: None,
        }
    }

    /// Set the maximum history size
    pub fn set_max_history(&mut self, max: usize) {
        self.config.max_history = max;
        self.trim_history();
    }

    /// Get the maximum history size
    pub fn max_history(&self) -> usize {
        self.config.max_history
    }

    /// Record an operation for undo
    pub fn record(
        &mut self,
        operation: TextOperation,
        selection_before: SelectionRange,
        selection_after: SelectionRange,
    ) {
        // Clear redo stack when new operation is recorded
        self.redo_stack.clear();

        let now = Instant::now();
        let should_merge = self.should_merge_with_previous(&operation, now);

        if should_merge {
            // Merge with the previous operation
            if let Some(last) = self.undo_stack.last_mut() {
                if let Some(merged) = last.operation.clone().merge(operation.clone()) {
                    last.operation = merged;
                    last.selection_after = selection_after;
                    last.timestamp = now;
                    self.last_operation_time = Some(now);
                    return;
                }
            }
        }

        // Create new entry
        let mut entry = UndoEntry::new(operation, selection_before, selection_after);

        // Apply group ID if in a group
        if let Some(group_id) = self.current_group_id {
            entry = entry.with_group(group_id);
        }

        self.undo_stack.push(entry);
        self.last_operation_time = Some(now);
        self.trim_history();
    }

    /// Check if the new operation should be merged with the previous one
    fn should_merge_with_previous(&self, operation: &TextOperation, now: Instant) -> bool {
        if !self.config.auto_group_typing {
            return false;
        }

        // Check if within time window
        if let Some(last_time) = self.last_operation_time {
            let elapsed = now.duration_since(last_time).as_millis() as u64;
            if elapsed > self.config.grouping_timeout_ms {
                return false;
            }
        } else {
            return false;
        }

        // Check if operations can be merged
        if let Some(last) = self.undo_stack.last() {
            return last.operation.can_merge_with(operation);
        }

        false
    }

    /// Trim history to max size
    fn trim_history(&mut self) {
        while self.undo_stack.len() > self.config.max_history {
            self.undo_stack.remove(0);
        }
    }

    /// Start a group of operations that should be undone together
    pub fn begin_group(&mut self) {
        if !self.in_group {
            self.current_group_id = Some(self.next_group_id);
            self.next_group_id += 1;
            self.in_group = true;
        }
    }

    /// End the current group
    pub fn end_group(&mut self) {
        self.current_group_id = None;
        self.in_group = false;
    }

    /// Check if currently in a group
    pub fn in_group(&self) -> bool {
        self.in_group
    }

    /// Perform undo
    ///
    /// Returns the entries that were undone (may be multiple if grouped)
    pub fn undo(&mut self) -> Option<Vec<UndoEntry>> {
        if self.undo_stack.is_empty() {
            return None;
        }

        let mut entries = Vec::new();

        // Pop the first entry
        let first = self.undo_stack.pop()?;
        let group_id = first.group_id;
        entries.push(first);

        // If this entry has a group ID, pop all entries with the same group ID
        if let Some(gid) = group_id {
            while let Some(entry) = self.undo_stack.last() {
                if entry.group_id == Some(gid) {
                    entries.push(self.undo_stack.pop().unwrap());
                } else {
                    break;
                }
            }
        }

        // Move entries to redo stack (in reverse order for correct redo)
        for entry in entries.iter().rev() {
            self.redo_stack.push(entry.clone());
        }

        Some(entries)
    }

    /// Perform redo
    ///
    /// Returns the entries that were redone (may be multiple if grouped)
    pub fn redo(&mut self) -> Option<Vec<UndoEntry>> {
        if self.redo_stack.is_empty() {
            return None;
        }

        let mut entries = Vec::new();

        // Pop the first entry
        let first = self.redo_stack.pop()?;
        let group_id = first.group_id;
        entries.push(first);

        // If this entry has a group ID, pop all entries with the same group ID
        if let Some(gid) = group_id {
            while let Some(entry) = self.redo_stack.last() {
                if entry.group_id == Some(gid) {
                    entries.push(self.redo_stack.pop().unwrap());
                } else {
                    break;
                }
            }
        }

        // Move entries back to undo stack (in reverse order)
        for entry in entries.iter().rev() {
            self.undo_stack.push(entry.clone());
        }

        Some(entries)
    }

    /// Check if undo is available
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// Check if redo is available
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get the number of undo entries
    pub fn undo_count(&self) -> usize {
        self.undo_stack.len()
    }

    /// Get the number of redo entries
    pub fn redo_count(&self) -> usize {
        self.redo_stack.len()
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
        self.current_group_id = None;
        self.in_group = false;
        self.last_operation_time = None;
    }

    /// Get the last undo entry without removing it
    pub fn peek_undo(&self) -> Option<&UndoEntry> {
        self.undo_stack.last()
    }

    /// Get the last redo entry without removing it
    pub fn peek_redo(&self) -> Option<&UndoEntry> {
        self.redo_stack.last()
    }

    /// Reset auto-grouping timer (call when user pauses typing)
    pub fn reset_grouping(&mut self) {
        self.last_operation_time = None;
    }
}

impl Default for UndoManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply undo entries to text
pub fn apply_undo(text: &str, entries: &[UndoEntry]) -> (String, SelectionRange) {
    let mut current_text = text.to_string();
    let mut final_selection = SelectionRange::default();

    // Apply inverse operations in reverse order
    for entry in entries.iter().rev() {
        let inverse = entry.operation.inverse();
        let result = inverse.apply(&current_text);
        current_text = result.text;
        final_selection = entry.selection_before;
    }

    (current_text, final_selection)
}

/// Apply redo entries to text
pub fn apply_redo(text: &str, entries: &[UndoEntry]) -> (String, SelectionRange) {
    let mut current_text = text.to_string();
    let mut final_selection = SelectionRange::default();

    // Apply operations in reverse order (they were reversed when put on redo stack)
    for entry in entries.iter().rev() {
        let result = entry.operation.apply(&current_text);
        current_text = result.text;
        final_selection = entry.selection_after;
    }

    (current_text, final_selection)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::selection::TextPosition;

    fn create_selection(offset: usize) -> SelectionRange {
        SelectionRange::collapsed_at(offset)
    }

    #[test]
    fn test_undo_manager_new() {
        let manager = UndoManager::new();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_record_and_undo() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false; // Disable auto-grouping for this test

        let op = TextOperation::insert(0, "Hello");
        manager.record(op, create_selection(0), create_selection(5));

        assert!(manager.can_undo());
        assert_eq!(manager.undo_count(), 1);

        let entries = manager.undo().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(!manager.can_undo());
        assert!(manager.can_redo());
    }

    #[test]
    fn test_undo_and_redo() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        let op = TextOperation::insert(0, "Hello");
        manager.record(op, create_selection(0), create_selection(5));

        manager.undo();
        assert!(manager.can_redo());

        let entries = manager.redo().unwrap();
        assert_eq!(entries.len(), 1);
        assert!(manager.can_undo());
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_new_operation_clears_redo() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        manager.record(TextOperation::insert(0, "A"), create_selection(0), create_selection(1));
        manager.undo();
        assert!(manager.can_redo());

        // Recording new operation should clear redo
        manager.record(TextOperation::insert(0, "B"), create_selection(0), create_selection(1));
        assert!(!manager.can_redo());
    }

    #[test]
    fn test_group_operations() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        manager.begin_group();
        manager.record(TextOperation::insert(0, "H"), create_selection(0), create_selection(1));
        manager.record(TextOperation::insert(1, "i"), create_selection(1), create_selection(2));
        manager.end_group();

        assert_eq!(manager.undo_count(), 2);

        // Undo should undo both operations at once
        let entries = manager.undo().unwrap();
        assert_eq!(entries.len(), 2);
        assert!(!manager.can_undo());
    }

    #[test]
    fn test_max_history() {
        let mut config = UndoConfig::default();
        config.max_history = 5;
        config.auto_group_typing = false;
        let mut manager = UndoManager::with_config(config);

        for i in 0..10 {
            manager.record(
                TextOperation::insert(i, "x"),
                create_selection(i),
                create_selection(i + 1),
            );
        }

        assert_eq!(manager.undo_count(), 5);
    }

    #[test]
    fn test_clear() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        manager.record(TextOperation::insert(0, "A"), create_selection(0), create_selection(1));
        manager.undo();

        manager.clear();
        assert!(!manager.can_undo());
        assert!(!manager.can_redo());
        assert_eq!(manager.undo_count(), 0);
        assert_eq!(manager.redo_count(), 0);
    }

    #[test]
    fn test_apply_undo() {
        let text = "Hello";
        let entries = vec![UndoEntry::new(
            TextOperation::insert(0, "Hello"),
            create_selection(0),
            create_selection(5),
        )];

        let (result, selection) = apply_undo(text, &entries);
        assert_eq!(result, "");
        assert_eq!(selection.focus.offset, 0);
    }

    #[test]
    fn test_apply_redo() {
        let text = "";
        let entries = vec![UndoEntry::new(
            TextOperation::insert(0, "Hello"),
            create_selection(0),
            create_selection(5),
        )];

        let (result, selection) = apply_redo(text, &entries);
        assert_eq!(result, "Hello");
        assert_eq!(selection.focus.offset, 5);
    }

    #[test]
    fn test_peek_undo_redo() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        assert!(manager.peek_undo().is_none());
        assert!(manager.peek_redo().is_none());

        manager.record(TextOperation::insert(0, "A"), create_selection(0), create_selection(1));

        assert!(manager.peek_undo().is_some());
        assert!(manager.peek_redo().is_none());

        manager.undo();

        assert!(manager.peek_undo().is_none());
        assert!(manager.peek_redo().is_some());
    }

    #[test]
    fn test_multiple_undo_redo() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        manager.record(TextOperation::insert(0, "A"), create_selection(0), create_selection(1));
        manager.record(TextOperation::insert(1, "B"), create_selection(1), create_selection(2));
        manager.record(TextOperation::insert(2, "C"), create_selection(2), create_selection(3));

        assert_eq!(manager.undo_count(), 3);

        manager.undo();
        manager.undo();

        assert_eq!(manager.undo_count(), 1);
        assert_eq!(manager.redo_count(), 2);

        manager.redo();

        assert_eq!(manager.undo_count(), 2);
        assert_eq!(manager.redo_count(), 1);
    }

    #[test]
    fn test_in_group() {
        let mut manager = UndoManager::new();

        assert!(!manager.in_group());

        manager.begin_group();
        assert!(manager.in_group());

        manager.end_group();
        assert!(!manager.in_group());
    }

    #[test]
    fn test_selection_preservation() {
        let mut manager = UndoManager::new();
        manager.config.auto_group_typing = false;

        let sel_before = SelectionRange::new(
            TextPosition::from_offset(0),
            TextPosition::from_offset(5),
        );
        let sel_after = SelectionRange::collapsed_at(10);

        manager.record(TextOperation::insert(5, "World"), sel_before, sel_after);

        let entry = manager.peek_undo().unwrap();
        assert_eq!(entry.selection_before.start_offset(), 0);
        assert_eq!(entry.selection_before.end_offset(), 5);
        assert_eq!(entry.selection_after.focus.offset, 10);
    }
}
