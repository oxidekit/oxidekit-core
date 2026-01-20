//! Tooltip component
//!
//! Contextual information popups that appear on hover or focus.
//! Supports multiple positions, delays, and rich content.

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius,
    hex_to_rgba,
};

// =============================================================================
// TOOLTIP PLACEMENT
// =============================================================================

/// Tooltip placement relative to trigger
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TooltipPlacement {
    #[default]
    Top,
    TopStart,
    TopEnd,
    Bottom,
    BottomStart,
    BottomEnd,
    Left,
    LeftStart,
    LeftEnd,
    Right,
    RightStart,
    RightEnd,
}

impl TooltipPlacement {
    /// Get arrow position for this placement
    fn arrow_position(&self) -> ArrowPosition {
        match self {
            Self::Top | Self::TopStart | Self::TopEnd => ArrowPosition::Bottom,
            Self::Bottom | Self::BottomStart | Self::BottomEnd => ArrowPosition::Top,
            Self::Left | Self::LeftStart | Self::LeftEnd => ArrowPosition::Right,
            Self::Right | Self::RightStart | Self::RightEnd => ArrowPosition::Left,
        }
    }
}

/// Arrow position on tooltip
#[derive(Debug, Clone, Copy, PartialEq)]
enum ArrowPosition {
    Top,
    Bottom,
    Left,
    Right,
}

// =============================================================================
// TOOLTIP VARIANT
// =============================================================================

/// Tooltip style variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum TooltipVariant {
    #[default]
    Dark,
    Light,
    Primary,
    Success,
    Warning,
    Danger,
}

impl TooltipVariant {
    /// Get background color
    fn bg_color(&self) -> &'static str {
        match self {
            Self::Dark => colors::SURFACE_ELEVATED,
            Self::Light => colors::TEXT_PRIMARY,
            Self::Primary => colors::PRIMARY,
            Self::Success => colors::SUCCESS,
            Self::Warning => colors::WARNING,
            Self::Danger => colors::DANGER,
        }
    }

    /// Get text color
    fn text_color(&self) -> &'static str {
        match self {
            Self::Dark => colors::TEXT_PRIMARY,
            Self::Light => colors::BACKGROUND,
            Self::Primary | Self::Success | Self::Danger => colors::PRIMARY_CONTRAST,
            Self::Warning => colors::BACKGROUND,
        }
    }

    /// Get border color (optional)
    fn border_color(&self) -> Option<&'static str> {
        match self {
            Self::Dark => Some(colors::BORDER),
            Self::Light => None,
            _ => None,
        }
    }
}

// =============================================================================
// TOOLTIP PROPERTIES
// =============================================================================

/// Tooltip properties
#[derive(Debug, Clone)]
pub struct TooltipProps {
    /// Tooltip content text
    pub content: String,
    /// Placement relative to trigger
    pub placement: TooltipPlacement,
    /// Style variant
    pub variant: TooltipVariant,
    /// Show arrow
    pub show_arrow: bool,
    /// Delay before showing (ms)
    pub delay_show: u32,
    /// Delay before hiding (ms)
    pub delay_hide: u32,
    /// Maximum width
    pub max_width: Option<f32>,
    /// Whether tooltip is currently visible
    pub visible: bool,
}

impl Default for TooltipProps {
    fn default() -> Self {
        Self {
            content: String::new(),
            placement: TooltipPlacement::Top,
            variant: TooltipVariant::Dark,
            show_arrow: true,
            delay_show: 200,
            delay_hide: 0,
            max_width: Some(300.0),
            visible: false,
        }
    }
}

impl TooltipProps {
    /// Create a tooltip with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set placement
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set variant
    pub fn variant(mut self, variant: TooltipVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Hide arrow
    pub fn no_arrow(mut self) -> Self {
        self.show_arrow = false;
        self
    }

    /// Set show delay
    pub fn delay_show(mut self, ms: u32) -> Self {
        self.delay_show = ms;
        self
    }

    /// Set hide delay
    pub fn delay_hide(mut self, ms: u32) -> Self {
        self.delay_hide = ms;
        self
    }

    /// Set max width
    pub fn max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }
}

// =============================================================================
// TOOLTIP COMPONENT
// =============================================================================

/// Tooltip component
pub struct Tooltip;

