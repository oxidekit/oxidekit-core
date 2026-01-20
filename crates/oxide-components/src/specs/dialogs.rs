//! Material Design 3 Dialogs & Overlays Component Specifications
//!
//! Includes: AlertDialog, FullScreenDialog, BottomSheet, Snackbar, Menu, DropdownMenu

use crate::spec::{
    AccessibilitySpec, ComponentSpec, EventSpec, KeyboardBehavior, PropSpec, SlotSpec,
    UsageExample, VariantSpec,
};
use std::collections::HashMap;

/// Creates the AlertDialog (basic dialog) component specification
///
/// A dialog for confirming actions or displaying important information.
/// M3 Reference: https://m3.material.io/components/dialogs/specs
pub fn alert_dialog_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.AlertDialog", "m3.containment")
        .name("AlertDialog")
        .description("A dialog for confirming actions or displaying important information")
        .version("1.0.0")
        .prop(PropSpec::bool("open", "Whether the dialog is visible").required())
        .prop(PropSpec::string("title", "Dialog headline text"))
        .prop(PropSpec::string("supporting_text", "Supporting body text"))
        .prop(PropSpec::string("icon", "Optional hero icon at top of dialog"))
        .prop(PropSpec::bool("dismissable", "Whether clicking scrim dismisses dialog").with_default(serde_json::json!(true)))
        .slot(SlotSpec::named("icon", "Custom icon content"))
        .slot(SlotSpec::named("headline", "Custom headline content"))
        .slot(SlotSpec::named("content", "Custom body content"))
        .slot(SlotSpec::named("actions", "Action buttons (typically 1-2)"))
        .event(EventSpec::new("on_dismiss", "Fired when dialog is dismissed"))
        .event(EventSpec::new("on_confirm", "Fired when confirm action is triggered"))
        .event(EventSpec::new("on_cancel", "Fired when cancel action is triggered"))
        .style_token("color.surface-container-high")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .style_token("color.scrim")
        .style_token("shape.corner.extra-large")
        .style_token("elevation.level3")
        .accessibility(AccessibilitySpec {
            role: Some("alertdialog".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Dismiss dialog (if dismissable)".into(),
                },
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Cycle focus within dialog".into(),
                },
                KeyboardBehavior {
                    key: "Enter".into(),
                    action: "Activate focused button".into(),
                },
            ],
            required_aria: vec![
                "aria-modal".into(),
                "aria-labelledby".into(),
                "aria-describedby".into(),
            ],
        })
        .example(UsageExample {
            title: "Basic Alert Dialog".into(),
            description: Some("Confirmation dialog".into()),
            code: r#"AlertDialog {
    open: show_dialog
    title: "Discard draft?"
    supporting_text: "Your changes will not be saved."
    on_dismiss: close_dialog
} {
    slot(actions) {
        TextButton { label: "Cancel" on_click: close_dialog }
        FilledButton { label: "Discard" on_click: discard_draft }
    }
}"#.into(),
        })
        .example(UsageExample {
            title: "Dialog with Icon".into(),
            description: Some("Alert with prominent icon".into()),
            code: r#"AlertDialog {
    open: show_error
    icon: "error"
    title: "Something went wrong"
    supporting_text: "Unable to complete the request. Please try again."
    on_dismiss: close_error
} {
    slot(actions) {
        FilledButton { label: "Try Again" on_click: retry }
    }
}"#.into(),
        })
        .build()
}

