//! Material Design 3 Text Field Component Specifications
//!
//! Includes: FilledTextField, OutlinedTextField

use crate::spec::{
    AccessibilitySpec, ComponentSpec, EventSpec, KeyboardBehavior, PropSpec, SlotSpec,
    UsageExample, VariantSpec,
};
use std::collections::HashMap;

/// Creates the FilledTextField component specification
///
/// A text field with a filled container background.
/// M3 Reference: https://m3.material.io/components/text-fields/specs#6cefdb2e-4a76-41c4-9ff9-1e0da8ec39e5
pub fn filled_text_field_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.FilledTextField", "m3.input")
        .name("FilledTextField")
        .description("A text input with filled container for collecting user text input")
        .version("1.0.0")
        // Core props
        .prop(PropSpec::string("value", "Current input value"))
        .prop(PropSpec::string("label", "Floating label text"))
        .prop(PropSpec::string("placeholder", "Placeholder text when empty and unfocused"))
        .prop(
            PropSpec::enum_type(
                "input_type",
                "Input type for keyboard and validation",
                vec![
                    "text".into(),
                    "password".into(),
                    "email".into(),
                    "number".into(),
                    "tel".into(),
                    "url".into(),
                    "search".into(),
                    "decimal".into(),
                ],
            )
            .with_default(serde_json::json!("text")),
        )
        // Supporting text
        .prop(PropSpec::string("helper_text", "Supporting text displayed below the field"))
        .prop(PropSpec::string("error_text", "Error message displayed when in error state"))
        .prop(PropSpec::number("character_count", "Current character count for display"))
        .prop(PropSpec::number("max_length", "Maximum character limit"))
        .prop(PropSpec::bool("show_character_count", "Display character counter"))
        // Icons
        .prop(PropSpec::string("leading_icon", "Material icon name for leading position"))
        .prop(PropSpec::string("trailing_icon", "Material icon name for trailing position"))
        .prop(PropSpec::bool("trailing_icon_clickable", "Whether trailing icon triggers on_trailing_icon_click"))
        // States
        .prop(PropSpec::bool("disabled", "Whether the field is disabled"))
        .prop(PropSpec::bool("readonly", "Whether the field is read-only"))
        .prop(PropSpec::bool("required", "Whether the field is required"))
        .prop(PropSpec::bool("error", "Whether the field is in error state"))
        .prop(PropSpec::bool("focused", "Whether the field is currently focused (controlled)"))
        // Number-specific
        .prop(PropSpec::number("min", "Minimum value (for number type)"))
        .prop(PropSpec::number("max", "Maximum value (for number type)"))
        .prop(PropSpec::number("step", "Step increment (for number type)"))
        // Text-specific
        .prop(PropSpec::string("autocomplete", "Browser autocomplete attribute"))
        .prop(PropSpec::string("pattern", "Validation regex pattern"))
        // Events
        .event(EventSpec::new("on_change", "Fired when value changes (debounced)"))
        .event(EventSpec::new("on_input", "Fired on every keystroke"))
        .event(EventSpec::new("on_focus", "Fired when field receives focus"))
        .event(EventSpec::new("on_blur", "Fired when field loses focus"))
        .event(EventSpec::new("on_submit", "Fired when Enter key is pressed"))
        .event(EventSpec::new("on_leading_icon_click", "Fired when leading icon is clicked"))
        .event(EventSpec::new("on_trailing_icon_click", "Fired when trailing icon is clicked"))
        // Style tokens
        .style_token("color.surface-container-highest")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .style_token("color.error")
        .style_token("shape.corner.extra-small-top")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.disabled")
        .accessibility(AccessibilitySpec {
            role: Some("textbox".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Move focus to next element".into(),
                },
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Submit (if applicable)".into(),
                },
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Clear focus or cancel".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .variant(VariantSpec {
            name: "default".into(),
            description: "Default state with inactive indicator".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "focused".into(),
            description: "Focused state with active indicator".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "error".into(),
            description: "Error state with error color indicator".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "disabled".into(),
            description: "Disabled state with reduced opacity".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Basic Filled Text Field".into(),
            description: Some("Simple text input with label".into()),
            code: r#"FilledTextField {
    label: "Username"
    value: username
    on_change: set_username
}"#.into(),
        })
        .example(UsageExample {
            title: "Text Field with Helper Text".into(),
            description: Some("Input with supporting helper text".into()),
            code: r#"FilledTextField {
    label: "Email"
    input_type: "email"
    value: email
    helper_text: "We'll never share your email"
    on_change: set_email
}"#.into(),
        })
        .example(UsageExample {
            title: "Text Field with Error".into(),
            description: Some("Input in error state with error message".into()),
            code: r#"FilledTextField {
    label: "Password"
    input_type: "password"
    value: password
    error: password_invalid
    error_text: "Password must be at least 8 characters"
    on_change: set_password
}"#.into(),
        })
        .example(UsageExample {
            title: "Text Field with Icons".into(),
            description: Some("Input with leading and trailing icons".into()),
            code: r#"FilledTextField {
    label: "Search"
    leading_icon: "search"
    trailing_icon: "clear"
    trailing_icon_clickable: true
    on_trailing_icon_click: clear_search
    value: search_query
    on_change: set_search_query
}"#.into(),
        })
        .build()
}

