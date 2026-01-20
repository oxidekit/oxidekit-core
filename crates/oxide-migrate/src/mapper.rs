//! Layout and Component Mapper
//!
//! Maps HTML structures and CSS framework components to OxideKit constructs.
//! Provides semantic mapping for sidebar, navbar, tables, cards, and other
//! common UI patterns.

use crate::analyzer::{AnalysisResult, ComponentType, DetectedComponent, Framework};
use crate::error::{IssueCategory, MigrateError, MigrateResult, MigrationIssue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Mapping result for a complete migration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingResult {
    /// Layout mapping (overall page structure)
    pub layout: LayoutMapping,
    /// Individual component mappings
    pub components: Vec<ComponentMapping>,
    /// Design pack parts generated
    pub design_parts: Vec<DesignPart>,
    /// Issues found during mapping
    pub issues: Vec<MigrationIssue>,
    /// Overall mapping confidence (0.0 to 1.0)
    pub confidence: f32,
}

/// Layout mapping for page structure
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LayoutMapping {
    /// Detected layout pattern
    pub pattern: LayoutPattern,
    /// Sidebar configuration (if detected)
    pub sidebar: Option<SidebarConfig>,
    /// Navbar configuration (if detected)
    pub navbar: Option<NavbarConfig>,
    /// Main content area configuration
    pub content: ContentConfig,
    /// Footer configuration (if detected)
    pub footer: Option<FooterConfig>,
    /// Grid system detected
    pub grid_system: GridSystem,
}

/// Common layout patterns
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LayoutPattern {
    /// No specific pattern detected
    #[default]
    Unknown,
    /// Sidebar + main content (admin dashboard style)
    SidebarLayout,
    /// Top navbar + main content
    NavbarLayout,
    /// Both sidebar and navbar
    FullDashboard,
    /// Simple single-column layout
    SingleColumn,
    /// Multi-column grid layout
    GridLayout,
    /// Landing page with hero section
    LandingPage,
}

impl std::fmt::Display for LayoutPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LayoutPattern::Unknown => write!(f, "Unknown"),
            LayoutPattern::SidebarLayout => write!(f, "Sidebar Layout"),
            LayoutPattern::NavbarLayout => write!(f, "Navbar Layout"),
            LayoutPattern::FullDashboard => write!(f, "Full Dashboard"),
            LayoutPattern::SingleColumn => write!(f, "Single Column"),
            LayoutPattern::GridLayout => write!(f, "Grid Layout"),
            LayoutPattern::LandingPage => write!(f, "Landing Page"),
        }
    }
}

/// Sidebar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SidebarConfig {
    /// Position (left or right)
    pub position: SidebarPosition,
    /// Width in pixels (if detected)
    pub width: Option<f32>,
    /// Is collapsible
    pub collapsible: bool,
    /// Has nested menu items
    pub has_nested_menus: bool,
    /// Has icons
    pub has_icons: bool,
    /// OxideKit component to use
    pub oxide_component: String,
    /// Menu items detected
    pub menu_items: Vec<MenuItemDef>,
}

/// Sidebar position
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SidebarPosition {
    #[default]
    Left,
    Right,
}

/// Menu item definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemDef {
    /// Menu item label
    pub label: String,
    /// Icon name (if detected)
    pub icon: Option<String>,
    /// Link/route
    pub link: Option<String>,
    /// Nested children
    pub children: Vec<MenuItemDef>,
    /// Is active/selected
    pub is_active: bool,
}

/// Navbar configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavbarConfig {
    /// Is fixed to top
    pub is_fixed: bool,
    /// Is transparent
    pub is_transparent: bool,
    /// Has brand/logo
    pub has_brand: bool,
    /// Brand text (if any)
    pub brand_text: Option<String>,
    /// Has search
    pub has_search: bool,
    /// Has user menu
    pub has_user_menu: bool,
    /// Has notifications
    pub has_notifications: bool,
    /// OxideKit component to use
    pub oxide_component: String,
}

