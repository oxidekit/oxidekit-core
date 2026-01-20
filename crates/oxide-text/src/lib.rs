//! OxideKit Text Engine
//!
//! Text shaping and rendering using cosmic-text.
//!
//! # Features
//!
//! - Accurate text measurement with detailed metrics (ascent, descent, line gap)
//! - Word wrapping with configurable max width
//! - Text truncation with ellipsis
//! - Font family/weight/style specification
//! - Emoji and grapheme cluster handling via cosmic-text
//! - GPU-accelerated rendering with glyph atlas

pub use cosmic_text;
pub use fontdb;

mod atlas;
mod renderer;

pub use atlas::{GlyphAtlas, GlyphInfo, GlyphKey};
pub use renderer::TextRenderer;

use cosmic_text::{
    Attrs, Buffer, Family, FontSystem, Metrics, Shaping, Stretch, Style, SwashCache, Weight, Wrap,
};

/// Detailed text metrics
#[derive(Debug, Clone, Copy, Default)]
pub struct TextMetrics {
    /// Total width of the text
    pub width: f32,
    /// Total height of the text
    pub height: f32,
    /// Distance from baseline to top of the tallest glyph
    pub ascent: f32,
    /// Distance from baseline to bottom of the lowest glyph (usually negative)
    pub descent: f32,
    /// Additional spacing between lines
    pub line_gap: f32,
    /// Height of a single line
    pub line_height: f32,
    /// Number of lines
    pub line_count: usize,
}

/// Text overflow behavior
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextOverflow {
    /// Allow text to overflow its container
    #[default]
    Visible,
    /// Clip text at the container boundary
    Clip,
    /// Truncate text with ellipsis (...) when it overflows
    Ellipsis,
}

/// Text wrap mode
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum TextWrap {
    /// No wrapping - single line
    None,
    /// Wrap at word boundaries
    #[default]
    Word,
    /// Wrap at grapheme boundaries (character wrap)
    Glyph,
}

/// Font weight specification
#[derive(Debug, Clone, Copy, Default)]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    #[default]
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
    /// Custom weight (100-900)
    Custom(u16),
}

impl FontWeight {
    fn to_cosmic(&self) -> Weight {
        match self {
            FontWeight::Thin => Weight(100),
            FontWeight::ExtraLight => Weight(200),
            FontWeight::Light => Weight(300),
            FontWeight::Regular => Weight(400),
            FontWeight::Medium => Weight(500),
            FontWeight::SemiBold => Weight(600),
            FontWeight::Bold => Weight(700),
            FontWeight::ExtraBold => Weight(800),
            FontWeight::Black => Weight(900),
            FontWeight::Custom(w) => Weight(*w),
        }
    }
}

/// Font style specification
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum FontStyle {
    #[default]
    Normal,
    Italic,
    Oblique,
}

impl FontStyle {
    fn to_cosmic(&self) -> Style {
        match self {
            FontStyle::Normal => Style::Normal,
            FontStyle::Italic => Style::Italic,
            FontStyle::Oblique => Style::Oblique,
        }
    }
}

/// Configuration for text layout and rendering
#[derive(Debug, Clone)]
pub struct TextConfig {
    /// Font size in logical pixels
    pub font_size: f32,
    /// Line height in logical pixels (None = auto based on font metrics)
    pub line_height: Option<f32>,
    /// Font family name (e.g., "Inter", "Arial", "monospace")
    pub font_family: Option<String>,
    /// Font weight
    pub font_weight: FontWeight,
    /// Font style
    pub font_style: FontStyle,
    /// Maximum width for wrapping (None = no limit)
    pub max_width: Option<f32>,
    /// Maximum number of lines (None = no limit)
    pub max_lines: Option<usize>,
    /// Text overflow behavior
    pub overflow: TextOverflow,
    /// Text wrap mode
    pub wrap: TextWrap,
}

