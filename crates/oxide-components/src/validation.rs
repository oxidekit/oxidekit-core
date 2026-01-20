//! Validation Layer
//!
//! Validates UI files against registered component specifications.

use crate::registry::ComponentRegistry;
use crate::spec::{ComponentSpec, PropSpec, PropType};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Result of validating a UI tree
pub type ValidationResult = Result<(), Vec<ValidationError>>;

/// A validation error with structured information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Unique error code for programmatic handling
    pub code: ValidationErrorCode,

    /// Human-readable message
    pub message: String,

    /// Component ID where error occurred
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component_id: Option<String>,

    /// Property name if applicable
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,

    /// Line number in source file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,

    /// Column number in source file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub column: Option<usize>,

    /// Severity level
    pub severity: ValidationSeverity,

    /// Suggestions for fixing the error
    #[serde(default)]
    pub suggestions: Vec<String>,
}

/// Validation error codes
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ValidationErrorCode {
    // Component errors (100-199)
    UnknownComponent,
    DeprecatedComponent,
    ComponentVersionMismatch,

    // Property errors (200-299)
    UnknownProperty,
    RequiredPropertyMissing,
    InvalidPropertyType,
    InvalidPropertyValue,
    PropertyConstraintViolation,
    DeprecatedProperty,

    // Event errors (300-399)
    UnknownEvent,
    InvalidEventHandler,

    // Slot/children errors (400-499)
    InvalidChild,
    TooFewChildren,
    TooManyChildren,
    RequiredSlotMissing,
    SlotNotAllowed,

    // Style errors (500-599)
    UnknownStyleToken,
    InvalidStyleOverride,

    // Accessibility errors (600-699)
    MissingAriaLabel,
    MissingAriaAttribute,
    AccessibilityViolation,

    // General errors (900-999)
    SyntaxError,
    InternalError,
}

impl ValidationErrorCode {
    /// Get the numeric code
    pub fn code(&self) -> u16 {
        match self {
            Self::UnknownComponent => 100,
            Self::DeprecatedComponent => 101,
            Self::ComponentVersionMismatch => 102,
            Self::UnknownProperty => 200,
            Self::RequiredPropertyMissing => 201,
            Self::InvalidPropertyType => 202,
            Self::InvalidPropertyValue => 203,
            Self::PropertyConstraintViolation => 204,
            Self::DeprecatedProperty => 205,
            Self::UnknownEvent => 300,
            Self::InvalidEventHandler => 301,
            Self::InvalidChild => 400,
            Self::TooFewChildren => 401,
            Self::TooManyChildren => 402,
            Self::RequiredSlotMissing => 403,
            Self::SlotNotAllowed => 404,
            Self::UnknownStyleToken => 500,
            Self::InvalidStyleOverride => 501,
            Self::MissingAriaLabel => 600,
            Self::MissingAriaAttribute => 601,
            Self::AccessibilityViolation => 602,
            Self::SyntaxError => 900,
            Self::InternalError => 999,
        }
    }
}

/// Severity level for validation errors
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ValidationSeverity {
    #[default]
    Error,
    Warning,
    Info,
    Hint,
}

impl ValidationError {
    pub fn unknown_component(component: &str, line: Option<usize>, column: Option<usize>) -> Self {
        Self {
            code: ValidationErrorCode::UnknownComponent,
            message: format!("Unknown component: '{}'", component),
            component_id: Some(component.to_string()),
            property: None,
            line,
            column,
            severity: ValidationSeverity::Error,
            suggestions: vec![
                "Check the component name for typos".into(),
                "Ensure the component pack is installed".into(),
            ],
        }
    }

    pub fn deprecated_component(
        component: &str,
        replacement: Option<&str>,
        line: Option<usize>,
    ) -> Self {
        let mut suggestions = vec![];
        if let Some(r) = replacement {
            suggestions.push(format!("Use '{}' instead", r));
        }

        Self {
            code: ValidationErrorCode::DeprecatedComponent,
            message: format!("Component '{}' is deprecated", component),
            component_id: Some(component.to_string()),
            property: None,
            line,
            column: None,
            severity: ValidationSeverity::Warning,
            suggestions,
        }
    }

