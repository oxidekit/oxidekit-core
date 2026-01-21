//! Tree node types and structures.
//!
//! This module defines the core `TreeNode` struct and related types for building
//! hierarchical tree structures.

use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

/// Unique identifier for a tree node.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(String);

impl NodeId {
    /// Create a new random node ID.
    pub fn new() -> Self {
        Self(Uuid::new_v4().to_string())
    }

    /// Create a node ID from a string.
    pub fn from_string(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// Get the ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<&str> for NodeId {
    fn from(s: &str) -> Self {
        Self::from_string(s)
    }
}

impl From<String> for NodeId {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

/// Icon type for tree nodes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum NodeIcon {
    /// No icon
    None,
    /// Built-in icon by name (e.g., "folder", "file", "document")
    Named(String),
    /// Custom icon path or URL
    Custom(String),
    /// Emoji icon
    Emoji(String),
    /// Icon that changes based on node state
    Dynamic {
        /// Icon when collapsed
        collapsed: Box<NodeIcon>,
        /// Icon when expanded
        expanded: Box<NodeIcon>,
    },
}

impl Default for NodeIcon {
    fn default() -> Self {
        Self::None
    }
}

impl NodeIcon {
    /// Create a named icon.
    pub fn named(name: impl Into<String>) -> Self {
        Self::Named(name.into())
    }

    /// Create a custom icon from a path.
    pub fn custom(path: impl Into<String>) -> Self {
        Self::Custom(path.into())
    }

    /// Create an emoji icon.
    pub fn emoji(emoji: impl Into<String>) -> Self {
        Self::Emoji(emoji.into())
    }

    /// Create a dynamic icon that changes based on expand state.
    pub fn dynamic(collapsed: NodeIcon, expanded: NodeIcon) -> Self {
        Self::Dynamic {
            collapsed: Box::new(collapsed),
            expanded: Box::new(expanded),
        }
    }

    /// Get the folder icon preset.
    pub fn folder() -> Self {
        Self::dynamic(Self::named("folder"), Self::named("folder-open"))
    }

    /// Get the file icon preset.
    pub fn file() -> Self {
        Self::named("file")
    }

    /// Get the document icon preset.
    pub fn document() -> Self {
        Self::named("document")
    }
}

/// State of a tree node.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct NodeState {
    /// Whether the node is expanded.
    pub expanded: bool,
    /// Whether the node is selected.
    pub selected: bool,
    /// Whether the node is focused.
    pub focused: bool,
    /// Whether the node is being edited.
    pub editing: bool,
    /// Whether the node is loading children.
    pub loading: bool,
    /// Whether the node has an error.
    pub error: Option<String>,
    /// Whether the node is disabled.
    pub disabled: bool,
    /// Custom state data.
    pub custom: HashMap<String, String>,
}

impl NodeState {
    /// Create a new default node state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the expanded state.
    pub fn with_expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Set the selected state.
    pub fn with_selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set the focused state.
    pub fn with_focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    /// Set the disabled state.
    pub fn with_disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Configuration for how a node's children are loaded.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ChildrenConfig {
    /// Children are loaded immediately (static children).
    Immediate(Vec<TreeNode>),
    /// Children should be loaded lazily when expanded.
    Lazy {
        /// Whether children have been loaded.
        loaded: bool,
        /// Cached children after loading.
        children: Vec<TreeNode>,
    },
    /// Node cannot have children (leaf node).
    None,
}

impl Default for ChildrenConfig {
    fn default() -> Self {
        Self::Immediate(Vec::new())
    }
}

impl ChildrenConfig {
    /// Create an immediate children config with given children.
    pub fn immediate(children: Vec<TreeNode>) -> Self {
        Self::Immediate(children)
    }

    /// Create a lazy children config.
    pub fn lazy() -> Self {
        Self::Lazy {
            loaded: false,
            children: Vec::new(),
        }
    }

    /// Create a leaf node config (no children).
    pub fn none() -> Self {
        Self::None
    }

    /// Check if children are loaded.
    pub fn is_loaded(&self) -> bool {
        match self {
            Self::Immediate(_) => true,
            Self::Lazy { loaded, .. } => *loaded,
            Self::None => true,
        }
    }

    /// Check if this is a lazy config.
    pub fn is_lazy(&self) -> bool {
        matches!(self, Self::Lazy { .. })
    }

    /// Check if this is a leaf node.
    pub fn is_leaf(&self) -> bool {
        matches!(self, Self::None)
    }

