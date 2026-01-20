//! Toast notification component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Toast variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ToastVariant {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Toast position
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ToastPosition {
    TopRight,
    #[default]
    BottomRight,
    TopLeft,
    BottomLeft,
    TopCenter,
    BottomCenter,
}

/// Toast properties
#[derive(Debug, Clone)]
pub struct ToastProps {
    pub message: String,
    pub variant: ToastVariant,
    pub title: Option<String>,
    pub duration_ms: u64,
    pub dismissible: bool,
    pub action: Option<ToastAction>,
}

/// Toast action button
#[derive(Debug, Clone)]
pub struct ToastAction {
    pub label: String,
    pub action_id: String,
}

impl Default for ToastProps {
    fn default() -> Self {
        Self {
            message: String::new(),
            variant: ToastVariant::Info,
            title: None,
            duration_ms: 5000,
            dismissible: true,
            action: None,
        }
    }
}

/// Toast component
pub struct Toast;

impl Toast {
    pub fn build(tree: &mut LayoutTree, props: ToastProps) -> NodeId {
        let toast_style = StyleBuilder::new()
            .flex_row()
            .align_start()
            .width(360.0)
            .padding(16.0)
            .gap(12.0)
            .build();

        let (bg_color, accent_color) = match props.variant {
            ToastVariant::Info => ("#1F2937", "#3B82F6"),
            ToastVariant::Success => ("#1F2937", "#22C55E"),
            ToastVariant::Warning => ("#1F2937", "#F59E0B"),
            ToastVariant::Error => ("#1F2937", "#EF4444"),
        };

        let toast_visual = NodeVisual::default()
            .with_background(hex_to_rgba(bg_color))
            .with_border(hex_to_rgba(accent_color), 1.0)
            .with_radius(12.0);

        // Icon
        let icon_style = StyleBuilder::new().size(20.0, 20.0).build();
        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba(accent_color))
            .with_radius(4.0);
        let icon = tree.new_visual_node(icon_style, icon_visual);

        // Content
        let content_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .gap(4.0)
            .build();
        let content = tree.new_node(content_style);

        // Dismiss button
        let dismiss_style = StyleBuilder::new().size(16.0, 16.0).build();
        let dismiss_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#6B7280"))
            .with_radius(2.0);
        let dismiss = tree.new_visual_node(dismiss_style, dismiss_visual);

        tree.new_visual_node_with_children(toast_style, toast_visual, &[icon, content, dismiss])
    }
}

/// Toast container for multiple toasts
pub struct ToastContainer;

impl ToastContainer {
    pub fn build(tree: &mut LayoutTree, position: ToastPosition, toasts: &[NodeId]) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .gap(8.0)
            .padding(16.0)
            .build();

        tree.new_node_with_children(container_style, toasts)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
