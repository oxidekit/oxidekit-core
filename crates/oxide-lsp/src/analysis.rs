//! Document Analysis Utilities
//!
//! Provides shared analysis capabilities used by multiple engines.

use tower_lsp::lsp_types::*;

/// Semantic analysis context for a document position
#[derive(Debug, Clone)]
pub struct AnalysisContext {
    /// The type of context we're in
    pub context_type: ContextType,
    /// Current component (if inside one)
    pub component: Option<String>,
    /// Current property (if on one)
    pub property: Option<String>,
    /// Nesting depth
    pub depth: usize,
}

/// Types of analysis contexts
#[derive(Debug, Clone, PartialEq)]
pub enum ContextType {
    /// At the root level of the document
    Root,
    /// Inside an app declaration
    App,
    /// Inside a component body
    Component,
    /// Inside a style block
    Style,
    /// On a property name
    PropertyName,
    /// On a property value
    PropertyValue,
    /// Inside a translation function
    Translation,
    /// Inside a string literal
    String,
}

impl AnalysisContext {
    /// Analyze context at a position in OUI content
    pub fn analyze(content: &str, position: Position) -> Self {
        let mut context = Self {
            context_type: ContextType::Root,
            component: None,
            property: None,
            depth: 0,
        };

        let target_line = position.line as usize;
        let target_col = position.character as usize;
        let mut current_component: Option<String> = None;
        let mut in_style = false;
        let mut brace_depth = 0;

        for (line_num, line) in content.lines().enumerate() {
            if line_num > target_line {
                break;
            }

            let line_to_analyze = if line_num == target_line {
                &line[..target_col.min(line.len())]
            } else {
                line
            };

            // Track brace depth and context
            for (col, ch) in line_to_analyze.chars().enumerate() {
                match ch {
                    '{' => {
                        brace_depth += 1;
                        // Check what opened this brace
                        let before: String = line_to_analyze[..col].trim().chars().collect();
                        if before.starts_with("app ") {
                            context.context_type = ContextType::App;
                        } else if before == "style" {
                            in_style = true;
                            context.context_type = ContextType::Style;
                        } else if let Some(component) = extract_component_before_brace(&before) {
                            current_component = Some(component.clone());
                            context.component = Some(component);
                            context.context_type = ContextType::Component;
                        }
                    }
                    '}' => {
                        brace_depth -= 1;
                        if in_style && brace_depth > 0 {
                            in_style = false;
                            context.context_type = if current_component.is_some() {
                                ContextType::Component
                            } else {
                                ContextType::App
                            };
                        }
                    }
                    _ => {}
                }
            }

            // Check for specific contexts on the target line
            if line_num == target_line {
                let before_cursor: String = line_to_analyze.chars().collect();
                let trimmed = before_cursor.trim();

                // Check for translation context
                if trimmed.contains("t(\"") || trimmed.contains("t('") {
                    context.context_type = ContextType::Translation;
                }

                // Check for string context
                let quote_count = trimmed.matches('"').count();
                if quote_count % 2 == 1 {
                    context.context_type = ContextType::String;
                }

                // Check for property context
                if let Some(colon_pos) = trimmed.rfind(':') {
                    let prop_name: String = trimmed[..colon_pos].trim().to_string();
                    if !prop_name.is_empty() && !prop_name.contains(' ') {
                        context.property = Some(prop_name);
                        // Cursor is on or after colon = PropertyValue context
                        context.context_type = ContextType::PropertyValue;
                    }
                } else if !trimmed.is_empty()
                    && !trimmed.ends_with('{')
                    && !trimmed.ends_with('}')
                {
                    context.context_type = ContextType::PropertyName;
                }
            }
        }

        context.depth = brace_depth.max(0) as usize;
        context
    }
}

/// Extract component name from text before opening brace
fn extract_component_before_brace(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let last = words.last()?;

    // Component names start with uppercase
    if last.chars().next()?.is_uppercase() {
        // Exclude keywords
        if !["App", "style"].contains(last) {
            return Some(last.to_string());
        }
    }

    None
}

/// OUI document structure
#[derive(Debug)]
pub struct OuiDocument {
    /// App name
    pub app_name: Option<String>,
    /// Root elements
    pub elements: Vec<OuiElement>,
}

/// Element in OUI document
#[derive(Debug)]
pub struct OuiElement {
    /// Component type
    pub component: String,
    /// Line number
    pub line: usize,
    /// Properties
    pub properties: Vec<OuiProperty>,
    /// Children
    pub children: Vec<OuiElement>,
    /// Style block
    pub style: Option<Vec<OuiProperty>>,
}

/// Property in OUI
#[derive(Debug)]
pub struct OuiProperty {
    pub name: String,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

impl OuiDocument {
    /// Parse OUI content (simplified parser for LSP use)
    pub fn parse(content: &str) -> Self {
        let mut doc = Self {
            app_name: None,
            elements: vec![],
        };

        // Extract app name
        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.starts_with("app ") {
                if let Some(name_end) = trimmed.find('{') {
                    doc.app_name = Some(trimmed[4..name_end].trim().to_string());
                }
                break;
            }
        }