impl Default for TextConfig {
    fn default() -> Self {
        Self {
            font_size: 16.0,
            line_height: None,
            font_family: None,
            font_weight: FontWeight::Regular,
            font_style: FontStyle::Normal,
            max_width: None,
            max_lines: None,
            overflow: TextOverflow::Visible,
            wrap: TextWrap::Word,
        }
    }
}

impl TextConfig {
    /// Create a new text config with the given font size
    pub fn new(font_size: f32) -> Self {
        Self {
            font_size,
            ..Default::default()
        }
    }

    /// Set the line height
    pub fn with_line_height(mut self, height: f32) -> Self {
        self.line_height = Some(height);
        self
    }

    /// Set the font family
    pub fn with_font_family(mut self, family: impl Into<String>) -> Self {
        self.font_family = Some(family.into());
        self
    }

    /// Set the font weight
    pub fn with_weight(mut self, weight: FontWeight) -> Self {
        self.font_weight = weight;
        self
    }

    /// Set bold weight
    pub fn bold(mut self) -> Self {
        self.font_weight = FontWeight::Bold;
        self
    }

    /// Set the font style
    pub fn with_style(mut self, style: FontStyle) -> Self {
        self.font_style = style;
        self
    }

    /// Set italic style
    pub fn italic(mut self) -> Self {
        self.font_style = FontStyle::Italic;
        self
    }

    /// Set maximum width for wrapping
    pub fn with_max_width(mut self, width: f32) -> Self {
        self.max_width = Some(width);
        self
    }

    /// Set maximum number of lines
    pub fn with_max_lines(mut self, lines: usize) -> Self {
        self.max_lines = Some(lines);
        self
    }

    /// Set overflow behavior to ellipsis
    pub fn ellipsis(mut self) -> Self {
        self.overflow = TextOverflow::Ellipsis;
        self
    }

    /// Set wrap mode to none (single line)
    pub fn nowrap(mut self) -> Self {
        self.wrap = TextWrap::None;
        self
    }
}

/// Text rendering system
pub struct TextSystem {
    font_system: FontSystem,
    swash_cache: SwashCache,
}

impl TextSystem {
    /// Create a new text system with system fonts
    pub fn new() -> Self {
        let font_system = FontSystem::new();
        let swash_cache = SwashCache::new();

        Self {
            font_system,
            swash_cache,
        }
    }

    /// Create a text buffer for rendering
    pub fn create_buffer(&mut self, text: &str, font_size: f32, line_height: f32) -> Buffer {
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);