/// Content area configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContentConfig {
    /// Max width (if constrained)
    pub max_width: Option<f32>,
    /// Uses container class
    pub uses_container: bool,
    /// Has breadcrumbs
    pub has_breadcrumbs: bool,
    /// Has page header
    pub has_page_header: bool,
}

/// Footer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FooterConfig {
    /// Is fixed to bottom
    pub is_fixed: bool,
    /// Has copyright
    pub has_copyright: bool,
    /// Has links
    pub has_links: bool,
    /// OxideKit component to use
    pub oxide_component: String,
}

/// Grid system detection
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GridSystem {
    /// Number of columns
    pub columns: u8,
    /// Gutter size in pixels
    pub gutter: Option<f32>,
    /// Breakpoints detected
    pub breakpoints: Vec<Breakpoint>,
    /// OxideKit equivalent
    pub oxide_equivalent: String,
}

/// Breakpoint definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Breakpoint {
    /// Breakpoint name (xs, sm, md, lg, xl)
    pub name: String,
    /// Min width in pixels
    pub min_width: f32,
}

/// Individual component mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMapping {
    /// Original component type
    pub source_type: ComponentType,
    /// Source CSS classes
    pub source_classes: Vec<String>,
    /// Target OxideKit component
    pub target_component: String,
    /// Property mappings
    pub prop_mappings: HashMap<String, String>,
    /// Mapping confidence (0.0 to 1.0)
    pub confidence: f32,
    /// Manual review needed
    pub needs_review: bool,
    /// Mapping notes
    pub notes: Vec<String>,
}

/// Design pack part definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPart {
    /// Part tag (e.g., "part:sidebar", "part:navbar")
    pub tag: String,
    /// Part name
    pub name: String,
    /// Description
    pub description: String,
    /// OxideKit component references
    pub components: Vec<String>,
    /// TOML configuration for this part
    pub config_toml: String,
    /// Confidence score
    pub confidence: f32,
}

/// Component mapper
pub struct ComponentMapper {
    /// Component mapping rules by framework
    mapping_rules: HashMap<Framework, Vec<MappingRule>>,
}

/// A mapping rule
struct MappingRule {
    /// Source component type
    source: ComponentType,
    /// Target OxideKit component
    target: &'static str,
    /// Confidence modifier
    confidence: f32,
    /// Property mappings
    props: Vec<(&'static str, &'static str)>,
}

impl ComponentMapper {
    /// Create a new component mapper
    pub fn new() -> Self {
        let mut mapping_rules = HashMap::new();

        // Bootstrap mappings
        mapping_rules.insert(Framework::Bootstrap, Self::bootstrap_rules());

        // Tailwind mappings
        mapping_rules.insert(Framework::Tailwind, Self::tailwind_rules());

        // Generic/custom mappings
        mapping_rules.insert(Framework::Custom, Self::generic_rules());

        Self { mapping_rules }
    }

