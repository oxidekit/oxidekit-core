//! Material Design 3 Navigation Component Specifications
//!
//! Includes: NavigationBar, NavigationRail, NavigationDrawer, TopAppBar, TabBar, Tab

use crate::spec::{
    AccessibilitySpec, ComponentSpec, EventSpec, KeyboardBehavior, PropSpec, SlotSpec,
    UsageExample, VariantSpec,
};
use std::collections::HashMap;

/// Creates the NavigationBar (bottom navigation) component specification
///
/// A bottom navigation bar for switching between primary destinations.
/// M3 Reference: https://m3.material.io/components/navigation-bar/specs
pub fn navigation_bar_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationBar", "m3.navigation")
        .name("NavigationBar")
        .description("A bottom navigation bar for switching between 3-5 primary destinations")
        .version("1.0.0")
        .prop(PropSpec::string("selected", "Currently selected destination value"))
        .prop(PropSpec::bool("hide_on_scroll", "Hide navigation bar when scrolling down"))
        .prop(
            PropSpec::enum_type(
                "label_behavior",
                "How labels are displayed",
                vec![
                    "always".into(),
                    "selected".into(),
                    "never".into(),
                ],
            )
            .with_default(serde_json::json!("always")),
        )
        .slot(
            SlotSpec::default_slot()
                .allow("m3.NavigationBarItem")
                .min(3)
                .max(5),
        )
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .style_token("color.surface-container")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .style_token("elevation.level2")
        .accessibility(AccessibilitySpec {
            role: Some("navigation".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowLeft/ArrowRight".into(),
                    action: "Navigate between items".into(),
                },
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select focused item".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Bottom Navigation Bar".into(),
            description: Some("Primary app navigation".into()),
            code: r#"NavigationBar {
    selected: current_tab
    on_change: set_current_tab
} {
    NavigationBarItem { value: "home" icon: "home" label: "Home" }
    NavigationBarItem { value: "search" icon: "search" label: "Search" }
    NavigationBarItem { value: "favorites" icon: "favorite" label: "Favorites" }
    NavigationBarItem { value: "profile" icon: "person" label: "Profile" }
}"#.into(),
        })
        .build()
}

/// Creates the NavigationBarItem component specification
pub fn navigation_bar_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationBarItem", "m3.navigation")
        .name("NavigationBarItem")
        .description("An individual destination within a NavigationBar")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this destination").required())
        .prop(PropSpec::string("icon", "Material icon name for unselected state").required())
        .prop(PropSpec::string("icon_selected", "Material icon name for selected state (defaults to filled version)"))
        .prop(PropSpec::string("label", "Text label for the destination"))
        .prop(PropSpec::bool("disabled", "Whether this item is disabled"))
        .prop(PropSpec::number("badge", "Badge count to display"))
        .prop(PropSpec::bool("badge_dot", "Show badge as dot indicator (no count)"))
        .accessibility(AccessibilitySpec {
            role: Some("tab".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec!["aria-selected".into()],
        })
        .build()
}

/// Creates the NavigationRail component specification
///
/// A side navigation rail for larger screens.
/// M3 Reference: https://m3.material.io/components/navigation-rail/specs
pub fn navigation_rail_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationRail", "m3.navigation")
        .name("NavigationRail")
        .description("A side navigation rail for medium to large screens")
        .version("1.0.0")
        .prop(PropSpec::string("selected", "Currently selected destination value"))
        .prop(
            PropSpec::enum_type(
                "alignment",
                "Vertical alignment of destinations",
                vec!["top".into(), "center".into(), "bottom".into()],
            )
            .with_default(serde_json::json!("top")),
        )
        .prop(
            PropSpec::enum_type(
                "label_type",
                "How labels are displayed",
                vec!["none".into(), "selected".into(), "all".into()],
            )
            .with_default(serde_json::json!("all")),
        )
        .prop(PropSpec::bool("show_fab", "Show floating action button"))
        .prop(PropSpec::string("fab_icon", "Icon for the FAB"))
        .prop(PropSpec::string("menu_icon", "Icon for optional menu button"))
        .slot(SlotSpec::named("header", "Header content (logo, menu button)"))
        .slot(SlotSpec::named("fab", "Custom FAB content"))
        .slot(
            SlotSpec::default_slot()
                .allow("m3.NavigationRailItem")
                .min(3)
                .max(7),
        )
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .event(EventSpec::new("on_fab_click", "Fired when FAB is clicked"))
        .event(EventSpec::new("on_menu_click", "Fired when menu button is clicked"))
        .style_token("color.surface")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .accessibility(AccessibilitySpec {
            role: Some("navigation".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowUp/ArrowDown".into(),
                    action: "Navigate between items".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Navigation Rail".into(),
            description: Some("Side navigation for tablets/desktop".into()),
            code: r#"NavigationRail {
    selected: current_section
    show_fab: true
    fab_icon: "edit"
    on_change: set_current_section
    on_fab_click: handle_compose
} {
    NavigationRailItem { value: "inbox" icon: "inbox" label: "Inbox" badge: unread_count }
    NavigationRailItem { value: "sent" icon: "send" label: "Sent" }
    NavigationRailItem { value: "drafts" icon: "drafts" label: "Drafts" }
}"#.into(),
        })
        .build()
}

