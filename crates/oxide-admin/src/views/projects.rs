//! Projects view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{Table, TableProps, TableColumn, ColumnWidth, TableRow, EmptyState, EmptyStateProps};
use crate::state::{AdminState, ProjectInfo, ViewMode};
use super::layout::build_page_header;

/// Build the projects list view
pub fn build_projects_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header with actions
    let header = build_page_header(
        tree,
        "Projects",
        Some("Manage your OxideKit projects"),
        &[], // Actions would be added here
    );
    children.push(header);

    // Projects content
    let projects = state.filtered_projects();

    if projects.is_empty() {
        let empty = EmptyState::no_data(tree, "projects");
        children.push(empty);
    } else {
        let projects_content = match state.view_mode {
            ViewMode::Grid => build_projects_grid(tree, &projects),
            ViewMode::List => build_projects_list(tree, &projects),
            ViewMode::Table => build_projects_table(tree, &projects),
        };
        children.push(projects_content);
    }

    tree.new_node_with_children(content_style, &children)
}

fn build_projects_grid(tree: &mut LayoutTree, projects: &[&ProjectInfo]) -> NodeId {
    let grid_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let cards: Vec<NodeId> = projects.iter()
        .map(|p| build_project_card(tree, p))
        .collect();

    tree.new_node_with_children(grid_style, &cards)
}

fn build_project_card(tree: &mut LayoutTree, project: &ProjectInfo) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width(280.0)
        .padding(16.0)
        .gap(12.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Header with icon
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(12.0)
        .build();

    let icon_style = StyleBuilder::new().size(40.0, 40.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(8.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    let title_style = StyleBuilder::new().flex_column().gap(2.0).build();
    let title = tree.new_node(title_style);

    let header = tree.new_node_with_children(header_style, &[icon, title]);

    // Description
    let desc_style = StyleBuilder::new().build();
    let desc = tree.new_node(desc_style);

    // Footer with status and actions
    let footer_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let status_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(6.0)
        .padding_xy(0.0, 8.0)
        .build();

    let status_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#22C55E20"))
        .with_radius(4.0);

    let status = tree.new_visual_node(status_style, status_visual);

    let actions_style = StyleBuilder::new()
        .flex_row()
        .gap(8.0)
        .build();

    let actions = tree.new_node(actions_style);

    let footer = tree.new_node_with_children(footer_style, &[status, actions]);

    tree.new_visual_node_with_children(card_style, card_visual, &[header, desc, footer])
}

fn build_projects_list(tree: &mut LayoutTree, projects: &[&ProjectInfo]) -> NodeId {
    let list_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(8.0)
        .build();

    let items: Vec<NodeId> = projects.iter()
        .map(|p| build_project_list_item(tree, p))
        .collect();

    tree.new_node_with_children(list_style, &items)
}

fn build_project_list_item(tree: &mut LayoutTree, project: &ProjectInfo) -> NodeId {
    let item_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .width_percent(1.0)
        .height(64.0)
        .padding_xy(0.0, 16.0)
        .gap(16.0)
        .build();

    let item_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(8.0);

    // Icon
    let icon_style = StyleBuilder::new().size(40.0, 40.0).build();
    let icon_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#3B82F6"))
        .with_radius(8.0);
    let icon = tree.new_visual_node(icon_style, icon_visual);

    // Info
    let info_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .gap(4.0)
        .build();
    let info = tree.new_node(info_style);

    // Status
    let status_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .gap(6.0)
        .padding_xy(4.0, 10.0)
        .build();

    let status_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#22C55E20"))
        .with_radius(4.0);

    let status = tree.new_visual_node(status_style, status_visual);

    // Actions
    let actions_style = StyleBuilder::new()
        .flex_row()
        .gap(8.0)
        .build();
    let actions = tree.new_node(actions_style);

    tree.new_visual_node_with_children(item_style, item_visual, &[icon, info, status, actions])
}

fn build_projects_table(tree: &mut LayoutTree, projects: &[&ProjectInfo]) -> NodeId {
    let columns = vec![
        TableColumn::text("name", "Name").width(ColumnWidth::Flex(2.0)),
        TableColumn::text("version", "Version").width(ColumnWidth::Fixed(100.0)),
        TableColumn::status("status", "Status"),
        TableColumn::text("path", "Path").width(ColumnWidth::Flex(2.0)),
        TableColumn::actions("actions"),
    ];

    let rows: Vec<TableRow> = projects.iter()
        .map(|p| {
            TableRow::new(&p.id)
                .cell("name", &p.name)
                .cell("version", &p.version)
                .cell("status", &format!("{:?}", p.status))
                .cell("path", &p.path.display().to_string())
        })
        .collect();

    let props = TableProps::new(columns)
        .selectable(true)
        .striped(true);

    Table::build(tree, props, &rows, &Default::default(), None)
}

/// Build project detail view
pub fn build_project_detail_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    if let Some(project) = state.current_project() {
        // Project header
        let header = build_project_detail_header(tree, project);

        // Project info sections
        let info = build_project_info(tree, project);

        tree.new_node_with_children(content_style, &[header, info])
    } else {
        let empty = EmptyState::error(tree, "Project not found");
        tree.new_node_with_children(content_style, &[empty])
    }
}

fn build_project_detail_header(tree: &mut LayoutTree, project: &ProjectInfo) -> NodeId {
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

    // Left side - icon and title
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

    let title_style = StyleBuilder::new()
        .flex_column()
        .gap(4.0)
        .build();
    let title = tree.new_node(title_style);

    let left = tree.new_node_with_children(left_style, &[icon, title]);

    // Right side - actions
    let right_style = StyleBuilder::new()
        .flex_row()
        .gap(12.0)
        .build();
    let right = tree.new_node(right_style);

    tree.new_visual_node_with_children(header_style, header_visual, &[left, right])
}

fn build_project_info(tree: &mut LayoutTree, project: &ProjectInfo) -> NodeId {
    let info_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    // Left column - project details
    let left_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .gap(16.0)
        .build();

    let left = tree.new_node(left_style);

    // Right column - sidebar with quick info
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