impl Tooltip {
    /// Build tooltip (content only, positioning handled by parent)
    pub fn build(tree: &mut LayoutTree, props: TooltipProps) -> NodeId {
        if !props.visible {
            // Return an empty/hidden node when not visible
            let style = StyleBuilder::new()
                .size(0.0, 0.0)
                .build();
            return tree.new_node(style);
        }

        let mut container_style = StyleBuilder::new()
            .flex_column()
            .padding_xy(spacing::SM, spacing::SM + spacing::XS);

        if let Some(max_w) = props.max_width {
            // Would use max_width if available, using fixed width as approximation
            container_style = container_style.width(max_w);
        }

        let container_style = container_style.build();

        let mut container_visual = NodeVisual::default()
            .with_background(hex_to_rgba(props.variant.bg_color()))
            .with_radius(radius::MD);

        if let Some(border) = props.variant.border_color() {
            container_visual = container_visual.with_border(hex_to_rgba(border), 1.0);
        }

        let mut children = Vec::new();

        // Content
        let content_style = StyleBuilder::new().build();
        let content = tree.new_node(content_style);
        children.push(content);

        let tooltip = tree.new_visual_node_with_children(container_style, container_visual, &children);

        // Add arrow if needed
        if props.show_arrow {
            let arrow = Self::build_arrow(tree, &props);
            let wrapper_style = StyleBuilder::new()
                .flex_column()
                .align_center()
                .build();

            match props.placement.arrow_position() {
                ArrowPosition::Bottom => {
                    tree.new_node_with_children(wrapper_style, &[tooltip, arrow])
                }
                ArrowPosition::Top => {
                    tree.new_node_with_children(wrapper_style, &[arrow, tooltip])
                }
                ArrowPosition::Left | ArrowPosition::Right => {
                    let row_style = StyleBuilder::new()
                        .flex_row()
                        .align_center()
                        .build();
                    if matches!(props.placement.arrow_position(), ArrowPosition::Left) {
                        tree.new_node_with_children(row_style, &[arrow, tooltip])
                    } else {
                        tree.new_node_with_children(row_style, &[tooltip, arrow])
                    }
                }
            }
        } else {
            tooltip
        }
    }

    /// Build tooltip arrow
    fn build_arrow(tree: &mut LayoutTree, props: &TooltipProps) -> NodeId {
        let arrow_size = 8.0;

        let arrow_style = StyleBuilder::new()
            .size(arrow_size, arrow_size)
            .build();

        // Arrow is a rotated square - simplified as a small square
        // In real implementation, would use CSS rotation or SVG triangle
        let arrow_visual = NodeVisual::default()
            .with_background(hex_to_rgba(props.variant.bg_color()))
            .with_radius(2.0);

        tree.new_visual_node(arrow_style, arrow_visual)
    }

    /// Build tooltip wrapper with trigger and tooltip
    pub fn build_wrapper(
        tree: &mut LayoutTree,
        trigger: NodeId,
        props: TooltipProps,
    ) -> NodeId {
        let wrapper_style = StyleBuilder::new()
            .build();

        let tooltip = Self::build(tree, props);

        tree.new_node_with_children(wrapper_style, &[trigger, tooltip])
    }
}

// =============================================================================
// POPOVER COMPONENT
// =============================================================================

/// Popover properties (richer tooltip variant)
#[derive(Debug, Clone)]
pub struct PopoverProps {
    /// Popover title
    pub title: Option<String>,
    /// Popover content
    pub content: String,
    /// Placement
    pub placement: TooltipPlacement,
    /// Whether popover is visible
    pub visible: bool,
    /// Close on outside click
    pub close_on_click_outside: bool,
    /// Show close button
    pub show_close: bool,
    /// Max width
    pub max_width: f32,
}

impl Default for PopoverProps {
    fn default() -> Self {
        Self {
            title: None,
            content: String::new(),
            placement: TooltipPlacement::Bottom,
            visible: false,
            close_on_click_outside: true,
            show_close: false,
            max_width: 320.0,
        }
    }
}

