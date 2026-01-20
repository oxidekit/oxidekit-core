//! Component specification types
//!
//! Defines the structure for component metadata, props, events, and slots.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Full specification for a UI component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSpec {
    /// Stable component ID (e.g., "ui.Button")
    pub id: String,

    /// Human-readable name
    pub name: String,

    /// Component description
    pub description: String,

    /// Owning pack (e.g., "ui.core")
    pub pack: String,

    /// Semantic version
    pub version: String,

    /// Deprecation info (if deprecated)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deprecated: Option<DeprecationInfo>,

    /// Property specifications
    #[serde(default)]
    pub props: Vec<PropSpec>,

    /// Event specifications
    #[serde(default)]
    pub events: Vec<EventSpec>,

    /// Slot/children specifications
    #[serde(default)]
    pub slots: Vec<SlotSpec>,

    /// Supported style tokens
    #[serde(default)]
    pub style_tokens: Vec<String>,

    /// Supported style overrides
    #[serde(default)]
    pub style_overrides: Vec<StyleOverride>,

    /// Accessibility metadata
    #[serde(default)]
    pub accessibility: AccessibilitySpec,

    /// Component variants (e.g., primary, secondary for Button)
    #[serde(default)]
    pub variants: Vec<VariantSpec>,

    /// Example usages for documentation/AI
    #[serde(default)]
    pub examples: Vec<UsageExample>,
}

/// Property specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropSpec {
    /// Property name
    pub name: String,

    /// Property type
    pub prop_type: PropType,

    /// Description
    pub description: String,

    /// Whether the property is required
    #[serde(default)]
    pub required: bool,

    /// Default value (as JSON)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<serde_json::Value>,

    /// Constraints
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub constraints: Option<PropConstraints>,
}

/// Property type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PropType {
    /// String value
    String,

    /// Number (f64)
    Number,

    /// Integer (i64)
    Integer,

    /// Boolean
    Bool,

    /// Enumeration with allowed values
    Enum { values: Vec<String> },

    /// Color (hex, rgba, or token reference)
    Color,

    /// Spacing value (number or token reference)
    Spacing,

    /// Size value (number, percentage, or keyword)
    Size,

    /// Reference to another component
    ComponentRef { allowed: Vec<String> },

    /// Array of a type
    Array { item_type: Box<PropType> },

    /// Object with specific shape
    Object { properties: HashMap<String, PropType> },

    /// Callback/event handler
    Callback { payload: Option<String> },

    /// Token reference (e.g., color.primary)
    Token { category: String },
}

/// Property constraints
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PropConstraints {
    /// Minimum value (for numbers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min: Option<f64>,

    /// Maximum value (for numbers)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max: Option<f64>,

    /// Minimum length (for strings/arrays)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,

    /// Maximum length (for strings/arrays)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,

    /// Regex pattern (for strings)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

/// Event specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventSpec {
    /// Event name (e.g., "on_click")
    pub name: String,

    /// Description
    pub description: String,

    /// Payload schema (as JSON Schema)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

/// Slot specification for child components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSpec {
    /// Slot name (e.g., "default", "header", "footer")
    pub name: String,

    /// Description
    pub description: String,

    /// Allowed child component IDs (empty = any)
    #[serde(default)]
    pub allowed_children: Vec<String>,

    /// Minimum number of children
    #[serde(default)]
    pub min_children: usize,

    /// Maximum number of children (0 = unlimited)
    #[serde(default)]
    pub max_children: usize,

    /// Whether the slot is required
    #[serde(default)]
    pub required: bool,
}

/// Style override specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleOverride {
    /// Override name
    pub name: String,

    /// Description
    pub description: String,

    /// Type of the override
    pub override_type: PropType,
}

/// Accessibility specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AccessibilitySpec {
    /// ARIA role
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Keyboard behaviors
    #[serde(default)]
    pub keyboard: Vec<KeyboardBehavior>,

    /// Whether component is focusable
    #[serde(default)]
    pub focusable: bool,

    /// Required ARIA attributes
    #[serde(default)]
    pub required_aria: Vec<String>,
}

/// Keyboard behavior specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardBehavior {
    /// Key or key combination
    pub key: String,

    /// Action description
    pub action: String,
}

/// Component variant specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariantSpec {
    /// Variant name (e.g., "primary", "secondary")
    pub name: String,

    /// Description
    pub description: String,

    /// Default prop overrides for this variant
    #[serde(default)]
    pub defaults: HashMap<String, serde_json::Value>,
}

/// Deprecation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationInfo {
    /// Version when deprecated
    pub since: String,

    /// Replacement component/approach
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,

    /// Deprecation message
    pub message: String,
}

/// Usage example for documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageExample {
    /// Example title
    pub title: String,

    /// Description
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// OUI code example
    pub code: String,
}

impl ComponentSpec {
    /// Create a new component spec builder
    pub fn builder(id: impl Into<String>, pack: impl Into<String>) -> ComponentSpecBuilder {
        ComponentSpecBuilder::new(id, pack)
    }

    /// Get a prop spec by name
    pub fn get_prop(&self, name: &str) -> Option<&PropSpec> {
        self.props.iter().find(|p| p.name == name)
    }

    /// Get an event spec by name
    pub fn get_event(&self, name: &str) -> Option<&EventSpec> {
        self.events.iter().find(|e| e.name == name)
    }

