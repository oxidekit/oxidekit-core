//! Design Pack Template Extractor
//!
//! Makes themes and design packs AI-extractable and composable.
//! Parts are tagged with stable IDs and intent labels for machine-readable extraction.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// A design pack containing templates, themes, and extractable parts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPack {
    /// Unique pack ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Version
    pub version: String,

    /// Author
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,

    /// License
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub license: Option<String>,

    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Preview image URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,

    /// Design tokens provided by this pack
    #[serde(default)]
    pub tokens: PackTokens,

    /// Template parts available for extraction
    #[serde(default)]
    pub parts: Vec<TemplatePart>,

    /// Component variants provided
    #[serde(default)]
    pub component_variants: Vec<ComponentVariantDef>,

    /// Layout templates included
    #[serde(default)]
    pub layouts: Vec<LayoutTemplate>,

    /// Composition graph
    #[serde(default)]
    pub composition: CompositionGraph,

    /// Required extensions
    #[serde(default)]
    pub required_extensions: Vec<String>,

    /// Example pages/screens
    #[serde(default)]
    pub examples: Vec<PackExample>,
}

/// Design tokens provided by a pack
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PackTokens {
    /// Color palette
    #[serde(default)]
    pub colors: HashMap<String, String>,

    /// Spacing scale
    #[serde(default)]
    pub spacing: HashMap<String, String>,

    /// Typography settings
    #[serde(default)]
    pub typography: HashMap<String, TypographyToken>,

    /// Border radius values
    #[serde(default)]
    pub radius: HashMap<String, String>,

    /// Shadow definitions
    #[serde(default)]
    pub shadows: HashMap<String, String>,
}

/// Typography token definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyToken {
    pub font_family: String,
    pub font_size: String,
    pub font_weight: String,
    pub line_height: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub letter_spacing: Option<String>,
}

/// A template part that can be extracted
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePart {
    /// Part ID (e.g., "navbar", "sidebar", "login_form")
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Part category
    pub category: PartCategory,

    /// Intent labels for discovery
    #[serde(default)]
    pub intents: Vec<String>,

    /// UI code for this part
    pub code: String,

    /// Required tokens
    #[serde(default)]
    pub required_tokens: Vec<String>,

    /// Required components
    #[serde(default)]
    pub required_components: Vec<String>,

    /// Required extensions
    #[serde(default)]
    pub required_extensions: Vec<String>,

    /// Dependencies on other parts
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Compatibility constraints
    #[serde(default)]
    pub compatibility: PartCompatibility,

    /// Available variations
    #[serde(default)]
    pub variations: Vec<PartVariation>,

    /// Preview metadata
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview: Option<PartPreview>,
}

/// Categories for template parts
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PartCategory {
    /// Navigation elements
    Navigation,
    /// Layout/structure
    Layout,
    /// Forms and inputs
    Forms,
    /// Data display
    Data,
    /// Authentication
    Auth,
    /// Dashboard widgets
    Dashboard,
    /// Marketing/landing
    Marketing,
    /// Settings/preferences
    Settings,
    /// Error/empty states
    States,
    /// Other/misc
    Other,
}

impl std::fmt::Display for PartCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Navigation => write!(f, "navigation"),
            Self::Layout => write!(f, "layout"),
            Self::Forms => write!(f, "forms"),
            Self::Data => write!(f, "data"),
            Self::Auth => write!(f, "auth"),
            Self::Dashboard => write!(f, "dashboard"),
            Self::Marketing => write!(f, "marketing"),
            Self::Settings => write!(f, "settings"),
            Self::States => write!(f, "states"),
            Self::Other => write!(f, "other"),
        }
    }
}

/// Compatibility constraints for a part
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PartCompatibility {
    /// Minimum core version
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_core_version: Option<String>,

    /// Maximum core version
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_core_version: Option<String>,

    /// Themes this part works with
    #[serde(default)]
    pub themes: Vec<String>,

    /// Breakpoints this part is designed for
    #[serde(default)]
    pub breakpoints: Vec<String>,
}

