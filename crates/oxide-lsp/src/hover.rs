//! Hover Engine
//!
//! Provides hover information including:
//! - Component descriptions
//! - Prop documentation
//! - Token resolution (where value comes from)
//! - Deprecation warnings

use crate::document::DocumentStore;
use crate::project::ProjectContext;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::lsp_types::*;

/// Hover engine
pub struct HoverEngine {
    documents: Arc<RwLock<DocumentStore>>,
    project: Arc<RwLock<Option<ProjectContext>>>,
}

impl HoverEngine {
    /// Create a new hover engine
    pub fn new(
        documents: Arc<RwLock<DocumentStore>>,
        project: Arc<RwLock<Option<ProjectContext>>>,
    ) -> Self {
        Self { documents, project }
    }

    /// Get hover information at a position
    pub async fn hover(&self, uri: &Url, position: Position) -> Option<Hover> {
        let docs = self.documents.read().await;
        let doc = docs.get(uri)?;

        let line = doc.get_line(position.line as usize)?;
        let word_info = doc.word_at(position.line as usize, position.character as usize)?;

        let project = self.project.read().await;

        // Try different hover providers
        if let Some(hover) = self.hover_component(&word_info.word, &project) {
            return Some(hover);
        }

        if let Some(hover) = self.hover_prop(&word_info.word, line, &project) {
            return Some(hover);
        }

        if let Some(hover) = self.hover_token(&word_info.word, &project) {
            return Some(hover);
        }

        if let Some(hover) = self.hover_keyword(&word_info.word) {
            return Some(hover);
        }

        None
    }

