//! Alert component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Alert variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AlertVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Alert properties
#[derive(Debug, Clone)]
pub struct AlertProps {
    pub title: Option<String>,
    pub message: String,
    pub variant: AlertVariant,
    pub dismissible: bool,
    pub icon: bool,
}

impl Default for AlertProps {
    fn default() -> Self {
        Self {
            title: None,
            message: String::new(),
            variant: AlertVariant::Info,
            dismissible: false,
            icon: true,
        }
    }
}

/// Alert component
pub struct Alert;

impl Alert {
    pub fn build(tree: &mut LayoutTree, props: AlertProps) -> NodeId {
        let alert_style = StyleBuilder::new()
            .flex_row()
            .align_start()
            .width_percent(1.0)
            .padding(16.0)
            .gap(12.0)
            .build();

        let (bg_color, border_color, text_color) = match props.variant {
            AlertVariant::Info => ("#1E3A5F", "#3B82F6", "#93C5FD"),
            AlertVariant::Success => ("#14532D", "#22C55E", "#86EFAC"),
            AlertVariant::Warning => ("#713F12", "#F59E0B", "#FCD34D"),
            AlertVariant::Error => ("#7F1D1D", "#EF4444", "#FCA5A5"),
        };

        let alert_visual = NodeVisual::default()
            .with_background(hex_to_rgba(bg_color))
            .with_border(hex_to_rgba(border_color), 1.0)
            .with_radius(8.0);

        let mut children = Vec::new();

        // Icon
        if props.icon {
            let icon_style = StyleBuilder::new().size(20.0, 20.0).build();
            let icon_visual = NodeVisual::default()
                .with_background(hex_to_rgba(border_color))
                .with_radius(4.0);
            let icon = tree.new_visual_node(icon_style, icon_visual);
            children.push(icon);
        }

        // Content
        let content_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .gap(4.0)
            .build();
        let content = tree.new_node(content_style);
        children.push(content);

        // Dismiss button
        if props.dismissible {
            let dismiss_style = StyleBuilder::new().size(16.0, 16.0).build();
            let dismiss_visual = NodeVisual::default()
                .with_background(hex_to_rgba(text_color))
                .with_radius(2.0);
            let dismiss = tree.new_visual_node(dismiss_style, dismiss_visual);
            children.push(dismiss);
        }

        tree.new_visual_node_with_children(alert_style, alert_visual, &children)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
