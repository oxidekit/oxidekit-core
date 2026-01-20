//! Settings view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Tabs, TabsProps, TabItem, TabsVariant, Input, InputProps, Select, SelectProps, Switch};
use crate::state::AdminState;
use super::layout::build_page_header;

/// Build the settings view
pub fn build_settings_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header
    let header = build_page_header(tree, "Settings", Some("Configure OxideKit preferences"), &[]);
    children.push(header);

    // Settings tabs
    let tabs = build_settings_tabs(tree);
    children.push(tabs);

    // Settings content (General section by default)
    let settings_content = build_general_settings(tree, state);
    children.push(settings_content);

    tree.new_node_with_children(content_style, &children)
}

fn build_settings_tabs(tree: &mut LayoutTree) -> NodeId {
    let props = TabsProps {
        items: vec![
            TabItem::new("general", "General"),
            TabItem::new("appearance", "Appearance"),
            TabItem::new("editor", "Editor"),
            TabItem::new("build", "Build"),
            TabItem::new("developer", "Developer"),
            TabItem::new("updates", "Updates"),
        ],
        active: "general".to_string(),
        variant: TabsVariant::Line,
        full_width: false,
    };

    Tabs::build(tree, props)
}

fn build_general_settings(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let section_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    // Build inner settings first to avoid mutable borrow conflicts
    let language_setting = build_language_setting(tree, state);
    let language_section = build_setting_section(
        tree,
        "Language & Region",
        "Set your preferred language and regional settings",
        &[language_setting],
    );

    let startup_1 = build_toggle_setting(tree, "Start on login", "Automatically launch when you log in", false);
    let startup_2 = build_toggle_setting(tree, "Start minimized", "Start in the system tray", false);
    let startup_section = build_setting_section(
        tree,
        "Startup",
        "Control how OxideKit Admin starts",
        &[startup_1, startup_2],
    );

    let behavior_1 = build_toggle_setting(tree, "Confirm before exit", "Show confirmation dialog when closing", true);
    let behavior_2 = build_toggle_setting(tree, "Check for updates", "Automatically check for updates on startup", true);
    let behavior_section = build_setting_section(
        tree,
        "Behavior",
        "General application behavior",
        &[behavior_1, behavior_2],
    );

    let data_setting = build_project_directories_setting(tree, state);
    let data_section = build_setting_section(
        tree,
        "Data & Storage",
        "Manage your data and storage settings",
        &[data_setting],
    );

    tree.new_node_with_children(section_style, &[language_section, startup_section, behavior_section, data_section])
}

fn build_setting_section(
    tree: &mut LayoutTree,
    title: &str,
    description: &str,
    settings: &[NodeId],
) -> NodeId {
    let section_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .padding(24.0)
        .gap(20.0)
        .build();

    let section_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Header
    let header_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();

    let title_style = StyleBuilder::new().build();
    let title_node = tree.new_node(title_style);

    let desc_style = StyleBuilder::new().build();
    let desc_node = tree.new_node(desc_style);

    let header = tree.new_node_with_children(header_style, &[title_node, desc_node]);

    // Divider
    let divider_style = StyleBuilder::new()
        .width_percent(1.0)
        .height(1.0)
        .build();

    let divider_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#374151"));

    let divider = tree.new_visual_node(divider_style, divider_visual);

    // Settings list
    let list_style = StyleBuilder::new()
        .flex_column()
        .gap(16.0)
        .build();

    let list = tree.new_node_with_children(list_style, settings);

    tree.new_visual_node_with_children(section_style, section_visual, &[header, divider, list])
}

fn build_toggle_setting(tree: &mut LayoutTree, label: &str, description: &str, value: bool) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .build();

    // Left - label and description
    let left_style = StyleBuilder::new()
        .flex_column()
        .gap(2.0)
        .build();

    let label_style = StyleBuilder::new().build();
    let label_node = tree.new_node(label_style);

    let desc_style = StyleBuilder::new().build();
    let desc_node = tree.new_node(desc_style);

    let left = tree.new_node_with_children(left_style, &[label_node, desc_node]);

    // Right - toggle
    let toggle = Switch::build(tree, value, false);

    tree.new_node_with_children(row_style, &[left, toggle])
}

fn build_language_setting(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .build();

    // Left - label
    let left_style = StyleBuilder::new()
        .flex_column()
        .gap(2.0)
        .build();

    let label_style = StyleBuilder::new().build();
    let label_node = tree.new_node(label_style);

    let desc_style = StyleBuilder::new().build();
    let desc_node = tree.new_node(desc_style);

    let left = tree.new_node_with_children(left_style, &[label_node, desc_node]);

    // Right - select
    let select_props = SelectProps {
        value: Some(state.settings.general.language.clone()),
        options: vec![],
        placeholder: "Select language".to_string(),
        ..Default::default()
    };

    let select = Select::build(tree, select_props);

    tree.new_node_with_children(row_style, &[left, select])
}

fn build_project_directories_setting(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let container_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(12.0)
        .build();

    // Label
    let label_style = StyleBuilder::new()
        .flex_column()
        .gap(2.0)
        .build();

    let title_style = StyleBuilder::new().build();
    let title = tree.new_node(title_style);

    let desc_style = StyleBuilder::new().build();
    let desc = tree.new_node(desc_style);

    let label = tree.new_node_with_children(label_style, &[title, desc]);

    // Directory list
    let list_style = StyleBuilder::new()
        .flex_column()
        .gap(8.0)
        .build();

    let dir_items: Vec<NodeId> = state.settings.project_directories.iter()
        .map(|dir| {
            let item_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .justify_between()
                .width_percent(1.0)
                .height(40.0)
                .padding_xy(0.0, 12.0)
                .build();

            let item_visual = NodeVisual::default()
                .with_background(hex_to_rgba("#111827"))
                .with_border(hex_to_rgba("#374151"), 1.0)
                .with_radius(6.0);

            tree.new_visual_node(item_style, item_visual)
        })
        .collect();

    let list = tree.new_node_with_children(list_style, &dir_items);

    // Add button
    let add_btn_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .height(40.0)
        .padding_xy(0.0, 16.0)
        .build();

    let add_btn_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(8.0);

    let add_btn = tree.new_visual_node(add_btn_style, add_btn_visual);

    tree.new_node_with_children(container_style, &[label, list, add_btn])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