    /// Get the default slot
    pub fn default_slot(&self) -> Option<&SlotSpec> {
        self.slots.iter().find(|s| s.name == "default")
    }

    /// Check if component is deprecated
    pub fn is_deprecated(&self) -> bool {
        self.deprecated.is_some()
    }
}

/// Builder for ComponentSpec
pub struct ComponentSpecBuilder {
    spec: ComponentSpec,
}

impl ComponentSpecBuilder {
    pub fn new(id: impl Into<String>, pack: impl Into<String>) -> Self {
        let id = id.into();
        let name = id.split('.').last().unwrap_or(&id).to_string();

        Self {
            spec: ComponentSpec {
                id,
                name,
                description: String::new(),
                pack: pack.into(),
                version: "0.1.0".to_string(),
                deprecated: None,
                props: Vec::new(),
                events: Vec::new(),
                slots: Vec::new(),
                style_tokens: Vec::new(),
                style_overrides: Vec::new(),
                accessibility: AccessibilitySpec::default(),
                variants: Vec::new(),
                examples: Vec::new(),
            },
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.spec.name = name.into();
        self
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.spec.description = desc.into();
        self
    }

    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.spec.version = version.into();
        self
    }

    pub fn prop(mut self, prop: PropSpec) -> Self {
        self.spec.props.push(prop);
        self
    }

    pub fn event(mut self, event: EventSpec) -> Self {
        self.spec.events.push(event);
        self
    }

    pub fn slot(mut self, slot: SlotSpec) -> Self {
        self.spec.slots.push(slot);
        self
    }

    pub fn style_token(mut self, token: impl Into<String>) -> Self {
        self.spec.style_tokens.push(token.into());
        self
    }

    pub fn variant(mut self, variant: VariantSpec) -> Self {
        self.spec.variants.push(variant);
        self
    }

    pub fn accessibility(mut self, a11y: AccessibilitySpec) -> Self {
        self.spec.accessibility = a11y;
        self
    }

    pub fn example(mut self, example: UsageExample) -> Self {
        self.spec.examples.push(example);
        self
    }

    pub fn build(self) -> ComponentSpec {
        self.spec
    }
}

impl PropSpec {
    /// Create a required string prop
    pub fn string(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prop_type: PropType::String,
            description: description.into(),
            required: false,
            default: None,
            constraints: None,
        }
    }

    /// Create a required number prop
    pub fn number(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prop_type: PropType::Number,
            description: description.into(),
            required: false,
            default: None,
            constraints: None,
        }
    }

    /// Create a boolean prop
    pub fn bool(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prop_type: PropType::Bool,
            description: description.into(),
            required: false,
            default: None,
            constraints: None,
        }
    }

    /// Create an enum prop
    pub fn enum_type(
        name: impl Into<String>,
        description: impl Into<String>,
        values: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            prop_type: PropType::Enum { values },
            description: description.into(),
            required: false,
            default: None,
            constraints: None,
        }
    }

    /// Create a color prop
    pub fn color(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            prop_type: PropType::Color,
            description: description.into(),
            required: false,
            default: None,
            constraints: None,
        }
    }

    /// Mark prop as required
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value
    pub fn with_default(mut self, value: serde_json::Value) -> Self {
        self.default = Some(value);
        self
    }

    /// Set constraints
    pub fn with_constraints(mut self, constraints: PropConstraints) -> Self {
        self.constraints = Some(constraints);
        self
    }
}

impl EventSpec {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            payload: None,
        }
    }

    pub fn with_payload(mut self, payload: serde_json::Value) -> Self {
        self.payload = Some(payload);
        self
    }
}

impl SlotSpec {
    pub fn default_slot() -> Self {
        Self {
            name: "default".to_string(),
            description: "Default slot for child content".to_string(),
            allowed_children: Vec::new(),
            min_children: 0,
            max_children: 0,
            required: false,
        }
    }

    pub fn named(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            allowed_children: Vec::new(),
            min_children: 0,
            max_children: 0,
            required: false,
        }
    }

    pub fn allow(mut self, component_id: impl Into<String>) -> Self {
        self.allowed_children.push(component_id.into());
        self
    }

    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    pub fn min(mut self, count: usize) -> Self {
        self.min_children = count;
        self
    }

    pub fn max(mut self, count: usize) -> Self {
        self.max_children = count;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_spec_builder() {
        let spec = ComponentSpec::builder("ui.Button", "ui.core")
            .description("A clickable button component")
            .prop(PropSpec::string("label", "Button text").required())
            .prop(PropSpec::enum_type(
                "variant",
                "Button style variant",
                vec!["primary".into(), "secondary".into(), "outline".into()],
            ).with_default(serde_json::json!("primary")))
            .prop(PropSpec::bool("disabled", "Whether button is disabled"))
            .event(EventSpec::new("on_click", "Fired when button is clicked"))
            .slot(SlotSpec::default_slot())
            .build();

        assert_eq!(spec.id, "ui.Button");
        assert_eq!(spec.pack, "ui.core");
        assert_eq!(spec.props.len(), 3);
        assert_eq!(spec.events.len(), 1);
        assert!(spec.get_prop("label").unwrap().required);
    }

    #[test]
    fn test_serialize_component_spec() {
        let spec = ComponentSpec::builder("ui.Button", "ui.core")
            .description("A button")
            .prop(PropSpec::string("label", "Text"))
            .build();

        let json = serde_json::to_string_pretty(&spec).unwrap();
        assert!(json.contains("ui.Button"));
    }
}
