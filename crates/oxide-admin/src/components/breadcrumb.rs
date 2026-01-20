//! Breadcrumb navigation component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Breadcrumb item
#[derive(Debug, Clone)]
pub struct BreadcrumbItem {
    pub label: String,
    pub href: Option<String>,
    pub icon: Option<String>,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: None,
            icon: None,
        }
    }

    pub fn link(label: impl Into<String>, href: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            href: Some(href.into()),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// Breadcrumb component
pub struct Breadcrumb;

impl Breadcrumb {
    pub fn build(tree: &mut LayoutTree, items: &[BreadcrumbItem]) -> NodeId {
        let breadcrumb_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(8.0)
            .build();

        let mut children = Vec::new();

        for (i, item) in items.iter().enumerate() {
            // Item
            let item_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(4.0)
                .build();

            let item_node = tree.new_node(item_style);
            children.push(item_node);

            // Separator (except for last item)
            if i < items.len() - 1 {
                let sep_style = StyleBuilder::new().size(16.0, 16.0).build();
                let sep_visual = NodeVisual::default()
                    .with_background(hex_to_rgba("#6B7280"))
                    .with_radius(2.0);
                let sep = tree.new_visual_node(sep_style, sep_visual);
                children.push(sep);
            }
        }

        tree.new_node_with_children(breadcrumb_style, &children)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