    /// Hover for component names
    fn hover_component(&self, word: &str, project: &Option<ProjectContext>) -> Option<Hover> {
        // Check project context first
        if let Some(ctx) = project {
            if let Some(schema) = ctx.get_component(word) {
                let mut content = format!("## {}\n\n{}", word, schema.description);

                if schema.deprecated {
                    content.push_str("\n\n**Deprecated**");
                    if let Some(msg) = &schema.deprecation_message {
                        content.push_str(&format!(": {}", msg));
                    }
                }

                // Add props documentation
                if !schema.props.is_empty() {
                    content.push_str("\n\n### Props\n\n");
                    for prop in &schema.props {
                        let required = if prop.required.unwrap_or(false) {
                            " *(required)*"
                        } else {
                            ""
                        };
                        content.push_str(&format!(
                            "- `{}`: `{}`{}\n",
                            prop.name, prop.prop_type, required
                        ));
                        if let Some(desc) = &prop.description {
                            content.push_str(&format!("  {}\n", desc));
                        }
                    }
                }

                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: content,
                    }),
                    range: None,
                });
            }
        }

        // Built-in components
        let builtin_docs = self.get_builtin_component_docs(word)?;
        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: builtin_docs,
            }),
            range: None,
        })
    }

    /// Get documentation for built-in components
    fn get_builtin_component_docs(&self, component: &str) -> Option<String> {
        let docs = match component {
            "Text" => r#"## Text

Display text content with customizable styling.

### Props

- `content`: `string` *(required)* - The text to display
- `size`: `number` - Font size in pixels
- `color`: `string` - Text color (hex, rgb, or token)
- `weight`: `string` - Font weight (normal, bold, 100-900)
- `align`: `string` - Text alignment (left, center, right)
- `wrap`: `boolean` - Whether text should wrap
"#,
            "Column" => r#"## Column

Vertical flex container that stacks children from top to bottom.

### Props

- `gap`: `number` - Space between children
- `align`: `string` - Cross-axis alignment (start, center, end)
- `justify`: `string` - Main-axis alignment (start, center, end, space-between, space-around)
- `width`: `dimension` - Container width (number, fill, wrap)
- `height`: `dimension` - Container height (number, fill, wrap)
- `padding`: `number` - Inner spacing
"#,
            "Row" => r#"## Row

Horizontal flex container that places children side by side.

### Props

- `gap`: `number` - Space between children
- `align`: `string` - Cross-axis alignment (start, center, end)
- `justify`: `string` - Main-axis alignment (start, center, end, space-between, space-around)
- `width`: `dimension` - Container width (number, fill, wrap)
- `height`: `dimension` - Container height (number, fill, wrap)
- `padding`: `number` - Inner spacing
"#,
            "Container" => r#"## Container

Generic container with full styling support.

### Props

- `width`: `dimension` - Container width
- `height`: `dimension` - Container height
- `padding`: `number` - Inner spacing
- `margin`: `number` - Outer spacing

### Style Props

Apply via `style { }` block:
- `background`: Background color
- `radius`: Border radius
- `border`: Border width
- `border_color`: Border color
- `shadow`: Box shadow
"#,
            "Button" => r#"## Button

Interactive button component.

### Props

- `label`: `string` *(required)* - Button text
- `variant`: `string` - Style variant (primary, secondary, outline, ghost)
- `size`: `string` - Button size (sm, md, lg)
- `disabled`: `boolean` - Whether button is disabled
- `loading`: `boolean` - Show loading state

### Events

- `on_click`: Triggered when button is clicked
"#,
            "Image" => r#"## Image

Display an image from a source.

### Props

- `src`: `string` *(required)* - Image source URL or path
- `alt`: `string` - Alt text for accessibility
- `width`: `dimension` - Image width
- `height`: `dimension` - Image height
- `fit`: `string` - Object fit (cover, contain, fill, none)
"#,
            "Stack" => r#"## Stack

Layer children on top of each other (z-axis stacking).

### Props

- `align`: `string` - Alignment for all children
- `width`: `dimension` - Stack width
- `height`: `dimension` - Stack height
"#,
            "Grid" => r#"## Grid

CSS Grid-like layout container.

### Props

- `columns`: `number` - Number of columns
- `rows`: `number` - Number of rows
- `gap`: `number` - Gap between cells
- `column_gap`: `number` - Horizontal gap
- `row_gap`: `number` - Vertical gap
"#,
            "Scroll" => r#"## Scroll

Scrollable container for overflow content.

### Props

- `direction`: `string` - Scroll direction (vertical, horizontal, both)
- `width`: `dimension` - Container width
- `height`: `dimension` - Container height
- `show_scrollbar`: `boolean` - Whether to show scrollbar
"#,
            "Input" => r#"## Input

Text input field.

### Props

- `value`: `string` - Current input value
- `placeholder`: `string` - Placeholder text
- `type`: `string` - Input type (text, password, email, number)
- `disabled`: `boolean` - Whether input is disabled
- `readonly`: `boolean` - Whether input is read-only

### Events

- `on_change`: Triggered when value changes
- `on_submit`: Triggered on Enter key
"#,
            _ => return None,
        };

        Some(docs.to_string())
    }

    /// Hover for property names
    fn hover_prop(
        &self,
        word: &str,
        line: &str,
        project: &Option<ProjectContext>,
    ) -> Option<Hover> {
        // Check if word looks like a property (followed by colon)
        if !line.contains(&format!("{}:", word)) {
            return None;
        }

        // Try to find the component context and get prop docs
        if let Some(ctx) = project {
            // This would need proper AST parsing to find the containing component
            // For now, check all components for this prop
            for (_id, schema) in &ctx.component_schemas {
                if let Some(prop) = schema.props.iter().find(|p| p.name == word) {
                    let mut content = format!("## {}\n\nType: `{}`", prop.name, prop.prop_type);

                    if let Some(desc) = &prop.description {
                        content.push_str(&format!("\n\n{}", desc));
                    }

                    if prop.required.unwrap_or(false) {
                        content.push_str("\n\n**Required**");
                    }

                    if let Some(values) = &prop.allowed_values {
                        content.push_str(&format!(
                            "\n\nAllowed values: `{}`",
                            values.join("`, `")
                        ));
                    }

                    if prop.deprecated.unwrap_or(false) {
                        content.push_str("\n\n**Deprecated**");
                    }

                    return Some(Hover {
                        contents: HoverContents::Markup(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: content,
                        }),
                        range: None,
                    });
                }
            }
        }

        // Built-in prop docs
        self.get_builtin_prop_docs(word).map(|docs| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs,
            }),
            range: None,
        })
    }

    /// Get documentation for common props
    fn get_builtin_prop_docs(&self, prop: &str) -> Option<String> {
        let docs = match prop {
            "content" => "## content\n\nType: `string`\n\nThe text content to display.",
            "size" => "## size\n\nType: `number`\n\nFont size in pixels or dimension value.",
            "color" => "## color\n\nType: `string`\n\nColor value. Accepts:\n- Hex: `#3B82F6`\n- RGB: `rgb(59, 130, 246)`\n- Token: `colors.primary`",
            "background" => "## background\n\nType: `string`\n\nBackground color. Same format as `color`.",
            "width" => "## width\n\nType: `dimension`\n\nWidth of the element. Accepts:\n- Number (pixels)\n- `fill` - Fill available space\n- `wrap` - Wrap to content",
            "height" => "## height\n\nType: `dimension`\n\nHeight of the element. Same format as `width`.",
            "padding" => "## padding\n\nType: `number`\n\nInner spacing in pixels.",
            "margin" => "## margin\n\nType: `number`\n\nOuter spacing in pixels.",
            "gap" => "## gap\n\nType: `number`\n\nSpace between children in layout containers.",
            "align" => "## align\n\nType: `string`\n\nCross-axis alignment.\n\nValues: `start`, `center`, `end`",
            "justify" => "## justify\n\nType: `string`\n\nMain-axis alignment.\n\nValues: `start`, `center`, `end`, `space-between`, `space-around`",
            "radius" => "## radius\n\nType: `number`\n\nBorder radius in pixels.",
            _ => return None,
        };

        Some(docs.to_string())
    }

    /// Hover for design tokens
    fn hover_token(&self, word: &str, project: &Option<ProjectContext>) -> Option<Hover> {
        // Check if it looks like a token reference
        if !word.contains('.') {
            return None;
        }

        // Check project tokens
        if let Some(ctx) = project {
            if let Some(value) = ctx.tokens.get(word) {
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!("## {}\n\nResolved value: `{}`", word, value),
                    }),
                    range: None,
                });
            }
        }

        // Default token values
        let default_tokens = [
            ("colors.primary", "#3B82F6"),
            ("colors.secondary", "#6B7280"),
            ("colors.background", "#0B0F14"),
            ("colors.surface", "#1F2937"),
            ("colors.text", "#E5E7EB"),
            ("spacing.xs", "4px"),
            ("spacing.sm", "8px"),
            ("spacing.md", "16px"),
            ("spacing.lg", "24px"),
            ("spacing.xl", "32px"),
            ("radius.sm", "4px"),
            ("radius.md", "8px"),
            ("radius.lg", "12px"),
        ];

        default_tokens
            .iter()
            .find(|(name, _)| *name == word)
            .map(|(name, value)| Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "## {}\n\nDefault value: `{}`\n\n*Override in theme.toml*",
                        name, value
                    ),
                }),
                range: None,
            })
    }

    /// Hover for OUI keywords
    fn hover_keyword(&self, word: &str) -> Option<Hover> {
        let docs = match word {
            "app" => "## app\n\nDeclares the root application component.\n\n```oui\napp MyApp {\n  // UI components here\n}\n```",
            "style" => "## style\n\nDefines visual styling for a component.\n\n```oui\nContainer {\n  style {\n    background: \"#1F2937\"\n    radius: 12\n    padding: 24\n  }\n}\n```",
            "fill" => "## fill\n\nDimension value that fills available space.\n\n```oui\nColumn {\n  width: fill\n  height: fill\n}\n```",
            "wrap" => "## wrap\n\nDimension value that wraps to content size.\n\n```oui\nContainer {\n  width: wrap\n  height: wrap\n}\n```",
            _ => return None,
        };

        Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: docs.to_string(),
            }),
            range: None,
        })
    }
}