    pub fn unknown_property(
        component: &str,
        property: &str,
        valid_props: &[String],
        line: Option<usize>,
    ) -> Self {
        let mut suggestions = vec![];

        // Find similar property names
        for valid in valid_props.iter().take(3) {
            if levenshtein_distance(property, valid) <= 2 {
                suggestions.push(format!("Did you mean '{}'?", valid));
            }
        }

        Self {
            code: ValidationErrorCode::UnknownProperty,
            message: format!("Unknown property '{}' on component '{}'", property, component),
            component_id: Some(component.to_string()),
            property: Some(property.to_string()),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions,
        }
    }

    pub fn required_property_missing(
        component: &str,
        property: &str,
        line: Option<usize>,
    ) -> Self {
        Self {
            code: ValidationErrorCode::RequiredPropertyMissing,
            message: format!(
                "Required property '{}' is missing on component '{}'",
                property, component
            ),
            component_id: Some(component.to_string()),
            property: Some(property.to_string()),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions: vec![format!("Add the '{}' property", property)],
        }
    }

    pub fn invalid_property_type(
        component: &str,
        property: &str,
        expected: &str,
        got: &str,
        line: Option<usize>,
    ) -> Self {
        Self {
            code: ValidationErrorCode::InvalidPropertyType,
            message: format!(
                "Property '{}' on '{}' expects {} but got {}",
                property, component, expected, got
            ),
            component_id: Some(component.to_string()),
            property: Some(property.to_string()),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions: vec![format!("Provide a {} value", expected)],
        }
    }

    pub fn invalid_enum_value(
        component: &str,
        property: &str,
        value: &str,
        allowed: &[String],
        line: Option<usize>,
    ) -> Self {
        Self {
            code: ValidationErrorCode::InvalidPropertyValue,
            message: format!(
                "Invalid value '{}' for property '{}' on '{}'",
                value, property, component
            ),
            component_id: Some(component.to_string()),
            property: Some(property.to_string()),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions: vec![format!("Allowed values: {}", allowed.join(", "))],
        }
    }

    pub fn invalid_child(
        parent: &str,
        child: &str,
        slot: Option<&str>,
        allowed: &[String],
        line: Option<usize>,
    ) -> Self {
        let slot_info = slot.map(|s| format!(" in slot '{}'", s)).unwrap_or_default();
        Self {
            code: ValidationErrorCode::InvalidChild,
            message: format!(
                "Component '{}' is not allowed as a child of '{}'{}",
                child, parent, slot_info
            ),
            component_id: Some(parent.to_string()),
            property: slot.map(String::from),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions: if allowed.is_empty() {
                vec![]
            } else {
                vec![format!("Allowed children: {}", allowed.join(", "))]
            },
        }
    }

    pub fn missing_aria_label(component: &str, line: Option<usize>) -> Self {
        Self {
            code: ValidationErrorCode::MissingAriaLabel,
            message: format!(
                "Component '{}' requires an aria_label for accessibility",
                component
            ),
            component_id: Some(component.to_string()),
            property: Some("aria_label".to_string()),
            line,
            column: None,
            severity: ValidationSeverity::Error,
            suggestions: vec!["Add an 'aria_label' property with a descriptive label".into()],
        }
    }
}

/// UI element for validation (simplified representation)
#[derive(Debug, Clone)]
pub struct UiElement {
    /// Component name/ID
    pub component: String,

    /// Properties
    pub props: Vec<(String, UiValue)>,

    /// Children elements
    pub children: Vec<UiElement>,

    /// Source location
    pub line: Option<usize>,
    pub column: Option<usize>,
}

/// Value types in UI files
#[derive(Debug, Clone)]
pub enum UiValue {
    String(String),
    Number(f64),
    Bool(bool),
    Ident(String),
    Array(Vec<UiValue>),
}

impl UiValue {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::String(_) => "string",
            Self::Number(_) => "number",
            Self::Bool(_) => "bool",
            Self::Ident(_) => "identifier",
            Self::Array(_) => "array",
        }
    }
}

/// Validator for UI trees
pub struct Validator {
    registry: Arc<ComponentRegistry>,
    strict_mode: bool,
}

