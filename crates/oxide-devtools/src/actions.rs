//! Context Action Menu System
//!
//! Provides a context menu system for rapid component editing.
//! Actions are constrained by component schemas to prevent invalid edits.

use crate::editor::{DevEditor, StyleValueChange};
use crate::tree::NodeHandle;
use oxide_components::{ComponentSpec, PropSpec, PropType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A context action that can be performed on a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextAction {
    /// Unique action ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Description
    pub description: String,
    /// Action category
    pub category: ActionCategory,
    /// Keyboard shortcut (if any)
    pub shortcut: Option<String>,
    /// Icon name (if any)
    pub icon: Option<String>,
    /// Whether action is enabled
    pub enabled: bool,
    /// Action payload
    pub payload: ActionPayload,
}

/// Category of context actions
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ActionCategory {
    /// Text editing actions
    Text,
    /// Style/visual actions
    Style,
    /// Layout actions
    Layout,
    /// Component structure actions
    Structure,
    /// State simulation actions
    State,
    /// Variant switching
    Variant,
    /// Navigation actions
    Navigation,
    /// Clipboard actions
    Clipboard,
}

/// Payload for an action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionPayload {
    /// Change a text property
    ChangeText {
        property: String,
        current_value: Option<String>,
    },
    /// Change a color
    ChangeColor {
        property: String,
        current_value: Option<String>,
        token_options: Vec<String>,
    },
    /// Change a numeric value
    ChangeNumber {
        property: String,
        current_value: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
        step: Option<f64>,
        unit: Option<String>,
    },
    /// Change an enum value
    ChangeEnum {
        property: String,
        current_value: Option<String>,
        options: Vec<String>,
    },
    /// Change a token reference
    ChangeToken {
        property: String,
        current_token: Option<String>,
        category: String,
        available_tokens: Vec<String>,
    },
    /// Change component variant
    ChangeVariant {
        current_variant: Option<String>,
        available_variants: Vec<String>,
    },
    /// Simulate a state
    SimulateState {
        state: String,
    },
    /// Wrap component
    WrapWith {
        wrapper_options: Vec<String>,
    },
    /// Add child component
    AddChild {
        child_options: Vec<String>,
        slot: Option<String>,
    },
    /// Copy as code
    CopyAsCode {
        format: CodeFormat,
    },
    /// Navigate to source
    GoToSource,
    /// Custom action
    Custom {
        data: serde_json::Value,
    },
}

/// Code format for export
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeFormat {
    /// OUI source format
    Oui,
    /// Rust code
    Rust,
    /// JSON
    Json,
}

/// Builder for context actions based on component spec
#[derive(Debug)]
pub struct ActionBuilder {
    spec: Option<ComponentSpec>,
    actions: Vec<ContextAction>,
    color_tokens: Vec<String>,
    spacing_tokens: Vec<String>,
}

impl ActionBuilder {
    /// Create a new action builder
    pub fn new() -> Self {
        Self {
            spec: None,
            actions: Vec::new(),
            color_tokens: vec![
                "color.primary".to_string(),
                "color.secondary".to_string(),
                "color.success".to_string(),
                "color.warning".to_string(),
                "color.danger".to_string(),
                "color.info".to_string(),
            ],
            spacing_tokens: vec![
                "spacing.xs".to_string(),
                "spacing.sm".to_string(),
                "spacing.md".to_string(),
                "spacing.lg".to_string(),
                "spacing.xl".to_string(),
            ],
        }
    }

    /// Set the component spec to build actions for
    pub fn for_spec(mut self, spec: ComponentSpec) -> Self {
        self.spec = Some(spec);
        self
    }

    /// Set available color tokens
    pub fn with_color_tokens(mut self, tokens: Vec<String>) -> Self {
        self.color_tokens = tokens;
        self
    }

    /// Set available spacing tokens
    pub fn with_spacing_tokens(mut self, tokens: Vec<String>) -> Self {
        self.spacing_tokens = tokens;
        self
    }

