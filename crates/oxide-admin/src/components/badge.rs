//! Badge component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Badge variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeVariant {
    #[default]
    Default,
    Primary,
    Success,
    Warning,
    Danger,
    Info,
    Outline,
}

/// Badge size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum BadgeSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Badge component
pub struct Badge;

impl Badge {
    pub fn build(
        tree: &mut LayoutTree,
        _label: &str,
        variant: BadgeVariant,
        size: BadgeSize,
    ) -> NodeId {
        let (height, padding_x) = match size {
            BadgeSize::Small => (18.0, 6.0),
            BadgeSize::Medium => (22.0, 8.0),
            BadgeSize::Large => (26.0, 10.0),
        };

        let style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .center()
            .height(height)
            .padding_xy(0.0, padding_x)
            .build();

        let (bg_color, border_color) = match variant {
            BadgeVariant::Default => (hex_to_rgba("#374151"), None),
            BadgeVariant::Primary => (hex_to_rgba("#3B82F6"), None),
            BadgeVariant::Success => (hex_to_rgba("#22C55E"), None),
            BadgeVariant::Warning => (hex_to_rgba("#F59E0B"), None),
            BadgeVariant::Danger => (hex_to_rgba("#EF4444"), None),
            BadgeVariant::Info => (hex_to_rgba("#06B6D4"), None),
            BadgeVariant::Outline => (hex_to_rgba("#0000"), Some(hex_to_rgba("#374151"))),
        };

        let mut visual = NodeVisual::default()
            .with_background(bg_color)
            .with_radius(height / 2.0);

        if let Some(border) = border_color {
            visual = visual.with_border(border, 1.0);
        }

        tree.new_visual_node(style, visual)
    }

    /// Build a dot badge (notification indicator)
    pub fn dot(tree: &mut LayoutTree, variant: BadgeVariant) -> NodeId {
        let style = StyleBuilder::new().size(8.0, 8.0).build();

        let color = match variant {
            BadgeVariant::Default => "#6B7280",
            BadgeVariant::Primary => "#3B82F6",
            BadgeVariant::Success => "#22C55E",
            BadgeVariant::Warning => "#F59E0B",
            BadgeVariant::Danger => "#EF4444",
            BadgeVariant::Info => "#06B6D4",
            BadgeVariant::Outline => "#6B7280",
        };

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba(color))
            .with_radius(4.0);

        tree.new_visual_node(style, visual)
    }

    /// Build a count badge (e.g., notification count)
    pub fn count(tree: &mut LayoutTree, _count: u32, variant: BadgeVariant) -> NodeId {
        let style = StyleBuilder::new()
            .flex_row()
            .center()
            .size(20.0, 20.0)
            .build();

        let color = match variant {
            BadgeVariant::Danger => "#EF4444",
            BadgeVariant::Primary => "#3B82F6",
            _ => "#EF4444",
        };

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba(color))
            .with_radius(10.0);

        tree.new_visual_node(style, visual)
    }
}

/// Status badge with icon
pub struct StatusBadge;

impl StatusBadge {
    pub fn build(tree: &mut LayoutTree, status: &str) -> NodeId {
        let (bg_color, dot_color) = match status.to_lowercase().as_str() {
            "active" | "online" | "running" => ("#22C55E20", "#22C55E"),
            "warning" | "degraded" => ("#F59E0B20", "#F59E0B"),
            "error" | "offline" | "failed" => ("#EF444420", "#EF4444"),
            "pending" | "paused" => ("#F59E0B20", "#F59E0B"),
            _ => ("#6B728020", "#6B7280"),
        };

        let style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(6.0)
            .height(24.0)
            .padding_xy(0.0, 10.0)
            .build();

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba(bg_color))
            .with_radius(12.0);

        // Dot
        let dot_style = StyleBuilder::new().size(6.0, 6.0).build();
        let dot_visual = NodeVisual::default()
            .with_background(hex_to_rgba(dot_color))
            .with_radius(3.0);

        let dot = tree.new_visual_node(dot_style, dot_visual);

        // Label placeholder
        let label_style = StyleBuilder::new().build();
        let label = tree.new_node(label_style);

        tree.new_visual_node_with_children(style, visual, &[dot, label])
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
