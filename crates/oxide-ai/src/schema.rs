//! AI Schema Types
//!
//! Defines the `oxide.ai.json` format - the machine-readable catalog for AI tools.

use oxide_components::{ComponentSpec, Theme, FontRegistry, TypographyRole};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::extensions::ExtensionSpec;
use crate::recipes::Recipe;
use crate::design_pack::DesignPack;

/// Schema version for tracking compatibility
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaVersion {
    /// Major version (breaking changes)
    pub major: u32,
    /// Minor version (new features)
    pub minor: u32,
}

impl SchemaVersion {
    pub fn new(major: u32, minor: u32) -> Self {
        Self { major, minor }
    }

    pub fn v1() -> Self {
        Self::new(1, 0)
    }

    pub fn is_compatible_with(&self, other: &SchemaVersion) -> bool {
        self.major == other.major && self.minor >= other.minor
    }
}

impl Default for SchemaVersion {
    fn default() -> Self {
        Self::v1()
    }
}

impl std::fmt::Display for SchemaVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

/// The complete AI-readable schema for OxideKit
///
/// This is the root structure exported as `oxide.ai.json`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSchema {
    /// Schema version for compatibility checking
    #[serde(rename = "oxide_ai_schema")]
    pub schema_version: String,

    /// OxideKit core version this schema was generated for
    pub core_version: String,

    /// Generation timestamp (ISO 8601)
    pub generated_at: String,

    /// Component specifications
    pub components: Vec<ComponentSpec>,

    /// Extension specifications
    #[serde(default)]
    pub extensions: Vec<ExtensionSpec>,

    /// Available design packs/themes
    #[serde(default)]
    pub design_packs: Vec<DesignPackSummary>,

    /// Recipe library
    #[serde(default)]
    pub recipes: Vec<Recipe>,

    /// Token catalog (available design tokens)
    #[serde(default)]
    pub tokens: TokenCatalog,

    /// Invalid usage patterns (anti-hallucination)
    #[serde(default)]
    pub invalid_patterns: Vec<InvalidPattern>,

    /// Compatibility constraints
    #[serde(default)]
    pub compatibility: CompatibilityInfo,
}

impl AiSchema {
    /// Create a new schema with default values
    pub fn new(core_version: &str) -> Self {
        Self {
            schema_version: crate::SCHEMA_VERSION.to_string(),
            core_version: core_version.to_string(),
            generated_at: chrono::Utc::now().to_rfc3339(),
            components: Vec::new(),
            extensions: Vec::new(),
            design_packs: Vec::new(),
            recipes: Vec::new(),
            tokens: TokenCatalog::default(),
            invalid_patterns: Vec::new(),
            compatibility: CompatibilityInfo::default(),
        }
    }

    /// Validate schema version compatibility
    pub fn is_compatible(&self) -> bool {
        self.schema_version == crate::SCHEMA_VERSION
    }
}

/// Summary of a design pack for the catalog
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPackSummary {
    /// Pack ID
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

    /// Tags for discovery
    #[serde(default)]
    pub tags: Vec<String>,

    /// Number of template parts available
    pub part_count: usize,

    /// Preview image URL
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub preview_url: Option<String>,
}

/// Catalog of available design tokens
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenCatalog {
    /// Color tokens
    #[serde(default)]
    pub colors: HashMap<String, TokenInfo>,

    /// Spacing tokens
    #[serde(default)]
    pub spacing: HashMap<String, TokenInfo>,

    /// Typography tokens
    #[serde(default)]
    pub typography: HashMap<String, TokenInfo>,

    /// Radius tokens
    #[serde(default)]
    pub radius: HashMap<String, TokenInfo>,

    /// Shadow tokens
    #[serde(default)]
    pub shadows: HashMap<String, TokenInfo>,

    /// Z-index tokens
    #[serde(default)]
    pub z_index: HashMap<String, TokenInfo>,
}

