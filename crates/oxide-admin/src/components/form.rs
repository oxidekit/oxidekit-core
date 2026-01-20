//! Form components (input, select, checkbox, switch, textarea)
//!
//! Production-ready form controls with consistent styling, states,
//! and accessibility support.

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius, sizes, typography,
    hex_to_rgba, input_colors, InteractionState,
};

// =============================================================================
// INPUT SIZE
// =============================================================================

/// Input size variants
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum InputSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl InputSize {
    /// Get height for this size
    pub fn height(&self) -> f32 {
        match self {
            Self::Small => sizes::INPUT_SM,
            Self::Medium => sizes::INPUT_MD,
            Self::Large => sizes::INPUT_LG,
        }
    }

    /// Get horizontal padding
    pub fn padding_x(&self) -> f32 {
        match self {
            Self::Small => spacing::SM + spacing::XS, // 12
            Self::Medium => spacing::SM + spacing::XS, // 12
            Self::Large => spacing::MD, // 16
        }
    }

    /// Get font size
    pub fn font_size(&self) -> f32 {
        match self {
            Self::Small => typography::SIZE_SM,
            Self::Medium => typography::SIZE_BASE,
            Self::Large => typography::SIZE_MD,
        }
    }

    /// Get icon size
    pub fn icon_size(&self) -> f32 {
        match self {
            Self::Small => sizes::ICON_SM,
            Self::Medium => sizes::ICON_MD,
            Self::Large => sizes::ICON_LG,
        }
    }
}

// =============================================================================
// INPUT TYPE
// =============================================================================

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

// =============================================================================
// INPUT COMPONENT
// =============================================================================

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
    pub state: InteractionState,
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
            state: InteractionState::Default,
        }
    }
}

impl InputProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn error(mut self, error: impl Into<String>) -> Self {
        self.error = Some(error.into());
        self
    }

    pub fn helper(mut self, helper: impl Into<String>) -> Self {
        self.helper = Some(helper.into());
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn input_type(mut self, input_type: InputType) -> Self {
        self.input_type = input_type;
        self
    }

    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    pub fn suffix(mut self, suffix: impl Into<String>) -> Self {
        self.suffix = Some(suffix.into());
        self
    }

    pub fn state(mut self, state: InteractionState) -> Self {
        self.state = state;
        self
    }

    fn effective_state(&self) -> InteractionState {
        if self.disabled {
            InteractionState::Disabled
        } else {
            self.state
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
            .gap(spacing::GAP_SM)
            .build();

        let mut children = Vec::new();

        // Label
        if let Some(_label) = &props.label {
            let label_node = Self::build_label(tree, &props);
            children.push(label_node);
        }

        // Input wrapper
        let input_node = Self::build_input_field(tree, &props);
        children.push(input_node);

        // Error or helper text
        if let Some(_error) = &props.error {
            let error_node = Self::build_error_text(tree);
            children.push(error_node);
        } else if let Some(_helper) = &props.helper {
            let helper_node = Self::build_helper_text(tree);
            children.push(helper_node);
        }

        tree.new_node_with_children(container_style, &children)
    }

    fn build_label(tree: &mut LayoutTree, props: &InputProps) -> NodeId {
        let label_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::GAP_XS)
            .build();

        let mut children = Vec::new();

        // Label text placeholder
        let text_style = StyleBuilder::new().build();
        let text = tree.new_node(text_style);
        children.push(text);

        // Required indicator
        if props.required {
            let required_style = StyleBuilder::new()
                .size(spacing::SM, spacing::SM)
                .build();
            let required_visual = NodeVisual::default()
                .with_background(hex_to_rgba(colors::DANGER))
                .with_radius(spacing::XS);
            let required = tree.new_visual_node(required_style, required_visual);
            children.push(required);
        }

        tree.new_node_with_children(label_style, &children)
    }

    fn build_input_field(tree: &mut LayoutTree, props: &InputProps) -> NodeId {
        let height = props.size.height();
        let padding_x = props.size.padding_x();

        let input_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(height)
            .padding_xy(0.0, padding_x)
            .gap(spacing::SM)
            .build();

        let has_error = props.error.is_some();
        let (bg_color, border_color, _text_color) = input_colors(has_error, props.effective_state());

        let mut input_visual = NodeVisual::default()
            .with_background(bg_color)
            .with_border(border_color, sizes::BORDER_DEFAULT)
            .with_radius(radius::INPUT);

        // Focus ring
        if props.effective_state() == InteractionState::Focus {
            input_visual = input_visual.with_border(
                hex_to_rgba(colors::BORDER_FOCUS),
                sizes::FOCUS_RING_WIDTH,
            );
        }

        let mut children = Vec::new();

        // Prefix
        if props.prefix.is_some() {
            let prefix_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .build();
            let prefix = tree.new_node(prefix_style);
            children.push(prefix);
        }

        // Input area (flex grow)
        let input_area_style = StyleBuilder::new()
            .flex_grow(1.0)
            .height_percent(1.0)
            .build();
        let input_area = tree.new_node(input_area_style);
        children.push(input_area);

        // Suffix
        if props.suffix.is_some() {
            let suffix_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .build();
            let suffix = tree.new_node(suffix_style);
            children.push(suffix);
        }

        tree.new_visual_node_with_children(input_style, input_visual, &children)
    }

    fn build_error_text(tree: &mut LayoutTree) -> NodeId {
        let error_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::GAP_XS)
            .build();

        // Error icon
        let icon_style = StyleBuilder::new()
            .size(sizes::ICON_SM, sizes::ICON_SM)
            .build();
        let icon_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::DANGER))
            .with_radius(radius::SM);
        let icon = tree.new_visual_node(icon_style, icon_visual);

        // Error text placeholder
        let text_style = StyleBuilder::new().build();
        let text = tree.new_node(text_style);

        tree.new_node_with_children(error_style, &[icon, text])
    }

    fn build_helper_text(tree: &mut LayoutTree) -> NodeId {
        let helper_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .build();

        tree.new_node(helper_style)
    }
}

