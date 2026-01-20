//! Component Recognition and Mapping
//!
//! Maps Figma components/frames to OxideKit UI components using:
//! - Name-based heuristics
//! - Structure analysis (auto-layout patterns)
//! - Variant detection
//! - Size/interaction metadata

use crate::error::{FigmaError, Result};
use crate::types::*;
use regex::Regex;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Component mapper for Figma to OxideKit
#[derive(Debug)]
pub struct ComponentMapper {
    /// Mapping rules
    rules: Vec<MappingRule>,

    /// Component registry (known OxideKit components)
    registry: ComponentRegistry,

    /// Configuration
    config: MapperConfig,
}

/// Configuration for component mapping
#[derive(Debug, Clone)]
pub struct MapperConfig {
    /// Minimum confidence threshold (0.0 - 1.0)
    pub min_confidence: f32,

    /// Whether to suggest new components for unmapped
    pub suggest_new_components: bool,

    /// Whether to use AI-assisted naming
    pub ai_assisted: bool,

    /// Strict mode - fail on low confidence
    pub strict: bool,
}

impl Default for MapperConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.6,
            suggest_new_components: true,
            ai_assisted: false,
            strict: false,
        }
    }
}

/// A mapping rule
#[derive(Debug, Clone)]
struct MappingRule {
    /// OxideKit component name
    target: OxideKitComponent,

    /// Name patterns to match
    name_patterns: Vec<Regex>,

    /// Structural requirements
    structure: Option<StructureRequirement>,

    /// Priority (higher = checked first)
    priority: u32,
}

/// Known OxideKit components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum OxideKitComponent {
    Button,
    IconButton,
    Card,
    Input,
    TextArea,
    Checkbox,
    Radio,
    Switch,
    Select,
    Tabs,
    TabPanel,
    Table,
    Modal,
    Dialog,
    Navbar,
    Toolbar,
    Sidebar,
    Sidenav,
    StatCard,
    Avatar,
    Badge,
    Chip,
    Tooltip,
    Dropdown,
    Menu,
    MenuItem,
    List,
    ListItem,
    Divider,
    Spinner,
    Progress,
    Alert,
    Toast,
    Breadcrumb,
    Pagination,
    /// Custom component (name provided)
    Custom(String),
}

impl OxideKitComponent {
    /// Get component name as string
    pub fn name(&self) -> &str {
        match self {
            Self::Button => "Button",
            Self::IconButton => "IconButton",
            Self::Card => "Card",
            Self::Input => "Input",
            Self::TextArea => "TextArea",
            Self::Checkbox => "Checkbox",
            Self::Radio => "Radio",
            Self::Switch => "Switch",
            Self::Select => "Select",
            Self::Tabs => "Tabs",
            Self::TabPanel => "TabPanel",
            Self::Table => "Table",
            Self::Modal => "Modal",
            Self::Dialog => "Dialog",
            Self::Navbar => "Navbar",
            Self::Toolbar => "Toolbar",
            Self::Sidebar => "Sidebar",
            Self::Sidenav => "Sidenav",
            Self::StatCard => "StatCard",
            Self::Avatar => "Avatar",
            Self::Badge => "Badge",
            Self::Chip => "Chip",
            Self::Tooltip => "Tooltip",
            Self::Dropdown => "Dropdown",
            Self::Menu => "Menu",
            Self::MenuItem => "MenuItem",
            Self::List => "List",
            Self::ListItem => "ListItem",
            Self::Divider => "Divider",
            Self::Spinner => "Spinner",
            Self::Progress => "Progress",
            Self::Alert => "Alert",
            Self::Toast => "Toast",
            Self::Breadcrumb => "Breadcrumb",
            Self::Pagination => "Pagination",
            Self::Custom(name) => name,
        }
    }
}

/// Structural requirement for mapping
#[derive(Debug, Clone)]
struct StructureRequirement {
    /// Required layout mode
    layout_mode: Option<LayoutMode>,

    /// Required child count range
    child_count: Option<(usize, usize)>,

    /// Required child types
    child_types: Vec<NodeType>,

    /// Must have specific props (e.g., text content)
    has_text: Option<bool>,

    /// Must have icon
    has_icon: Option<bool>,

