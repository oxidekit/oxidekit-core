//! Right-to-left (RTL) language support
//!
//! Provides utilities for handling RTL languages like Arabic, Hebrew, Persian, etc.

use serde::{Deserialize, Serialize};

/// Text direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    /// Left-to-right (default for most languages)
    #[default]
    Ltr,
    /// Right-to-left (Arabic, Hebrew, etc.)
    Rtl,
}

impl Direction {
    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ltr" | "left-to-right" | "l" => Some(Self::Ltr),
            "rtl" | "right-to-left" | "r" => Some(Self::Rtl),
            _ => None,
        }
    }

    /// Get the CSS direction value
    pub fn css_value(&self) -> &'static str {
        match self {
            Self::Ltr => "ltr",
            Self::Rtl => "rtl",
        }
    }

    /// Check if this is RTL
    pub fn is_rtl(&self) -> bool {
        matches!(self, Self::Rtl)
    }
}

impl std::fmt::Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.css_value())
    }
}

/// RTL support utilities
#[derive(Debug, Clone, Default)]
pub struct RtlSupport {
    /// Current text direction
    direction: Direction,
}

impl RtlSupport {
    /// Create a new RTL support instance
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }

    /// Create for a language code
    pub fn for_language(language: &str) -> Self {
        Self::new(Self::language_direction(language))
    }

    /// Get the direction for a language code
    pub fn language_direction(language: &str) -> Direction {
        let lang = language.split(|c| c == '-' || c == '_').next().unwrap_or(language);

        match lang.to_lowercase().as_str() {
            // RTL languages
            "ar" | // Arabic
            "he" | // Hebrew
            "fa" | // Persian/Farsi
            "ur" | // Urdu
            "yi" | // Yiddish
            "ps" | // Pashto
            "sd" | // Sindhi
            "ug" | // Uyghur
            "ku" | // Kurdish (some dialects)
            "dv" | // Divehi
            "ha" | // Hausa (Arabic script)
            "ks" | // Kashmiri
            "pa" | // Punjabi (Shahmukhi)
            "syr"  // Syriac
            => Direction::Rtl,

            // Everything else is LTR
            _ => Direction::Ltr,
        }
    }

    /// Get the current direction
    pub fn direction(&self) -> Direction {
        self.direction
    }

    /// Set the direction
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }

    /// Check if current direction is RTL
    pub fn is_rtl(&self) -> bool {
        self.direction.is_rtl()
    }

    /// Wrap text with appropriate Unicode directional markers
    pub fn wrap_text(&self, text: &str) -> String {
        if self.direction.is_rtl() {
            // Right-to-Left Embedding + text + Pop Directional Formatting
            format!("\u{202B}{}\u{202C}", text)
        } else {
            text.to_string()
        }
    }

    /// Wrap text to force a specific direction
    pub fn force_direction(&self, text: &str, direction: Direction) -> String {
        match direction {
            Direction::Ltr => format!("\u{202A}{}\u{202C}", text), // LRE + text + PDF
            Direction::Rtl => format!("\u{202B}{}\u{202C}", text), // RLE + text + PDF
        }
    }

    /// Get bidirectional algorithm hint for mixed content
    pub fn bidi_isolate(&self, text: &str) -> String {
        // First Strong Isolate + text + Pop Directional Isolate
        format!("\u{2068}{}\u{2069}", text)
    }

    /// Mirror a value for RTL (e.g., padding-left becomes padding-right)
    pub fn mirror_property<'a>(&self, property: &'a str) -> &'a str {
        if !self.direction.is_rtl() {
            return property;
        }

        match property {
            "left" => "right",
            "right" => "left",
            "padding-left" => "padding-right",
            "padding-right" => "padding-left",
            "margin-left" => "margin-right",
            "margin-right" => "margin-left",
            "border-left" => "border-right",
            "border-right" => "border-left",
            "text-align: left" => "text-align: right",
            "text-align: right" => "text-align: left",
            "float: left" => "float: right",
            "float: right" => "float: left",
            _ => property,
        }
    }

    /// Get logical property name (start/end instead of left/right)
    pub fn to_logical_property(&self, physical: &str) -> String {
        match physical {
            "left" => {
                if self.direction.is_rtl() {
                    "end".to_string()
                } else {
                    "start".to_string()
                }
            }
            "right" => {
                if self.direction.is_rtl() {
                    "start".to_string()
                } else {
                    "end".to_string()
                }
            }
            _ => physical.to_string(),
        }
    }

    /// Generate RTL-aware CSS properties
    pub fn rtl_css(&self) -> RtlCss {
        RtlCss::new(self.direction)
    }
}