    fn bootstrap_rules() -> Vec<MappingRule> {
        vec![
            MappingRule {
                source: ComponentType::Button,
                target: "ui.Button",
                confidence: 0.95,
                props: vec![
                    ("btn-primary", "variant=\"primary\""),
                    ("btn-secondary", "variant=\"secondary\""),
                    ("btn-success", "variant=\"success\""),
                    ("btn-danger", "variant=\"danger\""),
                    ("btn-warning", "variant=\"warning\""),
                    ("btn-info", "variant=\"info\""),
                    ("btn-outline-*", "variant=\"outline\""),
                    ("btn-sm", "size=\"sm\""),
                    ("btn-lg", "size=\"lg\""),
                ],
            },
            MappingRule {
                source: ComponentType::Card,
                target: "ui.Card",
                confidence: 0.9,
                props: vec![
                    ("card-header", "ui.Card.Header"),
                    ("card-body", "ui.Card.Body"),
                    ("card-footer", "ui.Card.Footer"),
                    ("card-title", "ui.Card.Title"),
                ],
            },
            MappingRule {
                source: ComponentType::Navbar,
                target: "ui.Toolbar",
                confidence: 0.85,
                props: vec![
                    ("navbar-brand", "ui.Toolbar.Brand"),
                    ("navbar-nav", "ui.Toolbar.Nav"),
                    ("navbar-expand-*", "responsive"),
                    ("navbar-dark", "theme=\"dark\""),
                    ("navbar-light", "theme=\"light\""),
                    ("fixed-top", "fixed"),
                ],
            },
            MappingRule {
                source: ComponentType::Sidebar,
                target: "ui.Sidenav",
                confidence: 0.85,
                props: vec![("sidebar-collapse", "collapsible")],
            },
            MappingRule {
                source: ComponentType::Table,
                target: "ui.DataTable",
                confidence: 0.9,
                props: vec![
                    ("table-striped", "striped"),
                    ("table-bordered", "bordered"),
                    ("table-hover", "hoverable"),
                    ("table-sm", "size=\"compact\""),
                    ("table-responsive", "responsive"),
                ],
            },
            MappingRule {
                source: ComponentType::Modal,
                target: "ui.Dialog",
                confidence: 0.9,
                props: vec![
                    ("modal-dialog", "ui.Dialog.Content"),
                    ("modal-header", "ui.Dialog.Header"),
                    ("modal-body", "ui.Dialog.Body"),
                    ("modal-footer", "ui.Dialog.Footer"),
                    ("modal-lg", "size=\"lg\""),
                    ("modal-sm", "size=\"sm\""),
                ],
            },
            MappingRule {
                source: ComponentType::Alert,
                target: "ui.Alert",
                confidence: 0.95,
                props: vec![
                    ("alert-primary", "variant=\"primary\""),
                    ("alert-success", "variant=\"success\""),
                    ("alert-danger", "variant=\"danger\""),
                    ("alert-warning", "variant=\"warning\""),
                    ("alert-info", "variant=\"info\""),
                    ("alert-dismissible", "dismissible"),
                ],
            },
            MappingRule {
                source: ComponentType::Badge,
                target: "ui.Badge",
                confidence: 0.95,
                props: vec![
                    ("badge-primary", "variant=\"primary\""),
                    ("badge-secondary", "variant=\"secondary\""),
                    ("badge-pill", "pill"),
                ],
            },
            MappingRule {
                source: ComponentType::Dropdown,
                target: "ui.Dropdown",
                confidence: 0.9,
                props: vec![
                    ("dropdown-menu", "ui.Dropdown.Menu"),
                    ("dropdown-item", "ui.Dropdown.Item"),
                    ("dropdown-divider", "ui.Dropdown.Divider"),
                ],
            },
            MappingRule {
                source: ComponentType::Tabs,
                target: "ui.Tabs",
                confidence: 0.9,
                props: vec![
                    ("nav-tabs", "ui.Tabs.List"),
                    ("nav-link", "ui.Tabs.Tab"),
                    ("tab-pane", "ui.Tabs.Panel"),
                ],
            },
            MappingRule {
                source: ComponentType::Accordion,
                target: "ui.Accordion",
                confidence: 0.85,
                props: vec![
                    ("accordion-item", "ui.Accordion.Item"),
                    ("accordion-header", "ui.Accordion.Header"),
                    ("accordion-body", "ui.Accordion.Content"),
                ],
            },
            MappingRule {
                source: ComponentType::Progress,
                target: "ui.Progress",
                confidence: 0.95,
                props: vec![("progress-bar", "ui.Progress.Bar")],
            },
            MappingRule {
                source: ComponentType::Spinner,
                target: "ui.Spinner",
                confidence: 0.95,
                props: vec![
                    ("spinner-border", "variant=\"border\""),
                    ("spinner-grow", "variant=\"grow\""),
                ],
            },
            MappingRule {
                source: ComponentType::Toast,
                target: "ui.Toast",
                confidence: 0.9,
                props: vec![
                    ("toast-header", "ui.Toast.Header"),
                    ("toast-body", "ui.Toast.Body"),
                ],
            },
            MappingRule {
                source: ComponentType::Breadcrumb,
                target: "ui.Breadcrumb",
                confidence: 0.95,
                props: vec![("breadcrumb-item", "ui.Breadcrumb.Item")],
            },
            MappingRule {
                source: ComponentType::Pagination,
                target: "ui.Pagination",
                confidence: 0.9,
                props: vec![
                    ("page-item", "ui.Pagination.Item"),
                    ("page-link", "ui.Pagination.Link"),
                ],
            },
            MappingRule {
                source: ComponentType::Form,
                target: "ui.Form",
                confidence: 0.9,
                props: vec![
                    ("form-group", "ui.FormField"),
                    ("form-label", "ui.FormField.Label"),
                    ("form-text", "ui.FormField.Help"),
                ],
            },
            MappingRule {
                source: ComponentType::Input,
                target: "ui.Input",
                confidence: 0.95,
                props: vec![
                    ("form-control-lg", "size=\"lg\""),
                    ("form-control-sm", "size=\"sm\""),
                    ("is-valid", "state=\"valid\""),
                    ("is-invalid", "state=\"invalid\""),
                ],
            },
            MappingRule {
                source: ComponentType::Select,
                target: "ui.Select",
                confidence: 0.9,
                props: vec![("form-select", "ui.Select")],
            },
            MappingRule {
                source: ComponentType::Checkbox,
                target: "ui.Checkbox",
                confidence: 0.9,
                props: vec![("form-check", "ui.Checkbox")],
            },
            MappingRule {
                source: ComponentType::Switch,
                target: "ui.Switch",
                confidence: 0.9,
                props: vec![("form-switch", "ui.Switch")],
            },
        ]
    }