    /// Build actions from the spec
    pub fn build(mut self) -> Vec<ContextAction> {
        self.actions.clear();

        // Add standard actions
        self.add_standard_actions();

        // Add spec-based actions
        if let Some(spec) = self.spec.clone() {
            self.add_prop_actions(&spec);
            self.add_variant_actions(&spec);
            self.add_slot_actions(&spec);
        }

        self.actions.clone()
    }

    /// Add standard actions available for all components
    fn add_standard_actions(&mut self) {
        // Navigation
        self.actions.push(ContextAction {
            id: "go_to_source".to_string(),
            name: "Go to Source".to_string(),
            description: "Open source file at component definition".to_string(),
            category: ActionCategory::Navigation,
            shortcut: Some("Cmd+Click".to_string()),
            icon: Some("file-code".to_string()),
            enabled: true,
            payload: ActionPayload::GoToSource,
        });

        // Clipboard actions
        self.actions.push(ContextAction {
            id: "copy_as_oui".to_string(),
            name: "Copy as OUI".to_string(),
            description: "Copy component code to clipboard".to_string(),
            category: ActionCategory::Clipboard,
            shortcut: Some("Cmd+C".to_string()),
            icon: Some("clipboard".to_string()),
            enabled: true,
            payload: ActionPayload::CopyAsCode {
                format: CodeFormat::Oui,
            },
        });

        self.actions.push(ContextAction {
            id: "copy_as_json".to_string(),
            name: "Copy as JSON".to_string(),
            description: "Copy component as JSON".to_string(),
            category: ActionCategory::Clipboard,
            shortcut: None,
            icon: Some("braces".to_string()),
            enabled: true,
            payload: ActionPayload::CopyAsCode {
                format: CodeFormat::Json,
            },
        });

        // State simulation
        for state in ["hover", "focus", "active", "disabled"] {
            self.actions.push(ContextAction {
                id: format!("simulate_{}", state),
                name: format!("Simulate {}", capitalize(state)),
                description: format!("Simulate {} state for styling", state),
                category: ActionCategory::State,
                shortcut: None,
                icon: Some("play".to_string()),
                enabled: true,
                payload: ActionPayload::SimulateState {
                    state: state.to_string(),
                },
            });
        }

        // Structure actions
        self.actions.push(ContextAction {
            id: "wrap_container".to_string(),
            name: "Wrap with Container".to_string(),
            description: "Wrap component with a container".to_string(),
            category: ActionCategory::Structure,
            shortcut: Some("Cmd+W".to_string()),
            icon: Some("box".to_string()),
            enabled: true,
            payload: ActionPayload::WrapWith {
                wrapper_options: vec![
                    "Container".to_string(),
                    "Card".to_string(),
                    "Row".to_string(),
                    "Column".to_string(),
                    "Stack".to_string(),
                ],
            },
        });
    }

    /// Add actions based on component props
    fn add_prop_actions(&mut self, spec: &ComponentSpec) {
        for prop in &spec.props {
            let action = self.action_for_prop(prop);
            if let Some(a) = action {
                self.actions.push(a);
            }
        }
    }

