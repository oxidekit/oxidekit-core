//! Accessibility Tree and Node Management
//!
//! This module provides the core accessibility tree structure that mirrors
//! the component tree and exposes it to assistive technologies.
//!
//! The accessibility tree is a hierarchical representation of the UI
//! with semantic information about each element's role, name, state,
//! and relationships.

use crate::role::Role;
use crate::state::{AccessibilityState, StateFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Unique identifier for accessibility nodes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(Uuid);

impl NodeId {
    /// Create a new unique node ID
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create a node ID from a UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the underlying UUID
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Bounding rectangle for an accessibility node
#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
pub struct Bounds {
    /// X position (left edge)
    pub x: f32,
    /// Y position (top edge)
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
}

impl Bounds {
    /// Create new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Create bounds at origin with given size
    pub fn from_size(width: f32, height: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width,
            height,
        }
    }

    /// Check if a point is within bounds
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }

    /// Get the center point
    pub fn center(&self) -> (f32, f32) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }

    /// Check if this bounds intersects another
    pub fn intersects(&self, other: &Bounds) -> bool {
        self.x < other.x + other.width
            && self.x + self.width > other.x
            && self.y < other.y + other.height
            && self.y + self.height > other.y
    }

    /// Get the right edge
    pub fn right(&self) -> f32 {
        self.x + self.width
    }

    /// Get the bottom edge
    pub fn bottom(&self) -> f32 {
        self.y + self.height
    }
}

/// Text direction for the node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TextDirection {
    /// Left-to-right
    #[default]
    Ltr,
    /// Right-to-left
    Rtl,
}

/// Actions that can be performed on a node
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Action {
    /// Click/activate the element
    Click,
    /// Focus the element
    Focus,
    /// Clear focus from the element
    Blur,
    /// Select the element
    Select,
    /// Scroll to make visible
    ScrollIntoView,
    /// Scroll up
    ScrollUp,
    /// Scroll down
    ScrollDown,
    /// Scroll left
    ScrollLeft,
    /// Scroll right
    ScrollRight,
    /// Show context menu
    ShowContextMenu,
    /// Set value
    SetValue,
    /// Increment value
    Increment,
    /// Decrement value
    Decrement,
    /// Expand
    Expand,
    /// Collapse
    Collapse,
}

/// An accessibility node in the tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityNode {
    /// Unique identifier
    pub id: NodeId,

    /// Semantic role
    pub role: Role,

    /// Accessible name (label)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Accessible value (for inputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,

    /// Accessible description
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Placeholder text (for inputs)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub placeholder: Option<String>,

    /// Accessibility state
    #[serde(default)]
    pub state: AccessibilityState,

    /// Bounding rectangle in screen coordinates
    #[serde(default)]
    pub bounds: Bounds,

    /// Tab index (-1 = not in tab order, 0+ = in tab order)
    #[serde(default)]
    pub tab_index: i32,

    /// Parent node ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<NodeId>,

    /// Child node IDs in order
    #[serde(default)]
    pub children: Vec<NodeId>,

    /// ID of element this node controls
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controls: Option<NodeId>,

    /// ID of element that labels this node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub labelled_by: Option<NodeId>,

    /// ID of element that describes this node
    #[serde(skip_serializing_if = "Option::is_none")]
    pub described_by: Option<NodeId>,

    /// ID of the active descendant (for composite widgets)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_descendant: Option<NodeId>,

    /// Text direction
    #[serde(default)]
    pub text_direction: TextDirection,

    /// Keyboard shortcut for this element
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keyboard_shortcut: Option<String>,

    /// Available actions
    #[serde(default)]
    pub actions: Vec<Action>,

    /// Whether this node is in the accessibility tree
    #[serde(default = "default_true")]
    pub in_tree: bool,

    /// Custom data (for platform-specific information)
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub custom_data: HashMap<String, String>,
}

fn default_true() -> bool {
    true
}

impl AccessibilityNode {
    /// Create a new accessibility node
    pub fn new(role: Role) -> Self {
        let mut state = AccessibilityState::new();
        if role.is_interactive() {
            state.flags |= StateFlags::FOCUSABLE;
        }

        Self {
            id: NodeId::new(),
            role,
            name: None,
            value: None,
            description: None,
            placeholder: None,
            state,
            bounds: Bounds::default(),
            tab_index: if role.is_interactive() { 0 } else { -1 },
            parent: None,
            children: Vec::new(),
            controls: None,
            labelled_by: None,
            described_by: None,
            active_descendant: None,
            text_direction: TextDirection::Ltr,
            keyboard_shortcut: None,
            actions: Self::default_actions_for_role(role),
            in_tree: true,
            custom_data: HashMap::new(),
        }
    }