    fn tailwind_rules() -> Vec<MappingRule> {
        // Tailwind is utility-first, so mappings are more about patterns than classes
        vec![
            MappingRule {
                source: ComponentType::Button,
                target: "ui.Button",
                confidence: 0.85,
                props: vec![
                    ("bg-blue-*", "variant=\"primary\""),
                    ("bg-gray-*", "variant=\"secondary\""),
                    ("bg-green-*", "variant=\"success\""),
                    ("bg-red-*", "variant=\"danger\""),
                    ("rounded-full", "pill"),
                    ("px-2 py-1", "size=\"sm\""),
                    ("px-6 py-3", "size=\"lg\""),
                ],
            },
            MappingRule {
                source: ComponentType::Card,
                target: "ui.Card",
                confidence: 0.8,
                props: vec![("shadow", "elevated"), ("rounded-lg", "rounded")],
            },
            MappingRule {
                source: ComponentType::Navbar,
                target: "ui.Toolbar",
                confidence: 0.8,
                props: vec![
                    ("fixed", "fixed"),
                    ("sticky", "sticky"),
                    ("bg-white", "theme=\"light\""),
                    ("bg-gray-900", "theme=\"dark\""),
                ],
            },
            MappingRule {
                source: ComponentType::Sidebar,
                target: "ui.Sidenav",
                confidence: 0.8,
                props: vec![("w-64", "width=\"256\"")],
            },
            MappingRule {
                source: ComponentType::Table,
                target: "ui.DataTable",
                confidence: 0.85,
                props: vec![
                    ("divide-y", "striped"),
                    ("hover:bg-*", "hoverable"),
                ],
            },
            MappingRule {
                source: ComponentType::Modal,
                target: "ui.Dialog",
                confidence: 0.85,
                props: vec![("fixed inset-0", "fullscreen")],
            },
            MappingRule {
                source: ComponentType::Alert,
                target: "ui.Alert",
                confidence: 0.85,
                props: vec![
                    ("bg-blue-100", "variant=\"info\""),
                    ("bg-green-100", "variant=\"success\""),
                    ("bg-red-100", "variant=\"danger\""),
                    ("bg-yellow-100", "variant=\"warning\""),
                ],
            },
            MappingRule {
                source: ComponentType::Badge,
                target: "ui.Badge",
                confidence: 0.9,
                props: vec![
                    ("rounded-full", "pill"),
                    ("text-xs", "size=\"sm\""),
                ],
            },
            MappingRule {
                source: ComponentType::Form,
                target: "ui.Form",
                confidence: 0.85,
                props: vec![("space-y-4", "gap=\"md\"")],
            },
            MappingRule {
                source: ComponentType::Input,
                target: "ui.Input",
                confidence: 0.9,
                props: vec![
                    ("border-red-500", "state=\"invalid\""),
                    ("border-green-500", "state=\"valid\""),
                ],
            },
        ]
    }