/// Creates the NavigationRailItem component specification
pub fn navigation_rail_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationRailItem", "m3.navigation")
        .name("NavigationRailItem")
        .description("An individual destination within a NavigationRail")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this destination").required())
        .prop(PropSpec::string("icon", "Material icon name for unselected state").required())
        .prop(PropSpec::string("icon_selected", "Material icon name for selected state"))
        .prop(PropSpec::string("label", "Text label for the destination"))
        .prop(PropSpec::bool("disabled", "Whether this item is disabled"))
        .prop(PropSpec::number("badge", "Badge count to display"))
        .prop(PropSpec::bool("badge_dot", "Show badge as dot indicator"))
        .accessibility(AccessibilitySpec {
            role: Some("tab".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec!["aria-selected".into()],
        })
        .build()
}

/// Creates the NavigationDrawer component specification
///
/// A navigation drawer for accessing app destinations.
/// M3 Reference: https://m3.material.io/components/navigation-drawer/specs
pub fn navigation_drawer_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationDrawer", "m3.navigation")
        .name("NavigationDrawer")
        .description("A navigation drawer providing access to destinations and app functionality")
        .version("1.0.0")
        .prop(PropSpec::bool("open", "Whether the drawer is open"))
        .prop(PropSpec::string("selected", "Currently selected destination value"))
        .prop(
            PropSpec::enum_type(
                "variant",
                "Drawer presentation style",
                vec!["standard".into(), "modal".into()],
            )
            .with_default(serde_json::json!("standard")),
        )
        .prop(
            PropSpec::enum_type(
                "anchor",
                "Drawer anchor position",
                vec!["start".into(), "end".into()],
            )
            .with_default(serde_json::json!("start")),
        )
        .prop(PropSpec::number("width", "Drawer width in dp").with_default(serde_json::json!(360)))
        .slot(SlotSpec::named("header", "Drawer header content"))
        .slot(SlotSpec::default_slot().allow("m3.NavigationDrawerItem").allow("m3.Divider").allow("m3.NavigationDrawerSection"))
        .event(EventSpec::new("on_close", "Fired when drawer is closed"))
        .event(EventSpec::new("on_change", "Fired when selection changes"))
        .style_token("color.surface-container-low")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.secondary-container")
        .style_token("color.on-secondary-container")
        .style_token("color.scrim")
        .style_token("shape.corner.large-end")
        .accessibility(AccessibilitySpec {
            role: Some("navigation".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Escape".into(),
                    action: "Close drawer (modal)".into(),
                },
                KeyboardBehavior {
                    key: "ArrowUp/ArrowDown".into(),
                    action: "Navigate between items".into(),
                },
            ],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Modal Navigation Drawer".into(),
            description: Some("Slide-out navigation drawer".into()),
            code: r#"NavigationDrawer {
    variant: "modal"
    open: drawer_open
    selected: current_page
    on_close: close_drawer
    on_change: navigate_to
} {
    slot(header) {
        DrawerHeader { title: "My App" subtitle: "user@email.com" }
    }
    NavigationDrawerItem { value: "home" icon: "home" label: "Home" }
    NavigationDrawerItem { value: "settings" icon: "settings" label: "Settings" }
    Divider {}
    NavigationDrawerItem { value: "help" icon: "help" label: "Help & Feedback" }
}"#.into(),
        })
        .build()
}

