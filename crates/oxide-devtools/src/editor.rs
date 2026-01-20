//! Dev Editor Core
//!
//! The main dev editor system that coordinates inspection, live editing,
//! and patch generation. This module is only available with the `dev-editor` feature.

use crate::inspector::{ComponentState, InspectorState};
use crate::patch::{EditPatch, PatchHistory};
use crate::tree::{ComponentTree, NodeHandle};
use oxide_components::{ComponentSpec, Theme};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The main dev editor state
#[derive(Debug)]
pub struct DevEditor {
    /// Whether the editor is active
    pub active: bool,
    /// Inspector state
    pub inspector: InspectorState,
    /// Component tree
    pub tree: ComponentTree,
    /// Current edit mode
    pub mode: EditMode,
    /// Temporary style overrides (ephemeral, reset on reload)
    pub overrides: StyleOverrides,
    /// Patch history for undo/redo
    pub history: PatchHistory,
    /// Registered component specs
    pub specs: HashMap<String, ComponentSpec>,
    /// Current theme (for token resolution)
    pub theme: Option<Theme>,
    /// Edit session ID
    pub session_id: String,
    /// Pending changes (not yet applied)
    pub pending_changes: Vec<PendingChange>,
    /// State simulation mode
    pub simulated_state: Option<SimulatedState>,
}

impl Default for DevEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl DevEditor {
    /// Create a new dev editor
    pub fn new() -> Self {
        Self {
            active: false,
            inspector: InspectorState::default(),
            tree: ComponentTree::new(),
            mode: EditMode::Inspect,
            overrides: StyleOverrides::new(),
            history: PatchHistory::new(),
            specs: HashMap::new(),
            theme: None,
            session_id: uuid::Uuid::new_v4().to_string(),
            pending_changes: Vec::new(),
            simulated_state: None,
        }
    }

    /// Activate the dev editor
    pub fn activate(&mut self) {
        self.active = true;
        self.inspector.visible = true;
        tracing::info!("Dev editor activated, session: {}", self.session_id);
    }

    /// Deactivate the dev editor
    pub fn deactivate(&mut self) {
        self.active = false;
        self.inspector.visible = false;
        self.clear_overrides();
        tracing::info!("Dev editor deactivated");
    }

    /// Toggle the dev editor
    pub fn toggle(&mut self) {
        if self.active {
            self.deactivate();
        } else {
            self.activate();
        }
    }