    fn generic_rules() -> Vec<MappingRule> {
        // Generic rules that work across frameworks
        vec![
            MappingRule {
                source: ComponentType::Button,
                target: "ui.Button",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Card,
                target: "ui.Card",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Navbar,
                target: "ui.Toolbar",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Sidebar,
                target: "ui.Sidenav",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Table,
                target: "ui.DataTable",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Modal,
                target: "ui.Dialog",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Alert,
                target: "ui.Alert",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Form,
                target: "ui.Form",
                confidence: 0.7,
                props: vec![],
            },
            MappingRule {
                source: ComponentType::Input,
                target: "ui.Input",
                confidence: 0.7,
                props: vec![],
            },
        ]
    }

    /// Map components from analysis result
    pub fn map(&self, analysis: &AnalysisResult) -> MigrateResult<MappingResult> {
        let mut issues = Vec::new();
        let mut component_mappings = Vec::new();
        let mut total_confidence = 0.0;

        // Get mapping rules for detected framework
        let rules = self
            .mapping_rules
            .get(&analysis.framework)
            .or_else(|| self.mapping_rules.get(&Framework::Custom))
            .ok_or_else(|| MigrateError::ComponentMapping("No mapping rules found".into()))?;

        // Map each detected component
        for component in &analysis.components {
            let mapping = self.map_component(component, rules, &analysis.framework);
            total_confidence += mapping.confidence;
            component_mappings.push(mapping);
        }

        // Detect layout pattern
        let layout = self.detect_layout(analysis, &component_mappings, &mut issues);

        // Generate design pack parts
        let design_parts = self.generate_design_parts(&layout, &component_mappings, &mut issues);

        // Calculate overall confidence
        let confidence = if component_mappings.is_empty() {
            0.5
        } else {
            total_confidence / component_mappings.len() as f32
        };

        // Add issues for unmapped components
        for component in &analysis.components {
            if !component.mappable {
                issues.push(
                    MigrationIssue::warning(
                        IssueCategory::ComponentMapping,
                        format!("Component {:?} has no OxideKit equivalent", component.component_type),
                    )
                    .with_suggestion("Consider using compat.webview or implementing a custom component"),
                );
            }
        }

        Ok(MappingResult {
            layout,
            components: component_mappings,
            design_parts,
            issues,
            confidence,
        })
    }