/// Variation of a template part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartVariation {
    /// Variation ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Modified code for this variation
    pub code: String,

    /// Additional tokens needed
    #[serde(default)]
    pub additional_tokens: Vec<String>,
}

/// Preview metadata for a part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartPreview {
    /// Screenshot URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_url: Option<String>,

    /// Light mode screenshot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_light: Option<String>,

    /// Dark mode screenshot
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_dark: Option<String>,

    /// Width in pixels
    #[serde(default)]
    pub width: u32,

    /// Height in pixels
    #[serde(default)]
    pub height: u32,
}

/// Component variant definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVariantDef {
    /// Base component ID
    pub component: String,

    /// Variant name
    pub variant: String,

    /// Description
    pub description: String,

    /// Style overrides
    #[serde(default)]
    pub styles: HashMap<String, String>,
}

/// Layout template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutTemplate {
    /// Layout ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Layout code
    pub code: String,

    /// Regions/slots in this layout
    #[serde(default)]
    pub regions: Vec<LayoutRegion>,
}

/// Region in a layout template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutRegion {
    /// Region ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Suggested parts for this region
    #[serde(default)]
    pub suggested_parts: Vec<String>,
}

/// Composition graph showing part dependencies
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompositionGraph {
    /// Nodes (parts)
    #[serde(default)]
    pub nodes: Vec<CompositionNode>,

    /// Edges (dependencies)
    #[serde(default)]
    pub edges: Vec<CompositionEdge>,
}

impl CompositionGraph {
    /// Get all dependencies for a part
    pub fn get_dependencies(&self, part_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.from == part_id)
            .map(|e| e.to.as_str())
            .collect()
    }

    /// Get all dependents of a part
    pub fn get_dependents(&self, part_id: &str) -> Vec<&str> {
        self.edges
            .iter()
            .filter(|e| e.to == part_id)
            .map(|e| e.from.as_str())
            .collect()
    }

    /// Get transitive closure of dependencies
    pub fn get_all_dependencies(&self, part_id: &str) -> HashSet<String> {
        let mut visited = HashSet::new();
        let mut queue = vec![part_id.to_string()];

        while let Some(current) = queue.pop() {
            if visited.contains(&current) {
                continue;
            }
            visited.insert(current.clone());

            for dep in self.get_dependencies(&current) {
                if !visited.contains(dep) {
                    queue.push(dep.to_string());
                }
            }
        }

        visited.remove(part_id);
        visited
    }
}

/// Node in composition graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionNode {
    /// Part ID
    pub id: String,

    /// Node type
    pub node_type: NodeType,
}

/// Type of composition node
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum NodeType {
    Part,
    Component,
    Token,
    Extension,
}

/// Edge in composition graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompositionEdge {
    /// Source part
    pub from: String,

    /// Target (dependency)
    pub to: String,

    /// Type of dependency
    pub edge_type: EdgeType,
}

/// Type of composition edge
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EdgeType {
    RequiresPart,
    RequiresComponent,
    RequiresToken,
    RequiresExtension,
}

/// Example page/screen in a design pack
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackExample {
    /// Example ID
    pub id: String,

    /// Display name
    pub name: String,

    /// Description
    pub description: String,

    /// Screenshot URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub screenshot_url: Option<String>,

    /// Parts used in this example
    #[serde(default)]
    pub parts_used: Vec<String>,

    /// Route/path for this example
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub route: Option<String>,
}

/// Result of extracting a part
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PartExtractionResult {
    /// The extracted part
    pub part: TemplatePart,

    /// UI fragments (may be multiple files)
    pub fragments: Vec<UiFragment>,

    /// Required tokens (with values from the pack)
    pub tokens: HashMap<String, String>,

    /// Required components (with specs)
    pub components: Vec<String>,

    /// Required extensions
    pub extensions: Vec<String>,

    /// Compatibility constraints
    pub compatibility: PartCompatibility,

    /// Recommended variations
    pub variations: Vec<PartVariation>,

    /// Warnings or notes
    #[serde(default)]
    pub warnings: Vec<String>,
}

