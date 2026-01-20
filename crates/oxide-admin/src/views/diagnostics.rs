//! Diagnostics view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Tabs, TabsProps, TabItem, TabsVariant, Alert, AlertProps, AlertVariant};
use crate::state::AdminState;
use super::layout::build_page_header;

/// Build the diagnostics view
pub fn build_diagnostics_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header
    let header = build_page_header(
        tree,
        "Diagnostics",
        Some("System health, logs, and debugging tools"),
        &[],
    );
    children.push(header);

    // Tabs
    let tabs = build_diagnostics_tabs(tree);
    children.push(tabs);

    // System overview (default)
    let overview = build_system_overview(tree, state);
    children.push(overview);

    tree.new_node_with_children(content_style, &children)
}

fn build_diagnostics_tabs(tree: &mut LayoutTree) -> NodeId {
    let props = TabsProps {
        items: vec![
            TabItem::new("overview", "Overview"),
            TabItem::new("logs", "Logs"),
            TabItem::new("performance", "Performance"),
            TabItem::new("network", "Network"),
            TabItem::new("storage", "Storage"),
        ],
        active: "overview".to_string(),
        variant: TabsVariant::Line,
        full_width: false,
    };

    Tabs::build(tree, props)
}

fn build_system_overview(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let overview_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    // System status alert
    let status_alert = Alert::build(tree, AlertProps {
        title: Some("System Status".to_string()),
        message: "All systems operational".to_string(),
        variant: AlertVariant::Success,
        dismissible: false,
        icon: true,
    });

    // Quick stats row
    let stats_row = build_stats_row(tree);

    // System info cards
    let info_row = build_info_row(tree);

    // Recent logs preview
    let logs_preview = build_logs_preview(tree);

    tree.new_node_with_children(overview_style, &[status_alert, stats_row, info_row, logs_preview])
}

fn build_stats_row(tree: &mut LayoutTree) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let stats = [
        ("CPU Usage", "23%", "#22C55E"),
        ("Memory", "4.2 GB", "#3B82F6"),
        ("Disk", "128 GB free", "#F59E0B"),
        ("Uptime", "3d 14h", "#8B5CF6"),
    ];

    let cards: Vec<NodeId> = stats.iter()
        .map(|(label, value, color)| build_mini_stat_card(tree, label, value, color))
        .collect();

    tree.new_node_with_children(row_style, &cards)
}

fn build_mini_stat_card(tree: &mut LayoutTree, label: &str, value: &str, color: &str) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .padding(16.0)
        .gap(8.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Label
    let label_style = StyleBuilder::new().build();
    let label_node = tree.new_node(label_style);

    // Value with color indicator
    let value_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .build();

    let indicator_style = StyleBuilder::new().size(8.0, 8.0).build();
    let indicator_visual = NodeVisual::default()
        .with_background(hex_to_rgba(color))
        .with_radius(4.0);
    let indicator = tree.new_visual_node(indicator_style, indicator_visual);

    let value_text = tree.new_node(StyleBuilder::new().build());

    let value_node = tree.new_node_with_children(value_style, &[indicator, value_text]);

    tree.new_visual_node_with_children(card_style, card_visual, &[label_node, value_node])
}

fn build_info_row(tree: &mut LayoutTree) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    // OxideKit info
    let oxide_info = build_info_card(tree, "OxideKit", vec![
        ("Version", env!("CARGO_PKG_VERSION")),
        ("Channel", "stable"),
        ("Build", "release"),
    ]);

    // System info
    let system_info = build_info_card(tree, "System", vec![
        ("OS", std::env::consts::OS),
        ("Arch", std::env::consts::ARCH),
        ("Family", std::env::consts::FAMILY),
    ]);

    // Runtime info
    let runtime_info = build_info_card(tree, "Runtime", vec![
        ("Renderer", "wgpu"),
        ("Layout", "taffy"),
        ("Text", "cosmic-text"),
    ]);

    tree.new_node_with_children(row_style, &[oxide_info, system_info, runtime_info])
}

fn build_info_card(tree: &mut LayoutTree, title: &str, items: Vec<(&str, &str)>) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Header
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .width_percent(1.0)
        .height(48.0)
        .padding_xy(0.0, 16.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0);

    let header = tree.new_visual_node(header_style, header_visual);

    // Items
    let items_style = StyleBuilder::new()
        .flex_column()
        .padding(16.0)
        .gap(8.0)
        .build();

    let item_nodes: Vec<NodeId> = items.iter()
        .map(|(label, value)| {
            let item_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .justify_between()
                .build();

            let label_style = StyleBuilder::new().build();
            let label_node = tree.new_node(label_style);

            let value_style = StyleBuilder::new().build();
            let value_node = tree.new_node(value_style);

            tree.new_node_with_children(item_style, &[label_node, value_node])
        })
        .collect();

    let items_node = tree.new_node_with_children(items_style, &item_nodes);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, items_node])
}

fn build_logs_preview(tree: &mut LayoutTree) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Header
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .height(48.0)
        .padding_xy(0.0, 16.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0);

    let header = tree.new_visual_node(header_style, header_visual);

    // Log entries (mock)
    let logs_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .height(200.0)
        .padding(8.0)
        .build();

    let logs_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#111827"));

    // Sample log entries
    let log_entries = [
        ("INFO", "Application started"),
        ("INFO", "Loaded 5 plugins"),
        ("DEBUG", "Theme changed to dark"),
        ("INFO", "Project scan complete: 12 projects found"),
    ];

    let entry_nodes: Vec<NodeId> = log_entries.iter()
        .map(|(level, msg)| {
            let entry_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(8.0)
                .height(24.0)
                .build();

            // Timestamp
            let time_style = StyleBuilder::new().width(60.0).build();
            let time = tree.new_node(time_style);

            // Level badge
            let level_style = StyleBuilder::new()
                .width(48.0)
                .padding_xy(2.0, 4.0)
                .build();

            let level_color = match *level {
                "INFO" => "#3B82F6",
                "DEBUG" => "#6B7280",
                "WARN" => "#F59E0B",
                "ERROR" => "#EF4444",
                _ => "#6B7280",
            };

            let level_visual = NodeVisual::default()
                .with_background(hex_to_rgba(&format!("{}20", level_color)))
                .with_radius(2.0);

            let level_node = tree.new_visual_node(level_style, level_visual);

            // Message
            let msg_style = StyleBuilder::new().flex_grow(1.0).build();
            let msg_node = tree.new_node(msg_style);

            tree.new_node_with_children(entry_style, &[time, level_node, msg_node])
        })
        .collect();

    let logs = tree.new_visual_node_with_children(logs_style, logs_visual, &entry_nodes);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, logs])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
