//! Material Design 3 Button Component Specifications
//!
//! Includes: FilledButton, OutlinedButton, TextButton, IconButton,
//! FloatingActionButton, ExtendedFAB, and SegmentedButton

use crate::spec::{
    AccessibilitySpec, ComponentSpec, EventSpec, KeyboardBehavior, PropSpec, SlotSpec,
    UsageExample, VariantSpec,
};
use std::collections::HashMap;

/// Creates the FilledButton component specification
///
/// A filled button with a solid background color, used for primary actions.
/// M3 Reference: https://m3.material.io/components/buttons/specs#0b1b7bd2-3de8-431a-afa1-d692e2e18b0d
pub fn filled_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.FilledButton", "m3.action")
        .name("FilledButton")
        .description("A high-emphasis button with a solid container color for primary actions")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Button text content").required())
        .prop(
            PropSpec::enum_type(
                "size",
                "Button size following M3 specifications",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
        .prop(PropSpec::string("icon", "Material icon name to display before label"))
        .prop(PropSpec::bool("loading", "Show loading indicator and disable interaction"))
        .prop(
            PropSpec::enum_type(
                "icon_position",
                "Position of icon relative to label",
                vec!["start".into(), "end".into()],
            )
            .with_default(serde_json::json!("start")),
        )
        .event(EventSpec::new("on_click", "Fired when button is clicked"))
        .event(EventSpec::new("on_focus", "Fired when button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when button loses focus"))
        .event(EventSpec::new("on_long_press", "Fired on long press (touch devices)"))
        .style_token("color.primary")
        .style_token("color.on-primary")
        .style_token("shape.corner.full")
        .style_token("state.hover")
        .style_token("state.pressed")
        .style_token("state.disabled")
        .style_token("elevation.level0")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate button".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate button".into(),
                },
            ],
            required_aria: vec![],
        })
        .example(UsageExample {
            title: "Basic Filled Button".into(),
            description: Some("A primary action button".into()),
            code: r#"FilledButton { label: "Submit" on_click: handle_submit }"#.into(),
        })
        .example(UsageExample {
            title: "Filled Button with Icon".into(),
            description: Some("Button with leading icon".into()),
            code: r#"FilledButton { label: "Add Item" icon: "add" on_click: handle_add }"#.into(),
        })
        .build()
}

/// Creates the FilledTonalButton component specification
///
/// A filled button with a tonal (secondary) container color.
/// M3 Reference: https://m3.material.io/components/buttons/specs#158f0a18-067a-4f96-8fe5-f19dda7ef20f
pub fn filled_tonal_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.FilledTonalButton", "m3.action")
        .name("FilledTonalButton")
        .description("A medium-emphasis button with a tonal container for secondary actions")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Button text content").required())
        .prop(
            PropSpec::enum_type(
                "size",
                "Button size following M3 specifications",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
        .prop(PropSpec::string("icon", "Material icon name to display before label"))
        .prop(PropSpec::bool("loading", "Show loading indicator and disable interaction"))
        .event(EventSpec::new("on_click", "Fired when button is clicked"))
        .event(EventSpec::new("on_focus", "Fired when button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when button loses focus"))
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .style_token("shape.corner.full")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate button".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate button".into(),
                },
            ],
            required_aria: vec![],
        })
        .example(UsageExample {
            title: "Filled Tonal Button".into(),
            description: Some("A secondary action button".into()),
            code: r#"FilledTonalButton { label: "Save Draft" on_click: handle_save }"#.into(),
        })
        .build()
}

