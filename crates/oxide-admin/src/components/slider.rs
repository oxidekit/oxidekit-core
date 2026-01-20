//! Slider component
//!
//! Range slider for selecting numeric values within a range.
//! Supports single value and range selection, marks, and tooltips.

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius,
    hex_to_rgba, InteractionState,
};

// =============================================================================
// SLIDER SIZE
// =============================================================================

/// Slider size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SliderSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl SliderSize {
    /// Track height
    pub fn track_height(&self) -> f32 {
        match self {
            Self::Small => 4.0,
            Self::Medium => 6.0,
            Self::Large => 8.0,
        }
    }

    /// Thumb size
    pub fn thumb_size(&self) -> f32 {
        match self {
            Self::Small => 14.0,
            Self::Medium => 18.0,
            Self::Large => 22.0,
        }
    }

    /// Mark size
    pub fn mark_size(&self) -> f32 {
        match self {
            Self::Small => 4.0,
            Self::Medium => 6.0,
            Self::Large => 8.0,
        }
    }
}

// =============================================================================
// SLIDER MARK
// =============================================================================

/// Slider mark for showing specific values
#[derive(Debug, Clone)]
pub struct SliderMark {
    /// Value at this mark
    pub value: f64,
    /// Optional label
    pub label: Option<String>,
}

impl SliderMark {
    /// Create a new mark
    pub fn new(value: f64) -> Self {
        Self { value, label: None }
    }

    /// Create a labeled mark
    pub fn labeled(value: f64, label: impl Into<String>) -> Self {
        Self {
            value,
            label: Some(label.into()),
        }
    }
}

// =============================================================================
// SLIDER PROPERTIES
// =============================================================================

/// Slider properties
#[derive(Debug, Clone)]
pub struct SliderProps {
    /// Current value (0.0 to 1.0 normalized, or absolute if min/max set)
    pub value: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step increment (0 = continuous)
    pub step: f64,
    /// Slider size
    pub size: SliderSize,
    /// Whether slider is disabled
    pub disabled: bool,
    /// Show value tooltip
    pub show_tooltip: bool,
    /// Marks to display
    pub marks: Vec<SliderMark>,
    /// Interaction state
    pub state: InteractionState,
    /// Orientation
    pub orientation: SliderOrientation,
}

/// Slider orientation
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum SliderOrientation {
    #[default]
    Horizontal,
    Vertical,
}

impl Default for SliderProps {
    fn default() -> Self {
        Self {
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            size: SliderSize::Medium,
            disabled: false,
            show_tooltip: true,
            marks: Vec::new(),
            state: InteractionState::Default,
            orientation: SliderOrientation::Horizontal,
        }
    }
}

impl SliderProps {
    /// Create a new slider
    pub fn new() -> Self {
        Self::default()
    }

    /// Set current value
    pub fn value(mut self, value: f64) -> Self {
        self.value = value.clamp(self.min, self.max);
        self
    }

    /// Set range
    pub fn range(mut self, min: f64, max: f64) -> Self {
        self.min = min;
        self.max = max;
        self
    }

    /// Set step
    pub fn step(mut self, step: f64) -> Self {
        self.step = step;
        self
    }

    /// Set size
    pub fn size(mut self, size: SliderSize) -> Self {
        self.size = size;
        self
    }

    /// Set disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set marks
    pub fn marks(mut self, marks: Vec<SliderMark>) -> Self {
        self.marks = marks;
        self
    }

    /// Add marks at regular intervals
    pub fn auto_marks(mut self, count: usize) -> Self {
        let step = (self.max - self.min) / count as f64;
        self.marks = (0..=count)
            .map(|i| SliderMark::new(self.min + step * i as f64))
            .collect();
        self
    }

    /// Set state
    pub fn state(mut self, state: InteractionState) -> Self {
        self.state = state;
        self
    }

    /// Get normalized value (0.0 to 1.0)
    pub fn normalized_value(&self) -> f64 {
        if self.max <= self.min {
            0.0
        } else {
            (self.value - self.min) / (self.max - self.min)
        }
    }
}

// =============================================================================
// SLIDER COMPONENT
// =============================================================================

/// Slider component
pub struct Slider;

impl Slider {
    /// Build a slider
    pub fn build(tree: &mut LayoutTree, props: SliderProps) -> NodeId {
        match props.orientation {
            SliderOrientation::Horizontal => Self::build_horizontal(tree, &props),
            SliderOrientation::Vertical => Self::build_vertical(tree, &props),
        }
    }