/// Creates the OutlinedTextField component specification
///
/// A text field with an outline border.
/// M3 Reference: https://m3.material.io/components/text-fields/specs#68b00bd6-ab40-4b4f-93d9-ed1c0bae9caf
pub fn outlined_text_field_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.OutlinedTextField", "m3.input")
        .name("OutlinedTextField")
        .description("A text input with outline border for collecting user text input")
        .version("1.0.0")
        // Core props
        .prop(PropSpec::string("value", "Current input value"))
        .prop(PropSpec::string("label", "Floating label text"))
        .prop(PropSpec::string("placeholder", "Placeholder text when empty and unfocused"))
        .prop(
            PropSpec::enum_type(
                "input_type",
                "Input type for keyboard and validation",
                vec![
                    "text".into(),
                    "password".into(),
                    "email".into(),
                    "number".into(),
                    "tel".into(),
                    "url".into(),
                    "search".into(),
                    "decimal".into(),
                ],
            )
            .with_default(serde_json::json!("text")),
        )
        // Supporting text
        .prop(PropSpec::string("helper_text", "Supporting text displayed below the field"))
        .prop(PropSpec::string("error_text", "Error message displayed when in error state"))
        .prop(PropSpec::number("character_count", "Current character count for display"))
        .prop(PropSpec::number("max_length", "Maximum character limit"))
        .prop(PropSpec::bool("show_character_count", "Display character counter"))
        // Prefix/Suffix
        .prop(PropSpec::string("prefix_text", "Text prefix displayed inside the field"))
        .prop(PropSpec::string("suffix_text", "Text suffix displayed inside the field"))
        // Icons
        .prop(PropSpec::string("leading_icon", "Material icon name for leading position"))
        .prop(PropSpec::string("trailing_icon", "Material icon name for trailing position"))
        .prop(PropSpec::bool("trailing_icon_clickable", "Whether trailing icon triggers on_trailing_icon_click"))
        // States
        .prop(PropSpec::bool("disabled", "Whether the field is disabled"))
        .prop(PropSpec::bool("readonly", "Whether the field is read-only"))
        .prop(PropSpec::bool("required", "Whether the field is required"))
        .prop(PropSpec::bool("error", "Whether the field is in error state"))
        .prop(PropSpec::bool("focused", "Whether the field is currently focused (controlled)"))
        // Number-specific
        .prop(PropSpec::number("min", "Minimum value (for number type)"))
        .prop(PropSpec::number("max", "Maximum value (for number type)"))
        .prop(PropSpec::number("step", "Step increment (for number type)"))
        // Text-specific
        .prop(PropSpec::string("autocomplete", "Browser autocomplete attribute"))
        .prop(PropSpec::string("pattern", "Validation regex pattern"))
        // Events
        .event(EventSpec::new("on_change", "Fired when value changes (debounced)"))
        .event(EventSpec::new("on_input", "Fired on every keystroke"))
        .event(EventSpec::new("on_focus", "Fired when field receives focus"))
        .event(EventSpec::new("on_blur", "Fired when field loses focus"))
        .event(EventSpec::new("on_submit", "Fired when Enter key is pressed"))
        .event(EventSpec::new("on_leading_icon_click", "Fired when leading icon is clicked"))
        .event(EventSpec::new("on_trailing_icon_click", "Fired when trailing icon is clicked"))
        // Style tokens
        .style_token("color.surface")
        .style_token("color.outline")
        .style_token("color.outline-variant")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .style_token("color.error")
        .style_token("shape.corner.extra-small")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.disabled")
        .accessibility(AccessibilitySpec {
            role: Some("textbox".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Move focus to next element".into(),
                },
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Submit (if applicable)".into(),
                },
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Clear focus or cancel".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .variant(VariantSpec {
            name: "default".into(),
            description: "Default state with outline border".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "focused".into(),
            description: "Focused state with primary color border".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "error".into(),
            description: "Error state with error color border".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "disabled".into(),
            description: "Disabled state with reduced opacity".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Basic Outlined Text Field".into(),
            description: Some("Simple text input with outline".into()),
            code: r#"OutlinedTextField {
    label: "Full Name"
    value: full_name
    on_change: set_full_name
}"#.into(),
        })
        .example(UsageExample {
            title: "Text Field with Prefix and Suffix".into(),
            description: Some("Input with prefix/suffix text".into()),
            code: r#"OutlinedTextField {
    label: "Amount"
    input_type: "decimal"
    prefix_text: "$"
    suffix_text: "USD"
    value: amount
    on_change: set_amount
}"#.into(),
        })
        .example(UsageExample {
            title: "Text Field with Character Count".into(),
            description: Some("Input with character counter".into()),
            code: r#"OutlinedTextField {
    label: "Bio"
    max_length: 200
    show_character_count: true
    value: bio
    on_change: set_bio
}"#.into(),
        })
        .build()
}

