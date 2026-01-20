//! Empty state component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Empty state properties
#[derive(Debug, Clone)]
pub struct EmptyStateProps {
    pub title: String,
    pub description: Option<String>,
    pub icon: Option<String>,
}

impl EmptyStateProps {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Empty state component
pub struct EmptyState;

impl EmptyState {
    pub fn build(tree: &mut LayoutTree, props: EmptyStateProps, action: Option<NodeId>) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .center()
            .width_percent(1.0)
            .padding(48.0)
            .gap(16.0)
            .build();

        let mut children = Vec::new();

        // Icon
        let icon_style = StyleBuilder::new()
            .size(64.0, 64.0)
            .flex_row()
            .center()
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_radius(32.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);
        children.push(icon);

        // Title
        let title_style = StyleBuilder::new()
            .flex_row()
            .center()
            .build();

        let title = tree.new_node(title_style);
        children.push(title);

        // Description
        if props.description.is_some() {
            let desc_style = StyleBuilder::new()
                .flex_row()
                .center()
                .build();

            let desc = tree.new_node(desc_style);
            children.push(desc);
        }

        // Action button
        if let Some(action_node) = action {
            children.push(action_node);
        }

        tree.new_node_with_children(container_style, &children)
    }

    /// Build a "no results" empty state
    pub fn no_results(tree: &mut LayoutTree) -> NodeId {
        let props = EmptyStateProps::new("No results found")
            .description("Try adjusting your search or filters")
            .icon("search");

        Self::build(tree, props, None)
    }

    /// Build a "no data" empty state
    pub fn no_data(tree: &mut LayoutTree, entity: &str) -> NodeId {
        let props = EmptyStateProps::new(format!("No {} yet", entity))
            .description(format!("Get started by creating your first {}", entity))
            .icon("plus");

        Self::build(tree, props, None)
    }

    /// Build an "error" empty state
    pub fn error(tree: &mut LayoutTree, message: &str) -> NodeId {
        let props = EmptyStateProps::new("Something went wrong")
            .description(message)
            .icon("alert");

        Self::build(tree, props, None)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