/// Creates the FullScreenDialog component specification
///
/// A full-screen dialog for complex tasks.
/// M3 Reference: https://m3.material.io/components/dialogs/specs#a01f2a47-1b18-4e73-9e5e-0018c8a9c586
pub fn full_screen_dialog_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.FullScreenDialog", "m3.containment")
        .name("FullScreenDialog")
        .description("A full-screen dialog for complex tasks requiring focus")
        .version("1.0.0")
        .prop(PropSpec::bool("open", "Whether the dialog is visible").required())
        .prop(PropSpec::string("title", "Dialog title in app bar"))
        .prop(PropSpec::string("close_icon", "Close/back icon").with_default(serde_json::json!("close")))
        .prop(PropSpec::string("action_label", "Action button label (e.g., 'Save')"))
        .prop(PropSpec::bool("action_disabled", "Whether the action button is disabled"))
        .slot(SlotSpec::named("header", "Custom app bar content"))
        .slot(SlotSpec::default_slot())
        .event(EventSpec::new("on_close", "Fired when close icon is clicked"))
        .event(EventSpec::new("on_action", "Fired when action button is clicked"))
        .style_token("color.surface")
        .style_token("color.on-surface")
        .style_token("color.primary")
        .accessibility(AccessibilitySpec {
            role: Some("dialog".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Close dialog".into(),
                },
                KeyboardBehavior {
                    key: "Tab".into(),
                    action: "Navigate within dialog".into(),
                },
            ],
            required_aria: vec!["aria-modal".into(), "aria-labelledby".into()],
        })
        .example(UsageExample {
            title: "Full Screen Dialog".into(),
            description: Some("Edit form in full-screen dialog".into()),
            code: r#"FullScreenDialog {
    open: show_editor
    title: "New Event"
    action_label: "Save"
    action_disabled: !form_valid
    on_close: close_editor
    on_action: save_event
} {
    EventForm { on_change: update_form }
}"#.into(),
        })
        .build()
}

/// Creates the BottomSheet component specification
///
/// A sheet anchored to the bottom of the screen.
/// M3 Reference: https://m3.material.io/components/bottom-sheets/specs
pub fn bottom_sheet_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.BottomSheet", "m3.containment")
        .name("BottomSheet")
        .description("A sheet anchored to the bottom of the screen for supplementary content")
        .version("1.0.0")
        .prop(PropSpec::bool("open", "Whether the sheet is visible").required())
        .prop(
            PropSpec::enum_type(
                "variant",
                "Bottom sheet type",
                vec!["standard".into(), "modal".into()],
            )
            .with_default(serde_json::json!("modal")),
        )
        .prop(PropSpec::bool("show_drag_handle", "Show drag handle indicator").with_default(serde_json::json!(true)))
        .prop(PropSpec::bool("dismissable", "Whether sheet can be dismissed by swiping down").with_default(serde_json::json!(true)))
        .prop(PropSpec::number("initial_height", "Initial height percentage (0-1)").with_default(serde_json::json!(0.5)))
        .prop(PropSpec::number("min_height", "Minimum height percentage").with_default(serde_json::json!(0.25)))
        .prop(PropSpec::number("max_height", "Maximum height percentage").with_default(serde_json::json!(0.9)))
        .prop(PropSpec::bool("full_height", "Expand to full height"))
        .slot(SlotSpec::default_slot())
        .event(EventSpec::new("on_dismiss", "Fired when sheet is dismissed"))
        .event(EventSpec::new("on_height_change", "Fired when sheet height changes"))
        .style_token("color.surface-container-low")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.outline")
        .style_token("color.scrim")
        .style_token("shape.corner.extra-large-top")
        .style_token("elevation.level1")
        .accessibility(AccessibilitySpec {
            role: Some("dialog".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Dismiss sheet (if dismissable)".into(),
                },
            ],
            required_aria: vec!["aria-modal".into()],
        })
        .variant(VariantSpec {
            name: "standard".into(),
            description: "Non-modal sheet that coexists with main content".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "modal".into(),
            description: "Modal sheet with scrim overlay".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Modal Bottom Sheet".into(),
            description: Some("Options sheet".into()),
            code: r#"BottomSheet {
    variant: "modal"
    open: show_options
    on_dismiss: close_options
} {
    ListTile { headline: "Share" leading_icon: "share" on_click: share }
    ListTile { headline: "Copy link" leading_icon: "link" on_click: copy_link }
    ListTile { headline: "Delete" leading_icon: "delete" on_click: delete }
}"#.into(),
        })
        .build()
}

