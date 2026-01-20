//! Material Design 3 Selection Controls Component Specifications
//!
//! Includes: Checkbox, RadioButton, RadioGroup, Switch, Slider

use crate::spec::{
    AccessibilitySpec, ComponentSpec, EventSpec, KeyboardBehavior, PropSpec, SlotSpec,
    UsageExample, VariantSpec,
};
use std::collections::HashMap;

/// Creates the Checkbox component specification
///
/// A checkbox for binary selection with optional indeterminate state.
/// M3 Reference: https://m3.material.io/components/checkbox/specs
pub fn checkbox_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Checkbox", "m3.selection")
        .name("Checkbox")
        .description("A checkbox control for binary selection with optional indeterminate state")
        .version("1.0.0")
        .prop(PropSpec::bool("checked", "Whether the checkbox is checked"))
        .prop(PropSpec::bool("indeterminate", "Whether the checkbox is in indeterminate state (partially checked)"))
        .prop(PropSpec::string("label", "Label text displayed next to the checkbox"))
        .prop(PropSpec::bool("disabled", "Whether the checkbox is disabled"))
        .prop(PropSpec::bool("error", "Whether the checkbox is in error state"))
        .prop(PropSpec::string("value", "Value identifier for form submission"))
        .prop(PropSpec::string("name", "Form field name"))
        .event(EventSpec::new("on_change", "Fired when checked state changes"))
        .event(EventSpec::new("on_focus", "Fired when checkbox receives focus"))
        .event(EventSpec::new("on_blur", "Fired when checkbox loses focus"))
        .style_token("color.primary")
        .style_token("color.on-primary")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.error")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("checkbox".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Toggle checkbox".into(),
                },
            ],
            required_aria: vec!["aria-checked".into()],
        })
        .variant(VariantSpec {
            name: "unchecked".into(),
            description: "Unselected state with empty box".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "checked".into(),
            description: "Selected state with checkmark".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "indeterminate".into(),
            description: "Partially selected state with dash".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Basic Checkbox".into(),
            description: Some("Simple checkbox with label".into()),
            code: r#"Checkbox {
    label: "Accept terms and conditions"
    checked: accepted
    on_change: set_accepted
}"#.into(),
        })
        .example(UsageExample {
            title: "Indeterminate Checkbox".into(),
            description: Some("Parent checkbox for partial selection".into()),
            code: r#"Checkbox {
    label: "Select All"
    checked: all_selected
    indeterminate: some_selected
    on_change: toggle_all
}"#.into(),
        })
        .build()
}

/// Creates the RadioButton component specification
///
/// A radio button for single selection from a group.
/// M3 Reference: https://m3.material.io/components/radio-button/specs
pub fn radio_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.RadioButton", "m3.selection")
        .name("RadioButton")
        .description("A radio button for selecting one option from a group")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this option").required())
        .prop(PropSpec::string("label", "Label text displayed next to the radio button"))
        .prop(PropSpec::bool("checked", "Whether this radio button is selected"))
        .prop(PropSpec::bool("disabled", "Whether the radio button is disabled"))
        .prop(PropSpec::string("name", "Group name for radio buttons (required for native form behavior)"))
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .event(EventSpec::new("on_focus", "Fired when radio button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when radio button loses focus"))
        .style_token("color.primary")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("radio".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Select radio button".into(),
                },
                KeyboardBehavior {
                    key: "ArrowDown/ArrowRight".into(),
                    action: "Move to next radio in group".into(),
                },
                KeyboardBehavior {
                    key: "ArrowUp/ArrowLeft".into(),
                    action: "Move to previous radio in group".into(),
                },
            ],
            required_aria: vec!["aria-checked".into()],
        })
        .example(UsageExample {
            title: "Radio Button".into(),
            description: Some("Individual radio button".into()),
            code: r#"RadioButton {
    value: "option1"
    label: "Option 1"
    checked: selected == "option1"
    on_change: set_selected
}"#.into(),
        })
        .build()
}

