//! OxideKit Layout Engine
//!
//! Provides flexbox-style layout using Taffy, with responsive layout primitives
//! for mobile, tablet, and desktop support.
//!
//! # Modules
//!
//! - [`responsive`]: Breakpoint-based responsive layouts
//! - [`safe_area`]: Safe area handling for notches and home indicators
//! - [`adaptive`]: Adaptive navigation patterns
//!
//! # Example
//!
//! ```
//! use oxide_layout::prelude::*;
//!
//! // Create a responsive value that changes based on screen size
//! let padding = Responsive::new(8.0)
//!     .at(Breakpoint::Md, 16.0)
//!     .at(Breakpoint::Lg, 24.0);
//!
//! // Get the current breakpoint from screen dimensions
//! let ctx = BreakpointContext::from_size(1024.0, 768.0, 2.0);
//! let current_padding = ctx.resolve(&padding);
//! ```

pub mod adaptive;
pub mod responsive;
pub mod safe_area;

pub use taffy;
pub use taffy::prelude::*;

use std::collections::HashMap;

/// Prelude module for convenient imports
///
/// Import everything commonly needed with:
/// ```
/// use oxide_layout::prelude::*;
/// ```
pub mod prelude {
    // Re-export taffy prelude
    pub use taffy::prelude::*;

    // Core types
    pub use crate::{
        centered_column, column, row, ComputedRect, LayoutTree, NodeVisual, StyleBuilder,
    };

    // Responsive types
    pub use crate::responsive::{Breakpoint, BreakpointContext, Orientation, Responsive};

    // Safe area types
    pub use crate::safe_area::{presets as safe_area_presets, SafeAreaContainer, SafeAreaEdges, SafeAreaInsets};

    // Adaptive types
    pub use crate::adaptive::{
        AdaptiveLayout, AdaptiveNavigation, BottomNavConfig, DynamicAdaptiveLayout, Layout,
        SidebarConfig, SimpleLayout, SplitViewConfig, SplitViewWidth,
    };
}

/// Computed rectangle from layout
#[derive(Debug, Clone, Copy, Default)]
pub struct ComputedRect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl ComputedRect {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// Visual properties for a UI node
#[derive(Debug, Clone)]
pub struct NodeVisual {
    /// Background color (RGBA 0-1)
    pub background: Option<[f32; 4]>,
    /// Border color (RGBA 0-1)
    pub border_color: Option<[f32; 4]>,
    /// Border width in pixels
    pub border_width: f32,
    /// Corner radius in pixels
    pub corner_radius: f32,
}

impl Default for NodeVisual {
    fn default() -> Self {
        Self {
            background: None,
            border_color: None,
            border_width: 0.0,
            corner_radius: 0.0,
        }
    }
}

impl NodeVisual {
    pub fn with_background(mut self, color: [f32; 4]) -> Self {
        self.background = Some(color);
        self
    }

    pub fn with_border(mut self, color: [f32; 4], width: f32) -> Self {
        self.border_color = Some(color);
        self.border_width = width;
        self
    }

    pub fn with_radius(mut self, radius: f32) -> Self {
        self.corner_radius = radius;
        self
    }
}

/// A UI node with layout and visual properties
#[derive(Debug, Clone)]
pub struct UINode {
    pub id: NodeId,
    pub visual: NodeVisual,
    pub children: Vec<UINode>,
}

/// A layout tree that can compute positions for UI elements
pub struct LayoutTree {
    taffy: TaffyTree,
    visuals: HashMap<NodeId, NodeVisual>,
}

impl LayoutTree {
    /// Create a new empty layout tree
    pub fn new() -> Self {
        Self {
            taffy: TaffyTree::new(),
            visuals: HashMap::new(),
        }
    }

    /// Create a new node with the given style
    pub fn new_node(&mut self, style: Style) -> NodeId {
        self.taffy.new_leaf(style).expect("Failed to create node")
    }

    /// Create a new node with visual properties
    pub fn new_visual_node(&mut self, style: Style, visual: NodeVisual) -> NodeId {
        let id = self.taffy.new_leaf(style).expect("Failed to create node");
        self.visuals.insert(id, visual);
        id
    }

    /// Create a new node with children
    pub fn new_node_with_children(&mut self, style: Style, children: &[NodeId]) -> NodeId {
        self.taffy
            .new_with_children(style, children)
            .expect("Failed to create node with children")
    }

