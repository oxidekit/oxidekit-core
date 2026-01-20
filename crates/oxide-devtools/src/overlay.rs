//! Layout Debugging Overlays
//!
//! Provides visual overlays for debugging layout, showing bounds,
//! padding, margins, and other spatial information.

use crate::inspector::LayoutInfo;
use crate::tree::{ComponentNode, ComponentTree, NodeHandle};
use serde::{Deserialize, Serialize};

/// Overlay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayConfig {
    /// Show content box (innermost)
    pub show_content: bool,
    /// Show padding box
    pub show_padding: bool,
    /// Show border box
    pub show_border: bool,
    /// Show margin box (outermost)
    pub show_margin: bool,
    /// Show flex/grid lines
    pub show_layout_lines: bool,
    /// Show component labels
    pub show_labels: bool,
    /// Show measurements
    pub show_measurements: bool,
    /// Highlight selected component
    pub highlight_selected: bool,
    /// Highlight hovered component
    pub highlight_hovered: bool,
    /// Color scheme for overlays
    pub colors: OverlayColors,
    /// Opacity for overlay fills
    pub fill_opacity: f32,
    /// Border width for overlay outlines
    pub stroke_width: f32,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            show_content: false,
            show_padding: true,
            show_border: true,
            show_margin: false,
            show_layout_lines: false,
            show_labels: true,
            show_measurements: true,
            highlight_selected: true,
            highlight_hovered: true,
            colors: OverlayColors::default(),
            fill_opacity: 0.2,
            stroke_width: 1.0,
        }
    }
}

/// Colors for overlay visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlayColors {
    /// Content box color
    pub content: String,
    /// Padding color
    pub padding: String,
    /// Border color
    pub border: String,
    /// Margin color
    pub margin: String,
    /// Selection highlight color
    pub selection: String,
    /// Hover highlight color
    pub hover: String,
    /// Measurement line color
    pub measurement: String,
    /// Label background color
    pub label_bg: String,
    /// Label text color
    pub label_text: String,
}

impl Default for OverlayColors {
    fn default() -> Self {
        Self {
            content: "#3B82F6".to_string(),    // Blue
            padding: "#22C55E".to_string(),    // Green
            border: "#F59E0B".to_string(),     // Yellow
            margin: "#EF4444".to_string(),     // Red
            selection: "#8B5CF6".to_string(),  // Purple
            hover: "#06B6D4".to_string(),      // Cyan
            measurement: "#6B7280".to_string(), // Gray
            label_bg: "#1F2937".to_string(),   // Dark gray
            label_text: "#E5E7EB".to_string(), // Light gray
        }
    }
}

/// A drawable overlay element
#[derive(Debug, Clone)]
pub enum OverlayElement {
    /// Rectangle outline
    RectOutline {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
        stroke_width: f32,
    },
    /// Filled rectangle
    RectFill {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: [f32; 4],
    },
    /// Text label
    Label {
        x: f32,
        y: f32,
        text: String,
        bg_color: [f32; 4],
        text_color: [f32; 4],
    },
    /// Measurement line
    MeasureLine {
        x1: f32,
        y1: f32,
        x2: f32,
        y2: f32,
        color: [f32; 4],
        label: String,
    },
    /// Horizontal guide line
    HorizontalGuide {
        y: f32,
        color: [f32; 4],
    },
    /// Vertical guide line
    VerticalGuide {
        x: f32,
        color: [f32; 4],
    },
}

/// Generates overlay elements for rendering
#[derive(Debug)]
pub struct OverlayRenderer {
    /// Configuration
    pub config: OverlayConfig,
    /// Currently selected handle
    selected: Option<NodeHandle>,
    /// Currently hovered handle
    hovered: Option<NodeHandle>,
    /// Generated overlay elements
    elements: Vec<OverlayElement>,
}

