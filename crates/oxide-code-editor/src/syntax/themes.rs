//! Syntax highlighting themes.

use serde::{Deserialize, Serialize};
use super::highlighter::TokenType;

/// Theme variant
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Theme {
    /// Dark theme
    #[default]
    Dark,
    /// Light theme
    Light,
    /// High contrast dark
    HighContrastDark,
    /// High contrast light
    HighContrastLight,
    /// Custom theme
    Custom,
}

impl Theme {
    /// Create dark theme
    pub fn dark() -> Self {
        Self::Dark
    }

    /// Create light theme
    pub fn light() -> Self {
        Self::Light
    }

    /// Get display name
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Dark => "Dark",
            Theme::Light => "Light",
            Theme::HighContrastDark => "High Contrast Dark",
            Theme::HighContrastLight => "High Contrast Light",
            Theme::Custom => "Custom",
        }
    }
}

/// Colors for tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenColors {
    /// Keyword color
    pub keyword: String,
    /// String color
    pub string: String,
    /// Number color
    pub number: String,
    /// Comment color
    pub comment: String,
    /// Function color
    pub function: String,
    /// Type color
    pub type_color: String,
    /// Variable color
    pub variable: String,
    /// Operator color
    pub operator: String,
    /// Default text color
    pub text: String,
}

impl Default for TokenColors {
    fn default() -> Self {
        // Dark theme defaults
        Self {
            keyword: "#569CD6".to_string(),
            string: "#CE9178".to_string(),
            number: "#B5CEA8".to_string(),
            comment: "#6A9955".to_string(),
            function: "#DCDCAA".to_string(),
            type_color: "#4EC9B0".to_string(),
            variable: "#9CDCFE".to_string(),
            operator: "#D4D4D4".to_string(),
            text: "#D4D4D4".to_string(),
        }
    }
}

impl TokenColors {
    /// Get color for token type
    pub fn color_for(&self, token_type: TokenType) -> &str {
        match token_type {
            TokenType::Keyword => &self.keyword,
            TokenType::String => &self.string,
            TokenType::Number => &self.number,
            TokenType::Comment => &self.comment,
            TokenType::Function => &self.function,
            TokenType::Type => &self.type_color,
            TokenType::Variable => &self.variable,
            TokenType::Operator | TokenType::Punctuation => &self.operator,
            _ => &self.text,
        }
    }
}

/// Full theme configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Token colors
    pub tokens: TokenColors,
    /// Background color
    pub background: String,
    /// Foreground color
    pub foreground: String,
    /// Selection color
    pub selection: String,
    /// Line highlight color
    pub line_highlight: String,
    /// Cursor color
    pub cursor: String,
    /// Gutter background
    pub gutter_background: String,
    /// Gutter foreground
    pub gutter_foreground: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            tokens: TokenColors::default(),
            background: "#1E1E1E".to_string(),
            foreground: "#D4D4D4".to_string(),
            selection: "#264F78".to_string(),
            line_highlight: "#2D2D2D".to_string(),
            cursor: "#FFFFFF".to_string(),
            gutter_background: "#1E1E1E".to_string(),
            gutter_foreground: "#858585".to_string(),
        }
    }
}