/// Hover info with source attribution
#[derive(Debug, Clone)]
pub struct HoverInfo {
    pub content: String,
    pub kind: HoverKind,
}

/// Types of hover information
#[derive(Debug, Clone, Copy)]
pub enum HoverKind {
    Component,
    Prop,
    Token,
    Keyword,
    TranslationKey,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_component_docs() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = HoverEngine::new(docs, project);

        assert!(engine.get_builtin_component_docs("Text").is_some());
        assert!(engine.get_builtin_component_docs("Column").is_some());
        assert!(engine.get_builtin_component_docs("Row").is_some());
        assert!(engine.get_builtin_component_docs("Button").is_some());
        assert!(engine.get_builtin_component_docs("FakeComponent").is_none());
    }

    #[test]
    fn test_builtin_prop_docs() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = HoverEngine::new(docs, project);

        assert!(engine.get_builtin_prop_docs("content").is_some());
        assert!(engine.get_builtin_prop_docs("width").is_some());
        assert!(engine.get_builtin_prop_docs("align").is_some());
        assert!(engine.get_builtin_prop_docs("fake_prop").is_none());
    }

    #[tokio::test]
    async fn test_hover_token() {
        let docs = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));
        let engine = HoverEngine::new(docs, project);

        let hover = engine.hover_token("colors.primary", &None);
        assert!(hover.is_some());

        let hover = engine.hover_token("notaoken", &None);
        assert!(hover.is_none());
    }
}
