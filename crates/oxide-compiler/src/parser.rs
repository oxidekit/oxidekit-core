//! Parser for .oui files
//!
//! Parses tokenized .oui source into an AST.

use crate::lexer::{Token, TokenKind};

/// AST node for a .oui program
#[derive(Debug, Clone)]
pub struct Program {
    pub app: AppDecl,
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
        let app = self.parse_app()?;
        Ok(Program { app })
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
}
