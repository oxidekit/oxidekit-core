//! Component Tree
//!
//! Provides a traversable tree structure for component visualization and inspection.
//! This module tracks the component hierarchy and enables efficient traversal,
//! hit testing, and path resolution.

use crate::inspector::{ComponentInfo, LayoutInfo, SourceLocation};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Handle to a node in the component tree
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeHandle(Uuid);

impl NodeHandle {
    /// Create a new unique handle
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    /// Create from an existing UUID
    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    /// Get the UUID
    pub fn uuid(&self) -> Uuid {
        self.0
    }

    /// Get as string
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

impl Default for NodeHandle {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for NodeHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A node in the component tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentNode {
    /// Unique handle for this node
    pub handle: NodeHandle,
    /// Component type ID (e.g., "ui.Button")
    pub component_id: String,
    /// Instance name (for debugging)
    pub name: String,
    /// Source location where this component is defined
    pub source: Option<SourceLocation>,
    /// Layout node ID (for mapping to oxide-layout)
    pub layout_node: Option<u64>,
    /// Parent node handle
    pub parent: Option<NodeHandle>,
    /// Child node handles
    pub children: Vec<NodeHandle>,
    /// Component properties (JSON)
    pub props: HashMap<String, serde_json::Value>,
    /// Style properties
    pub styles: HashMap<String, serde_json::Value>,
    /// Computed layout info (updated after layout pass)
    pub layout: LayoutInfo,
    /// Whether this node is expanded in the tree view
    pub expanded: bool,
    /// Whether this node is visible
    pub visible: bool,
    /// Component variant (if applicable)
    pub variant: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, String>,
}

impl ComponentNode {
    /// Create a new component node
    pub fn new(component_id: impl Into<String>) -> Self {
        let component_id = component_id.into();
        let name = component_id
            .split('.')
            .last()
            .unwrap_or(&component_id)
            .to_string();

        Self {
            handle: NodeHandle::new(),
            component_id,
            name,
            source: None,
            layout_node: None,
            parent: None,
            children: Vec::new(),
            props: HashMap::new(),
            styles: HashMap::new(),
            layout: LayoutInfo::default(),
            expanded: true,
            visible: true,
            variant: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the instance name
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the source location
    pub fn with_source(mut self, source: SourceLocation) -> Self {
        self.source = Some(source);
        self
    }

    /// Set the layout node ID
    pub fn with_layout_node(mut self, node_id: u64) -> Self {
        self.layout_node = Some(node_id);
        self
    }

    /// Set a property
    pub fn with_prop(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.props.insert(name.into(), value);
        self
    }

    /// Set a style
    pub fn with_style(mut self, name: impl Into<String>, value: serde_json::Value) -> Self {
        self.styles.insert(name.into(), value);
        self
    }

    /// Set the variant
    pub fn with_variant(mut self, variant: impl Into<String>) -> Self {
        self.variant = Some(variant.into());
        self
    }

    /// Get display name for tree view
    pub fn display_name(&self) -> String {
        if self.name != self.component_id.split('.').last().unwrap_or_default() {
            format!("{} ({})", self.name, self.component_id)
        } else {
            self.component_id.clone()
        }
    }

    /// Check if point is inside this component's layout bounds
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let (bx, by, bw, bh) = self.layout.border_box();
        x >= bx && x <= bx + bw && y >= by && y <= by + bh
    }
}

/// The component tree structure
#[derive(Debug, Default)]
pub struct ComponentTree {
    /// All nodes by handle
    nodes: HashMap<NodeHandle, ComponentNode>,
    /// Root node handle
    root: Option<NodeHandle>,
    /// Handle to layout node ID mapping
    layout_to_handle: HashMap<u64, NodeHandle>,
    /// Dirty flag for re-rendering tree view
    dirty: bool,
}

impl ComponentTree {
    /// Create a new empty component tree
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            layout_to_handle: HashMap::new(),
            dirty: false,
        }
    }

    /// Clear the tree
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.root = None;
        self.layout_to_handle.clear();
        self.dirty = true;
    }

    /// Get the root node handle
    pub fn root(&self) -> Option<NodeHandle> {
        self.root
    }

    /// Get a node by handle
    pub fn get(&self, handle: NodeHandle) -> Option<&ComponentNode> {
        self.nodes.get(&handle)
    }

    /// Get a mutable node by handle
    pub fn get_mut(&mut self, handle: NodeHandle) -> Option<&mut ComponentNode> {
        self.nodes.get_mut(&handle)
    }