/// A UI fragment from extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiFragment {
    /// Suggested file path
    pub path: String,

    /// Fragment content
    pub content: String,

    /// Description
    pub description: String,
}

/// Template part extractor
pub struct PartExtractor {
    pack: DesignPack,
}

impl PartExtractor {
    /// Create a new extractor for a design pack
    pub fn new(pack: DesignPack) -> Self {
        Self { pack }
    }

    /// List all available parts
    pub fn list_parts(&self) -> Vec<&TemplatePart> {
        self.pack.parts.iter().collect()
    }

    /// Search parts by query
    pub fn search_parts(&self, query: &str) -> Vec<&TemplatePart> {
        let query_lower = query.to_lowercase();

        self.pack
            .parts
            .iter()
            .filter(|p| {
                p.id.to_lowercase().contains(&query_lower)
                    || p.name.to_lowercase().contains(&query_lower)
                    || p.description.to_lowercase().contains(&query_lower)
                    || p.intents.iter().any(|i| i.to_lowercase().contains(&query_lower))
            })
            .collect()
    }

    /// Get parts by category
    pub fn get_parts_by_category(&self, category: PartCategory) -> Vec<&TemplatePart> {
        self.pack
            .parts
            .iter()
            .filter(|p| p.category == category)
            .collect()
    }

    /// Get a specific part
    pub fn get_part(&self, id: &str) -> Option<&TemplatePart> {
        self.pack.parts.iter().find(|p| p.id == id)
    }

    /// Extract a part with all dependencies
    pub fn extract_part(&self, id: &str, variation: Option<&str>) -> Option<PartExtractionResult> {
        let part = self.get_part(id)?;

        // Determine which code to use
        let code = if let Some(var_id) = variation {
            part.variations
                .iter()
                .find(|v| v.id == var_id)
                .map(|v| v.code.clone())
                .unwrap_or_else(|| part.code.clone())
        } else {
            part.code.clone()
        };

        // Build token map
        let mut tokens = HashMap::new();
        for token_name in &part.required_tokens {
            // Look up token value in pack
            if let Some(value) = self.resolve_token(token_name) {
                tokens.insert(token_name.clone(), value);
            }
        }

        // Get all dependencies
        let all_deps = self.pack.composition.get_all_dependencies(id);
        let mut components: Vec<String> = part.required_components.clone();
        let mut extensions: Vec<String> = part.required_extensions.clone();

        // Add dependencies' requirements
        for dep_id in &all_deps {
            if let Some(dep_part) = self.get_part(dep_id) {
                components.extend(dep_part.required_components.iter().cloned());
                extensions.extend(dep_part.required_extensions.iter().cloned());
            }
        }

        // Deduplicate
        components.sort();
        components.dedup();
        extensions.sort();
        extensions.dedup();

        let mut warnings = Vec::new();

        // Check for missing dependencies
        for dep in &all_deps {
            if self.get_part(dep).is_none() {
                warnings.push(format!("Dependency '{}' not found in pack", dep));
            }
        }

        Some(PartExtractionResult {
            part: part.clone(),
            fragments: vec![UiFragment {
                path: format!("ui/parts/{}.oui", id.replace('.', "/")),
                content: code,
                description: part.description.clone(),
            }],
            tokens,
            components,
            extensions,
            compatibility: part.compatibility.clone(),
            variations: part.variations.clone(),
            warnings,
        })
    }

    /// Resolve a token name to its value
    fn resolve_token(&self, name: &str) -> Option<String> {
        let parts: Vec<&str> = name.split('.').collect();
        if parts.len() < 2 {
            return None;
        }

        let category = parts[0];
        let token_name = parts[1..].join(".");

        match category {
            "color" => self.pack.tokens.colors.get(&token_name).cloned(),
            "spacing" => self.pack.tokens.spacing.get(&token_name).cloned(),
            "radius" => self.pack.tokens.radius.get(&token_name).cloned(),
            "shadow" => self.pack.tokens.shadows.get(&token_name).cloned(),
            _ => None,
        }
    }
}

