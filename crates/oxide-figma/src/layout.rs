//! Layout Translation
//!
//! Translates Figma Auto Layout to OxideKit layout constructs:
//! - Stacks (Row, Column)
//! - Grids
//! - Overlays
//! - Scroll containers
//! - Layout constraints (min/max, fill, hug)

use crate::error::{FigmaError, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Layout translator for Figma to OxideKit
#[derive(Debug)]
pub struct LayoutTranslator {
    config: LayoutConfig,
}

/// Configuration for layout translation
#[derive(Debug, Clone)]
pub struct LayoutConfig {
    /// Whether to preserve exact spacing or use tokens
    pub use_spacing_tokens: bool,

    /// Whether to preserve exact sizes or use constraints
    pub use_flexible_sizing: bool,

    /// Spacing token scale (for snapping)
    pub spacing_scale: Vec<f32>,

    /// Default unit for dimensions
    pub default_unit: DimensionUnit,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            use_spacing_tokens: true,
            use_flexible_sizing: true,
            spacing_scale: vec![0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 40.0, 48.0, 64.0],
            default_unit: DimensionUnit::Pixels,
        }
    }
}

/// Dimension unit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DimensionUnit {
    Pixels,
    Percent,
    Em,
    Rem,
}

/// Translated OxideKit layout
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OxideLayout {
    /// Layout type
    pub layout_type: OxideLayoutType,

    /// Layout direction (for stacks)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<Direction>,

    /// Gap between children
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gap: Option<Dimension>,

    /// Padding
    #[serde(skip_serializing_if = "Option::is_none")]
    pub padding: Option<Padding>,

    /// Width constraint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<SizeConstraint>,

    /// Height constraint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<SizeConstraint>,

    /// Min width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_width: Option<Dimension>,

    /// Max width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_width: Option<Dimension>,

    /// Min height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_height: Option<Dimension>,

    /// Max height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_height: Option<Dimension>,

    /// Main axis alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub justify: Option<Alignment>,

    /// Cross axis alignment
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align: Option<Alignment>,

    /// Self alignment (within parent)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub align_self: Option<Alignment>,

    /// Flex grow
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flex_grow: Option<f32>,

    /// Corner radius
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<Radius>,

    /// Overflow behavior
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overflow: Option<Overflow>,

    /// Position type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position: Option<Position>,

    /// Children layouts (for nested translation)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub children: Vec<OxideLayout>,

    /// Original Figma node ID (for reference)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub figma_node_id: Option<String>,
}

/// OxideKit layout types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OxideLayoutType {
    /// Row (horizontal stack)
    Row,
    /// Column (vertical stack)
    Column,
    /// Stack (for overlays)
    Stack,
    /// Grid layout
    Grid,
    /// Scroll container
    Scroll,
    /// Absolute positioned container
    Absolute,
    /// Single element (no layout)
    Box,
}

/// Direction for stacks
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Horizontal,
    Vertical,
}

/// Dimension value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dimension {
    pub value: f32,
    pub unit: DimensionUnit,
    /// Spacing token reference (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl Dimension {
    pub fn px(value: f32) -> Self {
        Self {
            value,
            unit: DimensionUnit::Pixels,
            token: None,
        }
    }

    pub fn percent(value: f32) -> Self {
        Self {
            value,
            unit: DimensionUnit::Percent,
            token: None,
        }
    }

    pub fn token(name: &str, value: f32) -> Self {
        Self {
            value,
            unit: DimensionUnit::Pixels,
            token: Some(name.to_string()),
        }
    }
}

/// Padding values
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    pub top: Dimension,
    pub right: Dimension,
    pub bottom: Dimension,
    pub left: Dimension,
}

impl Padding {
    pub fn uniform(value: f32) -> Self {
        let dim = Dimension::px(value);
        Self {
            top: dim.clone(),
            right: dim.clone(),
            bottom: dim.clone(),
            left: dim,
        }
    }

    pub fn symmetric(vertical: f32, horizontal: f32) -> Self {
        Self {
            top: Dimension::px(vertical),
            right: Dimension::px(horizontal),
            bottom: Dimension::px(vertical),
            left: Dimension::px(horizontal),
        }
    }