/// Creates the NavigationDrawerItem component specification
pub fn navigation_drawer_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationDrawerItem", "m3.navigation")
        .name("NavigationDrawerItem")
        .description("An individual item within a NavigationDrawer")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this item").required())
        .prop(PropSpec::string("icon", "Material icon name"))
        .prop(PropSpec::string("label", "Text label for the item").required())
        .prop(PropSpec::bool("disabled", "Whether this item is disabled"))
        .prop(PropSpec::number("badge", "Badge count to display"))
        .prop(PropSpec::string("badge_label", "Badge text label"))
        .accessibility(AccessibilitySpec {
            role: Some("menuitem".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec!["aria-selected".into()],
        })
        .build()
}

/// Creates the NavigationDrawerSection component specification
pub fn navigation_drawer_section_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.NavigationDrawerSection", "m3.navigation")
        .name("NavigationDrawerSection")
        .description("A labeled section within a NavigationDrawer")
        .version("1.0.0")
        .prop(PropSpec::string("headline", "Section headline text"))
        .slot(SlotSpec::default_slot().allow("m3.NavigationDrawerItem"))
        .build()
}

/// Creates the TopAppBar component specification
///
/// A top app bar for context and actions.
/// M3 Reference: https://m3.material.io/components/top-app-bar/specs
pub fn top_app_bar_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.TopAppBar", "m3.navigation")
        .name("TopAppBar")
        .description("A top app bar providing context, navigation, and actions")
        .version("1.0.0")
        .prop(PropSpec::string("title", "App bar title text"))
        .prop(
            PropSpec::enum_type(
                "variant",
                "App bar size/style variant",
                vec![
                    "center-aligned".into(),
                    "small".into(),
                    "medium".into(),
                    "large".into(),
                ],
            )
            .with_default(serde_json::json!("small")),
        )
        .prop(
            PropSpec::enum_type(
                "scroll_behavior",
                "Behavior when content scrolls",
                vec![
                    "fixed".into(),
                    "scroll".into(),
                    "enter-always".into(),
                    "enter-always-collapsed".into(),
                ],
            )
            .with_default(serde_json::json!("fixed")),
        )
        .prop(PropSpec::string("navigation_icon", "Navigation icon (e.g., 'menu', 'arrow_back')"))
        .prop(PropSpec::bool("elevated", "Show elevation shadow"))
        .prop(PropSpec::color("container_color", "Override container color"))
        .slot(SlotSpec::named("navigation", "Navigation icon button"))
        .slot(SlotSpec::named("title", "Custom title content"))
        .slot(SlotSpec::named("actions", "Action icon buttons (max 3)"))
        .event(EventSpec::new("on_navigation_click", "Fired when navigation icon is clicked"))
        .style_token("color.surface")
        .style_token("color.surface-container")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("elevation.level0")
        .style_token("elevation.level2")
        .accessibility(AccessibilitySpec {
            role: Some("banner".into()),
            focusable: false,
            keyboard: vec![],
            required_aria: vec![],
        })
        .variant(VariantSpec {
            name: "center-aligned".into(),
            description: "Title centered, single navigation and action icons".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "small".into(),
            description: "Compact app bar with left-aligned title".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "medium".into(),
            description: "Medium-height app bar with two-line title support".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "large".into(),
            description: "Large app bar with prominent title".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Small Top App Bar".into(),
            description: Some("Standard app bar".into()),
            code: r#"TopAppBar {
    variant: "small"
    title: "Page Title"
    navigation_icon: "menu"
    on_navigation_click: open_drawer
} {
    slot(actions) {
        IconButton { icon: "search" aria_label: "Search" }
        IconButton { icon: "more_vert" aria_label: "More options" }
    }
}"#.into(),
        })
        .example(UsageExample {
            title: "Large Top App Bar".into(),
            description: Some("Prominent app bar with large title".into()),
            code: r#"TopAppBar {
    variant: "large"
    title: "Welcome Back"
    scroll_behavior: "scroll"
    navigation_icon: "arrow_back"
    on_navigation_click: go_back
}"#.into(),
        })
        .build()
}