    /// Map a single component
    fn map_component(
        &self,
        component: &DetectedComponent,
        rules: &[MappingRule],
        _framework: &Framework,
    ) -> ComponentMapping {
        // Find matching rule
        let rule = rules.iter().find(|r| r.source == component.component_type);

        if let Some(rule) = rule {
            let mut prop_mappings = HashMap::new();
            let mut notes = Vec::new();

            // Map properties based on detected classes
            for class in &component.classes {
                for (pattern, oxide_prop) in &rule.props {
                    // Simple pattern matching (could be enhanced with proper glob)
                    let pattern_base = pattern.trim_end_matches('*');
                    if class.starts_with(pattern_base) || class == *pattern {
                        prop_mappings.insert(class.clone(), oxide_prop.to_string());
                    }
                }
            }

            // Add variant and size if detected
            if let Some(ref variant) = component.variant {
                notes.push(format!("Detected variant: {}", variant));
            }
            if let Some(ref size) = component.size {
                notes.push(format!("Detected size: {}", size));
            }

            ComponentMapping {
                source_type: component.component_type.clone(),
                source_classes: component.classes.clone(),
                target_component: rule.target.to_string(),
                prop_mappings,
                confidence: rule.confidence * component.mapping_confidence,
                needs_review: component.mapping_confidence < 0.7,
                notes,
            }
        } else {
            // No rule found
            ComponentMapping {
                source_type: component.component_type.clone(),
                source_classes: component.classes.clone(),
                target_component: "compat.Custom".into(),
                prop_mappings: HashMap::new(),
                confidence: 0.3,
                needs_review: true,
                notes: vec!["No direct OxideKit equivalent found".into()],
            }
        }
    }

    /// Detect layout pattern from components
    fn detect_layout(
        &self,
        analysis: &AnalysisResult,
        mappings: &[ComponentMapping],
        issues: &mut Vec<MigrationIssue>,
    ) -> LayoutMapping {
        let mut layout = LayoutMapping::default();

        let has_sidebar = analysis
            .components
            .iter()
            .any(|c| c.component_type == ComponentType::Sidebar);
        let has_navbar = analysis
            .components
            .iter()
            .any(|c| c.component_type == ComponentType::Navbar);

        // Determine layout pattern
        layout.pattern = match (has_sidebar, has_navbar) {
            (true, true) => LayoutPattern::FullDashboard,
            (true, false) => LayoutPattern::SidebarLayout,
            (false, true) => LayoutPattern::NavbarLayout,
            (false, false) => {
                // Check for grid usage
                let has_grid = analysis.components.iter().any(|c| {
                    matches!(
                        c.component_type,
                        ComponentType::Grid | ComponentType::Row | ComponentType::Column
                    )
                });
                if has_grid {
                    LayoutPattern::GridLayout
                } else {
                    LayoutPattern::SingleColumn
                }
            }
        };

        // Configure sidebar
        if has_sidebar {
            layout.sidebar = Some(SidebarConfig {
                position: SidebarPosition::Left,
                width: Some(256.0),
                collapsible: true,
                has_nested_menus: false,
                has_icons: true,
                oxide_component: "ui.Sidenav".into(),
                menu_items: vec![],
            });
        }

        // Configure navbar
        if has_navbar {
            let navbar_mapping = mappings
                .iter()
                .find(|m| m.source_type == ComponentType::Navbar);

            let is_fixed = navbar_mapping
                .map(|m| m.source_classes.iter().any(|c| c.contains("fixed")))
                .unwrap_or(false);

            layout.navbar = Some(NavbarConfig {
                is_fixed,
                is_transparent: false,
                has_brand: true,
                brand_text: None,
                has_search: false,
                has_user_menu: true,
                has_notifications: false,
                oxide_component: "ui.Toolbar".into(),
            });
        }

        // Configure content area
        layout.content = ContentConfig {
            max_width: Some(1280.0),
            uses_container: analysis
                .components
                .iter()
                .any(|c| c.classes.iter().any(|class| class.contains("container"))),
            has_breadcrumbs: analysis
                .components
                .iter()
                .any(|c| c.component_type == ComponentType::Breadcrumb),
            has_page_header: false,
        };

        // Configure grid system
        layout.grid_system = match analysis.framework {
            Framework::Bootstrap => GridSystem {
                columns: 12,
                gutter: Some(24.0),
                breakpoints: vec![
                    Breakpoint { name: "sm".into(), min_width: 576.0 },
                    Breakpoint { name: "md".into(), min_width: 768.0 },
                    Breakpoint { name: "lg".into(), min_width: 992.0 },
                    Breakpoint { name: "xl".into(), min_width: 1200.0 },
                    Breakpoint { name: "xxl".into(), min_width: 1400.0 },
                ],
                oxide_equivalent: "ui.Grid with 12 columns".into(),
            },
            Framework::Tailwind => GridSystem {
                columns: 12,
                gutter: Some(16.0),
                breakpoints: vec![
                    Breakpoint { name: "sm".into(), min_width: 640.0 },
                    Breakpoint { name: "md".into(), min_width: 768.0 },
                    Breakpoint { name: "lg".into(), min_width: 1024.0 },
                    Breakpoint { name: "xl".into(), min_width: 1280.0 },
                    Breakpoint { name: "2xl".into(), min_width: 1536.0 },
                ],
                oxide_equivalent: "ui.Grid with responsive columns".into(),
            },
            _ => GridSystem {
                columns: 12,
                gutter: Some(16.0),
                breakpoints: vec![],
                oxide_equivalent: "ui.Grid".into(),
            },
        };

        issues.push(MigrationIssue::info(
            IssueCategory::Layout,
            format!("Detected layout pattern: {}", layout.pattern),
        ));

        layout
    }