    pub fn from_values(top: f32, right: f32, bottom: f32, left: f32) -> Self {
        Self {
            top: Dimension::px(top),
            right: Dimension::px(right),
            bottom: Dimension::px(bottom),
            left: Dimension::px(left),
        }
    }
}

/// Size constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeConstraint {
    /// Fixed size
    Fixed(Dimension),
    /// Fill available space
    Fill,
    /// Hug content
    Hug,
    /// Percentage of parent
    Percent(f32),
    /// Fractional (like fr in CSS Grid)
    Fraction(f32),
}

/// Alignment options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Alignment {
    Start,
    Center,
    End,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
    Stretch,
    Baseline,
}

/// Corner radius
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Radius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_right: f32,
    pub bottom_left: f32,
    /// Radius token reference (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub token: Option<String>,
}

impl Radius {
    pub fn uniform(value: f32) -> Self {
        Self {
            top_left: value,
            top_right: value,
            bottom_right: value,
            bottom_left: value,
            token: None,
        }
    }

    pub fn from_values(top_left: f32, top_right: f32, bottom_right: f32, bottom_left: f32) -> Self {
        Self {
            top_left,
            top_right,
            bottom_right,
            bottom_left,
            token: None,
        }
    }
}

/// Overflow behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Overflow {
    Visible,
    Hidden,
    Scroll,
    Auto,
}

/// Position type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub position_type: PositionType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bottom: Option<Dimension>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left: Option<Dimension>,
}

/// Position type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PositionType {
    Relative,
    Absolute,
    Fixed,
}

