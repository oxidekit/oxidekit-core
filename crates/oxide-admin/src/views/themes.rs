//! Themes view

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;
use crate::components::{EmptyState, Tabs, TabsProps, TabItem, TabsVariant};
use crate::state::{AdminState, ThemeInfo};
use super::layout::build_page_header;

/// Build the themes view
pub fn build_themes_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    let mut children = Vec::new();

    // Page header
    let header = build_page_header(
        tree,
        "Themes",
        Some("Preview and customize application themes"),
        &[],
    );
    children.push(header);

    // Mode tabs (light/dark)
    let tabs = build_mode_tabs(tree);
    children.push(tabs);

    // Themes grid
    let themes = state.themes.all();
    let grid = build_themes_grid(tree, &themes, state.themes.active_id());
    children.push(grid);

    tree.new_node_with_children(content_style, &children)
}

fn build_mode_tabs(tree: &mut LayoutTree) -> NodeId {
    let props = TabsProps {
        items: vec![
            TabItem::new("all", "All Themes"),
            TabItem::new("dark", "Dark"),
            TabItem::new("light", "Light"),
            TabItem::new("custom", "Custom"),
        ],
        active: "all".to_string(),
        variant: TabsVariant::Pills,
        full_width: false,
    };

    Tabs::build(tree, props)
}

fn build_themes_grid(tree: &mut LayoutTree, themes: &[&ThemeInfo], active_id: Option<&str>) -> NodeId {
    let grid_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let cards: Vec<NodeId> = themes.iter()
        .map(|t| build_theme_card(tree, t, Some(t.id.as_str()) == active_id))
        .collect();

    tree.new_node_with_children(grid_style, &cards)
}

fn build_theme_card(tree: &mut LayoutTree, theme: &ThemeInfo, is_active: bool) -> NodeId {
    let card_style = StyleBuilder::new()
        .flex_column()
        .width(240.0)
        .build();

    let card_visual = NodeVisual::default()
        .with_background(hex_to_rgba("#1F2937"))
        .with_border(if is_active {
            hex_to_rgba("#3B82F6")
        } else {
            hex_to_rgba("#374151")
        }, if is_active { 2.0 } else { 1.0 })
        .with_radius(12.0);

    // Theme preview area
    let preview = build_theme_preview(tree, theme);

    // Theme info
    let info_style = StyleBuilder::new()
        .flex_column()
        .padding(12.0)
        .gap(8.0)
        .build();

    // Title row
    let title_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .justify_between()
        .build();

    let name_style = StyleBuilder::new().build();
    let name = tree.new_node(name_style);

    let mut title_children = vec![name];

    // Active badge
    if is_active {
        let badge_style = StyleBuilder::new()
            .padding_xy(4.0, 8.0)
            .build();

        let badge_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#3B82F620"))
            .with_radius(4.0);

        let badge = tree.new_visual_node(badge_style, badge_visual);
        title_children.push(badge);
    }

    let title = tree.new_node_with_children(title_style, &title_children);

    // Author
    let author_style = StyleBuilder::new().build();
    let author = tree.new_node(author_style);

    // Tags
    let tags_style = StyleBuilder::new()
        .flex_row()
        .gap(4.0)
        .build();

    let tag_nodes: Vec<NodeId> = theme.tags.iter()
        .take(3)
        .map(|tag| {
            let tag_style = StyleBuilder::new()
                .padding_xy(2.0, 6.0)
                .build();

            let tag_visual = NodeVisual::default()
                .with_background(hex_to_rgba("#374151"))
                .with_radius(4.0);

            tree.new_visual_node(tag_style, tag_visual)
        })
        .collect();

    let tags = tree.new_node_with_children(tags_style, &tag_nodes);

    let info = tree.new_node_with_children(info_style, &[title, author, tags]);

    tree.new_visual_node_with_children(card_style, card_visual, &[preview, info])
}