// =============================================================================
// TEXTAREA COMPONENT
// =============================================================================

/// Textarea component
pub struct Textarea;

impl Textarea {
    pub fn build(tree: &mut LayoutTree, rows: usize, _placeholder: &str) -> NodeId {
        Self::build_with_state(tree, rows, _placeholder, false, InteractionState::Default)
    }

    pub fn build_with_state(
        tree: &mut LayoutTree,
        rows: usize,
        _placeholder: &str,
        has_error: bool,
        state: InteractionState,
    ) -> NodeId {
        let height = (rows as f32 * typography::SIZE_MD * typography::LINE_HEIGHT_NORMAL) + spacing::MD;

        let textarea_style = StyleBuilder::new()
            .width_percent(1.0)
            .height(height)
            .padding(spacing::SM + spacing::XS)
            .build();

        let (bg_color, border_color, _text_color) = input_colors(has_error, state);

        let mut textarea_visual = NodeVisual::default()
            .with_background(bg_color)
            .with_border(border_color, sizes::BORDER_DEFAULT)
            .with_radius(radius::INPUT);

        if state == InteractionState::Focus {
            textarea_visual = textarea_visual.with_border(
                hex_to_rgba(colors::BORDER_FOCUS),
                sizes::FOCUS_RING_WIDTH,
            );
        }

        tree.new_visual_node(textarea_style, textarea_visual)
    }
}

// =============================================================================
// SELECT COMPONENT
// =============================================================================

/// Select option
#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub disabled: bool,
}

impl SelectOption {
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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
    pub error: Option<String>,
    pub state: InteractionState,
}

impl Default for SelectProps {
    fn default() -> Self {
        Self {
            value: None,
            options: Vec::new(),
            placeholder: "Select...".to_string(),
            size: InputSize::Medium,
            disabled: false,
            error: None,
            state: InteractionState::Default,
        }
    }
}