/// Creates the TabBar (Tabs) component specification
///
/// A tab bar for organizing content across different screens.
/// M3 Reference: https://m3.material.io/components/tabs/specs
pub fn tab_bar_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.TabBar", "m3.navigation")
        .name("TabBar")
        .description("A tab bar for organizing content across different views")
        .version("1.0.0")
        .prop(PropSpec::string("selected", "Currently selected tab value"))
        .prop(
            PropSpec::enum_type(
                "variant",
                "Tab style variant",
                vec!["primary".into(), "secondary".into()],
            )
            .with_default(serde_json::json!("primary")),
        )
        .prop(PropSpec::bool("scrollable", "Allow horizontal scrolling for many tabs"))
        .prop(PropSpec::bool("fixed", "Fixed width tabs that fill container"))
        .slot(SlotSpec::default_slot().allow("m3.Tab").min(2))
        .event(EventSpec::new("on_change", "Fired when tab selection changes"))
        .style_token("color.surface")
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .style_token("color.on-primary")
        .accessibility(AccessibilitySpec {
            role: Some("tablist".into()),
            focusable: false,
            keyboard: vec![
                KeyboardBehavior {
                    key: "ArrowLeft/ArrowRight".into(),
                    action: "Navigate between tabs".into(),
                },
                KeyboardBehavior {
                    key: "Home".into(),
                    action: "Go to first tab".into(),
                },
                KeyboardBehavior {
                    key: "End".into(),
                    action: "Go to last tab".into(),
                },
            ],
            required_aria: vec![],
        })
        .variant(VariantSpec {
            name: "primary".into(),
            description: "Primary tabs at top of content".into(),
            defaults: HashMap::new(),
        })
        .variant(VariantSpec {
            name: "secondary".into(),
            description: "Secondary tabs within content area".into(),
            defaults: HashMap::new(),
        })
        .example(UsageExample {
            title: "Primary Tab Bar".into(),
            description: Some("Main content navigation".into()),
            code: r#"TabBar {
    variant: "primary"
    selected: current_tab
    on_change: set_current_tab
} {
    Tab { value: "overview" label: "Overview" icon: "info" }
    Tab { value: "specs" label: "Specifications" }
    Tab { value: "reviews" label: "Reviews" }
}"#.into(),
        })
        .build()
}

/// Creates the Tab component specification
pub fn tab_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Tab", "m3.navigation")
        .name("Tab")
        .description("An individual tab within a TabBar")
        .version("1.0.0")
        .prop(PropSpec::string("value", "Value identifier for this tab").required())
        .prop(PropSpec::string("label", "Text label for the tab"))
        .prop(PropSpec::string("icon", "Material icon name"))
        .prop(PropSpec::bool("disabled", "Whether the tab is disabled"))
        .prop(PropSpec::number("badge", "Badge count to display"))
        .prop(PropSpec::bool("badge_dot", "Show badge as dot indicator"))
        .slot(SlotSpec::named("content", "Tab panel content"))
        .accessibility(AccessibilitySpec {
            role: Some("tab".into()),
            focusable: true,
            keyboard: vec![
                KeyboardBehavior {
                    key: "Enter/Space".into(),
                    action: "Select tab".into(),
                },
            ],
            required_aria: vec!["aria-selected".into(), "aria-controls".into()],
        })
        .build()
}

/// Creates the Breadcrumbs component specification
pub fn breadcrumbs_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.Breadcrumbs", "m3.navigation")
        .name("Breadcrumbs")
        .description("A breadcrumb trail showing navigation hierarchy")
        .version("1.0.0")
        .prop(PropSpec::string("separator", "Separator character/icon").with_default(serde_json::json!("/")))
        .prop(PropSpec::number("max_items", "Maximum items to show before collapsing"))
        .prop(PropSpec::number("items_before_collapse", "Items to show before ellipsis"))
        .prop(PropSpec::number("items_after_collapse", "Items to show after ellipsis"))
        .slot(SlotSpec::default_slot().allow("m3.BreadcrumbItem"))
        .style_token("color.on-surface")
        .style_token("color.on-surface-variant")
        .style_token("color.primary")
        .accessibility(AccessibilitySpec {
            role: Some("navigation".into()),
            focusable: false,
            keyboard: vec![],
            required_aria: vec!["aria-label".into()],
        })
        .example(UsageExample {
            title: "Breadcrumb Navigation".into(),
            description: Some("Show current location in hierarchy".into()),
            code: r#"Breadcrumbs { aria_label: "Breadcrumb" } {
    BreadcrumbItem { href: "/" label: "Home" }
    BreadcrumbItem { href: "/products" label: "Products" }
    BreadcrumbItem { label: "Category" current: true }
}"#.into(),
        })
        .build()
}