/// Creates the OutlinedButton component specification
///
/// A button with an outline border and no fill.
/// M3 Reference: https://m3.material.io/components/buttons/specs#de72d8b1-ba16-4cd7-989e-e2ad3293cf63
pub fn outlined_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.OutlinedButton", "m3.action")
        .name("OutlinedButton")
        .description("A medium-emphasis button with outline stroke for secondary actions")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Button text content").required())
        .prop(
            PropSpec::enum_type(
                "size",
                "Button size following M3 specifications",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
        .prop(PropSpec::string("icon", "Material icon name to display before label"))
        .prop(PropSpec::bool("loading", "Show loading indicator and disable interaction"))
        .event(EventSpec::new("on_click", "Fired when button is clicked"))
        .event(EventSpec::new("on_focus", "Fired when button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when button loses focus"))
        .style_token("color.outline")
        .style_token("color.primary")
        .style_token("shape.corner.full")
        .style_token("state.hover")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate button".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate button".into(),
                },
            ],
            required_aria: vec![],
        })
        .example(UsageExample {
            title: "Outlined Button".into(),
            description: Some("A secondary action with outline".into()),
            code: r#"OutlinedButton { label: "Cancel" on_click: handle_cancel }"#.into(),
        })
        .build()
}

/// Creates the TextButton component specification
///
/// A button with no container, just text.
/// M3 Reference: https://m3.material.io/components/buttons/specs#899b9107-0127-4a01-8f4c-87f19323a1b4
pub fn text_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.TextButton", "m3.action")
        .name("TextButton")
        .description("A low-emphasis button for less prominent actions")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Button text content").required())
        .prop(
            PropSpec::enum_type(
                "size",
                "Button size following M3 specifications",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
        .prop(PropSpec::string("icon", "Material icon name to display before label"))
        .event(EventSpec::new("on_click", "Fired when button is clicked"))
        .event(EventSpec::new("on_focus", "Fired when button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when button loses focus"))
        .style_token("color.primary")
        .style_token("state.hover")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate button".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate button".into(),
                },
            ],
            required_aria: vec![],
        })
        .example(UsageExample {
            title: "Text Button".into(),
            description: Some("A minimal emphasis button".into()),
            code: r#"TextButton { label: "Learn more" on_click: handle_learn_more }"#.into(),
        })
        .build()
}

/// Creates the IconButton component specification
///
/// A button that displays only an icon.
/// M3 Reference: https://m3.material.io/components/icon-buttons/specs
pub fn icon_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.IconButton", "m3.action")
        .name("IconButton")
        .description("A button that displays only an icon for compact actions")
        .version("1.0.0")
        .prop(PropSpec::string("icon", "Material icon name to display").required())
        .prop(
            PropSpec::string("aria_label", "Accessible label for screen readers (required for accessibility)").required(),
        )
        .prop(
            PropSpec::enum_type(
                "variant",
                "Icon button style variant",
                vec![
                    "standard".into(),
                    "filled".into(),
                    "filled-tonal".into(),
                    "outlined".into(),
                ],
            )
            .with_default(serde_json::json!("standard")),
        )
        .prop(
            PropSpec::enum_type(
                "size",
                "Icon button size",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
        .prop(PropSpec::bool("selected", "Whether the button is in selected/toggled state"))
        .prop(PropSpec::bool("toggle", "Enable toggle behavior (selected state changes on click)"))
        .event(EventSpec::new("on_click", "Fired when button is clicked"))
        .event(EventSpec::new("on_toggle", "Fired when toggle state changes"))
        .event(EventSpec::new("on_focus", "Fired when button receives focus"))
        .event(EventSpec::new("on_blur", "Fired when button loses focus"))
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .style_token("state.hover")
        .style_token("state.pressed")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate button".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate button".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .variant(VariantSpec {
            name: "standard".into(),
            description: "Icon with no container, subtle hover state".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "filled".into(),
            description: "Icon with filled container background".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "filled-tonal".into(),
            description: "Icon with tonal container background".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "outlined".into(),
            description: "Icon with outline border".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Standard Icon Button".into(),
            description: Some("Basic icon button for actions".into()),
            code: r#"IconButton { icon: "favorite" aria_label: "Add to favorites" on_click: handle_favorite }"#.into(),
        })
        .example(UsageExample {
            title: "Toggle Icon Button".into(),
            description: Some("Icon button with toggle state".into()),
            code: r#"IconButton { icon: "bookmark" aria_label: "Bookmark" toggle: true selected: is_bookmarked on_toggle: handle_bookmark }"#.into(),
        })
        .build()
}

/// Creates the FloatingActionButton (FAB) component specification
///
/// A floating action button for the primary action on a screen.
/// M3 Reference: https://m3.material.io/components/floating-action-button/specs
pub fn fab_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.FAB", "m3.action")
        .name("FAB")
        .description("A floating action button for the primary action on a screen")
        .version("1.0.0")
        .prop(PropSpec::string("icon", "Material icon name to display").required())
        .prop(PropSpec::string("aria_label", "Accessible label for screen readers").required())
        .prop(
            PropSpec::enum_type(
                "size",
                "FAB size variant",
                vec!["small".into(), "medium".into(), "large".into()],
            )
            .with_default(serde_json::json!("medium")),
        )
        .prop(
            PropSpec::enum_type(
                "color",
                "FAB color scheme",
                vec![
                    "primary".into(),
                    "secondary".into(),
                    "tertiary".into(),
                    "surface".into(),
                ],
            )
            .with_default(serde_json::json!("primary")),
        )
        .prop(PropSpec::bool("lowered", "Use lowered elevation (shadow)"))
        .event(EventSpec::new("on_click", "Fired when FAB is clicked"))
        .event(EventSpec::new("on_focus", "Fired when FAB receives focus"))
        .event(EventSpec::new("on_blur", "Fired when FAB loses focus"))
        .style_token("color.primary-container")
        .style_token("color.on-primary-container")
        .style_token("elevation.level3")
        .style_token("shape.corner.large")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate FAB".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate FAB".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Standard FAB".into(),
            description: Some("Primary floating action button".into()),
            code: r#"FAB { icon: "add" aria_label: "Create new item" on_click: handle_create }"#
                .into(),
        })
        .example(UsageExample {
            title: "Small FAB".into(),
            description: Some("Compact floating action button".into()),
            code: r#"FAB { icon: "edit" size: "small" aria_label: "Edit" on_click: handle_edit }"#
                .into(),
        })
        .build()
}

