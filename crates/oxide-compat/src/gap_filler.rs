//! AI Gap-Filler Integration
//!
//! When an AI assistant requests a component/plugin that doesn't exist in OxideKit,
//! this module provides:
//!
//! 1. Detection of "missing component/plugin"
//! 2. Proposal of the best path forward:
//!    - Use native equivalent if exists
//!    - Use compat.webview widget (temporary)
//!    - Scaffold a new OxideKit plugin (preferred long-term)
//! 3. Creation of correctly named, non-colliding package ID
//! 4. Optionally create a PR with the scaffold
//!
//! This turns "missing plugin" from a blocker into a pipeline.

use crate::naming::{CanonicalId, IdGenerator, IdSuggestion, Namespace};
use crate::scaffold::{GeneratedScaffold, ScaffoldKind, ScaffoldOptions, Scaffolder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use thiserror::Error;

/// Errors in gap-filler operations
#[derive(Error, Debug)]
pub enum GapFillerError {
    /// Component not found
    #[error("Component not found: {0}")]
    ComponentNotFound(String),

    /// Scaffold generation failed
    #[error("Scaffold generation failed: {0}")]
    ScaffoldFailed(String),

    /// PR creation failed
    #[error("PR creation failed: {0}")]
    PrCreationFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
}

/// Request from AI for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRequest {
    /// Name/description of the requested component
    pub query: String,
    /// Hint for the type of component
    pub kind_hint: Option<String>,
    /// Namespace hint
    pub namespace_hint: Option<String>,
    /// Props the component should have
    pub expected_props: Option<Vec<String>>,
    /// Events the component should emit
    pub expected_events: Option<Vec<String>>,
    /// Context about why this component is needed
    pub context: Option<String>,
}

/// Suggestion type from gap-filler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// Use an existing native component
    NativeEquivalent,
    /// Use a compatibility WebView widget (temporary)
    CompatWidget,
    /// Scaffold a new native plugin
    ScaffoldPlugin,
    /// Scaffold a new native component
    ScaffoldComponent,
    /// Use an existing community package
    CommunityPackage,
    /// Component cannot be created (e.g., security reasons)
    NotPossible,
}

/// A suggestion for handling a missing component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentSuggestion {
    /// Type of suggestion
    pub suggestion_type: SuggestionType,
    /// Suggested canonical ID (for new scaffolds)
    pub suggested_id: Option<CanonicalId>,
    /// Existing component ID (for equivalents)
    pub existing_id: Option<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning for this suggestion
    pub reasoning: String,
    /// Code example
    pub example_code: Option<String>,
    /// Migration notes
    pub migration_notes: Option<String>,
    /// Warnings
    pub warnings: Vec<String>,
}

/// Response from the gap-filler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFillerResponse {
    /// Original request
    pub request: ComponentRequest,
    /// Whether a suitable solution was found
    pub found_solution: bool,
    /// Primary suggestion
    pub primary_suggestion: Option<ComponentSuggestion>,
    /// Alternative suggestions
    pub alternative_suggestions: Vec<ComponentSuggestion>,
    /// Generated scaffold (if applicable)
    pub scaffold: Option<ScaffoldInfo>,
    /// PR information (if created)
    pub pr_info: Option<PrInfo>,
}

/// Information about a generated scaffold
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScaffoldInfo {
    /// Canonical ID
    pub id: CanonicalId,
    /// Files generated
    pub files: Vec<String>,
    /// Output directory
    pub output_dir: PathBuf,
    /// Next steps
    pub next_steps: Vec<String>,
}

/// Information about a created PR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrInfo {
    /// PR URL
    pub url: String,
    /// PR number
    pub number: u64,
    /// Repository
    pub repository: String,
    /// Branch name
    pub branch: String,
    /// PR title
    pub title: String,
    /// PR status
    pub status: PrStatus,
}

/// PR status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrStatus {
    /// PR is open
    Open,
    /// PR is merged
    Merged,
    /// PR is closed
    Closed,
    /// PR creation pending
    Pending,
}

