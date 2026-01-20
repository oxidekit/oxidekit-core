//! Command palette (Cmd/Ctrl+K) component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Command palette item
#[derive(Debug, Clone)]
pub struct CommandItem {
    pub id: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub shortcut: Option<String>,
    pub category: Option<String>,
}

impl CommandItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            icon: None,
            shortcut: None,
            category: None,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// Command palette properties
#[derive(Debug, Clone)]
pub struct CommandPaletteProps {
    pub query: String,
    pub items: Vec<CommandItem>,
    pub selected_index: usize,
    pub placeholder: String,
}

impl Default for CommandPaletteProps {
    fn default() -> Self {
        Self {
            query: String::new(),
            items: Vec::new(),
            selected_index: 0,
            placeholder: "Type a command or search...".to_string(),
        }
    }
}

/// Command palette component
pub struct CommandPalette;

impl CommandPalette {
    pub fn build(tree: &mut LayoutTree, props: CommandPaletteProps) -> NodeId {
        // Backdrop
        let backdrop_style = StyleBuilder::new()
            .size_full()
            .flex_column()
            .align_center()
            .padding(100.0)
            .build();

        let backdrop_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#00000080"));

        // Palette container
        let palette_style = StyleBuilder::new()
            .flex_column()
            .width(640.0)
            .build();

        let palette_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(12.0);

        // Search input
        let search_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(56.0)
            .padding_xy(0.0, 16.0)
            .gap(12.0)
            .build();

        let search_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        let search = tree.new_visual_node(search_style, search_visual);

        // Results container
        let results_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .padding(8.0)
            .gap(2.0)
            .build();

        let result_nodes: Vec<NodeId> = props.items.iter()
            .enumerate()
            .take(10) // Limit visible results
            .map(|(i, item)| Self::build_item(tree, item, i == props.selected_index))
            .collect();

        let results = tree.new_node_with_children(results_style, &result_nodes);

        // Footer
        let footer_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(40.0)
            .padding_xy(8.0, 16.0)
            .build();

        let footer_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        let footer = tree.new_visual_node(footer_style, footer_visual);

        let palette = tree.new_visual_node_with_children(
            palette_style,
            palette_visual,
            &[search, results, footer],
        );

        tree.new_visual_node_with_children(backdrop_style, backdrop_visual, &[palette])
    }

    fn build_item(tree: &mut LayoutTree, item: &CommandItem, selected: bool) -> NodeId {
        let item_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(44.0)
            .padding_xy(8.0, 12.0)
            .gap(12.0)
            .build();

        let item_visual = if selected {
            NodeVisual::default()
                .with_background(hex_to_rgba("#3B82F620"))
                .with_radius(8.0)
        } else {
            NodeVisual::default().with_radius(8.0)
        };

        // Icon placeholder
        let icon_style = StyleBuilder::new().size(20.0, 20.0).build();
        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#6B7280"))
            .with_radius(4.0);
        let icon = tree.new_visual_node(icon_style, icon_visual);

        // Label
        let label_style = StyleBuilder::new().flex_grow(1.0).build();
        let label = tree.new_node(label_style);

        // Shortcut
        if let Some(_shortcut) = &item.shortcut {
            let shortcut_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .padding_xy(4.0, 8.0)
                .build();

            let shortcut_visual = NodeVisual::default()
                .with_background(hex_to_rgba("#374151"))
                .with_radius(4.0);

            let shortcut_node = tree.new_visual_node(shortcut_style, shortcut_visual);

            tree.new_visual_node_with_children(item_style, item_visual, &[icon, label, shortcut_node])
        } else {
            tree.new_visual_node_with_children(item_style, item_visual, &[icon, label])
        }
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
