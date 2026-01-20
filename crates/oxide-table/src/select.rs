//! Selection handling for DataTable.
//!
//! This module provides row selection functionality including single selection,
//! multiple selection, select all, and selection state management.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::hash::Hash;

/// Selection mode for the table.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum SelectionMode {
    /// No selection allowed.
    #[default]
    None,
    /// Single row selection.
    Single,
    /// Multiple row selection.
    Multiple,
}

/// Selection state for the table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionState<K: Clone + Eq + Hash> {
    /// Selection mode.
    pub mode: SelectionMode,
    /// Selected row keys.
    pub selected: HashSet<K>,
    /// Last clicked row key (for shift-click range selection).
    pub last_clicked: Option<K>,
    /// Whether all visible rows are selected (for "select all" state).
    pub all_selected: bool,
    /// Keys that are excluded from "select all" (when all_selected is true).
    pub excluded: HashSet<K>,
}

impl<K: Clone + Eq + Hash> Default for SelectionState<K> {
    fn default() -> Self {
        Self {
            mode: SelectionMode::None,
            selected: HashSet::new(),
            last_clicked: None,
            all_selected: false,
            excluded: HashSet::new(),
        }
    }
}

impl<K: Clone + Eq + Hash> SelectionState<K> {
    /// Create a new selection state with the given mode.
    pub fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            selected: HashSet::new(),
            last_clicked: None,
            all_selected: false,
            excluded: HashSet::new(),
        }
    }

    /// Check if selection is enabled.
    pub fn is_enabled(&self) -> bool {
        self.mode != SelectionMode::None
    }

    /// Check if a row is selected.
    pub fn is_selected(&self, key: &K) -> bool {
        if self.all_selected {
            !self.excluded.contains(key)
        } else {
            self.selected.contains(key)
        }
    }

    /// Get the number of selected rows.
    /// Note: When all_selected is true, this returns usize::MAX as an indicator.
    /// Use `selected_count_for` with the total count for accurate numbers.
    pub fn selected_count(&self) -> usize {
        if self.all_selected {
            usize::MAX // Indicates "all" - caller should use selected_count_for
        } else {
            self.selected.len()
        }
    }

    /// Get the number of selected rows for a given total.
    pub fn selected_count_for(&self, total: usize) -> usize {
        if self.all_selected {
            total.saturating_sub(self.excluded.len())
        } else {
            self.selected.len()
        }
    }

    /// Check if no rows are selected.
    pub fn is_empty(&self) -> bool {
        !self.all_selected && self.selected.is_empty()
    }

    /// Check if all rows are selected (considering exclusions).
    pub fn is_all_selected(&self, total: usize) -> bool {
        if self.all_selected {
            self.excluded.is_empty()
        } else {
            self.selected.len() == total && total > 0
        }
    }

    /// Check if some but not all rows are selected (indeterminate state).
    pub fn is_indeterminate(&self, total: usize) -> bool {
        let count = self.selected_count_for(total);
        count > 0 && count < total
    }

    /// Select a row.
    pub fn select(&mut self, key: K) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.selected.clear();
                self.all_selected = false;
                self.excluded.clear();
                self.selected.insert(key.clone());
                self.last_clicked = Some(key);
            }
            SelectionMode::Multiple => {
                if self.all_selected {
                    self.excluded.remove(&key);
                } else {
                    self.selected.insert(key.clone());
                }
                self.last_clicked = Some(key);
            }
        }
    }

    /// Deselect a row.
    pub fn deselect(&mut self, key: &K) {
        if self.all_selected {
            self.excluded.insert(key.clone());
        } else {
            self.selected.remove(key);
        }
    }

    /// Toggle selection of a row.
    pub fn toggle(&mut self, key: K) {
        if self.is_selected(&key) {
            self.deselect(&key);
        } else {
            self.select(key);
        }
    }

    /// Handle click with modifiers for advanced selection.
    pub fn handle_click(&mut self, key: K, shift: bool, ctrl_or_cmd: bool, all_keys: &[K]) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.select(key);
            }
            SelectionMode::Multiple => {
                if shift {
                    // Range selection
                    if let Some(ref last) = self.last_clicked {
                        self.select_range(last.clone(), key.clone(), all_keys);
                    } else {
                        self.select(key);
                    }
                } else if ctrl_or_cmd {
                    // Toggle single item
                    self.toggle(key);
                } else {
                    // Single click without modifiers - select only this row
                    self.clear();
                    self.select(key);
                }
            }
        }
    }

    /// Select a range of rows (for shift-click).
    pub fn select_range(&mut self, from: K, to: K, all_keys: &[K]) {
        // Find indices of from and to
        let from_idx = all_keys.iter().position(|k| k == &from);
        let to_idx = all_keys.iter().position(|k| k == &to);

        if let (Some(from_idx), Some(to_idx)) = (from_idx, to_idx) {
            let (start, end) = if from_idx <= to_idx {
                (from_idx, to_idx)
            } else {
                (to_idx, from_idx)
            };

            // Select all rows in range
            for key in all_keys.iter().skip(start).take(end - start + 1) {
                if self.all_selected {
                    self.excluded.remove(key);
                } else {
                    self.selected.insert(key.clone());
                }
            }

            self.last_clicked = Some(to);
        }
    }

    /// Select all rows.
    pub fn select_all(&mut self) {
        if self.mode == SelectionMode::Multiple {
            self.all_selected = true;
            self.selected.clear();
            self.excluded.clear();
        }
    }

    /// Deselect all rows.
    pub fn deselect_all(&mut self) {
        self.clear();
    }

    /// Toggle select all / deselect all.
    pub fn toggle_all(&mut self, total: usize) {
        if self.is_all_selected(total) {
            self.deselect_all();
        } else {
            self.select_all();
        }
    }

    /// Clear all selections.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.all_selected = false;
        self.excluded.clear();
    }

    /// Get all selected keys (only works when not using all_selected mode).
    pub fn get_selected(&self) -> Vec<K> {
        self.selected.iter().cloned().collect()
    }

    /// Get selected keys for a given set of all keys.
    pub fn get_selected_from(&self, all_keys: &[K]) -> Vec<K> {
        if self.all_selected {
            all_keys
                .iter()
                .filter(|k| !self.excluded.contains(k))
                .cloned()
                .collect()
        } else {
            self.selected.iter().cloned().collect()
        }
    }

    /// Set selection from a list of keys.
    pub fn set_selected(&mut self, keys: impl IntoIterator<Item = K>) {
        self.clear();
        for key in keys {
            self.selected.insert(key);
        }
    }
}

