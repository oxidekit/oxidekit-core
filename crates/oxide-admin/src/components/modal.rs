//! Modal/Dialog component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Modal size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ModalSize {
    Small,
    #[default]
    Medium,
    Large,
    FullScreen,
}

/// Modal properties
#[derive(Debug, Clone)]
pub struct ModalProps {
    pub title: String,
    pub size: ModalSize,
    pub show_close_button: bool,
    pub closable_on_backdrop: bool,
}

impl Default for ModalProps {
    fn default() -> Self {
        Self {
            title: String::new(),
            size: ModalSize::Medium,
            show_close_button: true,
            closable_on_backdrop: true,
        }
    }
}

/// Modal component
pub struct Modal;

impl Modal {
    /// Build modal with content
    pub fn build(
        tree: &mut LayoutTree,
        props: ModalProps,
        header: Option<NodeId>,
        content: &[NodeId],
        footer: Option<NodeId>,
    ) -> NodeId {
        let (width, max_height) = match props.size {
            ModalSize::Small => (400.0, 300.0),
            ModalSize::Medium => (560.0, 500.0),
            ModalSize::Large => (800.0, 700.0),
            ModalSize::FullScreen => (0.0, 0.0), // Will use percent
        };

        // Backdrop
        let backdrop_style = StyleBuilder::new()
            .size_full()
            .flex_row()
            .center()
            .build();

        let backdrop_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#00000080"));

        // Modal container
        let modal_style = if props.size == ModalSize::FullScreen {
            StyleBuilder::new()
                .flex_column()
                .width_percent(0.9)
                .height_percent(0.9)
                .build()
        } else {
            StyleBuilder::new()
                .flex_column()
                .width(width)
                .build()
        };

        let modal_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_radius(16.0);

        let mut modal_children = Vec::new();

        // Header
        let header_node = header.unwrap_or_else(|| Self::build_header(tree, &props));
        modal_children.push(header_node);

        // Content
        let content_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .padding(24.0)
            .build();

        let content_node = if content.is_empty() {
            tree.new_node(content_style)
        } else {
            tree.new_node_with_children(content_style, content)
        };
        modal_children.push(content_node);

        // Footer
        if let Some(footer_node) = footer {
            modal_children.push(footer_node);
        }

        let modal = tree.new_visual_node_with_children(modal_style, modal_visual, &modal_children);

        tree.new_visual_node_with_children(backdrop_style, backdrop_visual, &[modal])
    }

    fn build_header(tree: &mut LayoutTree, props: &ModalProps) -> NodeId {
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(56.0)
            .padding_xy(0.0, 24.0)
            .build();

        let header_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        tree.new_visual_node(header_style, header_visual)
    }

    /// Build modal footer with actions
    pub fn build_footer(tree: &mut LayoutTree, actions: &[NodeId]) -> NodeId {
        let footer_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_end()
            .width_percent(1.0)
            .height(64.0)
            .padding_xy(16.0, 24.0)
            .gap(12.0)
            .build();

        let footer_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        tree.new_visual_node_with_children(footer_style, footer_visual, actions)
    }
}

/// Confirmation dialog builder
pub struct ConfirmDialog;

impl ConfirmDialog {
    pub fn build(
        tree: &mut LayoutTree,
        title: &str,
        message: &str,
        confirm_label: &str,
        cancel_label: &str,
        destructive: bool,
    ) -> NodeId {
        let props = ModalProps {
            title: title.to_string(),
            size: ModalSize::Small,
            ..Default::default()
        };

        Modal::build(tree, props, None, &[], None)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