/// Creates the TextArea component specification for multiline text input
///
/// Multiline text field using M3 styling
pub fn text_area_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.TextArea", "m3.input")
        .name("TextArea")
        .description("A multiline text input field for longer form content")
        .version("1.0.0")
        // Core props
        .prop(PropSpec::string("value", "Current textarea value"))
        .prop(PropSpec::string("label", "Floating label text"))
        .prop(PropSpec::string("placeholder", "Placeholder text when empty"))
        .prop(
            PropSpec::enum_type(
                "variant",
                "Text area style variant",
                vec!["filled".into(), "outlined".into()],
            )
            .with_default(serde_json::json!("outlined")),
        )
        // Size
        .prop(PropSpec::number("rows", "Number of visible text rows").with_default(serde_json::json!(4)))
        .prop(PropSpec::number("min_rows", "Minimum number of rows (for auto-resize)"))
        .prop(PropSpec::number("max_rows", "Maximum number of rows (for auto-resize)"))
        .prop(PropSpec::bool("auto_resize", "Automatically resize based on content"))
        // Supporting text
        .prop(PropSpec::string("helper_text", "Supporting text displayed below"))
        .prop(PropSpec::string("error_text", "Error message displayed when in error state"))
        .prop(PropSpec::number("max_length", "Maximum character limit"))
        .prop(PropSpec::bool("show_character_count", "Display character counter"))
        // States
        .prop(PropSpec::bool("disabled", "Whether the field is disabled"))
        .prop(PropSpec::bool("readonly", "Whether the field is read-only"))
        .prop(PropSpec::bool("required", "Whether the field is required"))
        .prop(PropSpec::bool("error", "Whether the field is in error state"))
        .prop(PropSpec::bool("resizable", "Whether the user can resize the textarea"))
        // Events
        .event(EventSpec::new("on_change", "Fired when value changes"))
        .event(EventSpec::new("on_input", "Fired on every keystroke"))
        .event(EventSpec::new("on_focus", "Fired when field receives focus"))
        .event(EventSpec::new("on_blur", "Fired when field loses focus"))
        // Style tokens
        .style_token("color.surface-container-highest")
        .style_token("color.outline")
        .style_token("color.on-surface")
        .style_token("color.primary")
        .style_token("color.error")
        .style_token("shape.corner.extra-small")
        .accessibility(AccessibilitySpec {
            role: Some("textbox".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Move focus to next element".into(),
                },
            ],
            required_aria: vec!["aria-label".into(), "aria-multiline".into()],
        })
        .example(UsageExample {
            title: "Basic Text Area".into(),
            description: Some("Multiline text input".into()),
            code: r#"TextArea {
    label: "Description"
    rows: 4
    value: description
    on_change: set_description
}"#.into(),
        })
        .example(UsageExample {
            title: "Auto-resize Text Area".into(),
            description: Some("Text area that grows with content".into()),
            code: r#"TextArea {
    label: "Comments"
    auto_resize: true
    min_rows: 2
    max_rows: 10
    value: comments
    on_change: set_comments
}"#.into(),
        })
        .build()
}

/// Returns all text field component specifications
pub fn all_text_field_specs() -> Vec<ComponentSpec> {
    vec![
        filled_text_field_spec(),
        outlined_text_field_spec(),
        text_area_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filled_text_field_spec() {
        let spec = filled_text_field_spec();
        assert_eq!(spec.id, "m3.FilledTextField");
        assert_eq!(spec.pack, "m3.input");
        assert!(spec.get_prop("value").is_some());
        assert!(spec.get_prop("label").is_some());
        assert!(spec.get_prop("error").is_some());
        assert!(spec.get_event("on_change").is_some());
    }

    #[test]
    fn test_outlined_text_field_has_prefix_suffix() {
        let spec = outlined_text_field_spec();
        assert!(spec.get_prop("prefix_text").is_some());
        assert!(spec.get_prop("suffix_text").is_some());
    }

    #[test]
    fn test_text_field_variants() {
        let spec = filled_text_field_spec();
        assert_eq!(spec.variants.len(), 4);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"default"));
        assert!(variant_names.contains(&"focused"));
        assert!(variant_names.contains(&"error"));
        assert!(variant_names.contains(&"disabled"));
    }

    #[test]
    fn test_text_area_multiline() {
        let spec = text_area_spec();
        assert!(spec.accessibility.required_aria.contains(&"aria-multiline".into()));
    }

    #[test]
    fn test_all_text_field_specs_count() {
        let specs = all_text_field_specs();
        assert_eq!(specs.len(), 3);
    }
}