/// Creates the Snackbar component specification
///
/// A brief message that appears at the bottom of the screen.
/// M3 Reference: https://m3.material.io/components/snackbar/specs
pub fn snackbar_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Snackbar", "m3.communication")
        .name("Snackbar")
        .description("A brief message that provides feedback about an operation")
        .version("1.0.0")
        .prop(PropSpec::string("message", "Snackbar message text").required())
        .prop(PropSpec::string("action_label", "Optional action button label"))
        .prop(PropSpec::bool("show_close_icon", "Show close/dismiss icon"))
        .prop(PropSpec::number("duration", "Auto-dismiss duration in ms (0 for indefinite)").with_default(serde_json::json!(4000)))
        .prop(PropSpec::bool("multi_line", "Allow multiple lines of text"))
        .prop(PropSpec::bool("long_action", "Place action button below message for long text"))
        .event(EventSpec::new("on_action", "Fired when action button is clicked"))
        .event(EventSpec::new("on_dismiss", "Fired when snackbar is dismissed"))
        .style_token("color.inverse-surface")
        .style_token("color.inverse-on-surface")
        .style_token("color.inverse-primary")
        .style_token("shape.corner.extra-small")
        .accessibility(AccessibilitySpec {
            role: Some("status".into()),
            focusable: false,
            keyboard: vec![],
            required_aria: vec!["aria-live".into()],
        })
        .example(UsageExample {
            title: "Basic Snackbar".into(),
            description: Some("Simple feedback message".into()),
            code: r#"Snackbar {
    message: "Item added to cart"
    duration: 4000
}"#.into(),
        })
        .example(UsageExample {
            title: "Snackbar with Action".into(),
            description: Some("Message with undo action".into()),
            code: r#"Snackbar {
    message: "Email archived"
    action_label: "Undo"
    on_action: undo_archive
    on_dismiss: clear_snackbar
}"#.into(),
        })
        .example(UsageExample {
            title: "Long Snackbar".into(),
            description: Some("Multi-line message with close".into()),
            code: r#"Snackbar {
    message: "Unable to connect to the server. Please check your internet connection and try again."
    multi_line: true
    show_close_icon: true
    duration: 0
    on_dismiss: dismiss_error
}"#.into(),
        })
        .build()
}

/// Creates the Menu component specification
///
/// A temporary surface for displaying a list of choices.
/// M3 Reference: https://m3.material.io/components/menus/specs
pub fn menu_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Menu", "m3.containment")
        .name("Menu")
        .description("A temporary surface displaying a list of choices")
        .version("1.0.0")
        .prop(PropSpec::bool("open", "Whether the menu is visible"))
        .prop(
            PropSpec::enum_type(
                "anchor",
                "Menu anchor position relative to trigger",
                vec![
                    "top-start".into(),
                    "top-end".into(),
                    "bottom-start".into(),
                    "bottom-end".into(),
                ],
            )
            .with_default(serde_json::json!("bottom-start")),
        )
        .prop(PropSpec::number("min_width", "Minimum menu width in dp"))
        .prop(PropSpec::number("max_width", "Maximum menu width in dp"))
        .prop(PropSpec::bool("quick_select", "Close immediately on selection").with_default(serde_json::json!(true)))
        .slot(SlotSpec::named("trigger", "Element that triggers the menu").required())
        .slot(
            SlotSpec::default_slot()
                .allow("m3.MenuItem")
                .allow("m3.MenuDivider")
                .allow("m3.SubMenu"),
        )
        .event(EventSpec::new("on_open", "Fired when menu opens"))
        .event(EventSpec::new("on_close", "Fired when menu closes"))
        .style_token("color.surface-container")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("shape.corner.extra-small")
        .style_token("elevation.level2")
        .accessibility(AccessibilitySpec {
            role: Some("menu".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowDown".into(),
                    action: "Move to next item".into(),
                },
                KeyboardBehavior {
                    key: "ArrowUp".into(),
                    action: "Move to previous item".into(),
                },
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select focused item".into(),
                },
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Close menu".into(),
                },
                KeyboardBehavior {
                    key: "ArrowRight".into(),
                    action: "Open submenu".into(),
                },
                KeyboardBehavior {
                    key: "ArrowLeft".into(),
                    action: "Close submenu".into(),
                },
            ],
            required_aria: vec!["aria-expanded".into()],
        })
        .example(UsageExample {
            title: "Basic Menu".into(),
            description: Some("Simple dropdown menu".into()),
            code: r#"Menu {
    open: menu_open
    on_close: close_menu
} {
    slot(trigger) {
        IconButton { icon: "more_vert" aria_label: "Options" on_click: toggle_menu }
    }
    MenuItem { label: "Edit" icon: "edit" on_click: edit_item }
    MenuItem { label: "Duplicate" icon: "content_copy" on_click: duplicate_item }
    MenuDivider {}
    MenuItem { label: "Delete" icon: "delete" destructive: true on_click: delete_item }
}"#.into(),
        })
        .build()
}

