//! Skeleton loading component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Skeleton variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SkeletonVariant {
    #[default]
    Text,
    Circular,
    Rectangular,
    Rounded,
}

/// Skeleton component for loading states
pub struct Skeleton;

impl Skeleton {
    /// Build a skeleton element
    pub fn build(
        tree: &mut LayoutTree,
        width: f32,
        height: f32,
        variant: SkeletonVariant,
    ) -> NodeId {
        let style = StyleBuilder::new()
            .size(width, height)
            .build();

        let radius = match variant {
            SkeletonVariant::Text => 4.0,
            SkeletonVariant::Circular => height / 2.0,
            SkeletonVariant::Rectangular => 0.0,
            SkeletonVariant::Rounded => 8.0,
        };

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba("#374151"))
            .with_radius(radius);

        tree.new_visual_node(style, visual)
    }

    /// Build text skeleton
    pub fn text(tree: &mut LayoutTree, width: f32) -> NodeId {
        Self::build(tree, width, 16.0, SkeletonVariant::Text)
    }

    /// Build paragraph skeleton
    pub fn paragraph(tree: &mut LayoutTree, lines: usize) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(8.0)
            .build();

        let line_widths = [1.0, 0.95, 0.9, 0.85, 0.6]; // Varying widths for natural look

        let lines: Vec<NodeId> = (0..lines)
            .map(|i| {
                let width_pct = line_widths[i % line_widths.len()];
                let line_style = StyleBuilder::new()
                    .width_percent(width_pct)
                    .height(16.0)
                    .build();

                let line_visual = NodeVisual::default()
                    .with_background(hex_to_rgba("#374151"))
                    .with_radius(4.0);

                tree.new_visual_node(line_style, line_visual)
            })
            .collect();

        tree.new_node_with_children(container_style, &lines)
    }

    /// Build avatar skeleton
    pub fn avatar(tree: &mut LayoutTree, size: f32) -> NodeId {
        Self::build(tree, size, size, SkeletonVariant::Circular)
    }

    /// Build card skeleton
    pub fn card(tree: &mut LayoutTree) -> NodeId {
        let card_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .padding(16.0)
            .gap(16.0)
            .build();

        let card_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(12.0);

        // Header row
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(12.0)
            .build();

        let avatar = Self::avatar(tree, 40.0);

        let header_text_style = StyleBuilder::new()
            .flex_column()
            .gap(4.0)
            .build();

        let title = Self::text(tree, 120.0);
        let subtitle = Self::text(tree, 80.0);

        let header_text = tree.new_node_with_children(header_text_style, &[title, subtitle]);
        let header = tree.new_node_with_children(header_style, &[avatar, header_text]);

        // Content
        let content = Self::paragraph(tree, 3);

        tree.new_visual_node_with_children(card_style, card_visual, &[header, content])
    }

    /// Build table skeleton
    pub fn table(tree: &mut LayoutTree, rows: usize, cols: usize) -> NodeId {
        let table_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .build();

        let row_nodes: Vec<NodeId> = (0..rows)
            .map(|_| {
                let row_style = StyleBuilder::new()
                    .flex_row()
                    .align_center()
                    .width_percent(1.0)
                    .height(48.0)
                    .padding_xy(0.0, 16.0)
                    .gap(16.0)
                    .build();

                let row_visual = NodeVisual::default()
                    .with_border(hex_to_rgba("#374151"), 1.0);

                let cells: Vec<NodeId> = (0..cols)
                    .map(|_| Self::text(tree, 100.0))
                    .collect();

                tree.new_visual_node_with_children(row_style, row_visual, &cells)
            })
            .collect();

        tree.new_node_with_children(table_style, &row_nodes)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