impl Validator {
    /// Create a new validator with a component registry
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self {
            registry,
            strict_mode: false,
        }
    }

    /// Enable strict mode (treats warnings as errors)
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Validate a UI element tree
    pub fn validate(&self, root: &UiElement) -> ValidationResult {
        let mut errors = Vec::new();
        self.validate_element(root, &mut errors);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn validate_element(&self, element: &UiElement, errors: &mut Vec<ValidationError>) {
        // Check if component exists
        let spec = match self.registry.get(&element.component) {
            Some(spec) => spec,
            None => {
                errors.push(ValidationError::unknown_component(
                    &element.component,
                    element.line,
                    element.column,
                ));
                return;
            }
        };

        // Check for deprecation
        if let Some(deprecation) = &spec.deprecated {
            errors.push(ValidationError::deprecated_component(
                &element.component,
                deprecation.replacement.as_deref(),
                element.line,
            ));
        }

        // Validate properties
        self.validate_props(element, &spec, errors);

        // Validate accessibility
        self.validate_accessibility(element, &spec, errors);

        // Validate children
        self.validate_children(element, &spec, errors);
    }

    fn validate_props(
        &self,
        element: &UiElement,
        spec: &ComponentSpec,
        errors: &mut Vec<ValidationError>,
    ) {
        let valid_prop_names: Vec<String> = spec.props.iter().map(|p| p.name.clone()).collect();

        // Check each provided prop
        for (name, value) in &element.props {
            // Skip event handlers
            if name.starts_with("on_") {
                // TODO: Validate event handlers
                continue;
            }

            // Check if prop exists
            let prop_spec = match spec.get_prop(name) {
                Some(p) => p,
                None => {
                    errors.push(ValidationError::unknown_property(
                        &element.component,
                        name,
                        &valid_prop_names,
                        element.line,
                    ));
                    continue;
                }
            };

            // Validate prop type
            self.validate_prop_value(element, name, value, prop_spec, errors);
        }

        // Check for missing required props
        for prop_spec in &spec.props {
            if prop_spec.required {
                let provided = element.props.iter().any(|(n, _)| n == &prop_spec.name);
                if !provided {
                    errors.push(ValidationError::required_property_missing(
                        &element.component,
                        &prop_spec.name,
                        element.line,
                    ));
                }
            }
        }
    }

    fn validate_prop_value(
        &self,
        element: &UiElement,
        prop_name: &str,
        value: &UiValue,
        spec: &PropSpec,
        errors: &mut Vec<ValidationError>,
    ) {
        match (&spec.prop_type, value) {
            // String types
            (PropType::String, UiValue::String(_)) => {}
            (PropType::String, UiValue::Ident(_)) => {} // Allow identifiers as strings

            // Number types
            (PropType::Number, UiValue::Number(_)) => {}
            (PropType::Integer, UiValue::Number(n)) => {
                if n.fract() != 0.0 {
                    errors.push(ValidationError::invalid_property_type(
                        &element.component,
                        prop_name,
                        "integer",
                        "float",
                        element.line,
                    ));
                }
            }

            // Boolean
            (PropType::Bool, UiValue::Bool(_)) => {}

            // Enum
            (PropType::Enum { values }, UiValue::String(s)) => {
                if !values.contains(s) {
                    errors.push(ValidationError::invalid_enum_value(
                        &element.component,
                        prop_name,
                        s,
                        values,
                        element.line,
                    ));
                }
            }
            (PropType::Enum { values }, UiValue::Ident(s)) => {
                if !values.contains(s) {
                    errors.push(ValidationError::invalid_enum_value(
                        &element.component,
                        prop_name,
                        s,
                        values,
                        element.line,
                    ));
                }
            }

            // Color
            (PropType::Color, UiValue::String(_)) => {} // Accept any string as color for now
            (PropType::Color, UiValue::Ident(_)) => {}   // Token reference

            // Spacing/Size
            (PropType::Spacing, UiValue::Number(_)) => {}
            (PropType::Spacing, UiValue::Ident(_)) => {} // Token reference
            (PropType::Size, UiValue::Number(_)) => {}
            (PropType::Size, UiValue::String(_)) => {}   // "fill", "auto", etc.
            (PropType::Size, UiValue::Ident(_)) => {}

            // Type mismatch
            _ => {
                errors.push(ValidationError::invalid_property_type(
                    &element.component,
                    prop_name,
                    &format!("{:?}", spec.prop_type),
                    value.type_name(),
                    element.line,
                ));
            }
        }
    }

    fn validate_accessibility(
        &self,
        element: &UiElement,
        spec: &ComponentSpec,
        errors: &mut Vec<ValidationError>,
    ) {
        // Check required ARIA attributes
        for required_aria in &spec.accessibility.required_aria {
            let prop_name = required_aria.replace('-', "_");
            let has_prop = element.props.iter().any(|(n, _)| n == &prop_name);
            if !has_prop {
                errors.push(ValidationError::missing_aria_label(
                    &element.component,
                    element.line,
                ));
            }
        }
    }

    fn validate_children(
        &self,
        element: &UiElement,
        spec: &ComponentSpec,
        errors: &mut Vec<ValidationError>,
    ) {
        // Get default slot
        let default_slot = spec.default_slot();

        // Validate each child
        for child in &element.children {
            // Check if child is allowed
            if let Some(slot) = default_slot {
                if !slot.allowed_children.is_empty() {
                    if !slot.allowed_children.contains(&child.component) {
                        errors.push(ValidationError::invalid_child(
                            &element.component,
                            &child.component,
                            Some("default"),
                            &slot.allowed_children,
                            child.line,
                        ));
                    }
                }
            }

            // Recursively validate child
            self.validate_element(child, errors);
        }

        // Check child count constraints
        if let Some(slot) = default_slot {
            let child_count = element.children.len();

            if slot.min_children > 0 && child_count < slot.min_children {
                errors.push(ValidationError {
                    code: ValidationErrorCode::TooFewChildren,
                    message: format!(
                        "Component '{}' requires at least {} children, but has {}",
                        element.component, slot.min_children, child_count
                    ),
                    component_id: Some(element.component.clone()),
                    property: None,
                    line: element.line,
                    column: None,
                    severity: ValidationSeverity::Error,
                    suggestions: vec![],
                });
            }

            if slot.max_children > 0 && child_count > slot.max_children {
                errors.push(ValidationError {
                    code: ValidationErrorCode::TooManyChildren,
                    message: format!(
                        "Component '{}' allows at most {} children, but has {}",
                        element.component, slot.max_children, child_count
                    ),
                    component_id: Some(element.component.clone()),
                    property: None,
                    line: element.line,
                    column: None,
                    severity: ValidationSeverity::Error,
                    suggestions: vec![],
                });
            }
        }
    }
}

