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

        tracing::debug!("Registered ui.core pack with {} components", 7);
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
