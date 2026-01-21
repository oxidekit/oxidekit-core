//! Lexer for .oui files
//!
//! Tokenizes .oui source code for the parser.

use std::iter::Peekable;
use std::str::Chars;

/// Token types
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    App,
    Component,
    Prop,
    Style,
    Import,
    From,
    On,

    // Literals
    Ident(String),
    String(String),
    Number(f64),
    Bool(bool),

    // Punctuation
    LBrace,    // {
    RBrace,    // }
    LParen,    // (
    RParen,    // )
    Colon,     // :
    Comma,     // ,
    Dot,       // .
    Equal,     // =
    Arrow,     // =>
    Question,  // ?

    // Special
    Eof,
}

/// A token with position information
#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub line: usize,
    pub column: usize,
}

/// Lexer state
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<Chars<'a>>,
    current: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self {
            source,
            chars: source.chars().peekable(),
            current: 0,
            line: 1,
            column: 1,
        }
    }

    /// Tokenize the entire source
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();

        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof {
                break;
            }
        }

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexerError> {
        self.skip_whitespace_and_comments();

        let line = self.line;
        let column = self.column;

        let kind = match self.peek() {
            None => TokenKind::Eof,
            Some(c) => match c {
                '{' => {
                    self.advance();
                    TokenKind::LBrace
                }
                '}' => {
                    self.advance();
                    TokenKind::RBrace
                }
                '(' => {
                    self.advance();
                    TokenKind::LParen
                }
                ')' => {
                    self.advance();
                    TokenKind::RParen
                }
                ':' => {
                    self.advance();
                    TokenKind::Colon
                }
                ',' => {
                    self.advance();
                    TokenKind::Comma
                }
                '.' => {
                    self.advance();
                    TokenKind::Dot
                }
                '=' => {
                    self.advance();
                    if self.peek() == Some('>') {
                        self.advance();
                        TokenKind::Arrow
                    } else {
                        TokenKind::Equal
                    }
                }
                '?' => {
                    self.advance();
                    TokenKind::Question
                }
                '"' => self.string()?,
                c if c.is_ascii_digit() || c == '-' => self.number()?,
                c if c.is_alphabetic() || c == '_' => self.identifier_or_keyword(),
                c => {
                    return Err(LexerError {
                        line,
                        column,
                        message: format!("Unexpected character: '{}'", c),
                    })
                }
            },
        };

        Ok(Token { kind, line, column })
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn advance(&mut self) -> Option<char> {
        let c = self.chars.next();
        if let Some(ch) = c {
            self.current += ch.len_utf8();
            if ch == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
        }
        c
    }

    fn skip_whitespace_and_comments(&mut self) {
        loop {
            match self.peek() {
                Some(c) if c.is_whitespace() => {
                    self.advance();
                }
                Some('/') => {
                    // Check for comments
                    let remaining = &self.source[self.current..];
                    if remaining.starts_with("//") {
                        // Single-line comment
                        while let Some(c) = self.peek() {
                            if c == '\n' {
                                break;
                            }
                            self.advance();
                        }
                    } else if remaining.starts_with("/*") {
                        // Multi-line comment
                        self.advance(); // /
                        self.advance(); // *
                        while let Some(c) = self.advance() {
                            if c == '*' && self.peek() == Some('/') {
                                self.advance();
                                break;
                            }
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn string(&mut self) -> Result<TokenKind, LexerError> {
        let line = self.line;
        let column = self.column;
        self.advance(); // consume opening quote

        let mut value = String::new();
        loop {
            match self.advance() {
                Some('"') => break,
                Some('\\') => {
                    // Handle escape sequences
                    match self.advance() {
                        Some('n') => value.push('\n'),
                        Some('t') => value.push('\t'),
                        Some('r') => value.push('\r'),
                        Some('"') => value.push('"'),
                        Some('\\') => value.push('\\'),
                        Some(c) => {
                            return Err(LexerError {
                                line,
                                column,
                                message: format!("Invalid escape sequence: \\{}", c),
                            })
                        }
                        None => {
                            return Err(LexerError {
                                line,
                                column,
                                message: "Unterminated escape sequence".to_string(),
                            })
                        }
                    }
                }
                Some(c) => value.push(c),
                None => {
                    return Err(LexerError {
                        line,
                        column,
                        message: "Unterminated string".to_string(),
                    })
                }
            }
        }

        Ok(TokenKind::String(value))
    }

    fn number(&mut self) -> Result<TokenKind, LexerError> {
        let mut value = String::new();

        // Handle negative sign
        if self.peek() == Some('-') {
            value.push(self.advance().unwrap());
        }

        // Integer part
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                value.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Decimal part
        if self.peek() == Some('.') {
            value.push(self.advance().unwrap());
            while let Some(c) = self.peek() {
                if c.is_ascii_digit() {
                    value.push(self.advance().unwrap());
                } else {
                    break;
                }
            }
        }

        let num: f64 = value.parse().map_err(|_| LexerError {
            line: self.line,
            column: self.column,
            message: format!("Invalid number: {}", value),
        })?;

        Ok(TokenKind::Number(num))
    }

    fn identifier_or_keyword(&mut self) -> TokenKind {
        let mut value = String::new();

        while let Some(c) = self.peek() {
            if c.is_alphanumeric() || c == '_' {
                value.push(self.advance().unwrap());
            } else if c == '-' {
                // Allow hyphens in identifiers (e.g., my-site, my-app)
                // But only if followed by an alphanumeric character
                // This prevents capturing trailing hyphens or the minus operator
                let remaining = &self.source[self.current..];
                if remaining.len() > 1 {
                    let next_char = remaining.chars().nth(1);
                    if let Some(nc) = next_char {
                        if nc.is_alphanumeric() {
                            value.push(self.advance().unwrap()); // consume the hyphen
                            continue;
                        }
                    }
                }
                break;
            } else {
                break;
            }
        }

        // Check for keywords
        match value.as_str() {
            "app" => TokenKind::App,
            "component" => TokenKind::Component,
            "prop" => TokenKind::Prop,
            "style" => TokenKind::Style,
            "import" => TokenKind::Import,
            "from" => TokenKind::From,
            "on" => TokenKind::On,
            "true" => TokenKind::Bool(true),
            "false" => TokenKind::Bool(false),
            _ => TokenKind::Ident(value),
        }
    }
}

/// Lexer error
#[derive(Debug)]
pub struct LexerError {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let source = r#"app Hello { Text { content: "Hi" } }"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::App);
        assert!(matches!(tokens[1].kind, TokenKind::Ident(ref s) if s == "Hello"));
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
    }

    #[test]
    fn test_tokenize_numbers() {
        let source = "42 3.14 -10";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::Number(42.0));
        assert_eq!(tokens[1].kind, TokenKind::Number(3.14));
        assert_eq!(tokens[2].kind, TokenKind::Number(-10.0));
    }

    #[test]
    fn test_tokenize_comments() {
        let source = r#"
            // This is a comment
            app Hello { /* multi
            line */ }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::App);
    }

    #[test]
    fn test_tokenize_hyphenated_identifiers() {
        let source = "app my-site { Text { content: \"Hello\" } }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert_eq!(tokens[0].kind, TokenKind::App);
        assert!(matches!(tokens[1].kind, TokenKind::Ident(ref s) if s == "my-site"));
        assert_eq!(tokens[2].kind, TokenKind::LBrace);
    }

    #[test]
    fn test_hyphen_vs_minus() {
        // Ensure hyphen in identifier doesn't break number parsing
        let source = "my-var -10";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();

        assert!(matches!(tokens[0].kind, TokenKind::Ident(ref s) if s == "my-var"));
        assert_eq!(tokens[1].kind, TokenKind::Number(-10.0));
    }
}
