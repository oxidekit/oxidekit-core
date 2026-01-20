//! Completion Engine
//!
//! Provides intelligent autocomplete for:
//! - Component IDs (ui.Button, ui.DataTable)
//! - Props (with types and allowed enum values)
//! - Slots / children
//! - Token references (colors.primary, radius.md)
//! - Typography roles
//! - Translation keys (t("auth.login.title"))

use crate::document::DocumentStore;
use crate::project::ProjectContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Completion engine
pub struct CompletionEngine {
    documents: Arc<RwLock<DocumentStore>>,
    project: Arc<RwLock<Option<ProjectContext>>>,
}

impl CompletionEngine {
    /// Create a new completion engine
    pub fn new(
        documents: Arc<RwLock<DocumentStore>>,
        project: Arc<RwLock<Option<ProjectContext>>>,
    ) -> Self {
        Self { documents, project }
    }

    /// Generate completions at a position
    pub async fn complete(&self, uri: &Url, position: Position) -> Vec<CompletionItem> {
        let docs = self.documents.read().await;
        let doc = match docs.get(uri) {
            Some(d) => d,
            None => return vec![],
        };

        let line = match doc.get_line(position.line as usize) {
            Some(l) => l,
            None => return vec![],
        };

        let col = position.character as usize;
        let context = self.analyze_context(line, col);

        let project = self.project.read().await;

        match context {
            CompletionContext::ComponentName => self.complete_component(&project),
            CompletionContext::PropName(component) => {
                self.complete_prop(&project, &component)
            }
            CompletionContext::PropValue(component, prop) => {
                self.complete_prop_value(&project, &component, &prop)
            }
            CompletionContext::Token => self.complete_token(&project),
            CompletionContext::TranslationKey => self.complete_i18n(&project),
            CompletionContext::Unknown => self.complete_default(&project),
        }
    }

    /// Analyze the completion context from line content
    fn analyze_context(&self, line: &str, col: usize) -> CompletionContext {
        let before: String = line.chars().take(col).collect();
        let trimmed = before.trim();

        // Check for translation key context: t("...
        if trimmed.contains("t(\"") && !trimmed.ends_with("\")") {
            return CompletionContext::TranslationKey;
        }

        // Check for token reference context
        if trimmed.contains("colors.") || trimmed.contains("spacing.") || trimmed.contains("radius.") {
            return CompletionContext::Token;
        }

        // Check for prop value context: propName:
        if let Some(colon_pos) = trimmed.rfind(':') {
            let after_colon = &trimmed[colon_pos + 1..].trim();
            if after_colon.is_empty() || !after_colon.starts_with('"') {
                // Extract prop name
                let before_colon = &trimmed[..colon_pos].trim();
                if let Some(prop_name) = before_colon.split_whitespace().last() {
                    // Try to find the component context
                    if let Some(component) = self.find_containing_component(line, col) {
                        return CompletionContext::PropValue(component, prop_name.to_string());
                    }
                }
            }
        }

        // Check for prop name context (inside a component block)
        if trimmed.ends_with('{') || (trimmed.contains('{') && !trimmed.contains('}')) {
            if let Some(component) = self.extract_component_name(trimmed) {
                return CompletionContext::PropName(component);
            }
        }

        // Default to component name at start of line or after whitespace
        if trimmed.is_empty() || trimmed.ends_with('}') {
            return CompletionContext::ComponentName;
        }

        // Check if we're typing a component name
        let last_word: String = trimmed.chars().rev().take_while(|c| c.is_alphanumeric() || *c == '_').collect::<String>().chars().rev().collect();
        if !last_word.is_empty() && last_word.chars().next().unwrap().is_uppercase() {
            return CompletionContext::ComponentName;
        }

        CompletionContext::Unknown
    }

    fn find_containing_component(&self, _line: &str, _col: usize) -> Option<String> {
        // Simplified: In a real implementation, we'd parse the AST
        // For now, return None to fallback to generic completions
        None
    }