    /// Get children if available.
    pub fn children(&self) -> &[TreeNode] {
        match self {
            Self::Immediate(children) => children,
            Self::Lazy { children, .. } => children,
            Self::None => &[],
        }
    }

    /// Get mutable children if available.
    pub fn children_mut(&mut self) -> &mut Vec<TreeNode> {
        match self {
            Self::Immediate(children) => children,
            Self::Lazy { children, .. } => children,
            Self::None => {
                // Convert to immediate to allow adding children
                *self = Self::Immediate(Vec::new());
                if let Self::Immediate(children) = self {
                    children
                } else {
                    unreachable!()
                }
            }
        }
    }

    /// Set lazy children as loaded.
    pub fn set_loaded(&mut self, loaded_children: Vec<TreeNode>) {
        if let Self::Lazy { loaded, children } = self {
            *loaded = true;
            *children = loaded_children;
        }
    }
}

/// Custom data that can be attached to a node.
pub trait NodeData: Send + Sync + fmt::Debug {
    /// Get the data as Any for downcasting.
    fn as_any(&self) -> &dyn Any;

    /// Get the data as mutable Any for downcasting.
    fn as_any_mut(&mut self) -> &mut dyn Any;

    /// Clone the data into a box.
    fn clone_box(&self) -> Box<dyn NodeData>;
}

impl Clone for Box<dyn NodeData> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Implement NodeData for simple types that are Clone + Send + Sync + Debug.
impl<T: Clone + Send + Sync + fmt::Debug + 'static> NodeData for T {
    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn clone_box(&self) -> Box<dyn NodeData> {
        Box::new(self.clone())
    }
}

/// A node in the tree structure.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TreeNode {
    /// Unique identifier for this node.
    pub id: NodeId,
    /// Display label for the node.
    pub label: String,
    /// Icon for the node.
    pub icon: NodeIcon,
    /// Current state of the node.
    pub state: NodeState,
    /// Children configuration.
    pub children: ChildrenConfig,
    /// Parent node ID (None for root nodes).
    pub parent_id: Option<NodeId>,
    /// Depth in the tree (0 for root nodes).
    pub depth: usize,
    /// Whether the node is draggable.
    pub draggable: bool,
    /// Whether the node accepts drops.
    pub droppable: bool,
    /// Whether the node is checkable.
    pub checkable: bool,
    /// Check state for checkbox selection.
    pub check_state: CheckState,
    /// Context menu items for this node.
    pub context_menu: Vec<ContextMenuItem>,
    /// Custom actions for this node.
    pub actions: Vec<NodeAction>,
    /// Additional metadata.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl TreeNode {
    /// Create a new tree node with the given ID and label.
    pub fn new(id: impl Into<NodeId>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: NodeIcon::default(),
            state: NodeState::default(),
            children: ChildrenConfig::default(),
            parent_id: None,
            depth: 0,
            draggable: true,
            droppable: true,
            checkable: false,
            check_state: CheckState::Unchecked,
            context_menu: Vec::new(),
            actions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Create a new tree node with a generated ID.
    pub fn with_label(label: impl Into<String>) -> Self {
        Self::new(NodeId::new(), label)
    }

    /// Set the node icon.
    pub fn icon(mut self, icon: NodeIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Set immediate children.
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = ChildrenConfig::Immediate(children);
        self
    }

    /// Configure as a lazy-loading node.
    pub fn lazy(mut self) -> Self {
        self.children = ChildrenConfig::Lazy {
            loaded: false,
            children: Vec::new(),
        };
        self
    }

    /// Configure as a leaf node (no children).
    pub fn leaf(mut self) -> Self {
        self.children = ChildrenConfig::None;
        self
    }

    /// Set the expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.state.expanded = expanded;
        self
    }

    /// Set the selected state.
    pub fn selected(mut self, selected: bool) -> Self {
        self.state.selected = selected;
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.state.disabled = disabled;
        self
    }

    /// Set whether the node is draggable.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Set whether the node accepts drops.
    pub fn droppable(mut self, droppable: bool) -> Self {
        self.droppable = droppable;
        self
    }

    /// Set whether the node is checkable.
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    /// Set context menu items.
    pub fn context_menu(mut self, items: Vec<ContextMenuItem>) -> Self {
        self.context_menu = items;
        self
    }

    /// Set node actions.
    pub fn actions(mut self, actions: Vec<NodeAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Set metadata value.
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if this node has children.
    pub fn has_children(&self) -> bool {
        match &self.children {
            ChildrenConfig::Immediate(children) => !children.is_empty(),
            ChildrenConfig::Lazy { loaded, children } => !*loaded || !children.is_empty(),
            ChildrenConfig::None => false,
        }
    }

    /// Check if this is a leaf node.
    pub fn is_leaf(&self) -> bool {
        self.children.is_leaf()
    }

    /// Check if children need to be loaded.
    pub fn needs_load(&self) -> bool {
        matches!(
            &self.children,
            ChildrenConfig::Lazy {
                loaded: false,
                ..
            }
        )
    }

    /// Get the node's children.
    pub fn get_children(&self) -> &[TreeNode] {
        self.children.children()
    }

    /// Get mutable reference to children.
    pub fn get_children_mut(&mut self) -> &mut Vec<TreeNode> {
        self.children.children_mut()
    }

    /// Add a child node.
    pub fn add_child(&mut self, mut child: TreeNode) {
        child.parent_id = Some(self.id.clone());
        child.depth = self.depth + 1;
        self.children.children_mut().push(child);
    }

    /// Remove a child by ID.
    pub fn remove_child(&mut self, child_id: &NodeId) -> Option<TreeNode> {
        let children = self.children.children_mut();
        if let Some(pos) = children.iter().position(|c| &c.id == child_id) {
            Some(children.remove(pos))
        } else {
            None
        }
    }

    /// Insert a child at a specific index.
    pub fn insert_child(&mut self, index: usize, mut child: TreeNode) {
        child.parent_id = Some(self.id.clone());
        child.depth = self.depth + 1;
        let children = self.children.children_mut();
        let index = index.min(children.len());
        children.insert(index, child);
    }

    /// Get the path from root to this node (list of labels).
    pub fn path_labels(&self) -> Vec<String> {
        // This would need the full tree to compute properly
        // For now, just return the node's own label
        vec![self.label.clone()]
    }
}