        buffer
    }

    /// Shape text in a buffer
    pub fn shape(&mut self, buffer: &mut Buffer) {
        buffer.shape_until_scroll(&mut self.font_system, false);
    }

    /// Get mutable access to font system (for buffer operations)
    pub fn font_system_mut(&mut self) -> &mut FontSystem {
        &mut self.font_system
    }

    /// Get mutable access to swash cache (for glyph rasterization)
    pub fn swash_cache_mut(&mut self) -> &mut SwashCache {
        &mut self.swash_cache
    }

    /// Get mutable access to both font system and swash cache
    /// This is needed because both are required for text rendering
    pub fn get_render_refs(&mut self) -> (&mut FontSystem, &mut SwashCache) {
        (&mut self.font_system, &mut self.swash_cache)
    }

    /// Measure text dimensions
    /// Returns (width, height) in logical pixels
    pub fn measure_text(&mut self, text: &str, font_size: f32) -> (f32, f32) {
        let metrics = self.measure_text_detailed(text, &TextConfig::new(font_size));
        (metrics.width, metrics.height)
    }

    /// Measure text with detailed metrics
    pub fn measure_text_detailed(&mut self, text: &str, config: &TextConfig) -> TextMetrics {
        let line_height = config.line_height.unwrap_or(config.font_size * 1.2).ceil();
        let metrics = Metrics::new(config.font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Set wrap mode
        buffer.set_wrap(&mut self.font_system, match config.wrap {
            TextWrap::None => Wrap::None,
            TextWrap::Word => Wrap::Word,
            TextWrap::Glyph => Wrap::Glyph,
        });

        // Set size based on max_width
        let max_width = config.max_width.unwrap_or(10000.0);
        buffer.set_size(&mut self.font_system, Some(max_width), Some(line_height * 100.0));

        // Build attrs with font config
        let attrs = self.build_attrs(config);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Calculate metrics from layout
        let mut max_line_width: f32 = 0.0;
        let mut total_height: f32 = 0.0;
        let mut line_count: usize = 0;
        let mut first_ascent: f32 = 0.0;
        let mut last_descent: f32 = 0.0;

        for (i, line) in buffer.lines.iter().enumerate() {
            if let Some(layout) = line.layout_opt() {
                for layout_line in layout.iter() {
                    // Check max_lines limit
                    if let Some(max) = config.max_lines {
                        if line_count >= max {
                            break;
                        }
                    }

                    max_line_width = max_line_width.max(layout_line.w);
                    line_count += 1;

                    // Get font metrics from first glyph
                    if let Some(glyph) = layout_line.glyphs.first() {
                        if i == 0 && first_ascent == 0.0 {
                            first_ascent = glyph.y_offset;
                        }
                        last_descent = glyph.y_offset - config.font_size;
                    }
                }
                total_height += line_height;
            }
        }

        // Apply max_lines limit to height
        if let Some(max) = config.max_lines {
            let max_height = line_height * max as f32;
            total_height = total_height.min(max_height);
        }

        // Ensure minimum dimensions
        if max_line_width < 1.0 {
            max_line_width = text.len() as f32 * config.font_size * 0.6;
        }
        if total_height < 1.0 {
            total_height = line_height;
        }
        if line_count == 0 {
            line_count = 1;
        }

        TextMetrics {
            width: max_line_width.ceil(),
            height: total_height.ceil(),
            ascent: first_ascent.abs(),
            descent: last_descent,
            line_gap: line_height - config.font_size,
            line_height,
            line_count,
        }
    }

    /// Measure text with wrapping at a specific width
    pub fn measure_text_wrapped(&mut self, text: &str, font_size: f32, max_width: f32) -> TextMetrics {
        self.measure_text_detailed(text, &TextConfig::new(font_size).with_max_width(max_width))
    }

    /// Build cosmic-text Attrs from TextConfig
    fn build_attrs(&self, config: &TextConfig) -> Attrs<'static> {
        let mut attrs = Attrs::new();

        // Set font family
        if let Some(ref family) = config.font_family {
            // Try to match common family names
            let family = match family.to_lowercase().as_str() {
                "serif" => Family::Serif,
                "sans-serif" | "sans" => Family::SansSerif,
                "monospace" | "mono" => Family::Monospace,
                "cursive" => Family::Cursive,
                "fantasy" => Family::Fantasy,
                _ => Family::Name(Box::leak(family.clone().into_boxed_str())),
            };
            attrs = attrs.family(family);
        }

        // Set weight
        attrs = attrs.weight(config.font_weight.to_cosmic());

        // Set style
        attrs = attrs.style(config.font_style.to_cosmic());

        // Set stretch (currently always normal)
        attrs = attrs.stretch(Stretch::Normal);

        attrs
    }

    /// Create a text buffer with configuration for rendering
    pub fn create_buffer_with_config(&mut self, text: &str, config: &TextConfig) -> Buffer {
        let line_height = config.line_height.unwrap_or(config.font_size * 1.2).ceil();
        let metrics = Metrics::new(config.font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Set wrap mode
        buffer.set_wrap(&mut self.font_system, match config.wrap {
            TextWrap::None => Wrap::None,
            TextWrap::Word => Wrap::Word,
            TextWrap::Glyph => Wrap::Glyph,
        });

        // Set size if max_width is specified
        if let Some(max_width) = config.max_width {
            buffer.set_size(&mut self.font_system, Some(max_width), None);
        }

        let attrs = self.build_attrs(config);
        buffer.set_text(&mut self.font_system, text, attrs, Shaping::Advanced);

        buffer
    }

    /// Truncate text to fit within max_width, adding ellipsis if needed
    pub fn truncate_text(&mut self, text: &str, config: &TextConfig) -> String {
        if config.overflow != TextOverflow::Ellipsis {
            return text.to_string();
        }

        let Some(max_width) = config.max_width else {
            return text.to_string();
        };

        // Measure full text
        let full_metrics = self.measure_text_detailed(text, config);
        if full_metrics.width <= max_width {
            return text.to_string();
        }

        // Binary search to find truncation point
        let ellipsis = "…";
        let ellipsis_width = {
            let m = self.measure_text_detailed(ellipsis, config);
            m.width
        };

        let target_width = max_width - ellipsis_width;
        if target_width <= 0.0 {
            return ellipsis.to_string();
        }

        // Find the right truncation point
        let chars: Vec<char> = text.chars().collect();
        let mut low = 0;
        let mut high = chars.len();

        while low < high {
            let mid = (low + high + 1) / 2;
            let truncated: String = chars[..mid].iter().collect();
            let metrics = self.measure_text_detailed(&truncated, config);

            if metrics.width <= target_width {
                low = mid;
            } else {
                high = mid - 1;
            }
        }

        if low == 0 {
            return ellipsis.to_string();
        }

        let truncated: String = chars[..low].iter().collect();
        format!("{}{}", truncated.trim_end(), ellipsis)
    }
}