    fn extract_component_name(&self, text: &str) -> Option<String> {
        // Find the component name before the opening brace
        let brace_pos = text.rfind('{')?;
        let before_brace = text[..brace_pos].trim();
        let component = before_brace.split_whitespace().last()?;

        if component.chars().next()?.is_uppercase() {
            Some(component.to_string())
        } else {
            None
        }
    }

    fn complete_component(&self, project: &Option<ProjectContext>) -> Vec<CompletionItem> {
        let mut items = vec![];

        // Built-in components
        let builtins = vec![
            ("Text", "Display text content", "text"),
            ("Column", "Vertical layout container", "column"),
            ("Row", "Horizontal layout container", "row"),
            ("Container", "Generic container with styling", "container"),
            ("Button", "Interactive button", "button"),
            ("Image", "Display an image", "image"),
            ("Stack", "Overlapping layout", "stack"),
            ("Grid", "Grid layout", "grid"),
            ("Scroll", "Scrollable container", "scroll"),
            ("Input", "Text input field", "input"),
        ];

        for (name, desc, snippet_name) in builtins {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some(desc.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**{}**\n\n{}", name, desc),
                })),
                insert_text: Some(format!("{} {{\n\t$0\n}}", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                sort_text: Some(format!("0{}", snippet_name)),
                ..Default::default()
            });
        }

        // Add components from project context
        if let Some(ctx) = project {
            for id in ctx.component_ids() {
                if let Some(schema) = ctx.get_component(id) {
                    items.push(CompletionItem {
                        label: id.clone(),
                        kind: Some(CompletionItemKind::CLASS),
                        detail: Some(schema.description.clone()),
                        deprecated: if schema.deprecated {
                            Some(true)
                        } else {
                            None
                        },
                        sort_text: Some(format!("1{}", id)),
                        ..Default::default()
                    });
                }
            }
        }

        items
    }

    fn complete_prop(&self, project: &Option<ProjectContext>, component: &str) -> Vec<CompletionItem> {
        let mut items = vec![];

        // Get component schema
        if let Some(ctx) = project {
            if let Some(schema) = ctx.get_component(component) {
                for prop in &schema.props {
                    let required = prop.required.unwrap_or(false);
                    items.push(CompletionItem {
                        label: prop.name.clone(),
                        kind: Some(CompletionItemKind::PROPERTY),
                        detail: Some(format!("{}: {}", prop.name, prop.prop_type)),
                        documentation: prop.description.as_ref().map(|d| {
                            Documentation::MarkupContent(MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: d.clone(),
                            })
                        }),
                        insert_text: Some(format!("{}: $0", prop.name)),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        sort_text: Some(if required {
                            format!("0{}", prop.name)
                        } else {
                            format!("1{}", prop.name)
                        }),
                        deprecated: prop.deprecated,
                        ..Default::default()
                    });
                }
                return items;
            }
        }

        // Fallback: common props
        let common_props = vec![
            ("content", "string", "Text content"),
            ("size", "number", "Font size or dimension"),
            ("color", "string", "Color value"),
            ("background", "string", "Background color"),
            ("width", "dimension", "Width (number, fill, or wrap)"),
            ("height", "dimension", "Height (number, fill, or wrap)"),
            ("padding", "number", "Inner spacing"),
            ("margin", "number", "Outer spacing"),
            ("gap", "number", "Space between children"),
            ("align", "alignment", "Cross-axis alignment"),
            ("justify", "alignment", "Main-axis alignment"),
            ("radius", "number", "Border radius"),
        ];

        for (name, typ, desc) in common_props {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some(format!("{}: {}", name, typ)),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: desc.to_string(),
                })),
                insert_text: Some(format!("{}: $0", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }

        items
    }

    fn complete_prop_value(
        &self,
        project: &Option<ProjectContext>,
        component: &str,
        prop: &str,
    ) -> Vec<CompletionItem> {
        let mut items = vec![];

        // Get enum values from schema
        if let Some(ctx) = project {
            if let Some(schema) = ctx.get_component(component) {
                if let Some(prop_schema) = schema.props.iter().find(|p| p.name == prop) {
                    if let Some(allowed) = &prop_schema.allowed_values {
                        for value in allowed {
                            items.push(CompletionItem {
                                label: value.clone(),
                                kind: Some(CompletionItemKind::ENUM_MEMBER),
                                ..Default::default()
                            });
                        }
                        return items;
                    }
                }
            }
        }

        // Fallback: common prop values
        match prop {
            "align" | "justify" => {
                for value in &["start", "center", "end", "space-between", "space-around"] {
                    items.push(CompletionItem {
                        label: value.to_string(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        ..Default::default()
                    });
                }
            }
            "width" | "height" => {
                for value in &["fill", "wrap"] {
                    items.push(CompletionItem {
                        label: value.to_string(),
                        kind: Some(CompletionItemKind::ENUM_MEMBER),
                        ..Default::default()
                    });
                }
            }
            _ => {}
        }

        items
    }

    fn complete_token(&self, project: &Option<ProjectContext>) -> Vec<CompletionItem> {
        let mut items = vec![];

        if let Some(ctx) = project {
            for name in ctx.token_names() {
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: ctx.tokens.get(name).cloned(),
                    ..Default::default()
                });
            }
        }

        // Fallback tokens
        if items.is_empty() {
            let tokens = vec![
                ("colors.primary", "#3B82F6"),
                ("colors.secondary", "#6B7280"),
                ("colors.background", "#0B0F14"),
                ("colors.surface", "#1F2937"),
                ("colors.text", "#E5E7EB"),
                ("spacing.xs", "4"),
                ("spacing.sm", "8"),
                ("spacing.md", "16"),
                ("spacing.lg", "24"),
                ("spacing.xl", "32"),
                ("radius.sm", "4"),
                ("radius.md", "8"),
                ("radius.lg", "12"),
            ];

            for (name, value) in tokens {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some(value.to_string()),
                    ..Default::default()
                });
            }
        }

        items
    }

    fn complete_i18n(&self, project: &Option<ProjectContext>) -> Vec<CompletionItem> {
        let mut items = vec![];

        if let Some(ctx) = project {
            for key in &ctx.i18n_keys {
                items.push(CompletionItem {
                    label: key.clone(),
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some("Translation key".to_string()),
                    ..Default::default()
                });
            }
        }

        items
    }

    fn complete_default(&self, project: &Option<ProjectContext>) -> Vec<CompletionItem> {
        // Combine component and common completions
        let mut items = self.complete_component(project);
        items.extend(self.complete_token(project));
        items
    }
}