impl PartialEq for TreeNode {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for TreeNode {}

impl Hash for TreeNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state);
    }
}

/// Check state for checkbox selection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum CheckState {
    /// Not checked.
    #[default]
    Unchecked,
    /// Fully checked.
    Checked,
    /// Partially checked (some children are checked).
    Indeterminate,
}

impl CheckState {
    /// Toggle the check state.
    pub fn toggle(self) -> Self {
        match self {
            Self::Unchecked | Self::Indeterminate => Self::Checked,
            Self::Checked => Self::Unchecked,
        }
    }

    /// Check if the state is checked.
    pub fn is_checked(self) -> bool {
        matches!(self, Self::Checked)
    }

    /// Check if the state is indeterminate.
    pub fn is_indeterminate(self) -> bool {
        matches!(self, Self::Indeterminate)
    }
}

/// Context menu item for a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ContextMenuItem {
    /// Unique ID for the item.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Optional icon.
    pub icon: Option<NodeIcon>,
    /// Whether the item is disabled.
    pub disabled: bool,
    /// Whether this is a separator.
    pub separator: bool,
    /// Submenu items.
    pub submenu: Vec<ContextMenuItem>,
    /// Keyboard shortcut hint.
    pub shortcut: Option<String>,
}

impl ContextMenuItem {
    /// Create a new context menu item.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            separator: false,
            submenu: Vec::new(),
            shortcut: None,
        }
    }

    /// Create a separator.
    pub fn separator() -> Self {
        Self {
            id: String::new(),
            label: String::new(),
            icon: None,
            disabled: false,
            separator: true,
            submenu: Vec::new(),
            shortcut: None,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: NodeIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set submenu items.
    pub fn submenu(mut self, items: Vec<ContextMenuItem>) -> Self {
        self.submenu = items;
        self
    }

    /// Set keyboard shortcut.
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }
}

/// An action that can be performed on a node.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct NodeAction {
    /// Unique ID for the action.
    pub id: String,
    /// Display label.
    pub label: String,
    /// Icon for the action.
    pub icon: Option<NodeIcon>,
    /// Tooltip text.
    pub tooltip: Option<String>,
    /// Whether the action is visible.
    pub visible: bool,
    /// Whether the action is disabled.
    pub disabled: bool,
}