    /// Create a new node with children and visual properties
    pub fn new_visual_node_with_children(
        &mut self,
        style: Style,
        visual: NodeVisual,
        children: &[NodeId],
    ) -> NodeId {
        let id = self
            .taffy
            .new_with_children(style, children)
            .expect("Failed to create node with children");
        self.visuals.insert(id, visual);
        id
    }

    /// Set visual properties for a node
    pub fn set_visual(&mut self, node: NodeId, visual: NodeVisual) {
        self.visuals.insert(node, visual);
    }

    /// Get visual properties for a node
    pub fn get_visual(&self, node: NodeId) -> Option<&NodeVisual> {
        self.visuals.get(&node)
    }

    /// Compute layout for the tree starting from root
    pub fn compute_layout(&mut self, root: NodeId, available_space: Size<AvailableSpace>) {
        self.taffy
            .compute_layout(root, available_space)
            .expect("Failed to compute layout");
    }

    /// Get the computed layout for a node
    pub fn get_layout(&self, node: NodeId) -> &Layout {
        self.taffy.layout(node).expect("Node not found")
    }

    /// Get computed rectangle for a node (absolute position)
    pub fn get_rect(&self, node: NodeId) -> ComputedRect {
        let layout = self.get_layout(node);
        ComputedRect {
            x: layout.location.x,
            y: layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }

    /// Get computed rectangle with absolute position by traversing parents
    pub fn get_absolute_rect(&self, node: NodeId, parent_offset: (f32, f32)) -> ComputedRect {
        let layout = self.get_layout(node);
        ComputedRect {
            x: parent_offset.0 + layout.location.x,
            y: parent_offset.1 + layout.location.y,
            width: layout.size.width,
            height: layout.size.height,
        }
    }

    /// Get children of a node
    pub fn children(&self, node: NodeId) -> Vec<NodeId> {
        self.taffy.children(node).unwrap_or_default()
    }

    /// Iterate over all nodes depth-first, computing absolute positions
    pub fn traverse<F>(&self, root: NodeId, mut callback: F)
    where
        F: FnMut(NodeId, ComputedRect, Option<&NodeVisual>),
    {
        self.traverse_recursive(root, (0.0, 0.0), &mut callback);
    }

    fn traverse_recursive<F>(&self, node: NodeId, parent_offset: (f32, f32), callback: &mut F)
    where
        F: FnMut(NodeId, ComputedRect, Option<&NodeVisual>),
    {
        let rect = self.get_absolute_rect(node, parent_offset);
        let visual = self.visuals.get(&node);
        callback(node, rect, visual);

        let new_offset = (rect.x, rect.y);
        for child in self.children(node) {
            self.traverse_recursive(child, new_offset, callback);
        }
    }
}

impl Default for LayoutTree {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Style Builders
// ============================================================================

/// Builder for creating styles more ergonomically
#[derive(Default)]
pub struct StyleBuilder {
    style: Style,
}

impl StyleBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn flex_column(mut self) -> Self {
        self.style.display = Display::Flex;
        self.style.flex_direction = FlexDirection::Column;
        self
    }

    pub fn flex_row(mut self) -> Self {
        self.style.display = Display::Flex;
        self.style.flex_direction = FlexDirection::Row;
        self
    }

    pub fn center(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Center);
        self.style.justify_content = Some(JustifyContent::Center);
        self
    }

    pub fn align_center(mut self) -> Self {
        self.style.align_items = Some(AlignItems::Center);
        self
    }

    pub fn justify_center(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::Center);
        self
    }

