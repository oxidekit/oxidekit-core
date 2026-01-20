//! Selection handling for the tree view.
//!
//! This module provides selection modes and state management for tree node selection,
//! including single selection, multiple selection, and checkbox-based selection.

use crate::node::{CheckState, NodeId, TreeNode};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Selection mode for the tree view.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SelectionMode {
    /// No selection allowed.
    #[default]
    None,
    /// Only one node can be selected at a time.
    Single,
    /// Multiple nodes can be selected.
    Multiple,
    /// Checkbox-based selection with parent/child cascading.
    Checkbox,
}

impl SelectionMode {
    /// Check if selection is enabled.
    pub fn is_enabled(&self) -> bool {
        !matches!(self, Self::None)
    }

    /// Check if multiple selection is allowed.
    pub fn allows_multiple(&self) -> bool {
        matches!(self, Self::Multiple | Self::Checkbox)
    }

    /// Check if this is checkbox mode.
    pub fn is_checkbox(&self) -> bool {
        matches!(self, Self::Checkbox)
    }
}

/// State for tracking selected nodes.
#[derive(Clone, Debug, Default)]
pub struct SelectionState {
    /// Currently selected node IDs.
    selected: HashSet<NodeId>,
    /// The last selected node (for range selection).
    anchor: Option<NodeId>,
    /// Currently focused node (for keyboard navigation).
    focused: Option<NodeId>,
    /// Selection mode.
    mode: SelectionMode,
}