impl SelectProps {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: Vec<SelectOption>) -> Self {
        self.options = options;
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn size(mut self, size: InputSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn state(mut self, state: InteractionState) -> Self {
        self.state = state;
        self
    }

    fn effective_state(&self) -> InteractionState {
        if self.disabled {
            InteractionState::Disabled
        } else {
            self.state
        }
    }
}

/// Select/dropdown component
pub struct Select;

impl Select {
    pub fn build(tree: &mut LayoutTree, props: SelectProps) -> NodeId {
        let height = props.size.height();
        let padding_x = props.size.padding_x();

        let select_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .height(height)
            .padding_xy(0.0, padding_x)
            .build();

        let has_error = props.error.is_some();
        let (bg_color, border_color, _text_color) = input_colors(has_error, props.effective_state());

        let mut select_visual = NodeVisual::default()
            .with_background(bg_color)
            .with_border(border_color, sizes::BORDER_DEFAULT)
            .with_radius(radius::INPUT);

        if props.effective_state() == InteractionState::Focus {
            select_visual = select_visual.with_border(
                hex_to_rgba(colors::BORDER_FOCUS),
                sizes::FOCUS_RING_WIDTH,
            );
        }

        // Selected value placeholder
        let value_style = StyleBuilder::new()
            .flex_grow(1.0)
            .build();
        let value = tree.new_node(value_style);

        // Arrow icon
        let arrow_size = props.size.icon_size();
        let arrow_style = StyleBuilder::new()
            .size(arrow_size, arrow_size)
            .build();
        let arrow_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::TEXT_SECONDARY))
            .with_radius(radius::SM);
        let arrow = tree.new_visual_node(arrow_style, arrow_visual);

        tree.new_visual_node_with_children(select_style, select_visual, &[value, arrow])
    }
}

// =============================================================================
// CHECKBOX COMPONENT
// =============================================================================

/// Checkbox size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CheckboxSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl CheckboxSize {
    fn size(&self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
        }
    }

    fn check_size(&self) -> f32 {
        match self {
            Self::Small => 10.0,
            Self::Medium => 12.0,
            Self::Large => 14.0,
        }
    }
}

/// Checkbox component
pub struct Checkbox;

impl Checkbox {
    pub fn build(
        tree: &mut LayoutTree,
        checked: bool,
        label: Option<&str>,
        disabled: bool,
    ) -> NodeId {
        Self::build_with_size(tree, checked, label, disabled, CheckboxSize::Medium, InteractionState::Default)
    }

    pub fn build_with_size(
        tree: &mut LayoutTree,
        checked: bool,
        label: Option<&str>,
        disabled: bool,
        size: CheckboxSize,
        state: InteractionState,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::SM)
            .build();

        // Checkbox box
        let box_size = size.size();
        let box_style = StyleBuilder::new()
            .size(box_size, box_size)
            .flex_row()
            .center()
            .build();

        let effective_state = if disabled { InteractionState::Disabled } else { state };

        let box_visual = if checked {
            NodeVisual::default()
                .with_background(if disabled {
                    hex_to_rgba(colors::DISABLED_BG)
                } else {
                    hex_to_rgba(colors::PRIMARY)
                })
                .with_radius(radius::SM)
        } else {
            let border_color = match effective_state {
                InteractionState::Hover => colors::BORDER_STRONG,
                InteractionState::Focus => colors::BORDER_FOCUS,
                InteractionState::Disabled => colors::DISABLED_BG,
                _ => colors::BORDER,
            };
            NodeVisual::default()
                .with_background(hex_to_rgba(colors::SURFACE_ELEVATED))
                .with_border(hex_to_rgba(border_color), sizes::BORDER_DEFAULT)
                .with_radius(radius::SM)
        };

        let mut checkbox_children = Vec::new();

        // Check mark
        if checked {
            let check_size = size.check_size();
            let check_style = StyleBuilder::new()
                .size(check_size, check_size)
                .build();
            let check_visual = NodeVisual::default()
                .with_background(hex_to_rgba(colors::PRIMARY_CONTRAST))
                .with_radius(radius::SM);
            let check = tree.new_visual_node(check_style, check_visual);
            checkbox_children.push(check);
        }