    /// Create action for a specific prop
    fn action_for_prop(&self, prop: &PropSpec) -> Option<ContextAction> {
        let (category, payload) = match &prop.prop_type {
            PropType::String => (
                ActionCategory::Text,
                ActionPayload::ChangeText {
                    property: prop.name.clone(),
                    current_value: None,
                },
            ),
            PropType::Color => (
                ActionCategory::Style,
                ActionPayload::ChangeColor {
                    property: prop.name.clone(),
                    current_value: None,
                    token_options: self.color_tokens.clone(),
                },
            ),
            PropType::Number => {
                let (min, max, step) = if let Some(constraints) = &prop.constraints {
                    (constraints.min, constraints.max, None)
                } else {
                    (None, None, None)
                };
                (
                    ActionCategory::Style,
                    ActionPayload::ChangeNumber {
                        property: prop.name.clone(),
                        current_value: None,
                        min,
                        max,
                        step,
                        unit: None,
                    },
                )
            }
            PropType::Spacing => (
                ActionCategory::Layout,
                ActionPayload::ChangeToken {
                    property: prop.name.clone(),
                    current_token: None,
                    category: "spacing".to_string(),
                    available_tokens: self.spacing_tokens.clone(),
                },
            ),
            PropType::Enum { values } => (
                ActionCategory::Style,
                ActionPayload::ChangeEnum {
                    property: prop.name.clone(),
                    current_value: None,
                    options: values.clone(),
                },
            ),
            PropType::Token { category } => (
                ActionCategory::Style,
                ActionPayload::ChangeToken {
                    property: prop.name.clone(),
                    current_token: None,
                    category: category.clone(),
                    available_tokens: Vec::new(), // Would be populated from theme
                },
            ),
            _ => return None,
        };

        Some(ContextAction {
            id: format!("change_{}", prop.name),
            name: format!("Change {}", humanize(&prop.name)),
            description: prop.description.clone(),
            category,
            shortcut: None,
            icon: None,
            enabled: true,
            payload,
        })
    }

    /// Add variant actions
    fn add_variant_actions(&mut self, spec: &ComponentSpec) {
        if !spec.variants.is_empty() {
            let variants: Vec<String> = spec.variants.iter().map(|v| v.name.clone()).collect();
            self.actions.push(ContextAction {
                id: "change_variant".to_string(),
                name: "Change Variant".to_string(),
                description: "Switch component variant".to_string(),
                category: ActionCategory::Variant,
                shortcut: Some("V".to_string()),
                icon: Some("palette".to_string()),
                enabled: true,
                payload: ActionPayload::ChangeVariant {
                    current_variant: None,
                    available_variants: variants,
                },
            });
        }
    }

    /// Add slot actions
    fn add_slot_actions(&mut self, spec: &ComponentSpec) {
        for slot in &spec.slots {
            let child_options = if slot.allowed_children.is_empty() {
                vec!["Any".to_string()]
            } else {
                slot.allowed_children.clone()
            };

            self.actions.push(ContextAction {
                id: format!("add_to_{}", slot.name),
                name: format!("Add to {}", humanize(&slot.name)),
                description: slot.description.clone(),
                category: ActionCategory::Structure,
                shortcut: None,
                icon: Some("plus".to_string()),
                enabled: true,
                payload: ActionPayload::AddChild {
                    child_options,
                    slot: Some(slot.name.clone()),
                },
            });
        }
    }
}

impl Default for ActionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Context menu manager
#[derive(Debug)]
pub struct ContextMenu {
    /// Whether menu is visible
    pub visible: bool,
    /// Position (x, y)
    pub position: (f32, f32),
    /// Target component handle
    pub target: Option<NodeHandle>,
    /// Available actions
    pub actions: Vec<ContextAction>,
    /// Search filter
    pub filter: String,
    /// Selected action index
    pub selected_index: usize,
    /// Actions grouped by category
    pub grouped: HashMap<ActionCategory, Vec<ContextAction>>,
}

impl Default for ContextMenu {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextMenu {
    /// Create a new context menu
    pub fn new() -> Self {
        Self {
            visible: false,
            position: (0.0, 0.0),
            target: None,
            actions: Vec::new(),
            filter: String::new(),
            selected_index: 0,
            grouped: HashMap::new(),
        }
    }

    /// Show menu at position for target
    pub fn show(&mut self, x: f32, y: f32, target: NodeHandle, actions: Vec<ContextAction>) {
        self.visible = true;
        self.position = (x, y);
        self.target = Some(target);
        self.actions = actions.clone();
        self.filter.clear();
        self.selected_index = 0;

        // Group by category
        self.grouped.clear();
        for action in actions {
            self.grouped
                .entry(action.category)
                .or_default()
                .push(action);
        }
    }