impl SelectionState {
    /// Create a new selection state with the given mode.
    pub fn new(mode: SelectionMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Get the selection mode.
    pub fn mode(&self) -> SelectionMode {
        self.mode
    }

    /// Set the selection mode.
    pub fn set_mode(&mut self, mode: SelectionMode) {
        self.mode = mode;
        if mode == SelectionMode::Single && self.selected.len() > 1 {
            // Keep only the anchor or first selected
            if let Some(anchor) = &self.anchor {
                self.selected.retain(|id| id == anchor);
            } else {
                let first = self.selected.iter().next().cloned();
                self.selected.clear();
                if let Some(id) = first {
                    self.selected.insert(id);
                }
            }
        } else if mode == SelectionMode::None {
            self.clear();
        }
    }

    /// Get all selected node IDs.
    pub fn selected(&self) -> &HashSet<NodeId> {
        &self.selected
    }

    /// Get the anchor node ID.
    pub fn anchor(&self) -> Option<&NodeId> {
        self.anchor.as_ref()
    }

    /// Get the focused node ID.
    pub fn focused(&self) -> Option<&NodeId> {
        self.focused.as_ref()
    }

    /// Set the focused node.
    pub fn set_focused(&mut self, id: Option<NodeId>) {
        self.focused = id;
    }

    /// Check if a node is selected.
    pub fn is_selected(&self, id: &NodeId) -> bool {
        self.selected.contains(id)
    }

    /// Check if a node is focused.
    pub fn is_focused(&self, id: &NodeId) -> bool {
        self.focused.as_ref() == Some(id)
    }

    /// Get the number of selected nodes.
    pub fn count(&self) -> usize {
        self.selected.len()
    }

    /// Check if any nodes are selected.
    pub fn has_selection(&self) -> bool {
        !self.selected.is_empty()
    }

    /// Select a single node, clearing other selections.
    pub fn select(&mut self, id: NodeId) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.selected.clear();
                self.selected.insert(id.clone());
                self.anchor = Some(id.clone());
                self.focused = Some(id);
            }
            SelectionMode::Multiple | SelectionMode::Checkbox => {
                self.selected.clear();
                self.selected.insert(id.clone());
                self.anchor = Some(id.clone());
                self.focused = Some(id);
            }
        }
    }

    /// Toggle selection of a node.
    pub fn toggle(&mut self, id: NodeId) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                if self.selected.contains(&id) {
                    self.selected.remove(&id);
                    self.anchor = None;
                } else {
                    self.selected.clear();
                    self.selected.insert(id.clone());
                    self.anchor = Some(id.clone());
                }
                self.focused = Some(id);
            }
            SelectionMode::Multiple | SelectionMode::Checkbox => {
                if self.selected.contains(&id) {
                    self.selected.remove(&id);
                } else {
                    self.selected.insert(id.clone());
                }
                self.anchor = Some(id.clone());
                self.focused = Some(id);
            }
        }
    }

    /// Add a node to the selection (for Ctrl+Click).
    pub fn add(&mut self, id: NodeId) {
        match self.mode {
            SelectionMode::None => {}
            SelectionMode::Single => {
                self.selected.clear();
                self.selected.insert(id.clone());
                self.anchor = Some(id.clone());
                self.focused = Some(id);
            }
            SelectionMode::Multiple | SelectionMode::Checkbox => {
                self.selected.insert(id.clone());
                self.anchor = Some(id.clone());
                self.focused = Some(id);
            }
        }
    }

    /// Remove a node from the selection.
    pub fn remove(&mut self, id: &NodeId) {
        self.selected.remove(id);
        if self.anchor.as_ref() == Some(id) {
            self.anchor = self.selected.iter().next().cloned();
        }
        if self.focused.as_ref() == Some(id) {
            self.focused = self.anchor.clone();
        }
    }

    /// Select a range of nodes (for Shift+Click).
    ///
    /// This requires a list of visible nodes in order to determine the range.
    pub fn select_range(&mut self, to: NodeId, visible_order: &[NodeId]) {
        if !self.mode.allows_multiple() {
            self.select(to);
            return;
        }

        let anchor = self.anchor.clone().unwrap_or_else(|| to.clone());

        // Find positions in the visible order
        let anchor_pos = visible_order.iter().position(|id| id == &anchor);
        let to_pos = visible_order.iter().position(|id| id == &to);

        match (anchor_pos, to_pos) {
            (Some(start), Some(end)) => {
                let (start, end) = if start <= end {
                    (start, end)
                } else {
                    (end, start)
                };

                self.selected.clear();
                for id in visible_order.iter().skip(start).take(end - start + 1) {
                    self.selected.insert(id.clone());
                }
                self.anchor = Some(anchor);
                self.focused = Some(to);
            }
            _ => {
                // Fallback to simple selection
                self.select(to);
            }
        }
    }

    /// Select all visible nodes.
    pub fn select_all(&mut self, visible_nodes: &[NodeId]) {
        if !self.mode.allows_multiple() {
            return;
        }
        self.selected = visible_nodes.iter().cloned().collect();
        if let Some(first) = visible_nodes.first() {
            self.anchor = Some(first.clone());
        }
    }

    /// Clear all selections.
    pub fn clear(&mut self) {
        self.selected.clear();
        self.anchor = None;
        // Keep focus
    }

    /// Clear focus.
    pub fn clear_focus(&mut self) {
        self.focused = None;
    }

    /// Select multiple nodes at once.
    pub fn select_multiple(&mut self, ids: impl IntoIterator<Item = NodeId>) {
        if !self.mode.allows_multiple() {
            // In single mode, select only the first
            if let Some(id) = ids.into_iter().next() {
                self.select(id);
            }
            return;
        }

        self.selected.clear();
        for id in ids {
            self.selected.insert(id);
        }
        self.anchor = self.selected.iter().next().cloned();
    }

    /// Get selected nodes as a sorted vector based on visible order.
    pub fn selected_in_order(&self, visible_order: &[NodeId]) -> Vec<NodeId> {
        visible_order
            .iter()
            .filter(|id| self.selected.contains(id))
            .cloned()
            .collect()
    }
}

/// Result of a checkbox operation.
#[derive(Clone, Debug)]
pub struct CheckboxResult {
    /// Nodes that were checked.
    pub checked: Vec<NodeId>,
    /// Nodes that were unchecked.
    pub unchecked: Vec<NodeId>,
    /// Nodes that became indeterminate.
    pub indeterminate: Vec<NodeId>,
}