    /// Generate design pack parts from mappings
    fn generate_design_parts(
        &self,
        layout: &LayoutMapping,
        mappings: &[ComponentMapping],
        _issues: &mut Vec<MigrationIssue>,
    ) -> Vec<DesignPart> {
        let mut parts = Vec::new();

        // Sidebar part
        if let Some(ref sidebar) = layout.sidebar {
            let config = format!(
                r#"[part.sidebar]
component = "{}"
position = "{:?}"
width = {}
collapsible = {}
"#,
                sidebar.oxide_component,
                sidebar.position,
                sidebar.width.unwrap_or(256.0),
                sidebar.collapsible
            );

            parts.push(DesignPart {
                tag: "part:sidebar".into(),
                name: "Sidebar Navigation".into(),
                description: "Main sidebar navigation component".into(),
                components: vec!["ui.Sidenav".into(), "ui.Menu".into()],
                config_toml: config,
                confidence: 0.85,
            });
        }

        // Navbar part
        if let Some(ref navbar) = layout.navbar {
            let config = format!(
                r#"[part.navbar]
component = "{}"
fixed = {}
has_brand = {}
has_user_menu = {}
"#,
                navbar.oxide_component, navbar.is_fixed, navbar.has_brand, navbar.has_user_menu
            );

            parts.push(DesignPart {
                tag: "part:navbar".into(),
                name: "Top Navigation Bar".into(),
                description: "Main navigation toolbar".into(),
                components: vec!["ui.Toolbar".into()],
                config_toml: config,
                confidence: 0.85,
            });
        }

        // Data table part (if tables detected)
        let has_tables = mappings
            .iter()
            .any(|m| matches!(m.source_type, ComponentType::Table | ComponentType::DataTable));

        if has_tables {
            let config = r#"[part.datatable]
component = "ui.DataTable"
pagination = true
sorting = true
filtering = true
"#;

            parts.push(DesignPart {
                tag: "part:datatable".into(),
                name: "Data Table".into(),
                description: "Sortable, filterable data table component".into(),
                components: vec!["ui.DataTable".into()],
                config_toml: config.into(),
                confidence: 0.9,
            });
        }

        // Statistics/dashboard cards (if stat components detected)
        let has_stats = mappings.iter().any(|m| {
            matches!(
                m.source_type,
                ComponentType::Statistics | ComponentType::Card
            )
        });

        if has_stats {
            let config = r#"[part.stat_cards]
component = "ui.Card"
variant = "stat"
# Used for dashboard metric display
"#;

            parts.push(DesignPart {
                tag: "part:stat_cards".into(),
                name: "Statistics Cards".into(),
                description: "Dashboard statistic display cards".into(),
                components: vec!["ui.Card".into(), "ui.Stat".into()],
                config_toml: config.into(),
                confidence: 0.8,
            });
        }

        // Form components (if forms detected)
        let has_forms = mappings
            .iter()
            .any(|m| matches!(m.source_type, ComponentType::Form | ComponentType::Input));

        if has_forms {
            let config = r#"[part.login_form]
component = "ui.Form"
# Example form layout for authentication
fields = ["username", "password"]
submit_label = "Sign In"
"#;

            parts.push(DesignPart {
                tag: "part:login_form".into(),
                name: "Login Form".into(),
                description: "Authentication form template".into(),
                components: vec!["ui.Form".into(), "ui.Input".into(), "ui.Button".into()],
                config_toml: config.into(),
                confidence: 0.75,
            });
        }

        parts
    }