/// Create an example admin design pack
pub fn example_admin_pack() -> DesignPack {
    DesignPack {
        id: "oxide.admin".to_string(),
        name: "OxideKit Admin".to_string(),
        description: "A comprehensive admin panel design pack with all essential components".to_string(),
        version: "0.1.0".to_string(),
        author: Some("OxideKit Team".to_string()),
        license: Some("MIT".to_string()),
        tags: vec!["admin".into(), "dashboard".into(), "enterprise".into()],
        preview_url: Some("https://oxidekit.com/packs/admin/preview.png".into()),
        tokens: PackTokens {
            colors: [
                ("primary".into(), "#3B82F6".into()),
                ("primary.hover".into(), "#2563EB".into()),
                ("secondary".into(), "#6366F1".into()),
                ("success".into(), "#22C55E".into()),
                ("warning".into(), "#F59E0B".into()),
                ("danger".into(), "#EF4444".into()),
                ("background".into(), "#09090B".into()),
                ("surface".into(), "#18181B".into()),
                ("surface.elevated".into(), "#27272A".into()),
                ("text.primary".into(), "#FFFFFF".into()),
                ("text.secondary".into(), "#A1A1AA".into()),
                ("border".into(), "#3F3F46".into()),
            ]
            .into_iter()
            .collect(),
            spacing: [
                ("xs".into(), "4px".into()),
                ("sm".into(), "8px".into()),
                ("md".into(), "16px".into()),
                ("lg".into(), "24px".into()),
                ("xl".into(), "32px".into()),
            ]
            .into_iter()
            .collect(),
            ..Default::default()
        },
        parts: vec![
            TemplatePart {
                id: "navbar".into(),
                name: "Navigation Bar".into(),
                description: "Top navigation bar with logo, menu, and user avatar".into(),
                category: PartCategory::Navigation,
                intents: vec!["navigation".into(), "header".into(), "top-bar".into()],
                code: r#"
HStack {
    height: 64
    padding_x: spacing.lg
    background: token.color.surface
    border_bottom: token.color.border
    justify: "space-between"
    align: "center"

    // Logo
    HStack {
        spacing: spacing.sm

        Image {
            src: "/logo.svg"
            width: 32
            height: 32
        }

        Text {
            role: "title"
            content: app_name
        }
    }

    // Navigation links
    HStack {
        spacing: spacing.lg

        for link in nav_links {
            NavLink {
                href: link.href
                label: link.label
                active: current_path == link.href
            }
        }
    }

    // User menu
    HStack {
        spacing: spacing.sm

        IconButton {
            icon: "bell"
            aria_label: "Notifications"
        }

        Avatar {
            src: user.avatar
            fallback: user.initials
            size: "sm"
        }
    }
}
"#.into(),
                required_tokens: vec![
                    "color.surface".into(),
                    "color.border".into(),
                    "spacing.lg".into(),
                    "spacing.sm".into(),
                ],
                required_components: vec![
                    "ui.Text".into(),
                    "ui.IconButton".into(),
                    "ui.Avatar".into(),
                ],
                required_extensions: vec![],
                depends_on: vec![],
                compatibility: PartCompatibility::default(),
                variations: vec![
                    PartVariation {
                        id: "transparent".into(),
                        name: "Transparent".into(),
                        description: "Transparent background, suitable for hero sections".into(),
                        code: "/* same code with background: transparent */".into(),
                        additional_tokens: vec![],
                    },
                ],
                preview: Some(PartPreview {
                    screenshot_url: Some("https://oxidekit.com/packs/admin/navbar.png".into()),
                    screenshot_light: None,
                    screenshot_dark: None,
                    width: 1200,
                    height: 64,
                }),
            },
            TemplatePart {
                id: "sidebar".into(),
                name: "Sidebar".into(),
                description: "Collapsible sidebar with navigation menu".into(),
                category: PartCategory::Navigation,
                intents: vec!["sidebar".into(), "navigation".into(), "menu".into()],
                code: r#"
VStack {
    width: sidebar_collapsed ? 64 : 240
    background: token.color.surface
    border_right: token.color.border
    transition: "width 200ms"

    // Menu items
    VStack {
        padding: spacing.sm
        spacing: spacing.xs
        flex: 1

        for item in menu_items {
            SidebarItem {
                icon: item.icon
                label: item.label
                collapsed: sidebar_collapsed
                active: current_path == item.href
                on_click: || navigate(item.href)
            }
        }
    }

    // Collapse toggle
    HStack {
        padding: spacing.sm
        justify: "center"

        IconButton {
            icon: sidebar_collapsed ? "chevron-right" : "chevron-left"
            on_click: toggle_sidebar
        }
    }
}
"#.into(),
                required_tokens: vec![
                    "color.surface".into(),
                    "color.border".into(),
                    "spacing.sm".into(),
                    "spacing.xs".into(),
                ],
                required_components: vec!["ui.IconButton".into()],
                required_extensions: vec![],
                depends_on: vec![],
                compatibility: PartCompatibility::default(),
                variations: vec![],
                preview: None,
            },
            TemplatePart {
                id: "login_form".into(),
                name: "Login Form".into(),
                description: "Complete login form with email, password, and remember me".into(),
                category: PartCategory::Auth,
                intents: vec!["login".into(), "auth".into(), "sign-in".into()],
                code: r#"
Card {
    variant: "elevated"
    max_width: 400
    padding: spacing.xl

    VStack {
        spacing: spacing.lg
        align: "center"

        Text {
            role: "heading"
            content: "Sign In"
        }

        TextField {
            label: "Email"
            type: "email"
            bind: email
            required: true
        }

        TextField {
            label: "Password"
            type: "password"
            bind: password
            required: true
        }

        Checkbox {
            label: "Remember me"
            bind: remember_me
        }

        Button {
            variant: "primary"
            label: "Sign In"
            full_width: true
            on_click: handle_login
        }
    }
}
"#.into(),
                required_tokens: vec!["spacing.lg".into(), "spacing.xl".into()],
                required_components: vec![
                    "ui.Card".into(),
                    "ui.Text".into(),
                    "ui.Button".into(),
                ],
                required_extensions: vec!["oxide.forms".into()],
                depends_on: vec![],
                compatibility: PartCompatibility::default(),
                variations: vec![],
                preview: None,
            },
            TemplatePart {
                id: "datatable".into(),
                name: "Data Table".into(),
                description: "Full-featured data table with sorting, filtering, pagination".into(),
                category: PartCategory::Data,
                intents: vec!["table".into(), "data".into(), "list".into(), "grid".into()],
                code: r#"
VStack {
    spacing: spacing.md

    // Toolbar
    HStack {
        justify: "space-between"

        TextField {
            placeholder: "Search..."
            icon: "search"
            bind: search_query
        }

        Button {
            variant: "primary"
            label: "Add New"
            icon: "plus"
            on_click: open_create
        }
    }

    // Table
    DataTable {
        data: filtered_data
        loading: is_loading
        sortable: true
        selectable: true

        for column in columns {
            TableColumn {
                key: column.key
                header: column.header
                sortable: column.sortable
            }
        }
    }

    // Pagination
    Pagination {
        current: current_page
        total: total_pages
        on_change: set_page
    }
}
"#.into(),
                required_tokens: vec!["spacing.md".into()],
                required_components: vec!["ui.Button".into()],
                required_extensions: vec!["oxide.tables".into()],
                depends_on: vec![],
                compatibility: PartCompatibility::default(),
                variations: vec![],
                preview: None,
            },
            TemplatePart {
                id: "stat_cards".into(),
                name: "Stat Cards".into(),
                description: "Grid of stat cards showing key metrics".into(),
                category: PartCategory::Dashboard,
                intents: vec!["stats".into(), "metrics".into(), "kpi".into(), "cards".into()],
                code: r#"
Grid {
    columns: 4
    gap: spacing.md

    for stat in stats {
        Card {
            padding: spacing.lg

            VStack {
                spacing: spacing.sm

                HStack {
                    justify: "space-between"

                    Text {
                        role: "caption"
                        content: stat.label
                    }

                    Badge {
                        variant: stat.trend > 0 ? "success" : "danger"
                        label: format!("{}%", stat.trend)
                    }
                }

                Text {
                    role: "title"
                    content: stat.value
                }
            }
        }
    }
}
"#.into(),
                required_tokens: vec!["spacing.md".into(), "spacing.lg".into(), "spacing.sm".into()],
                required_components: vec!["ui.Card".into(), "ui.Text".into(), "ui.Badge".into()],
                required_extensions: vec![],
                depends_on: vec![],
                compatibility: PartCompatibility::default(),
                variations: vec![],
                preview: None,
            },
        ],
        component_variants: vec![],
        layouts: vec![
            LayoutTemplate {
                id: "admin".into(),
                name: "Admin Layout".into(),
                description: "Standard admin layout with sidebar and header".into(),
                code: r#"
Layout {
    HStack {
        spacing: 0

        // Sidebar
        region: "sidebar"

        // Main content
        VStack {
            flex: 1

            // Header
            region: "header"

            // Content
            VStack {
                flex: 1
                padding: spacing.lg
                overflow: "auto"

                region: "content"
            }
        }
    }
}
"#.into(),
                regions: vec![
                    LayoutRegion {
                        id: "sidebar".into(),
                        name: "Sidebar".into(),
                        description: "Left sidebar for navigation".into(),
                        suggested_parts: vec!["sidebar".into()],
                    },
                    LayoutRegion {
                        id: "header".into(),
                        name: "Header".into(),
                        description: "Top header bar".into(),
                        suggested_parts: vec!["navbar".into()],
                    },
                    LayoutRegion {
                        id: "content".into(),
                        name: "Content".into(),
                        description: "Main content area".into(),
                        suggested_parts: vec![],
                    },
                ],
            },
        ],
        composition: CompositionGraph::default(),
        required_extensions: vec!["oxide.forms".into(), "oxide.tables".into()],
        examples: vec![
            PackExample {
                id: "dashboard".into(),
                name: "Dashboard".into(),
                description: "Example dashboard with stats and charts".into(),
                screenshot_url: Some("https://oxidekit.com/packs/admin/dashboard.png".into()),
                parts_used: vec!["navbar".into(), "sidebar".into(), "stat_cards".into()],
                route: Some("/dashboard".into()),
            },
            PackExample {
                id: "users".into(),
                name: "Users List".into(),
                description: "User management table".into(),
                screenshot_url: Some("https://oxidekit.com/packs/admin/users.png".into()),
                parts_used: vec!["navbar".into(), "sidebar".into(), "datatable".into()],
                route: Some("/users".into()),
            },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_pack() {
        let pack = example_admin_pack();
        assert_eq!(pack.id, "oxide.admin");
        assert!(!pack.parts.is_empty());
    }

    #[test]
    fn test_part_extractor() {
        let pack = example_admin_pack();
        let extractor = PartExtractor::new(pack);

        let parts = extractor.list_parts();
        assert!(!parts.is_empty());

        let navbar = extractor.get_part("navbar");
        assert!(navbar.is_some());
    }

    #[test]
    fn test_search_parts() {
        let pack = example_admin_pack();
        let extractor = PartExtractor::new(pack);

        let results = extractor.search_parts("login");
        assert!(!results.is_empty());
        assert!(results.iter().any(|p| p.id == "login_form"));
    }

    #[test]
    fn test_extract_part() {
        let pack = example_admin_pack();
        let extractor = PartExtractor::new(pack);

        let result = extractor.extract_part("login_form", None);
        assert!(result.is_some());

        let result = result.unwrap();
        assert!(!result.fragments.is_empty());
        assert!(result.extensions.contains(&"oxide.forms".to_string()));
    }

    #[test]
    fn test_composition_graph() {
        let mut graph = CompositionGraph::default();
        graph.edges.push(CompositionEdge {
            from: "dashboard".into(),
            to: "stat_cards".into(),
            edge_type: EdgeType::RequiresPart,
        });

        let deps = graph.get_dependencies("dashboard");
        assert!(deps.contains(&"stat_cards"));
    }
}