    /// Size constraints
    size_range: Option<SizeRange>,
}

/// Size range constraint
#[derive(Debug, Clone, Copy)]
struct SizeRange {
    min_width: Option<f32>,
    max_width: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
}

/// Result of component mapping
#[derive(Debug, Clone)]
pub struct ComponentMapping {
    /// Original Figma node
    pub figma_node_id: String,

    /// Original Figma name
    pub figma_name: String,

    /// Mapped OxideKit component
    pub component: OxideKitComponent,

    /// Mapping confidence (0.0 - 1.0)
    pub confidence: MappingConfidence,

    /// Extracted props
    pub props: HashMap<String, PropValue>,

    /// Extracted variants
    pub variants: Vec<VariantMapping>,

    /// Extracted slots (children)
    pub slots: Vec<SlotMapping>,

    /// Warnings/suggestions
    pub warnings: Vec<String>,
}

/// Confidence level of mapping
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MappingConfidence(pub f32);

impl MappingConfidence {
    pub fn high() -> Self { Self(0.9) }
    pub fn medium() -> Self { Self(0.7) }
    pub fn low() -> Self { Self(0.5) }
    pub fn uncertain() -> Self { Self(0.3) }

    pub fn is_confident(&self) -> bool { self.0 >= 0.6 }
    pub fn value(&self) -> f32 { self.0 }
}

/// Prop value
#[derive(Debug, Clone)]
pub enum PropValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Enum(String),
    Token(String),
}

/// Variant mapping
#[derive(Debug, Clone)]
pub struct VariantMapping {
    pub prop_name: String,
    pub variant_value: String,
    pub figma_variant_name: String,
}

/// Slot mapping
#[derive(Debug, Clone)]
pub struct SlotMapping {
    pub slot_name: String,
    pub figma_node_id: String,
    pub content_type: SlotContentType,
}

/// Slot content type
#[derive(Debug, Clone)]
pub enum SlotContentType {
    Text,
    Icon,
    Component(OxideKitComponent),
    Mixed,
}

/// Component registry (known OxideKit components)
#[derive(Debug, Default)]
struct ComponentRegistry {
    components: HashMap<String, RegisteredComponent>,
}

/// A registered component
#[derive(Debug, Clone)]
struct RegisteredComponent {
    name: String,
    props: Vec<String>,
    slots: Vec<String>,
    variants: Vec<String>,
}