        doc
    }
}

/// Get semantic tokens for a document
pub fn get_semantic_tokens(content: &str) -> Vec<SemanticToken> {
    let mut tokens = vec![];
    let mut prev_line = 0u32;
    let mut prev_start = 0u32;

    for (line_num, line) in content.lines().enumerate() {
        let line_num = line_num as u32;
        let mut col = 0;

        for token in tokenize_line(line) {
            let delta_line = line_num - prev_line;
            let delta_start = if delta_line == 0 {
                token.start as u32 - prev_start
            } else {
                token.start as u32
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: token.length as u32,
                token_type: token.token_type as u32,
                token_modifiers_bitset: token.modifiers,
            });

            prev_line = line_num;
            prev_start = token.start as u32;
            col = token.start + token.length;
        }

        let _ = col; // Suppress unused warning
    }

    tokens
}

/// Token from line tokenization
struct LineToken {
    start: usize,
    length: usize,
    token_type: usize,
    modifiers: u32,
}

/// Tokenize a single line
fn tokenize_line(line: &str) -> Vec<LineToken> {
    let mut tokens = vec![];

    let keywords = ["app", "style"];
    let mut i = 0;

    while i < line.len() {
        let remaining = &line[i..];

        // Skip whitespace
        if remaining.starts_with(char::is_whitespace) {
            i += 1;
            continue;
        }

        // Check for keywords
        let mut found_keyword = false;
        for keyword in keywords {
            if remaining.starts_with(keyword)
                && remaining
                    .chars()
                    .nth(keyword.len())
                    .map(|c| !c.is_alphanumeric() && c != '_')
                    .unwrap_or(true)
            {
                tokens.push(LineToken {
                    start: i,
                    length: keyword.len(),
                    token_type: 0, // KEYWORD
                    modifiers: 0,
                });
                i += keyword.len();
                found_keyword = true;
                break;
            }
        }

        if found_keyword {
            continue;
        }

        // Check for component name (PascalCase)
        if remaining.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            let len = remaining
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count();
            tokens.push(LineToken {
                start: i,
                length: len,
                token_type: 1, // TYPE (component)
                modifiers: 0,
            });
            i += len;
            continue;
        }

        // Check for property name (identifier followed by colon)
        if remaining.chars().next().map(|c| c.is_alphabetic()).unwrap_or(false) {
            let len = remaining
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .count();
            if remaining[len..].trim_start().starts_with(':') {
                tokens.push(LineToken {
                    start: i,
                    length: len,
                    token_type: 2, // PROPERTY
                    modifiers: 0,
                });
            }
            i += len;
            continue;
        }

        // Check for string
        if remaining.starts_with('"') {
            let end = remaining[1..]
                .find('"')
                .map(|p| p + 2)
                .unwrap_or(remaining.len());
            tokens.push(LineToken {
                start: i,
                length: end,
                token_type: 3, // STRING
                modifiers: 0,
            });
            i += end;
            continue;
        }

        // Check for number
        if remaining.chars().next().map(|c| c.is_numeric()).unwrap_or(false) {
            let len = remaining
                .chars()
                .take_while(|c| c.is_numeric() || *c == '.')
                .count();
            tokens.push(LineToken {
                start: i,
                length: len,
                token_type: 4, // NUMBER
                modifiers: 0,
            });
            i += len;
            continue;
        }

        // Check for comment
        if remaining.starts_with("//") {
            tokens.push(LineToken {
                start: i,
                length: remaining.len(),
                token_type: 7, // COMMENT
                modifiers: 0,
            });
            break;
        }

        i += 1;
    }

    tokens
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_analysis() {
        let content = r#"app MyApp {
    Column {
        gap: 16
        Text {
            content: "Hello"
        }
    }
}"#;

        // Inside Column, on gap property value (character 14 is after "gap: ")
        let ctx = AnalysisContext::analyze(content, Position { line: 2, character: 14 });
        assert_eq!(ctx.component, Some("Column".to_string()));
        assert!(matches!(ctx.context_type, ContextType::PropertyValue));

        // Inside Text
        let ctx = AnalysisContext::analyze(content, Position { line: 4, character: 15 });
        assert_eq!(ctx.component, Some("Text".to_string()));
    }

    #[test]
    fn test_oui_document_parse() {
        let content = "app HelloApp {\n    Text { content: \"Hello\" }\n}";
        let doc = OuiDocument::parse(content);
        assert_eq!(doc.app_name, Some("HelloApp".to_string()));
    }

    #[test]
    fn test_tokenize_line() {
        let tokens = tokenize_line("Text { content: \"hello\" }");
        assert!(!tokens.is_empty());
        // First token should be the component name
        assert_eq!(tokens[0].token_type, 1); // TYPE
    }
}
