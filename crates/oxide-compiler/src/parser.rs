//! Parser for .oui files
//!
//! Parses tokenized .oui source into an AST.

use crate::lexer::{Token, TokenKind};

/// AST node for a .oui program
#[derive(Debug, Clone)]
pub struct Program {
    /// Import statements
    pub imports: Vec<Import>,
    /// Component definitions
    pub components: Vec<ComponentDef>,
    /// The main app declaration
    pub app: AppDecl,
}

/// An import statement
#[derive(Debug, Clone)]
pub struct Import {
    /// Components being imported
    pub items: Vec<String>,
    /// Source path (file or module)
    pub from: String,
}

/// A custom component definition
#[derive(Debug, Clone)]
pub struct ComponentDef {
    /// Component name
    pub name: String,
    /// Component props (typed parameters)
    pub props: Vec<PropDef>,
    /// Component body (the UI tree)
    pub body: Element,
}

/// A prop definition with type and optional default
#[derive(Debug, Clone)]
pub struct PropDef {
    /// Prop name
    pub name: String,
    /// Prop type
    pub prop_type: PropType,
    /// Whether the prop is optional
    pub optional: bool,
    /// Default value (if any)
    pub default: Option<Value>,
}

/// Prop types
#[derive(Debug, Clone, PartialEq)]
pub enum PropType {
    String,
    Number,
    Bool,
    Color,
    Any,
}

/// App declaration
#[derive(Debug, Clone)]
pub struct AppDecl {
    pub name: String,
    pub children: Vec<Element>,
}

/// A UI element (component usage)
#[derive(Debug, Clone)]
pub struct Element {
    pub name: String,
    pub properties: Vec<PropertyDecl>,
    pub style: Option<StyleBlock>,
    pub children: Vec<Element>,
}

/// A property declaration
#[derive(Debug, Clone)]
pub struct PropertyDecl {
    pub name: String,
    pub value: Value,
}

/// A style block
#[derive(Debug, Clone)]
pub struct StyleBlock {
    pub properties: Vec<PropertyDecl>,
}

/// A value
#[derive(Debug, Clone)]
pub enum Value {
    String(String),
    Number(f64),
    Bool(bool),
    Ident(String),
}