impl Default for OverlayRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl OverlayRenderer {
    /// Create a new overlay renderer
    pub fn new() -> Self {
        Self {
            config: OverlayConfig::default(),
            selected: None,
            hovered: None,
            elements: Vec::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: OverlayConfig) -> Self {
        Self {
            config,
            selected: None,
            hovered: None,
            elements: Vec::new(),
        }
    }

    /// Set selected component
    pub fn set_selected(&mut self, handle: Option<NodeHandle>) {
        self.selected = handle;
    }

    /// Set hovered component
    pub fn set_hovered(&mut self, handle: Option<NodeHandle>) {
        self.hovered = handle;
    }

    /// Generate overlays for the entire tree
    pub fn generate_for_tree(&mut self, tree: &ComponentTree) {
        self.elements.clear();

        tree.traverse(|node, _depth| {
            self.generate_for_node(node);
        });

        // Add selection/hover highlights on top
        if let Some(handle) = self.selected {
            if let Some(node) = tree.get(handle) {
                self.generate_selection_overlay(node);
            }
        }

        if let Some(handle) = self.hovered {
            if self.selected != self.hovered {
                if let Some(node) = tree.get(handle) {
                    self.generate_hover_overlay(node);
                }
            }
        }
    }

    /// Generate overlay for a single node
    fn generate_for_node(&mut self, node: &ComponentNode) {
        let layout = &node.layout;

        // Margin box (outermost)
        if self.config.show_margin {
            let (mx, my, mw, mh) = layout.margin_box();
            self.elements.push(OverlayElement::RectFill {
                x: mx,
                y: my,
                width: mw,
                height: mh,
                color: self.hex_to_rgba_with_alpha(&self.config.colors.margin, self.config.fill_opacity),
            });
        }

        // Border box
        if self.config.show_border {
            let (bx, by, bw, bh) = layout.border_box();
            self.elements.push(OverlayElement::RectOutline {
                x: bx,
                y: by,
                width: bw,
                height: bh,
                color: self.hex_to_rgba(&self.config.colors.border),
                stroke_width: self.config.stroke_width,
            });
        }

        // Padding visualization
        if self.config.show_padding {
            let (bx, by, bw, bh) = layout.border_box();
            let padding = &layout.padding;

            // Top padding
            if padding[0] > 0.0 {
                self.elements.push(OverlayElement::RectFill {
                    x: bx,
                    y: by,
                    width: bw,
                    height: padding[0],
                    color: self.hex_to_rgba_with_alpha(&self.config.colors.padding, self.config.fill_opacity),
                });
            }

            // Bottom padding
            if padding[2] > 0.0 {
                self.elements.push(OverlayElement::RectFill {
                    x: bx,
                    y: by + bh - padding[2],
                    width: bw,
                    height: padding[2],
                    color: self.hex_to_rgba_with_alpha(&self.config.colors.padding, self.config.fill_opacity),
                });
            }

            // Left padding
            if padding[3] > 0.0 {
                self.elements.push(OverlayElement::RectFill {
                    x: bx,
                    y: by + padding[0],
                    width: padding[3],
                    height: bh - padding[0] - padding[2],
                    color: self.hex_to_rgba_with_alpha(&self.config.colors.padding, self.config.fill_opacity),
                });
            }

            // Right padding
            if padding[1] > 0.0 {
                self.elements.push(OverlayElement::RectFill {
                    x: bx + bw - padding[1],
                    y: by + padding[0],
                    width: padding[1],
                    height: bh - padding[0] - padding[2],
                    color: self.hex_to_rgba_with_alpha(&self.config.colors.padding, self.config.fill_opacity),
                });
            }
        }

        // Content box (innermost)
        if self.config.show_content {
            let (cx, cy, cw, ch) = layout.content_box();
            self.elements.push(OverlayElement::RectOutline {
                x: cx,
                y: cy,
                width: cw,
                height: ch,
                color: self.hex_to_rgba(&self.config.colors.content),
                stroke_width: self.config.stroke_width,
            });
        }

        // Component label
        if self.config.show_labels {
            let (bx, by, _, _) = layout.border_box();
            self.elements.push(OverlayElement::Label {
                x: bx,
                y: by - 16.0,
                text: node.display_name(),
                bg_color: self.hex_to_rgba(&self.config.colors.label_bg),
                text_color: self.hex_to_rgba(&self.config.colors.label_text),
            });
        }
    }

    /// Generate selection highlight overlay
    fn generate_selection_overlay(&mut self, node: &ComponentNode) {
        if !self.config.highlight_selected {
            return;
        }

        let layout = &node.layout;
        let (bx, by, bw, bh) = layout.border_box();

        // Selection outline (thicker)
        self.elements.push(OverlayElement::RectOutline {
            x: bx - 2.0,
            y: by - 2.0,
            width: bw + 4.0,
            height: bh + 4.0,
            color: self.hex_to_rgba(&self.config.colors.selection),
            stroke_width: 2.0,
        });

        // Measurements if enabled
        if self.config.show_measurements {
            self.generate_measurements(layout);
        }
    }

    /// Generate hover highlight overlay
    fn generate_hover_overlay(&mut self, node: &ComponentNode) {
        if !self.config.highlight_hovered {
            return;
        }

        let layout = &node.layout;
        let (bx, by, bw, bh) = layout.border_box();

        // Hover fill
        self.elements.push(OverlayElement::RectFill {
            x: bx,
            y: by,
            width: bw,
            height: bh,
            color: self.hex_to_rgba_with_alpha(&self.config.colors.hover, 0.1),
        });

        // Hover outline
        self.elements.push(OverlayElement::RectOutline {
            x: bx,
            y: by,
            width: bw,
            height: bh,
            color: self.hex_to_rgba(&self.config.colors.hover),
            stroke_width: 1.0,
        });
    }

    /// Generate measurement lines
    fn generate_measurements(&mut self, layout: &LayoutInfo) {
        let (bx, by, bw, bh) = layout.border_box();
        let color = self.hex_to_rgba(&self.config.colors.measurement);

        // Width measurement
        self.elements.push(OverlayElement::MeasureLine {
            x1: bx,
            y1: by + bh + 8.0,
            x2: bx + bw,
            y2: by + bh + 8.0,
            color,
            label: format!("{:.0}px", bw),
        });

        // Height measurement
        self.elements.push(OverlayElement::MeasureLine {
            x1: bx + bw + 8.0,
            y1: by,
            x2: bx + bw + 8.0,
            y2: by + bh,
            color,
            label: format!("{:.0}px", bh),
        });
    }

    /// Get generated overlay elements
    pub fn elements(&self) -> &[OverlayElement] {
        &self.elements
    }

    /// Clear all overlay elements
    pub fn clear(&mut self) {
        self.elements.clear();
    }

    /// Parse hex color to RGBA array
    fn hex_to_rgba(&self, hex: &str) -> [f32; 4] {
        self.hex_to_rgba_with_alpha(hex, 1.0)
    }

    /// Parse hex color to RGBA array with custom alpha
    fn hex_to_rgba_with_alpha(&self, hex: &str, alpha: f32) -> [f32; 4] {
        let hex = hex.trim_start_matches('#');
        if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
            let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
            let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
            [
                r as f32 / 255.0,
                g as f32 / 255.0,
                b as f32 / 255.0,
                alpha,
            ]
        } else {
            [1.0, 1.0, 1.0, alpha]
        }
    }
}