/// Completion context types
#[derive(Debug)]
enum CompletionContext {
    /// Completing a component name
    ComponentName,
    /// Completing a prop name for a specific component
    PropName(String),
    /// Completing a prop value for a specific prop
    PropValue(String, String),
    /// Completing a design token
    Token,
    /// Completing a translation key
    TranslationKey,
    /// Unknown context
    Unknown,
}

/// Completion item with additional metadata
#[derive(Debug, Clone)]
pub struct OxideCompletionItem {
    pub label: String,
    pub kind: CompletionKind,
    pub detail: Option<String>,
    pub documentation: Option<String>,
    pub insert_text: Option<String>,
    pub deprecated: bool,
}

/// Completion item kinds
#[derive(Debug, Clone, Copy)]
pub enum CompletionKind {
    Component,
    Property,
    EnumValue,
    Token,
    TranslationKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_completion_context_detection() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = CompletionEngine::new(docs, project);

        // Test component name context
        let ctx = engine.analyze_context("    ", 4);
        assert!(matches!(ctx, CompletionContext::ComponentName));

        // Test prop name context
        let ctx = engine.analyze_context("Text {", 6);
        assert!(matches!(ctx, CompletionContext::PropName(_)));

        // Test translation key context
        let ctx = engine.analyze_context("content: t(\"auth.", 17);
        assert!(matches!(ctx, CompletionContext::TranslationKey));

        // Test token context
        let ctx = engine.analyze_context("color: colors.", 14);
        assert!(matches!(ctx, CompletionContext::Token));
    }
}