/// Manager for checkbox selection with cascading behavior.
#[derive(Clone, Debug, Default)]
pub struct CheckboxManager {
    /// Check state for each node.
    states: std::collections::HashMap<NodeId, CheckState>,
}

impl CheckboxManager {
    /// Create a new checkbox manager.
    pub fn new() -> Self {
        Self::default()
    }

    /// Get the check state of a node.
    pub fn get_state(&self, id: &NodeId) -> CheckState {
        self.states.get(id).copied().unwrap_or(CheckState::Unchecked)
    }

    /// Set the check state of a node.
    pub fn set_state(&mut self, id: NodeId, state: CheckState) {
        if state == CheckState::Unchecked {
            self.states.remove(&id);
        } else {
            self.states.insert(id, state);
        }
    }

    /// Toggle the check state of a node.
    pub fn toggle(&mut self, id: &NodeId) -> CheckState {
        let new_state = self.get_state(id).toggle();
        self.set_state(id.clone(), new_state);
        new_state
    }

    /// Toggle a node and cascade to children and update parents.
    ///
    /// Returns the result of the operation.
    pub fn toggle_with_cascade(
        &mut self,
        node: &TreeNode,
        get_parent: impl Fn(&NodeId) -> Option<TreeNode>,
        get_children: impl Fn(&NodeId) -> Vec<TreeNode>,
    ) -> CheckboxResult {
        let new_state = self.get_state(&node.id).toggle();
        let mut result = CheckboxResult {
            checked: Vec::new(),
            unchecked: Vec::new(),
            indeterminate: Vec::new(),
        };

        // Set this node's state
        self.set_state(node.id.clone(), new_state);
        match new_state {
            CheckState::Checked => result.checked.push(node.id.clone()),
            CheckState::Unchecked => result.unchecked.push(node.id.clone()),
            CheckState::Indeterminate => result.indeterminate.push(node.id.clone()),
        }

        // Cascade to children
        self.cascade_to_children(&node.id, new_state, &get_children, &mut result);

        // Update parents
        self.update_parents(&node.id, &get_parent, &get_children, &mut result);

        result
    }

    /// Cascade check state to all descendants.
    fn cascade_to_children(
        &mut self,
        node_id: &NodeId,
        state: CheckState,
        get_children: &impl Fn(&NodeId) -> Vec<TreeNode>,
        result: &mut CheckboxResult,
    ) {
        // Don't cascade indeterminate state
        if state == CheckState::Indeterminate {
            return;
        }

        for child in get_children(node_id) {
            self.set_state(child.id.clone(), state);
            match state {
                CheckState::Checked => result.checked.push(child.id.clone()),
                CheckState::Unchecked => result.unchecked.push(child.id.clone()),
                CheckState::Indeterminate => result.indeterminate.push(child.id.clone()),
            }
            self.cascade_to_children(&child.id, state, get_children, result);
        }
    }

    /// Update parent states based on children.
    fn update_parents(
        &mut self,
        node_id: &NodeId,
        get_parent: &impl Fn(&NodeId) -> Option<TreeNode>,
        get_children: &impl Fn(&NodeId) -> Vec<TreeNode>,
        result: &mut CheckboxResult,
    ) {
        if let Some(parent) = get_parent(node_id) {
            let children = get_children(&parent.id);
            let parent_state = self.calculate_parent_state(&children);
            let old_state = self.get_state(&parent.id);

            if old_state != parent_state {
                self.set_state(parent.id.clone(), parent_state);
                match parent_state {
                    CheckState::Checked => result.checked.push(parent.id.clone()),
                    CheckState::Unchecked => result.unchecked.push(parent.id.clone()),
                    CheckState::Indeterminate => result.indeterminate.push(parent.id.clone()),
                }
            }

            // Continue up the tree
            self.update_parents(&parent.id, get_parent, get_children, result);
        }
    }