fn build_theme_preview(tree: &mut LayoutTree, theme: &ThemeInfo) -> NodeId {
    let preview_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .height(140.0)
        .padding(12.0)
        .gap(8.0)
        .build();

    let preview_visual = NodeVisual::default()
        .with_background(hex_to_rgba(&theme.preview_colors.background));

    // Mini UI preview
    // Header bar
    let header_style = StyleBuilder::new()
        .flex_row()
        .align_center()
        .width_percent(1.0)
        .height(24.0)
        .padding_xy(0.0, 8.0)
        .gap(4.0)
        .build();

    let header_visual = NodeVisual::default()
        .with_background(hex_to_rgba(&theme.preview_colors.surface))
        .with_radius(4.0);

    // Window controls
    let dot_style = StyleBuilder::new().size(8.0, 8.0).build();
    let dot1 = tree.new_visual_node(
        dot_style.clone(),
        NodeVisual::default().with_background(hex_to_rgba("#EF4444")).with_radius(4.0),
    );
    let dot2 = tree.new_visual_node(
        dot_style.clone(),
        NodeVisual::default().with_background(hex_to_rgba("#F59E0B")).with_radius(4.0),
    );
    let dot3 = tree.new_visual_node(
        dot_style,
        NodeVisual::default().with_background(hex_to_rgba("#22C55E")).with_radius(4.0),
    );

    let header = tree.new_visual_node_with_children(header_style, header_visual, &[dot1, dot2, dot3]);

    // Content area
    let content_style = StyleBuilder::new()
        .flex_row()
        .flex_grow(1.0)
        .width_percent(1.0)
        .gap(8.0)
        .build();

    // Sidebar
    let sidebar_style = StyleBuilder::new()
        .width(40.0)
        .height_percent(1.0)
        .build();

    let sidebar_visual = NodeVisual::default()
        .with_background(hex_to_rgba(&theme.preview_colors.surface))
        .with_radius(4.0);

    let sidebar = tree.new_visual_node(sidebar_style, sidebar_visual);

    // Main content
    let main_style = StyleBuilder::new()
        .flex_column()
        .flex_grow(1.0)
        .gap(4.0)
        .build();

    // Color swatches
    let swatch_row_style = StyleBuilder::new()
        .flex_row()
        .gap(4.0)
        .build();

    let swatch_style = StyleBuilder::new().size(20.0, 20.0).build();
    let primary = tree.new_visual_node(
        swatch_style.clone(),
        NodeVisual::default().with_background(hex_to_rgba(&theme.preview_colors.primary)).with_radius(4.0),
    );
    let secondary = tree.new_visual_node(
        swatch_style.clone(),
        NodeVisual::default().with_background(hex_to_rgba(&theme.preview_colors.secondary)).with_radius(4.0),
    );
    let accent = tree.new_visual_node(
        swatch_style,
        NodeVisual::default().with_background(hex_to_rgba(&theme.preview_colors.accent)).with_radius(4.0),
    );

    let swatches = tree.new_node_with_children(swatch_row_style, &[primary, secondary, accent]);

    // Text lines
    let text_style = StyleBuilder::new()
        .width_percent(0.8)
        .height(8.0)
        .build();

    let text_visual = NodeVisual::default()
        .with_background(hex_to_rgba(&theme.preview_colors.text))
        .with_radius(2.0);

    let text1 = tree.new_visual_node(text_style.clone(), text_visual.clone());
    let text2 = tree.new_visual_node(
        StyleBuilder::new().width_percent(0.6).height(8.0).build(),
        text_visual,
    );

    let main = tree.new_node_with_children(main_style, &[swatches, text1, text2]);

    let content = tree.new_node_with_children(content_style, &[sidebar, main]);

    tree.new_visual_node_with_children(preview_style, preview_visual, &[header, content])
}

/// Build theme preview view (full screen)
pub fn build_theme_preview_view(tree: &mut LayoutTree, state: &AdminState) -> NodeId {
    let content_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .gap(24.0)
        .build();

    if let Some(theme) = state.current_theme() {
        let header = build_page_header(
            tree,
            &theme.name,
            Some(&theme.description),
            &[],
        );

        let preview = build_full_preview(tree, theme);

        tree.new_node_with_children(content_style, &[header, preview])
    } else {
        let empty = EmptyState::error(tree, "Theme not found");
        tree.new_node_with_children(content_style, &[empty])
    }
}

fn build_full_preview(tree: &mut LayoutTree, theme: &ThemeInfo) -> NodeId {
    let preview_style = StyleBuilder::new()
        .flex_column()
        .width_percent(1.0)
        .height(500.0)
        .padding(24.0)
        .gap(24.0)
        .build();

    let preview_visual = NodeVisual::default()
        .with_background(hex_to_rgba(&theme.preview_colors.background))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(12.0);

    // Color palette display
    let palette_style = StyleBuilder::new()
        .flex_row()
        .width_percent(1.0)
        .gap(16.0)
        .build();

    let colors = [
        ("Primary", &theme.preview_colors.primary),
        ("Secondary", &theme.preview_colors.secondary),
        ("Background", &theme.preview_colors.background),
        ("Surface", &theme.preview_colors.surface),
        ("Text", &theme.preview_colors.text),
        ("Accent", &theme.preview_colors.accent),
    ];

    let color_swatches: Vec<NodeId> = colors.iter()
        .map(|(label, color)| build_color_swatch(tree, label, color))
        .collect();

    let palette = tree.new_node_with_children(palette_style, &color_swatches);

    tree.new_visual_node_with_children(preview_style, preview_visual, &[palette])
}

fn build_color_swatch(tree: &mut LayoutTree, label: &str, color: &str) -> NodeId {
    let swatch_style = StyleBuilder::new()
        .flex_column()
        .align_center()
        .gap(8.0)
        .build();

    let color_style = StyleBuilder::new()
        .size(60.0, 60.0)
        .build();

    let color_visual = NodeVisual::default()
        .with_background(hex_to_rgba(color))
        .with_border(hex_to_rgba("#374151"), 1.0)
        .with_radius(8.0);

    let color_node = tree.new_visual_node(color_style, color_visual);

    let label_style = StyleBuilder::new().build();
    let label_node = tree.new_node(label_style);

    let value_style = StyleBuilder::new().build();
    let value_node = tree.new_node(value_style);

    tree.new_node_with_children(swatch_style, &[color_node, label_node, value_node])
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