/// Creates the RadioGroup component specification
///
/// A container for radio buttons ensuring single selection.
pub fn radio_group_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.RadioGroup", "m3.selection")
        .name("RadioGroup")
        .description("A container component that manages a group of radio buttons")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Currently selected value"))
        .prop(PropSpec::string("name", "Form field name for the group").required())
        .prop(PropSpec::string("label", "Accessible label for the group"))
        .prop(PropSpec::bool("disabled", "Disable all radio buttons in the group"))
        .prop(PropSpec::bool("required", "Whether a selection is required"))
        .prop(
            PropSpec::enum_type(
                "orientation",
                "Layout orientation of the radio buttons",
                vec!["vertical".into(), "horizontal".into()],
            )
            .with_default(serde_json::json!("vertical")),
        )
        .prop(PropSpec::number("gap", "Space between radio buttons"))
        .slot(SlotSpec::default_slot().allow("m3.RadioButton").min(2))
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .accessibility(AccessibilitySpec {
            role: Some("radiogroup".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowDown/ArrowRight".into(),
                    action: "Move to next radio".into(),
                },
                KeyboardBehavior {
                    key: "ArrowUp/ArrowLeft".into(),
                    action: "Move to previous radio".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Radio Group".into(),
            description: Some("Group of radio buttons".into()),
            code: r#"RadioGroup {
    name: "color"
    value: selected_color
    on_change: set_selected_color
} {
    RadioButton { value: "red" label: "Red" }
    RadioButton { value: "green" label: "Green" }
    RadioButton { value: "blue" label: "Blue" }
}"#.into(),
        })
        .build()
}

/// Creates the Switch component specification
///
/// A toggle switch for on/off states.
/// M3 Reference: https://m3.material.io/components/switch/specs
pub fn switch_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Switch", "m3.selection")
        .name("Switch")
        .description("A toggle switch for binary on/off selections")
        .version("1.0.0")
        .prop(PropSpec::bool("checked", "Whether the switch is on"))
        .prop(PropSpec::string("label", "Label text displayed next to the switch"))
        .prop(PropSpec::bool("disabled", "Whether the switch is disabled"))
        .prop(PropSpec::string("icon", "Icon to show in the thumb (both states)"))
        .prop(PropSpec::string("icon_on", "Icon to show when switch is on"))
        .prop(PropSpec::string("icon_off", "Icon to show when switch is off"))
        .prop(PropSpec::bool("show_icons", "Whether to show icons in thumb").with_default(serde_json::json!(false)))
        .prop(PropSpec::string("value", "Value identifier for form submission"))
        .prop(PropSpec::string("name", "Form field name"))
        .event(EventSpec::new("on_change", "Fired when switch state changes"))
        .event(EventSpec::new("on_focus", "Fired when switch receives focus"))
        .event(EventSpec::new("on_blur", "Fired when switch loses focus"))
        .style_token("color.primary")
        .style_token("color.on-primary")
        .style_token("color.primary-container")
        .style_token("color.on-primary-container")
        .style_token("color.surface-container-highest")
        .style_token("color.outline")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("switch".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Toggle switch".into(),
                },
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Toggle switch".into(),
                },
            ],
            required_aria: vec!["aria-checked".into()],
        })
        .variant(VariantSpec {
            name: "off".into(),
            description: "Switch in off position".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "on".into(),
            description: "Switch in on position".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Basic Switch".into(),
            description: Some("Simple toggle switch".into()),
            code: r#"Switch {
    label: "Dark Mode"
    checked: dark_mode
    on_change: set_dark_mode
}"#.into(),
        })
        .example(UsageExample {
            title: "Switch with Icons".into(),
            description: Some("Switch showing state icons".into()),
            code: r#"Switch {
    label: "Wi-Fi"
    checked: wifi_enabled
    show_icons: true
    icon_on: "wifi"
    icon_off: "wifi_off"
    on_change: set_wifi_enabled
}"#.into(),
        })
        .build()
}