/// Box model visualizer for a single component
#[derive(Debug, Clone)]
pub struct BoxModelView {
    /// Layout info
    pub layout: LayoutInfo,
    /// Margin values (top, right, bottom, left)
    pub margin: [f32; 4],
    /// Border values
    pub border: [f32; 4],
    /// Padding values
    pub padding: [f32; 4],
    /// Content dimensions (width, height)
    pub content: (f32, f32),
}

impl BoxModelView {
    /// Create from layout info
    pub fn from_layout(layout: &LayoutInfo) -> Self {
        let (_, _, cw, ch) = layout.content_box();
        Self {
            layout: layout.clone(),
            margin: layout.margin,
            border: [layout.border_width; 4],
            padding: layout.padding,
            content: (cw, ch),
        }
    }

    /// Generate ASCII box model representation
    pub fn to_ascii(&self) -> String {
        let mut s = String::new();
        s.push_str("┌─ margin ─────────────────┐\n");
        s.push_str(&format!("│ {:>6} {:>6} {:>6}    │\n",
            format!("{:.0}", self.margin[3]),
            format!("{:.0}", self.margin[0]),
            format!("{:.0}", self.margin[1])
        ));
        s.push_str("│ ┌─ border ────────────┐  │\n");
        s.push_str("│ │ ┌─ padding ──────┐  │  │\n");
        s.push_str(&format!("│ │ │ {:>6} x {:<6} │  │  │\n",
            format!("{:.0}", self.content.0),
            format!("{:.0}", self.content.1)
        ));
        s.push_str("│ │ └─────────────────┘  │  │\n");
        s.push_str("│ └─────────────────────┘  │\n");
        s.push_str(&format!("│        {:>6}           │\n", format!("{:.0}", self.margin[2])));
        s.push_str("└─────────────────────────┘\n");
        s
    }
}