/// Creates the ExtendedFAB component specification
///
/// A floating action button with text label.
/// M3 Reference: https://m3.material.io/components/extended-fab/specs
pub fn extended_fab_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.ExtendedFAB", "m3.action")
        .name("ExtendedFAB")
        .description("A floating action button with an extended text label")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Text label for the FAB").required())
        .prop(PropSpec::string("icon", "Material icon name (optional but recommended)"))
        .prop(
            PropSpec::enum_type(
                "color",
                "FAB color scheme",
                vec![
                    "primary".into(),
                    "secondary".into(),
                    "tertiary".into(),
                    "surface".into(),
                ],
            )
            .with_default(serde_json::json!("primary")),
        )
        .prop(PropSpec::bool("lowered", "Use lowered elevation (shadow)"))
        .prop(PropSpec::bool("expanded", "Whether the FAB is in expanded state (showing label)").with_default(serde_json::json!(true)))
        .event(EventSpec::new("on_click", "Fired when FAB is clicked"))
        .event(EventSpec::new("on_focus", "Fired when FAB receives focus"))
        .event(EventSpec::new("on_blur", "Fired when FAB loses focus"))
        .style_token("color.primary-container")
        .style_token("color.on-primary-container")
        .style_token("elevation.level3")
        .style_token("shape.corner.large")
        .accessibility(AccessibilitySpec {
            role: Some("button".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate FAB".into(),
                },
                KeyboardBehavior {
                    key: "Space".into(),
                    action: "Activate FAB".into(),
                },
            ],
            required_aria: vec![],
        })
        .example(UsageExample {
            title: "Extended FAB".into(),
            description: Some("FAB with icon and label".into()),
            code: r#"ExtendedFAB { icon: "edit" label: "Compose" on_click: handle_compose }"#
                .into(),
        })
        .build()
}

