//! Syntax highlighting for code blocks.

use serde::{Deserialize, Serialize};

/// Token type for syntax highlighting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TokenType {
    /// Keyword
    Keyword,
    /// String
    String,
    /// Number
    Number,
    /// Comment
    Comment,
    /// Function
    Function,
    /// Type
    Type,
    /// Variable
    Variable,
    /// Operator
    Operator,
    /// Plain text
    Text,
}

/// Token style
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenStyle {
    /// Color
    pub color: String,
    /// Bold
    pub bold: bool,
    /// Italic
    pub italic: bool,
}

impl Default for TokenStyle {
    fn default() -> Self {
        Self {
            color: "#D4D4D4".to_string(),
            bold: false,
            italic: false,
        }
    }
}

/// Highlight theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighlightTheme {
    /// Theme name
    pub name: String,
    /// Background color
    pub background: String,
    /// Default text color
    pub foreground: String,
    /// Token styles
    pub styles: std::collections::HashMap<TokenType, TokenStyle>,
}

impl Default for HighlightTheme {
    fn default() -> Self {
        Self {
            name: "default".to_string(),
            background: "#1E1E1E".to_string(),
            foreground: "#D4D4D4".to_string(),
            styles: std::collections::HashMap::new(),
        }
    }
}

/// Language definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDefinition {
    /// Language name
    pub name: String,
    /// File extensions
    pub extensions: Vec<String>,
    /// Keywords
    pub keywords: Vec<String>,
}

impl LanguageDefinition {
    /// Create a new language
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            extensions: Vec::new(),
            keywords: Vec::new(),
        }
    }
}

/// Language registry
#[derive(Debug, Clone, Default)]
pub struct LanguageRegistry {
    /// Registered languages
    languages: std::collections::HashMap<String, LanguageDefinition>,
}

impl LanguageRegistry {
    /// Create new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a language
    pub fn register(&mut self, lang: LanguageDefinition) {
        self.languages.insert(lang.name.clone(), lang);
    }

    /// Get language by name
    pub fn get(&self, name: &str) -> Option<&LanguageDefinition> {
        self.languages.get(name)
    }
}

/// Syntax highlighter
#[derive(Debug, Clone, Default)]
pub struct SyntaxHighlighter {
    /// Theme
    pub theme: HighlightTheme,
    /// Language registry
    pub languages: LanguageRegistry,
}

impl SyntaxHighlighter {
    /// Create new highlighter
    pub fn new() -> Self {
        Self::default()
    }

    /// Set theme
    pub fn with_theme(mut self, theme: HighlightTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Highlight code
    pub fn highlight(&self, code: &str, _language: &str) -> String {
        // Basic implementation - just escape HTML
        code.replace('&', "&amp;")
            .replace('<', "&lt;")
            .replace('>', "&gt;")
    }
}

/// Code block view
#[derive(Debug, Clone, Default)]
pub struct CodeBlockView {
    /// Code content
    pub code: String,
    /// Language
    pub language: String,
    /// Show line numbers
    pub line_numbers: bool,
    /// Show copy button
    pub copy_button: bool,
    /// Filename
    pub filename: Option<String>,
}

impl CodeBlockView {
    /// Create new code block
    pub fn new() -> Self {
        Self::default()
    }

    /// Set language
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }

    /// Set code
    pub fn code(mut self, code: impl Into<String>) -> Self {
        self.code = code.into();
        self
    }

    /// Show line numbers
    pub fn line_numbers(mut self, show: bool) -> Self {
        self.line_numbers = show;
        self
    }

    /// Show copy button
    pub fn copy_button(mut self, show: bool) -> Self {
        self.copy_button = show;
        self
    }

    /// Set filename
    pub fn filename(mut self, name: impl Into<String>) -> Self {
        self.filename = Some(name.into());
        self
    }
}