/// Distance measurement between two points
#[derive(Debug, Clone)]
pub struct DistanceMeasurement {
    /// Start point
    pub from: (f32, f32),
    /// End point
    pub to: (f32, f32),
    /// Distance in pixels
    pub distance: f32,
    /// Direction (horizontal, vertical, diagonal)
    pub direction: MeasureDirection,
}

/// Direction of measurement
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasureDirection {
    Horizontal,
    Vertical,
    Diagonal,
}

impl DistanceMeasurement {
    /// Create a new measurement
    pub fn new(from: (f32, f32), to: (f32, f32)) -> Self {
        let dx = to.0 - from.0;
        let dy = to.1 - from.1;
        let distance = (dx * dx + dy * dy).sqrt();

        let direction = if dy.abs() < 0.01 {
            MeasureDirection::Horizontal
        } else if dx.abs() < 0.01 {
            MeasureDirection::Vertical
        } else {
            MeasureDirection::Diagonal
        };

        Self {
            from,
            to,
            distance,
            direction,
        }
    }

    /// Measure horizontal distance
    pub fn horizontal(from: (f32, f32), to: (f32, f32)) -> Self {
        Self {
            from,
            to: (to.0, from.1),
            distance: (to.0 - from.0).abs(),
            direction: MeasureDirection::Horizontal,
        }
    }

    /// Measure vertical distance
    pub fn vertical(from: (f32, f32), to: (f32, f32)) -> Self {
        Self {
            from,
            to: (from.0, to.1),
            distance: (to.1 - from.1).abs(),
            direction: MeasureDirection::Vertical,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_config_default() {
        let config = OverlayConfig::default();
        assert!(config.show_padding);
        assert!(config.highlight_selected);
    }

    #[test]
    fn test_overlay_renderer() {
        let mut renderer = OverlayRenderer::new();
        let tree = ComponentTree::new();

        renderer.generate_for_tree(&tree);
        assert!(renderer.elements().is_empty());
    }

    #[test]
    fn test_hex_to_rgba() {
        let renderer = OverlayRenderer::new();
        let rgba = renderer.hex_to_rgba("#FF0000");
        assert!((rgba[0] - 1.0).abs() < 0.01);
        assert!((rgba[1] - 0.0).abs() < 0.01);
        assert!((rgba[2] - 0.0).abs() < 0.01);
        assert!((rgba[3] - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_box_model_view() {
        let layout = LayoutInfo {
            x: 10.0,
            y: 10.0,
            width: 100.0,
            height: 50.0,
            padding: [5.0, 5.0, 5.0, 5.0],
            margin: [10.0, 10.0, 10.0, 10.0],
            border_width: 1.0,
            ..Default::default()
        };

        let view = BoxModelView::from_layout(&layout);
        assert_eq!(view.margin, [10.0, 10.0, 10.0, 10.0]);

        let ascii = view.to_ascii();
        assert!(ascii.contains("margin"));
        assert!(ascii.contains("padding"));
    }

    #[test]
    fn test_distance_measurement() {
        let m = DistanceMeasurement::new((0.0, 0.0), (3.0, 4.0));
        assert!((m.distance - 5.0).abs() < 0.01);
        assert_eq!(m.direction, MeasureDirection::Diagonal);

        let h = DistanceMeasurement::horizontal((0.0, 10.0), (100.0, 20.0));
        assert_eq!(h.distance, 100.0);
        assert_eq!(h.direction, MeasureDirection::Horizontal);
    }
}