/// Information about a design token
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenInfo {
    /// Token description
    pub description: String,

    /// Default value (as string representation)
    pub default_value: String,

    /// Category within the token namespace
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,

    /// Which themes override this token
    #[serde(default)]
    pub theme_overrides: Vec<String>,
}

/// Pattern that AIs should avoid (anti-hallucination)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidPattern {
    /// Pattern identifier
    pub id: String,

    /// What the AI might incorrectly generate
    pub pattern: String,

    /// Why this is wrong
    pub reason: String,

    /// Component this applies to (if specific)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub component: Option<String>,

    /// The correct approach
    pub correct_approach: String,

    /// Example of correct usage
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correct_example: Option<String>,
}

/// Compatibility information for version pinning
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CompatibilityInfo {
    /// Minimum supported OxideKit core version
    pub min_core_version: String,

    /// Maximum supported core version (if known)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_core_version: Option<String>,

    /// Breaking changes from previous versions
    #[serde(default)]
    pub breaking_changes: Vec<BreakingChange>,

    /// Deprecation warnings
    #[serde(default)]
    pub deprecations: Vec<DeprecationWarning>,
}

/// A breaking change between versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakingChange {
    /// Version where change occurred
    pub since_version: String,

    /// What changed
    pub description: String,

    /// Migration guide
    pub migration: String,
}

/// A deprecation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeprecationWarning {
    /// What is deprecated
    pub subject: String,

    /// Type (component, prop, extension, etc.)
    pub subject_type: String,

    /// Version when deprecated
    pub since_version: String,

    /// When it will be removed (if known)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub removal_version: Option<String>,

    /// Replacement or alternative
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<String>,
}

/// AI Catalog - the main interface for generating and querying the schema
pub struct AiCatalog {
    /// The underlying schema
    schema: AiSchema,

    /// Component registry reference
    component_registry: oxide_components::ComponentRegistry,
}

impl AiCatalog {
    /// Create a new catalog with core components
    pub fn with_core() -> Self {
        let registry = oxide_components::ComponentRegistry::with_core_components();
        let mut schema = AiSchema::new(crate::VERSION);

        // Add core components from registry
        for component_id in registry.list_components() {
            if let Some(spec) = registry.get(&component_id) {
                schema.components.push((*spec).clone());
            }
        }

        // Add core tokens
        schema.tokens = Self::build_token_catalog();

        // Add invalid patterns (anti-hallucination)
        schema.invalid_patterns = Self::build_invalid_patterns();

        Self {
            schema,
            component_registry: registry,
        }
    }

    /// Create an empty catalog
    pub fn empty() -> Self {
        Self {
            schema: AiSchema::new(crate::VERSION),
            component_registry: oxide_components::ComponentRegistry::new(),
        }
    }

    /// Add an extension to the catalog
    pub fn add_extension(&mut self, extension: ExtensionSpec) {
        self.schema.extensions.push(extension);
    }

    /// Add a design pack to the catalog
    pub fn add_design_pack(&mut self, pack: &DesignPack) {
        self.schema.design_packs.push(DesignPackSummary {
            id: pack.id.clone(),
            name: pack.name.clone(),
            description: pack.description.clone(),
            version: pack.version.clone(),
            author: pack.author.clone(),
            tags: pack.tags.clone(),
            part_count: pack.parts.len(),
            preview_url: pack.preview_url.clone(),
        });
    }

    /// Add a recipe to the catalog
    pub fn add_recipe(&mut self, recipe: Recipe) {
        self.schema.recipes.push(recipe);
    }

    /// Get the schema
    pub fn schema(&self) -> &AiSchema {
        &self.schema
    }