/// Selection event data.
#[derive(Debug, Clone)]
pub struct SelectionEvent<K: Clone> {
    /// The row key that was clicked.
    pub key: K,
    /// Whether the row is now selected.
    pub selected: bool,
    /// All currently selected keys.
    pub all_selected: Vec<K>,
    /// Whether shift was held.
    pub shift: bool,
    /// Whether ctrl/cmd was held.
    pub ctrl_or_cmd: bool,
}

/// Selection change callback type.
pub type OnSelectionChange<K> = Box<dyn Fn(&SelectionEvent<K>) + Send + Sync>;

/// Row checkbox state for rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckboxState {
    /// Not checked.
    Unchecked,
    /// Checked.
    Checked,
    /// Indeterminate (some children selected).
    Indeterminate,
}

impl CheckboxState {
    /// Create checkbox state from selection state.
    pub fn from_selection<K: Clone + Eq + Hash>(
        selection: &SelectionState<K>,
        key: &K,
    ) -> Self {
        if selection.is_selected(key) {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        }
    }

    /// Create header checkbox state.
    pub fn header_state<K: Clone + Eq + Hash>(
        selection: &SelectionState<K>,
        total: usize,
    ) -> Self {
        if selection.is_all_selected(total) {
            CheckboxState::Checked
        } else if selection.is_indeterminate(total) {
            CheckboxState::Indeterminate
        } else {
            CheckboxState::Unchecked
        }
    }
}

/// Selection toolbar data for bulk actions.
#[derive(Debug, Clone)]
pub struct SelectionToolbar {
    /// Number of selected items.
    pub count: usize,
    /// Total number of items.
    pub total: usize,
    /// Whether all are selected.
    pub all_selected: bool,
}

impl SelectionToolbar {
    /// Create toolbar data from selection state.
    pub fn from_selection<K: Clone + Eq + Hash>(
        selection: &SelectionState<K>,
        total: usize,
    ) -> Self {
        Self {
            count: selection.selected_count_for(total),
            total,
            all_selected: selection.is_all_selected(total),
        }
    }

    /// Get the display text for the selection.
    pub fn display_text(&self) -> String {
        if self.all_selected {
            format!("All {} items selected", self.total)
        } else if self.count == 1 {
            "1 item selected".to_string()
        } else {
            format!("{} items selected", self.count)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_mode() {
        let state = SelectionState::<u32>::new(SelectionMode::None);
        assert!(!state.is_enabled());

        let state = SelectionState::<u32>::new(SelectionMode::Single);
        assert!(state.is_enabled());

        let state = SelectionState::<u32>::new(SelectionMode::Multiple);
        assert!(state.is_enabled());
    }

    #[test]
    fn test_single_selection() {
        let mut state = SelectionState::new(SelectionMode::Single);

        state.select(1);
        assert!(state.is_selected(&1));
        assert!(!state.is_selected(&2));

        state.select(2);
        assert!(!state.is_selected(&1)); // Single mode clears previous
        assert!(state.is_selected(&2));
    }

    #[test]
    fn test_multiple_selection() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);

        assert!(state.is_selected(&1));
        assert!(state.is_selected(&2));
        assert_eq!(state.selected_count(), 2);
    }

    #[test]
    fn test_toggle_selection() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.toggle(1);
        assert!(state.is_selected(&1));