impl NodeAction {
    /// Create a new node action.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            tooltip: None,
            visible: true,
            disabled: false,
        }
    }

    /// Set the icon.
    pub fn icon(mut self, icon: NodeIcon) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Set the tooltip.
    pub fn tooltip(mut self, tooltip: impl Into<String>) -> Self {
        self.tooltip = Some(tooltip.into());
        self
    }

    /// Set visibility.
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Set disabled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Create an edit action.
    pub fn edit() -> Self {
        Self::new("edit", "Edit")
            .icon(NodeIcon::named("edit"))
            .tooltip("Edit this item")
    }

    /// Create a delete action.
    pub fn delete() -> Self {
        Self::new("delete", "Delete")
            .icon(NodeIcon::named("delete"))
            .tooltip("Delete this item")
    }

    /// Create an add action.
    pub fn add() -> Self {
        Self::new("add", "Add")
            .icon(NodeIcon::named("add"))
            .tooltip("Add a new item")
    }
}

/// Builder for creating tree nodes fluently.
#[derive(Default)]
pub struct TreeNodeBuilder {
    id: Option<NodeId>,
    label: String,
    icon: NodeIcon,
    expanded: bool,
    children: Vec<TreeNode>,
    lazy: bool,
    leaf: bool,
    draggable: bool,
    droppable: bool,
    checkable: bool,
    disabled: bool,
    context_menu: Vec<ContextMenuItem>,
    actions: Vec<NodeAction>,
    metadata: HashMap<String, serde_json::Value>,
}

