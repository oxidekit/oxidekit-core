//! Button component
//!
//! A versatile button component with multiple variants, sizes, and states.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Button variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonVariant {
    #[default]
    Primary,
    Secondary,
    Outline,
    Ghost,
    Danger,
    Success,
    Link,
}

/// Button size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Button properties
#[derive(Debug, Clone)]
pub struct ButtonProps {
    /// Button label text
    pub label: String,

    /// Button variant
    pub variant: ButtonVariant,

    /// Button size
    pub size: ButtonSize,

    /// Whether button is disabled
    pub disabled: bool,

    /// Whether button is loading
    pub loading: bool,

    /// Icon name (left side)
    pub icon: Option<String>,

    /// Icon on right side
    pub icon_right: Option<String>,

    /// Full width button
    pub full_width: bool,

    /// Button ID for event handling
    pub id: Option<String>,
}

impl Default for ButtonProps {
    fn default() -> Self {
        Self {
            label: String::new(),
            variant: ButtonVariant::default(),
            size: ButtonSize::default(),
            disabled: false,
            loading: false,
            icon: None,
            icon_right: None,
            full_width: false,
            id: None,
        }
    }
}

impl ButtonProps {
    /// Create a new button with label
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            ..Default::default()
        }
    }

    /// Set button variant
    pub fn variant(mut self, variant: ButtonVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Set button size
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set loading state
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set full width
    pub fn full_width(mut self, full_width: bool) -> Self {
        self.full_width = full_width;
        self
    }

    /// Set ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Button component
pub struct Button;

impl Button {
    /// Build button node in layout tree
    pub fn build(tree: &mut LayoutTree, props: ButtonProps) -> NodeId {
        let (style, visual) = Self::create_style_and_visual(&props);
        tree.new_visual_node(style, visual)
    }

    /// Create style and visual for button
    fn create_style_and_visual(props: &ButtonProps) -> (Style, NodeVisual) {
        let (height, padding_x, padding_y, font_size) = match props.size {
            ButtonSize::Small => (28.0, 12.0, 4.0, 12.0),
            ButtonSize::Medium => (36.0, 16.0, 8.0, 14.0),
            ButtonSize::Large => (44.0, 20.0, 12.0, 16.0),
        };

        let mut builder = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_center()
            .height(height)
            .padding_xy(padding_y, padding_x)
            .gap(8.0);

        if props.full_width {
            builder = builder.width_percent(1.0);
        }

        let style = builder.build();

        let (bg_color, border_color, border_width) = if props.disabled {
            (
                Some(hex_to_rgba("#374151")),
                None,
                0.0,
            )
        } else {
            match props.variant {
                ButtonVariant::Primary => (Some(hex_to_rgba("#3B82F6")), None, 0.0),
                ButtonVariant::Secondary => (Some(hex_to_rgba("#4B5563")), None, 0.0),
                ButtonVariant::Outline => (None, Some(hex_to_rgba("#374151")), 1.0),
                ButtonVariant::Ghost => (None, None, 0.0),
                ButtonVariant::Danger => (Some(hex_to_rgba("#EF4444")), None, 0.0),
                ButtonVariant::Success => (Some(hex_to_rgba("#22C55E")), None, 0.0),
                ButtonVariant::Link => (None, None, 0.0),
            }
        };

        let mut visual = NodeVisual::default().with_radius(6.0);

        if let Some(bg) = bg_color {
            visual = visual.with_background(bg);
        }

        if let Some(border) = border_color {
            visual = visual.with_border(border, border_width);
        }

        (style, visual)
    }

    /// Get text color for button variant
    pub fn text_color(variant: ButtonVariant, disabled: bool) -> [f32; 4] {
        if disabled {
            hex_to_rgba("#6B7280")
        } else {
            match variant {
                ButtonVariant::Primary
                | ButtonVariant::Secondary
                | ButtonVariant::Danger
                | ButtonVariant::Success => hex_to_rgba("#FFFFFF"),
                ButtonVariant::Outline | ButtonVariant::Ghost => hex_to_rgba("#E5E7EB"),
                ButtonVariant::Link => hex_to_rgba("#3B82F6"),
            }
        }
    }
}

/// Icon-only button
pub struct IconButton;

impl IconButton {
    /// Build icon button node
    pub fn build(tree: &mut LayoutTree, icon: &str, variant: ButtonVariant, size: ButtonSize) -> NodeId {
        let dimension = match size {
            ButtonSize::Small => 28.0,
            ButtonSize::Medium => 36.0,
            ButtonSize::Large => 44.0,
        };

        let style = StyleBuilder::new()
            .size(dimension, dimension)
            .flex_row()
            .center()
            .build();

        let bg_color = match variant {
            ButtonVariant::Primary => Some(hex_to_rgba("#3B82F6")),
            ButtonVariant::Secondary => Some(hex_to_rgba("#4B5563")),
            ButtonVariant::Ghost => None,
            _ => Some(hex_to_rgba("#374151")),
        };

        let mut visual = NodeVisual::default().with_radius(dimension / 2.0);
        if let Some(bg) = bg_color {
            visual = visual.with_background(bg);
        }

        tree.new_visual_node(style, visual)
    }
}

/// Button group for grouping related buttons
pub struct ButtonGroup;

impl ButtonGroup {
    /// Build button group container
    pub fn build(tree: &mut LayoutTree, children: &[NodeId]) -> NodeId {
        let style = StyleBuilder::new()
            .flex_row()
            .gap(0.0)
            .build();

        tree.new_node_with_children(style, children)
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
    fn test_button_props() {
        let props = ButtonProps::new("Click me")
            .variant(ButtonVariant::Primary)
            .size(ButtonSize::Large);

        assert_eq!(props.label, "Click me");
        assert_eq!(props.variant, ButtonVariant::Primary);
        assert_eq!(props.size, ButtonSize::Large);
    }

    #[test]
    fn test_button_build() {
        let mut tree = LayoutTree::new();
        let props = ButtonProps::new("Test");
        let _node = Button::build(&mut tree, props);
        // Successfully built a node - test passes if no panic
    }
}
