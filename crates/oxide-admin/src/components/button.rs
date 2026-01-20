//! Button component
//!
//! A versatile button component with multiple variants, sizes, and states.
//! Supports hover, active, focus, and disabled states with proper visual feedback.

use oxide_layout::{NodeId, LayoutTree, Style, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius, sizes, typography,
    hex_to_rgba, button_colors, InteractionState,
};

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

impl ButtonVariant {
    /// Convert variant to string for token lookup
    fn as_str(&self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Outline => "outline",
            Self::Ghost => "ghost",
            Self::Danger => "danger",
            Self::Success => "success",
            Self::Link => "link",
        }
    }
}

/// Button size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ButtonSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ButtonSize {
    /// Get height for this size
    pub fn height(&self) -> f32 {
        match self {
            Self::Small => sizes::BUTTON_SM,
            Self::Medium => sizes::BUTTON_MD,
            Self::Large => sizes::BUTTON_LG,
        }
    }

    /// Get horizontal padding for this size
    pub fn padding_x(&self) -> f32 {
        match self {
            Self::Small => spacing::SM + spacing::XS,  // 12
            Self::Medium => spacing::MD,               // 16
            Self::Large => spacing::MD + spacing::XS,  // 20
        }
    }

    /// Get vertical padding for this size
    pub fn padding_y(&self) -> f32 {
        match self {
            Self::Small => spacing::XS,   // 4
            Self::Medium => spacing::SM,  // 8
            Self::Large => spacing::SM + spacing::XS,  // 12
        }
    }

    /// Get font size for this size
    pub fn font_size(&self) -> f32 {
        match self {
            Self::Small => typography::SIZE_SM,   // 12
            Self::Medium => typography::SIZE_BASE, // 14
            Self::Large => typography::SIZE_MD,   // 16
        }
    }

    /// Get icon size for this size
    pub fn icon_size(&self) -> f32 {
        match self {
            Self::Small => sizes::ICON_SM,   // 16
            Self::Medium => sizes::ICON_MD,  // 20
            Self::Large => sizes::ICON_LG,   // 24
        }
    }
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

    /// Current interaction state (for rendering)
    pub state: InteractionState,

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
            state: InteractionState::Default,
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

    /// Set interaction state (hover, active, focus)
    pub fn state(mut self, state: InteractionState) -> Self {
        self.state = state;
        self
    }

    /// Set icon
    pub fn icon(mut self, icon: impl Into<String>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Set right icon
    pub fn icon_right(mut self, icon: impl Into<String>) -> Self {
        self.icon_right = Some(icon.into());
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

    /// Get the effective interaction state (disabled overrides other states)
    fn effective_state(&self) -> InteractionState {
        if self.disabled || self.loading {
            InteractionState::Disabled
        } else {
            self.state
        }
    }
}

/// Button component
pub struct Button;

impl Button {
    /// Build button node in layout tree
    pub fn build(tree: &mut LayoutTree, props: ButtonProps) -> NodeId {
        let (style, visual) = Self::create_style_and_visual(&props);

        let mut children = Vec::new();

        // Left icon
        if let Some(_icon_name) = &props.icon {
            let icon_node = Self::build_icon(tree, &props, false);
            children.push(icon_node);
        }

        // Loading spinner (replaces content when loading)
        if props.loading {
            let spinner = Self::build_spinner(tree, &props);
            children.push(spinner);
        }

        // Right icon
        if let Some(_icon_name) = &props.icon_right {
            let icon_node = Self::build_icon(tree, &props, true);
            children.push(icon_node);
        }

        if children.is_empty() {
            tree.new_visual_node(style, visual)
        } else {
            tree.new_visual_node_with_children(style, visual, &children)
        }
    }

    /// Build button for a specific state (useful for hover/active previews)
    pub fn build_with_state(
        tree: &mut LayoutTree,
        mut props: ButtonProps,
        state: InteractionState,
    ) -> NodeId {
        props.state = state;
        Self::build(tree, props)
    }

    /// Create style and visual for button
    fn create_style_and_visual(props: &ButtonProps) -> (Style, NodeVisual) {
        let height = props.size.height();
        let padding_x = props.size.padding_x();
        let padding_y = props.size.padding_y();

        let mut builder = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_center()
            .height(height)
            .padding_xy(padding_y, padding_x)
            .gap(spacing::SM);

        if props.full_width {
            builder = builder.width_percent(1.0);
        }

        let style = builder.build();

        // Get colors based on variant and state
        let effective_state = props.effective_state();
        let (bg_color, border_color, _text_color) =
            button_colors(props.variant.as_str(), effective_state);

        let mut visual = NodeVisual::default().with_radius(radius::BUTTON);

        if let Some(bg) = bg_color {
            visual = visual.with_background(bg);
        }

        if let Some(border) = border_color {
            visual = visual.with_border(border, sizes::BORDER_DEFAULT);
        }

        // Add focus ring
        if effective_state == InteractionState::Focus {
            // Focus ring is typically rendered as an outer element or box-shadow
            // For now, we enhance the border
            visual = visual.with_border(hex_to_rgba(colors::BORDER_FOCUS), sizes::FOCUS_RING_WIDTH);
        }

        (style, visual)
    }

    /// Build icon placeholder node
    fn build_icon(tree: &mut LayoutTree, props: &ButtonProps, _is_right: bool) -> NodeId {
        let icon_size = props.size.icon_size();
        let (_, _, text_color) = button_colors(props.variant.as_str(), props.effective_state());

        let style = StyleBuilder::new()
            .size(icon_size, icon_size)
            .build();

        let visual = NodeVisual::default()
            .with_background(text_color)
            .with_radius(radius::SM);

        tree.new_visual_node(style, visual)
    }

    /// Build loading spinner
    fn build_spinner(tree: &mut LayoutTree, props: &ButtonProps) -> NodeId {
        let icon_size = props.size.icon_size();
        let (_, _, text_color) = button_colors(props.variant.as_str(), InteractionState::Default);

        let style = StyleBuilder::new()
            .size(icon_size, icon_size)
            .build();

        // Spinner would be animated in actual implementation
        let visual = NodeVisual::default()
            .with_background(text_color)
            .with_radius(icon_size / 2.0);

        tree.new_visual_node(style, visual)
    }

    /// Get text color for button variant and state
    pub fn text_color(variant: ButtonVariant, state: InteractionState) -> [f32; 4] {
        let (_, _, text_color) = button_colors(variant.as_str(), state);
        text_color
    }

    /// Get text color for disabled state
    pub fn text_color_disabled() -> [f32; 4] {
        hex_to_rgba(colors::DISABLED_TEXT)
    }
}

/// Icon-only button
pub struct IconButton;

impl IconButton {
    /// Build icon button node
    pub fn build(
        tree: &mut LayoutTree,
        _icon: &str,
        variant: ButtonVariant,
        size: ButtonSize,
    ) -> NodeId {
        Self::build_with_state(tree, _icon, variant, size, InteractionState::Default)
    }

    /// Build icon button with specific state
    pub fn build_with_state(
        tree: &mut LayoutTree,
        _icon: &str,
        variant: ButtonVariant,
        size: ButtonSize,
        state: InteractionState,
    ) -> NodeId {
        let dimension = size.height();
        let icon_size = size.icon_size();

        let style = StyleBuilder::new()
            .size(dimension, dimension)
            .flex_row()
            .center()
            .build();

        let (bg_color, border_color, text_color) = button_colors(variant.as_str(), state);

        let mut visual = NodeVisual::default().with_radius(dimension / 2.0);

        if let Some(bg) = bg_color {
            visual = visual.with_background(bg);
        }

        if let Some(border) = border_color {
            visual = visual.with_border(border, sizes::BORDER_DEFAULT);
        }

        // Add focus ring
        if state == InteractionState::Focus {
            visual = visual.with_border(hex_to_rgba(colors::BORDER_FOCUS), sizes::FOCUS_RING_WIDTH);
        }

        // Icon placeholder
        let icon_style = StyleBuilder::new()
            .size(icon_size, icon_size)
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(text_color)
            .with_radius(radius::SM);

        let icon_node = tree.new_visual_node(icon_style, icon_visual);

        tree.new_visual_node_with_children(style, visual, &[icon_node])
    }
}

/// Button group for grouping related buttons
pub struct ButtonGroup;

impl ButtonGroup {
    /// Build button group container
    pub fn build(tree: &mut LayoutTree, children: &[NodeId]) -> NodeId {
        let style = StyleBuilder::new()
            .flex_row()
            .gap(0.0) // No gap - buttons connect
            .build();

        tree.new_node_with_children(style, children)
    }

    /// Build button group with spacing
    pub fn build_spaced(tree: &mut LayoutTree, children: &[NodeId]) -> NodeId {
        let style = StyleBuilder::new()
            .flex_row()
            .gap(spacing::SM)
            .build();

        tree.new_node_with_children(style, children)
    }
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
    fn test_button_size_values() {
        assert_eq!(ButtonSize::Small.height(), sizes::BUTTON_SM);
        assert_eq!(ButtonSize::Medium.height(), sizes::BUTTON_MD);
        assert_eq!(ButtonSize::Large.height(), sizes::BUTTON_LG);
    }

    #[test]
    fn test_button_effective_state() {
        let props = ButtonProps::new("Test").disabled(true).state(InteractionState::Hover);
        assert_eq!(props.effective_state(), InteractionState::Disabled);

        let props = ButtonProps::new("Test").state(InteractionState::Hover);
        assert_eq!(props.effective_state(), InteractionState::Hover);
    }

    #[test]
    fn test_button_build() {
        let mut tree = LayoutTree::new();
        let props = ButtonProps::new("Test");
        let _node = Button::build(&mut tree, props);
        // Successfully built a node - test passes if no panic
    }

    #[test]
    fn test_button_variants_have_colors() {
        let variants = [
            ButtonVariant::Primary,
            ButtonVariant::Secondary,
            ButtonVariant::Outline,
            ButtonVariant::Ghost,
            ButtonVariant::Danger,
            ButtonVariant::Success,
            ButtonVariant::Link,
        ];

        for variant in variants {
            let _color = Button::text_color(variant, InteractionState::Default);
            // Should not panic
        }
    }

    #[test]
    fn test_icon_button_build() {
        let mut tree = LayoutTree::new();
        let _node = IconButton::build(&mut tree, "home", ButtonVariant::Ghost, ButtonSize::Medium);
        // Successfully built a node - test passes if no panic
    }
}