/// Creates the Slider component specification (continuous)
///
/// A slider for selecting a value within a range.
/// M3 Reference: https://m3.material.io/components/sliders/specs
pub fn slider_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Slider", "m3.selection")
        .name("Slider")
        .description("A slider for selecting a continuous value within a range")
        .version("1.0.0")
        .prop(PropSpec::number("value", "Current slider value").required())
        .prop(PropSpec::number("min", "Minimum value").with_default(serde_json::json!(0)))
        .prop(PropSpec::number("max", "Maximum value").with_default(serde_json::json!(100)))
        .prop(PropSpec::number("step", "Step increment (0 for continuous)").with_default(serde_json::json!(0)))
        .prop(PropSpec::bool("disabled", "Whether the slider is disabled"))
        .prop(PropSpec::string("label", "Accessible label for the slider"))
        .prop(PropSpec::bool("show_value_label", "Show value indicator above thumb while dragging"))
        .prop(PropSpec::bool("show_tick_marks", "Show tick marks for discrete steps"))
        .prop(
            PropSpec::enum_type(
                "orientation",
                "Slider orientation",
                vec!["horizontal".into(), "vertical".into()],
            )
            .with_default(serde_json::json!("horizontal")),
        )
        .event(EventSpec::new("on_change", "Fired when value changes"))
        .event(EventSpec::new("on_change_start", "Fired when drag starts"))
        .event(EventSpec::new("on_change_end", "Fired when drag ends"))
        .style_token("color.primary")
        .style_token("color.primary-container")
        .style_token("color.surface-container-highest")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("slider".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowRight/ArrowUp".into(),
                    action: "Increase value".into(),
                },
                KeyboardBehavior {
                    key: "ArrowLeft/ArrowDown".into(),
                    action: "Decrease value".into(),
                },
                KeyboardBehavior {
                    key: "Home".into(),
                    action: "Set to minimum".into(),
                },
                KeyboardBehavior {
                    key: "End".into(),
                    action: "Set to maximum".into(),
                },
                KeyboardBehavior {
                    key: "PageUp".into(),
                    action: "Increase by large step".into(),
                },
                KeyboardBehavior {
                    key: "PageDown".into(),
                    action: "Decrease by large step".into(),
                },
            ],
            required_aria: vec![
                "aria-valuemin".into(),
                "aria-valuemax".into(),
                "aria-valuenow".into(),
            ],
        })
        .example(UsageExample {
            title: "Continuous Slider".into(),
            description: Some("Slider for continuous values".into()),
            code: r#"Slider {
    label: "Volume"
    value: volume
    min: 0
    max: 100
    on_change: set_volume
}"#.into(),
        })
        .example(UsageExample {
            title: "Discrete Slider".into(),
            description: Some("Slider with discrete steps".into()),
            code: r#"Slider {
    label: "Quality"
    value: quality
    min: 1
    max: 5
    step: 1
    show_tick_marks: true
    on_change: set_quality
}"#.into(),
        })
        .build()
}

/// Creates the RangeSlider component specification
///
/// A slider for selecting a range of values.
pub fn range_slider_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.RangeSlider", "m3.selection")
        .name("RangeSlider")
        .description("A slider for selecting a range between two values")
        .version("1.0.0")
        .prop(PropSpec::number("value_start", "Start value of the range").required())
        .prop(PropSpec::number("value_end", "End value of the range").required())
        .prop(PropSpec::number("min", "Minimum allowed value").with_default(serde_json::json!(0)))
        .prop(PropSpec::number("max", "Maximum allowed value").with_default(serde_json::json!(100)))
        .prop(PropSpec::number("step", "Step increment (0 for continuous)").with_default(serde_json::json!(0)))
        .prop(PropSpec::number("min_distance", "Minimum distance between start and end values"))
        .prop(PropSpec::bool("disabled", "Whether the slider is disabled"))
        .prop(PropSpec::string("label", "Accessible label for the slider"))
        .prop(PropSpec::bool("show_value_label", "Show value indicators above thumbs while dragging"))
        .prop(PropSpec::bool("show_tick_marks", "Show tick marks for discrete steps"))
        .event(EventSpec::new("on_change", "Fired when either value changes"))
        .event(EventSpec::new("on_change_start", "Fired when drag starts"))
        .event(EventSpec::new("on_change_end", "Fired when drag ends"))
        .style_token("color.primary")
        .style_token("color.primary-container")
        .style_token("color.surface-container-highest")
        .style_token("state.hover")
        .style_token("state.focused")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("group".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Move between thumbs".into(),
                },
                KeyboardBehavior {
                    key: "ArrowRight/ArrowUp".into(),
                    action: "Increase focused thumb value".into(),
                },
                KeyboardBehavior {
                    key: "ArrowLeft/ArrowDown".into(),
                    action: "Decrease focused thumb value".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Range Slider".into(),
            description: Some("Select a price range".into()),
            code: r#"RangeSlider {
    label: "Price Range"
    value_start: price_min
    value_end: price_max
    min: 0
    max: 1000
    on_change: handle_price_change
}"#.into(),
        })
        .build()
}

