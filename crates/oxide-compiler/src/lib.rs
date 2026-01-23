//! OxideKit UI Compiler
//!
//! Compiles `.oui` files to intermediate representation (IR) for the runtime.

mod lexer;
mod parser;
mod static_gen;

pub use static_gen::generate_html;

use lexer::Lexer;
use parser::{Element, Parser, Value};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Compiler error types
#[derive(Debug, Error)]
pub enum CompilerError {
    #[error("Lexer error at line {line}, column {column}: {message}")]
    LexerError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Parse error at line {line}, column {column}: {message}")]
    ParseError {
        line: usize,
        column: usize,
        message: String,
    },

    #[error("Invalid component: {0}")]
    InvalidComponent(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Component IR - intermediate representation of a UI component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentIR {
    /// Stable component ID for hot reload
    pub id: String,
    /// Component type (e.g., "Text", "Column", "Row")
    pub kind: String,
    /// Component properties
    pub props: Vec<Property>,
    /// Style properties (separate for clarity)
    pub style: Vec<Property>,
    /// Event handlers (on click, on hover, etc.)
    pub handlers: Vec<HandlerIR>,
    /// Child components
    pub children: Vec<ComponentIR>,
}

/// Event handler IR
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerIR {
    /// Event type (click, hover, etc.)
    pub event: String,
    /// Handler expression
    pub handler: String,
}

/// A property on a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Property {
    pub name: String,
    pub value: PropertyValue,
}

/// Property value types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Number(f64),
    Bool(bool),
    /// State binding: references a reactive state value
    Binding { var: String },
}

/// Compile a .oui file to IR
///
/// # Arguments
/// * `source` - The .oui source code
///
/// # Returns
/// * `Ok(ComponentIR)` - The compiled component tree (root element)
/// * `Err(CompilerError)` - Compilation error
pub fn compile(source: &str) -> Result<ComponentIR, CompilerError> {
    tracing::debug!("Compiling OUI source ({} bytes)", source.len());

    // Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize().map_err(|e| CompilerError::LexerError {
        line: e.line,
        column: e.column,
        message: e.message,
    })?;

    // Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse().map_err(|e| CompilerError::ParseError {
        line: e.line,
        column: e.column,
        message: e.message,
    })?;

    // Generate IR
    let mut id_counter = 0;
    let children: Vec<ComponentIR> = program
        .app
        .children
        .into_iter()
        .map(|e| element_to_ir(e, &mut id_counter))
        .collect();

    // If there's only one root child, return it directly
    // Otherwise wrap in a Column
    if children.len() == 1 {
        Ok(children.into_iter().next().unwrap())
    } else {
        Ok(ComponentIR {
            id: format!("root_{}", id_counter),
            kind: "Column".to_string(),
            props: vec![],
            style: vec![],
            handlers: vec![],
            children,
        })
    }
}

/// Convert an AST element to IR
fn element_to_ir(element: Element, id_counter: &mut usize) -> ComponentIR {
    *id_counter += 1;
    let id = format!("{}_{}", element.name.to_lowercase(), id_counter);

    let props: Vec<Property> = element
        .properties
        .into_iter()
        .map(|p| Property {
            name: p.name,
            value: value_to_property_value(p.value),
        })
        .collect();

    let style: Vec<Property> = element
        .style
        .map(|s| {
            s.properties
                .into_iter()
                .map(|p| Property {
                    name: p.name,
                    value: value_to_property_value(p.value),
                })
                .collect()
        })
        .unwrap_or_default();

    let handlers: Vec<HandlerIR> = element
        .handlers
        .into_iter()
        .map(|h| HandlerIR {
            event: h.event,
            handler: h.handler,
        })
        .collect();

    let children: Vec<ComponentIR> = element
        .children
        .into_iter()
        .map(|e| element_to_ir(e, id_counter))
        .collect();

    ComponentIR {
        id,
        kind: element.name,
        props,
        style,
        handlers,
        children,
    }
}