/// Creates the SegmentedButton component specification
///
/// A segmented button for selecting from a set of options.
/// M3 Reference: https://m3.material.io/components/segmented-buttons/specs
pub fn segmented_button_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.SegmentedButton", "m3.action")
        .name("SegmentedButton")
        .description("A group of buttons for selecting one or more options from a set")
        .version("1.0.0")
        .prop(
            PropSpec::enum_type(
                "selection_mode",
                "How many segments can be selected",
                vec!["single".into(), "multiple".into()],
            )
            .with_default(serde_json::json!("single")),
        )
        .prop(PropSpec::string("value", "Currently selected value(s) - comma-separated for multiple"))
        .prop(PropSpec::bool("disabled", "Whether the entire button group is disabled"))
        .prop(
            PropSpec::enum_type(
                "density",
                "Button density/height",
                vec!["default".into(), "comfortable".into(), "compact".into()],
            )
            .with_default(serde_json::json!("default")),
        )
        .slot(
            SlotSpec::default_slot()
                .allow("m3.SegmentedButtonSegment")
                .min(2)
                .max(5),
        )
        .event(EventSpec::new(
            "on_change",
            "Fired when selection changes, provides selected value(s)",
        ))
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .style_token("color.outline")
        .style_token("shape.corner.full")
        .accessibility(AccessibilitySpec {
            role: Some("group".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowLeft/ArrowRight".into(),
                    action: "Navigate between segments".into(),
                },
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select focused segment".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Single Select Segmented Button".into(),
            description: Some("Choose one option".into()),
            code: r#"SegmentedButton { value: selected_view on_change: handle_view_change } {
    SegmentedButtonSegment { value: "day" label: "Day" }
    SegmentedButtonSegment { value: "week" label: "Week" }
    SegmentedButtonSegment { value: "month" label: "Month" }
}"#
            .into(),
        })
        .build()
}

/// Creates the SegmentedButtonSegment component specification
///
/// An individual segment within a SegmentedButton.
pub fn segmented_button_segment_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.SegmentedButtonSegment", "m3.action")
        .name("SegmentedButtonSegment")
        .description("An individual segment within a SegmentedButton")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this segment").required())
        .prop(PropSpec::string("label", "Text label for the segment"))
        .prop(PropSpec::string("icon", "Material icon name"))
        .prop(PropSpec::bool("disabled", "Whether this segment is disabled"))
        .prop(PropSpec::bool("show_selected_icon", "Show checkmark when selected").with_default(serde_json::json!(true)))
        .accessibility(AccessibilitySpec {
            role: Some("radio".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select segment".into(),
                },
            ],
            required_aria: vec!["aria-checked".into()],
        })
        .build()
}

/// Returns all button component specifications
pub fn all_button_specs() -> Vec<ComponentSpec> {
    vec![
        filled_button_spec(),
        filled_tonal_button_spec(),
        outlined_button_spec(),
        text_button_spec(),
        icon_button_spec(),
        fab_spec(),
        extended_fab_spec(),
        segmented_button_spec(),
        segmented_button_segment_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filled_button_spec() {
        let spec = filled_button_spec();
        assert_eq!(spec.id, "m3.FilledButton");
        assert_eq!(spec.pack, "m3.action");
        assert!(spec.get_prop("label").unwrap().required);
        assert!(spec.get_event("on_click").is_some());
        assert_eq!(spec.accessibility.role, Some("button".into()));
    }

    #[test]
    fn test_icon_button_variants() {
        let spec = icon_button_spec();
        assert_eq!(spec.variants.len(), 4);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"standard"));
        assert!(variant_names.contains(&"filled"));
        assert!(variant_names.contains(&"filled-tonal"));
        assert!(variant_names.contains(&"outlined"));
    }

    #[test]
    fn test_fab_accessibility() {
        let spec = fab_spec();
        assert!(spec.accessibility.focusable);
        assert!(spec.accessibility.required_aria.contains(&"aria-label".into()));
    }

    #[test]
    fn test_segmented_button_slot() {
        let spec = segmented_button_spec();
        let default_slot = spec.default_slot().unwrap();
        assert!(default_slot.allowed_children.contains(&"m3.SegmentedButtonSegment".into()));
        assert_eq!(default_slot.min_children, 2);
        assert_eq!(default_slot.max_children, 5);
    }

    #[test]
    fn test_all_button_specs_count() {
        let specs = all_button_specs();
        assert_eq!(specs.len(), 9);
    }
}