    /// Create a node with a specific ID
    pub fn with_id(mut self, id: NodeId) -> Self {
        self.id = id;
        self
    }

    /// Set the accessible name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    /// Set the accessible value
    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set the accessible description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the bounds
    pub fn with_bounds(mut self, bounds: Bounds) -> Self {
        self.bounds = bounds;
        self
    }

    /// Set the tab index
    pub fn with_tab_index(mut self, index: i32) -> Self {
        self.tab_index = index;
        self
    }

    /// Set the state
    pub fn with_state(mut self, state: AccessibilityState) -> Self {
        self.state = state;
        self
    }

    /// Set the keyboard shortcut
    pub fn with_keyboard_shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.keyboard_shortcut = Some(shortcut.into());
        self
    }

    /// Add a child node ID
    pub fn add_child(&mut self, child_id: NodeId) {
        self.children.push(child_id);
    }

    /// Remove a child node ID
    pub fn remove_child(&mut self, child_id: &NodeId) -> bool {
        if let Some(pos) = self.children.iter().position(|id| id == child_id) {
            self.children.remove(pos);
            true
        } else {
            false
        }
    }

    /// Check if the node is focusable
    pub fn is_focusable(&self) -> bool {
        self.tab_index >= 0 && self.state.is_focusable()
    }

    /// Check if the node can receive keyboard focus
    pub fn can_receive_focus(&self) -> bool {
        self.is_focusable() && self.in_tree
    }

    /// Get the accessible name, computing from various sources
    pub fn computed_name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    /// Check if the node has any children
    pub fn has_children(&self) -> bool {
        !self.children.is_empty()
    }

    /// Check if the node supports a specific action
    pub fn supports_action(&self, action: Action) -> bool {
        self.actions.contains(&action)
    }

    /// Get default actions for a role
    fn default_actions_for_role(role: Role) -> Vec<Action> {
        let mut actions = vec![Action::ScrollIntoView];

        if role.is_interactive() {
            actions.push(Action::Focus);
            actions.push(Action::Click);
        }

        match role {
            Role::Button | Role::Link | Role::Menuitem | Role::Tab => {
                actions.push(Action::Click);
            }
            Role::Checkbox | Role::Switch | Role::Radio => {
                actions.push(Action::Click);
                actions.push(Action::Select);
            }
            Role::Textbox | Role::Searchbox => {
                actions.push(Action::SetValue);
            }
            Role::Slider | Role::Spinbutton => {
                actions.push(Action::Increment);
                actions.push(Action::Decrement);
                actions.push(Action::SetValue);
            }
            Role::Combobox | Role::Tree | Role::Treeitem => {
                actions.push(Action::Expand);
                actions.push(Action::Collapse);
            }
            Role::Scrollbar => {
                actions.push(Action::ScrollUp);
                actions.push(Action::ScrollDown);
            }
            _ => {}
        }

        actions
    }
}

/// Builder for constructing accessibility nodes
pub struct NodeBuilder {
    node: AccessibilityNode,
}

impl NodeBuilder {
    /// Create a new builder for the given role
    pub fn new(role: Role) -> Self {
        Self {
            node: AccessibilityNode::new(role),
        }
    }

    /// Set the node ID
    pub fn id(mut self, id: NodeId) -> Self {
        self.node.id = id;
        self
    }

    /// Set the accessible name
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.node.name = Some(name.into());
        self
    }

    /// Set the accessible value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.node.value = Some(value.into());
        self
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.node.description = Some(desc.into());
        self
    }

    /// Set the bounds
    pub fn bounds(mut self, x: f32, y: f32, width: f32, height: f32) -> Self {
        self.node.bounds = Bounds::new(x, y, width, height);
        self
    }

    /// Set the tab index
    pub fn tab_index(mut self, index: i32) -> Self {
        self.node.tab_index = index;
        self
    }

    /// Mark as disabled
    pub fn disabled(mut self) -> Self {
        self.node.state.set_disabled(true);
        self
    }

    /// Mark as expanded
    pub fn expanded(mut self) -> Self {
        self.node.state.set_expanded(true);
        self
    }

    /// Mark as selected
    pub fn selected(mut self) -> Self {
        self.node.state.set_selected(true);
        self
    }

    /// Set the keyboard shortcut
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.node.keyboard_shortcut = Some(shortcut.into());
        self
    }

    /// Add a child ID
    pub fn child(mut self, child_id: NodeId) -> Self {
        self.node.children.push(child_id);
        self
    }

    /// Set the controls relationship
    pub fn controls(mut self, target: NodeId) -> Self {
        self.node.controls = Some(target);
        self
    }

    /// Set the labelled-by relationship
    pub fn labelled_by(mut self, label_id: NodeId) -> Self {
        self.node.labelled_by = Some(label_id);
        self
    }

    /// Set the described-by relationship
    pub fn described_by(mut self, desc_id: NodeId) -> Self {
        self.node.described_by = Some(desc_id);
        self
    }

    /// Build the node
    pub fn build(self) -> AccessibilityNode {
        self.node
    }
}