    /// Hide menu
    pub fn hide(&mut self) {
        self.visible = false;
        self.target = None;
        self.actions.clear();
        self.grouped.clear();
    }

    /// Update filter
    pub fn set_filter(&mut self, filter: &str) {
        self.filter = filter.to_lowercase();
        self.selected_index = 0;
    }

    /// Get filtered actions
    pub fn filtered_actions(&self) -> Vec<&ContextAction> {
        if self.filter.is_empty() {
            return self.actions.iter().collect();
        }

        self.actions
            .iter()
            .filter(|a| {
                a.name.to_lowercase().contains(&self.filter)
                    || a.description.to_lowercase().contains(&self.filter)
            })
            .collect()
    }

    /// Select next action
    pub fn select_next(&mut self) {
        let count = self.filtered_actions().len();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    /// Select previous action
    pub fn select_prev(&mut self) {
        let count = self.filtered_actions().len();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    /// Get selected action
    pub fn selected_action(&self) -> Option<&ContextAction> {
        self.filtered_actions().get(self.selected_index).copied()
    }

    /// Execute selected action
    pub fn execute_selected(&self, editor: &mut DevEditor) -> Option<ActionResult> {
        let action = self.selected_action()?;
        execute_action(action, self.target?, editor)
    }
}

/// Result of executing an action
#[derive(Debug, Clone)]
pub enum ActionResult {
    /// Action completed successfully
    Success(String),
    /// Action requires user input
    NeedsInput(InputRequest),
    /// Action was cancelled
    Cancelled,
    /// Action failed
    Error(String),
}

/// Request for user input
#[derive(Debug, Clone)]
pub struct InputRequest {
    /// Input type
    pub input_type: InputType,
    /// Current value
    pub current_value: Option<String>,
    /// Validation constraints
    pub constraints: InputConstraints,
    /// Callback ID for when input is provided
    pub callback_id: String,
}

/// Type of input needed
#[derive(Debug, Clone)]
pub enum InputType {
    /// Text input
    Text { placeholder: String },
    /// Color picker
    Color { tokens: Vec<String> },
    /// Number slider
    Number { min: f64, max: f64, step: f64 },
    /// Select from options
    Select { options: Vec<String> },
}

/// Constraints for input validation
#[derive(Debug, Clone, Default)]
pub struct InputConstraints {
    pub required: bool,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
}

/// Execute a context action
pub fn execute_action(
    action: &ContextAction,
    target: NodeHandle,
    editor: &mut DevEditor,
) -> Option<ActionResult> {
    match &action.payload {
        ActionPayload::ChangeColor { property, token_options, .. } => {
            Some(ActionResult::NeedsInput(InputRequest {
                input_type: InputType::Color {
                    tokens: token_options.clone(),
                },
                current_value: None,
                constraints: InputConstraints::default(),
                callback_id: format!("color_{}_{}", target, property),
            }))
        }

        ActionPayload::ChangeText { property, current_value } => {
            Some(ActionResult::NeedsInput(InputRequest {
                input_type: InputType::Text {
                    placeholder: format!("Enter {}", property),
                },
                current_value: current_value.clone(),
                constraints: InputConstraints::default(),
                callback_id: format!("text_{}_{}", target, property),
            }))
        }

        ActionPayload::ChangeNumber { property, min, max, step, .. } => {
            Some(ActionResult::NeedsInput(InputRequest {
                input_type: InputType::Number {
                    min: min.unwrap_or(0.0),
                    max: max.unwrap_or(1000.0),
                    step: step.unwrap_or(1.0),
                },
                current_value: None,
                constraints: InputConstraints::default(),
                callback_id: format!("number_{}_{}", target, property),
            }))
        }

        ActionPayload::ChangeEnum { property, options, .. } => {
            Some(ActionResult::NeedsInput(InputRequest {
                input_type: InputType::Select {
                    options: options.clone(),
                },
                current_value: None,
                constraints: InputConstraints::default(),
                callback_id: format!("enum_{}_{}", target, property),
            }))
        }

        ActionPayload::ChangeVariant { available_variants, .. } => {
            Some(ActionResult::NeedsInput(InputRequest {
                input_type: InputType::Select {
                    options: available_variants.clone(),
                },
                current_value: None,
                constraints: InputConstraints::default(),
                callback_id: format!("variant_{}", target),
            }))
        }

        ActionPayload::SimulateState { state } => {
            let mut component_state = crate::inspector::ComponentState::default();
            match state.as_str() {
                "hover" => component_state.hovered = true,
                "focus" => component_state.focused = true,
                "active" => component_state.active = true,
                "disabled" => component_state.disabled = true,
                _ => {}
            }
            editor.simulate_state(target, component_state);
            Some(ActionResult::Success(format!("Simulating {} state", state)))
        }

        ActionPayload::GoToSource => {
            // Would trigger source navigation
            Some(ActionResult::Success("Opening source...".to_string()))
        }

        ActionPayload::CopyAsCode { format } => {
            // Would generate and copy code
            Some(ActionResult::Success(format!("Copied as {:?}", format)))
        }

        _ => Some(ActionResult::Error("Action not implemented".to_string())),
    }
}

/// Apply input result to editor
pub fn apply_input(
    callback_id: &str,
    value: &str,
    editor: &mut DevEditor,
) -> Result<(), String> {
    // Parse callback ID to get action type, target, and property
    let parts: Vec<&str> = callback_id.split('_').collect();
    if parts.len() < 3 {
        return Err("Invalid callback ID".to_string());
    }

    let action_type = parts[0];
    let target_str = parts[1];
    let property = parts[2..].join("_");

    let target = uuid::Uuid::parse_str(target_str)
        .map(NodeHandle::from_uuid)
        .map_err(|e| e.to_string())?;

    let style_change = match action_type {
        "color" => StyleValueChange::color(value),
        "text" => StyleValueChange::String(value.to_string()),
        "number" => {
            let num: f64 = value.parse().map_err(|e: std::num::ParseFloatError| e.to_string())?;
            StyleValueChange::number(num)
        }
        "enum" | "variant" => StyleValueChange::Enum(value.to_string()),
        _ => return Err("Unknown action type".to_string()),
    };

    editor.apply_override(target, &property, style_change);
    Ok(())
}

// Helper functions
fn capitalize(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn humanize(s: &str) -> String {
    s.replace('_', " ")
        .split_whitespace()
        .map(capitalize)
        .collect::<Vec<_>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_builder() {
        let builder = ActionBuilder::new();
        let actions = builder.build();

        // Should have standard actions
        assert!(!actions.is_empty());
        assert!(actions.iter().any(|a| a.id == "go_to_source"));
    }

    #[test]
    fn test_context_menu() {
        let mut menu = ContextMenu::new();
        assert!(!menu.visible);

        let handle = NodeHandle::new();
        let actions = ActionBuilder::new().build();

        menu.show(100.0, 200.0, handle, actions);
        assert!(menu.visible);
        assert_eq!(menu.position, (100.0, 200.0));
        assert!(!menu.actions.is_empty());

        menu.hide();
        assert!(!menu.visible);
    }

    #[test]
    fn test_action_filter() {
        let mut menu = ContextMenu::new();
        let handle = NodeHandle::new();
        let actions = ActionBuilder::new().build();

        menu.show(0.0, 0.0, handle, actions);
        let initial_count = menu.filtered_actions().len();

        menu.set_filter("source");
        let filtered_count = menu.filtered_actions().len();
        assert!(filtered_count < initial_count);
        assert!(filtered_count > 0);
    }

    #[test]
    fn test_humanize() {
        assert_eq!(humanize("background_color"), "Background Color");
        assert_eq!(humanize("text"), "Text");
    }
}