/// Known native components for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeComponent {
    /// Component ID
    pub id: String,
    /// Component name
    pub name: String,
    /// Description
    pub description: String,
    /// Keywords for matching
    pub keywords: Vec<String>,
    /// Example usage
    pub example: Option<String>,
}

/// Gap-filler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GapFillerConfig {
    /// Allow scaffold generation
    pub allow_scaffold: bool,
    /// Allow PR creation
    pub allow_pr_creation: bool,
    /// Target repository for PRs
    pub pr_target_repo: Option<String>,
    /// Minimum confidence for suggestions
    pub min_confidence: f32,
    /// Prefer native solutions
    pub prefer_native: bool,
}

impl Default for GapFillerConfig {
    fn default() -> Self {
        Self {
            allow_scaffold: true,
            allow_pr_creation: false,
            pr_target_repo: Some("oxidekit/oxidekit-extensions".to_string()),
            min_confidence: 0.3,
            prefer_native: true,
        }
    }
}

/// AI Gap-Filler service
pub struct GapFiller {
    /// Configuration
    config: GapFillerConfig,
    /// ID generator
    id_generator: IdGenerator,
    /// Scaffolder
    scaffolder: Scaffolder,
    /// Known native components
    native_components: Vec<NativeComponent>,
    /// Known community packages
    community_packages: Vec<NativeComponent>,
}

impl GapFiller {
    /// Create a new gap-filler
    pub fn new(config: GapFillerConfig) -> Self {
        Self {
            config,
            id_generator: IdGenerator::new(),
            scaffolder: Scaffolder::new(),
            native_components: Self::default_native_components(),
            community_packages: Vec::new(),
        }
    }

    /// Load known native components
    fn default_native_components() -> Vec<NativeComponent> {
        vec![
            NativeComponent {
                id: "ui.button".to_string(),
                name: "Button".to_string(),
                description: "Standard button component".to_string(),
                keywords: vec!["button", "click", "action", "submit"].into_iter().map(String::from).collect(),
                example: Some("<Button on:click={handler}>Click me</Button>".to_string()),
            },
            NativeComponent {
                id: "ui.input".to_string(),
                name: "Input".to_string(),
                description: "Text input field".to_string(),
                keywords: vec!["input", "text", "field", "form", "textbox"].into_iter().map(String::from).collect(),
                example: Some("<Input bind:value={text} placeholder=\"Enter text\" />".to_string()),
            },
            NativeComponent {
                id: "ui.list".to_string(),
                name: "List".to_string(),
                description: "List component for displaying items".to_string(),
                keywords: vec!["list", "items", "collection", "array"].into_iter().map(String::from).collect(),
                example: Some("<List items={data}>{|item| <ListItem>{item.name}</ListItem>}</List>".to_string()),
            },
            NativeComponent {
                id: "ui.modal".to_string(),
                name: "Modal".to_string(),
                description: "Modal dialog".to_string(),
                keywords: vec!["modal", "dialog", "popup", "overlay"].into_iter().map(String::from).collect(),
                example: Some("<Modal open={isOpen} on:close={handleClose}>Content</Modal>".to_string()),
            },
            NativeComponent {
                id: "ui.table".to_string(),
                name: "Table".to_string(),
                description: "Data table component".to_string(),
                keywords: vec!["table", "grid", "data", "rows", "columns"].into_iter().map(String::from).collect(),
                example: Some("<Table columns={cols} data={rows} />".to_string()),
            },
        ]
    }

    /// Load community packages from registry
    pub fn load_community_packages(&mut self, packages: Vec<NativeComponent>) {
        self.community_packages = packages;
    }