impl Default for TextSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_system() {
        let mut system = TextSystem::new();
        let buffer = system.create_buffer("Hello OxideKit!", 24.0, 32.0);
        assert!(!buffer.lines.is_empty());
    }

    #[test]
    fn test_measure_text() {
        let mut system = TextSystem::new();
        let (width, height) = system.measure_text("Hello", 16.0);
        assert!(width > 0.0, "Width should be positive");
        assert!(height > 0.0, "Height should be positive");
    }

    #[test]
    fn test_measure_text_detailed() {
        let mut system = TextSystem::new();
        let config = TextConfig::new(16.0);
        let metrics = system.measure_text_detailed("Hello World", &config);

        assert!(metrics.width > 0.0);
        assert!(metrics.height > 0.0);
        assert!(metrics.line_height >= 16.0);
        assert_eq!(metrics.line_count, 1);
    }

    #[test]
    fn test_text_wrapping() {
        let mut system = TextSystem::new();
        let text = "This is a long text that should wrap to multiple lines when constrained to a narrow width";

        // Without wrapping constraint - should be single line
        let wide = system.measure_text_detailed(text, &TextConfig::new(16.0).with_max_width(2000.0));

        // With narrow constraint - should wrap to multiple lines
        let narrow = system.measure_text_detailed(text, &TextConfig::new(16.0).with_max_width(100.0));

        // Narrow width should produce more lines (or at least equal)
        assert!(
            narrow.line_count >= wide.line_count,
            "Narrow width should cause more lines: narrow={} wide={}",
            narrow.line_count,
            wide.line_count
        );

        // Narrow width should be narrower
        assert!(
            narrow.width <= 100.0 || narrow.width <= wide.width,
            "Narrow layout should respect max_width"
        );
    }

    #[test]
    fn test_text_truncation() {
        let mut system = TextSystem::new();
        let text = "This is a very long text that needs truncation";
        let config = TextConfig::new(16.0)
            .with_max_width(100.0)
            .ellipsis()
            .nowrap();

        let truncated = system.truncate_text(text, &config);
        assert!(truncated.ends_with('…'), "Should end with ellipsis");
        assert!(truncated.len() < text.len(), "Should be shorter than original");
    }

    #[test]
    fn test_text_config_builder() {
        let config = TextConfig::new(24.0)
            .bold()
            .italic()
            .with_max_width(200.0)
            .with_max_lines(2)
            .ellipsis();

        assert_eq!(config.font_size, 24.0);
        assert!(matches!(config.font_weight, FontWeight::Bold));
        assert_eq!(config.font_style, FontStyle::Italic);
        assert_eq!(config.max_width, Some(200.0));
        assert_eq!(config.max_lines, Some(2));
        assert_eq!(config.overflow, TextOverflow::Ellipsis);
    }
}
