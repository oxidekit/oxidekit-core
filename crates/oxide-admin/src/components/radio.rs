//! Radio button component
//!
//! Radio buttons for single-selection from a group of options.
//! Supports multiple sizes, disabled states, and proper focus handling.

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius, sizes,
    hex_to_rgba, InteractionState,
};

// =============================================================================
// RADIO SIZE
// =============================================================================

/// Radio button size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RadioSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl RadioSize {
    /// Get outer circle size
    pub fn size(&self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
        }
    }

    /// Get inner dot size when selected
    pub fn dot_size(&self) -> f32 {
        match self {
            Self::Small => 8.0,
            Self::Medium => 10.0,
            Self::Large => 12.0,
        }
    }

    /// Get border width
    pub fn border_width(&self) -> f32 {
        match self {
            Self::Small => 1.5,
            Self::Medium => 2.0,
            Self::Large => 2.0,
        }
    }
}

// =============================================================================
// RADIO OPTION
// =============================================================================

/// Radio option for radio groups
#[derive(Debug, Clone)]
pub struct RadioOption {
    /// Option value (unique identifier)
    pub value: String,
    /// Display label
    pub label: String,
    /// Optional description text
    pub description: Option<String>,
    /// Whether this option is disabled
    pub disabled: bool,
}

impl RadioOption {
    /// Create a new radio option
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: false,
        }
    }

    /// Add description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set disabled state
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

// =============================================================================
// RADIO COMPONENT
// =============================================================================

/// Radio button component
pub struct Radio;

impl Radio {
    /// Build a single radio button
    pub fn build(
        tree: &mut LayoutTree,
        selected: bool,
        label: Option<&str>,
        disabled: bool,
    ) -> NodeId {
        Self::build_with_size(tree, selected, label, disabled, RadioSize::Medium, InteractionState::Default)
    }

    /// Build a radio button with specific size and state
    pub fn build_with_size(
        tree: &mut LayoutTree,
        selected: bool,
        label: Option<&str>,
        disabled: bool,
        size: RadioSize,
        state: InteractionState,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::SM)
            .build();

        // Radio circle
        let circle_size = size.size();
        let circle_style = StyleBuilder::new()
            .size(circle_size, circle_size)
            .flex_row()
            .center()
            .build();

        let effective_state = if disabled { InteractionState::Disabled } else { state };

        let border_color = if selected {
            if disabled {
                colors::DISABLED_BG
            } else {
                colors::PRIMARY
            }
        } else {
            match effective_state {
                InteractionState::Hover => colors::BORDER_STRONG,
                InteractionState::Focus => colors::BORDER_FOCUS,
                InteractionState::Disabled => colors::DISABLED_BG,
                _ => colors::BORDER,
            }
        };

        let bg_color = if disabled {
            colors::SURFACE
        } else {
            colors::SURFACE_ELEVATED
        };

        let circle_visual = NodeVisual::default()
            .with_background(hex_to_rgba(bg_color))
            .with_border(hex_to_rgba(border_color), size.border_width())
            .with_radius(circle_size / 2.0);

        let mut radio_children = Vec::new();

        // Inner dot when selected
        if selected {
            let dot_size = size.dot_size();
            let dot_style = StyleBuilder::new()
                .size(dot_size, dot_size)
                .build();

            let dot_color = if disabled {
                colors::DISABLED_TEXT
            } else {
                colors::PRIMARY
            };

            let dot_visual = NodeVisual::default()
                .with_background(hex_to_rgba(dot_color))
                .with_radius(dot_size / 2.0);

            let dot = tree.new_visual_node(dot_style, dot_visual);
            radio_children.push(dot);
        }

        let radio = if radio_children.is_empty() {
            tree.new_visual_node(circle_style, circle_visual)
        } else {
            tree.new_visual_node_with_children(circle_style, circle_visual, &radio_children)
        };

        // Add label if provided
        if label.is_some() {
            let label_style = StyleBuilder::new().build();
            let label_node = tree.new_node(label_style);
            tree.new_node_with_children(container_style, &[radio, label_node])
        } else {
            tree.new_node_with_children(container_style, &[radio])
        }
    }

    /// Build radio with label and description
    pub fn build_with_description(
        tree: &mut LayoutTree,
        selected: bool,
        label: &str,
        description: &str,
        disabled: bool,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_start()
            .gap(spacing::SM)
            .build();

        // Radio circle
        let radio = Self::build(tree, selected, None, disabled);

        // Text content
        let text_style = StyleBuilder::new()
            .flex_column()
            .gap(spacing::XS)
            .build();

        // Label
        let label_style = StyleBuilder::new().build();
        let label_node = tree.new_node(label_style);

        // Description
        let desc_style = StyleBuilder::new().build();
        let desc_node = tree.new_node(desc_style);

        let text = tree.new_node_with_children(text_style, &[label_node, desc_node]);

        tree.new_node_with_children(container_style, &[radio, text])
    }
}