/// Parser state
pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    /// Parse the token stream into a program
    pub fn parse(&mut self) -> Result<Program, ParseError> {
        // Parse imports first
        let mut imports = Vec::new();
        while self.check(TokenKind::Import) {
            imports.push(self.parse_import()?);
        }

        // Parse component definitions
        let mut components = Vec::new();
        while self.check(TokenKind::Component) {
            components.push(self.parse_component_def()?);
        }

        // Then parse the app
        let app = self.parse_app()?;
        Ok(Program {
            imports,
            components,
            app,
        })
    }

    /// Parse a component definition: component MyButton { props { ... } body { ... } }
    fn parse_component_def(&mut self) -> Result<ComponentDef, ParseError> {
        self.expect(TokenKind::Component)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        // Parse props block if present
        let mut props = Vec::new();
        if self.check(TokenKind::Prop) {
            self.advance(); // consume 'prop' keyword
            self.expect(TokenKind::LBrace)?;

            while !self.check(TokenKind::RBrace) && !self.is_at_end() {
                props.push(self.parse_prop_def()?);
            }

            self.expect(TokenKind::RBrace)?;
        }

        // Parse the component body (single element)
        let body = self.parse_element()?;

        self.expect(TokenKind::RBrace)?;

        Ok(ComponentDef { name, props, body })
    }

    /// Parse a prop definition: name: Type = default
    fn parse_prop_def(&mut self) -> Result<PropDef, ParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;

        // Parse type (with optional ?)
        let (prop_type, optional) = self.parse_prop_type()?;

        // Parse default value if present
        let default = if self.check(TokenKind::Equal) {
            self.advance(); // consume =
            Some(self.parse_value()?)
        } else {
            None
        };

        Ok(PropDef {
            name,
            prop_type,
            optional,
            default,
        })
    }

    /// Parse a prop type: String, Number, Bool, Color, or Type?
    fn parse_prop_type(&mut self) -> Result<(PropType, bool), ParseError> {
        let type_name = self.expect_ident()?;

        // Check for optional marker (?)
        let optional = if self.peek().kind == TokenKind::Question {
            self.advance();
            true
        } else {
            false
        };

        let prop_type = match type_name.to_lowercase().as_str() {
            "string" => PropType::String,
            "number" | "int" | "float" => PropType::Number,
            "bool" | "boolean" => PropType::Bool,
            "color" => PropType::Color,
            _ => PropType::Any,
        };

        Ok((prop_type, optional))
    }

    /// Parse an import statement: import { A, B } from "path"
    fn parse_import(&mut self) -> Result<Import, ParseError> {
        self.expect(TokenKind::Import)?;
        self.expect(TokenKind::LBrace)?;

        // Parse items (comma-separated identifiers)
        let mut items = Vec::new();
        loop {
            let name = self.expect_ident()?;
            items.push(name);

            if self.check(TokenKind::Comma) {
                self.advance(); // consume comma
            } else {
                break;
            }
        }

        self.expect(TokenKind::RBrace)?;
        self.expect(TokenKind::From)?;

        // Get the path string
        let from = match &self.peek().kind {
            TokenKind::String(s) => {
                let path = s.clone();
                self.advance();
                path
            }
            _ => {
                let token = self.peek().clone();
                return Err(ParseError {
                    line: token.line,
                    column: token.column,
                    message: format!("Expected string path after 'from', found {:?}", token.kind),
                });
            }
        };

        Ok(Import { items, from })
    }

    fn parse_app(&mut self) -> Result<AppDecl, ParseError> {
        self.expect(TokenKind::App)?;
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut children = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            children.push(self.parse_element()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(AppDecl { name, children })
    }

    fn parse_element(&mut self) -> Result<Element, ParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        let mut style = None;
        let mut children = Vec::new();

        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            // Check if this is a style block
            if self.check(TokenKind::Style) {
                self.advance();
                style = Some(self.parse_style_block()?);
            }
            // Check if this is a child element (capitalized identifier followed by brace)
            else if self.is_element_start() {
                children.push(self.parse_element()?);
            }
            // Otherwise it's a property
            else {
                properties.push(self.parse_property()?);
            }
        }

        self.expect(TokenKind::RBrace)?;

        Ok(Element {
            name,
            properties,
            style,
            children,
        })
    }

    fn parse_style_block(&mut self) -> Result<StyleBlock, ParseError> {
        self.expect(TokenKind::LBrace)?;

        let mut properties = Vec::new();
        while !self.check(TokenKind::RBrace) && !self.is_at_end() {
            properties.push(self.parse_property()?);
        }

        self.expect(TokenKind::RBrace)?;

        Ok(StyleBlock { properties })
    }

    fn parse_property(&mut self) -> Result<PropertyDecl, ParseError> {
        let name = self.expect_ident()?;
        self.expect(TokenKind::Colon)?;
        let value = self.parse_value()?;

        Ok(PropertyDecl { name, value })
    }

    fn parse_value(&mut self) -> Result<Value, ParseError> {
        let token = self.advance();
        match &token.kind {
            TokenKind::String(s) => Ok(Value::String(s.clone())),
            TokenKind::Number(n) => Ok(Value::Number(*n)),
            TokenKind::Bool(b) => Ok(Value::Bool(*b)),
            TokenKind::Ident(s) => Ok(Value::Ident(s.clone())),
            _ => Err(ParseError {
                line: token.line,
                column: token.column,
                message: format!("Expected value, found {:?}", token.kind),
            }),
        }
    }

    fn is_element_start(&self) -> bool {
        if let TokenKind::Ident(name) = &self.peek().kind {
            // Elements start with uppercase (convention)
            // or check if next token is LBrace
            if let Some(next) = self.tokens.get(self.current + 1) {
                return next.kind == TokenKind::LBrace
                    && name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
            }
        }
        false
    }

    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.peek().clone();
        if std::mem::discriminant(&token.kind) == std::mem::discriminant(&kind) {
            self.advance();
            Ok(token)
        } else {
            Err(ParseError {
                line: token.line,
                column: token.column,
                message: format!("Expected {:?}, found {:?}", kind, token.kind),
            })
        }
    }

    fn expect_ident(&mut self) -> Result<String, ParseError> {
        let token = self.advance();
        if let TokenKind::Ident(name) = &token.kind {
            Ok(name.clone())
        } else {
            Err(ParseError {
                line: token.line,
                column: token.column,
                message: format!("Expected identifier, found {:?}", token.kind),
            })
        }
    }

    fn check(&self, kind: TokenKind) -> bool {
        std::mem::discriminant(&self.peek().kind) == std::mem::discriminant(&kind)
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token {
            kind: TokenKind::Eof,
            line: 0,
            column: 0,
        })
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.tokens
            .get(self.current - 1)
            .cloned()
            .unwrap_or(Token {
                kind: TokenKind::Eof,
                line: 0,
                column: 0,
            })
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::Eof
    }
}