    /// Process a component request
    pub fn process(&self, request: ComponentRequest) -> GapFillerResponse {
        let query_lower = request.query.to_lowercase();

        // 1. Try to find a native equivalent
        if self.config.prefer_native {
            if let Some(suggestion) = self.find_native_equivalent(&query_lower) {
                if suggestion.confidence >= self.config.min_confidence {
                    return GapFillerResponse {
                        request,
                        found_solution: true,
                        primary_suggestion: Some(suggestion),
                        alternative_suggestions: Vec::new(),
                        scaffold: None,
                        pr_info: None,
                    };
                }
            }
        }

        // 2. Try to find a community package
        if let Some(suggestion) = self.find_community_package(&query_lower) {
            if suggestion.confidence >= self.config.min_confidence {
                return GapFillerResponse {
                    request,
                    found_solution: true,
                    primary_suggestion: Some(suggestion),
                    alternative_suggestions: Vec::new(),
                    scaffold: None,
                    pr_info: None,
                };
            }
        }

        // 3. Suggest scaffolding a new component
        let id_suggestion = self.suggest_id(&request);
        let scaffold_suggestion = self.create_scaffold_suggestion(&request, &id_suggestion);

        // 4. Consider compat widget as alternative
        let compat_alternative = self.create_compat_suggestion(&request);

        let mut alternatives = Vec::new();
        if let Some(compat) = compat_alternative {
            alternatives.push(compat);
        }

        GapFillerResponse {
            request,
            found_solution: true,
            primary_suggestion: Some(scaffold_suggestion),
            alternative_suggestions: alternatives,
            scaffold: None,
            pr_info: None,
        }
    }

    /// Find a native equivalent
    fn find_native_equivalent(&self, query: &str) -> Option<ComponentSuggestion> {
        let mut best_match: Option<(&NativeComponent, f32)> = None;

        for component in &self.native_components {
            let score = self.calculate_match_score(query, &component.keywords, &component.description);
            if score > best_match.as_ref().map(|(_, s)| *s).unwrap_or(0.0) {
                best_match = Some((component, score));
            }
        }

        best_match.map(|(component, score)| ComponentSuggestion {
            suggestion_type: SuggestionType::NativeEquivalent,
            suggested_id: None,
            existing_id: Some(component.id.clone()),
            confidence: score,
            reasoning: format!(
                "Found native equivalent '{}': {}",
                component.name, component.description
            ),
            example_code: component.example.clone(),
            migration_notes: None,
            warnings: Vec::new(),
        })
    }

    /// Find a community package
    fn find_community_package(&self, query: &str) -> Option<ComponentSuggestion> {
        let mut best_match: Option<(&NativeComponent, f32)> = None;

        for package in &self.community_packages {
            let score = self.calculate_match_score(query, &package.keywords, &package.description);
            if score > best_match.as_ref().map(|(_, s)| *s).unwrap_or(0.0) {
                best_match = Some((package, score));
            }
        }

        best_match.map(|(package, score)| ComponentSuggestion {
            suggestion_type: SuggestionType::CommunityPackage,
            suggested_id: None,
            existing_id: Some(package.id.clone()),
            confidence: score,
            reasoning: format!(
                "Found community package '{}': {}",
                package.name, package.description
            ),
            example_code: package.example.clone(),
            migration_notes: Some("Install with: oxide add <package-id>".to_string()),
            warnings: vec!["Community packages are not officially maintained".to_string()],
        })
    }

    /// Calculate match score between query and keywords
    fn calculate_match_score(&self, query: &str, keywords: &[String], description: &str) -> f32 {
        let query_words: Vec<&str> = query.split_whitespace().collect();
        let mut score = 0.0;
        let mut matches = 0;

        for word in &query_words {
            for keyword in keywords {
                if keyword.contains(word) || word.contains(keyword.as_str()) {
                    matches += 1;
                    score += 0.3;
                }
            }

            if description.to_lowercase().contains(word) {
                score += 0.1;
            }
        }

        // Normalize by query length
        if !query_words.is_empty() {
            score /= query_words.len() as f32;
        }

        // Boost for multiple matches
        if matches > 1 {
            score += 0.1 * (matches as f32 - 1.0);
        }

        score.min(1.0)
    }

    /// Suggest an ID for a new component
    fn suggest_id(&self, request: &ComponentRequest) -> IdSuggestion {
        let namespace_hint = request
            .namespace_hint
            .as_ref()
            .and_then(|s| Namespace::from_str(s));

        self.id_generator.suggest(&request.query, namespace_hint)
    }

