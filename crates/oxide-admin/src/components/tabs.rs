//! Tabs component

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Tab item
#[derive(Debug, Clone)]
pub struct TabItem {
    pub id: String,
    pub label: String,
    pub icon: Option<String>,
    pub badge: Option<u32>,
    pub disabled: bool,
}

impl TabItem {
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            disabled: false,
        }
    }

    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn badge(mut self, count: u32) -> Self {
        self.badge = Some(count);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Tabs variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TabsVariant {
    #[default]
    Line,
    Pills,
    Boxed,
}

/// Tabs properties
#[derive(Debug, Clone)]
pub struct TabsProps {
    pub items: Vec<TabItem>,
    pub active: String,
    pub variant: TabsVariant,
    pub full_width: bool,
}

impl Default for TabsProps {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            active: String::new(),
            variant: TabsVariant::Line,
            full_width: false,
        }
    }
}

/// Tabs component
pub struct Tabs;

impl Tabs {
    pub fn build(tree: &mut LayoutTree, props: TabsProps) -> NodeId {
        let tabs_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(if props.variant == TabsVariant::Pills { 8.0 } else { 0.0 })
            .build();

        let tabs_visual = match props.variant {
            TabsVariant::Line => NodeVisual::default()
                .with_border(hex_to_rgba("#374151"), 1.0),
            TabsVariant::Pills => NodeVisual::default(),
            TabsVariant::Boxed => NodeVisual::default()
                .with_background(hex_to_rgba("#1F2937"))
                .with_radius(8.0),
        };

        let tab_nodes: Vec<NodeId> = props.items.iter()
            .map(|item| Self::build_tab(tree, item, &props))
            .collect();

        tree.new_visual_node_with_children(tabs_style, tabs_visual, &tab_nodes)
    }

    fn build_tab(tree: &mut LayoutTree, item: &TabItem, props: &TabsProps) -> NodeId {
        let is_active = item.id == props.active;

        let tab_style_builder = StyleBuilder::new()
            .flex_row()
            .align_center()
            .height(40.0)
            .padding_xy(8.0, 16.0)
            .gap(8.0);

        let tab_style = if props.full_width {
            tab_style_builder.flex_grow(1.0).build()
        } else {
            tab_style_builder.build()
        };

        let tab_visual = match props.variant {
            TabsVariant::Line => {
                let mut v = NodeVisual::default();
                if is_active {
                    v = v.with_border(hex_to_rgba("#3B82F6"), 2.0);
                }
                v
            }
            TabsVariant::Pills => {
                if is_active {
                    NodeVisual::default()
                        .with_background(hex_to_rgba("#3B82F6"))
                        .with_radius(6.0)
                } else {
                    NodeVisual::default().with_radius(6.0)
                }
            }
            TabsVariant::Boxed => {
                if is_active {
                    NodeVisual::default()
                        .with_background(hex_to_rgba("#374151"))
                        .with_radius(6.0)
                } else {
                    NodeVisual::default().with_radius(6.0)
                }
            }
        };

        tree.new_visual_node(tab_style, tab_visual)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
