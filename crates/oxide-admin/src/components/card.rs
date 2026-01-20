//! Card component
//!
//! Container component for grouping related content with optional header, footer, and actions.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Card variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CardVariant {
    #[default]
    Default,
    Elevated,
    Outlined,
    Flat,
}

/// Card properties
#[derive(Debug, Clone)]
pub struct CardProps {
    /// Card variant
    pub variant: CardVariant,

    /// Card title
    pub title: Option<String>,

    /// Card subtitle
    pub subtitle: Option<String>,

    /// Whether card is clickable
    pub clickable: bool,

    /// Whether card is selected
    pub selected: bool,

    /// Padding size
    pub padding: CardPadding,

    /// Card ID for event handling
    pub id: Option<String>,
}

/// Card padding options
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CardPadding {
    None,
    Small,
    #[default]
    Medium,
    Large,
}

impl Default for CardProps {
    fn default() -> Self {
        Self {
            variant: CardVariant::default(),
            title: None,
            subtitle: None,
            clickable: false,
            selected: false,
            padding: CardPadding::default(),
            id: None,
        }
    }
}

impl CardProps {
    /// Create a new card with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set card variant
    pub fn variant(mut self, variant: CardVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set card title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set card subtitle
    pub fn subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Set clickable state
    pub fn clickable(mut self, clickable: bool) -> Self {
        self.clickable = clickable;
        self
    }

    /// Set selected state
    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    /// Set padding
    pub fn padding(mut self, padding: CardPadding) -> Self {
        self.padding = padding;
        self
    }

    /// Set ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Card component
pub struct Card;

impl Card {
    /// Build card node in layout tree
    pub fn build(tree: &mut LayoutTree, props: CardProps, children: &[NodeId]) -> NodeId {
        let (style, visual) = Self::create_style_and_visual(&props);

        if children.is_empty() {
            tree.new_visual_node(style, visual)
        } else {
            tree.new_visual_node_with_children(style, visual, children)
        }
    }

    /// Build card with header
    pub fn build_with_header(
        tree: &mut LayoutTree,
        props: CardProps,
        header_children: &[NodeId],
        body_children: &[NodeId],
    ) -> NodeId {
        // Header section
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .padding_xy(12.0, 16.0)
            .build();

        let header_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        let header = if header_children.is_empty() {
            tree.new_visual_node(header_style, header_visual)
        } else {
            tree.new_visual_node_with_children(header_style, header_visual, header_children)
        };

        // Body section
        let padding = match props.padding {
            CardPadding::None => 0.0,
            CardPadding::Small => 8.0,
            CardPadding::Medium => 16.0,
            CardPadding::Large => 24.0,
        };

        let body_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .width_percent(1.0)
            .padding(padding)
            .build();

        let body = if body_children.is_empty() {
            tree.new_node(body_style)
        } else {
            tree.new_node_with_children(body_style, body_children)
        };

        // Card container
        let (card_style, card_visual) = Self::create_style_and_visual(&props);
        tree.new_visual_node_with_children(card_style, card_visual, &[header, body])
    }

    /// Build card with header and footer
    pub fn build_full(
        tree: &mut LayoutTree,
        props: CardProps,
        header_children: &[NodeId],
        body_children: &[NodeId],
        footer_children: &[NodeId],
    ) -> NodeId {
        // Header section
        let header_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .padding_xy(12.0, 16.0)
            .build();

        let header_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        let header = tree.new_visual_node_with_children(header_style, header_visual, header_children);

        // Body section
        let padding = match props.padding {
            CardPadding::None => 0.0,
            CardPadding::Small => 8.0,
            CardPadding::Medium => 16.0,
            CardPadding::Large => 24.0,
        };

        let body_style = StyleBuilder::new()
            .flex_column()
            .flex_grow(1.0)
            .width_percent(1.0)
            .padding(padding)
            .build();

        let body = tree.new_node_with_children(body_style, body_children);

        // Footer section
        let footer_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_end()
            .width_percent(1.0)
            .padding_xy(12.0, 16.0)
            .gap(8.0)
            .build();

        let footer_visual = NodeVisual::default()
            .with_border(hex_to_rgba("#374151"), 1.0);

        let footer = tree.new_visual_node_with_children(footer_style, footer_visual, footer_children);

        // Card container
        let (card_style, card_visual) = Self::create_style_and_visual(&props);
        tree.new_visual_node_with_children(card_style, card_visual, &[header, body, footer])
    }

    /// Create style and visual for card
    fn create_style_and_visual(props: &CardProps) -> (Style, NodeVisual) {
        let padding = match props.padding {
            CardPadding::None => 0.0,
            CardPadding::Small => 8.0,
            CardPadding::Medium => 16.0,
            CardPadding::Large => 24.0,
        };

        let style = StyleBuilder::new()
            .flex_column()
            .padding(padding)
            .build();

        let (bg_color, border_color, border_width, radius) = match props.variant {
            CardVariant::Default => (
                Some(hex_to_rgba("#1F2937")),
                Some(hex_to_rgba("#374151")),
                1.0,
                12.0,
            ),
            CardVariant::Elevated => (
                Some(hex_to_rgba("#1F2937")),
                None,
                0.0,
                12.0,
            ),
            CardVariant::Outlined => (
                None,
                Some(hex_to_rgba("#374151")),
                1.0,
                12.0,
            ),
            CardVariant::Flat => (
                Some(hex_to_rgba("#111827")),
                None,
                0.0,
                8.0,
            ),
        };

        let mut visual = NodeVisual::default().with_radius(radius);

        if let Some(bg) = bg_color {
            visual = visual.with_background(bg);
        }

        if let Some(border) = border_color {
            visual = visual.with_border(border, border_width);
        }

        // Selection state
        if props.selected {
            visual = visual.with_border(hex_to_rgba("#3B82F6"), 2.0);
        }

        (style, visual)
    }
}

/// Stat card for displaying metrics
pub struct StatCard;

impl StatCard {
    /// Build stat card
    pub fn build(
        tree: &mut LayoutTree,
        label: &str,
        value: &str,
        change: Option<(&str, bool)>, // (change text, is positive)
        icon: Option<&str>,
    ) -> NodeId {
        let card_style = StyleBuilder::new()
            .flex_column()
            .padding(16.0)
            .gap(8.0)
            .build();

        let card_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(12.0);

        // Label row
        let label_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .build();

        let label_node = tree.new_node(label_style);

        // Value
        let value_style = StyleBuilder::new()
            .height(32.0)
            .build();

        let value_node = tree.new_node(value_style);

        // Change indicator (if provided)
        let mut children = vec![label_node, value_node];

        if let Some((change_text, is_positive)) = change {
            let change_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(4.0)
                .build();

            let change_visual = NodeVisual::default()
                .with_background(if is_positive {
                    hex_to_rgba("#22C55E20")
                } else {
                    hex_to_rgba("#EF444420")
                })
                .with_radius(4.0);

            let change_node = tree.new_visual_node(change_style, change_visual);
            children.push(change_node);
        }

        tree.new_visual_node_with_children(card_style, card_visual, &children)
    }
}

/// Media card for content with image
pub struct MediaCard;

impl MediaCard {
    /// Build media card with image placeholder
    pub fn build(
        tree: &mut LayoutTree,
        image_height: f32,
        title: &str,
        description: Option<&str>,
    ) -> NodeId {
        // Image placeholder
        let image_style = StyleBuilder::new()
            .width_percent(1.0)
            .height(image_height)
            .build();

        let image_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#374151"));

        let image = tree.new_visual_node(image_style, image_visual);

        // Content section
        let content_style = StyleBuilder::new()
            .flex_column()
            .padding(16.0)
            .gap(8.0)
            .build();

        let content = tree.new_node(content_style);

        // Card container
        let card_style = StyleBuilder::new()
            .flex_column()
            .build();

        let card_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#1F2937"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(12.0);

        tree.new_visual_node_with_children(card_style, card_visual, &[image, content])
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
    fn test_card_props() {
        let props = CardProps::new()
            .variant(CardVariant::Elevated)
            .title("Test Card")
            .clickable(true);

        assert_eq!(props.variant, CardVariant::Elevated);
        assert_eq!(props.title, Some("Test Card".to_string()));
        assert!(props.clickable);
    }

    #[test]
    fn test_card_build() {
        let mut tree = LayoutTree::new();
        let props = CardProps::new();
        let _node = Card::build(&mut tree, props, &[]);
        // Successfully built a node - test passes if no panic
    }
}
