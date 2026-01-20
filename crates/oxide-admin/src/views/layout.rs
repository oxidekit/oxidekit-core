//! Admin layout system

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Sidebar, SidebarProps, SidebarSection, SidebarItem, Topbar, TopbarProps};
use crate::state::{AdminState, AdminRoute};

/// Build the main admin layout with sidebar and content
pub fn build_admin_layout(
    tree: &mut LayoutTree,
    state: &AdminState,
    content: NodeId,
) -> NodeId {
    let layout_style = StyleBuilder::new()
        .flex_row()
        .size_full()
        .build();

    // Build sidebar
    let sidebar = build_sidebar(tree, state);

    // Build main content area
    let main_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .height_percent(1.0)
        .build();

    // Topbar
    let topbar = build_topbar(tree, state);

    // Content wrapper
    let content_wrapper_style = StyleBuilder::new()
        .flex_grow(1.0)
        .width_percent(1.0)
        .padding(24.0)
        .build();

    let content_wrapper_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#0B0F14"));

    let content_wrapper = tree.new_visual_node_with_children(
        content_wrapper_style,
        content_wrapper_visual,
        &[content],
    );

    let main = tree.new_node_with_children(main_style, &[topbar, content_wrapper]);

    tree.new_node_with_children(layout_style, &[sidebar, main])
}

/// Build the sidebar navigation
fn build_sidebar(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let props = SidebarProps {
        width: 240.0,
        collapsed_width: 64.0,
        collapsed: state.sidebar_collapsed,
        show_header: true,
        show_user_section: true,
    };

    let sections = vec![
        SidebarSection::new(vec![
            SidebarItem::new("dashboard", "Dashboard")
                .icon("home")
                .active(matches!(state.route, AdminRoute::Dashboard)),
        ]),
        SidebarSection::titled("Manage", vec![
            SidebarItem::new("projects", "Projects")
                .icon("folder")
                .badge(state.projects.len() as u32)
                .active(matches!(state.route, AdminRoute::Projects | AdminRoute::ProjectDetail(_))),
            SidebarItem::new("plugins", "Plugins")
                .icon("puzzle")
                .badge(state.plugins.len() as u32)
                .active(matches!(state.route, AdminRoute::Plugins | AdminRoute::PluginDetail(_))),
            SidebarItem::new("themes", "Themes")
                .icon("palette")
                .active(matches!(state.route, AdminRoute::Themes | AdminRoute::ThemePreview(_))),
        ]),
        SidebarSection::titled("System", vec![
            SidebarItem::new("settings", "Settings")
                .icon("settings")
                .active(matches!(state.route, AdminRoute::Settings)),
            SidebarItem::new("diagnostics", "Diagnostics")
                .icon("terminal")
                .active(matches!(state.route, AdminRoute::Diagnostics)),
            SidebarItem::new("updates", "Updates")
                .icon("download")
                .badge(state.updates.as_ref().map(|u| u.available_count() as u32).unwrap_or(0))
                .active(matches!(state.route, AdminRoute::Updates)),
        ]),
    ];

    Sidebar::build(tree, props, &sections, None, None)
}

/// Build the topbar
fn build_topbar(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let props = TopbarProps {
        height: 64.0,
        show_search: true,
        show_notifications: true,
        show_user_menu: true,
        show_theme_toggle: true,
        title: Some(route_to_title(&state.route)),
    };

    Topbar::build(tree, props, None, None)
}

/// Convert route to page title
fn route_to_title(route: &AdminRoute) -> String {
    match route {
        AdminRoute::Dashboard => "Dashboard".to_string(),
        AdminRoute::Projects => "Projects".to_string(),
        AdminRoute::ProjectDetail(id) => format!("Project: {}", id),
        AdminRoute::Plugins => "Plugins".to_string(),
        AdminRoute::PluginDetail(id) => format!("Plugin: {}", id),
        AdminRoute::Themes => "Themes".to_string(),
        AdminRoute::ThemePreview(id) => format!("Theme: {}", id),
        AdminRoute::Settings => "Settings".to_string(),
        AdminRoute::Diagnostics => "Diagnostics".to_string(),
        AdminRoute::Updates => "Updates".to_string(),
    }
}

/// Page header component
pub fn build_page_header(
    tree: &mut LayoutTree,
    title: &str,
    description: Option<&str>,
    actions: &[NodeId],
) -> NodeId {
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .padding_xy(24.0, 0.0)
        .build();

    // Left side - title and description
    let left_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();

    let title_style = StyleBuilder::new().build();
    let title_node = tree.new_node(title_style);

    let left_children = if description.is_some() {
        let desc_style = StyleBuilder::new().build();
        let desc_node = tree.new_node(desc_style);
        vec![title_node, desc_node]
    } else {
        vec![title_node]
    };

    let left = tree.new_node_with_children(left_style, &left_children);

    // Right side - actions
    let right_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(12.0)
        .build();

    let right = tree.new_node_with_children(right_style, actions);

    tree.new_node_with_children(header_style, &[left, right])
}

/// Content grid layout
pub fn build_content_grid(
    tree: &mut LayoutTree,
    columns: usize,
    gap: f32,
    items: &[NodeId],
) -> NodeId {
    let grid_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(gap)
        .build();

    // For simplicity, we use flex-wrap behavior
    // In a real implementation, this would use CSS Grid or calculate column widths

    tree.new_node_with_children(grid_style, items)
}

/// Split pane layout
pub fn build_split_pane(
    tree: &mut LayoutTree,
    left: NodeId,
    right: NodeId,
    left_width: f32,
    resizable: bool,
) -> NodeId {
    let container_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .height_percent(1.0)
        .build();

    // Left pane
    let left_style = StyleBuilder::new()
        .width(left_width)
        .height_percent(1.0)
        .build();

    let left_pane = tree.new_node_with_children(left_style, &[left]);

    // Divider
    let divider_style = StyleBuilder::new()
        .width(1.0)
        .height_percent(1.0)
        .build();

    let divider_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#374151"));

    let divider = tree.new_visual_node(divider_style, divider_visual);

    // Right pane
    let right_style = StyleBuilder::new()
        .flex_grow(1.0)
        .height_percent(1.0)
        .build();

    let right_pane = tree.new_node_with_children(right_style, &[right]);

    tree.new_node_with_children(container_style, &[left_pane, divider, right_pane])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
