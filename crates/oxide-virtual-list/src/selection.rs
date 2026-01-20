//! Selection management for virtual lists.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Multi-select mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MultiSelectMode {
    /// No multi-select
    #[default]
    None,
    /// Ctrl/Cmd + click for multi-select
    Modifier,
    /// Always multi-select
    Always,
    /// Checkbox-based selection
    Checkbox,
}

/// Keyboard action for selection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyboardAction {
    /// Select next item
    SelectNext,
    /// Select previous item
    SelectPrevious,
    /// Extend selection down
    ExtendDown,
    /// Extend selection up
    ExtendUp,
    /// Select all
    SelectAll,
    /// Clear selection
    ClearSelection,
    /// Toggle current selection
    ToggleCurrent,
}

/// Selection range
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SelectionRange {
    /// Start index
    pub start: usize,
    /// End index
    pub end: usize,
}

impl SelectionRange {
    /// Create a new selection range
    pub fn new(start: usize, end: usize) -> Self {
        Self {
            start: start.min(end),
            end: start.max(end),
        }
    }

    /// Create single item range
    pub fn single(index: usize) -> Self {
        Self {
            start: index,
            end: index,
        }
    }

    /// Check if range contains index
    pub fn contains(&self, index: usize) -> bool {
        index >= self.start && index <= self.end
    }

    /// Get length of range
    pub fn len(&self) -> usize {
        self.end - self.start + 1
    }

    /// Check if range is empty
    pub fn is_empty(&self) -> bool {
        false // A range always has at least one element
    }

    /// Iterate over range
    pub fn iter(&self) -> impl Iterator<Item = usize> {
        self.start..=self.end
    }
}

/// Selection change event
#[derive(Debug, Clone)]
pub struct SelectionChange {
    /// Newly selected indices
    pub added: Vec<usize>,
    /// Deselected indices
    pub removed: Vec<usize>,
}

/// Selection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionConfig {
    /// Multi-select mode
    pub multi_select: MultiSelectMode,
    /// Enable keyboard navigation
    pub keyboard_navigation: bool,
    /// Enable range selection with shift
    pub range_selection: bool,
}

impl Default for SelectionConfig {
    fn default() -> Self {
        Self {
            multi_select: MultiSelectMode::None,
            keyboard_navigation: true,
            range_selection: true,
        }
    }
}

/// Selection state
#[derive(Debug, Clone, Default)]
pub struct SelectionState {
    /// Selected indices
    pub selected: HashSet<usize>,
    /// Focus index
    pub focus: Option<usize>,
    /// Anchor index for range selection
    pub anchor: Option<usize>,
}

impl SelectionState {
    /// Create new selection state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if index is selected
    pub fn is_selected(&self, index: usize) -> bool {
        self.selected.contains(&index)
    }

    /// Get selected count
    pub fn count(&self) -> usize {
        self.selected.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.selected.is_empty()
    }

    /// Get selected as sorted vec
    pub fn to_vec(&self) -> Vec<usize> {
        let mut v: Vec<_> = self.selected.iter().copied().collect();
        v.sort();
        v
    }
}

/// Selection controller
#[derive(Debug, Clone)]
pub struct SelectionController {
    /// Configuration
    pub config: SelectionConfig,
    /// State
    pub state: SelectionState,
    /// Total items
    pub total_items: usize,
}

impl Default for SelectionController {
    fn default() -> Self {
        Self {
            config: SelectionConfig::default(),
            state: SelectionState::default(),
            total_items: 0,
        }
    }
}

impl SelectionController {
    /// Create new selection controller
    pub fn new() -> Self {
        Self::default()
    }

    /// Set total items
    pub fn set_total(&mut self, total: usize) {
        self.total_items = total;
        // Remove selections beyond new total
        self.state.selected.retain(|&i| i < total);
    }

    /// Select single item (replacing existing selection)
    pub fn select(&mut self, index: usize) -> SelectionChange {
        let removed: Vec<_> = self.state.selected.drain().collect();
        self.state.selected.insert(index);
        self.state.focus = Some(index);
        self.state.anchor = Some(index);
        SelectionChange {
            added: vec![index],
            removed,
        }
    }

    /// Toggle selection
    pub fn toggle(&mut self, index: usize) -> SelectionChange {
        if self.state.selected.contains(&index) {
            self.state.selected.remove(&index);
            SelectionChange {
                added: vec![],
                removed: vec![index],
            }
        } else {
            self.state.selected.insert(index);
            SelectionChange {
                added: vec![index],
                removed: vec![],
            }
        }
    }

    /// Select range from anchor
    pub fn select_range(&mut self, index: usize) -> SelectionChange {
        let anchor = self.state.anchor.unwrap_or(index);
        let range = SelectionRange::new(anchor, index);
        let added: Vec<_> = range.iter().filter(|i| !self.state.selected.contains(i)).collect();
        for i in range.iter() {
            self.state.selected.insert(i);
        }
        self.state.focus = Some(index);
        SelectionChange {
            added,
            removed: vec![],
        }
    }

    /// Select all
    pub fn select_all(&mut self) -> SelectionChange {
        let added: Vec<_> = (0..self.total_items)
            .filter(|i| !self.state.selected.contains(i))
            .collect();
        self.state.selected = (0..self.total_items).collect();
        SelectionChange {
            added,
            removed: vec![],
        }
    }

    /// Clear selection
    pub fn clear(&mut self) -> SelectionChange {
        let removed: Vec<_> = self.state.selected.drain().collect();
        SelectionChange {
            added: vec![],
            removed,
        }
    }
}