    /// Set the current theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = Some(theme);
    }

    /// Register a component spec
    pub fn register_spec(&mut self, spec: ComponentSpec) {
        self.specs.insert(spec.id.clone(), spec);
    }

    /// Get a component spec by ID
    pub fn get_spec(&self, component_id: &str) -> Option<&ComponentSpec> {
        self.specs.get(component_id)
    }

    /// Set edit mode
    pub fn set_mode(&mut self, mode: EditMode) {
        self.mode = mode;
        tracing::debug!("Edit mode changed to {:?}", mode);
    }

    /// Handle component selection
    pub fn select(&mut self, handle: NodeHandle) {
        if let Some(info) = self.tree.to_component_info(handle) {
            self.inspector.select(info.id.clone());

            // Build breadcrumb
            let path = self.tree.path_to(handle);
            let breadcrumb: Vec<_> = path
                .iter()
                .filter_map(|h| self.tree.get(*h))
                .map(|n| crate::inspector::BreadcrumbItem {
                    id: n.handle.to_string(),
                    name: n.name.clone(),
                    component_type: n.component_id.clone(),
                })
                .collect();
            self.inspector.set_breadcrumb(breadcrumb);
        }
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.inspector.clear_selection();
    }

    /// Get currently selected handle
    pub fn selected_handle(&self) -> Option<NodeHandle> {
        self.inspector
            .selected
            .as_ref()
            .and_then(|id| uuid::Uuid::parse_str(id).ok())
            .map(NodeHandle::from_uuid)
    }

    /// Apply a temporary style override
    pub fn apply_override(&mut self, handle: NodeHandle, property: &str, value: StyleValueChange) {
        self.overrides.set(handle, property.to_string(), value.clone());

        // Record pending change
        self.pending_changes.push(PendingChange {
            handle,
            property: property.to_string(),
            value,
            applied: true,
        });

        tracing::debug!("Applied override: {} = {:?}", property, self.overrides.get(handle, property));
    }

    /// Remove a style override
    pub fn remove_override(&mut self, handle: NodeHandle, property: &str) {
        self.overrides.remove(handle, property);
    }

    /// Clear all overrides
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
        self.pending_changes.clear();
    }

    /// Get override for a component property
    pub fn get_override(&self, handle: NodeHandle, property: &str) -> Option<&StyleValueChange> {
        self.overrides.get(handle, property)
    }

    /// Check if a component has overrides
    pub fn has_overrides(&self, handle: NodeHandle) -> bool {
        self.overrides.has_any(handle)
    }

    /// Commit pending changes to patch
    pub fn commit_changes(&mut self, description: &str) -> Option<EditPatch> {
        if self.pending_changes.is_empty() {
            return None;
        }

        let patch = EditPatch::from_changes(&self.pending_changes, description);
        self.history.push(patch.clone());
        self.pending_changes.clear();

        tracing::info!("Committed changes: {}", description);
        Some(patch)
    }

    /// Discard pending changes
    pub fn discard_changes(&mut self) {
        for change in &self.pending_changes {
            self.overrides.remove(change.handle, &change.property);
        }
        self.pending_changes.clear();
    }

    /// Undo last change
    pub fn undo(&mut self) -> Option<EditPatch> {
        let patch = self.history.undo()?;
        self.apply_patch_reverse(&patch);
        Some(patch)
    }

    /// Redo last undone change
    pub fn redo(&mut self) -> Option<EditPatch> {
        let patch = self.history.redo()?;
        self.apply_patch(&patch);
        Some(patch)
    }

    /// Apply a patch
    fn apply_patch(&mut self, patch: &EditPatch) {
        for op in &patch.operations {
            if let Some(handle) = uuid::Uuid::parse_str(&op.component_id)
                .ok()
                .map(NodeHandle::from_uuid)
            {
                if let Some(new_value) = &op.new_value {
                    self.overrides.set(handle, op.property.clone(), new_value.clone());
                }
            }
        }
    }

    /// Apply a patch in reverse (for undo)
    fn apply_patch_reverse(&mut self, patch: &EditPatch) {
        for op in &patch.operations {
            if let Some(handle) = uuid::Uuid::parse_str(&op.component_id)
                .ok()
                .map(NodeHandle::from_uuid)
            {
                if let Some(old_value) = &op.old_value {
                    self.overrides.set(handle, op.property.clone(), old_value.clone());
                } else {
                    self.overrides.remove(handle, &op.property);
                }
            }
        }
    }

    /// Simulate a component state (hover, focus, etc.)
    pub fn simulate_state(&mut self, handle: NodeHandle, state: ComponentState) {
        self.simulated_state = Some(SimulatedState { handle, state });
    }

    /// Clear state simulation
    pub fn clear_simulated_state(&mut self) {
        self.simulated_state = None;
    }

    /// Get simulated state for a component
    pub fn get_simulated_state(&self, handle: NodeHandle) -> Option<&ComponentState> {
        self.simulated_state
            .as_ref()
            .filter(|s| s.handle == handle)
            .map(|s| &s.state)
    }

    /// Navigate to next component in tree
    pub fn navigate_next(&mut self) {
        if let Some(handle) = self.selected_handle() {
            // Try next sibling first
            if let Some(next) = self.tree.next_sibling(handle) {
                self.select(next);
                return;
            }
            // Try first child
            if let Some(node) = self.tree.get(handle) {
                if let Some(first_child) = node.children.first() {
                    self.select(*first_child);
                    return;
                }
            }
            // Go up to parent's next sibling
            if let Some(node) = self.tree.get(handle) {
                if let Some(parent) = node.parent {
                    if let Some(next) = self.tree.next_sibling(parent) {
                        self.select(next);
                    }
                }
            }
        } else if let Some(root) = self.tree.root() {
            self.select(root);
        }
    }

    /// Navigate to previous component in tree
    pub fn navigate_prev(&mut self) {
        if let Some(handle) = self.selected_handle() {
            // Try previous sibling
            if let Some(prev) = self.tree.prev_sibling(handle) {
                // Go to last descendant of previous sibling
                let mut target = prev;
                while let Some(node) = self.tree.get(target) {
                    if let Some(last_child) = node.children.last() {
                        target = *last_child;
                    } else {
                        break;
                    }
                }
                self.select(target);
                return;
            }
            // Go to parent
            if let Some(node) = self.tree.get(handle) {
                if let Some(parent) = node.parent {
                    self.select(parent);
                }
            }
        }
    }

    /// Navigate into first child
    pub fn navigate_into(&mut self) {
        if let Some(handle) = self.selected_handle() {
            if let Some(node) = self.tree.get(handle) {
                if let Some(first_child) = node.children.first() {
                    self.select(*first_child);
                }
            }
        }
    }

    /// Navigate to parent
    pub fn navigate_out(&mut self) {
        if let Some(handle) = self.selected_handle() {
            if let Some(node) = self.tree.get(handle) {
                if let Some(parent) = node.parent {
                    self.select(parent);
                }
            }
        }
    }

    /// Get editor statistics
    pub fn stats(&self) -> EditorStats {
        EditorStats {
            active: self.active,
            component_count: self.tree.len(),
            override_count: self.overrides.count(),
            pending_change_count: self.pending_changes.len(),
            undo_available: self.history.can_undo(),
            redo_available: self.history.can_redo(),
        }
    }
}

/// Edit mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum EditMode {
    /// Inspection only (no editing)
    #[default]
    Inspect,
    /// Live editing mode
    Edit,
    /// State simulation mode
    Simulate,
    /// Measuring mode
    Measure,
}

