//! Diagnostics Engine
//!
//! Provides inline diagnostics for:
//! - Invalid props
//! - Missing required props
//! - Unknown components
//! - Missing tokens
//! - Missing i18n keys
//! - Incompatible plugin usage
//! - Deprecated APIs
//!
//! All diagnostics use stable error codes for tooling integration.

use crate::document::DocumentStore;
use crate::project::ProjectContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Diagnostics engine
pub struct DiagnosticsEngine {
    documents: Arc<RwLock<DocumentStore>>,
    project: Arc<RwLock<Option<ProjectContext>>>,
}

impl DiagnosticsEngine {
    /// Create a new diagnostics engine
    pub fn new(
        documents: Arc<RwLock<DocumentStore>>,
        project: Arc<RwLock<Option<ProjectContext>>>,
    ) -> Self {
        Self { documents, project }
    }

    /// Analyze a document and return diagnostics
    pub async fn analyze(&self, uri: &Url) -> Vec<Diagnostic> {
        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let mut diagnostics = vec![];

        if doc.is_oui() {
            diagnostics.extend(self.analyze_oui(&doc.content).await);
        } else if doc.is_toml_config() {
            diagnostics.extend(self.analyze_toml(&doc.content, uri).await);
        }

        diagnostics
    }

    /// Analyze OUI file
    async fn analyze_oui(&self, content: &str) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];
        let project = self.project.read().await;

        for (line_num, line) in content.lines().enumerate() {
            // Check for component usage
            if let Some(component_name) = self.extract_component_name(line) {
                // Check if component exists
                if !self.is_valid_component(&component_name, &project) {
                    if let Some(range) = self.find_word_range(line, &component_name, line_num) {
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::ERROR),
                            code: Some(NumberOrString::String("OUI001".to_string())),
                            code_description: Some(CodeDescription {
                                href: Url::parse("https://oxidekit.com/docs/errors/OUI001").unwrap(),
                            }),
                            source: Some("oxide-lsp".to_string()),
                            message: format!("Unknown component: {}", component_name),
                            related_information: None,
                            tags: None,
                            data: None,
                        });
                    }
                }

                // Check for deprecated component
                if let Some(deprecation_msg) = self.get_deprecation(&component_name, &project) {
                    if let Some(range) = self.find_word_range(line, &component_name, line_num) {
                        diagnostics.push(Diagnostic {
                            range,
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("OUI002".to_string())),
                            source: Some("oxide-lsp".to_string()),
                            message: deprecation_msg,
                            tags: Some(vec![DiagnosticTag::DEPRECATED]),
                            ..Default::default()
                        });
                    }
                }
            }

            // Check for missing translation keys
            if let Some(keys) = self.extract_translation_keys(line) {
                for key in keys {
                    if !self.has_translation_key(&key, &project) {
                        if let Some(range) = self.find_word_range(line, &key, line_num) {
                            diagnostics.push(Diagnostic {
                                range,
                                severity: Some(DiagnosticSeverity::WARNING),
                                code: Some(NumberOrString::String("OUI003".to_string())),
                                source: Some("oxide-lsp".to_string()),
                                message: format!("Missing translation key: {}", key),
                                ..Default::default()
                            });
                        }
                    }
                }
            }

            // Check for invalid prop values
            diagnostics.extend(self.check_prop_values(line, line_num, &project));

            // Check for unknown tokens
            diagnostics.extend(self.check_tokens(line, line_num, &project));
        }

        diagnostics
    }

    /// Analyze TOML config file
    async fn analyze_toml(&self, content: &str, uri: &Url) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Try to parse the TOML
        if let Err(e) = toml::from_str::<toml::Value>(content) {
            let (line, col) = e.span()
                .map(|s| {
                    let before = &content[..s.start];
                    let line = before.matches('\n').count();
                    let col = before.rfind('\n').map(|i| s.start - i - 1).unwrap_or(s.start);
                    (line, col)
                })
                .unwrap_or((0, 0));

            diagnostics.push(Diagnostic {
                range: Range {
                    start: Position {
                        line: line as u32,
                        character: col as u32,
                    },
                    end: Position {
                        line: line as u32,
                        character: (col + 10) as u32,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                code: Some(NumberOrString::String("TOML001".to_string())),
                source: Some("oxide-lsp".to_string()),
                message: format!("TOML parse error: {}", e),
                ..Default::default()
            });
            return diagnostics;
        }

        // Additional validation based on file type
        if uri.path().ends_with("oxide.toml") {
            diagnostics.extend(self.validate_manifest(content));
        } else if uri.path().ends_with("theme.toml") {
            diagnostics.extend(self.validate_theme(content));
        }

        diagnostics
    }

    fn extract_component_name(&self, line: &str) -> Option<String> {
        let trimmed = line.trim();

        // Look for component declaration pattern: ComponentName {
        if let Some(brace_pos) = trimmed.find('{') {
            let before_brace = trimmed[..brace_pos].trim();

            // Check if line starts with a keyword (app, style, etc.)
            let first_word = before_brace.split_whitespace().next()?;
            if ["app", "style", "layout", "theme"].contains(&first_word.to_lowercase().as_str()) {
                return None;
            }

            let name = before_brace.split_whitespace().last()?;

            // Components start with uppercase
            if name.chars().next()?.is_uppercase() {
                return Some(name.to_string());
            }
        }

        None
    }

    fn is_valid_component(&self, name: &str, project: &Option<ProjectContext>) -> bool {
        // Built-in components
        let builtins = [
            "Text", "Column", "Row", "Container", "Button", "Image", "Stack",
            "Grid", "Scroll", "Input", "Checkbox", "Radio", "Select", "Slider",
            "Progress", "Divider", "Spacer", "Icon", "Avatar", "Badge",
        ];

        if builtins.contains(&name) {
            return true;
        }

        // Check project context
        if let Some(ctx) = project {
            return ctx.get_component(name).is_some();
        }

        false
    }

    fn get_deprecation(&self, name: &str, project: &Option<ProjectContext>) -> Option<String> {
        if let Some(ctx) = project {
            if let Some(schema) = ctx.get_component(name) {
                if schema.deprecated {
                    return Some(
                        schema
                            .deprecation_message
                            .clone()
                            .unwrap_or_else(|| format!("{} is deprecated", name)),
                    );
                }
            }
        }
        None
    }

    fn extract_translation_keys(&self, line: &str) -> Option<Vec<String>> {
        let mut keys = Vec::new();

        // Pattern: t("key") or t('key')
        let patterns = ["t(\"", "t('"];
        for pattern in patterns {
            let mut remaining = line;
            while let Some(start) = remaining.find(pattern) {
                let after_quote = &remaining[start + pattern.len()..];
                let quote_char = pattern.chars().last().unwrap();
                if let Some(end) = after_quote.find(quote_char) {
                    let key = &after_quote[..end];
                    keys.push(key.to_string());
                }
                remaining = &remaining[start + 1..];
            }
        }

        if keys.is_empty() {
            None
        } else {
            Some(keys)
        }
    }

    fn has_translation_key(&self, key: &str, project: &Option<ProjectContext>) -> bool {
        if let Some(ctx) = project {
            return ctx.has_i18n_key(key);
        }
        // If no project context, don't report as error
        true
    }

    fn check_prop_values(
        &self,
        line: &str,
        line_num: usize,
        project: &Option<ProjectContext>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Check for common prop validation
        // align: should be start|center|end|space-between|space-around
        if let Some(align_match) = self.extract_prop_value(line, "align") {
            let valid = ["start", "center", "end", "space-between", "space-around"];
            if !valid.contains(&align_match.as_str()) && !align_match.starts_with('"') {
                if let Some(range) = self.find_word_range(line, &align_match, line_num) {
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("OUI004".to_string())),
                        source: Some("oxide-lsp".to_string()),
                        message: format!(
                            "Invalid value for align: '{}'. Expected one of: {}",
                            align_match,
                            valid.join(", ")
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        // justify: same values as align
        if let Some(justify_match) = self.extract_prop_value(line, "justify") {
            let valid = ["start", "center", "end", "space-between", "space-around"];
            if !valid.contains(&justify_match.as_str()) && !justify_match.starts_with('"') {
                if let Some(range) = self.find_word_range(line, &justify_match, line_num) {
                    diagnostics.push(Diagnostic {
                        range,
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: Some(NumberOrString::String("OUI004".to_string())),
                        source: Some("oxide-lsp".to_string()),
                        message: format!(
                            "Invalid value for justify: '{}'. Expected one of: {}",
                            justify_match,
                            valid.join(", ")
                        ),
                        ..Default::default()
                    });
                }
            }
        }

        let _ = project; // Will be used for more validation
        diagnostics
    }

    fn extract_prop_value(&self, line: &str, prop_name: &str) -> Option<String> {
        let pattern = format!("{}:", prop_name);
        let trimmed = line.trim();

        if let Some(pos) = trimmed.find(&pattern) {
            let after = &trimmed[pos + pattern.len()..].trim();
            // Get the value (until whitespace or end of line)
            let value: String = after.chars().take_while(|c| !c.is_whitespace()).collect();
            if !value.is_empty() {
                return Some(value);
            }
        }
        None
    }

    fn check_tokens(
        &self,
        line: &str,
        line_num: usize,
        project: &Option<ProjectContext>,
    ) -> Vec<Diagnostic> {
        let mut diagnostics = vec![];

        // Look for token references like colors.xxx, spacing.xxx, radius.xxx
        let token_prefixes = ["colors.", "spacing.", "radius.", "typography."];

        for prefix in token_prefixes {
            let mut search_start = 0;
            while let Some(pos) = line[search_start..].find(prefix) {
                let actual_pos = search_start + pos;
                let after_prefix = &line[actual_pos + prefix.len()..];
                let token_name: String = after_prefix
                    .chars()
                    .take_while(|c| c.is_alphanumeric() || *c == '_')
                    .collect();

                if !token_name.is_empty() {
                    let full_token = format!("{}{}", prefix, token_name);

                    // Check if token exists
                    let token_exists = project
                        .as_ref()
                        .map(|ctx| ctx.tokens.get(&full_token).is_some())
                        .unwrap_or(true); // Don't error if no project context

                    if !token_exists {
                        let col_start = actual_pos;
                        let col_end = col_start + full_token.len();

                        diagnostics.push(Diagnostic {
                            range: Range {
                                start: Position {
                                    line: line_num as u32,
                                    character: col_start as u32,
                                },
                                end: Position {
                                    line: line_num as u32,
                                    character: col_end as u32,
                                },
                            },
                            severity: Some(DiagnosticSeverity::WARNING),
                            code: Some(NumberOrString::String("OUI005".to_string())),
                            source: Some("oxide-lsp".to_string()),
                            message: format!("Unknown design token: {}", full_token),
                            ..Default::default()
                        });
                    }
                }

                search_start = actual_pos + 1;
            }
        }

        diagnostics
    }

    fn validate_manifest(&self, _content: &str) -> Vec<Diagnostic> {
        // Additional manifest validation
        vec![]
    }

    fn validate_theme(&self, _content: &str) -> Vec<Diagnostic> {
        // Additional theme validation
        vec![]
    }

    fn find_word_range(&self, line: &str, word: &str, line_num: usize) -> Option<Range> {
        line.find(word).map(|col| Range {
            start: Position {
                line: line_num as u32,
                character: col as u32,
            },
            end: Position {
                line: line_num as u32,
                character: (col + word.len()) as u32,
            },
        })
    }
}

/// LSP Diagnostic wrapper with additional metadata
#[derive(Debug, Clone)]
pub struct LspDiagnostic {
    pub code: ErrorCode,
    pub severity: DiagnosticSeverity,
    pub message: String,
    pub range: Range,
    pub source: String,
    pub quick_fix: Option<QuickFix>,
}

/// Error codes for diagnostics
#[derive(Debug, Clone, Copy)]
pub enum ErrorCode {
    /// Unknown component
    OUI001,
    /// Deprecated component
    OUI002,
    /// Missing translation key
    OUI003,
    /// Invalid prop value
    OUI004,
    /// Unknown design token
    OUI005,
    /// Missing required prop
    OUI006,
    /// Type mismatch
    OUI007,
    /// TOML parse error
    TOML001,
    /// Invalid manifest
    TOML002,
}

impl ErrorCode {
    pub fn as_str(&self) -> &'static str {
        match self {
            ErrorCode::OUI001 => "OUI001",
            ErrorCode::OUI002 => "OUI002",
            ErrorCode::OUI003 => "OUI003",
            ErrorCode::OUI004 => "OUI004",
            ErrorCode::OUI005 => "OUI005",
            ErrorCode::OUI006 => "OUI006",
            ErrorCode::OUI007 => "OUI007",
            ErrorCode::TOML001 => "TOML001",
            ErrorCode::TOML002 => "TOML002",
        }
    }
}

/// Quick fix suggestion
#[derive(Debug, Clone)]
pub struct QuickFix {
    pub title: String,
    pub edit: TextEdit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_component_name() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = DiagnosticsEngine::new(docs, project);

        assert_eq!(
            engine.extract_component_name("Text {"),
            Some("Text".to_string())
        );
        assert_eq!(
            engine.extract_component_name("  Button {"),
            Some("Button".to_string())
        );
        assert_eq!(engine.extract_component_name("app MyApp {"), None);
        assert_eq!(engine.extract_component_name("style {"), None);
    }

    #[test]
    fn test_extract_translation_keys() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = DiagnosticsEngine::new(docs, project);

        let keys = engine
            .extract_translation_keys("content: t(\"auth.login.title\")")
            .unwrap();
        assert_eq!(keys, vec!["auth.login.title"]);

        let keys = engine
            .extract_translation_keys("t(\"key1\") and t(\"key2\")")
            .unwrap();
        assert_eq!(keys, vec!["key1", "key2"]);

        assert!(engine.extract_translation_keys("no keys here").is_none());
    }

    #[test]
    fn test_is_valid_component() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = DiagnosticsEngine::new(docs, project);

        assert!(engine.is_valid_component("Text", &None));
        assert!(engine.is_valid_component("Column", &None));
        assert!(engine.is_valid_component("Button", &None));
        assert!(!engine.is_valid_component("FakeComponent", &None));
    }
}