/// Simple Levenshtein distance for typo suggestions
fn levenshtein_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let a_len = a_chars.len();
    let b_len = b_chars.len();

    if a_len == 0 {
        return b_len;
    }
    if b_len == 0 {
        return a_len;
    }

    let mut matrix = vec![vec![0; b_len + 1]; a_len + 1];

    for i in 0..=a_len {
        matrix[i][0] = i;
    }
    for j in 0..=b_len {
        matrix[0][j] = j;
    }

    for i in 1..=a_len {
        for j in 1..=b_len {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[a_len][b_len]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_registry() -> Arc<ComponentRegistry> {
        Arc::new(ComponentRegistry::with_core_components())
    }

    #[test]
    fn test_valid_button() {
        let registry = setup_registry();
        let validator = Validator::new(registry);

        let element = UiElement {
            component: "ui.Button".into(),
            props: vec![
                ("label".into(), UiValue::String("Click me".into())),
                ("variant".into(), UiValue::String("primary".into())),
            ],
            children: vec![],
            line: Some(1),
            column: None,
        };

        let result = validator.validate(&element);
        assert!(result.is_ok());
    }

    #[test]
    fn test_unknown_component() {
        let registry = setup_registry();
        let validator = Validator::new(registry);

        let element = UiElement {
            component: "ui.FakeComponent".into(),
            props: vec![],
            children: vec![],
            line: Some(1),
            column: None,
        };

        let result = validator.validate(&element);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors[0].code, ValidationErrorCode::UnknownComponent);
    }

    #[test]
    fn test_unknown_property() {
        let registry = setup_registry();
        let validator = Validator::new(registry);

        let element = UiElement {
            component: "ui.Button".into(),
            props: vec![("fake_prop".into(), UiValue::String("value".into()))],
            children: vec![],
            line: Some(1),
            column: None,
        };

        let result = validator.validate(&element);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors[0].code, ValidationErrorCode::UnknownProperty);
    }

    #[test]
    fn test_invalid_enum_value() {
        let registry = setup_registry();
        let validator = Validator::new(registry);

        let element = UiElement {
            component: "ui.Button".into(),
            props: vec![("variant".into(), UiValue::String("invalid_variant".into()))],
            children: vec![],
            line: Some(1),
            column: None,
        };

        let result = validator.validate(&element);
        assert!(result.is_err());

        let errors = result.unwrap_err();
        assert_eq!(errors[0].code, ValidationErrorCode::InvalidPropertyValue);
    }

    #[test]
    fn test_levenshtein() {
        assert_eq!(levenshtein_distance("kitten", "sitting"), 3);
        assert_eq!(levenshtein_distance("label", "lable"), 2);
        assert_eq!(levenshtein_distance("", "test"), 4);
    }
}
