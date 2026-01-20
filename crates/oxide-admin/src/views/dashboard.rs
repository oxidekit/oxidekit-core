//! Dashboard view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Card, CardProps, StatCard, Chart, ChartProps, ChartType, ChartSeries};
use crate::state::AdminState;

/// Build the dashboard view
pub fn build_dashboard_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Stats row
    let stats = build_stats_row(tree, state);
    children.push(stats);

    // Charts row
    let charts = build_charts_row(tree, state);
    children.push(charts);

    // Recent activity and quick actions
    let bottom_row = build_bottom_row(tree, state);
    children.push(bottom_row);

    tree.new_node_with_children(content_style, &children)
}

fn build_stats_row(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let stats = state.dashboard_stats();

    let cards = vec![
        StatCard::build(
            tree,
            "Total Projects",
            &stats.total_projects.to_string(),
            Some(("+2 this week", true)),
            Some("folder"),
        ),
        StatCard::build(
            tree,
            "Active Projects",
            &stats.active_projects.to_string(),
            None,
            Some("play"),
        ),
        StatCard::build(
            tree,
            "Installed Plugins",
            &stats.installed_plugins.to_string(),
            Some((&format!("{} enabled", stats.enabled_plugins), true)),
            Some("puzzle"),
        ),
        StatCard::build(
            tree,
            "Available Updates",
            &stats.updates_available.to_string(),
            if stats.updates_available > 0 {
                Some(("Updates available", false))
            } else {
                None
            },
            Some("download"),
        ),
    ];

    tree.new_node_with_children(row_style, &cards)
}

fn build_charts_row(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    // Activity chart
    let activity_props = ChartProps {
        chart_type: ChartType::Line,
        title: Some("Build Activity".to_string()),
        series: vec![
            ChartSeries::new("Builds", vec![12.0, 19.0, 3.0, 5.0, 2.0, 3.0, 9.0], "#3B82F6"),
        ],
        labels: vec!["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .into_iter().map(String::from).collect(),
        height: 250.0,
        ..Default::default()
    };

    let activity_chart = build_chart_card(tree, activity_props);

    // Resource usage chart
    let resource_props = ChartProps {
        chart_type: ChartType::Bar,
        title: Some("Resource Usage".to_string()),
        series: vec![
            ChartSeries::new("CPU", vec![65.0, 59.0, 80.0, 81.0, 56.0], "#22C55E"),
            ChartSeries::new("Memory", vec![28.0, 48.0, 40.0, 19.0, 86.0], "#F59E0B"),
        ],
        labels: vec!["App 1", "App 2", "App 3", "App 4", "App 5"]
            .into_iter().map(String::from).collect(),
        height: 250.0,
        ..Default::default()
    };

    let resource_chart = build_chart_card(tree, resource_props);

    tree.new_node_with_children(row_style, &[activity_chart, resource_chart])
}

fn build_chart_card(tree: &mut LayoutTree, props: ChartProps) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .padding(16.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    let chart = Chart::build(tree, props);

    tree.new_visual_node_with_children(card_style, card_visual, &[chart])
}

fn build_bottom_row(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let row_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    // Recent projects
    let recent_projects = build_recent_projects(tree, state);

    // Quick actions
    let quick_actions = build_quick_actions(tree);

    tree.new_node_with_children(row_style, &[recent_projects, quick_actions])
}

fn build_recent_projects(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
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
        .justify_between()
        .width_percent(1.0)
        .height(56.0)
        .padding_xy(0.0, 16.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0);

    let header = tree.new_visual_node(header_style, header_visual);

    // Project list
    let list_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .build();

    let projects = state.projects.recent(5);
    let project_items: Vec<NodeId> = projects.iter()
        .map(|p| build_project_item(tree, &p.name, &p.version))
        .collect();

    let list = tree.new_node_with_children(list_style, &project_items);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, list])
}

fn build_project_item(tree: &mut LayoutTree, name: &str, version: &str) -> NodeId {
    let item_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .width_percent(1.0)
        .height(48.0)
        .padding_xy(0.0, 16.0)
        .build();

    let item_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0);

    // Left side - icon and name
    let left_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(12.0)
        .build();

    let icon_style = StyleBuilder::new().size(32.0, 32.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(6.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    let name_style = StyleBuilder::new().build();
    let name_node = tree.new_node(name_style);

    let left = tree.new_node_with_children(left_style, &[icon, name_node]);

    // Right side - version and arrow
    let right_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(8.0)
        .build();

    let version_style = StyleBuilder::new().build();
    let version_node = tree.new_node(version_style);

    let arrow_style = StyleBuilder::new().size(16.0, 16.0).build();
    let arrow_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#6B7280"))
        .with_radius(2.0);
    let arrow = tree.new_visual_node(arrow_style, arrow_visual);

    let right = tree.new_node_with_children(right_style, &[version_node, arrow]);

    tree.new_visual_node_with_children(item_style, item_visual, &[left, right])
}

fn build_quick_actions(tree: &mut LayoutTree) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width(300.0)
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
        .height(56.0)
        .padding_xy(0.0, 16.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_border(hex_to_rgba("#374151"), 1.0);

    let header = tree.new_visual_node(header_style, header_visual);

    // Actions list
    let list_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .padding(8.0)
        .gap(4.0)
        .build();

    let actions = [
        ("New Project", "plus"),
        ("Install Plugin", "puzzle"),
        ("Check Updates", "download"),
        ("Open Settings", "settings"),
    ];

    let action_items: Vec<NodeId> = actions.iter()
        .map(|(label, icon)| build_action_item(tree, label, icon))
        .collect();

    let list = tree.new_node_with_children(list_style, &action_items);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, list])
}

fn build_action_item(tree: &mut LayoutTree, label: &str, icon: &str) -> NodeId {
    let item_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .width_percent(1.0)
        .height(40.0)
        .padding_xy(8.0, 12.0)
        .gap(12.0)
        .build();

    let item_visual = NodeVisual::default()
        .with_radius(8.0);

    let icon_style = StyleBuilder::new().size(20.0, 20.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#6B7280"))
        .with_radius(4.0);
    let icon_node = tree.new_visual_node(icon_style, icon_visual);

    let label_style = StyleBuilder::new().build();
    let label_node = tree.new_node(label_style);

    tree.new_visual_node_with_children(item_style, item_visual, &[icon_node, label_node])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
