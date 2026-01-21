//! Language definitions for syntax highlighting.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported programming languages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    /// Rust
    Rust,
    /// JavaScript
    JavaScript,
    /// TypeScript
    TypeScript,
    /// Python
    Python,
    /// Go
    Go,
    /// Java
    Java,
    /// C
    C,
    /// C++
    Cpp,
    /// C#
    CSharp,
    /// HTML
    Html,
    /// CSS
    Css,
    /// JSON
    Json,
    /// YAML
    Yaml,
    /// Markdown
    Markdown,
    /// SQL
    Sql,
    /// Shell/Bash
    Shell,
    /// Plain text
    PlainText,
}

impl Language {
    /// Get language from file extension
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "rs" => Some(Language::Rust),
            "js" | "mjs" => Some(Language::JavaScript),
            "ts" | "tsx" => Some(Language::TypeScript),
            "py" => Some(Language::Python),
            "go" => Some(Language::Go),
            "java" => Some(Language::Java),
            "c" | "h" => Some(Language::C),
            "cpp" | "cc" | "cxx" | "hpp" => Some(Language::Cpp),
            "cs" => Some(Language::CSharp),
            "html" | "htm" => Some(Language::Html),
            "css" => Some(Language::Css),
            "json" => Some(Language::Json),
            "yaml" | "yml" => Some(Language::Yaml),
            "md" | "markdown" => Some(Language::Markdown),
            "sql" => Some(Language::Sql),
            "sh" | "bash" | "zsh" => Some(Language::Shell),
            _ => None,
        }
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Language::Rust => "Rust",
            Language::JavaScript => "JavaScript",
            Language::TypeScript => "TypeScript",
            Language::Python => "Python",
            Language::Go => "Go",
            Language::Java => "Java",
            Language::C => "C",
            Language::Cpp => "C++",
            Language::CSharp => "C#",
            Language::Html => "HTML",
            Language::Css => "CSS",
            Language::Json => "JSON",
            Language::Yaml => "YAML",
            Language::Markdown => "Markdown",
            Language::Sql => "SQL",
            Language::Shell => "Shell",
            Language::PlainText => "Plain Text",
        }
    }

    /// Get the single-line comment prefix for this language
    pub fn comment_prefix(&self) -> &'static str {
        match self {
            Language::Rust => "//",
            Language::JavaScript | Language::TypeScript => "//",
            Language::Python => "#",
            Language::Go => "//",
            Language::Java => "//",
            Language::C | Language::Cpp => "//",
            Language::CSharp => "//",
            Language::Html => "<!--",
            Language::Css => "/*",
            Language::Json => "//",
            Language::Yaml => "#",
            Language::Markdown => "<!--",
            Language::Sql => "--",
            Language::Shell => "#",
            Language::PlainText => "//",
        }
    }
}

impl Default for Language {
    fn default() -> Self {
        Language::PlainText
    }
}

/// Language definition with keywords and patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageDefinition {
    /// Language identifier
    pub language: Language,
    /// Keywords
    pub keywords: Vec<String>,
    /// String delimiters
    pub string_delimiters: Vec<String>,
    /// Comment prefix (single line)
    pub comment_prefix: Option<String>,
    /// Block comment delimiters (start, end)
    pub block_comment: Option<(String, String)>,
}

impl LanguageDefinition {
    /// Create a new language definition
    pub fn new(language: Language) -> Self {
        Self {
            language,
            keywords: Vec::new(),
            string_delimiters: vec!["\"".to_string(), "'".to_string()],
            comment_prefix: None,
            block_comment: None,
        }
    }
}

/// Language configuration (alias)
pub type LanguageConfig = LanguageDefinition;

/// Registry of language definitions
#[derive(Debug, Clone, Default)]
pub struct LanguageRegistry {
    /// Registered languages
    languages: HashMap<Language, LanguageDefinition>,
}

impl LanguageRegistry {
    /// Create a new registry
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a language
    pub fn register(&mut self, definition: LanguageDefinition) {
        self.languages.insert(definition.language, definition);
    }

    /// Get language definition
    pub fn get(&self, language: Language) -> Option<&LanguageDefinition> {
        self.languages.get(&language)
    }

    /// List all registered languages
    pub fn languages(&self) -> Vec<Language> {
        self.languages.keys().copied().collect()
    }
}