/// Creates the BreadcrumbItem component specification
pub fn breadcrumb_item_spec() -> ComponentSpec {
    ComponentSpec::builder("m3.BreadcrumbItem", "m3.navigation")
        .name("BreadcrumbItem")
        .description("An individual item within Breadcrumbs")
        .version("1.0.0")
        .prop(PropSpec::string("label", "Text label for the breadcrumb").required())
        .prop(PropSpec::string("href", "Link destination"))
        .prop(PropSpec::string("icon", "Material icon name"))
        .prop(PropSpec::bool("current", "Whether this is the current page"))
        .prop(PropSpec::bool("disabled", "Whether this item is disabled"))
        .event(EventSpec::new("on_click", "Fired when item is clicked"))
        .accessibility(AccessibilitySpec {
            role: Some("link".into()),
            focusable: true,
            keyboard: vec![],
            required_aria: vec!["aria-current".into()],
        })
        .build()
}

/// Returns all navigation component specifications
pub fn all_navigation_specs() -> Vec<ComponentSpec> {
    vec![
        navigation_bar_spec(),
        navigation_bar_item_spec(),
        navigation_rail_spec(),
        navigation_rail_item_spec(),
        navigation_drawer_spec(),
        navigation_drawer_item_spec(),
        navigation_drawer_section_spec(),
        top_app_bar_spec(),
        tab_bar_spec(),
        tab_spec(),
        breadcrumbs_spec(),
        breadcrumb_item_spec(),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_navigation_bar_spec() {
        let spec = navigation_bar_spec();
        assert_eq!(spec.id, "m3.NavigationBar");
        assert_eq!(spec.pack, "m3.navigation");
        let slot = spec.default_slot().unwrap();
        assert_eq!(slot.min_children, 3);
        assert_eq!(slot.max_children, 5);
    }

    #[test]
    fn test_navigation_rail_has_fab_support() {
        let spec = navigation_rail_spec();
        assert!(spec.get_prop("show_fab").is_some());
        assert!(spec.get_prop("fab_icon").is_some());
        assert!(spec.get_event("on_fab_click").is_some());
    }

    #[test]
    fn test_navigation_drawer_variants() {
        let spec = navigation_drawer_spec();
        if let Some(prop) = spec.get_prop("variant") {
            match &prop.prop_type {
                crate::spec::PropType::Enum { values } => {
                    assert!(values.contains(&"standard".to_string()));
                    assert!(values.contains(&"modal".to_string()));
                }
                _ => panic!("Expected enum type"),
            }
        }
    }

    #[test]
    fn test_top_app_bar_variants() {
        let spec = top_app_bar_spec();
        assert_eq!(spec.variants.len(), 4);
        let variant_names: Vec<&str> = spec.variants.iter().map(|v| v.name.as_str()).collect();
        assert!(variant_names.contains(&"center-aligned"));
        assert!(variant_names.contains(&"small"));
        assert!(variant_names.contains(&"medium"));
        assert!(variant_names.contains(&"large"));
    }

    #[test]
    fn test_tab_bar_accessibility() {
        let spec = tab_bar_spec();
        assert_eq!(spec.accessibility.role, Some("tablist".into()));
    }

    #[test]
    fn test_tab_accessibility() {
        let spec = tab_spec();
        assert_eq!(spec.accessibility.role, Some("tab".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-selected".into()));
        assert!(spec.accessibility.required_aria.contains(&"aria-controls".into()));
    }

    #[test]
    fn test_all_navigation_specs_count() {
        let specs = all_navigation_specs();
        assert_eq!(specs.len(), 12);
    }
}