        let checkbox = if checkbox_children.is_empty() {
            tree.new_visual_node(box_style, box_visual)
        } else {
            tree.new_visual_node_with_children(box_style, box_visual, &checkbox_children)
        };

        if label.is_some() {
            let label_style = StyleBuilder::new().build();
            let label_node = tree.new_node(label_style);
            tree.new_node_with_children(container_style, &[checkbox, label_node])
        } else {
            tree.new_node_with_children(container_style, &[checkbox])
        }
    }

    /// Build indeterminate checkbox
    pub fn build_indeterminate(
        tree: &mut LayoutTree,
        label: Option<&str>,
        disabled: bool,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::SM)
            .build();

        let box_style = StyleBuilder::new()
            .size(20.0, 20.0)
            .flex_row()
            .center()
            .build();

        let box_visual = NodeVisual::default()
            .with_background(if disabled {
                hex_to_rgba(colors::DISABLED_BG)
            } else {
                hex_to_rgba(colors::PRIMARY)
            })
            .with_radius(radius::SM);

        // Indeterminate dash
        let dash_style = StyleBuilder::new()
            .size(10.0, 2.0)
            .build();
        let dash_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::PRIMARY_CONTRAST))
            .with_radius(1.0);
        let dash = tree.new_visual_node(dash_style, dash_visual);

        let checkbox = tree.new_visual_node_with_children(box_style, box_visual, &[dash]);

        if label.is_some() {
            let label_style = StyleBuilder::new().build();
            let label_node = tree.new_node(label_style);
            tree.new_node_with_children(container_style, &[checkbox, label_node])
        } else {
            tree.new_node_with_children(container_style, &[checkbox])
        }
    }
}

// =============================================================================
// SWITCH COMPONENT
// =============================================================================

/// Switch size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SwitchSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl SwitchSize {
    fn track_width(&self) -> f32 {
        match self {
            Self::Small => 36.0,
            Self::Medium => 44.0,
            Self::Large => 52.0,
        }
    }

    fn track_height(&self) -> f32 {
        match self {
            Self::Small => 20.0,
            Self::Medium => 24.0,
            Self::Large => 28.0,
        }
    }

    fn thumb_size(&self) -> f32 {
        match self {
            Self::Small => 16.0,
            Self::Medium => 20.0,
            Self::Large => 24.0,
        }
    }
}

/// Switch/toggle component
pub struct Switch;

impl Switch {
    pub fn build(tree: &mut LayoutTree, on: bool, disabled: bool) -> NodeId {
        Self::build_with_size(tree, on, disabled, SwitchSize::Medium, InteractionState::Default)
    }

    pub fn build_with_size(
        tree: &mut LayoutTree,
        on: bool,
        disabled: bool,
        size: SwitchSize,
        state: InteractionState,
    ) -> NodeId {
        let track_width = size.track_width();
        let track_height = size.track_height();
        let thumb_size = size.thumb_size();
        let padding = (track_height - thumb_size) / 2.0;

        let track_style = StyleBuilder::new()
            .size(track_width, track_height)
            .flex_row()
            .align_center()
            .padding_xy(0.0, padding)
            .build();

        let effective_state = if disabled { InteractionState::Disabled } else { state };

        let track_color = if on {
            if disabled {
                colors::DISABLED_BG
            } else {
                match effective_state {
                    InteractionState::Hover => colors::PRIMARY_LIGHT,
                    InteractionState::Active => colors::PRIMARY_DARK,
                    _ => colors::PRIMARY,
                }
            }
        } else {
            if disabled {
                colors::DISABLED_BG
            } else {
                match effective_state {
                    InteractionState::Hover => colors::BORDER_STRONG,
                    _ => colors::SURFACE_VARIANT,
                }
            }
        };

        let track_visual = NodeVisual::default()
            .with_background(hex_to_rgba(track_color))
            .with_radius(track_height / 2.0);

        // Thumb
        let thumb_style = StyleBuilder::new()
            .size(thumb_size, thumb_size)
            .build();

        let thumb_visual = NodeVisual::default()
            .with_background(hex_to_rgba(if disabled { colors::TEXT_DISABLED } else { colors::PRIMARY_CONTRAST }))
            .with_radius(thumb_size / 2.0);

        let thumb = tree.new_visual_node(thumb_style, thumb_visual);

        // Spacer for positioning (when on, thumb goes to right)
        if on {
            let spacer_style = StyleBuilder::new()
                .flex_grow(1.0)
                .build();
            let spacer = tree.new_node(spacer_style);
            tree.new_visual_node_with_children(track_style, track_visual, &[spacer, thumb])
        } else {
            tree.new_visual_node_with_children(track_style, track_visual, &[thumb])
        }
    }