    /// Get node by layout node ID
    pub fn get_by_layout_node(&self, layout_node: u64) -> Option<&ComponentNode> {
        self.layout_to_handle
            .get(&layout_node)
            .and_then(|h| self.nodes.get(h))
    }

    /// Get handle by layout node ID
    pub fn handle_for_layout_node(&self, layout_node: u64) -> Option<NodeHandle> {
        self.layout_to_handle.get(&layout_node).copied()
    }

    /// Add a node to the tree
    pub fn add(&mut self, mut node: ComponentNode) -> NodeHandle {
        let handle = node.handle;

        // Register layout node mapping
        if let Some(layout_id) = node.layout_node {
            self.layout_to_handle.insert(layout_id, handle);
        }

        // If no root, this becomes root
        if self.root.is_none() {
            self.root = Some(handle);
        }

        self.nodes.insert(handle, node);
        self.dirty = true;

        handle
    }

    /// Add a child to a parent node
    pub fn add_child(&mut self, parent: NodeHandle, mut child: ComponentNode) -> Option<NodeHandle> {
        if !self.nodes.contains_key(&parent) {
            return None;
        }

        let child_handle = child.handle;
        child.parent = Some(parent);

        // Register layout node mapping
        if let Some(layout_id) = child.layout_node {
            self.layout_to_handle.insert(layout_id, child_handle);
        }

        // Add child to parent's children list
        if let Some(parent_node) = self.nodes.get_mut(&parent) {
            parent_node.children.push(child_handle);
        }

        self.nodes.insert(child_handle, child);
        self.dirty = true;

        Some(child_handle)
    }

    /// Remove a node and all its descendants
    pub fn remove(&mut self, handle: NodeHandle) -> bool {
        let node = match self.nodes.remove(&handle) {
            Some(n) => n,
            None => return false,
        };

        // Remove from layout mapping
        if let Some(layout_id) = node.layout_node {
            self.layout_to_handle.remove(&layout_id);
        }

        // Remove from parent's children list
        if let Some(parent_handle) = node.parent {
            if let Some(parent) = self.nodes.get_mut(&parent_handle) {
                parent.children.retain(|c| *c != handle);
            }
        }

        // Remove all descendants
        for child_handle in node.children {
            self.remove(child_handle);
        }

        // Update root if needed
        if self.root == Some(handle) {
            self.root = None;
        }

        self.dirty = true;
        true
    }

    /// Get the path from root to a node
    pub fn path_to(&self, handle: NodeHandle) -> Vec<NodeHandle> {
        let mut path = Vec::new();
        let mut current = Some(handle);

        while let Some(h) = current {
            path.push(h);
            current = self.nodes.get(&h).and_then(|n| n.parent);
        }

        path.reverse();
        path
    }

    /// Get all ancestors of a node (not including self)
    pub fn ancestors(&self, handle: NodeHandle) -> Vec<NodeHandle> {
        let mut ancestors = Vec::new();
        let mut current = self.nodes.get(&handle).and_then(|n| n.parent);

        while let Some(h) = current {
            ancestors.push(h);
            current = self.nodes.get(&h).and_then(|n| n.parent);
        }

        ancestors
    }

    /// Get all descendants of a node (not including self)
    pub fn descendants(&self, handle: NodeHandle) -> Vec<NodeHandle> {
        let mut descendants = Vec::new();
        let mut stack = vec![handle];

        while let Some(h) = stack.pop() {
            if let Some(node) = self.nodes.get(&h) {
                for child in &node.children {
                    descendants.push(*child);
                    stack.push(*child);
                }
            }
        }

        descendants
    }

