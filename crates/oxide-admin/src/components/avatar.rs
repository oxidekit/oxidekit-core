//! Avatar component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Avatar size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AvatarSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl AvatarSize {
    pub fn pixels(&self) -> f32 {
        match self {
            Self::ExtraSmall => 24.0,
            Self::Small => 32.0,
            Self::Medium => 40.0,
            Self::Large => 48.0,
            Self::ExtraLarge => 64.0,
        }
    }
}

/// Avatar component
pub struct Avatar;

impl Avatar {
    /// Build avatar with initials fallback
    pub fn build(tree: &mut LayoutTree, _name: &str, size: AvatarSize) -> NodeId {
        let dim = size.pixels();

        let style = StyleBuilder::new()
            .size(dim, dim)
            .flex_row()
            .center()
            .build();

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba("#4B5563"))
            .with_radius(dim / 2.0);

        tree.new_visual_node(style, visual)
    }

    /// Build avatar with image placeholder
    pub fn image(tree: &mut LayoutTree, _src: &str, size: AvatarSize) -> NodeId {
        let dim = size.pixels();

        let style = StyleBuilder::new()
            .size(dim, dim)
            .build();

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba("#4B5563"))
            .with_radius(dim / 2.0);

        tree.new_visual_node(style, visual)
    }

    /// Build avatar with status indicator
    pub fn with_status(
        tree: &mut LayoutTree,
        name: &str,
        size: AvatarSize,
        status: AvatarStatus,
    ) -> NodeId {
        let dim = size.pixels();

        let container_style = StyleBuilder::new()
            .size(dim, dim)
            .build();

        // Avatar
        let avatar = Self::build(tree, name, size);

        // Status indicator
        let indicator_size = dim * 0.25;
        let indicator_style = StyleBuilder::new()
            .size(indicator_size, indicator_size)
            .build();

        let indicator_color = match status {
            AvatarStatus::Online => "#22C55E",
            AvatarStatus::Away => "#F59E0B",
            AvatarStatus::Busy => "#EF4444",
            AvatarStatus::Offline => "#6B7280",
        };

        let indicator_visual = NodeVisual::default()
            .with_background(hex_to_rgba(indicator_color))
            .with_border(hex_to_rgba("#1F2937"), 2.0)
            .with_radius(indicator_size / 2.0);

        let indicator = tree.new_visual_node(indicator_style, indicator_visual);

        tree.new_node_with_children(container_style, &[avatar, indicator])
    }
}

/// Avatar status
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AvatarStatus {
    Online,
    Away,
    Busy,
    #[default]
    Offline,
}

/// Avatar group for stacked avatars
pub struct AvatarGroup;

impl AvatarGroup {
    pub fn build(tree: &mut LayoutTree, names: &[&str], size: AvatarSize, max: usize) -> NodeId {
        let style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .build();

        let visible = names.iter().take(max);
        let overflow = names.len().saturating_sub(max);

        let mut children: Vec<NodeId> = visible
            .map(|name| Avatar::build(tree, name, size))
            .collect();

        // Overflow indicator
        if overflow > 0 {
            let dim = size.pixels();
            let overflow_style = StyleBuilder::new()
                .size(dim, dim)
                .flex_row()
                .center()
                .build();

            let overflow_visual = NodeVisual::default()
                .with_background(hex_to_rgba("#374151"))
                .with_radius(dim / 2.0);

            let overflow_node = tree.new_visual_node(overflow_style, overflow_visual);
            children.push(overflow_node);
        }

        tree.new_node_with_children(style, &children)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
