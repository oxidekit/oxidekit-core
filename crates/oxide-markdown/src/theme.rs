//! Markdown themes.

use serde::{Deserialize, Serialize};

/// Markdown theme preset
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MarkdownTheme {
    /// GitHub-like theme
    #[default]
    GitHub,
    /// Dark theme
    Dark,
    /// Light theme
    Light,
    /// Custom theme
    Custom,
}

impl MarkdownTheme {
    /// Get theme name
    pub fn name(&self) -> &'static str {
        match self {
            MarkdownTheme::GitHub => "GitHub",
            MarkdownTheme::Dark => "Dark",
            MarkdownTheme::Light => "Light",
            MarkdownTheme::Custom => "Custom",
        }
    }
}

/// Theme colors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    /// Background color
    pub background: String,
    /// Text color
    pub text: String,
    /// Link color
    pub link: String,
    /// Code background
    pub code_background: String,
    /// Code text
    pub code_text: String,
    /// Blockquote border
    pub blockquote_border: String,
    /// Heading color
    pub heading: String,
    /// Border color
    pub border: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        // GitHub-like defaults
        Self {
            background: "#ffffff".to_string(),
            text: "#24292e".to_string(),
            link: "#0366d6".to_string(),
            code_background: "#f6f8fa".to_string(),
            code_text: "#24292e".to_string(),
            blockquote_border: "#dfe2e5".to_string(),
            heading: "#24292e".to_string(),
            border: "#e1e4e8".to_string(),
        }
    }
}

/// Typography tokens
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyTokens {
    /// Base font size (px)
    pub base_size: f32,
    /// Line height
    pub line_height: f32,
    /// Heading scale factor
    pub heading_scale: f32,
    /// Code font family
    pub code_font: String,
    /// Body font family
    pub body_font: String,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            base_size: 16.0,
            line_height: 1.6,
            heading_scale: 1.25,
            code_font: "monospace".to_string(),
            body_font: "sans-serif".to_string(),
        }
    }
}

/// Full theme configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Theme preset
    pub preset: MarkdownTheme,
    /// Colors
    pub colors: ThemeColors,
    /// Typography
    pub typography: TypographyTokens,
}

impl ThemeConfig {
    /// Create from preset
    pub fn from_preset(preset: MarkdownTheme) -> Self {
        let colors = match preset {
            MarkdownTheme::Dark => ThemeColors {
                background: "#0d1117".to_string(),
                text: "#c9d1d9".to_string(),
                link: "#58a6ff".to_string(),
                code_background: "#161b22".to_string(),
                code_text: "#c9d1d9".to_string(),
                blockquote_border: "#30363d".to_string(),
                heading: "#c9d1d9".to_string(),
                border: "#30363d".to_string(),
            },
            _ => ThemeColors::default(),
        };

        Self {
            preset,
            colors,
            typography: TypographyTokens::default(),
        }
    }
}