    /// Export mapping result to TOML
    pub fn to_toml(&self, result: &MappingResult) -> MigrateResult<String> {
        Ok(toml::to_string_pretty(result)?)
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
    fn test_layout_pattern_display() {
        assert_eq!(LayoutPattern::FullDashboard.to_string(), "Full Dashboard");
        assert_eq!(LayoutPattern::SidebarLayout.to_string(), "Sidebar Layout");
    }

    #[test]
    fn test_map_bootstrap_button() {
        let mapper = ComponentMapper::new();
        let rules = mapper.mapping_rules.get(&Framework::Bootstrap).unwrap();

        let component = DetectedComponent {
            component_type: ComponentType::Button,
            classes: vec!["btn".into(), "btn-primary".into(), "btn-lg".into()],
            source_file: Some("index.html".into()),
            occurrences: 5,
            variant: Some("primary".into()),
            size: Some("lg".into()),
            mappable: true,
            mapping_confidence: 0.9,
        };

        let mapping = mapper.map_component(&component, rules, &Framework::Bootstrap);

        assert_eq!(mapping.target_component, "ui.Button");
        assert!(mapping.confidence > 0.8);
        assert!(mapping.prop_mappings.contains_key("btn-primary"));
    }

    #[test]
    fn test_layout_detection() {
        let mapper = ComponentMapper::new();
        let mut analysis = AnalysisResult::default();
        analysis.framework = Framework::Bootstrap;
        analysis.components = vec![
            DetectedComponent::new(ComponentType::Sidebar).with_classes(vec!["sidebar".into()]),
            DetectedComponent::new(ComponentType::Navbar).with_classes(vec!["navbar".into()]),
        ];

        let result = mapper.map(&analysis).unwrap();

        assert_eq!(result.layout.pattern, LayoutPattern::FullDashboard);
        assert!(result.layout.sidebar.is_some());
        assert!(result.layout.navbar.is_some());
    }

    #[test]
    fn test_design_parts_generation() {
        let mapper = ComponentMapper::new();
        let mut analysis = AnalysisResult::default();
        analysis.framework = Framework::Bootstrap;
        analysis.components = vec![
            DetectedComponent::new(ComponentType::Sidebar),
            DetectedComponent::new(ComponentType::Table),
        ];

        let result = mapper.map(&analysis).unwrap();

        let part_tags: Vec<_> = result.design_parts.iter().map(|p| &p.tag).collect();
        assert!(part_tags.contains(&&"part:sidebar".to_string()));
        assert!(part_tags.contains(&&"part:datatable".to_string()));
    }

    #[test]
    fn test_grid_system_bootstrap() {
        let mapper = ComponentMapper::new();
        let mut analysis = AnalysisResult::default();
        analysis.framework = Framework::Bootstrap;

        let result = mapper.map(&analysis).unwrap();

        assert_eq!(result.layout.grid_system.columns, 12);
        assert!(result.layout.grid_system.gutter.is_some());
        assert!(!result.layout.grid_system.breakpoints.is_empty());
    }
}