// =============================================================================
// RADIO GROUP COMPONENT
// =============================================================================

/// Radio group properties
#[derive(Debug, Clone)]
pub struct RadioGroupProps {
    /// Options to display
    pub options: Vec<RadioOption>,
    /// Currently selected value
    pub value: Option<String>,
    /// Group name/ID
    pub name: String,
    /// Size of radio buttons
    pub size: RadioSize,
    /// Layout direction
    pub direction: RadioGroupDirection,
    /// Whether entire group is disabled
    pub disabled: bool,
    /// Label for the group
    pub label: Option<String>,
    /// Error message
    pub error: Option<String>,
}

/// Radio group layout direction
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum RadioGroupDirection {
    #[default]
    Vertical,
    Horizontal,
}

impl Default for RadioGroupProps {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            value: None,
            name: String::new(),
            size: RadioSize::Medium,
            direction: RadioGroupDirection::Vertical,
            disabled: false,
            label: None,
            error: None,
        }
    }
}

impl RadioGroupProps {
    /// Create a new radio group
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set options
    pub fn options(mut self, options: Vec<RadioOption>) -> Self {
        self.options = options;
        self
    }

    /// Set selected value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set size
    pub fn size(mut self, size: RadioSize) -> Self {
        self.size = size;
        self
    }

    /// Set direction
    pub fn direction(mut self, direction: RadioGroupDirection) -> Self {
        self.direction = direction;
        self
    }

    /// Set disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set error
    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }
}

/// Radio group component
pub struct RadioGroup;

impl RadioGroup {
    /// Build a radio group
    pub fn build(tree: &mut LayoutTree, props: RadioGroupProps) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .gap(spacing::SM)
            .build();

        let mut children = Vec::new();

        // Group label
        if let Some(_label) = &props.label {
            let label_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .build();
            let label_node = tree.new_node(label_style);
            children.push(label_node);
        }

        // Options container
        let options_style = match props.direction {
            RadioGroupDirection::Vertical => StyleBuilder::new()
                .flex_column()
                .gap(spacing::SM)
                .build(),
            RadioGroupDirection::Horizontal => StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(spacing::MD)
                .build(),
        };

        let option_nodes: Vec<NodeId> = props.options.iter()
            .map(|opt| {
                let selected = props.value.as_ref() == Some(&opt.value);
                let disabled = props.disabled || opt.disabled;

                if let Some(desc) = &opt.description {
                    Radio::build_with_description(tree, selected, &opt.label, desc, disabled)
                } else {
                    Radio::build(tree, selected, Some(&opt.label), disabled)
                }
            })
            .collect();

        let options = tree.new_node_with_children(options_style, &option_nodes);
        children.push(options);

        // Error message
        if let Some(_error) = &props.error {
            let error_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(spacing::XS)
                .build();

            // Error icon
            let icon_style = StyleBuilder::new()
                .size(sizes::ICON_SM, sizes::ICON_SM)
                .build();
            let icon_visual = NodeVisual::default()
                .with_background(hex_to_rgba(colors::DANGER))
                .with_radius(radius::SM);
            let icon = tree.new_visual_node(icon_style, icon_visual);

            // Error text
            let text_style = StyleBuilder::new().build();
            let text = tree.new_node(text_style);

            let error_node = tree.new_node_with_children(error_style, &[icon, text]);
            children.push(error_node);
        }

        tree.new_node_with_children(container_style, &children)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_radio_size() {
        assert_eq!(RadioSize::Small.size(), 16.0);
        assert_eq!(RadioSize::Medium.size(), 20.0);
        assert_eq!(RadioSize::Large.size(), 24.0);
    }

    #[test]
    fn test_radio_build() {
        let mut tree = LayoutTree::new();
        let _node = Radio::build(&mut tree, true, Some("Option 1"), false);
    }

    #[test]
    fn test_radio_group_build() {
        let mut tree = LayoutTree::new();
        let props = RadioGroupProps::new("test-group")
            .options(vec![
                RadioOption::new("a", "Option A"),
                RadioOption::new("b", "Option B"),
                RadioOption::new("c", "Option C"),
            ])
            .value("a")
            .label("Choose an option");

        let _node = RadioGroup::build(&mut tree, props);
    }

    #[test]
    fn test_radio_option() {
        let option = RadioOption::new("val", "Label")
            .description("Some description")
            .disabled(true);

        assert_eq!(option.value, "val");
        assert_eq!(option.label, "Label");
        assert!(option.description.is_some());
        assert!(option.disabled);
    }
}