impl LayoutTranslator {
    /// Create a new layout translator
    pub fn new() -> Self {
        Self::with_config(LayoutConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: LayoutConfig) -> Self {
        Self { config }
    }

    /// Translate a Figma node to OxideKit layout
    pub fn translate(&self, node: &Node) -> Result<OxideLayout> {
        debug!(node_name = %node.name, node_type = ?node.node_type, "Translating layout");

        let layout_type = self.determine_layout_type(node);
        let direction = self.get_direction(node);

        let mut layout = OxideLayout {
            layout_type,
            direction,
            gap: self.translate_gap(node),
            padding: self.translate_padding(node),
            width: self.translate_width_constraint(node),
            height: self.translate_height_constraint(node),
            min_width: node.min_width.map(Dimension::px),
            max_width: node.max_width.map(Dimension::px),
            min_height: node.min_height.map(Dimension::px),
            max_height: node.max_height.map(Dimension::px),
            justify: self.translate_main_axis_alignment(node),
            align: self.translate_cross_axis_alignment(node),
            align_self: self.translate_self_alignment(node),
            flex_grow: self.translate_flex_grow(node),
            radius: self.translate_radius(node),
            overflow: self.translate_overflow(node),
            position: self.translate_position(node),
            children: Vec::new(),
            figma_node_id: Some(node.id.clone()),
        };

        // Recursively translate children
        for child in &node.children {
            if child.visible {
                let child_layout = self.translate(child)?;
                layout.children.push(child_layout);
            }
        }

        Ok(layout)
    }

    /// Translate a full Figma file
    pub fn translate_file(&self, file: &FigmaFile) -> Result<Vec<OxideLayout>> {
        info!(file_name = %file.name, "Translating Figma file layouts");

        let mut layouts = Vec::new();

        for page in &file.document.children {
            for frame in &page.children {
                if frame.visible {
                    let layout = self.translate(frame)?;
                    layouts.push(layout);
                }
            }
        }

        info!(total_layouts = layouts.len(), "Layout translation complete");

        Ok(layouts)
    }

    /// Determine OxideKit layout type from Figma node
    fn determine_layout_type(&self, node: &Node) -> OxideLayoutType {
        // Check for auto-layout
        if let Some(mode) = &node.layout_mode {
            match mode {
                LayoutMode::Horizontal => return OxideLayoutType::Row,
                LayoutMode::Vertical => return OxideLayoutType::Column,
                LayoutMode::None => {}
            }
        }

        // Check for absolute positioning
        if let Some(constraints) = &node.constraints {
            // If constraints suggest absolute positioning
            let is_absolute = matches!(
                (&constraints.horizontal, &constraints.vertical),
                (ConstraintType::Left, ConstraintType::Top)
                    | (ConstraintType::Right, ConstraintType::Top)
                    | (ConstraintType::Left, ConstraintType::Bottom)
                    | (ConstraintType::Right, ConstraintType::Bottom)
            ) && node.layout_mode.is_none();

            if is_absolute {
                return OxideLayoutType::Absolute;
            }
        }

        // Check for overlapping children (stack/overlay)
        if self.has_overlapping_children(node) {
            return OxideLayoutType::Stack;
        }

        // Default to Box
        OxideLayoutType::Box
    }

    /// Check if node has overlapping children
    fn has_overlapping_children(&self, node: &Node) -> bool {
        let children: Vec<_> = node.children.iter()
            .filter(|c| c.visible && c.absolute_bounding_box.is_some())
            .collect();

        for i in 0..children.len() {
            for j in (i + 1)..children.len() {
                let a = children[i].absolute_bounding_box.as_ref().unwrap();
                let b = children[j].absolute_bounding_box.as_ref().unwrap();

                // Check for overlap
                if a.x < b.x + b.width
                    && a.x + a.width > b.x
                    && a.y < b.y + b.height
                    && a.y + a.height > b.y
                {
                    return true;
                }
            }
        }

        false
    }

    /// Get direction from layout mode
    fn get_direction(&self, node: &Node) -> Option<Direction> {
        node.layout_mode.as_ref().and_then(|mode| match mode {
            LayoutMode::Horizontal => Some(Direction::Horizontal),
            LayoutMode::Vertical => Some(Direction::Vertical),
            LayoutMode::None => None,
        })
    }

    /// Translate gap (item spacing)
    fn translate_gap(&self, node: &Node) -> Option<Dimension> {
        if node.item_spacing > 0.0 {
            if self.config.use_spacing_tokens {
                let (token, value) = self.snap_to_spacing_token(node.item_spacing);
                Some(Dimension::token(&token, value))
            } else {
                Some(Dimension::px(node.item_spacing))
            }
        } else {
            None
        }
    }

    /// Translate padding
    fn translate_padding(&self, node: &Node) -> Option<Padding> {
        let has_padding = node.padding_top > 0.0
            || node.padding_right > 0.0
            || node.padding_bottom > 0.0
            || node.padding_left > 0.0;

        if has_padding {
            Some(Padding::from_values(
                node.padding_top,
                node.padding_right,
                node.padding_bottom,
                node.padding_left,
            ))
        } else {
            None
        }
    }

    /// Translate width constraint
    fn translate_width_constraint(&self, node: &Node) -> Option<SizeConstraint> {
        if let Some(mode) = &node.primary_axis_sizing_mode {
            if let Some(layout_mode) = &node.layout_mode {
                if *layout_mode == LayoutMode::Horizontal {
                    return self.translate_axis_sizing(mode, node.absolute_bounding_box.as_ref());
                }
            }
        }

        if let Some(mode) = &node.counter_axis_sizing_mode {
            if let Some(layout_mode) = &node.layout_mode {
                if *layout_mode == LayoutMode::Vertical {
                    return self.translate_axis_sizing(mode, node.absolute_bounding_box.as_ref());
                }
            }
        }

        // Fallback to fixed size from bounding box
        node.absolute_bounding_box
            .as_ref()
            .map(|bbox| SizeConstraint::Fixed(Dimension::px(bbox.width)))
    }

    /// Translate height constraint
    fn translate_height_constraint(&self, node: &Node) -> Option<SizeConstraint> {
        if let Some(mode) = &node.primary_axis_sizing_mode {
            if let Some(layout_mode) = &node.layout_mode {
                if *layout_mode == LayoutMode::Vertical {
                    return self.translate_axis_sizing(mode, node.absolute_bounding_box.as_ref());
                }
            }
        }

        if let Some(mode) = &node.counter_axis_sizing_mode {
            if let Some(layout_mode) = &node.layout_mode {
                if *layout_mode == LayoutMode::Horizontal {
                    return self.translate_axis_sizing(mode, node.absolute_bounding_box.as_ref());
                }
            }
        }

        // Fallback to fixed size
        node.absolute_bounding_box
            .as_ref()
            .map(|bbox| SizeConstraint::Fixed(Dimension::px(bbox.height)))
    }

    /// Translate axis sizing mode
    fn translate_axis_sizing(
        &self,
        mode: &AxisSizingMode,
        bbox: Option<&Rectangle>,
    ) -> Option<SizeConstraint> {
        match mode {
            AxisSizingMode::Auto => Some(SizeConstraint::Hug),
            AxisSizingMode::Fixed => bbox.map(|b| SizeConstraint::Fixed(Dimension::px(b.width))),
        }
    }

    /// Translate main axis alignment
    fn translate_main_axis_alignment(&self, node: &Node) -> Option<Alignment> {
        node.primary_axis_align_items.as_ref().map(|align| match align {
            AlignItems::Min => Alignment::Start,
            AlignItems::Center => Alignment::Center,
            AlignItems::Max => Alignment::End,
            AlignItems::SpaceBetween => Alignment::SpaceBetween,
            AlignItems::Baseline => Alignment::Baseline,
        })
    }

    /// Translate cross axis alignment
    fn translate_cross_axis_alignment(&self, node: &Node) -> Option<Alignment> {
        node.counter_axis_align_items.as_ref().map(|align| match align {
            AlignItems::Min => Alignment::Start,
            AlignItems::Center => Alignment::Center,
            AlignItems::Max => Alignment::End,
            AlignItems::SpaceBetween => Alignment::SpaceBetween,
            AlignItems::Baseline => Alignment::Baseline,
        })
    }

    /// Translate self alignment
    fn translate_self_alignment(&self, node: &Node) -> Option<Alignment> {
        node.layout_align.as_ref().map(|align| match align {
            LayoutAlign::Inherit => Alignment::Start,
            LayoutAlign::Stretch => Alignment::Stretch,
            LayoutAlign::Min => Alignment::Start,
            LayoutAlign::Center => Alignment::Center,
            LayoutAlign::Max => Alignment::End,
        })
    }

    /// Translate flex grow
    fn translate_flex_grow(&self, node: &Node) -> Option<f32> {
        if node.layout_grow > 0.0 {
            Some(node.layout_grow)
        } else if let Some(LayoutAlign::Stretch) = &node.layout_align {
            Some(1.0)
        } else {
            None
        }
    }

    /// Translate corner radius
    fn translate_radius(&self, node: &Node) -> Option<Radius> {
        if let Some(radii) = &node.rectangle_corner_radii {
            if radii.iter().any(|r| *r > 0.0) {
                return Some(Radius::from_values(radii[0], radii[1], radii[2], radii[3]));
            }
        }

        if node.corner_radius > 0.0 {
            return Some(Radius::uniform(node.corner_radius));
        }

        None
    }

    /// Translate overflow
    fn translate_overflow(&self, node: &Node) -> Option<Overflow> {
        // Figma doesn't have explicit overflow, but we can infer from clips
        // For now, default to visible for most cases
        None
    }

    /// Translate position
    fn translate_position(&self, node: &Node) -> Option<Position> {
        // Check if this node is absolutely positioned
        if let Some(constraints) = &node.constraints {
            match (&constraints.horizontal, &constraints.vertical) {
                (ConstraintType::Scale, ConstraintType::Scale) => None,
                _ => {
                    // May need absolute positioning
                    if node.layout_mode.is_none() && !node.children.is_empty() {
                        return Some(Position {
                            position_type: PositionType::Relative,
                            top: None,
                            right: None,
                            bottom: None,
                            left: None,
                        });
                    }
                    None
                }
            }
        } else {
            None
        }
    }

    /// Snap value to nearest spacing token
    fn snap_to_spacing_token(&self, value: f32) -> (String, f32) {
        let snapped = self.config.spacing_scale
            .iter()
            .min_by(|a, b| {
                ((**a - value).abs())
                    .partial_cmp(&((**b - value).abs()))
                    .unwrap()
            })
            .copied()
            .unwrap_or(value);

        let token_name = match snapped as u32 {
            0 => "spacing.none",
            1..=4 => "spacing.xs",
            5..=8 => "spacing.sm",
            9..=16 => "spacing.md",
            17..=24 => "spacing.lg",
            25..=32 => "spacing.xl",
            _ => "spacing.xxl",
        };

        (token_name.to_string(), snapped)
    }

    /// Generate OUI markup from layout
    pub fn to_oui(&self, layout: &OxideLayout) -> String {
        let mut output = String::new();
        self.write_oui_node(layout, &mut output, 0);
        output
    }

    /// Write OUI node
    fn write_oui_node(&self, layout: &OxideLayout, output: &mut String, indent: usize) {
        let indent_str = "  ".repeat(indent);

        // Determine element name
        let element = match layout.layout_type {
            OxideLayoutType::Row => "Row",
            OxideLayoutType::Column => "Column",
            OxideLayoutType::Stack => "Stack",
            OxideLayoutType::Grid => "Grid",
            OxideLayoutType::Scroll => "ScrollView",
            OxideLayoutType::Absolute => "Absolute",
            OxideLayoutType::Box => "Box",
        };

        // Start element
        output.push_str(&format!("{}<{}", indent_str, element));

        // Add attributes
        if let Some(gap) = &layout.gap {
            if let Some(token) = &gap.token {
                output.push_str(&format!(" gap=\"{{{}}}\"", token));
            } else {
                output.push_str(&format!(" gap=\"{}\"", gap.value));
            }
        }

        if let Some(justify) = &layout.justify {
            output.push_str(&format!(" justify=\"{:?}\"", justify).to_lowercase());
        }

        if let Some(align) = &layout.align {
            output.push_str(&format!(" align=\"{:?}\"", align).to_lowercase());
        }

        if let Some(padding) = &layout.padding {
            // Simplify if uniform
            if padding.top.value == padding.right.value
                && padding.right.value == padding.bottom.value
                && padding.bottom.value == padding.left.value
            {
                output.push_str(&format!(" padding=\"{}\"", padding.top.value));
            } else if padding.top.value == padding.bottom.value
                && padding.left.value == padding.right.value
            {
                output.push_str(&format!(
                    " padding=\"{} {}\"",
                    padding.top.value, padding.left.value
                ));
            } else {
                output.push_str(&format!(
                    " padding=\"{} {} {} {}\"",
                    padding.top.value,
                    padding.right.value,
                    padding.bottom.value,
                    padding.left.value
                ));
            }
        }

        if let Some(radius) = &layout.radius {
            if radius.top_left == radius.top_right
                && radius.top_right == radius.bottom_right
                && radius.bottom_right == radius.bottom_left
            {
                output.push_str(&format!(" radius=\"{}\"", radius.top_left));
            }
        }

        if let Some(flex_grow) = &layout.flex_grow {
            if *flex_grow > 0.0 {
                output.push_str(&format!(" flex=\"{}\"", flex_grow));
            }
        }

        // Handle children
        if layout.children.is_empty() {
            output.push_str(" />\n");
        } else {
            output.push_str(">\n");

            for child in &layout.children {
                self.write_oui_node(child, output, indent + 1);
            }

            output.push_str(&format!("{}</{}>\n", indent_str, element));
        }
    }
}

impl Default for LayoutTranslator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OxideLayout {
    fn default() -> Self {
        Self {
            layout_type: OxideLayoutType::Box,
            direction: None,
            gap: None,
            padding: None,
            width: None,
            height: None,
            min_width: None,
            max_width: None,
            min_height: None,
            max_height: None,
            justify: None,
            align: None,
            align_self: None,
            flex_grow: None,
            radius: None,
            overflow: None,
            position: None,
            children: Vec::new(),
            figma_node_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dimension_px() {
        let dim = Dimension::px(16.0);
        assert_eq!(dim.value, 16.0);
        assert_eq!(dim.unit, DimensionUnit::Pixels);
    }

    #[test]
    fn test_dimension_token() {
        let dim = Dimension::token("spacing.md", 16.0);
        assert_eq!(dim.token, Some("spacing.md".to_string()));
    }

    #[test]
    fn test_padding_uniform() {
        let padding = Padding::uniform(16.0);
        assert_eq!(padding.top.value, 16.0);
        assert_eq!(padding.right.value, 16.0);
        assert_eq!(padding.bottom.value, 16.0);
        assert_eq!(padding.left.value, 16.0);
    }

    #[test]
    fn test_radius_uniform() {
        let radius = Radius::uniform(8.0);
        assert_eq!(radius.top_left, 8.0);
        assert_eq!(radius.bottom_right, 8.0);
    }

    #[test]
    fn test_snap_to_spacing_token() {
        let translator = LayoutTranslator::new();
        let (token, value) = translator.snap_to_spacing_token(15.0);
        assert_eq!(value, 16.0);
        assert!(token.contains("spacing"));
    }
}
