//! Icon component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Icon size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum IconSize {
    ExtraSmall,
    Small,
    #[default]
    Medium,
    Large,
    ExtraLarge,
}

impl IconSize {
    pub fn pixels(&self) -> f32 {
        match self {
            Self::ExtraSmall => 12.0,
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
            Self::ExtraLarge => 32.0,
        }
    }
}

/// Icon component (placeholder - icons would be rendered via font or SVG)
pub struct Icon;

impl Icon {
    /// Build icon placeholder
    pub fn build(tree: &mut LayoutTree, _name: &str, size: IconSize, color: Option<&str>) -> NodeId {
        let dim = size.pixels();

        let style = StyleBuilder::new()
            .size(dim, dim)
            .build();

        let fill_color = color.unwrap_or("#9CA3AF");

        let visual = NodeVisual::default()
            .with_background(hex_to_rgba(fill_color))
            .with_radius(2.0);

        tree.new_visual_node(style, visual)
    }

    /// Common icons
    pub fn home(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "home", size, None)
    }

    pub fn settings(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "settings", size, None)
    }

    pub fn user(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "user", size, None)
    }

    pub fn search(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "search", size, None)
    }

    pub fn plus(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "plus", size, None)
    }

    pub fn check(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "check", size, Some("#22C55E"))
    }

    pub fn x(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "x", size, Some("#EF4444"))
    }

    pub fn warning(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "warning", size, Some("#F59E0B"))
    }

    pub fn info(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "info", size, Some("#3B82F6"))
    }

    pub fn chevron_down(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "chevron-down", size, None)
    }

    pub fn chevron_right(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "chevron-right", size, None)
    }

    pub fn folder(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "folder", size, None)
    }

    pub fn file(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "file", size, None)
    }

    pub fn refresh(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "refresh", size, None)
    }

    pub fn download(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "download", size, None)
    }

    pub fn upload(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "upload", size, None)
    }

    pub fn trash(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "trash", size, Some("#EF4444"))
    }

    pub fn edit(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "edit", size, None)
    }

    pub fn copy(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "copy", size, None)
    }

    pub fn external_link(tree: &mut LayoutTree, size: IconSize) -> NodeId {
        Self::build(tree, "external-link", size, None)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