    /// Create a scaffold suggestion
    fn create_scaffold_suggestion(
        &self,
        request: &ComponentRequest,
        id_suggestion: &IdSuggestion,
    ) -> ComponentSuggestion {
        let kind = match request.kind_hint.as_deref() {
            Some("component") => SuggestionType::ScaffoldComponent,
            _ => SuggestionType::ScaffoldPlugin,
        };

        ComponentSuggestion {
            suggestion_type: kind,
            suggested_id: Some(id_suggestion.id.clone()),
            existing_id: None,
            confidence: id_suggestion.confidence,
            reasoning: format!(
                "No existing equivalent found. Suggesting to scaffold a new {} with ID '{}'. {}",
                match kind {
                    SuggestionType::ScaffoldComponent => "component",
                    _ => "plugin",
                },
                id_suggestion.id,
                id_suggestion.reasoning
            ),
            example_code: None,
            migration_notes: Some(format!(
                "After scaffolding, implement the {} and submit for review.",
                match kind {
                    SuggestionType::ScaffoldComponent => "component",
                    _ => "plugin",
                }
            )),
            warnings: Vec::new(),
        }
    }

    /// Create a compat widget suggestion
    fn create_compat_suggestion(&self, request: &ComponentRequest) -> Option<ComponentSuggestion> {
        // Only suggest compat for certain types of requests
        let query_lower = request.query.to_lowercase();
        let is_ui_request = query_lower.contains("widget")
            || query_lower.contains("ui")
            || query_lower.contains("component")
            || query_lower.contains("chart")
            || query_lower.contains("calendar");

        if !is_ui_request {
            return None;
        }

        Some(ComponentSuggestion {
            suggestion_type: SuggestionType::CompatWidget,
            suggested_id: None,
            existing_id: None,
            confidence: 0.4,
            reasoning: "If a web-based solution already exists, you can temporarily use it via compat.webview while developing a native alternative.".to_string(),
            example_code: Some(r#"
// In oxide.toml:
[policy]
allow_webview = true

// Usage:
<WebWidget source="bundled:widgets/my-widget" />
"#.to_string()),
            migration_notes: Some("WARNING: compat.webview adds a web surface and increases attack surface. Plan to migrate to native OxideKit components.".to_string()),
            warnings: vec![
                "NOT RECOMMENDED for production".to_string(),
                "Increases attack surface".to_string(),
                "Consider as temporary solution only".to_string(),
            ],
        })
    }

    /// Generate a scaffold for a suggestion
    pub fn generate_scaffold(
        &self,
        suggestion: &ComponentSuggestion,
        output_dir: PathBuf,
    ) -> Result<GeneratedScaffold, GapFillerError> {
        if !self.config.allow_scaffold {
            return Err(GapFillerError::ScaffoldFailed(
                "Scaffold generation is disabled".to_string(),
            ));
        }

        let id = suggestion
            .suggested_id
            .as_ref()
            .ok_or_else(|| GapFillerError::ScaffoldFailed("No ID suggested".to_string()))?;

        let kind = match suggestion.suggestion_type {
            SuggestionType::ScaffoldComponent => ScaffoldKind::Component,
            SuggestionType::ScaffoldPlugin => ScaffoldKind::Plugin,
            SuggestionType::CompatWidget => ScaffoldKind::Widget,
            _ => {
                return Err(GapFillerError::ScaffoldFailed(
                    "Cannot scaffold this suggestion type".to_string(),
                ))
            }
        };

        let options = ScaffoldOptions {
            output_dir,
            id: id.clone(),
            kind,
            description: suggestion.reasoning.clone(),
            include_tests: true,
            include_ai_spec: true,
            include_docs: true,
            ..Default::default()
        };

        self.scaffolder
            .generate(&options)
            .map_err(|e| GapFillerError::ScaffoldFailed(e.to_string()))
    }

    /// Create a PR with the generated scaffold
    pub fn create_pr(
        &self,
        scaffold: &GeneratedScaffold,
        _pr_options: PrOptions,
    ) -> Result<PrInfo, GapFillerError> {
        if !self.config.allow_pr_creation {
            return Err(GapFillerError::PrCreationFailed(
                "PR creation is disabled".to_string(),
            ));
        }

        let repo = self.config.pr_target_repo.as_ref().ok_or_else(|| {
            GapFillerError::PrCreationFailed("No target repository configured".to_string())
        })?;

        // In a real implementation, this would use git2 and the GitHub API
        // For now, return a placeholder
        Ok(PrInfo {
            url: format!("https://github.com/{}/pull/0", repo),
            number: 0,
            repository: repo.clone(),
            branch: format!("scaffold/{}", scaffold.id),
            title: format!("Add {} scaffold", scaffold.id),
            status: PrStatus::Pending,
        })
    }
}

impl Default for GapFiller {
    fn default() -> Self {
        Self::new(GapFillerConfig::default())
    }
}

/// Options for PR creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrOptions {
    /// PR title
    pub title: Option<String>,
    /// PR description
    pub description: Option<String>,
    /// Labels to add
    pub labels: Vec<String>,
    /// Draft PR
    pub draft: bool,
}