impl TreeNodeBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self {
            draggable: true,
            droppable: true,
            ..Default::default()
        }
    }

    /// Set the node ID.
    pub fn id(mut self, id: impl Into<NodeId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the node label.
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = label.into();
        self
    }

    /// Set the node icon.
    pub fn icon(mut self, icon: NodeIcon) -> Self {
        self.icon = icon;
        self
    }

    /// Set the expanded state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Add children.
    pub fn children(mut self, children: Vec<TreeNode>) -> Self {
        self.children = children;
        self
    }

    /// Configure as lazy-loading.
    pub fn lazy(mut self) -> Self {
        self.lazy = true;
        self
    }

    /// Configure as a leaf node.
    pub fn leaf(mut self) -> Self {
        self.leaf = true;
        self
    }

    /// Set draggable.
    pub fn draggable(mut self, draggable: bool) -> Self {
        self.draggable = draggable;
        self
    }

    /// Set droppable.
    pub fn droppable(mut self, droppable: bool) -> Self {
        self.droppable = droppable;
        self
    }

    /// Set checkable.
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    /// Set disabled.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set context menu.
    pub fn context_menu(mut self, items: Vec<ContextMenuItem>) -> Self {
        self.context_menu = items;
        self
    }

    /// Set actions.
    pub fn actions(mut self, actions: Vec<NodeAction>) -> Self {
        self.actions = actions;
        self
    }

    /// Add metadata.
    pub fn metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Build the tree node.
    pub fn build(self) -> TreeNode {
        let id = self.id.unwrap_or_else(NodeId::new);
        let children = if self.leaf {
            ChildrenConfig::None
        } else if self.lazy {
            ChildrenConfig::Lazy {
                loaded: false,
                children: Vec::new(),
            }
        } else {
            ChildrenConfig::Immediate(self.children)
        };

        TreeNode {
            id,
            label: self.label,
            icon: self.icon,
            state: NodeState {
                expanded: self.expanded,
                disabled: self.disabled,
                ..Default::default()
            },
            children,
            parent_id: None,
            depth: 0,
            draggable: self.draggable,
            droppable: self.droppable,
            checkable: self.checkable,
            check_state: CheckState::Unchecked,
            context_menu: self.context_menu,
            actions: self.actions,
            metadata: self.metadata,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_id_creation() {
        let id1 = NodeId::new();
        let id2 = NodeId::new();
        assert_ne!(id1, id2);

        let id3 = NodeId::from_string("test-id");
        assert_eq!(id3.as_str(), "test-id");
    }

    #[test]
    fn test_node_id_from_str() {
        let id: NodeId = "my-node".into();
        assert_eq!(id.as_str(), "my-node");
    }

    #[test]
    fn test_node_icon_presets() {
        let folder = NodeIcon::folder();
        assert!(matches!(folder, NodeIcon::Dynamic { .. }));

        let file = NodeIcon::file();
        assert!(matches!(file, NodeIcon::Named(n) if n == "file"));
    }

    #[test]
    fn test_tree_node_creation() {
        let node = TreeNode::new("root", "Root Node");
        assert_eq!(node.id.as_str(), "root");
        assert_eq!(node.label, "Root Node");
        assert!(!node.state.expanded);
        assert!(!node.state.selected);
    }

    #[test]
    fn test_tree_node_with_children() {
        let child1 = TreeNode::new("child1", "Child 1");
        let child2 = TreeNode::new("child2", "Child 2");

        let parent = TreeNode::new("parent", "Parent").children(vec![child1, child2]);

        assert!(parent.has_children());
        assert_eq!(parent.get_children().len(), 2);
    }

    #[test]
    fn test_tree_node_add_child() {
        let mut parent = TreeNode::new("parent", "Parent");
        let child = TreeNode::new("child", "Child");

        parent.add_child(child);

        assert_eq!(parent.get_children().len(), 1);
        assert_eq!(parent.get_children()[0].parent_id, Some(NodeId::from_string("parent")));
        assert_eq!(parent.get_children()[0].depth, 1);
    }

    #[test]
    fn test_tree_node_remove_child() {
        let mut parent = TreeNode::new("parent", "Parent")
            .children(vec![
                TreeNode::new("child1", "Child 1"),
                TreeNode::new("child2", "Child 2"),
            ]);

        let removed = parent.remove_child(&NodeId::from_string("child1"));
        assert!(removed.is_some());
        assert_eq!(parent.get_children().len(), 1);
        assert_eq!(parent.get_children()[0].id.as_str(), "child2");
    }

    #[test]
    fn test_tree_node_lazy() {
        let node = TreeNode::new("lazy", "Lazy Node").lazy();
        assert!(node.needs_load());
        assert!(!node.children.is_loaded());
    }

    #[test]
    fn test_tree_node_leaf() {
        let node = TreeNode::new("leaf", "Leaf Node").leaf();
        assert!(node.is_leaf());
        assert!(!node.has_children());
    }

    #[test]
    fn test_check_state_toggle() {
        assert_eq!(CheckState::Unchecked.toggle(), CheckState::Checked);
        assert_eq!(CheckState::Checked.toggle(), CheckState::Unchecked);
        assert_eq!(CheckState::Indeterminate.toggle(), CheckState::Checked);
    }

    #[test]
    fn test_context_menu_item() {
        let item = ContextMenuItem::new("copy", "Copy")
            .icon(NodeIcon::named("copy"))
            .shortcut("Ctrl+C");

        assert_eq!(item.id, "copy");
        assert_eq!(item.label, "Copy");
        assert!(item.icon.is_some());
        assert_eq!(item.shortcut, Some("Ctrl+C".to_string()));
    }

    #[test]
    fn test_context_menu_separator() {
        let sep = ContextMenuItem::separator();
        assert!(sep.separator);
    }

    #[test]
    fn test_node_action_presets() {
        let edit = NodeAction::edit();
        assert_eq!(edit.id, "edit");

        let delete = NodeAction::delete();
        assert_eq!(delete.id, "delete");

        let add = NodeAction::add();
        assert_eq!(add.id, "add");
    }

    #[test]
    fn test_tree_node_builder() {
        let node = TreeNodeBuilder::new()
            .id("built-node")
            .label("Built Node")
            .icon(NodeIcon::folder())
            .expanded(true)
            .draggable(true)
            .checkable(true)
            .build();

        assert_eq!(node.id.as_str(), "built-node");
        assert_eq!(node.label, "Built Node");
        assert!(node.state.expanded);
        assert!(node.draggable);
        assert!(node.checkable);
    }

    #[test]
    fn test_tree_node_builder_lazy() {
        let node = TreeNodeBuilder::new()
            .label("Lazy Built")
            .lazy()
            .build();

        assert!(node.needs_load());
    }

    #[test]
    fn test_tree_node_builder_leaf() {
        let node = TreeNodeBuilder::new()
            .label("Leaf Built")
            .leaf()
            .build();

        assert!(node.is_leaf());
    }

    #[test]
    fn test_node_metadata() {
        let node = TreeNode::new("meta", "Meta Node")
            .with_metadata("file_size", serde_json::json!(1024))
            .with_metadata("file_type", serde_json::json!("text"));

        assert_eq!(
            node.get_metadata("file_size"),
            Some(&serde_json::json!(1024))
        );
        assert_eq!(
            node.get_metadata("file_type"),
            Some(&serde_json::json!("text"))
        );
    }

    #[test]
    fn test_children_config_set_loaded() {
        let mut config = ChildrenConfig::lazy();
        assert!(!config.is_loaded());

        let children = vec![TreeNode::new("child", "Child")];
        config.set_loaded(children);

        assert!(config.is_loaded());
        assert_eq!(config.children().len(), 1);
    }
}