impl ComponentMapper {
    /// Create a new component mapper
    pub fn new() -> Self {
        Self::with_config(MapperConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: MapperConfig) -> Self {
        let rules = Self::build_default_rules();
        let registry = Self::build_default_registry();

        Self {
            rules,
            registry,
            config,
        }
    }

    /// Build default mapping rules
    fn build_default_rules() -> Vec<MappingRule> {
        vec![
            // Button
            MappingRule {
                target: OxideKitComponent::Button,
                name_patterns: vec![
                    Regex::new(r"(?i)^button$").unwrap(),
                    Regex::new(r"(?i)btn").unwrap(),
                    Regex::new(r"(?i)cta").unwrap(),
                    Regex::new(r"(?i)action[-_]?button").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 3)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(40.0),
                        max_width: Some(400.0),
                        min_height: Some(24.0),
                        max_height: Some(64.0),
                    }),
                }),
                priority: 100,
            },
            // IconButton
            MappingRule {
                target: OxideKitComponent::IconButton,
                name_patterns: vec![
                    Regex::new(r"(?i)icon[-_]?button").unwrap(),
                    Regex::new(r"(?i)icon[-_]?btn").unwrap(),
                    Regex::new(r"(?i)button[-_]?icon").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((1, 1)),
                    child_types: vec![],
                    has_text: Some(false),
                    has_icon: Some(true),
                    size_range: Some(SizeRange {
                        min_width: Some(24.0),
                        max_width: Some(64.0),
                        min_height: Some(24.0),
                        max_height: Some(64.0),
                    }),
                }),
                priority: 95,
            },
            // Card
            MappingRule {
                target: OxideKitComponent::Card,
                name_patterns: vec![
                    Regex::new(r"(?i)^card$").unwrap(),
                    Regex::new(r"(?i)card[-_]").unwrap(),
                    Regex::new(r"(?i)[-_]card$").unwrap(),
                    Regex::new(r"(?i)panel").unwrap(),
                    Regex::new(r"(?i)tile").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Vertical),
                    child_count: Some((1, 20)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(100.0),
                        max_width: None,
                        min_height: Some(50.0),
                        max_height: None,
                    }),
                }),
                priority: 80,
            },
            // Input
            MappingRule {
                target: OxideKitComponent::Input,
                name_patterns: vec![
                    Regex::new(r"(?i)^input$").unwrap(),
                    Regex::new(r"(?i)text[-_]?field").unwrap(),
                    Regex::new(r"(?i)text[-_]?input").unwrap(),
                    Regex::new(r"(?i)form[-_]?field").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 5)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(80.0),
                        max_width: Some(800.0),
                        min_height: Some(24.0),
                        max_height: Some(64.0),
                    }),
                }),
                priority: 90,
            },
            // TextArea
            MappingRule {
                target: OxideKitComponent::TextArea,
                name_patterns: vec![
                    Regex::new(r"(?i)text[-_]?area").unwrap(),
                    Regex::new(r"(?i)multiline").unwrap(),
                    Regex::new(r"(?i)textarea").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: None,
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(100.0),
                        max_width: None,
                        min_height: Some(60.0),
                        max_height: None,
                    }),
                }),
                priority: 85,
            },
            // Checkbox
            MappingRule {
                target: OxideKitComponent::Checkbox,
                name_patterns: vec![
                    Regex::new(r"(?i)checkbox").unwrap(),
                    Regex::new(r"(?i)check[-_]?box").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 3)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(16.0),
                        max_width: Some(300.0),
                        min_height: Some(16.0),
                        max_height: Some(48.0),
                    }),
                }),
                priority: 90,
            },
            // Radio
            MappingRule {
                target: OxideKitComponent::Radio,
                name_patterns: vec![
                    Regex::new(r"(?i)radio").unwrap(),
                    Regex::new(r"(?i)radio[-_]?button").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 3)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(16.0),
                        max_width: Some(300.0),
                        min_height: Some(16.0),
                        max_height: Some(48.0),
                    }),
                }),
                priority: 90,
            },
            // Switch
            MappingRule {
                target: OxideKitComponent::Switch,
                name_patterns: vec![
                    Regex::new(r"(?i)switch").unwrap(),
                    Regex::new(r"(?i)toggle").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 3)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(32.0),
                        max_width: Some(100.0),
                        min_height: Some(16.0),
                        max_height: Some(40.0),
                    }),
                }),
                priority: 90,
            },
            // Select
            MappingRule {
                target: OxideKitComponent::Select,
                name_patterns: vec![
                    Regex::new(r"(?i)select").unwrap(),
                    Regex::new(r"(?i)dropdown").unwrap(),
                    Regex::new(r"(?i)picker").unwrap(),
                    Regex::new(r"(?i)combo[-_]?box").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 5)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(80.0),
                        max_width: Some(400.0),
                        min_height: Some(24.0),
                        max_height: Some(64.0),
                    }),
                }),
                priority: 85,
            },
            // Tabs
            MappingRule {
                target: OxideKitComponent::Tabs,
                name_patterns: vec![
                    Regex::new(r"(?i)tabs").unwrap(),
                    Regex::new(r"(?i)tab[-_]?bar").unwrap(),
                    Regex::new(r"(?i)tab[-_]?list").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((2, 20)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: None,
                }),
                priority: 80,
            },
            // Table
            MappingRule {
                target: OxideKitComponent::Table,
                name_patterns: vec![
                    Regex::new(r"(?i)table").unwrap(),
                    Regex::new(r"(?i)data[-_]?grid").unwrap(),
                    Regex::new(r"(?i)datagrid").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Vertical),
                    child_count: Some((2, 100)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(200.0),
                        max_width: None,
                        min_height: Some(100.0),
                        max_height: None,
                    }),
                }),
                priority: 75,
            },
            // Modal
            MappingRule {
                target: OxideKitComponent::Modal,
                name_patterns: vec![
                    Regex::new(r"(?i)modal").unwrap(),
                    Regex::new(r"(?i)overlay").unwrap(),
                    Regex::new(r"(?i)popup").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((1, 20)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(200.0),
                        max_width: None,
                        min_height: Some(100.0),
                        max_height: None,
                    }),
                }),
                priority: 70,
            },
            // Dialog
            MappingRule {
                target: OxideKitComponent::Dialog,
                name_patterns: vec![
                    Regex::new(r"(?i)dialog").unwrap(),
                    Regex::new(r"(?i)alert[-_]?dialog").unwrap(),
                    Regex::new(r"(?i)confirm").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Vertical),
                    child_count: Some((2, 10)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(200.0),
                        max_width: Some(600.0),
                        min_height: Some(100.0),
                        max_height: Some(600.0),
                    }),
                }),
                priority: 75,
            },
            // Navbar
            MappingRule {
                target: OxideKitComponent::Navbar,
                name_patterns: vec![
                    Regex::new(r"(?i)navbar").unwrap(),
                    Regex::new(r"(?i)nav[-_]?bar").unwrap(),
                    Regex::new(r"(?i)header").unwrap(),
                    Regex::new(r"(?i)top[-_]?bar").unwrap(),
                    Regex::new(r"(?i)app[-_]?bar").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 20)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(300.0),
                        max_width: None,
                        min_height: Some(40.0),
                        max_height: Some(100.0),
                    }),
                }),
                priority: 70,
            },
            // Toolbar
            MappingRule {
                target: OxideKitComponent::Toolbar,
                name_patterns: vec![
                    Regex::new(r"(?i)toolbar").unwrap(),
                    Regex::new(r"(?i)tool[-_]?bar").unwrap(),
                    Regex::new(r"(?i)action[-_]?bar").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((2, 20)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(100.0),
                        max_width: None,
                        min_height: Some(32.0),
                        max_height: Some(80.0),
                    }),
                }),
                priority: 70,
            },
            // Sidebar
            MappingRule {
                target: OxideKitComponent::Sidebar,
                name_patterns: vec![
                    Regex::new(r"(?i)sidebar").unwrap(),
                    Regex::new(r"(?i)side[-_]?bar").unwrap(),
                    Regex::new(r"(?i)sidenav").unwrap(),
                    Regex::new(r"(?i)side[-_]?nav").unwrap(),
                    Regex::new(r"(?i)navigation").unwrap(),
                    Regex::new(r"(?i)drawer").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Vertical),
                    child_count: Some((1, 50)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(48.0),
                        max_width: Some(400.0),
                        min_height: Some(200.0),
                        max_height: None,
                    }),
                }),
                priority: 70,
            },
            // StatCard
            MappingRule {
                target: OxideKitComponent::StatCard,
                name_patterns: vec![
                    Regex::new(r"(?i)stat[-_]?card").unwrap(),
                    Regex::new(r"(?i)stats").unwrap(),
                    Regex::new(r"(?i)metric").unwrap(),
                    Regex::new(r"(?i)kpi").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Vertical),
                    child_count: Some((2, 6)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(100.0),
                        max_width: Some(400.0),
                        min_height: Some(60.0),
                        max_height: Some(200.0),
                    }),
                }),
                priority: 75,
            },
            // Avatar
            MappingRule {
                target: OxideKitComponent::Avatar,
                name_patterns: vec![
                    Regex::new(r"(?i)avatar").unwrap(),
                    Regex::new(r"(?i)profile[-_]?pic").unwrap(),
                    Regex::new(r"(?i)user[-_]?image").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((0, 2)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(16.0),
                        max_width: Some(128.0),
                        min_height: Some(16.0),
                        max_height: Some(128.0),
                    }),
                }),
                priority: 85,
            },
            // Badge
            MappingRule {
                target: OxideKitComponent::Badge,
                name_patterns: vec![
                    Regex::new(r"(?i)badge").unwrap(),
                    Regex::new(r"(?i)tag").unwrap(),
                    Regex::new(r"(?i)label").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((0, 2)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(16.0),
                        max_width: Some(100.0),
                        min_height: Some(16.0),
                        max_height: Some(32.0),
                    }),
                }),
                priority: 85,
            },
            // Alert
            MappingRule {
                target: OxideKitComponent::Alert,
                name_patterns: vec![
                    Regex::new(r"(?i)alert").unwrap(),
                    Regex::new(r"(?i)banner").unwrap(),
                    Regex::new(r"(?i)notification").unwrap(),
                    Regex::new(r"(?i)message").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: Some(LayoutMode::Horizontal),
                    child_count: Some((1, 5)),
                    child_types: vec![],
                    has_text: Some(true),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(200.0),
                        max_width: None,
                        min_height: Some(40.0),
                        max_height: Some(150.0),
                    }),
                }),
                priority: 75,
            },
            // Spinner
            MappingRule {
                target: OxideKitComponent::Spinner,
                name_patterns: vec![
                    Regex::new(r"(?i)spinner").unwrap(),
                    Regex::new(r"(?i)loading").unwrap(),
                    Regex::new(r"(?i)loader").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((0, 2)),
                    child_types: vec![],
                    has_text: Some(false),
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(16.0),
                        max_width: Some(64.0),
                        min_height: Some(16.0),
                        max_height: Some(64.0),
                    }),
                }),
                priority: 85,
            },
            // Progress
            MappingRule {
                target: OxideKitComponent::Progress,
                name_patterns: vec![
                    Regex::new(r"(?i)progress").unwrap(),
                    Regex::new(r"(?i)progress[-_]?bar").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((0, 3)),
                    child_types: vec![],
                    has_text: None,
                    has_icon: None,
                    size_range: Some(SizeRange {
                        min_width: Some(50.0),
                        max_width: None,
                        min_height: Some(4.0),
                        max_height: Some(32.0),
                    }),
                }),
                priority: 85,
            },
            // Divider
            MappingRule {
                target: OxideKitComponent::Divider,
                name_patterns: vec![
                    Regex::new(r"(?i)divider").unwrap(),
                    Regex::new(r"(?i)separator").unwrap(),
                    Regex::new(r"(?i)hr").unwrap(),
                ],
                structure: Some(StructureRequirement {
                    layout_mode: None,
                    child_count: Some((0, 0)),
                    child_types: vec![],
                    has_text: Some(false),
                    has_icon: Some(false),
                    size_range: Some(SizeRange {
                        min_width: None,
                        max_width: None,
                        min_height: Some(0.5),
                        max_height: Some(4.0),
                    }),
                }),
                priority: 90,
            },
        ]
    }

    /// Build default component registry
    fn build_default_registry() -> ComponentRegistry {
        let mut registry = ComponentRegistry::default();

        // Register known components with their props and slots
        registry.register("Button", vec!["variant", "size", "disabled"], vec!["default", "icon"]);
        registry.register("IconButton", vec!["variant", "size", "disabled"], vec!["icon"]);
        registry.register("Card", vec!["variant", "padding"], vec!["header", "default", "footer"]);
        registry.register("Input", vec!["variant", "size", "disabled", "placeholder"], vec!["prefix", "suffix"]);
        registry.register("Checkbox", vec!["checked", "disabled", "indeterminate"], vec!["label"]);
        registry.register("Radio", vec!["checked", "disabled"], vec!["label"]);
        registry.register("Switch", vec!["checked", "disabled"], vec!["label"]);
        registry.register("Select", vec!["variant", "size", "disabled", "placeholder"], vec!["options"]);
        registry.register("Tabs", vec!["variant", "size"], vec!["tabs"]);
        registry.register("Table", vec!["variant", "striped"], vec!["columns", "rows"]);
        registry.register("Modal", vec!["open", "size"], vec!["header", "default", "footer"]);
        registry.register("Dialog", vec!["open", "variant"], vec!["title", "default", "actions"]);
        registry.register("Navbar", vec!["variant", "sticky"], vec!["brand", "nav", "actions"]);
        registry.register("Sidebar", vec!["collapsed", "width"], vec!["header", "nav", "footer"]);
        registry.register("StatCard", vec!["variant"], vec!["icon", "value", "label", "change"]);
        registry.register("Avatar", vec!["size", "variant"], vec!["default", "fallback"]);
        registry.register("Badge", vec!["variant", "size"], vec!["default"]);
        registry.register("Alert", vec!["variant", "closable"], vec!["icon", "title", "description"]);
        registry.register("Spinner", vec!["size", "variant"], vec![]);
        registry.register("Progress", vec!["value", "max", "variant"], vec![]);
        registry.register("Divider", vec!["variant", "orientation"], vec![]);

        registry
    }

    /// Map a single Figma node to OxideKit component
    pub fn map_node(&self, node: &Node) -> Result<Option<ComponentMapping>> {
        // Skip invisible nodes
        if !node.visible {
            return Ok(None);
        }

        // Only map frames, components, and instances
        match node.node_type {
            NodeType::Frame | NodeType::Component | NodeType::Instance => {}
            _ => return Ok(None),
        }

        // Try each rule in priority order
        let mut best_match: Option<(MappingRule, f32)> = None;

        for rule in &self.rules {
            let confidence = self.evaluate_rule(rule, node);

            if confidence > self.config.min_confidence {
                if best_match.is_none() || confidence > best_match.as_ref().unwrap().1 {
                    best_match = Some((rule.clone(), confidence));
                }
            }
        }

        if let Some((rule, confidence)) = best_match {
            let mapping = self.create_mapping(node, &rule, confidence)?;
            return Ok(Some(mapping));
        }

        // No match found - suggest new component if enabled
        if self.config.suggest_new_components {
            warn!(
                node_name = %node.name,
                "No matching component found, suggesting custom component"
            );

            let mapping = ComponentMapping {
                figma_node_id: node.id.clone(),
                figma_name: node.name.clone(),
                component: OxideKitComponent::Custom(self.to_pascal_case(&node.name)),
                confidence: MappingConfidence::uncertain(),
                props: HashMap::new(),
                variants: Vec::new(),
                slots: Vec::new(),
                warnings: vec![format!(
                    "No OxideKit component matches '{}'. Consider creating a custom component.",
                    node.name
                )],
            };

            return Ok(Some(mapping));
        }

        Ok(None)
    }

    /// Map all components in a Figma file
    pub fn map_file(&self, file: &FigmaFile) -> Result<Vec<ComponentMapping>> {
        info!(file_name = %file.name, "Mapping Figma file to OxideKit components");

        let mut mappings = Vec::new();

        // Walk document tree
        for page in &file.document.children {
            self.map_node_recursive(page, &mut mappings)?;
        }

        info!(
            total_mappings = mappings.len(),
            "Component mapping complete"
        );

        Ok(mappings)
    }

    /// Recursively map nodes
    fn map_node_recursive(&self, node: &Node, mappings: &mut Vec<ComponentMapping>) -> Result<()> {
        // Try to map this node
        if let Some(mapping) = self.map_node(node)? {
            mappings.push(mapping);
        }

        // Recurse into children
        for child in &node.children {
            self.map_node_recursive(child, mappings)?;
        }

        Ok(())
    }

    /// Evaluate a rule against a node
    fn evaluate_rule(&self, rule: &MappingRule, node: &Node) -> f32 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Name pattern matching (weight: 0.4)
        max_score += 0.4;
        for pattern in &rule.name_patterns {
            if pattern.is_match(&node.name) {
                score += 0.4;
                break;
            }
        }

        // Structure requirements (weight: 0.6)
        if let Some(req) = &rule.structure {
            // Layout mode
            if let Some(expected_mode) = &req.layout_mode {
                max_score += 0.15;
                if node.layout_mode.as_ref() == Some(expected_mode) {
                    score += 0.15;
                }
            }

            // Child count
            if let Some((min, max)) = req.child_count {
                max_score += 0.1;
                let count = node.children.len();
                if count >= min && count <= max {
                    score += 0.1;
                }
            }

            // Has text
            if let Some(expected_text) = req.has_text {
                max_score += 0.1;
                let has_text = self.node_has_text(node);
                if has_text == expected_text {
                    score += 0.1;
                }
            }

            // Size range
            if let Some(size) = &req.size_range {
                max_score += 0.15;
                if self.node_matches_size(node, size) {
                    score += 0.15;
                }
            }

            // Has icon
            if let Some(expected_icon) = req.has_icon {
                max_score += 0.1;
                let has_icon = self.node_has_icon(node);
                if has_icon == expected_icon {
                    score += 0.1;
                }
            }
        }

        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }

    /// Check if node has text content
    fn node_has_text(&self, node: &Node) -> bool {
        if node.node_type == NodeType::Text {
            return true;
        }

        for child in &node.children {
            if self.node_has_text(child) {
                return true;
            }
        }

        false
    }

    /// Check if node has icon
    fn node_has_icon(&self, node: &Node) -> bool {
        let name_lower = node.name.to_lowercase();
        if name_lower.contains("icon") || name_lower.contains("svg") {
            return true;
        }

        // Check for vector children (likely icons)
        for child in &node.children {
            match child.node_type {
                NodeType::Vector | NodeType::BooleanOperation => return true,
                _ => {}
            }
            if self.node_has_icon(child) {
                return true;
            }
        }

        false
    }

    /// Check if node matches size constraints
    fn node_matches_size(&self, node: &Node, size: &SizeRange) -> bool {
        let bbox = match &node.absolute_bounding_box {
            Some(b) => b,
            None => return true, // No size info, assume match
        };

        if let Some(min) = size.min_width {
            if bbox.width < min {
                return false;
            }
        }
        if let Some(max) = size.max_width {
            if bbox.width > max {
                return false;
            }
        }
        if let Some(min) = size.min_height {
            if bbox.height < min {
                return false;
            }
        }
        if let Some(max) = size.max_height {
            if bbox.height > max {
                return false;
            }
        }

        true
    }

    /// Create a mapping from a matched rule
    fn create_mapping(&self, node: &Node, rule: &MappingRule, confidence: f32) -> Result<ComponentMapping> {
        let mut props = HashMap::new();
        let mut warnings = Vec::new();

        // Extract props based on component type
        self.extract_props(node, &rule.target, &mut props);

        // Extract variants
        let variants = self.extract_variants(node, &rule.target);

        // Extract slots
        let slots = self.extract_slots(node, &rule.target);

        // Add warnings for low confidence
        if confidence < 0.7 {
            warnings.push(format!(
                "Low confidence mapping ({:.0}%). Verify this is the correct component.",
                confidence * 100.0
            ));
        }

        Ok(ComponentMapping {
            figma_node_id: node.id.clone(),
            figma_name: node.name.clone(),
            component: rule.target.clone(),
            confidence: MappingConfidence(confidence),
            props,
            variants,
            slots,
            warnings,
        })
    }

    /// Extract props from node
    fn extract_props(&self, node: &Node, component: &OxideKitComponent, props: &mut HashMap<String, PropValue>) {
        match component {
            OxideKitComponent::Button => {
                // Detect variant from name
                let name_lower = node.name.to_lowercase();
                if name_lower.contains("primary") {
                    props.insert("variant".into(), PropValue::Enum("primary".into()));
                } else if name_lower.contains("secondary") {
                    props.insert("variant".into(), PropValue::Enum("secondary".into()));
                } else if name_lower.contains("outline") {
                    props.insert("variant".into(), PropValue::Enum("outline".into()));
                } else if name_lower.contains("ghost") {
                    props.insert("variant".into(), PropValue::Enum("ghost".into()));
                }

                // Detect size
                if let Some(bbox) = &node.absolute_bounding_box {
                    let size = if bbox.height <= 28.0 {
                        "sm"
                    } else if bbox.height <= 40.0 {
                        "md"
                    } else {
                        "lg"
                    };
                    props.insert("size".into(), PropValue::Enum(size.into()));
                }

                // Detect disabled state
                if name_lower.contains("disabled") {
                    props.insert("disabled".into(), PropValue::Boolean(true));
                }
            }

            OxideKitComponent::Input => {
                // Detect variant
                let name_lower = node.name.to_lowercase();
                if name_lower.contains("outline") {
                    props.insert("variant".into(), PropValue::Enum("outline".into()));
                } else if name_lower.contains("filled") {
                    props.insert("variant".into(), PropValue::Enum("filled".into()));
                }

                // Detect placeholder text
                for child in &node.children {
                    if child.node_type == NodeType::Text {
                        if let Some(text) = &child.characters {
                            if text.contains("placeholder") || child.opacity < 1.0 {
                                props.insert("placeholder".into(), PropValue::String(text.clone()));
                            }
                        }
                    }
                }
            }

            OxideKitComponent::Checkbox | OxideKitComponent::Radio => {
                let name_lower = node.name.to_lowercase();
                if name_lower.contains("checked") || name_lower.contains("selected") {
                    props.insert("checked".into(), PropValue::Boolean(true));
                }
                if name_lower.contains("disabled") {
                    props.insert("disabled".into(), PropValue::Boolean(true));
                }
            }

            OxideKitComponent::Switch => {
                let name_lower = node.name.to_lowercase();
                if name_lower.contains("on") || name_lower.contains("active") {
                    props.insert("checked".into(), PropValue::Boolean(true));
                }
            }

            _ => {}
        }
    }

    /// Extract variants from node (for component instances)
    fn extract_variants(&self, node: &Node, _component: &OxideKitComponent) -> Vec<VariantMapping> {
        let mut variants = Vec::new();

        // Parse Figma component properties
        for (key, prop) in &node.component_properties {
            if prop.property_type == ComponentPropertyType::Variant {
                if let Some(value) = prop.value.as_str() {
                    variants.push(VariantMapping {
                        prop_name: key.clone(),
                        variant_value: value.to_string(),
                        figma_variant_name: format!("{}={}", key, value),
                    });
                }
            }
        }

        variants
    }

    /// Extract slots from node
    fn extract_slots(&self, node: &Node, component: &OxideKitComponent) -> Vec<SlotMapping> {
        let mut slots = Vec::new();

        // Check registered slots for this component
        if let Some(registered) = self.registry.components.get(component.name()) {
            for slot_name in &registered.slots {
                // Look for matching child
                for child in &node.children {
                    let child_name_lower = child.name.to_lowercase();
                    if child_name_lower.contains(&slot_name.to_lowercase()) {
                        let content_type = self.detect_content_type(child);
                        slots.push(SlotMapping {
                            slot_name: slot_name.clone(),
                            figma_node_id: child.id.clone(),
                            content_type,
                        });
                        break;
                    }
                }
            }
        }

        // Default slot for remaining children
        if slots.is_empty() && !node.children.is_empty() {
            for child in &node.children {
                let content_type = self.detect_content_type(child);
                slots.push(SlotMapping {
                    slot_name: "default".to_string(),
                    figma_node_id: child.id.clone(),
                    content_type,
                });
            }
        }

        slots
    }

    /// Detect content type of a node
    fn detect_content_type(&self, node: &Node) -> SlotContentType {
        match node.node_type {
            NodeType::Text => SlotContentType::Text,
            NodeType::Vector | NodeType::BooleanOperation => SlotContentType::Icon,
            NodeType::Frame | NodeType::Component | NodeType::Instance => {
                // Check if it maps to a known component
                if let Ok(Some(mapping)) = self.map_node(node) {
                    SlotContentType::Component(mapping.component)
                } else {
                    SlotContentType::Mixed
                }
            }
            _ => SlotContentType::Mixed,
        }
    }

    /// Convert to PascalCase
    fn to_pascal_case(&self, s: &str) -> String {
        s.split(|c: char| !c.is_alphanumeric())
            .filter(|s| !s.is_empty())
            .map(|word| {
                let mut chars = word.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }
}

impl ComponentRegistry {
    fn register(&mut self, name: &str, props: Vec<&str>, slots: Vec<&str>) {
        self.components.insert(
            name.to_string(),
            RegisteredComponent {
                name: name.to_string(),
                props: props.into_iter().map(String::from).collect(),
                slots: slots.into_iter().map(String::from).collect(),
                variants: Vec::new(),
            },
        );
    }
}

impl Default for ComponentMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_name() {
        assert_eq!(OxideKitComponent::Button.name(), "Button");
        assert_eq!(OxideKitComponent::Custom("MyComponent".into()).name(), "MyComponent");
    }

    #[test]
    fn test_mapping_confidence() {
        assert!(MappingConfidence::high().is_confident());
        assert!(MappingConfidence::medium().is_confident());
        assert!(!MappingConfidence::uncertain().is_confident());
    }

    #[test]
    fn test_to_pascal_case() {
        let mapper = ComponentMapper::new();
        assert_eq!(mapper.to_pascal_case("hello-world"), "HelloWorld");
        assert_eq!(mapper.to_pascal_case("some_component"), "SomeComponent");
        assert_eq!(mapper.to_pascal_case("My Button"), "MyButton");
    }
}
