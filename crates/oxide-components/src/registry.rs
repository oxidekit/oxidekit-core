//! Component Registry
//!
//! Central registry for all UI components with discovery and lookup.

use crate::spec::ComponentSpec;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Global component registry
pub struct ComponentRegistry {
    /// Components indexed by ID
    components: RwLock<HashMap<String, Arc<ComponentSpec>>>,

    /// Components indexed by pack
    packs: RwLock<HashMap<String, Vec<String>>>,
}

impl ComponentRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            components: RwLock::new(HashMap::new()),
            packs: RwLock::new(HashMap::new()),
        }
    }

    /// Create a registry with core components pre-registered
    pub fn with_core_components() -> Self {
        let registry = Self::new();
        registry.register_core_pack();
        registry
    }

    /// Register a component
    pub fn register(&self, spec: ComponentSpec) -> Result<(), RegistryError> {
        let id = spec.id.clone();
        let pack = spec.pack.clone();

        // Check for duplicate
        {
            let components = self.components.read().map_err(|_| RegistryError::LockError)?;
            if components.contains_key(&id) {
                return Err(RegistryError::DuplicateComponent(id));
            }
        }

        // Insert component
        {
            let mut components = self.components.write().map_err(|_| RegistryError::LockError)?;
            components.insert(id.clone(), Arc::new(spec));
        }

        // Update pack index
        {
            let mut packs = self.packs.write().map_err(|_| RegistryError::LockError)?;
            packs.entry(pack).or_default().push(id);
        }

        Ok(())
    }

    /// Get a component by ID
    pub fn get(&self, id: &str) -> Option<Arc<ComponentSpec>> {
        self.components.read().ok()?.get(id).cloned()
    }

    /// Check if a component exists
    pub fn contains(&self, id: &str) -> bool {
        self.components
            .read()
            .map(|c| c.contains_key(id))
            .unwrap_or(false)
    }

    /// Get all components in a pack
    pub fn get_pack(&self, pack: &str) -> Vec<Arc<ComponentSpec>> {
        let packs = match self.packs.read() {
            Ok(p) => p,
            Err(_) => return Vec::new(),
        };

        let components = match self.components.read() {
            Ok(c) => c,
            Err(_) => return Vec::new(),
        };

        packs
            .get(pack)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| components.get(id).cloned())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// List all registered component IDs
    pub fn list_components(&self) -> Vec<String> {
        self.components
            .read()
            .map(|c| c.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// List all registered packs
    pub fn list_packs(&self) -> Vec<String> {
        self.packs
            .read()
            .map(|p| p.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Export registry to AI-compatible JSON
    pub fn export_ai_json(&self) -> Result<String, RegistryError> {
        let components = self
            .components
            .read()
            .map_err(|_| RegistryError::LockError)?;

        let export = AiExport {
            version: "1.0".to_string(),
            components: components
                .values()
                .map(|c| (**c).clone())
                .collect(),
        };

        serde_json::to_string_pretty(&export).map_err(|e| RegistryError::SerializationError(e.to_string()))
    }

    /// Register the core UI pack
    fn register_core_pack(&self) {
        use crate::spec::*;

        // Button component
        let button = ComponentSpec::builder("ui.Button", "ui.core")
            .name("Button")
            .description("A clickable button component for triggering actions")
            .prop(PropSpec::string("label", "Button text content"))
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Visual style variant",
                    vec![
                        "primary".into(),
                        "secondary".into(),
                        "outline".into(),
                        "ghost".into(),
                        "danger".into(),
                    ],
                )
                .with_default(serde_json::json!("primary")),
            )
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Button size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
            .prop(PropSpec::bool("loading", "Whether to show loading spinner"))
            .prop(PropSpec::string("icon", "Icon name to display"))
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
            .style_token("color.primary")
            .style_token("color.text")
            .style_token("spacing.button")
            .style_token("radius.button")
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
            .variant(VariantSpec {
                name: "primary".into(),
                description: "Primary action button with filled background".into(),
                defaults: HashMap::new(),
            })
            .variant(VariantSpec {
                name: "secondary".into(),
                description: "Secondary action button with muted styling".into(),
                defaults: HashMap::new(),
            })
            .example(UsageExample {
                title: "Basic Button".into(),
                description: Some("A simple primary button".into()),
                code: r#"Button { label: "Click me" on_click: handle_click }"#.into(),
            })
            .build();

        let _ = self.register(button);

        // IconButton component
        let icon_button = ComponentSpec::builder("ui.IconButton", "ui.core")
            .name("IconButton")
            .description("A button that displays only an icon")
            .prop(PropSpec::string("icon", "Icon name to display").required())
            .prop(PropSpec::string("aria_label", "Accessible label for the button").required())
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Visual style variant",
                    vec!["default".into(), "ghost".into(), "outline".into()],
                )
                .with_default(serde_json::json!("default")),
            )
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Button size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("disabled", "Whether the button is disabled"))
            .event(EventSpec::new("on_click", "Fired when button is clicked"))
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
            .build();

        let _ = self.register(icon_button);

        // Card component
        let card = ComponentSpec::builder("ui.Card", "ui.core")
            .name("Card")
            .description("A container component with optional header, content, and footer sections")
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Card style variant",
                    vec!["elevated".into(), "outlined".into(), "filled".into()],
                )
                .with_default(serde_json::json!("elevated")),
            )
            .prop(PropSpec::bool("interactive", "Whether card responds to hover/click"))
            .prop(PropSpec::bool("selected", "Whether card is in selected state"))
            .slot(SlotSpec::default_slot())
            .slot(SlotSpec::named("header", "Card header content"))
            .slot(SlotSpec::named("footer", "Card footer content"))
            .event(EventSpec::new("on_click", "Fired when card is clicked (if interactive)"))
            .style_token("color.surface")
            .style_token("color.border")
            .style_token("radius.card")
            .style_token("shadow.card")
            .accessibility(AccessibilitySpec {
                role: Some("article".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(card);

        // Divider component
        let divider = ComponentSpec::builder("ui.Divider", "ui.core")
            .name("Divider")
            .description("A visual separator between content sections")
            .prop(
                PropSpec::enum_type(
                    "orientation",
                    "Divider orientation",
                    vec!["horizontal".into(), "vertical".into()],
                )
                .with_default(serde_json::json!("horizontal")),
            )
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Divider style",
                    vec!["solid".into(), "dashed".into(), "dotted".into()],
                )
                .with_default(serde_json::json!("solid")),
            )
            .prop(PropSpec::string("label", "Optional label text in the middle"))
            .style_token("color.divider")
            .accessibility(AccessibilitySpec {
                role: Some("separator".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(divider);

        // Badge component
        let badge = ComponentSpec::builder("ui.Badge", "ui.core")
            .name("Badge")
            .description("A small label for status or counts")
            .prop(PropSpec::string("label", "Badge text content"))
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Badge color variant",
                    vec![
                        "default".into(),
                        "primary".into(),
                        "success".into(),
                        "warning".into(),
                        "danger".into(),
                    ],
                )
                .with_default(serde_json::json!("default")),
            )
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Badge size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("dot", "Show as dot indicator without text"))
            .prop(PropSpec::number("max", "Maximum value before showing overflow"))
            .style_token("color.badge")
            .build();

        let _ = self.register(badge);

        // Avatar component
        let avatar = ComponentSpec::builder("ui.Avatar", "ui.core")
            .name("Avatar")
            .description("A component for displaying user profile images or initials")
            .prop(PropSpec::string("src", "Image source URL"))
            .prop(PropSpec::string("alt", "Alternative text for accessibility"))
            .prop(PropSpec::string("fallback", "Text to show when image fails (usually initials)"))
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Avatar size",
                    vec!["xs".into(), "sm".into(), "md".into(), "lg".into(), "xl".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(
                PropSpec::enum_type(
                    "shape",
                    "Avatar shape",
                    vec!["circle".into(), "square".into()],
                )
                .with_default(serde_json::json!("circle")),
            )
            .prop(PropSpec::bool("bordered", "Whether to show a border"))
            .style_token("color.avatar.background")
            .style_token("color.avatar.text")
            .accessibility(AccessibilitySpec {
                role: Some("img".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-label".into()],
            })
            .build();

        let _ = self.register(avatar);

        // Text component
        let text = ComponentSpec::builder("ui.Text", "ui.core")
            .name("Text")
            .description("A component for rendering text with typography roles")
            .prop(PropSpec::string("content", "Text content to display").required())
            .prop(
                PropSpec::enum_type(
                    "role",
                    "Typography role",
                    vec![
                        "body".into(),
                        "heading".into(),
                        "title".into(),
                        "caption".into(),
                        "label".into(),
                        "code".into(),
                    ],
                )
                .with_default(serde_json::json!("body")),
            )
            .prop(PropSpec::number("size", "Font size override"))
            .prop(PropSpec::color("color", "Text color override"))
            .prop(
                PropSpec::enum_type(
                    "weight",
                    "Font weight",
                    vec![
                        "thin".into(),
                        "light".into(),
                        "normal".into(),
                        "medium".into(),
                        "semibold".into(),
                        "bold".into(),
                    ],
                )
                .with_default(serde_json::json!("normal")),
            )
            .prop(
                PropSpec::enum_type(
                    "align",
                    "Text alignment",
                    vec!["left".into(), "center".into(), "right".into()],
                )
                .with_default(serde_json::json!("left")),
            )
            .prop(PropSpec::bool("truncate", "Whether to truncate with ellipsis"))
            .prop(PropSpec::number("lines", "Max lines before truncation"))
            .style_token("typography.body")
            .style_token("typography.heading")
            .style_token("color.text")
            .build();

        let _ = self.register(text);

        // Input/TextField component
        let input = ComponentSpec::builder("ui.Input", "ui.core")
            .name("Input")
            .description("A text input field for single-line user input")
            .prop(PropSpec::string("value", "Current input value"))
            .prop(PropSpec::string("placeholder", "Placeholder text when empty"))
            .prop(PropSpec::string("label", "Label text above the input"))
            .prop(
                PropSpec::enum_type(
                    "type",
                    "Input type",
                    vec![
                        "text".into(),
                        "password".into(),
                        "email".into(),
                        "number".into(),
                        "tel".into(),
                        "url".into(),
                        "search".into(),
                    ],
                )
                .with_default(serde_json::json!("text")),
            )
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Input size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("disabled", "Whether the input is disabled"))
            .prop(PropSpec::bool("readonly", "Whether the input is read-only"))
            .prop(PropSpec::bool("required", "Whether the input is required"))
            .prop(PropSpec::string("error", "Error message to display"))
            .prop(PropSpec::string("helper", "Helper text below the input"))
            .prop(PropSpec::string("prefix", "Prefix content (icon or text)"))
            .prop(PropSpec::string("suffix", "Suffix content (icon or text)"))
            .prop(PropSpec::number("min", "Minimum value (for number type)"))
            .prop(PropSpec::number("max", "Maximum value (for number type)"))
            .prop(PropSpec::number("step", "Step increment (for number type)"))
            .prop(PropSpec::number("maxlength", "Maximum character length"))
            .event(EventSpec::new("on_change", "Fired when value changes"))
            .event(EventSpec::new("on_input", "Fired on every keystroke"))
            .event(EventSpec::new("on_focus", "Fired when input receives focus"))
            .event(EventSpec::new("on_blur", "Fired when input loses focus"))
            .event(EventSpec::new("on_enter", "Fired when Enter key is pressed"))
            .style_token("color.input.background")
            .style_token("color.input.border")
            .style_token("color.input.text")
            .style_token("color.error")
            .accessibility(AccessibilitySpec {
                role: Some("textbox".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Tab".into(),
                        action: "Move focus to next element".into(),
                    },
                ],
                required_aria: vec!["aria-label".into()],
            })
            .example(UsageExample {
                title: "Basic Input".into(),
                description: Some("A simple text input".into()),
                code: r#"Input { placeholder: "Enter your name" on_change: handle_change }"#.into(),
            })
            .build();

        let _ = self.register(input);

        // TextArea component
        let textarea = ComponentSpec::builder("ui.TextArea", "ui.core")
            .name("TextArea")
            .description("A multi-line text input field")
            .prop(PropSpec::string("value", "Current textarea value"))
            .prop(PropSpec::string("placeholder", "Placeholder text when empty"))
            .prop(PropSpec::string("label", "Label text above the textarea"))
            .prop(PropSpec::number("rows", "Number of visible text rows").with_default(serde_json::json!(4)))
            .prop(PropSpec::bool("resize", "Whether the textarea is resizable").with_default(serde_json::json!(true)))
            .prop(PropSpec::bool("disabled", "Whether the textarea is disabled"))
            .prop(PropSpec::bool("readonly", "Whether the textarea is read-only"))
            .prop(PropSpec::string("error", "Error message to display"))
            .prop(PropSpec::number("maxlength", "Maximum character length"))
            .prop(PropSpec::bool("show_count", "Show character count"))
            .event(EventSpec::new("on_change", "Fired when value changes"))
            .event(EventSpec::new("on_input", "Fired on every keystroke"))
            .accessibility(AccessibilitySpec {
                role: Some("textbox".into()),
                focusable: true,
                keyboard: vec![],
                required_aria: vec!["aria-label".into(), "aria-multiline".into()],
            })
            .build();

        let _ = self.register(textarea);

        // Select component
        let select = ComponentSpec::builder("ui.Select", "ui.core")
            .name("Select")
            .description("A dropdown selection component")
            .prop(PropSpec::string("value", "Currently selected value"))
            .prop(PropSpec::string("placeholder", "Placeholder text when nothing selected"))
            .prop(PropSpec::string("label", "Label text above the select"))
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Select size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("disabled", "Whether the select is disabled"))
            .prop(PropSpec::bool("searchable", "Whether options can be searched"))
            .prop(PropSpec::bool("clearable", "Whether selection can be cleared"))
            .prop(PropSpec::bool("multiple", "Whether multiple selections are allowed"))
            .prop(PropSpec::string("error", "Error message to display"))
            .slot(SlotSpec::default_slot().allow("ui.Option"))
            .event(EventSpec::new("on_change", "Fired when selection changes"))
            .event(EventSpec::new("on_open", "Fired when dropdown opens"))
            .event(EventSpec::new("on_close", "Fired when dropdown closes"))
            .accessibility(AccessibilitySpec {
                role: Some("combobox".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "ArrowDown".into(),
                        action: "Open dropdown / Move to next option".into(),
                    },
                    KeyboardBehavior {
                        key: "ArrowUp".into(),
                        action: "Move to previous option".into(),
                    },
                    KeyboardBehavior {
                        key: "Enter".into(),
                        action: "Select highlighted option".into(),
                    },
                    KeyboardBehavior {
                        key: "Escape".into(),
                        action: "Close dropdown".into(),
                    },
                ],
                required_aria: vec!["aria-expanded".into(), "aria-haspopup".into()],
            })
            .build();

        let _ = self.register(select);

        // Checkbox component
        let checkbox = ComponentSpec::builder("ui.Checkbox", "ui.core")
            .name("Checkbox")
            .description("A checkbox input for boolean values")
            .prop(PropSpec::bool("checked", "Whether the checkbox is checked"))
            .prop(PropSpec::bool("indeterminate", "Whether the checkbox is in indeterminate state"))
            .prop(PropSpec::string("label", "Label text next to the checkbox"))
            .prop(PropSpec::bool("disabled", "Whether the checkbox is disabled"))
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Checkbox size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .event(EventSpec::new("on_change", "Fired when checked state changes"))
            .style_token("color.primary")
            .style_token("color.checkbox.border")
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
            .build();

        let _ = self.register(checkbox);

        // Radio component
        let radio = ComponentSpec::builder("ui.Radio", "ui.core")
            .name("Radio")
            .description("A radio button for single selection from a group")
            .prop(PropSpec::string("value", "Value of this radio option"))
            .prop(PropSpec::string("name", "Group name for radio buttons"))
            .prop(PropSpec::bool("checked", "Whether the radio is selected"))
            .prop(PropSpec::string("label", "Label text next to the radio"))
            .prop(PropSpec::bool("disabled", "Whether the radio is disabled"))
            .event(EventSpec::new("on_change", "Fired when selection changes"))
            .accessibility(AccessibilitySpec {
                role: Some("radio".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Space".into(),
                        action: "Select radio".into(),
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
            .build();

        let _ = self.register(radio);

        // Switch/Toggle component
        let switch = ComponentSpec::builder("ui.Switch", "ui.core")
            .name("Switch")
            .description("A toggle switch for boolean values")
            .prop(PropSpec::bool("checked", "Whether the switch is on"))
            .prop(PropSpec::string("label", "Label text next to the switch"))
            .prop(PropSpec::bool("disabled", "Whether the switch is disabled"))
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Switch size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::string("on_label", "Label for on state"))
            .prop(PropSpec::string("off_label", "Label for off state"))
            .event(EventSpec::new("on_change", "Fired when switch state changes"))
            .style_token("color.primary")
            .style_token("color.switch.track")
            .accessibility(AccessibilitySpec {
                role: Some("switch".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Space".into(),
                        action: "Toggle switch".into(),
                    },
                ],
                required_aria: vec!["aria-checked".into()],
            })
            .build();

        let _ = self.register(switch);

        // Slider component
        let slider = ComponentSpec::builder("ui.Slider", "ui.core")
            .name("Slider")
            .description("A slider input for selecting numeric values within a range")
            .prop(PropSpec::number("value", "Current slider value"))
            .prop(PropSpec::number("min", "Minimum value").with_default(serde_json::json!(0)))
            .prop(PropSpec::number("max", "Maximum value").with_default(serde_json::json!(100)))
            .prop(PropSpec::number("step", "Step increment").with_default(serde_json::json!(1)))
            .prop(PropSpec::bool("disabled", "Whether the slider is disabled"))
            .prop(PropSpec::bool("show_value", "Show current value label"))
            .prop(PropSpec::bool("show_ticks", "Show tick marks"))
            .prop(
                PropSpec::enum_type(
                    "orientation",
                    "Slider orientation",
                    vec!["horizontal".into(), "vertical".into()],
                )
                .with_default(serde_json::json!("horizontal")),
            )
            .event(EventSpec::new("on_change", "Fired when value changes"))
            .event(EventSpec::new("on_change_end", "Fired when dragging ends"))
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
                ],
                required_aria: vec!["aria-valuemin".into(), "aria-valuemax".into(), "aria-valuenow".into()],
            })
            .build();

        let _ = self.register(slider);

        // Progress component
        let progress = ComponentSpec::builder("ui.Progress", "ui.core")
            .name("Progress")
            .description("A progress indicator showing completion status")
            .prop(PropSpec::number("value", "Current progress value (0-100)"))
            .prop(PropSpec::bool("indeterminate", "Show indeterminate animation"))
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Progress style",
                    vec!["linear".into(), "circular".into()],
                )
                .with_default(serde_json::json!("linear")),
            )
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Progress size",
                    vec!["sm".into(), "md".into(), "lg".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::color("color", "Progress bar color"))
            .prop(PropSpec::bool("show_value", "Show percentage label"))
            .style_token("color.primary")
            .style_token("color.progress.track")
            .accessibility(AccessibilitySpec {
                role: Some("progressbar".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-valuemin".into(), "aria-valuemax".into(), "aria-valuenow".into()],
            })
            .build();

        let _ = self.register(progress);

        // Spinner component
        let spinner = ComponentSpec::builder("ui.Spinner", "ui.core")
            .name("Spinner")
            .description("A loading spinner indicator")
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Spinner size",
                    vec!["xs".into(), "sm".into(), "md".into(), "lg".into(), "xl".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::color("color", "Spinner color"))
            .prop(PropSpec::string("label", "Accessible label for screen readers"))
            .style_token("color.primary")
            .accessibility(AccessibilitySpec {
                role: Some("status".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-label".into()],
            })
            .build();

        let _ = self.register(spinner);

        // Modal/Dialog component
        let modal = ComponentSpec::builder("ui.Modal", "ui.core")
            .name("Modal")
            .description("A modal dialog overlay")
            .prop(PropSpec::bool("open", "Whether the modal is open").required())
            .prop(PropSpec::string("title", "Modal title"))
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Modal size",
                    vec!["sm".into(), "md".into(), "lg".into(), "xl".into(), "full".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::bool("closable", "Whether the modal can be closed by user").with_default(serde_json::json!(true)))
            .prop(PropSpec::bool("close_on_overlay", "Close when clicking outside").with_default(serde_json::json!(true)))
            .prop(PropSpec::bool("close_on_escape", "Close when pressing Escape").with_default(serde_json::json!(true)))
            .slot(SlotSpec::default_slot())
            .slot(SlotSpec::named("header", "Custom header content"))
            .slot(SlotSpec::named("footer", "Footer content (usually buttons)"))
            .event(EventSpec::new("on_close", "Fired when modal is closed"))
            .event(EventSpec::new("on_open", "Fired when modal is opened"))
            .style_token("color.surface")
            .style_token("color.overlay")
            .style_token("radius.modal")
            .style_token("shadow.modal")
            .accessibility(AccessibilitySpec {
                role: Some("dialog".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Escape".into(),
                        action: "Close modal".into(),
                    },
                    KeyboardBehavior {
                        key: "Tab".into(),
                        action: "Cycle focus within modal".into(),
                    },
                ],
                required_aria: vec!["aria-modal".into(), "aria-labelledby".into()],
            })
            .build();

        let _ = self.register(modal);

        // Toast/Snackbar component
        let toast = ComponentSpec::builder("ui.Toast", "ui.core")
            .name("Toast")
            .description("A temporary notification message")
            .prop(PropSpec::string("message", "Toast message").required())
            .prop(PropSpec::string("title", "Optional toast title"))
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Toast type",
                    vec!["info".into(), "success".into(), "warning".into(), "error".into()],
                )
                .with_default(serde_json::json!("info")),
            )
            .prop(PropSpec::number("duration", "Auto-dismiss duration in ms (0 = manual)").with_default(serde_json::json!(5000)))
            .prop(PropSpec::bool("dismissible", "Whether toast can be manually dismissed").with_default(serde_json::json!(true)))
            .prop(
                PropSpec::enum_type(
                    "position",
                    "Toast position on screen",
                    vec![
                        "top-left".into(),
                        "top-center".into(),
                        "top-right".into(),
                        "bottom-left".into(),
                        "bottom-center".into(),
                        "bottom-right".into(),
                    ],
                )
                .with_default(serde_json::json!("bottom-center")),
            )
            .prop(PropSpec::string("action_label", "Optional action button label"))
            .event(EventSpec::new("on_dismiss", "Fired when toast is dismissed"))
            .event(EventSpec::new("on_action", "Fired when action button is clicked"))
            .accessibility(AccessibilitySpec {
                role: Some("alert".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-live".into()],
            })
            .build();

        let _ = self.register(toast);

        // Tabs component
        let tabs = ComponentSpec::builder("ui.Tabs", "ui.core")
            .name("Tabs")
            .description("A tabbed navigation component")
            .prop(PropSpec::string("value", "Currently active tab value"))
            .prop(PropSpec::string("default_value", "Default active tab"))
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Tab style variant",
                    vec!["line".into(), "enclosed".into(), "pills".into()],
                )
                .with_default(serde_json::json!("line")),
            )
            .prop(
                PropSpec::enum_type(
                    "orientation",
                    "Tabs orientation",
                    vec!["horizontal".into(), "vertical".into()],
                )
                .with_default(serde_json::json!("horizontal")),
            )
            .prop(PropSpec::bool("fitted", "Whether tabs should fill available space"))
            .slot(SlotSpec::default_slot().allow("ui.Tab"))
            .event(EventSpec::new("on_change", "Fired when active tab changes"))
            .accessibility(AccessibilitySpec {
                role: Some("tablist".into()),
                focusable: true,
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
            .build();

        let _ = self.register(tabs);

        // Tab component
        let tab = ComponentSpec::builder("ui.Tab", "ui.core")
            .name("Tab")
            .description("A single tab within a Tabs component")
            .prop(PropSpec::string("value", "Tab identifier value").required())
            .prop(PropSpec::string("label", "Tab label text").required())
            .prop(PropSpec::string("icon", "Optional tab icon"))
            .prop(PropSpec::bool("disabled", "Whether the tab is disabled"))
            .slot(SlotSpec::default_slot())
            .accessibility(AccessibilitySpec {
                role: Some("tab".into()),
                focusable: true,
                keyboard: vec![],
                required_aria: vec!["aria-selected".into(), "aria-controls".into()],
            })
            .build();

        let _ = self.register(tab);

        // Accordion component
        let accordion = ComponentSpec::builder("ui.Accordion", "ui.core")
            .name("Accordion")
            .description("A collapsible accordion component")
            .prop(PropSpec::string("value", "Currently expanded item(s)"))
            .prop(PropSpec::bool("multiple", "Allow multiple items to be expanded").with_default(serde_json::json!(false)))
            .prop(PropSpec::bool("collapsible", "Allow all items to be collapsed").with_default(serde_json::json!(true)))
            .slot(SlotSpec::default_slot().allow("ui.AccordionItem"))
            .event(EventSpec::new("on_change", "Fired when expanded items change"))
            .accessibility(AccessibilitySpec {
                role: Some("group".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(accordion);

        // AccordionItem component
        let accordion_item = ComponentSpec::builder("ui.AccordionItem", "ui.core")
            .name("AccordionItem")
            .description("A single item within an Accordion")
            .prop(PropSpec::string("value", "Item identifier value").required())
            .prop(PropSpec::string("title", "Item header title").required())
            .prop(PropSpec::string("subtitle", "Optional subtitle"))
            .prop(PropSpec::bool("disabled", "Whether the item is disabled"))
            .slot(SlotSpec::default_slot())
            .accessibility(AccessibilitySpec {
                role: Some("region".into()),
                focusable: false,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Enter/Space".into(),
                        action: "Toggle accordion item".into(),
                    },
                ],
                required_aria: vec!["aria-expanded".into()],
            })
            .build();

        let _ = self.register(accordion_item);

        // Tooltip component
        let tooltip = ComponentSpec::builder("ui.Tooltip", "ui.core")
            .name("Tooltip")
            .description("A tooltip that appears on hover/focus")
            .prop(PropSpec::string("content", "Tooltip content text").required())
            .prop(
                PropSpec::enum_type(
                    "placement",
                    "Tooltip position relative to trigger",
                    vec![
                        "top".into(),
                        "top-start".into(),
                        "top-end".into(),
                        "bottom".into(),
                        "bottom-start".into(),
                        "bottom-end".into(),
                        "left".into(),
                        "right".into(),
                    ],
                )
                .with_default(serde_json::json!("top")),
            )
            .prop(PropSpec::number("delay", "Delay before showing (ms)").with_default(serde_json::json!(200)))
            .prop(PropSpec::bool("arrow", "Show arrow pointing to trigger").with_default(serde_json::json!(true)))
            .slot(SlotSpec::default_slot())
            .style_token("color.tooltip.background")
            .style_token("color.tooltip.text")
            .accessibility(AccessibilitySpec {
                role: Some("tooltip".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-describedby".into()],
            })
            .build();

        let _ = self.register(tooltip);

        // Menu component
        let menu = ComponentSpec::builder("ui.Menu", "ui.core")
            .name("Menu")
            .description("A dropdown menu component")
            .prop(PropSpec::bool("open", "Whether the menu is open"))
            .prop(
                PropSpec::enum_type(
                    "placement",
                    "Menu placement relative to trigger",
                    vec![
                        "bottom-start".into(),
                        "bottom-end".into(),
                        "top-start".into(),
                        "top-end".into(),
                    ],
                )
                .with_default(serde_json::json!("bottom-start")),
            )
            .slot(SlotSpec::named("trigger", "Element that triggers the menu").required())
            .slot(SlotSpec::default_slot().allow("ui.MenuItem").allow("ui.MenuDivider"))
            .event(EventSpec::new("on_open", "Fired when menu opens"))
            .event(EventSpec::new("on_close", "Fired when menu closes"))
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
                        action: "Select item".into(),
                    },
                    KeyboardBehavior {
                        key: "Escape".into(),
                        action: "Close menu".into(),
                    },
                ],
                required_aria: vec!["aria-expanded".into()],
            })
            .build();

        let _ = self.register(menu);

        // MenuItem component
        let menu_item = ComponentSpec::builder("ui.MenuItem", "ui.core")
            .name("MenuItem")
            .description("A single item within a Menu")
            .prop(PropSpec::string("label", "Item label text").required())
            .prop(PropSpec::string("icon", "Optional item icon"))
            .prop(PropSpec::string("shortcut", "Keyboard shortcut hint"))
            .prop(PropSpec::bool("disabled", "Whether the item is disabled"))
            .prop(PropSpec::bool("destructive", "Whether this is a destructive action"))
            .event(EventSpec::new("on_click", "Fired when item is clicked"))
            .accessibility(AccessibilitySpec {
                role: Some("menuitem".into()),
                focusable: true,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(menu_item);

        // Table component
        let table = ComponentSpec::builder("ui.Table", "ui.core")
            .name("Table")
            .description("A data table component")
            .prop(PropSpec::bool("striped", "Alternate row colors"))
            .prop(PropSpec::bool("hoverable", "Highlight row on hover"))
            .prop(PropSpec::bool("bordered", "Show cell borders"))
            .prop(PropSpec::bool("compact", "Reduce cell padding"))
            .prop(PropSpec::bool("sticky_header", "Keep header visible when scrolling"))
            .slot(SlotSpec::named("header", "Table header row"))
            .slot(SlotSpec::default_slot())
            .style_token("color.table.background")
            .style_token("color.table.border")
            .style_token("color.table.stripe")
            .accessibility(AccessibilitySpec {
                role: Some("table".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(table);

        // Pagination component
        let pagination = ComponentSpec::builder("ui.Pagination", "ui.core")
            .name("Pagination")
            .description("A pagination control for navigating pages")
            .prop(PropSpec::number("page", "Current page number").required())
            .prop(PropSpec::number("total", "Total number of items").required())
            .prop(PropSpec::number("per_page", "Items per page").with_default(serde_json::json!(10)))
            .prop(PropSpec::number("siblings", "Number of sibling pages to show").with_default(serde_json::json!(1)))
            .prop(PropSpec::bool("show_first_last", "Show first/last page buttons").with_default(serde_json::json!(true)))
            .prop(PropSpec::bool("show_prev_next", "Show prev/next buttons").with_default(serde_json::json!(true)))
            .event(EventSpec::new("on_change", "Fired when page changes"))
            .accessibility(AccessibilitySpec {
                role: Some("navigation".into()),
                focusable: true,
                keyboard: vec![],
                required_aria: vec!["aria-label".into()],
            })
            .build();

        let _ = self.register(pagination);

        // Sidebar component
        let sidebar = ComponentSpec::builder("ui.Sidebar", "ui.core")
            .name("Sidebar")
            .description("A collapsible sidebar navigation component")
            .prop(PropSpec::bool("collapsed", "Whether the sidebar is collapsed"))
            .prop(PropSpec::number("width", "Sidebar width when expanded").with_default(serde_json::json!(240)))
            .prop(PropSpec::number("collapsed_width", "Sidebar width when collapsed").with_default(serde_json::json!(64)))
            .prop(
                PropSpec::enum_type(
                    "position",
                    "Sidebar position",
                    vec!["left".into(), "right".into()],
                )
                .with_default(serde_json::json!("left")),
            )
            .prop(PropSpec::bool("resizable", "Allow user to resize"))
            .slot(SlotSpec::default_slot())
            .slot(SlotSpec::named("header", "Sidebar header (logo area)"))
            .slot(SlotSpec::named("footer", "Sidebar footer"))
            .event(EventSpec::new("on_collapse", "Fired when collapse state changes"))
            .event(EventSpec::new("on_resize", "Fired when sidebar is resized"))
            .style_token("color.sidebar.background")
            .accessibility(AccessibilitySpec {
                role: Some("complementary".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-label".into()],
            })
            .build();

        let _ = self.register(sidebar);

        // AppBar/Navbar component
        let app_bar = ComponentSpec::builder("ui.AppBar", "ui.core")
            .name("AppBar")
            .description("A top application bar/header")
            .prop(
                PropSpec::enum_type(
                    "position",
                    "AppBar position behavior",
                    vec!["static".into(), "fixed".into(), "sticky".into()],
                )
                .with_default(serde_json::json!("static")),
            )
            .prop(PropSpec::bool("elevated", "Show elevation shadow"))
            .prop(PropSpec::color("background", "Background color override"))
            .slot(SlotSpec::named("start", "Left section content"))
            .slot(SlotSpec::default_slot())
            .slot(SlotSpec::named("end", "Right section content"))
            .style_token("color.appbar.background")
            .style_token("shadow.appbar")
            .accessibility(AccessibilitySpec {
                role: Some("banner".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(app_bar);

        // Container component
        let container = ComponentSpec::builder("ui.Container", "ui.core")
            .name("Container")
            .description("A container that centers content and applies max-width")
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Container max-width",
                    vec!["sm".into(), "md".into(), "lg".into(), "xl".into(), "full".into()],
                )
                .with_default(serde_json::json!("lg")),
            )
            .prop(PropSpec::bool("center", "Center children horizontally").with_default(serde_json::json!(true)))
            .prop(PropSpec::number("padding", "Container padding"))
            .slot(SlotSpec::default_slot())
            .style_token("spacing.container")
            .build();

        let _ = self.register(container);

        // Row component
        let row = ComponentSpec::builder("ui.Row", "ui.core")
            .name("Row")
            .description("A horizontal flex container")
            .prop(PropSpec::number("gap", "Gap between children"))
            .prop(
                PropSpec::enum_type(
                    "align",
                    "Vertical alignment",
                    vec!["start".into(), "center".into(), "end".into(), "stretch".into(), "baseline".into()],
                )
                .with_default(serde_json::json!("stretch")),
            )
            .prop(
                PropSpec::enum_type(
                    "justify",
                    "Horizontal distribution",
                    vec!["start".into(), "center".into(), "end".into(), "space-between".into(), "space-around".into(), "space-evenly".into()],
                )
                .with_default(serde_json::json!("start")),
            )
            .prop(PropSpec::bool("wrap", "Allow wrapping to next line"))
            .prop(PropSpec::bool("reverse", "Reverse child order"))
            .slot(SlotSpec::default_slot())
            .build();

        let _ = self.register(row);

        // Column component
        let column = ComponentSpec::builder("ui.Column", "ui.core")
            .name("Column")
            .description("A vertical flex container")
            .prop(PropSpec::number("gap", "Gap between children"))
            .prop(
                PropSpec::enum_type(
                    "align",
                    "Horizontal alignment",
                    vec!["start".into(), "center".into(), "end".into(), "stretch".into()],
                )
                .with_default(serde_json::json!("stretch")),
            )
            .prop(
                PropSpec::enum_type(
                    "justify",
                    "Vertical distribution",
                    vec!["start".into(), "center".into(), "end".into(), "space-between".into(), "space-around".into(), "space-evenly".into()],
                )
                .with_default(serde_json::json!("start")),
            )
            .prop(PropSpec::bool("reverse", "Reverse child order"))
            .slot(SlotSpec::default_slot())
            .build();

        let _ = self.register(column);

        // Icon component
        let icon = ComponentSpec::builder("ui.Icon", "ui.core")
            .name("Icon")
            .description("An icon component supporting multiple icon sets")
            .prop(PropSpec::string("name", "Icon name").required())
            .prop(
                PropSpec::enum_type(
                    "size",
                    "Icon size",
                    vec!["xs".into(), "sm".into(), "md".into(), "lg".into(), "xl".into()],
                )
                .with_default(serde_json::json!("md")),
            )
            .prop(PropSpec::color("color", "Icon color"))
            .prop(PropSpec::bool("spin", "Apply spinning animation"))
            .style_token("color.icon")
            .accessibility(AccessibilitySpec {
                role: Some("img".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-hidden".into()],
            })
            .build();

        let _ = self.register(icon);

        // Image component
        let image = ComponentSpec::builder("ui.Image", "ui.core")
            .name("Image")
            .description("An image component with loading states")
            .prop(PropSpec::string("src", "Image source URL").required())
            .prop(PropSpec::string("alt", "Alt text for accessibility").required())
            .prop(PropSpec::number("width", "Image width"))
            .prop(PropSpec::number("height", "Image height"))
            .prop(
                PropSpec::enum_type(
                    "fit",
                    "Object fit behavior",
                    vec!["cover".into(), "contain".into(), "fill".into(), "none".into(), "scale-down".into()],
                )
                .with_default(serde_json::json!("cover")),
            )
            .prop(PropSpec::number("radius", "Border radius"))
            .prop(PropSpec::string("fallback", "Fallback image or element on error"))
            .prop(PropSpec::bool("lazy", "Enable lazy loading").with_default(serde_json::json!(true)))
            .event(EventSpec::new("on_load", "Fired when image loads"))
            .event(EventSpec::new("on_error", "Fired when image fails to load"))
            .accessibility(AccessibilitySpec {
                role: Some("img".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-label".into()],
            })
            .build();

        let _ = self.register(image);

        // Link component
        let link = ComponentSpec::builder("ui.Link", "ui.core")
            .name("Link")
            .description("A hyperlink component")
            .prop(PropSpec::string("href", "Link destination URL").required())
            .prop(PropSpec::string("label", "Link text"))
            .prop(PropSpec::bool("external", "Open in new tab"))
            .prop(PropSpec::bool("underline", "Show underline").with_default(serde_json::json!(true)))
            .prop(PropSpec::color("color", "Link color"))
            .event(EventSpec::new("on_click", "Fired when link is clicked"))
            .style_token("color.link")
            .style_token("color.link.hover")
            .accessibility(AccessibilitySpec {
                role: Some("link".into()),
                focusable: true,
                keyboard: vec![
                    KeyboardBehavior {
                        key: "Enter".into(),
                        action: "Navigate to link".into(),
                    },
                ],
                required_aria: vec![],
            })
            .build();

        let _ = self.register(link);

        // Skeleton component (loading placeholder)
        let skeleton = ComponentSpec::builder("ui.Skeleton", "ui.core")
            .name("Skeleton")
            .description("A loading placeholder component")
            .prop(
                PropSpec::enum_type(
                    "variant",
                    "Skeleton shape",
                    vec!["text".into(), "circular".into(), "rectangular".into()],
                )
                .with_default(serde_json::json!("text")),
            )
            .prop(PropSpec::number("width", "Skeleton width"))
            .prop(PropSpec::number("height", "Skeleton height"))
            .prop(PropSpec::bool("animate", "Enable shimmer animation").with_default(serde_json::json!(true)))
            .style_token("color.skeleton")
            .accessibility(AccessibilitySpec {
                role: Some("status".into()),
                focusable: false,
                keyboard: vec![],
                required_aria: vec!["aria-busy".into()],
            })
            .build();

        let _ = self.register(skeleton);

        tracing::debug!("Registered ui.core pack with {} components", 32);
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Component already registered: {0}")]
    DuplicateComponent(String),

    #[error("Component not found: {0}")]
    NotFound(String),

    #[error("Failed to acquire lock")]
    LockError,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Export format for AI tools
#[derive(Debug, Serialize, Deserialize)]
pub struct AiExport {
    /// Export format version
    pub version: String,

    /// All component specs
    pub components: Vec<ComponentSpec>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_creation() {
        let registry = ComponentRegistry::new();
        assert!(registry.list_components().is_empty());
    }

    #[test]
    fn test_core_pack_registration() {
        let registry = ComponentRegistry::with_core_components();
        let components = registry.list_components();

        assert!(components.contains(&"ui.Button".to_string()));
        assert!(components.contains(&"ui.Card".to_string()));
        assert!(components.contains(&"ui.Text".to_string()));
    }

    #[test]
    fn test_get_component() {
        let registry = ComponentRegistry::with_core_components();
        let button = registry.get("ui.Button");

        assert!(button.is_some());
        let button = button.unwrap();
        assert_eq!(button.name, "Button");
        assert!(button.get_prop("label").is_some());
    }

    #[test]
    fn test_get_pack() {
        let registry = ComponentRegistry::with_core_components();
        let core_pack = registry.get_pack("ui.core");

        assert!(!core_pack.is_empty());
    }

    #[test]
    fn test_export_ai_json() {
        let registry = ComponentRegistry::with_core_components();
        let json = registry.export_ai_json().unwrap();

        assert!(json.contains("ui.Button"));
        assert!(json.contains("props"));
        assert!(json.contains("events"));
    }
}