    /// Export to JSON string
    pub fn export_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.schema)
    }

    /// Export to JSON with deterministic ordering (for stable diffs)
    pub fn export_json_stable(&self) -> Result<String, serde_json::Error> {
        // Sort components by ID for stable output
        let mut sorted_schema = self.schema.clone();
        sorted_schema.components.sort_by(|a, b| a.id.cmp(&b.id));
        sorted_schema.extensions.sort_by(|a, b| a.id.cmp(&b.id));
        sorted_schema.recipes.sort_by(|a, b| a.id.cmp(&b.id));
        sorted_schema.design_packs.sort_by(|a, b| a.id.cmp(&b.id));

        serde_json::to_string_pretty(&sorted_schema)
    }

    /// Search components by query
    pub fn search_components(&self, query: &str) -> Vec<&ComponentSpec> {
        let query_lower = query.to_lowercase();

        self.schema
            .components
            .iter()
            .filter(|c| {
                c.id.to_lowercase().contains(&query_lower)
                    || c.name.to_lowercase().contains(&query_lower)
                    || c.description.to_lowercase().contains(&query_lower)
            })
            .collect()
    }

    /// Get a component by ID
    pub fn get_component(&self, id: &str) -> Option<&ComponentSpec> {
        self.schema.components.iter().find(|c| c.id == id)
    }

    /// List all component IDs
    pub fn list_component_ids(&self) -> Vec<&str> {
        self.schema.components.iter().map(|c| c.id.as_str()).collect()
    }

    /// Search recipes by tags
    pub fn search_recipes(&self, tags: &[&str]) -> Vec<&Recipe> {
        self.schema
            .recipes
            .iter()
            .filter(|r| tags.iter().any(|t| r.tags.contains(&t.to_string())))
            .collect()
    }

    /// Get a recipe by ID
    pub fn get_recipe(&self, id: &str) -> Option<&Recipe> {
        self.schema.recipes.iter().find(|r| r.id == id)
    }

    /// Build the token catalog from theme defaults
    fn build_token_catalog() -> TokenCatalog {
        let mut catalog = TokenCatalog::default();

        // Color tokens
        let colors = [
            ("color.primary", "Primary brand color", "#3B82F6"),
            ("color.secondary", "Secondary brand color", "#6366F1"),
            ("color.success", "Success/positive color", "#22C55E"),
            ("color.warning", "Warning/caution color", "#F59E0B"),
            ("color.danger", "Danger/error color", "#EF4444"),
            ("color.text.primary", "Primary text color", "#FFFFFF"),
            ("color.text.secondary", "Secondary text color", "#A1A1AA"),
            ("color.text.muted", "Muted/disabled text", "#71717A"),
            ("color.background", "Page background", "#09090B"),
            ("color.surface", "Surface/card background", "#18181B"),
            ("color.border", "Border color", "#27272A"),
        ];

        for (name, desc, default) in colors {
            catalog.colors.insert(
                name.to_string(),
                TokenInfo {
                    description: desc.to_string(),
                    default_value: default.to_string(),
                    category: None,
                    theme_overrides: vec!["dark".into(), "light".into()],
                },
            );
        }

        // Spacing tokens
        let spacing = [
            ("spacing.xs", "Extra small spacing", "4px"),
            ("spacing.sm", "Small spacing", "8px"),
            ("spacing.md", "Medium spacing", "16px"),
            ("spacing.lg", "Large spacing", "24px"),
            ("spacing.xl", "Extra large spacing", "32px"),
            ("spacing.2xl", "2x extra large spacing", "48px"),
        ];

        for (name, desc, default) in spacing {
            catalog.spacing.insert(
                name.to_string(),
                TokenInfo {
                    description: desc.to_string(),
                    default_value: default.to_string(),
                    category: None,
                    theme_overrides: vec![],
                },
            );
        }

        // Radius tokens
        let radius = [
            ("radius.none", "No border radius", "0"),
            ("radius.sm", "Small border radius", "4px"),
            ("radius.md", "Medium border radius", "8px"),
            ("radius.lg", "Large border radius", "12px"),
            ("radius.full", "Full/circle radius", "9999px"),
        ];

        for (name, desc, default) in radius {
            catalog.radius.insert(
                name.to_string(),
                TokenInfo {
                    description: desc.to_string(),
                    default_value: default.to_string(),
                    category: None,
                    theme_overrides: vec![],
                },
            );
        }

        catalog
    }

    /// Build invalid patterns for anti-hallucination
    fn build_invalid_patterns() -> Vec<InvalidPattern> {
        vec![
            InvalidPattern {
                id: "button-glow-prop".into(),
                pattern: "Button { glow: true }".into(),
                reason: "The 'glow' prop does not exist on Button".into(),
                component: Some("ui.Button".into()),
                correct_approach: "Use variant='primary' for emphasis or apply custom styles".into(),
                correct_example: Some("Button { variant: \"primary\" label: \"Click\" }".into()),
            },
            InvalidPattern {
                id: "card-shadow-level".into(),
                pattern: "Card { shadow_level: 3 }".into(),
                reason: "Card uses 'variant' for elevation, not 'shadow_level'".into(),
                component: Some("ui.Card".into()),
                correct_approach: "Use variant='elevated' for shadow effect".into(),
                correct_example: Some("Card { variant: \"elevated\" }".into()),
            },
            InvalidPattern {
                id: "button-type-prop".into(),
                pattern: "Button { type: \"submit\" }".into(),
                reason: "OxideKit buttons don't have HTML-style 'type' prop".into(),
                component: Some("ui.Button".into()),
                correct_approach: "Handle form submission via on_click event".into(),
                correct_example: Some("Button { label: \"Submit\" on_click: submit_form }".into()),
            },
            InvalidPattern {
                id: "text-as-prop".into(),
                pattern: "Text { as: \"h1\" }".into(),
                reason: "Text uses 'role' for semantic meaning, not 'as'".into(),
                component: Some("ui.Text".into()),
                correct_approach: "Use role='heading' for headings".into(),
                correct_example: Some("Text { role: \"heading\" content: \"Title\" }".into()),
            },
            InvalidPattern {
                id: "class-className".into(),
                pattern: "className=\"...\" or class=\"...\"".into(),
                reason: "OxideKit uses design tokens, not CSS classes".into(),
                component: None,
                correct_approach: "Use style tokens and component props instead of classes".into(),
                correct_example: None,
            },
            InvalidPattern {
                id: "style-object".into(),
                pattern: "style={{ color: 'red' }}".into(),
                reason: "OxideKit doesn't use inline style objects".into(),
                component: None,
                correct_approach: "Use design tokens: color: token.color.danger".into(),
                correct_example: None,
            },
            InvalidPattern {
                id: "onclick-lowercase".into(),
                pattern: "onclick=... or onClick=...".into(),
                reason: "OxideKit uses snake_case for event handlers".into(),
                component: None,
                correct_approach: "Use on_click (snake_case) for event handlers".into(),
                correct_example: Some("Button { on_click: handle_click }".into()),
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_schema_version() {
        let v1 = SchemaVersion::v1();
        assert_eq!(v1.to_string(), "1.0");
        assert!(v1.is_compatible_with(&SchemaVersion::new(1, 0)));
    }

    #[test]
    fn test_ai_catalog_creation() {
        let catalog = AiCatalog::with_core();
        assert!(!catalog.schema.components.is_empty());
        assert!(catalog.get_component("ui.Button").is_some());
    }

    #[test]
    fn test_search_components() {
        let catalog = AiCatalog::with_core();
        let results = catalog.search_components("button");
        assert!(!results.is_empty());
    }

    #[test]
    fn test_export_json() {
        let catalog = AiCatalog::with_core();
        let json = catalog.export_json().unwrap();
        assert!(json.contains("oxide_ai_schema"));
        assert!(json.contains("ui.Button"));
    }

    #[test]
    fn test_invalid_patterns() {
        let catalog = AiCatalog::with_core();
        assert!(!catalog.schema.invalid_patterns.is_empty());
    }
}
