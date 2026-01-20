//! OxideKit Text Engine
//!
//! Text shaping and rendering using cosmic-text.

pub use cosmic_text;
pub use fontdb;

mod atlas;
mod renderer;

pub use atlas::{GlyphAtlas, GlyphInfo, GlyphKey};
pub use renderer::TextRenderer;

use cosmic_text::{Attrs, Buffer, FontSystem, Metrics, Shaping, SwashCache};

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
        // Use line height of 1.2x font size as standard
        let line_height = (font_size * 1.2).ceil();
        let metrics = Metrics::new(font_size, line_height);
        let mut buffer = Buffer::new(&mut self.font_system, metrics);

        // Set a wide max width to allow measuring natural width
        buffer.set_size(&mut self.font_system, Some(10000.0), Some(line_height * 10.0));
        buffer.set_text(&mut self.font_system, text, Attrs::new(), Shaping::Advanced);
        buffer.shape_until_scroll(&mut self.font_system, false);

        // Calculate the actual width from layout runs
        let mut max_width: f32 = 0.0;
        let mut total_height: f32 = 0.0;

        for line in buffer.lines.iter() {
            if let Some(layout) = line.layout_opt() {
                for layout_line in layout.iter() {
                    let line_width = layout_line.w;
                    max_width = max_width.max(line_width);
                }
                total_height += line_height;
            }
        }

        // Ensure minimum dimensions
        if max_width < 1.0 {
            max_width = text.len() as f32 * font_size * 0.6; // Rough estimate
        }
        if total_height < 1.0 {
            total_height = line_height;
        }

        (max_width.ceil(), total_height.ceil())
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
}