/// Style value change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StyleValueChange {
    /// Color value (hex string)
    Color(String),
    /// Numeric value with optional unit
    Number { value: f64, unit: Option<String> },
    /// String value
    String(String),
    /// Boolean value
    Bool(bool),
    /// Token reference
    Token(String),
    /// Enum value
    Enum(String),
    /// Remove/unset the value
    Unset,
}

impl StyleValueChange {
    pub fn color(hex: impl Into<String>) -> Self {
        Self::Color(hex.into())
    }

    pub fn number(value: f64) -> Self {
        Self::Number { value, unit: None }
    }

    pub fn number_with_unit(value: f64, unit: impl Into<String>) -> Self {
        Self::Number {
            value,
            unit: Some(unit.into()),
        }
    }

    pub fn token(path: impl Into<String>) -> Self {
        Self::Token(path.into())
    }

    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Color(c) => serde_json::json!(c),
            Self::Number { value, unit } => {
                if let Some(u) = unit {
                    serde_json::json!(format!("{}{}", value, u))
                } else {
                    serde_json::json!(value)
                }
            }
            Self::String(s) => serde_json::json!(s),
            Self::Bool(b) => serde_json::json!(b),
            Self::Token(t) => serde_json::json!({ "token": t }),
            Self::Enum(e) => serde_json::json!(e),
            Self::Unset => serde_json::Value::Null,
        }
    }
}

/// Collection of style overrides
#[derive(Debug, Default)]
pub struct StyleOverrides {
    overrides: HashMap<NodeHandle, HashMap<String, StyleValueChange>>,
}

impl StyleOverrides {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, handle: NodeHandle, property: String, value: StyleValueChange) {
        self.overrides
            .entry(handle)
            .or_default()
            .insert(property, value);
    }

    pub fn get(&self, handle: NodeHandle, property: &str) -> Option<&StyleValueChange> {
        self.overrides.get(&handle)?.get(property)
    }

    pub fn remove(&mut self, handle: NodeHandle, property: &str) {
        if let Some(props) = self.overrides.get_mut(&handle) {
            props.remove(property);
            if props.is_empty() {
                self.overrides.remove(&handle);
            }
        }
    }

    pub fn has_any(&self, handle: NodeHandle) -> bool {
        self.overrides
            .get(&handle)
            .map(|p| !p.is_empty())
            .unwrap_or(false)
    }

    pub fn clear(&mut self) {
        self.overrides.clear();
    }

    pub fn count(&self) -> usize {
        self.overrides.values().map(|p| p.len()).sum()
    }

    pub fn all_for(&self, handle: NodeHandle) -> Option<&HashMap<String, StyleValueChange>> {
        self.overrides.get(&handle)
    }
}

/// Pending change (not yet committed)
#[derive(Debug, Clone)]
pub struct PendingChange {
    pub handle: NodeHandle,
    pub property: String,
    pub value: StyleValueChange,
    pub applied: bool,
}

/// Simulated component state
#[derive(Debug, Clone)]
pub struct SimulatedState {
    pub handle: NodeHandle,
    pub state: ComponentState,
}

/// Editor statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EditorStats {
    pub active: bool,
    pub component_count: usize,
    pub override_count: usize,
    pub pending_change_count: usize,
    pub undo_available: bool,
    pub redo_available: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tree::ComponentNode;

    #[test]
    fn test_editor_lifecycle() {
        let mut editor = DevEditor::new();
        assert!(!editor.active);

        editor.activate();
        assert!(editor.active);
        assert!(editor.inspector.visible);

        editor.deactivate();
        assert!(!editor.active);
    }

    #[test]
    fn test_style_overrides() {
        let mut overrides = StyleOverrides::new();
        let handle = NodeHandle::new();

        overrides.set(handle, "color".to_string(), StyleValueChange::color("#FF0000"));
        assert!(overrides.has_any(handle));

        let color = overrides.get(handle, "color").unwrap();
        assert!(matches!(color, StyleValueChange::Color(_)));

        overrides.remove(handle, "color");
        assert!(!overrides.has_any(handle));
    }

    #[test]
    fn test_editor_selection() {
        let mut editor = DevEditor::new();

        let node = ComponentNode::new("ui.Button");
        let handle = editor.tree.add(node);

        editor.select(handle);
        assert_eq!(editor.selected_handle(), Some(handle));

        editor.clear_selection();
        assert!(editor.selected_handle().is_none());
    }

    #[test]
    fn test_pending_changes() {
        let mut editor = DevEditor::new();
        let handle = NodeHandle::new();

        editor.apply_override(handle, "color", StyleValueChange::color("#FF0000"));
        assert_eq!(editor.pending_changes.len(), 1);
        assert!(editor.has_overrides(handle));

        editor.discard_changes();
        assert!(editor.pending_changes.is_empty());
        assert!(!editor.has_overrides(handle));
    }

    #[test]
    fn test_edit_mode() {
        let mut editor = DevEditor::new();
        assert_eq!(editor.mode, EditMode::Inspect);

        editor.set_mode(EditMode::Edit);
        assert_eq!(editor.mode, EditMode::Edit);
    }
}