/// Parse error
#[derive(Debug)]
pub struct ParseError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_simple_app() {
        let source = r##"
            app HelloApp {
                Text {
                    content: "Hello"
                }
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.app.name, "HelloApp");
        assert_eq!(program.app.children.len(), 1);
        assert_eq!(program.app.children[0].name, "Text");
    }

    #[test]
    fn test_parse_nested_elements() {
        let source = r##"
            app MyApp {
                Column {
                    align: center

                    Text {
                        content: "Title"
                        size: 24
                    }

                    Row {
                        gap: 16

                        Text { content: "A" }
                        Text { content: "B" }
                    }
                }
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.app.children[0].name, "Column");
        assert_eq!(program.app.children[0].children.len(), 2); // Text and Row
    }

    #[test]
    fn test_parse_style_block() {
        let source = r##"
            app MyApp {
                Container {
                    style {
                        background: "#1F2937"
                        radius: 8
                    }
                }
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        let container = &program.app.children[0];
        assert!(container.style.is_some());
        assert_eq!(container.style.as_ref().unwrap().properties.len(), 2);
    }

    #[test]
    fn test_parse_import() {
        let source = r##"
            import { Button, Card } from "./components/ui.oui"

            app MyApp {
                Card {
                    Button { text: "Click" }
                }
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.imports.len(), 1);
        assert_eq!(program.imports[0].items, vec!["Button", "Card"]);
        assert_eq!(program.imports[0].from, "./components/ui.oui");
        assert_eq!(program.app.children[0].name, "Card");
    }

    #[test]
    fn test_parse_multiple_imports() {
        let source = r##"
            import { Navbar } from "./navbar.oui"
            import { Footer } from "./footer.oui"

            app MyApp {
                Navbar {}
                Footer {}
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.imports.len(), 2);
        assert_eq!(program.imports[0].items, vec!["Navbar"]);
        assert_eq!(program.imports[1].items, vec!["Footer"]);
    }

    #[test]
    fn test_parse_component_def() {
        let source = r##"
            component MyButton {
                prop {
                    text: String
                    disabled: Bool?
                    size: Number = 16
                }

                Button {
                    content: text
                    disabled: disabled
                }
            }

            app MyApp {
                MyButton { text: "Click me" }
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.components.len(), 1);
        assert_eq!(program.components[0].name, "MyButton");
        assert_eq!(program.components[0].props.len(), 3);

        // Check props
        let props = &program.components[0].props;
        assert_eq!(props[0].name, "text");
        assert_eq!(props[0].prop_type, PropType::String);
        assert!(!props[0].optional);

        assert_eq!(props[1].name, "disabled");
        assert_eq!(props[1].prop_type, PropType::Bool);
        assert!(props[1].optional);

        assert_eq!(props[2].name, "size");
        assert_eq!(props[2].prop_type, PropType::Number);
        assert!(props[2].default.is_some());
    }

    #[test]
    fn test_parse_component_without_props() {
        let source = r##"
            component Divider {
                Container {
                    height: 1
                    background: "#374151"
                }
            }

            app MyApp {
                Divider {}
            }
        "##;

        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();

        assert_eq!(program.components.len(), 1);
        assert_eq!(program.components[0].name, "Divider");
        assert!(program.components[0].props.is_empty());
        assert_eq!(program.components[0].body.name, "Container");
    }
}
