//! Sidebar navigation component
//!
//! Main navigation sidebar with collapsible sections and icons.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Sidebar item type
#[derive(Debug, Clone)]
pub struct SidebarItem {
    /// Item ID
    pub id: String,
    /// Display label
    pub label: String,
    /// Icon name
    pub icon: Option<String>,
    /// Badge count (notifications, etc.)
    pub badge: Option<u32>,
    /// Whether item is active
    pub active: bool,
    /// Child items for nested navigation
    pub children: Vec<SidebarItem>,
    /// Whether section is expanded
    pub expanded: bool,
}

impl SidebarItem {
    /// Create a new sidebar item
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            active: false,
            children: Vec::new(),
            expanded: true,
        }
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set badge count
    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    /// Set active state
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }

    /// Add child item
    pub fn child(mut self, item: SidebarItem) -> Self {
        self.children.push(item);
        self
    }

    /// Set expanded state
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

/// Sidebar section for grouping items
#[derive(Debug, Clone)]
pub struct SidebarSection {
    /// Section title (optional)
    pub title: Option<String>,
    /// Items in section
    pub items: Vec<SidebarItem>,
}

impl SidebarSection {
    /// Create new section without title
    pub fn new(items: Vec<SidebarItem>) -> Self {
        Self { title: None, items }
    }

    /// Create new section with title
    pub fn titled(title: impl Into<String>, items: Vec<SidebarItem>) -> Self {
        Self {
            title: Some(title.into()),
            items,
        }
    }
}

/// Sidebar properties
#[derive(Debug, Clone)]
pub struct SidebarProps {
    /// Sidebar width when expanded
    pub width: f32,
    /// Sidebar width when collapsed
    pub collapsed_width: f32,
    /// Whether sidebar is collapsed
    pub collapsed: bool,
    /// Show logo/branding area
    pub show_header: bool,
    /// Show user section at bottom
    pub show_user_section: bool,
}

impl Default for SidebarProps {
    fn default() -> Self {
        Self {
            width: 240.0,
            collapsed_width: 64.0,
            collapsed: false,
            show_header: true,
            show_user_section: true,
        }
    }
}

/// Sidebar component
pub struct Sidebar;

impl Sidebar {
    /// Build sidebar layout
    pub fn build(
        tree: &mut LayoutTree,
        props: SidebarProps,
        sections: &[SidebarSection],
        header: Option<NodeId>,
        footer: Option<NodeId>,
    ) -> NodeId {
        let width = if props.collapsed {
            props.collapsed_width
        } else {
            props.width
        };

        let sidebar_style = StyleBuilder::new()
            .flex_column()
            .width(width)
            .height_percent(1.0)
            .build();

        let sidebar_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_border(hex_to_rgba("#374151"), 1.0);

        let mut children = Vec::new();

        // Header section (logo/branding)
        if props.show_header {
            let header_node = header.unwrap_or_else(|| {
                Self::build_default_header(tree, props.collapsed)
            });
            children.push(header_node);
        }

        // Navigation sections
        let nav = Self::build_navigation(tree, &props, sections);
        children.push(nav);

        // User section at bottom
        if props.show_user_section {
            let user_node = footer.unwrap_or_else(|| {
                Self::build_default_user_section(tree, props.collapsed)
            });
            children.push(user_node);
        }

        tree.new_visual_node_with_children(sidebar_style, sidebar_visual, &children)
    }

    /// Build default header with logo
    fn build_default_header(tree: &mut LayoutTree, collapsed: bool) -> NodeId {
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(64.0)
            .padding_xy(0.0, if collapsed { 12.0 } else { 16.0 })
            .build();

        let header_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        // Logo placeholder
        let logo_style = StyleBuilder::new()
            .size(32.0, 32.0)
            .build();

        let logo_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#3B82F6"))
            .with_radius(8.0);

        let logo = tree.new_visual_node(logo_style, logo_visual);

        tree.new_visual_node_with_children(header_style, header_visual, &[logo])
    }

    /// Build navigation sections
    fn build_navigation(
        tree: &mut LayoutTree,
        props: &SidebarProps,
        sections: &[SidebarSection],
    ) -> NodeId {
        let nav_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .width_percent(1.0)
            .padding(8.0)
            .gap(16.0)
            .build();

        let section_nodes: Vec<NodeId> = sections
            .iter()
            .map(|section| Self::build_section(tree, props, section))
            .collect();

        tree.new_node_with_children(nav_style, &section_nodes)
    }