/// Convert AST value to IR property value
fn value_to_property_value(value: Value) -> PropertyValue {
    match value {
        Value::String(s) => {
            // Check if this is a binding: {variable_name}
            if s.starts_with('{') && s.ends_with('}') && s.len() > 2 {
                let var = s[1..s.len()-1].trim().to_string();
                PropertyValue::Binding { var }
            } else {
                PropertyValue::String(s)
            }
        }
        Value::Number(n) => PropertyValue::Number(n),
        Value::Bool(b) => PropertyValue::Bool(b),
        Value::Ident(s) => PropertyValue::String(s), // Identifiers become strings for now
    }
}

/// Compile a .oui file from disk
pub fn compile_file(path: &std::path::Path) -> Result<ComponentIR, CompilerError> {
    let source = std::fs::read_to_string(path)?;
    compile(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compile_simple() {
        let source = r##"
            app HelloApp {
                Text {
                    content: "Hello OxideKit!"
                    size: 48
                    color: "#E5E7EB"
                }
            }
        "##;
        let result = compile(source).unwrap();

        assert_eq!(result.kind, "Text");
        assert_eq!(result.props.len(), 3);
    }

    #[test]
    fn test_compile_nested() {
        let source = r##"
            app MyApp {
                Column {
                    align: center
                    justify: center

                    Text {
                        content: "Title"
                        size: 32
                    }

                    Row {
                        gap: 16

                        Text { content: "A" }
                        Text { content: "B" }
                    }
                }
            }
        "##;
        let result = compile(source).unwrap();

        assert_eq!(result.kind, "Column");
        assert_eq!(result.children.len(), 2);
        assert_eq!(result.children[0].kind, "Text");
        assert_eq!(result.children[1].kind, "Row");
        assert_eq!(result.children[1].children.len(), 2);
    }

    #[test]
    fn test_compile_with_style() {
        let source = r##"
            app MyApp {
                Container {
                    style {
                        background: "#1F2937"
                        radius: 12
                        padding: 24
                    }

                    Text { content: "Styled" }
                }
            }
        "##;
        let result = compile(source).unwrap();

        assert_eq!(result.kind, "Container");
        assert_eq!(result.style.len(), 3);
        assert_eq!(result.children.len(), 1);
    }

    #[test]
    fn test_compile_complete_app() {
        let source = r##"
            app HelloApp {
                Column {
                    align: center
                    justify: center
                    width: fill
                    height: fill
                    gap: 24

                    style {
                        background: "#0B0F14"
                    }

                    Text {
                        content: "Hello OxideKit!"
                        size: 48
                        color: "#E5E7EB"
                    }

                    Text {
                        content: "A Rust-native application platform"
                        size: 20
                        color: "#9CA3AF"
                    }

                    Row {
                        gap: 16

                        Container {
                            style {
                                padding: 12
                                radius: 8
                                background: "#3B82F6"
                            }

                            Text {
                                content: "Get Started"
                                color: "#FFFFFF"
                            }
                        }

                        Container {
                            style {
                                padding: 12
                                radius: 8
                                border: 1
                                border_color: "#374151"
                            }

                            Text {
                                content: "Learn More"
                                color: "#9CA3AF"
                            }
                        }
                    }
                }
            }
        "##;
        let result = compile(source).unwrap();

        assert_eq!(result.kind, "Column");
        assert_eq!(result.children.len(), 3); // Two Text + Row
    }

    #[test]
    fn test_compile_with_handlers() {
        let source = r##"
            app TestApp {
                Container {
                    padding: 16
                    background: "#6366F1"
                    on click => navigate("/send")

                    Text {
                        content: "Send"
                    }
                }
            }
        "##;
        let result = compile(source).unwrap();

        assert_eq!(result.kind, "Container");
        assert_eq!(result.handlers.len(), 1, "Container should have 1 handler");
        assert_eq!(result.handlers[0].event, "click");
        assert!(result.handlers[0].handler.contains("navigate"), "Handler should contain navigate");
        assert!(result.handlers[0].handler.contains("/send"), "Handler should contain /send");
        println!("Handler: {}", result.handlers[0].handler);
    }
}