        state.toggle(1);
        assert!(!state.is_selected(&1));
    }

    #[test]
    fn test_select_all() {
        let mut state = SelectionState::<u32>::new(SelectionMode::Multiple);

        state.select_all();
        assert!(state.all_selected);
        assert!(state.is_selected(&1));
        assert!(state.is_selected(&999));
    }

    #[test]
    fn test_select_all_with_exclusions() {
        let mut state = SelectionState::<u32>::new(SelectionMode::Multiple);

        state.select_all();
        state.deselect(&5);

        assert!(state.is_selected(&1));
        assert!(!state.is_selected(&5));
        assert!(state.excluded.contains(&5));
    }

    #[test]
    fn test_selected_count_for() {
        let mut state = SelectionState::<u32>::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);
        assert_eq!(state.selected_count_for(10), 2);

        state.select_all();
        assert_eq!(state.selected_count_for(10), 10);

        state.deselect(&5);
        assert_eq!(state.selected_count_for(10), 9);
    }

    #[test]
    fn test_is_all_selected() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);
        state.select(3);

        assert!(state.is_all_selected(3));
        assert!(!state.is_all_selected(4));
    }

    #[test]
    fn test_is_indeterminate() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);

        assert!(!state.is_indeterminate(2)); // All selected
        assert!(state.is_indeterminate(3)); // Some selected
        assert!(!state.is_indeterminate(0)); // Edge case
    }

    #[test]
    fn test_range_selection() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let keys = vec![1, 2, 3, 4, 5];

        state.select(2);
        state.select_range(2, 4, &keys);

        assert!(!state.is_selected(&1));
        assert!(state.is_selected(&2));
        assert!(state.is_selected(&3));
        assert!(state.is_selected(&4));
        assert!(!state.is_selected(&5));
    }

    #[test]
    fn test_handle_click_ctrl() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let keys = vec![1, 2, 3];

        state.handle_click(1, false, false, &keys);
        state.handle_click(2, false, true, &keys); // Ctrl-click

        assert!(state.is_selected(&1));
        assert!(state.is_selected(&2));
    }

    #[test]
    fn test_handle_click_shift() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let keys = vec![1, 2, 3, 4, 5];

        state.handle_click(1, false, false, &keys);
        state.handle_click(4, true, false, &keys); // Shift-click

        assert!(state.is_selected(&1));
        assert!(state.is_selected(&2));
        assert!(state.is_selected(&3));
        assert!(state.is_selected(&4));
        assert!(!state.is_selected(&5));
    }

    #[test]
    fn test_get_selected_from() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let keys = vec![1, 2, 3, 4, 5];

        state.select_all();
        state.deselect(&3);

        let selected = state.get_selected_from(&keys);
        assert_eq!(selected.len(), 4);
        assert!(!selected.contains(&3));
    }

    #[test]
    fn test_clear() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);
        state.clear();

        assert!(state.is_empty());
        assert_eq!(state.selected_count(), 0);
    }

    #[test]
    fn test_checkbox_state() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);

        assert_eq!(CheckboxState::from_selection(&state, &1), CheckboxState::Checked);
        assert_eq!(CheckboxState::from_selection(&state, &2), CheckboxState::Unchecked);
    }

    #[test]
    fn test_header_checkbox_state() {
        let mut state = SelectionState::<u32>::new(SelectionMode::Multiple);

        assert_eq!(CheckboxState::header_state(&state, 5), CheckboxState::Unchecked);

        state.select(1);
        state.select(2);
        assert_eq!(CheckboxState::header_state(&state, 5), CheckboxState::Indeterminate);

        state.select_all();
        assert_eq!(CheckboxState::header_state(&state, 5), CheckboxState::Checked);
    }

    #[test]
    fn test_selection_toolbar() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);

        let toolbar = SelectionToolbar::from_selection(&state, 10);
        assert_eq!(toolbar.count, 2);
        assert_eq!(toolbar.total, 10);
        assert!(!toolbar.all_selected);
        assert_eq!(toolbar.display_text(), "2 items selected");
    }

    #[test]
    fn test_selection_toolbar_single() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        state.select(1);

        let toolbar = SelectionToolbar::from_selection(&state, 10);
        assert_eq!(toolbar.display_text(), "1 item selected");
    }

    #[test]
    fn test_selection_toolbar_all() {
        let mut state = SelectionState::<u32>::new(SelectionMode::Multiple);
        state.select_all();

        let toolbar = SelectionToolbar::from_selection(&state, 100);
        assert!(toolbar.all_selected);
        assert_eq!(toolbar.display_text(), "All 100 items selected");
    }

    #[test]
    fn test_set_selected() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.set_selected(vec![1, 3, 5]);

        assert!(state.is_selected(&1));
        assert!(!state.is_selected(&2));
        assert!(state.is_selected(&3));
        assert!(!state.is_selected(&4));
        assert!(state.is_selected(&5));
    }

    #[test]
    fn test_toggle_all() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.select(1);
        state.select(2);

        state.toggle_all(5); // Not all selected, so select all
        assert!(state.all_selected);

        state.toggle_all(5); // All selected, so deselect all
        assert!(!state.all_selected);
        assert!(state.is_empty());
    }
}