/// Creates the ChipGroup component specification (for filter chips)
///
/// A group of selectable chips.
pub fn chip_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Chip", "m3.selection")
        .name("Chip")
        .description("A compact element for selections, filters, or actions")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Text label for the chip").required())
        .prop(
            PropSpec::enum_type(
                "variant",
                "Chip style variant",
                vec![
                    "assist".into(),
                    "filter".into(),
                    "input".into(),
                    "suggestion".into(),
                ],
            )
            .with_default(serde_json::json!("filter")),
        )
        .prop(PropSpec::string("icon", "Leading icon name"))
        .prop(PropSpec::string("avatar", "Avatar image URL (for input chips)"))
        .prop(PropSpec::bool("selected", "Whether the chip is selected (filter chips)"))
        .prop(PropSpec::bool("elevated", "Use elevated style"))
        .prop(PropSpec::bool("disabled", "Whether the chip is disabled"))
        .prop(PropSpec::bool("closable", "Show close button (input chips)"))
        .prop(PropSpec::string("value", "Value identifier"))
        .event(EventSpec::new("on_click", "Fired when chip is clicked"))
        .event(EventSpec::new("on_close", "Fired when close button is clicked"))
        .event(EventSpec::new("on_select", "Fired when selection changes (filter chips)"))
        .style_token("color.surface-container-low")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .style_token("shape.corner.small")
        .style_token("elevation.level1")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Activate chip".into(),
                },
                KeyboardBehavior {
                    key: "Backspace/Delete".into(),
                    action: "Remove chip (if closable)".into(),
                },
            ],
            required_aria: vec![],
        })
        .variant(VariantSpec {
            name: "assist".into(),
            description: "Assist chip for smart actions".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "filter".into(),
            description: "Filter chip for selections".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "input".into(),
            description: "Input chip representing user input".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "suggestion".into(),
            description: "Suggestion chip for quick actions".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Filter Chip".into(),
            description: Some("Selectable filter chip".into()),
            code: r#"Chip {
    variant: "filter"
    label: "Vegetarian"
    selected: filters.vegetarian
    on_select: toggle_vegetarian
}"#.into(),
        })
        .example(UsageExample {
            title: "Input Chip".into(),
            description: Some("Removable input chip".into()),
            code: r#"Chip {
    variant: "input"
    label: email
    closable: true
    on_close: remove_email
}"#.into(),
        })
        .build()
}

/// Returns all selection control component specifications
pub fn all_selection_specs() -> Vec<ComponentSpec> {
    vec![
        checkbox_spec(),
        radio_button_spec(),
        radio_group_spec(),
        switch_spec(),
        slider_spec(),
        range_slider_spec(),
        chip_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checkbox_spec() {
        let spec = checkbox_spec();
        assert_eq!(spec.id, "m3.Checkbox");
        assert_eq!(spec.pack, "m3.selection");
        assert!(spec.get_prop("checked").is_some());
        assert!(spec.get_prop("indeterminate").is_some());
        assert!(spec.get_event("on_change").is_some());
    }

    #[test]
    fn test_checkbox_has_indeterminate_variant() {
        let spec = checkbox_spec();
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"indeterminate"));
    }

    #[test]
    fn test_radio_group_slot() {
        let spec = radio_group_spec();
        let slot = spec.default_slot().unwrap();
        assert!(slot.allowed_children.contains(&"m3.RadioButton".into()));
        assert_eq!(slot.min_children, 2);
    }

    #[test]
    fn test_switch_accessibility() {
        let spec = switch_spec();
        assert_eq!(spec.accessibility.role, Some("switch".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-checked".into()));
    }

    #[test]
    fn test_slider_accessibility() {
        let spec = slider_spec();
        assert_eq!(spec.accessibility.role, Some("slider".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-valuemin".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-valuemax".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-valuenow".into()));
    }

    #[test]
    fn test_range_slider_props() {
        let spec = range_slider_spec();
        assert!(spec.get_prop("value_start").is_some());
        assert!(spec.get_prop("value_end").is_some());
        assert!(spec.get_prop("min_distance").is_some());
    }

    #[test]
    fn test_chip_variants() {
        let spec = chip_spec();
        assert_eq!(spec.variants.len(), 4);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"assist"));
        assert!(variant_names.contains(&"filter"));
        assert!(variant_names.contains(&"input"));
        assert!(variant_names.contains(&"suggestion"));
    }

    #[test]
    fn test_all_selection_specs_count() {
        let specs = all_selection_specs();
        assert_eq!(specs.len(), 7);
    }
}