    /// Build switch with label
    pub fn build_with_label(
        tree: &mut LayoutTree,
        on: bool,
        _label: &str,
        disabled: bool,
    ) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .gap(spacing::SM + spacing::XS)
            .build();

        let switch = Self::build(tree, on, disabled);

        let label_style = StyleBuilder::new().build();
        let label_node = tree.new_node(label_style);

        tree.new_node_with_children(container_style, &[switch, label_node])
    }
}

// =============================================================================
// FORM GROUP
// =============================================================================

/// Form group for organizing form fields
pub struct FormGroup;

impl FormGroup {
    pub fn build(tree: &mut LayoutTree, children: &[NodeId], inline: bool) -> NodeId {
        let style = if inline {
            StyleBuilder::new()
                .flex_row()
                .align_end()
                .gap(spacing::MD)
                .build()
        } else {
            StyleBuilder::new()
                .flex_column()
                .gap(spacing::MD)
                .build()
        };

        tree.new_node_with_children(style, children)
    }

    /// Build form section with title
    pub fn build_section(
        tree: &mut LayoutTree,
        _title: &str,
        _description: Option<&str>,
        children: &[NodeId],
    ) -> NodeId {
        let section_style = StyleBuilder::new()
            .flex_column()
            .gap(spacing::MD)
            .build();

        let header_style = StyleBuilder::new()
            .flex_column()
            .gap(spacing::XS)
            .build();

        // Title
        let title_style = StyleBuilder::new().build();
        let title = tree.new_node(title_style);

        // Description
        let desc_style = StyleBuilder::new().build();
        let desc = tree.new_node(desc_style);

        let header = tree.new_node_with_children(header_style, &[title, desc]);

        // Fields container
        let fields_style = StyleBuilder::new()
            .flex_column()
            .gap(spacing::MD)
            .build();
        let fields = tree.new_node_with_children(fields_style, children);

        tree.new_node_with_children(section_style, &[header, fields])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_props() {
        let props = InputProps::new()
            .label("Username")
            .placeholder("Enter username")
            .required(true);

        assert_eq!(props.label, Some("Username".to_string()));
        assert!(props.required);
    }

    #[test]
    fn test_input_sizes() {
        assert_eq!(InputSize::Small.height(), sizes::INPUT_SM);
        assert_eq!(InputSize::Medium.height(), sizes::INPUT_MD);
        assert_eq!(InputSize::Large.height(), sizes::INPUT_LG);
    }

    #[test]
    fn test_input_build() {
        let mut tree = LayoutTree::new();
        let props = InputProps::new().label("Test");
        let _node = Input::build(&mut tree, props);
    }

    #[test]
    fn test_checkbox_build() {
        let mut tree = LayoutTree::new();
        let _node = Checkbox::build(&mut tree, true, Some("Accept terms"), false);
    }

    #[test]
    fn test_switch_build() {
        let mut tree = LayoutTree::new();
        let _node = Switch::build(&mut tree, true, false);
    }

    #[test]
    fn test_select_build() {
        let mut tree = LayoutTree::new();
        let props = SelectProps::new()
            .options(vec![
                SelectOption::new("1", "Option 1"),
                SelectOption::new("2", "Option 2"),
            ])
            .placeholder("Choose...");
        let _node = Select::build(&mut tree, props);
    }
}