/// Creates the MenuItem component specification
pub fn menu_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.MenuItem", "m3.containment")
        .name("MenuItem")
        .description("An individual item within a Menu")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Menu item text").required())
        .prop(PropSpec::string("icon", "Leading icon name"))
        .prop(PropSpec::string("trailing_icon", "Trailing icon name"))
        .prop(PropSpec::string("trailing_text", "Trailing text (e.g., shortcut)"))
        .prop(PropSpec::bool("disabled", "Whether the item is disabled"))
        .prop(PropSpec::bool("destructive", "Whether this is a destructive action"))
        .prop(PropSpec::bool("selected", "Whether this item is selected (for selection menus)"))
        .prop(PropSpec::string("value", "Value identifier"))
        .event(EventSpec::new("on_click", "Fired when item is clicked"))
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.error")
        .style_token("state.hover")
        .style_token("state.focused")
        .accessibility(AccessibilitySpec {
            role: Some("menuitem".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec![],
        })
        .build()
}

/// Creates the MenuDivider component specification
pub fn menu_divider_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.MenuDivider", "m3.containment")
        .name("MenuDivider")
        .description("A visual separator within a Menu")
        .version("1.0.0")
        .style_token("color.outline-variant")
        .accessibility(AccessibilitySpec {
            role: Some("separator".into()),
            focusable: false,
            keyboard: vec![],
            required_aria: vec![],
        })
        .build()
}

/// Creates the SubMenu component specification
pub fn sub_menu_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.SubMenu", "m3.containment")
        .name("SubMenu")
        .description("A nested menu within a Menu")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Submenu trigger label").required())
        .prop(PropSpec::string("icon", "Leading icon name"))
        .prop(PropSpec::bool("disabled", "Whether the submenu is disabled"))
        .slot(
            SlotSpec::default_slot()
                .allow("m3.MenuItem")
                .allow("m3.MenuDivider")
                .allow("m3.SubMenu"),
        )
        .accessibility(AccessibilitySpec {
            role: Some("menuitem".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowRight".into(),
                    action: "Open submenu".into(),
                },
            ],
            required_aria: vec!["aria-haspopup".into(), "aria-expanded".into()],
        })
        .build()
}

