//! Form components (input, select, checkbox, etc.)

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};
use oxide_render::Color;

/// Input size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputSize {
    Small,
    #[default]
    Medium,
    Large,
}

/// Input properties
#[derive(Debug, Clone)]
pub struct InputProps {
    pub value: String,
    pub placeholder: String,
    pub label: Option<String>,
    pub error: Option<String>,
    pub helper: Option<String>,
    pub size: InputSize,
    pub disabled: bool,
    pub readonly: bool,
    pub required: bool,
    pub input_type: InputType,
    pub prefix: Option<String>,
    pub suffix: Option<String>,
}

/// Input type
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputType {
    #[default]
    Text,
    Password,
    Email,
    Number,
    Search,
    Url,
    Tel,
}

impl Default for InputProps {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: String::new(),
            label: None,
            error: None,
            helper: None,
            size: InputSize::Medium,
            disabled: false,
            readonly: false,
            required: false,
            input_type: InputType::Text,
            prefix: None,
            suffix: None,
        }
    }
}

/// Text input component
pub struct Input;

impl Input {
    pub fn build(tree: &mut LayoutTree, props: InputProps) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(6.0)
            .build();

        let mut children = Vec::new();

        // Label
        if let Some(_label) = &props.label {
            let label_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(4.0)
                .build();
            let label = tree.new_node(label_style);
            children.push(label);
        }

        // Input wrapper
        let height = match props.size {
            InputSize::Small => 32.0,
            InputSize::Medium => 40.0,
            InputSize::Large => 48.0,
        };

        let input_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(height)
            .padding_xy(0.0, 12.0)
            .build();

        let border_color = if props.error.is_some() {
            "#EF4444"
        } else if props.disabled {
            "#4B5563"
        } else {
            "#374151"
        };

        let input_visual = NodeVisual::default()
            .with_background(if props.disabled { hex_to_rgba("#1F2937") } else { hex_to_rgba("#111827") })
            .with_border(hex_to_rgba(border_color), 1.0)
            .with_radius(8.0);

        let input = tree.new_visual_node(input_style, input_visual);
        children.push(input);

        // Error or helper text
        if let Some(_error) = &props.error {
            let error_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .gap(4.0)
                .build();
            let error = tree.new_node(error_style);
            children.push(error);
        } else if let Some(_helper) = &props.helper {
            let helper_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .build();
            let helper = tree.new_node(helper_style);
            children.push(helper);
        }

        tree.new_node_with_children(container_style, &children)
    }
}

/// Textarea component
pub struct Textarea;

impl Textarea {
    pub fn build(tree: &mut LayoutTree, rows: usize, placeholder: &str) -> NodeId {
        let height = (rows as f32 * 24.0) + 16.0;

        let textarea_style = StyleBuilder::new()
            .width_percent(1.0)
            .height(height)
            .padding(12.0)
            .build();

        let textarea_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(8.0);

        tree.new_visual_node(textarea_style, textarea_visual)
    }
}

/// Select/dropdown component
pub struct Select;

impl Select {
    pub fn build(tree: &mut LayoutTree, props: SelectProps) -> NodeId {
        let height = match props.size {
            InputSize::Small => 32.0,
            InputSize::Medium => 40.0,
            InputSize::Large => 48.0,
        };

        let select_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(height)
            .padding_xy(0.0, 12.0)
            .build();

        let select_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#111827"))
            .with_border(hex_to_rgba("#374151"), 1.0)
            .with_radius(8.0);

        // Arrow icon
        let arrow_style = StyleBuilder::new().size(16.0, 16.0).build();
        let arrow_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#6B7280"))
            .with_radius(2.0);
        let arrow = tree.new_visual_node(arrow_style, arrow_visual);

        tree.new_visual_node_with_children(select_style, select_visual, &[arrow])
    }
}

/// Select properties
#[derive(Debug, Clone)]
pub struct SelectProps {
    pub value: Option<String>,
    pub options: Vec<SelectOption>,
    pub placeholder: String,
    pub size: InputSize,
    pub disabled: bool,
}

/// Select option
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            value: None,
            options: Vec::new(),
            placeholder: "Select...".to_string(),
            size: InputSize::Medium,
            disabled: false,
        }
    }
}

/// Checkbox component
pub struct Checkbox;

impl Checkbox {
    pub fn build(tree: &mut LayoutTree, checked: bool, label: Option<&str>, disabled: bool) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(8.0)
            .build();

        // Checkbox box
        let box_style = StyleBuilder::new().size(20.0, 20.0).build();
        let box_visual = if checked {
            NodeVisual::default()
                .with_background(hex_to_rgba("#3B82F6"))
                .with_radius(4.0)
        } else {
            NodeVisual::default()
                .with_background(hex_to_rgba("#111827"))
                .with_border(hex_to_rgba("#374151"), 1.0)
                .with_radius(4.0)
        };

        let checkbox = tree.new_visual_node(box_style, box_visual);

        if label.is_some() {
            let label_style = StyleBuilder::new().build();
            let label_node = tree.new_node(label_style);
            tree.new_node_with_children(container_style, &[checkbox, label_node])
        } else {
            tree.new_node_with_children(container_style, &[checkbox])
        }
    }
}

/// Switch/toggle component
pub struct Switch;

impl Switch {
    pub fn build(tree: &mut LayoutTree, on: bool, disabled: bool) -> NodeId {
        let track_style = StyleBuilder::new()
            .size(44.0, 24.0)
            .flex_row()
            .align_center()
            .padding_xy(0.0, 2.0)
            .build();

        let track_visual = NodeVisual::default()
            .with_background(if on { hex_to_rgba("#3B82F6") } else { hex_to_rgba("#374151") })
            .with_radius(12.0);

        // Thumb
        let thumb_style = StyleBuilder::new()
            .size(20.0, 20.0)
            .build();

        let thumb_visual = NodeVisual::default()
            .with_background(hex_to_rgba("#FFFFFF"))
            .with_radius(10.0);

        let thumb = tree.new_visual_node(thumb_style, thumb_visual);

        tree.new_visual_node_with_children(track_style, track_visual, &[thumb])
    }
}

/// Form group for organizing form fields
pub struct FormGroup;

impl FormGroup {
    pub fn build(tree: &mut LayoutTree, children: &[NodeId], inline: bool) -> NodeId {
        let style = if inline {
            StyleBuilder::new()
                .flex_row()
                .align_end()
                .gap(16.0)
                .build()
        } else {
            StyleBuilder::new()
                .flex_column()
                .gap(16.0)
                .build()
        };

        tree.new_node_with_children(style, children)
    }
}

fn hex_to_rgba(hex: &str) -> [f32; 4] {
    Color::from_hex(hex).map(|c| c.to_array()).unwrap_or([1.0, 1.0, 1.0, 1.0])
}