impl Default for PrOptions {
    fn default() -> Self {
        Self {
            title: None,
            description: None,
            labels: vec!["scaffold".to_string(), "auto-generated".to_string()],
            draft: true,
        }
    }
}

/// MCP tools exposed by the gap-filler
pub mod mcp_tools {
    use super::*;

    /// Tool: oxide.find_component
    #[derive(Debug, Serialize, Deserialize)]
    pub struct FindComponentRequest {
        /// Search query
        pub query: String,
        /// Optional namespace hint
        pub namespace: Option<String>,
    }

    /// Tool: oxide.scaffold_plugin
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ScaffoldPluginRequest {
        /// Plugin ID
        pub id: String,
        /// Plugin kind
        pub kind: String,
        /// Description
        pub description: Option<String>,
        /// Output directory
        pub output_dir: Option<String>,
    }

    /// Tool: oxide.scaffold_component
    #[derive(Debug, Serialize, Deserialize)]
    pub struct ScaffoldComponentRequest {
        /// Pack ID (parent plugin)
        pub pack: String,
        /// Component name
        pub name: String,
        /// Props specification
        pub props: Option<HashMap<String, String>>,
        /// Events specification
        pub events: Option<Vec<String>>,
    }

    /// Tool: oxide.create_pr
    #[derive(Debug, Serialize, Deserialize)]
    pub struct CreatePrRequest {
        /// Target repository
        pub target_repo: String,
        /// Branch name
        pub branch: String,
        /// Files to include
        pub files: Vec<String>,
        /// Commit message
        pub message: String,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap_filler_creation() {
        let gap_filler = GapFiller::default();
        assert!(gap_filler.config.allow_scaffold);
    }

    #[test]
    fn test_find_native_equivalent() {
        let gap_filler = GapFiller::default();
        let request = ComponentRequest {
            query: "button component".to_string(),
            kind_hint: None,
            namespace_hint: None,
            expected_props: None,
            expected_events: None,
            context: None,
        };

        let response = gap_filler.process(request);
        assert!(response.found_solution);
        assert!(response.primary_suggestion.is_some());

        let suggestion = response.primary_suggestion.unwrap();
        assert_eq!(suggestion.suggestion_type, SuggestionType::NativeEquivalent);
    }

    #[test]
    fn test_scaffold_suggestion() {
        let gap_filler = GapFiller::default();
        let request = ComponentRequest {
            query: "very specific custom widget xyz".to_string(),
            kind_hint: Some("component".to_string()),
            namespace_hint: None,
            expected_props: None,
            expected_events: None,
            context: None,
        };

        let response = gap_filler.process(request);
        assert!(response.found_solution);

        let suggestion = response.primary_suggestion.unwrap();
        // Should suggest scaffolding since no native equivalent exists
        assert!(
            suggestion.suggestion_type == SuggestionType::ScaffoldComponent
                || suggestion.suggestion_type == SuggestionType::ScaffoldPlugin
        );
    }

    #[test]
    fn test_match_score() {
        let gap_filler = GapFiller::default();
        let keywords = vec!["button".to_string(), "click".to_string(), "action".to_string()];

        let score1 = gap_filler.calculate_match_score("button", &keywords, "A button component");
        let score2 = gap_filler.calculate_match_score("something else", &keywords, "A button component");

        assert!(score1 > score2);
    }
}