/// Creates the DropdownMenu (exposed dropdown) component specification
///
/// A menu that replaces the triggering element with the selected value.
pub fn dropdown_menu_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.DropdownMenu", "m3.containment")
        .name("DropdownMenu")
        .description("A dropdown menu that displays the selected option in a text field")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Currently selected value"))
        .prop(PropSpec::string("label", "Label for the dropdown"))
        .prop(PropSpec::string("placeholder", "Placeholder text when no selection"))
        .prop(
            PropSpec::enum_type(
                "variant",
                "Dropdown style variant",
                vec!["filled".into(), "outlined".into()],
            )
            .with_default(serde_json::json!("filled")),
        )
        .prop(PropSpec::bool("disabled", "Whether the dropdown is disabled"))
        .prop(PropSpec::bool("error", "Whether the dropdown is in error state"))
        .prop(PropSpec::string("error_text", "Error message to display"))
        .prop(PropSpec::string("helper_text", "Helper text below dropdown"))
        .prop(PropSpec::bool("required", "Whether a selection is required"))
        .slot(SlotSpec::default_slot().allow("m3.DropdownMenuItem"))
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .event(EventSpec::new("on_open", "Fired when dropdown opens"))
        .event(EventSpec::new("on_close", "Fired when dropdown closes"))
        .style_token("color.surface-container-highest")
        .style_token("color.outline")
        .style_token("color.on-surface")
        .style_token("color.primary")
        .style_token("color.error")
        .style_token("shape.corner.extra-small")
        .accessibility(AccessibilitySpec {
            role: Some("combobox".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowDown".into(),
                    action: "Open dropdown / move to next option".into(),
                },
                KeyboardBehavior {
                    key: "ArrowUp".into(),
                    action: "Move to previous option".into(),
                },
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select highlighted option".into(),
                },
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Close dropdown".into(),
                },
                KeyboardBehavior {
                    key: "Home".into(),
                    action: "Go to first option".into(),
                },
                KeyboardBehavior {
                    key: "End".into(),
                    action: "Go to last option".into(),
                },
            ],
            required_aria: vec!["aria-expanded".into(), "aria-haspopup".into()],
        })
        .example(UsageExample {
            title: "Dropdown Menu".into(),
            description: Some("Select from options".into()),
            code: r#"DropdownMenu {
    label: "Country"
    value: selected_country
    on_change: set_selected_country
} {
    DropdownMenuItem { value: "us" label: "United States" }
    DropdownMenuItem { value: "uk" label: "United Kingdom" }
    DropdownMenuItem { value: "ca" label: "Canada" }
}"#.into(),
        })
        .build()
}

/// Creates the DropdownMenuItem component specification
pub fn dropdown_menu_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.DropdownMenuItem", "m3.containment")
        .name("DropdownMenuItem")
        .description("An individual option within a DropdownMenu")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this option").required())
        .prop(PropSpec::string("label", "Display text for this option").required())
        .prop(PropSpec::string("icon", "Leading icon name"))
        .prop(PropSpec::bool("disabled", "Whether this option is disabled"))
        .accessibility(AccessibilitySpec {
            role: Some("option".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec!["aria-selected".into()],
        })
        .build()
}

/// Creates the Tooltip component specification
///
/// A brief, informative message that appears when users hover over or focus on an element.
/// M3 Reference: https://m3.material.io/components/tooltips/specs
pub fn tooltip_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Tooltip", "m3.communication")
        .name("Tooltip")
        .description("A brief message that appears on hover or focus to describe an element")
        .version("1.0.0")
        .prop(PropSpec::string("text", "Tooltip text content").required())
        .prop(
            PropSpec::enum_type(
                "variant",
                "Tooltip style variant",
                vec!["plain".into(), "rich".into()],
            )
            .with_default(serde_json::json!("plain")),
        )
        .prop(PropSpec::string("subhead", "Subhead for rich tooltips"))
        .prop(PropSpec::string("action_label", "Action button label (rich tooltips)"))
        .prop(
            PropSpec::enum_type(
                "placement",
                "Tooltip placement",
                vec![
                    "top".into(),
                    "bottom".into(),
                    "left".into(),
                    "right".into(),
                ],
            )
            .with_default(serde_json::json!("top")),
        )
        .prop(PropSpec::number("enter_delay", "Delay before showing (ms)").with_default(serde_json::json!(500)))
        .prop(PropSpec::number("leave_delay", "Delay before hiding (ms)").with_default(serde_json::json!(0)))
        .prop(PropSpec::bool("persistent", "Keep visible until dismissed (rich tooltips)"))
        .slot(SlotSpec::default_slot())
        .event(EventSpec::new("on_action", "Fired when action button is clicked (rich tooltips)"))
        .style_token("color.inverse-surface")
        .style_token("color.inverse-on-surface")
        .style_token("color.surface-container")
        .style_token("color.on-surface-variant")
        .style_token("shape.corner.extra-small")
        .accessibility(AccessibilitySpec {
            role: Some("tooltip".into()),
            focusable: false,
            keyboard: vec![],
            required_aria: vec!["aria-describedby".into()],
        })
        .variant(VariantSpec {
            name: "plain".into(),
            description: "Simple text tooltip".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "rich".into(),
            description: "Detailed tooltip with optional action".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Plain Tooltip".into(),
            description: Some("Simple text tooltip".into()),
            code: r#"Tooltip { text: "Add to favorites" } {
    IconButton { icon: "favorite_border" aria_label: "Add to favorites" }
}"#.into(),
        })
        .example(UsageExample {
            title: "Rich Tooltip".into(),
            description: Some("Detailed tooltip with action".into()),
            code: r#"Tooltip {
    variant: "rich"
    subhead: "Grant Camera Access"
    text: "Allow app to use your camera for video calls"
    action_label: "Learn more"
    on_action: show_camera_info
} {
    IconButton { icon: "videocam" aria_label: "Start video" }
}"#.into(),
        })
        .build()
}