/// The accessibility tree containing all nodes
#[derive(Debug, Default)]
pub struct AccessibilityTree {
    /// All nodes indexed by ID
    nodes: HashMap<NodeId, AccessibilityNode>,

    /// Root node ID
    root_id: Option<NodeId>,

    /// Landmark nodes for quick navigation
    landmarks: Vec<NodeId>,

    /// Heading nodes for quick navigation
    headings: Vec<NodeId>,

    /// Focusable nodes in tab order
    focus_order: Vec<NodeId>,

    /// Dirty flag for focus order rebuild
    focus_order_dirty: bool,
}

impl AccessibilityTree {
    /// Create a new empty tree
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the root node
    pub fn set_root(&mut self, node: AccessibilityNode) {
        self.root_id = Some(node.id);
        self.insert_node(node);
    }

    /// Get the root node ID
    pub fn root_id(&self) -> Option<NodeId> {
        self.root_id
    }

    /// Get the root node
    pub fn root(&self) -> Option<&AccessibilityNode> {
        self.root_id.and_then(|id| self.nodes.get(&id))
    }

    /// Insert a node into the tree
    pub fn insert_node(&mut self, node: AccessibilityNode) {
        // Track landmarks
        if node.role.is_landmark() {
            self.landmarks.push(node.id);
        }

        // Track headings
        if node.role == Role::Heading {
            self.headings.push(node.id);
        }

        self.focus_order_dirty = true;
        self.nodes.insert(node.id, node);
    }

    /// Insert a node as a child of a parent
    pub fn insert_child(&mut self, parent_id: NodeId, mut node: AccessibilityNode) {
        node.parent = Some(parent_id);
        let child_id = node.id;

        self.insert_node(node);

        if let Some(parent) = self.nodes.get_mut(&parent_id) {
            parent.children.push(child_id);
        }
    }

    /// Get a node by ID
    pub fn get(&self, id: NodeId) -> Option<&AccessibilityNode> {
        self.nodes.get(&id)
    }

    /// Get a mutable node by ID
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut AccessibilityNode> {
        self.focus_order_dirty = true;
        self.nodes.get_mut(&id)
    }