    /// Calculate the check state for a parent based on its children.
    fn calculate_parent_state(&self, children: &[TreeNode]) -> CheckState {
        if children.is_empty() {
            return CheckState::Unchecked;
        }

        let mut checked_count = 0;
        let mut unchecked_count = 0;
        let mut has_indeterminate = false;

        for child in children {
            match self.get_state(&child.id) {
                CheckState::Checked => checked_count += 1,
                CheckState::Unchecked => unchecked_count += 1,
                CheckState::Indeterminate => has_indeterminate = true,
            }
        }

        if has_indeterminate {
            CheckState::Indeterminate
        } else if unchecked_count == children.len() {
            CheckState::Unchecked
        } else if checked_count == children.len() {
            CheckState::Checked
        } else {
            CheckState::Indeterminate
        }
    }

    /// Get all checked node IDs.
    pub fn get_checked(&self) -> Vec<NodeId> {
        self.states
            .iter()
            .filter(|(_, state)| **state == CheckState::Checked)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Get all checked leaf nodes (nodes without checked children).
    pub fn get_checked_leaves(
        &self,
        get_children: impl Fn(&NodeId) -> Vec<TreeNode>,
    ) -> Vec<NodeId> {
        self.states
            .iter()
            .filter(|(id, state)| {
                **state == CheckState::Checked && {
                    let children = get_children(id);
                    children.is_empty()
                        || !children.iter().any(|c| self.get_state(&c.id) == CheckState::Checked)
                }
            })
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Clear all check states.
    pub fn clear(&mut self) {
        self.states.clear();
    }

    /// Set check state for multiple nodes.
    pub fn set_checked(&mut self, ids: impl IntoIterator<Item = NodeId>, checked: bool) {
        let state = if checked {
            CheckState::Checked
        } else {
            CheckState::Unchecked
        };
        for id in ids {
            self.set_state(id, state);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_mode() {
        assert!(!SelectionMode::None.is_enabled());
        assert!(SelectionMode::Single.is_enabled());
        assert!(SelectionMode::Multiple.is_enabled());
        assert!(SelectionMode::Checkbox.is_enabled());

        assert!(!SelectionMode::None.allows_multiple());
        assert!(!SelectionMode::Single.allows_multiple());
        assert!(SelectionMode::Multiple.allows_multiple());
        assert!(SelectionMode::Checkbox.allows_multiple());
    }

    #[test]
    fn test_selection_state_single() {
        let mut state = SelectionState::new(SelectionMode::Single);

        state.select(NodeId::from_string("a"));
        assert!(state.is_selected(&NodeId::from_string("a")));
        assert_eq!(state.count(), 1);

        state.select(NodeId::from_string("b"));
        assert!(!state.is_selected(&NodeId::from_string("a")));
        assert!(state.is_selected(&NodeId::from_string("b")));
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_selection_state_multiple() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.add(NodeId::from_string("a"));
        state.add(NodeId::from_string("b"));
        state.add(NodeId::from_string("c"));

        assert!(state.is_selected(&NodeId::from_string("a")));
        assert!(state.is_selected(&NodeId::from_string("b")));
        assert!(state.is_selected(&NodeId::from_string("c")));
        assert_eq!(state.count(), 3);
    }

    #[test]
    fn test_selection_state_toggle() {
        let mut state = SelectionState::new(SelectionMode::Multiple);

        state.toggle(NodeId::from_string("a"));
        assert!(state.is_selected(&NodeId::from_string("a")));

        state.toggle(NodeId::from_string("a"));
        assert!(!state.is_selected(&NodeId::from_string("a")));
    }

    #[test]
    fn test_selection_state_range() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let visible = vec![
            NodeId::from_string("a"),
            NodeId::from_string("b"),
            NodeId::from_string("c"),
            NodeId::from_string("d"),
            NodeId::from_string("e"),
        ];

        state.select(NodeId::from_string("b"));
        state.select_range(NodeId::from_string("d"), &visible);

        assert!(!state.is_selected(&NodeId::from_string("a")));
        assert!(state.is_selected(&NodeId::from_string("b")));
        assert!(state.is_selected(&NodeId::from_string("c")));
        assert!(state.is_selected(&NodeId::from_string("d")));
        assert!(!state.is_selected(&NodeId::from_string("e")));
    }

    #[test]
    fn test_selection_state_select_all() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        let visible = vec![
            NodeId::from_string("a"),
            NodeId::from_string("b"),
            NodeId::from_string("c"),
        ];

        state.select_all(&visible);
        assert_eq!(state.count(), 3);
    }

    #[test]
    fn test_selection_state_clear() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        state.add(NodeId::from_string("a"));
        state.add(NodeId::from_string("b"));

        state.clear();
        assert_eq!(state.count(), 0);
        assert!(!state.has_selection());
    }

    #[test]
    fn test_selection_state_focus() {
        let mut state = SelectionState::new(SelectionMode::Single);

        state.set_focused(Some(NodeId::from_string("a")));
        assert!(state.is_focused(&NodeId::from_string("a")));

        state.clear_focus();
        assert!(state.focused().is_none());
    }

    #[test]
    fn test_checkbox_manager_basic() {
        let mut manager = CheckboxManager::new();

        assert_eq!(manager.get_state(&NodeId::from_string("a")), CheckState::Unchecked);

        manager.set_state(NodeId::from_string("a"), CheckState::Checked);
        assert_eq!(manager.get_state(&NodeId::from_string("a")), CheckState::Checked);

        manager.toggle(&NodeId::from_string("a"));
        assert_eq!(manager.get_state(&NodeId::from_string("a")), CheckState::Unchecked);
    }

    #[test]
    fn test_checkbox_manager_get_checked() {
        let mut manager = CheckboxManager::new();

        manager.set_state(NodeId::from_string("a"), CheckState::Checked);
        manager.set_state(NodeId::from_string("b"), CheckState::Unchecked);
        manager.set_state(NodeId::from_string("c"), CheckState::Checked);

        let checked = manager.get_checked();
        assert_eq!(checked.len(), 2);
    }

    #[test]
    fn test_checkbox_manager_clear() {
        let mut manager = CheckboxManager::new();

        manager.set_state(NodeId::from_string("a"), CheckState::Checked);
        manager.set_state(NodeId::from_string("b"), CheckState::Checked);

        manager.clear();
        assert_eq!(manager.get_state(&NodeId::from_string("a")), CheckState::Unchecked);
        assert_eq!(manager.get_state(&NodeId::from_string("b")), CheckState::Unchecked);
    }

    #[test]
    fn test_checkbox_manager_set_checked_multiple() {
        let mut manager = CheckboxManager::new();

        manager.set_checked(
            vec![
                NodeId::from_string("a"),
                NodeId::from_string("b"),
                NodeId::from_string("c"),
            ],
            true,
        );

        assert_eq!(manager.get_checked().len(), 3);

        manager.set_checked(vec![NodeId::from_string("b")], false);
        assert_eq!(manager.get_checked().len(), 2);
    }

    #[test]
    fn test_selection_state_mode_change() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        state.add(NodeId::from_string("a"));
        state.add(NodeId::from_string("b"));
        state.add(NodeId::from_string("c"));

        state.set_mode(SelectionMode::Single);
        assert_eq!(state.count(), 1);
    }

    #[test]
    fn test_selection_remove() {
        let mut state = SelectionState::new(SelectionMode::Multiple);
        state.add(NodeId::from_string("a"));
        state.add(NodeId::from_string("b"));

        state.remove(&NodeId::from_string("a"));
        assert!(!state.is_selected(&NodeId::from_string("a")));
        assert!(state.is_selected(&NodeId::from_string("b")));
    }
}