/// Returns all dialog and overlay component specifications
pub fn all_dialog_specs() -> Vec<ComponentSpec> {
    vec![
        alert_dialog_spec(),
        full_screen_dialog_spec(),
        bottom_sheet_spec(),
        snackbar_spec(),
        menu_spec(),
        menu_item_spec(),
        menu_divider_spec(),
        sub_menu_spec(),
        dropdown_menu_spec(),
        dropdown_menu_item_spec(),
        tooltip_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_dialog_spec() {
        let spec = alert_dialog_spec();
        assert_eq!(spec.id, "m3.AlertDialog");
        assert_eq!(spec.pack, "m3.containment");
        assert!(spec.get_prop("open").unwrap().required);
        assert_eq!(spec.accessibility.role, Some("alertdialog".into()));
    }

    #[test]
    fn test_alert_dialog_accessibility() {
        let spec = alert_dialog_spec();
        assert!(spec.accessibility.required_aria.contains(&"aria-modal".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-labelledby".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-describedby".into()));
    }

    #[test]
    fn test_bottom_sheet_variants() {
        let spec = bottom_sheet_spec();
        assert_eq!(spec.variants.len(), 2);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"standard"));
        assert!(variant_names.contains(&"modal"));
    }

    #[test]
    fn test_snackbar_spec() {
        let spec = snackbar_spec();
        assert_eq!(spec.id, "m3.Snackbar");
        assert_eq!(spec.pack, "m3.communication");
        assert!(spec.get_prop("message").unwrap().required);
        assert_eq!(spec.accessibility.role, Some("status".into()));
    }

    #[test]
    fn test_menu_accessibility() {
        let spec = menu_spec();
        assert_eq!(spec.accessibility.role, Some("menu".into()));
        let keyboard_keys: Vec<&str> = spec.accessibility.keyboard.iter().map(|k| k.key.as_str()).collect();
        assert!(keyboard_keys.contains(&"ArrowDown"));
        assert!(keyboard_keys.contains(&"ArrowUp"));
        assert!(keyboard_keys.contains(&"Escape"));
    }

    #[test]
    fn test_dropdown_menu_accessibility() {
        let spec = dropdown_menu_spec();
        assert_eq!(spec.accessibility.role, Some("combobox".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-expanded".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-haspopup".into()));
    }

    #[test]
    fn test_tooltip_variants() {
        let spec = tooltip_spec();
        assert_eq!(spec.variants.len(), 2);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"plain"));
        assert!(variant_names.contains(&"rich"));
    }

    #[test]
    fn test_all_dialog_specs_count() {
        let specs = all_dialog_specs();
        assert_eq!(specs.len(), 11);
    }
}
