//! Plugins view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{EmptyState, Tabs, TabsProps, TabItem, TabsVariant};
use crate::state::{AdminState, PluginInfo, PluginCategory};
use super::layout::build_page_header;

/// Build the plugins view
pub fn build_plugins_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header
    let header = build_page_header(
        tree,
        "Plugins",
        Some("Manage installed plugins and extensions"),
        &[],
    );
    children.push(header);

    // Category tabs
    let tabs = build_category_tabs(tree);
    children.push(tabs);

    // Plugins grid
    let plugins = state.filtered_plugins();
    if plugins.is_empty() {
        let empty = EmptyState::no_data(tree, "plugins");
        children.push(empty);
    } else {
        let grid = build_plugins_grid(tree, &plugins);
        children.push(grid);
    }

    tree.new_node_with_children(content_style, &children)
}

fn build_category_tabs(tree: &mut LayoutTree) -> NodeId {
    let props = TabsProps {
        items: vec![
            TabItem::new("all", "All"),
            TabItem::new("ui", "UI Components"),
            TabItem::new("data", "Data & API"),
            TabItem::new("native", "Native"),
            TabItem::new("design", "Design Packs"),
        ],
        active: "all".to_string(),
        variant: TabsVariant::Pills,
        full_width: false,
    };

    Tabs::build(tree, props)
}

fn build_plugins_grid(tree: &mut LayoutTree, plugins: &[&PluginInfo]) -> NodeId {
    let grid_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let cards: Vec<NodeId> = plugins.iter()
        .map(|p| build_plugin_card(tree, p))
        .collect();

    tree.new_node_with_children(grid_style, &cards)
}

fn build_plugin_card(tree: &mut LayoutTree, plugin: &PluginInfo) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width(280.0)
        .padding(16.0)
        .gap(12.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(if plugin.enabled {
            hex_to_rgba("#374151")
        } else {
            hex_to_rgba("#4B5563")
        }, 1.0)
        .with_radius(12.0);

    // Header
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let icon_style = StyleBuilder::new().size(40.0, 40.0).build();
    let icon_color = match plugin.category {
        PluginCategory::UI => "#3B82F6",
        PluginCategory::Data => "#22C55E",
        PluginCategory::Native => "#F59E0B",
        PluginCategory::Design => "#EC4899",
        PluginCategory::Dev => "#8B5CF6",
        PluginCategory::Integration => "#06B6D4",
        PluginCategory::Other => "#6B7280",
    };

    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba(icon_color))
        .with_radius(8.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    // Enable/disable toggle placeholder
    let toggle_style = StyleBuilder::new().size(44.0, 24.0).build();
    let toggle_visual = NodeVisual::default()
        .with_background(if plugin.enabled {
            hex_to_rgba("#3B82F6")
        } else {
            hex_to_rgba("#374151")
        })
        .with_radius(12.0);
    let toggle = tree.new_visual_node(toggle_style, toggle_visual);

    let header = tree.new_node_with_children(header_style, &[icon, toggle]);

    // Title and description
    let info_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();
    let info = tree.new_node(info_style);

    // Footer with version and category
    let footer_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let version_style = StyleBuilder::new().build();
    let version = tree.new_node(version_style);

    let category_style = StyleBuilder::new()
        .padding_xy(4.0, 8.0)
        .build();
    let category_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#374151"))
        .with_radius(4.0);
    let category = tree.new_visual_node(category_style, category_visual);

    let footer = tree.new_node_with_children(footer_style, &[version, category]);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, info, footer])
}

/// Build plugin detail view
pub fn build_plugin_detail_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    if let Some(plugin) = state.current_plugin() {
        let header = build_plugin_detail_header(tree, plugin);
        let info = build_plugin_info(tree, plugin);

        tree.new_node_with_children(content_style, &[header, info])
    } else {
        let empty = EmptyState::error(tree, "Plugin not found");
        tree.new_node_with_children(content_style, &[empty])
    }
}

fn build_plugin_detail_header(tree: &mut LayoutTree, plugin: &PluginInfo) -> NodeId {
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .padding(24.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    let left_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(16.0)
        .build();

    let icon_style = StyleBuilder::new().size(64.0, 64.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(12.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    let title_style = StyleBuilder::new().flex_column().gap(4.0).build();
    let title = tree.new_node(title_style);

    let left = tree.new_node_with_children(left_style, &[icon, title]);

    let right_style = StyleBuilder::new().flex_row().gap(12.0).build();
    let right = tree.new_node(right_style);

    tree.new_visual_node_with_children(header_style, header_visual, &[left, right])
}

fn build_plugin_info(tree: &mut LayoutTree, plugin: &PluginInfo) -> NodeId {
    let info_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let left_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .gap(16.0)
        .build();
    let left = tree.new_node(left_style);

    let right_style = StyleBuilder::new()
        .flex_column()
        .width(300.0)
        .gap(16.0)
        .build();
    let right = tree.new_node(right_style);

    tree.new_node_with_children(info_style, &[left, right])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