    /// Build a sidebar section
    fn build_section(
        tree: &mut LayoutTree,
        props: &SidebarProps,
        section: &SidebarSection,
    ) -> NodeId {
        let section_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(2.0)
            .build();

        let mut children = Vec::new();

        // Section title (if any and not collapsed)
        if let Some(title) = &section.title {
            if !props.collapsed {
                let title_style = StyleBuilder::new()
                    .flex_row()
                    .align_center()
                    .height(24.0)
                    .padding_xy(0.0, 12.0)
                    .build();

                let title_node = tree.new_node(title_style);
                children.push(title_node);
            }
        }

        // Items
        for item in &section.items {
            let item_node = Self::build_item(tree, props, item, 0);
            children.push(item_node);
        }

        tree.new_node_with_children(section_style, &children)
    }

    /// Build a sidebar item
    fn build_item(
        tree: &mut LayoutTree,
        props: &SidebarProps,
        item: &SidebarItem,
        depth: usize,
    ) -> NodeId {
        let indent = depth as f32 * 12.0;

        let item_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(40.0)
            .padding_xy(8.0, 12.0 + indent)
            .gap(12.0)
            .build();

        let item_visual = if item.active {
            NodeVisual::default()
                .with_background(hex_to_rgba("#3B82F620"))
                .with_radius(8.0)
        } else {
            NodeVisual::default().with_radius(8.0)
        };

        let mut children = Vec::new();

        // Icon placeholder
        if item.icon.is_some() || props.collapsed {
            let icon_style = StyleBuilder::new()
                .size(20.0, 20.0)
                .build();

            let icon_visual = NodeVisual::default()
                .with_background(if item.active {
                    hex_to_rgba("#3B82F6")
                } else {
                    hex_to_rgba("#6B7280")
                })
                .with_radius(4.0);

            let icon = tree.new_visual_node(icon_style, icon_visual);
            children.push(icon);
        }

        // Label (hidden when collapsed)
        if !props.collapsed {
            let label_style = StyleBuilder::new()
                .flex_grow(1.0)
                .build();

            let label = tree.new_node(label_style);
            children.push(label);
        }

        // Badge (if any and not collapsed)
        if let Some(count) = item.badge {
            if !props.collapsed && count > 0 {
                let badge_style = StyleBuilder::new()
                    .size(20.0, 20.0)
                    .build();

                let badge_visual = NodeVisual::default()
                    .with_background(hex_to_rgba("#EF4444"))
                    .with_radius(10.0);

                let badge = tree.new_visual_node(badge_style, badge_visual);
                children.push(badge);
            }
        }

        let item_node = tree.new_visual_node_with_children(item_style, item_visual, &children);

        // Build container with children if expanded
        if !item.children.is_empty() && item.expanded {
            let container_style = StyleBuilder::new()
                .flex_column()
                .width_percent(1.0)
                .build();

            let mut container_children = vec![item_node];
            for child in &item.children {
                let child_node = Self::build_item(tree, props, child, depth + 1);
                container_children.push(child_node);
            }

            tree.new_node_with_children(container_style, &container_children)
        } else {
            item_node
        }
    }

    /// Build default user section
    fn build_default_user_section(tree: &mut LayoutTree, collapsed: bool) -> NodeId {
        let user_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(64.0)
            .padding(12.0)
            .gap(12.0)
            .build();

        let user_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        // Avatar
        let avatar_style = StyleBuilder::new()
            .size(36.0, 36.0)
            .build();

        let avatar_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#4B5563"))
            .with_radius(18.0);

        let avatar = tree.new_visual_node(avatar_style, avatar_visual);

        if collapsed {
            tree.new_visual_node_with_children(user_style, user_visual, &[avatar])
        } else {
            // User info
            let info_style = StyleBuilder::new()
                .flex_column()
                .flex_grow(1.0)
                .build();

            let info = tree.new_node(info_style);

            tree.new_visual_node_with_children(user_style, user_visual, &[avatar, info])
        }
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
    fn test_sidebar_item() {
        let item = SidebarItem::new("dashboard", "Dashboard")
            .icon("home")
            .badge(5)
            .active(true);

        assert_eq!(item.id, "dashboard");
        assert_eq!(item.label, "Dashboard");
        assert_eq!(item.badge, Some(5));
        assert!(item.active);
    }

    #[test]
    fn test_sidebar_section() {
        let section = SidebarSection::titled("Main", vec![
            SidebarItem::new("home", "Home"),
            SidebarItem::new("settings", "Settings"),
        ]);

        assert_eq!(section.title, Some("Main".to_string()));
        assert_eq!(section.items.len(), 2);
    }
}