    pub fn justify_between(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::SpaceBetween);
        self
    }

    pub fn justify_start(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::FlexStart);
        self
    }

    pub fn justify_end(mut self) -> Self {
        self.style.justify_content = Some(JustifyContent::FlexEnd);
        self
    }

    pub fn align_start(mut self) -> Self {
        self.style.align_items = Some(AlignItems::FlexStart);
        self
    }

    pub fn align_end(mut self) -> Self {
        self.style.align_items = Some(AlignItems::FlexEnd);
        self
    }

    pub fn size_full(mut self) -> Self {
        self.style.size = Size {
            width: Dimension::Percent(1.0),
            height: Dimension::Percent(1.0),
        };
        self
    }

    pub fn size(mut self, width: f32, height: f32) -> Self {
        self.style.size = Size {
            width: Dimension::Length(width),
            height: Dimension::Length(height),
        };
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.style.size.width = Dimension::Length(width);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.style.size.height = Dimension::Length(height);
        self
    }

    pub fn width_percent(mut self, percent: f32) -> Self {
        self.style.size.width = Dimension::Percent(percent);
        self
    }

    pub fn height_percent(mut self, percent: f32) -> Self {
        self.style.size.height = Dimension::Percent(percent);
        self
    }

    pub fn padding(mut self, value: f32) -> Self {
        self.style.padding = Rect {
            left: LengthPercentage::Length(value),
            right: LengthPercentage::Length(value),
            top: LengthPercentage::Length(value),
            bottom: LengthPercentage::Length(value),
        };
        self
    }

    pub fn padding_xy(mut self, horizontal: f32, vertical: f32) -> Self {
        self.style.padding = Rect {
            left: LengthPercentage::Length(horizontal),
            right: LengthPercentage::Length(horizontal),
            top: LengthPercentage::Length(vertical),
            bottom: LengthPercentage::Length(vertical),
        };
        self
    }

    pub fn margin(mut self, value: f32) -> Self {
        self.style.margin = Rect {
            left: LengthPercentageAuto::Length(value),
            right: LengthPercentageAuto::Length(value),
            top: LengthPercentageAuto::Length(value),
            bottom: LengthPercentageAuto::Length(value),
        };
        self
    }

    pub fn gap(mut self, value: f32) -> Self {
        self.style.gap = Size {
            width: LengthPercentage::Length(value),
            height: LengthPercentage::Length(value),
        };
        self
    }

    pub fn flex_grow(mut self, value: f32) -> Self {
        self.style.flex_grow = value;
        self
    }

    pub fn flex_shrink(mut self, value: f32) -> Self {
        self.style.flex_shrink = value;
        self
    }

    pub fn build(self) -> Style {
        self.style
    }
}

/// Create a centered column style
pub fn centered_column() -> Style {
    StyleBuilder::new()
        .flex_column()
        .center()
        .size_full()
        .build()
}

/// Create a row style
pub fn row() -> Style {
    StyleBuilder::new().flex_row().build()
}

/// Create a column style
pub fn column() -> Style {
    StyleBuilder::new().flex_column().build()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layout_tree() {
        let mut tree = LayoutTree::new();
        let root = tree.new_node(centered_column());
        tree.compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(800.0),
                height: AvailableSpace::Definite(600.0),
            },
        );
        let layout = tree.get_layout(root);
        assert_eq!(layout.size.width, 800.0);
        assert_eq!(layout.size.height, 600.0);
    }

    #[test]
    fn test_style_builder() {
        let style = StyleBuilder::new()
            .flex_column()
            .center()
            .size(200.0, 100.0)
            .padding(16.0)
            .build();

        assert_eq!(style.flex_direction, FlexDirection::Column);
        assert_eq!(style.align_items, Some(AlignItems::Center));
    }

    #[test]
    fn test_visual_nodes() {
        let mut tree = LayoutTree::new();

        let visual = NodeVisual::default()
            .with_background([0.1, 0.1, 0.2, 1.0])
            .with_radius(8.0);

        let node = tree.new_visual_node(
            StyleBuilder::new().size(100.0, 50.0).build(),
            visual,
        );

        let retrieved = tree.get_visual(node).unwrap();
        assert!(retrieved.background.is_some());
        assert_eq!(retrieved.corner_radius, 8.0);
    }

    #[test]
    fn test_traverse() {
        let mut tree = LayoutTree::new();

        let child1 = tree.new_visual_node(
            StyleBuilder::new().size(50.0, 50.0).build(),
            NodeVisual::default().with_background([1.0, 0.0, 0.0, 1.0]),
        );

        let child2 = tree.new_visual_node(
            StyleBuilder::new().size(50.0, 50.0).build(),
            NodeVisual::default().with_background([0.0, 1.0, 0.0, 1.0]),
        );

        let root = tree.new_visual_node_with_children(
            StyleBuilder::new().flex_row().gap(10.0).build(),
            NodeVisual::default(),
            &[child1, child2],
        );

        tree.compute_layout(
            root,
            Size {
                width: AvailableSpace::Definite(200.0),
                height: AvailableSpace::Definite(100.0),
            },
        );

        let mut visited = Vec::new();
        tree.traverse(root, |node, rect, _visual| {
            visited.push((node, rect));
        });

        assert_eq!(visited.len(), 3);
    }
}