/// RTL-aware CSS property generator
#[derive(Debug, Clone)]
pub struct RtlCss {
    direction: Direction,
}

impl RtlCss {
    /// Create a new RTL CSS generator
    pub fn new(direction: Direction) -> Self {
        Self { direction }
    }

    /// Get inline-start (left for LTR, right for RTL)
    pub fn inline_start(&self) -> &'static str {
        if self.direction.is_rtl() {
            "right"
        } else {
            "left"
        }
    }

    /// Get inline-end (right for LTR, left for RTL)
    pub fn inline_end(&self) -> &'static str {
        if self.direction.is_rtl() {
            "left"
        } else {
            "right"
        }
    }

    /// Get margin-inline-start property name
    pub fn margin_start(&self) -> &'static str {
        if self.direction.is_rtl() {
            "margin-right"
        } else {
            "margin-left"
        }
    }

    /// Get margin-inline-end property name
    pub fn margin_end(&self) -> &'static str {
        if self.direction.is_rtl() {
            "margin-left"
        } else {
            "margin-right"
        }
    }

    /// Get padding-inline-start property name
    pub fn padding_start(&self) -> &'static str {
        if self.direction.is_rtl() {
            "padding-right"
        } else {
            "padding-left"
        }
    }

    /// Get padding-inline-end property name
    pub fn padding_end(&self) -> &'static str {
        if self.direction.is_rtl() {
            "padding-left"
        } else {
            "padding-right"
        }
    }

    /// Generate direction CSS declaration
    pub fn direction_declaration(&self) -> String {
        format!("direction: {};", self.direction.css_value())
    }

    /// Generate transform for horizontal flip
    pub fn transform_flip(&self) -> Option<&'static str> {
        if self.direction.is_rtl() {
            Some("transform: scaleX(-1);")
        } else {
            None
        }
    }
}

/// RTL context for components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RtlContext {
    /// Text direction
    pub direction: Direction,

    /// Writing mode (horizontal-tb, vertical-rl, etc.)
    pub writing_mode: WritingMode,
}

impl Default for RtlContext {
    fn default() -> Self {
        Self {
            direction: Direction::Ltr,
            writing_mode: WritingMode::HorizontalTb,
        }
    }
}

/// CSS writing-mode values
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WritingMode {
    /// Horizontal, top to bottom (most languages)
    #[default]
    HorizontalTb,
    /// Vertical, right to left (traditional Chinese, Japanese)
    VerticalRl,
    /// Vertical, left to right
    VerticalLr,
}

impl WritingMode {
    /// Get the CSS value
    pub fn css_value(&self) -> &'static str {
        match self {
            Self::HorizontalTb => "horizontal-tb",
            Self::VerticalRl => "vertical-rl",
            Self::VerticalLr => "vertical-lr",
        }
    }
}

impl std::fmt::Display for WritingMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.css_value())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_direction() {
        assert_eq!(RtlSupport::language_direction("ar"), Direction::Rtl);
        assert_eq!(RtlSupport::language_direction("ar-SA"), Direction::Rtl);
        assert_eq!(RtlSupport::language_direction("he"), Direction::Rtl);
        assert_eq!(RtlSupport::language_direction("fa"), Direction::Rtl);
        assert_eq!(RtlSupport::language_direction("en"), Direction::Ltr);
        assert_eq!(RtlSupport::language_direction("en-US"), Direction::Ltr);
        assert_eq!(RtlSupport::language_direction("zh"), Direction::Ltr);
    }

    #[test]
    fn test_mirror_property() {
        let rtl = RtlSupport::new(Direction::Rtl);
        assert_eq!(rtl.mirror_property("left"), "right");
        assert_eq!(rtl.mirror_property("padding-left"), "padding-right");

        let ltr = RtlSupport::new(Direction::Ltr);
        assert_eq!(ltr.mirror_property("left"), "left");
    }

    #[test]
    fn test_rtl_css() {
        let rtl = RtlCss::new(Direction::Rtl);
        assert_eq!(rtl.inline_start(), "right");
        assert_eq!(rtl.inline_end(), "left");

        let ltr = RtlCss::new(Direction::Ltr);
        assert_eq!(ltr.inline_start(), "left");
        assert_eq!(ltr.inline_end(), "right");
    }

    #[test]
    fn test_wrap_text() {
        let rtl = RtlSupport::new(Direction::Rtl);
        let wrapped = rtl.wrap_text("مرحبا");
        assert!(wrapped.starts_with('\u{202B}'));
        assert!(wrapped.ends_with('\u{202C}'));
    }
}