    /// Build horizontal slider
    fn build_horizontal(tree: &mut LayoutTree, props: &SliderProps) -> NodeId {
        let track_height = props.size.track_height();
        let thumb_size = props.size.thumb_size();

        // Container (includes space for thumb overflow)
        let container_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(spacing::SM)
            .build();

        let mut children = Vec::new();

        // Track container
        let track_container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(thumb_size)
            .build();

        // Track background
        let track_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(track_height)
            .build();

        let track_visual = NodeVisual::default()
            .with_background(hex_to_rgba(if props.disabled {
                colors::DISABLED_BG
            } else {
                colors::SURFACE_VARIANT
            }))
            .with_radius(track_height / 2.0);

        // Filled track (progress)
        let filled_width = (props.normalized_value() * 100.0) as f32;
        let filled_style = StyleBuilder::new()
            .width_percent(filled_width / 100.0)
            .height_percent(1.0)
            .build();

        let effective_state = if props.disabled { InteractionState::Disabled } else { props.state };

        let filled_color = if props.disabled {
            colors::DISABLED_BG
        } else {
            match effective_state {
                InteractionState::Hover | InteractionState::Active => colors::PRIMARY_LIGHT,
                _ => colors::PRIMARY,
            }
        };

        let filled_visual = NodeVisual::default()
            .with_background(hex_to_rgba(filled_color))
            .with_radius(track_height / 2.0);

        let filled = tree.new_visual_node(filled_style, filled_visual);

        let track = tree.new_visual_node_with_children(track_style, track_visual, &[filled]);

        // Thumb
        let thumb = Self::build_thumb(tree, props);

        let track_container = tree.new_node_with_children(track_container_style, &[track, thumb]);
        children.push(track_container);

        // Marks
        if !props.marks.is_empty() {
            let marks = Self::build_marks(tree, props);
            children.push(marks);
        }

        tree.new_node_with_children(container_style, &children)
    }

    /// Build vertical slider
    fn build_vertical(tree: &mut LayoutTree, props: &SliderProps) -> NodeId {
        let track_height = props.size.track_height();
        let thumb_size = props.size.thumb_size();

        // Container
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .height_percent(1.0)
            .gap(spacing::SM)
            .build();

        let mut children = Vec::new();

        // Track container
        let track_container_style = StyleBuilder::new()
            .flex_column()
            .align_center()
            .height_percent(1.0)
            .width(thumb_size)
            .build();

        // Track background
        let track_style = StyleBuilder::new()
            .flex_column()
            .align_center()
            .height_percent(1.0)
            .width(track_height)
            .build();

        let track_visual = NodeVisual::default()
            .with_background(hex_to_rgba(if props.disabled {
                colors::DISABLED_BG
            } else {
                colors::SURFACE_VARIANT
            }))
            .with_radius(track_height / 2.0);

        let track = tree.new_visual_node(track_style, track_visual);

        // Thumb
        let thumb = Self::build_thumb(tree, props);

        let track_container = tree.new_node_with_children(track_container_style, &[track, thumb]);
        children.push(track_container);

        tree.new_node_with_children(container_style, &children)
    }

    /// Build slider thumb
    fn build_thumb(tree: &mut LayoutTree, props: &SliderProps) -> NodeId {
        let thumb_size = props.size.thumb_size();

        let thumb_style = StyleBuilder::new()
            .size(thumb_size, thumb_size)
            .build();

        let effective_state = if props.disabled { InteractionState::Disabled } else { props.state };

        let thumb_color = if props.disabled {
            colors::DISABLED_BG
        } else {
            colors::PRIMARY_CONTRAST
        };

        let border_color = if props.disabled {
            colors::BORDER
        } else {
            match effective_state {
                InteractionState::Hover => colors::PRIMARY_LIGHT,
                InteractionState::Active => colors::PRIMARY_DARK,
                InteractionState::Focus => colors::BORDER_FOCUS,
                _ => colors::PRIMARY,
            }
        };

        let mut thumb_visual = NodeVisual::default()
            .with_background(hex_to_rgba(thumb_color))
            .with_border(hex_to_rgba(border_color), 2.0)
            .with_radius(thumb_size / 2.0);

        // Focus ring
        if effective_state == InteractionState::Focus {
            // Could add shadow for focus ring effect
        }

        tree.new_visual_node(thumb_style, thumb_visual)
    }

    /// Build marks
    fn build_marks(tree: &mut LayoutTree, props: &SliderProps) -> NodeId {
        let mark_size = props.size.mark_size();

        let marks_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .justify_between()
            .width_percent(1.0)
            .build();

        let mark_nodes: Vec<NodeId> = props.marks.iter()
            .map(|mark| {
                let mark_container_style = StyleBuilder::new()
                    .flex_column()
                    .align_center()
                    .gap(spacing::XS)
                    .build();

                // Mark dot
                let dot_style = StyleBuilder::new()
                    .size(mark_size, mark_size)
                    .build();

                let normalized = if props.max > props.min {
                    (mark.value - props.min) / (props.max - props.min)
                } else {
                    0.0
                };

                let dot_color = if normalized <= props.normalized_value() {
                    colors::PRIMARY
                } else {
                    colors::SURFACE_VARIANT
                };

                let dot_visual = NodeVisual::default()
                    .with_background(hex_to_rgba(dot_color))
                    .with_radius(mark_size / 2.0);

                let dot = tree.new_visual_node(dot_style, dot_visual);

                if mark.label.is_some() {
                    let label_style = StyleBuilder::new().build();
                    let label = tree.new_node(label_style);
                    tree.new_node_with_children(mark_container_style, &[dot, label])
                } else {
                    tree.new_node_with_children(mark_container_style, &[dot])
                }
            })
            .collect();

        tree.new_node_with_children(marks_style, &mark_nodes)
    }
}

