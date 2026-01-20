//! Topbar component
//!
//! Application header with search, user menu, and actions.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Topbar properties
#[derive(Debug, Clone)]
pub struct TopbarProps {
    /// Topbar height
    pub height: f32,
    /// Show search
    pub show_search: bool,
    /// Show notifications
    pub show_notifications: bool,
    /// Show user menu
    pub show_user_menu: bool,
    /// Show theme toggle
    pub show_theme_toggle: bool,
    /// Title/breadcrumb
    pub title: Option<String>,
}

impl Default for TopbarProps {
    fn default() -> Self {
        Self {
            height: 64.0,
            show_search: true,
            show_notifications: true,
            show_user_menu: true,
            show_theme_toggle: true,
            title: None,
        }
    }
}

/// Topbar component
pub struct Topbar;

impl Topbar {
    /// Build topbar layout
    pub fn build(
        tree: &mut LayoutTree,
        props: TopbarProps,
        left_content: Option<NodeId>,
        right_content: Option<NodeId>,
    ) -> NodeId {
        let topbar_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(props.height)
            .padding_xy(0.0, 24.0)
            .build();

        let topbar_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#0B0F14"))
            .with_border(hex_to_rgba("#374151"), 1.0);

        let mut children = Vec::new();

        // Left section (title/breadcrumb or custom)
        let left = left_content.unwrap_or_else(|| {
            Self::build_left_section(tree, &props)
        });
        children.push(left);

        // Right section (search, actions, user)
        let right = right_content.unwrap_or_else(|| {
            Self::build_right_section(tree, &props)
        });
        children.push(right);

        tree.new_visual_node_with_children(topbar_style, topbar_visual, &children)
    }

    /// Build left section (title/breadcrumb)
    fn build_left_section(tree: &mut LayoutTree, props: &TopbarProps) -> NodeId {
        let left_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(16.0)
            .build();

        // Title placeholder
        let title_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .build();

        let title = tree.new_node(title_style);

        tree.new_node_with_children(left_style, &[title])
    }

    /// Build right section (actions)
    fn build_right_section(tree: &mut LayoutTree, props: &TopbarProps) -> NodeId {
        let right_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(16.0)
            .build();

        let mut children = Vec::new();

        // Search
        if props.show_search {
            let search = Self::build_search(tree);
            children.push(search);
        }

        // Theme toggle
        if props.show_theme_toggle {
            let toggle = Self::build_icon_button(tree, "theme");
            children.push(toggle);
        }

        // Notifications
        if props.show_notifications {
            let notif = Self::build_notification_button(tree);
            children.push(notif);
        }

        // User menu
        if props.show_user_menu {
            let user = Self::build_user_menu(tree);
            children.push(user);
        }

        tree.new_node_with_children(right_style, &children)
    }

    /// Build search input
    fn build_search(tree: &mut LayoutTree) -> NodeId {
        let search_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width(320.0)
            .height(40.0)
            .padding_xy(0.0, 12.0)
            .gap(8.0)
            .build();

        let search_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(8.0);

        // Search icon placeholder
        let icon_style = StyleBuilder::new()
            .size(16.0, 16.0)
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#6B7280"))
            .with_radius(2.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);

        // Placeholder text area
        let text_style = StyleBuilder::new()
            .flex_grow(1.0)
            .build();

        let text = tree.new_node(text_style);

        // Shortcut hint
        let hint_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .padding_xy(2.0, 6.0)
            .build();

        let hint_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#374151"))
            .with_radius(4.0);

        let hint = tree.new_visual_node(hint_style, hint_visual);

        tree.new_visual_node_with_children(search_style, search_visual, &[icon, text, hint])
    }

    /// Build icon button
    fn build_icon_button(tree: &mut LayoutTree, _icon: &str) -> NodeId {
        let btn_style = StyleBuilder::new()
            .size(40.0, 40.0)
            .flex_row()
            .center()
            .build();

        let btn_visual = NodeVisual::default()
            .with_radius(8.0);

        // Icon placeholder
        let icon_style = StyleBuilder::new()
            .size(20.0, 20.0)
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#9CA3AF"))
            .with_radius(2.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);

        tree.new_visual_node_with_children(btn_style, btn_visual, &[icon])
    }

    /// Build notification button with badge
    fn build_notification_button(tree: &mut LayoutTree) -> NodeId {
        let btn_style = StyleBuilder::new()
            .size(40.0, 40.0)
            .flex_row()
            .center()
            .build();

        let btn_visual = NodeVisual::default()
            .with_radius(8.0);

        // Icon placeholder
        let icon_style = StyleBuilder::new()
            .size(20.0, 20.0)
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#9CA3AF"))
            .with_radius(2.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);

        tree.new_visual_node_with_children(btn_style, btn_visual, &[icon])
    }

    /// Build user menu trigger
    fn build_user_menu(tree: &mut LayoutTree) -> NodeId {
        let user_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(8.0)
            .padding_xy(8.0, 8.0)
            .build();

        let user_visual = NodeVisual::default()
            .with_radius(8.0);

        // Avatar
        let avatar_style = StyleBuilder::new()
            .size(32.0, 32.0)
            .build();

        let avatar_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#4B5563"))
            .with_radius(16.0);

        let avatar = tree.new_visual_node(avatar_style, avatar_visual);

        // Name placeholder
        let name_style = StyleBuilder::new()
            .width(80.0)
            .height(16.0)
            .build();

        let name = tree.new_node(name_style);

        // Dropdown icon
        let dropdown_style = StyleBuilder::new()
            .size(16.0, 16.0)
            .build();

        let dropdown_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#6B7280"))
            .with_radius(2.0);

        let dropdown = tree.new_visual_node(dropdown_style, dropdown_visual);

        tree.new_visual_node_with_children(user_style, user_visual, &[avatar, name, dropdown])
    }
}

/// Convert hex color to RGBA array
fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex)
        .map(|c| c.to_array())
        .unwrap_or([1.0, 1.0, 1.0, 1.0])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_topbar_build() {
        let mut tree = LayoutTree::new();
        let props = TopbarProps::default();
        let _node = Topbar::build(&mut tree, props, None, None);
        // Successfully built a node - test passes if no panic
    }
}