impl PopoverProps {
    /// Create a popover with content
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            content: content.into(),
            ..Default::default()
        }
    }

    /// Set title
    pub fn title(mut self, title: impl Into<String>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set placement
    pub fn placement(mut self, placement: TooltipPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set visibility
    pub fn visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    /// Show close button
    pub fn with_close(mut self) -> Self {
        self.show_close = true;
        self
    }
}

/// Popover component
pub struct Popover;

impl Popover {
    /// Build a popover
    pub fn build(tree: &mut LayoutTree, props: PopoverProps) -> NodeId {
        if !props.visible {
            let style = StyleBuilder::new()
                .size(0.0, 0.0)
                .build();
            return tree.new_node(style);
        }

        let container_style = StyleBuilder::new()
            .flex_column()
            .width(props.max_width)
            .build();

        let container_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::SURFACE))
            .with_border(hex_to_rgba(colors::BORDER), 1.0)
            .with_radius(radius::LG);

        let mut children = Vec::new();

        // Header (if title or close button)
        if props.title.is_some() || props.show_close {
            let header_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .justify_between()
                .width_percent(1.0)
                .padding_xy(spacing::SM + spacing::XS, spacing::MD)
                .build();

            let header_visual = NodeVisual::default()
                .with_border(hex_to_rgba(colors::BORDER), 1.0);

            let mut header_children = Vec::new();

            // Title
            if props.title.is_some() {
                let title_style = StyleBuilder::new().build();
                let title = tree.new_node(title_style);
                header_children.push(title);
            }

            // Close button
            if props.show_close {
                let close_style = StyleBuilder::new()
                    .size(24.0, 24.0)
                    .flex_row()
                    .center()
                    .build();
                let close_visual = NodeVisual::default()
                    .with_radius(radius::SM);
                let close = tree.new_visual_node(close_style, close_visual);
                header_children.push(close);
            }

            let header = tree.new_visual_node_with_children(header_style, header_visual, &header_children);
            children.push(header);
        }

        // Content
        let content_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .padding(spacing::MD)
            .build();

        let content = tree.new_node(content_style);
        children.push(content);

        tree.new_visual_node_with_children(container_style, container_visual, &children)
    }
}

// =============================================================================
// HELP TEXT / INFO ICON
// =============================================================================

/// Help text with info icon and tooltip
pub struct HelpText;

impl HelpText {
    /// Build help text with icon
    pub fn build(tree: &mut LayoutTree, _text: &str) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::XS)
            .build();

        // Info icon
        let icon_style = StyleBuilder::new()
            .size(16.0, 16.0)
            .flex_row()
            .center()
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::INFO))
            .with_radius(8.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);

        tree.new_node_with_children(container_style, &[icon])
    }

    /// Build info icon with tooltip
    pub fn build_with_tooltip(tree: &mut LayoutTree, tooltip_text: &str) -> NodeId {
        let icon_style = StyleBuilder::new()
            .size(16.0, 16.0)
            .flex_row()
            .center()
            .build();

        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::TEXT_SECONDARY))
            .with_radius(8.0);

        let icon = tree.new_visual_node(icon_style, icon_visual);

        // Tooltip
        let tooltip = Tooltip::build(
            tree,
            TooltipProps::new(tooltip_text)
                .placement(TooltipPlacement::Top)
                .visible(false), // Would be shown on hover via events
        );

        let wrapper_style = StyleBuilder::new().build();
        tree.new_node_with_children(wrapper_style, &[icon, tooltip])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tooltip_props() {
        let props = TooltipProps::new("Hello world")
            .placement(TooltipPlacement::Bottom)
            .variant(TooltipVariant::Primary)
            .delay_show(500);

        assert_eq!(props.content, "Hello world");
        assert_eq!(props.placement, TooltipPlacement::Bottom);
        assert_eq!(props.delay_show, 500);
    }

    #[test]
    fn test_tooltip_build() {
        let mut tree = LayoutTree::new();
        let props = TooltipProps::new("Tooltip content").visible(true);
        let _node = Tooltip::build(&mut tree, props);
    }

    #[test]
    fn test_tooltip_hidden() {
        let mut tree = LayoutTree::new();
        let props = TooltipProps::new("Hidden").visible(false);
        let _node = Tooltip::build(&mut tree, props);
    }

    #[test]
    fn test_popover_build() {
        let mut tree = LayoutTree::new();
        let props = PopoverProps::new("Popover content")
            .title("Popover Title")
            .with_close()
            .visible(true);
        let _node = Popover::build(&mut tree, props);
    }

    #[test]
    fn test_placement_arrow_position() {
        assert!(matches!(
            TooltipPlacement::Top.arrow_position(),
            ArrowPosition::Bottom
        ));
        assert!(matches!(
            TooltipPlacement::Bottom.arrow_position(),
            ArrowPosition::Top
        ));
        assert!(matches!(
            TooltipPlacement::Left.arrow_position(),
            ArrowPosition::Right
        ));
        assert!(matches!(
            TooltipPlacement::Right.arrow_position(),
            ArrowPosition::Left
        ));
    }
}