    /// Get siblings of a node (not including self)
    pub fn siblings(&self, handle: NodeHandle) -> Vec<NodeHandle> {
        self.nodes
            .get(&handle)
            .and_then(|n| n.parent)
            .and_then(|parent| self.nodes.get(&parent))
            .map(|parent| {
                parent
                    .children
                    .iter()
                    .filter(|c| **c != handle)
                    .copied()
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Get next sibling
    pub fn next_sibling(&self, handle: NodeHandle) -> Option<NodeHandle> {
        let parent = self.nodes.get(&handle)?.parent?;
        let parent_node = self.nodes.get(&parent)?;
        let pos = parent_node.children.iter().position(|c| *c == handle)?;
        parent_node.children.get(pos + 1).copied()
    }

    /// Get previous sibling
    pub fn prev_sibling(&self, handle: NodeHandle) -> Option<NodeHandle> {
        let parent = self.nodes.get(&handle)?.parent?;
        let parent_node = self.nodes.get(&parent)?;
        let pos = parent_node.children.iter().position(|c| *c == handle)?;
        if pos == 0 {
            None
        } else {
            parent_node.children.get(pos - 1).copied()
        }
    }

    /// Perform hit test at coordinates
    /// Returns the deepest (most specific) component containing the point
    pub fn hit_test(&self, x: f32, y: f32) -> Option<NodeHandle> {
        let root = self.root?;
        self.hit_test_recursive(root, x, y)
    }

    fn hit_test_recursive(&self, handle: NodeHandle, x: f32, y: f32) -> Option<NodeHandle> {
        let node = self.nodes.get(&handle)?;

        if !node.visible || !node.contains_point(x, y) {
            return None;
        }

        // Check children first (they're on top)
        for child in node.children.iter().rev() {
            if let Some(hit) = self.hit_test_recursive(*child, x, y) {
                return Some(hit);
            }
        }

        // No child hit, this node is the target
        Some(handle)
    }

    /// Get all visible nodes at coordinates (from front to back)
    pub fn hit_test_all(&self, x: f32, y: f32) -> Vec<NodeHandle> {
        let mut hits = Vec::new();
        if let Some(root) = self.root {
            self.hit_test_all_recursive(root, x, y, &mut hits);
        }
        hits
    }

    fn hit_test_all_recursive(&self, handle: NodeHandle, x: f32, y: f32, hits: &mut Vec<NodeHandle>) {
        if let Some(node) = self.nodes.get(&handle) {
            if node.visible && node.contains_point(x, y) {
                // Add children first (they're on top)
                for child in node.children.iter().rev() {
                    self.hit_test_all_recursive(*child, x, y, hits);
                }
                hits.push(handle);
            }
        }
    }

    /// Traverse the tree depth-first
    pub fn traverse<F>(&self, mut callback: F)
    where
        F: FnMut(&ComponentNode, usize),
    {
        if let Some(root) = self.root {
            self.traverse_recursive(root, 0, &mut callback);
        }
    }

    fn traverse_recursive<F>(&self, handle: NodeHandle, depth: usize, callback: &mut F)
    where
        F: FnMut(&ComponentNode, usize),
    {
        if let Some(node) = self.nodes.get(&handle) {
            callback(node, depth);
            for child in &node.children {
                self.traverse_recursive(*child, depth + 1, callback);
            }
        }
    }

    /// Traverse visible nodes only
    pub fn traverse_visible<F>(&self, mut callback: F)
    where
        F: FnMut(&ComponentNode, usize),
    {
        if let Some(root) = self.root {
            self.traverse_visible_recursive(root, 0, &mut callback);
        }
    }

    fn traverse_visible_recursive<F>(&self, handle: NodeHandle, depth: usize, callback: &mut F)
    where
        F: FnMut(&ComponentNode, usize),
    {
        if let Some(node) = self.nodes.get(&handle) {
            if !node.visible {
                return;
            }
            callback(node, depth);
            if node.expanded {
                for child in &node.children {
                    self.traverse_visible_recursive(*child, depth + 1, callback);
                }
            }
        }
    }

    /// Update layout info for a node
    pub fn update_layout(&mut self, handle: NodeHandle, layout: LayoutInfo) {
        if let Some(node) = self.nodes.get_mut(&handle) {
            node.layout = layout;
        }
    }

    /// Update layout from layout node ID
    pub fn update_layout_by_id(&mut self, layout_node: u64, layout: LayoutInfo) {
        if let Some(handle) = self.layout_to_handle.get(&layout_node).copied() {
            self.update_layout(handle, layout);
        }
    }

    /// Get total node count
    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    /// Check if tree is empty
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    /// Check and clear dirty flag
    pub fn take_dirty(&mut self) -> bool {
        let was_dirty = self.dirty;
        self.dirty = false;
        was_dirty
    }

    /// Mark tree as dirty
    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    /// Convert node to ComponentInfo for inspector
    pub fn to_component_info(&self, handle: NodeHandle) -> Option<ComponentInfo> {
        let node = self.nodes.get(&handle)?;

        let mut info = ComponentInfo::new(handle.to_string(), &node.component_id);
        info.name = node.name.clone();
        info.source = node.source.clone();
        info.layout = node.layout.clone();
        info.variant = node.variant.clone();
        info.children_count = node.children.len();
        info.parent_id = node.parent.map(|h| h.to_string());

        // Convert props
        for (name, value) in &node.props {
            info.props.push(crate::inspector::PropertyInfo {
                name: name.clone(),
                prop_type: "unknown".to_string(),
                value: value.clone(),
                default_value: None,
                is_set: true,
                required: false,
                description: String::new(),
                token_ref: None,
            });
        }

        Some(info)
    }

    /// Export tree structure as JSON for debugging
    pub fn to_json(&self) -> serde_json::Value {
        if let Some(root) = self.root {
            self.node_to_json(root)
        } else {
            serde_json::Value::Null
        }
    }

    fn node_to_json(&self, handle: NodeHandle) -> serde_json::Value {
        let Some(node) = self.nodes.get(&handle) else {
            return serde_json::Value::Null;
        };

        let children: Vec<serde_json::Value> = node
            .children
            .iter()
            .map(|c| self.node_to_json(*c))
            .collect();

        serde_json::json!({
            "handle": handle.to_string(),
            "component_id": node.component_id,
            "name": node.name,
            "layout": {
                "x": node.layout.x,
                "y": node.layout.y,
                "width": node.layout.width,
                "height": node.layout.height
            },
            "children": children
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_handle() {
        let h1 = NodeHandle::new();
        let h2 = NodeHandle::new();
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_component_node() {
        let node = ComponentNode::new("ui.Button")
            .with_name("submit-btn")
            .with_prop("label", serde_json::json!("Submit"));

        assert_eq!(node.component_id, "ui.Button");
        assert_eq!(node.name, "submit-btn");
        assert!(node.props.contains_key("label"));
    }

    #[test]
    fn test_tree_operations() {
        let mut tree = ComponentTree::new();

        let root = ComponentNode::new("ui.Container");
        let root_handle = tree.add(root);

        let child1 = ComponentNode::new("ui.Button");
        let child1_handle = tree.add_child(root_handle, child1).unwrap();

        let child2 = ComponentNode::new("ui.Text");
        let child2_handle = tree.add_child(root_handle, child2).unwrap();

        assert_eq!(tree.len(), 3);
        assert_eq!(tree.root(), Some(root_handle));

        // Check parent-child relationships
        let root_node = tree.get(root_handle).unwrap();
        assert_eq!(root_node.children.len(), 2);

        let child1_node = tree.get(child1_handle).unwrap();
        assert_eq!(child1_node.parent, Some(root_handle));

        // Test path
        let path = tree.path_to(child1_handle);
        assert_eq!(path, vec![root_handle, child1_handle]);

        // Test siblings
        let siblings = tree.siblings(child1_handle);
        assert_eq!(siblings, vec![child2_handle]);
    }

    #[test]
    fn test_hit_test() {
        let mut tree = ComponentTree::new();

        let mut root = ComponentNode::new("ui.Container");
        root.layout = LayoutInfo {
            x: 0.0,
            y: 0.0,
            width: 100.0,
            height: 100.0,
            ..Default::default()
        };
        let root_handle = tree.add(root);

        let mut child = ComponentNode::new("ui.Button");
        child.layout = LayoutInfo {
            x: 10.0,
            y: 10.0,
            width: 50.0,
            height: 30.0,
            ..Default::default()
        };
        let child_handle = tree.add_child(root_handle, child).unwrap();

        // Hit child
        let hit = tree.hit_test(25.0, 20.0);
        assert_eq!(hit, Some(child_handle));

        // Hit root (outside child)
        let hit = tree.hit_test(80.0, 80.0);
        assert_eq!(hit, Some(root_handle));

        // Miss everything
        let hit = tree.hit_test(150.0, 150.0);
        assert!(hit.is_none());
    }

    #[test]
    fn test_traverse() {
        let mut tree = ComponentTree::new();

        let root = ComponentNode::new("Root");
        let root_handle = tree.add(root);

        let child1 = ComponentNode::new("Child1");
        tree.add_child(root_handle, child1);

        let child2 = ComponentNode::new("Child2");
        tree.add_child(root_handle, child2);

        let mut visited = Vec::new();
        tree.traverse(|node, depth| {
            visited.push((node.component_id.clone(), depth));
        });

        assert_eq!(visited.len(), 3);
        assert_eq!(visited[0], ("Root".to_string(), 0));
        assert_eq!(visited[1].1, 1);
        assert_eq!(visited[2].1, 1);
    }

    #[test]
    fn test_remove() {
        let mut tree = ComponentTree::new();

        let root = ComponentNode::new("Root");
        let root_handle = tree.add(root);

        let child = ComponentNode::new("Child");
        let child_handle = tree.add_child(root_handle, child).unwrap();

        assert_eq!(tree.len(), 2);

        tree.remove(child_handle);
        assert_eq!(tree.len(), 1);

        let root_node = tree.get(root_handle).unwrap();
        assert!(root_node.children.is_empty());
    }
}