// =============================================================================
// RANGE SLIDER COMPONENT
// =============================================================================

/// Range slider properties (for selecting a range)
#[derive(Debug, Clone)]
pub struct RangeSliderProps {
    /// Lower value
    pub low: f64,
    /// Upper value
    pub high: f64,
    /// Minimum value
    pub min: f64,
    /// Maximum value
    pub max: f64,
    /// Step increment
    pub step: f64,
    /// Slider size
    pub size: SliderSize,
    /// Whether slider is disabled
    pub disabled: bool,
    /// Minimum gap between handles
    pub min_gap: f64,
}

impl Default for RangeSliderProps {
    fn default() -> Self {
        Self {
            low: 25.0,
            high: 75.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            size: SliderSize::Medium,
            disabled: false,
            min_gap: 0.0,
        }
    }
}

/// Range slider component
pub struct RangeSlider;

impl RangeSlider {
    /// Build a range slider
    pub fn build(tree: &mut LayoutTree, props: RangeSliderProps) -> NodeId {
        let track_height = props.size.track_height();
        let thumb_size = props.size.thumb_size();

        let container_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(thumb_size)
            .build();

        // Track
        let track_style = StyleBuilder::new()
            .flex_row()
            .align_center()
            .width_percent(1.0)
            .height(track_height)
            .build();

        let track_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::SURFACE_VARIANT))
            .with_radius(track_height / 2.0);

        // Filled range
        let range = props.max - props.min;
        let low_pct = if range > 0.0 { (props.low - props.min) / range } else { 0.0 };
        let high_pct = if range > 0.0 { (props.high - props.min) / range } else { 1.0 };

        let range_style = StyleBuilder::new()
            .width_percent((high_pct - low_pct) as f32)
            .height_percent(1.0)
            .build();

        let range_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::PRIMARY))
            .with_radius(track_height / 2.0);

        let range_fill = tree.new_visual_node(range_style, range_visual);

        let track = tree.new_visual_node_with_children(track_style, track_visual, &[range_fill]);

        // Low thumb
        let low_thumb_style = StyleBuilder::new()
            .size(thumb_size, thumb_size)
            .build();
        let low_thumb_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::PRIMARY_CONTRAST))
            .with_border(hex_to_rgba(colors::PRIMARY), 2.0)
            .with_radius(thumb_size / 2.0);
        let low_thumb = tree.new_visual_node(low_thumb_style, low_thumb_visual);

        // High thumb
        let high_thumb_style = StyleBuilder::new()
            .size(thumb_size, thumb_size)
            .build();
        let high_thumb_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::PRIMARY_CONTRAST))
            .with_border(hex_to_rgba(colors::PRIMARY), 2.0)
            .with_radius(thumb_size / 2.0);
        let high_thumb = tree.new_visual_node(high_thumb_style, high_thumb_visual);

        tree.new_node_with_children(container_style, &[track, low_thumb, high_thumb])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slider_size() {
        assert_eq!(SliderSize::Small.thumb_size(), 14.0);
        assert_eq!(SliderSize::Medium.thumb_size(), 18.0);
        assert_eq!(SliderSize::Large.thumb_size(), 22.0);
    }

    #[test]
    fn test_slider_props() {
        let props = SliderProps::new()
            .range(0.0, 100.0)
            .value(50.0)
            .step(5.0);

        assert_eq!(props.min, 0.0);
        assert_eq!(props.max, 100.0);
        assert_eq!(props.value, 50.0);
        assert_eq!(props.normalized_value(), 0.5);
    }

    #[test]
    fn test_slider_build() {
        let mut tree = LayoutTree::new();
        let props = SliderProps::new().value(50.0);
        let _node = Slider::build(&mut tree, props);
    }

    #[test]
    fn test_range_slider_build() {
        let mut tree = LayoutTree::new();
        let props = RangeSliderProps {
            low: 25.0,
            high: 75.0,
            ..Default::default()
        };
        let _node = RangeSlider::build(&mut tree, props);
    }

    #[test]
    fn test_slider_marks() {
        let mut tree = LayoutTree::new();
        let props = SliderProps::new()
            .range(0.0, 100.0)
            .value(50.0)
            .auto_marks(4);

        assert_eq!(props.marks.len(), 5); // 0, 25, 50, 75, 100
        let _node = Slider::build(&mut tree, props);
    }
}