    /// Remove a node and all its descendants
    pub fn remove(&mut self, id: NodeId) -> Option<AccessibilityNode> {
        // Remove from parent's children list
        if let Some(node) = self.nodes.get(&id) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get_mut(&parent_id) {
                    parent.remove_child(&id);
                }
            }
        }

        // Remove descendants
        if let Some(node) = self.nodes.get(&id) {
            let children: Vec<NodeId> = node.children.clone();
            for child_id in children {
                self.remove(child_id);
            }
        }

        // Remove from tracking lists
        self.landmarks.retain(|l| *l != id);
        self.headings.retain(|h| *h != id);
        self.focus_order_dirty = true;

        self.nodes.remove(&id)
    }

    /// Get all landmark nodes
    pub fn landmarks(&self) -> impl Iterator<Item = &AccessibilityNode> {
        self.landmarks.iter().filter_map(|id| self.nodes.get(id))
    }

    /// Get all heading nodes
    pub fn headings(&self) -> impl Iterator<Item = &AccessibilityNode> {
        self.headings.iter().filter_map(|id| self.nodes.get(id))
    }

    /// Get headings at a specific level
    pub fn headings_at_level(&self, level: u8) -> impl Iterator<Item = &AccessibilityNode> {
        self.headings().filter(move |node| {
            node.state
                .level
                .map_or(false, |l| l.level() == level)
        })
    }

    /// Rebuild the focus order based on tab indices and spatial position
    pub fn rebuild_focus_order(&mut self) {
        self.focus_order.clear();

        let mut focusable: Vec<_> = self
            .nodes
            .values()
            .filter(|n| n.can_receive_focus())
            .collect();

        // Sort by: tab_index > 0 first (in order), then tab_index == 0 (by position)
        focusable.sort_by(|a, b| {
            // Positive tab indices come first, in order
            let a_positive = a.tab_index > 0;
            let b_positive = b.tab_index > 0;

            match (a_positive, b_positive) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                (true, true) => a.tab_index.cmp(&b.tab_index),
                (false, false) => {
                    // Both are 0 or less - sort by spatial position (top-left to bottom-right)
                    let a_pos = (a.bounds.y as i32, a.bounds.x as i32);
                    let b_pos = (b.bounds.y as i32, b.bounds.x as i32);
                    a_pos.cmp(&b_pos)
                }
            }
        });

        self.focus_order = focusable.into_iter().map(|n| n.id).collect();
        self.focus_order_dirty = false;
    }

    /// Get the focus order, rebuilding if necessary
    pub fn focus_order(&mut self) -> &[NodeId] {
        if self.focus_order_dirty {
            self.rebuild_focus_order();
        }
        &self.focus_order
    }

    /// Find the next focusable node after the given ID
    pub fn next_focusable(&mut self, current: NodeId) -> Option<NodeId> {
        if self.focus_order_dirty {
            self.rebuild_focus_order();
        }

        let current_idx = self.focus_order.iter().position(|id| *id == current)?;
        let next_idx = (current_idx + 1) % self.focus_order.len();
        self.focus_order.get(next_idx).copied()
    }

    /// Find the previous focusable node before the given ID
    pub fn prev_focusable(&mut self, current: NodeId) -> Option<NodeId> {
        if self.focus_order_dirty {
            self.rebuild_focus_order();
        }

        let current_idx = self.focus_order.iter().position(|id| *id == current)?;
        let prev_idx = if current_idx == 0 {
            self.focus_order.len().saturating_sub(1)
        } else {
            current_idx - 1
        };
        self.focus_order.get(prev_idx).copied()
    }

    /// Get the first focusable node
    pub fn first_focusable(&mut self) -> Option<NodeId> {
        if self.focus_order_dirty {
            self.rebuild_focus_order();
        }
        self.focus_order.first().copied()
    }

    /// Get the last focusable node
    pub fn last_focusable(&mut self) -> Option<NodeId> {
        if self.focus_order_dirty {
            self.rebuild_focus_order();
        }
        self.focus_order.last().copied()
    }

    /// Find a node by hit testing at coordinates
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeId> {
        // Find the deepest node containing the point
        let mut result: Option<(NodeId, usize)> = None;

        for node in self.nodes.values() {
            if node.in_tree && node.bounds.contains(x, y) {
                let depth = self.node_depth(node.id);
                match &result {
                    None => result = Some((node.id, depth)),
                    Some((_, current_depth)) if depth > *current_depth => {
                        result = Some((node.id, depth));
                    }
                    _ => {}
                }
            }
        }

        result.map(|(id, _)| id)
    }

    /// Get the depth of a node in the tree
    fn node_depth(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = id;

        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent_id) = node.parent {
                depth += 1;
                current = parent_id;
            } else {
                break;
            }
        }

        depth
    }

    /// Get the number of nodes in the tree
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if the tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Clear the entire tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root_id = None;
        self.landmarks.clear();
        self.headings.clear();
        self.focus_order.clear();
        self.focus_order_dirty = false;
    }

    /// Iterate over all nodes
    pub fn iter(&self) -> impl Iterator<Item = &AccessibilityNode> {
        self.nodes.values()
    }

    /// Get all children of a node
    pub fn children(&self, id: NodeId) -> impl Iterator<Item = &AccessibilityNode> {
        self.nodes
            .get(&id)
            .into_iter()
            .flat_map(|node| node.children.iter())
            .filter_map(|child_id| self.nodes.get(child_id))
    }

    /// Get ancestors of a node (from parent to root)
    pub fn ancestors(&self, id: NodeId) -> Vec<NodeId> {
        let mut ancestors = Vec::new();
        let mut current = id;

        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent_id) = node.parent {
                ancestors.push(parent_id);
                current = parent_id;
            } else {
                break;
            }
        }

        ancestors
    }

    /// Find nodes matching a predicate
    pub fn find<F>(&self, predicate: F) -> impl Iterator<Item = &AccessibilityNode>
    where
        F: Fn(&AccessibilityNode) -> bool,
    {
        self.nodes.values().filter(move |node| predicate(node))
    }

    /// Find nodes by role
    pub fn find_by_role(&self, role: Role) -> impl Iterator<Item = &AccessibilityNode> {
        self.find(move |node| node.role == role)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_uniqueness() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10.0, 20.0, 100.0, 50.0);
        assert!(bounds.contains(50.0, 40.0));
        assert!(!bounds.contains(5.0, 40.0));
        assert!(!bounds.contains(50.0, 100.0));
    }

    #[test]
    fn test_bounds_intersects() {
        let a = Bounds::new(0.0, 0.0, 100.0, 100.0);
        let b = Bounds::new(50.0, 50.0, 100.0, 100.0);
        let c = Bounds::new(200.0, 200.0, 50.0, 50.0);

        assert!(a.intersects(&b));
        assert!(!a.intersects(&c));
    }

    #[test]
    fn test_node_creation() {
        let node = AccessibilityNode::new(Role::Button).with_name("Submit");
        assert_eq!(node.role, Role::Button);
        assert_eq!(node.name, Some("Submit".to_string()));
        assert!(node.is_focusable());
        assert_eq!(node.tab_index, 0);
    }

    #[test]
    fn test_node_builder() {
        let node = NodeBuilder::new(Role::Textbox)
            .name("Email")
            .description("Enter your email address")
            .bounds(10.0, 20.0, 200.0, 30.0)
            .build();

        assert_eq!(node.role, Role::Textbox);
        assert_eq!(node.name, Some("Email".to_string()));
        assert_eq!(
            node.description,
            Some("Enter your email address".to_string())
        );
        assert_eq!(node.bounds.x, 10.0);
    }

    #[test]
    fn test_node_disabled() {
        let node = NodeBuilder::new(Role::Button).name("Disabled").disabled().build();
        assert!(node.state.is_disabled());
        assert!(!node.is_focusable());
    }

    #[test]
    fn test_tree_basic_operations() {
        let mut tree = AccessibilityTree::new();
        assert!(tree.is_empty());

        let root = AccessibilityNode::new(Role::Main).with_name("Main Content");
        let root_id = root.id;
        tree.set_root(root);

        assert!(!tree.is_empty());
        assert_eq!(tree.len(), 1);
        assert_eq!(tree.root_id(), Some(root_id));
    }

    #[test]
    fn test_tree_parent_child() {
        let mut tree = AccessibilityTree::new();

        let root = AccessibilityNode::new(Role::Main);
        let root_id = root.id;
        tree.set_root(root);

        let child = AccessibilityNode::new(Role::Button).with_name("Click Me");
        let child_id = child.id;
        tree.insert_child(root_id, child);

        assert_eq!(tree.len(), 2);

        let child_node = tree.get(child_id).unwrap();
        assert_eq!(child_node.parent, Some(root_id));

        let root_node = tree.get(root_id).unwrap();
        assert!(root_node.children.contains(&child_id));
    }

    #[test]
    fn test_tree_landmarks() {
        let mut tree = AccessibilityTree::new();

        tree.insert_node(AccessibilityNode::new(Role::Main));
        tree.insert_node(AccessibilityNode::new(Role::Navigation));
        tree.insert_node(AccessibilityNode::new(Role::Banner));
        tree.insert_node(AccessibilityNode::new(Role::Button));

        let landmarks: Vec<_> = tree.landmarks().collect();
        assert_eq!(landmarks.len(), 3);
    }

    #[test]
    fn test_tree_headings() {
        let mut tree = AccessibilityTree::new();

        let mut h1 = AccessibilityNode::new(Role::Heading).with_name("Title");
        h1.state = crate::state::AccessibilityState::heading(1);
        tree.insert_node(h1);

        let mut h2 = AccessibilityNode::new(Role::Heading).with_name("Subtitle");
        h2.state = crate::state::AccessibilityState::heading(2);
        tree.insert_node(h2);

        let headings: Vec<_> = tree.headings().collect();
        assert_eq!(headings.len(), 2);

        let h1_headings: Vec<_> = tree.headings_at_level(1).collect();
        assert_eq!(h1_headings.len(), 1);
    }

    #[test]
    fn test_tree_focus_order() {
        let mut tree = AccessibilityTree::new();

        let btn1 = AccessibilityNode::new(Role::Button)
            .with_name("First")
            .with_bounds(Bounds::new(0.0, 0.0, 50.0, 20.0));
        let btn1_id = btn1.id;
        tree.insert_node(btn1);

        let btn2 = AccessibilityNode::new(Role::Button)
            .with_name("Second")
            .with_bounds(Bounds::new(60.0, 0.0, 50.0, 20.0));
        let btn2_id = btn2.id;
        tree.insert_node(btn2);

        let focus_order = tree.focus_order();
        assert_eq!(focus_order.len(), 2);

        assert_eq!(tree.next_focusable(btn1_id), Some(btn2_id));
        assert_eq!(tree.prev_focusable(btn2_id), Some(btn1_id));
    }

    #[test]
    fn test_tree_focus_order_wraps() {
        let mut tree = AccessibilityTree::new();

        let btn1 = AccessibilityNode::new(Role::Button);
        let btn1_id = btn1.id;
        tree.insert_node(btn1);

        let btn2 = AccessibilityNode::new(Role::Button);
        let btn2_id = btn2.id;
        tree.insert_node(btn2);

        // Forward wraps from last to first
        assert_eq!(tree.next_focusable(btn2_id), Some(btn1_id));

        // Backward wraps from first to last
        assert_eq!(tree.prev_focusable(btn1_id), Some(btn2_id));
    }

    #[test]
    fn test_tree_hit_test() {
        let mut tree = AccessibilityTree::new();

        let outer = AccessibilityNode::new(Role::Group)
            .with_bounds(Bounds::new(0.0, 0.0, 200.0, 200.0));
        let outer_id = outer.id;
        tree.set_root(outer);

        let inner = AccessibilityNode::new(Role::Button)
            .with_bounds(Bounds::new(50.0, 50.0, 50.0, 50.0));
        let inner_id = inner.id;
        tree.insert_child(outer_id, inner);

        // Hit on inner element should return inner
        assert_eq!(tree.hit_test(75.0, 75.0), Some(inner_id));

        // Hit on outer but not inner should return outer
        assert_eq!(tree.hit_test(10.0, 10.0), Some(outer_id));

        // Hit outside should return None
        assert_eq!(tree.hit_test(300.0, 300.0), None);
    }

    #[test]
    fn test_tree_remove() {
        let mut tree = AccessibilityTree::new();

        let root = AccessibilityNode::new(Role::Main);
        let root_id = root.id;
        tree.set_root(root);

        let child = AccessibilityNode::new(Role::Button);
        let child_id = child.id;
        tree.insert_child(root_id, child);

        assert_eq!(tree.len(), 2);

        tree.remove(child_id);
        assert_eq!(tree.len(), 1);
        assert!(tree.get(child_id).is_none());

        // Parent's children should be updated
        let root = tree.get(root_id).unwrap();
        assert!(!root.children.contains(&child_id));
    }

    #[test]
    fn test_tree_ancestors() {
        let mut tree = AccessibilityTree::new();

        let root = AccessibilityNode::new(Role::Main);
        let root_id = root.id;
        tree.set_root(root);

        let parent = AccessibilityNode::new(Role::Navigation);
        let parent_id = parent.id;
        tree.insert_child(root_id, parent);

        let child = AccessibilityNode::new(Role::Button);
        let child_id = child.id;
        tree.insert_child(parent_id, child);

        let ancestors = tree.ancestors(child_id);
        assert_eq!(ancestors.len(), 2);
        assert_eq!(ancestors[0], parent_id);
        assert_eq!(ancestors[1], root_id);
    }

    #[test]
    fn test_tree_find_by_role() {
        let mut tree = AccessibilityTree::new();

        tree.insert_node(AccessibilityNode::new(Role::Button));
        tree.insert_node(AccessibilityNode::new(Role::Button));
        tree.insert_node(AccessibilityNode::new(Role::Link));

        let buttons: Vec<_> = tree.find_by_role(Role::Button).collect();
        assert_eq!(buttons.len(), 2);
    }

    #[test]
    fn test_node_serialization() {
        let node = NodeBuilder::new(Role::Button)
            .name("Submit")
            .description("Submit the form")
            .build();

        let json = serde_json::to_string(&node).unwrap();
        assert!(json.contains("\"role\":\"button\""));
        assert!(json.contains("\"name\":\"Submit\""));

        let parsed: AccessibilityNode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role, Role::Button);
        assert_eq!(parsed.name, Some("Submit".to_string()));
    }
}
