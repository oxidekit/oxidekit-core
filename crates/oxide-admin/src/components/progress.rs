//! Progress indicator components
//!
//! Linear and circular progress indicators for showing loading states
//! and operation progress.

use oxide_layout::{NodeId, LayoutTree, StyleBuilder, NodeVisual};

use super::tokens::{
    colors, spacing, radius,
    hex_to_rgba,
};

// =============================================================================
// PROGRESS SIZE
// =============================================================================

/// Progress indicator size
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ProgressSize {
    Small,
    #[default]
    Medium,
    Large,
}

impl ProgressSize {
    /// Linear progress height
    pub fn height(&self) -> f32 {
        match self {
            Self::Small => 4.0,
            Self::Medium => 8.0,
            Self::Large => 12.0,
        }
    }

    /// Circular progress size
    pub fn circular_size(&self) -> f32 {
        match self {
            Self::Small => 24.0,
            Self::Medium => 40.0,
            Self::Large => 64.0,
        }
    }

    /// Circular progress stroke width
    pub fn stroke_width(&self) -> f32 {
        match self {
            Self::Small => 3.0,
            Self::Medium => 4.0,
            Self::Large => 6.0,
        }
    }
}

// =============================================================================
// PROGRESS VARIANT
// =============================================================================

/// Progress color variant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum ProgressVariant {
    #[default]
    Primary,
    Success,
    Warning,
    Danger,
    Info,
}

impl ProgressVariant {
    /// Get color for this variant
    fn color(&self) -> &'static str {
        match self {
            Self::Primary => colors::PRIMARY,
            Self::Success => colors::SUCCESS,
            Self::Warning => colors::WARNING,
            Self::Danger => colors::DANGER,
            Self::Info => colors::INFO,
        }
    }

    /// Get background color for this variant (lighter)
    fn bg_color(&self) -> &'static str {
        match self {
            Self::Primary => colors::SURFACE_VARIANT,
            Self::Success => colors::SUCCESS_BG,
            Self::Warning => colors::WARNING_BG,
            Self::Danger => colors::DANGER_BG,
            Self::Info => colors::INFO_BG,
        }
    }
}

// =============================================================================
// LINEAR PROGRESS
// =============================================================================

/// Linear progress properties
#[derive(Debug, Clone)]
pub struct LinearProgressProps {
    /// Current value (0.0 to 1.0 for determinate, None for indeterminate)
    pub value: Option<f64>,
    /// Size
    pub size: ProgressSize,
    /// Color variant
    pub variant: ProgressVariant,
    /// Show value label
    pub show_label: bool,
    /// Label format (e.g., "{value}%" or "Loading...")
    pub label_format: Option<String>,
    /// Striped animation
    pub striped: bool,
    /// Animated (for striped or indeterminate)
    pub animated: bool,
}

impl Default for LinearProgressProps {
    fn default() -> Self {
        Self {
            value: Some(0.0),
            size: ProgressSize::Medium,
            variant: ProgressVariant::Primary,
            show_label: false,
            label_format: None,
            striped: false,
            animated: false,
        }
    }
}

impl LinearProgressProps {
    /// Create a determinate progress bar
    pub fn new(value: f64) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            ..Default::default()
        }
    }

    /// Create an indeterminate progress bar
    pub fn indeterminate() -> Self {
        Self {
            value: None,
            animated: true,
            ..Default::default()
        }
    }

    /// Set value
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value.clamp(0.0, 1.0));
        self
    }

    /// Set size
    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }

    /// Set variant
    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show label
    pub fn with_label(mut self) -> Self {
        self.show_label = true;
        self
    }

    /// Set custom label format
    pub fn label_format(mut self, format: impl Into<String>) -> Self {
        self.label_format = Some(format.into());
        self.show_label = true;
        self
    }

    /// Enable striped style
    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    /// Enable animation
    pub fn animated(mut self, animated: bool) -> Self {
        self.animated = animated;
        self
    }
}

/// Linear progress component
pub struct LinearProgress;

impl LinearProgress {
    /// Build a linear progress bar
    pub fn build(tree: &mut LayoutTree, props: LinearProgressProps) -> NodeId {
        let height = props.size.height();

        let container_style = StyleBuilder::new()
            .flex_column()
            .width_percent(1.0)
            .gap(spacing::XS)
            .build();

        let mut children = Vec::new();

        // Label row (if showing)
        if props.show_label {
            let label_style = StyleBuilder::new()
                .flex_row()
                .align_center()
                .justify_between()
                .width_percent(1.0)
                .build();

            // Label text placeholder
            let text_style = StyleBuilder::new().build();
            let text = tree.new_node(text_style);

            // Value text
            let value_style = StyleBuilder::new().build();
            let value = tree.new_node(value_style);

            let label = tree.new_node_with_children(label_style, &[text, value]);
            children.push(label);
        }

        // Track
        let track_style = StyleBuilder::new()
            .flex_row()
            .width_percent(1.0)
            .height(height)
            .build();

        let track_visual = NodeVisual::default()
            .with_background(hex_to_rgba(props.variant.bg_color()))
            .with_radius(height / 2.0);

        // Progress fill
        let fill_width = props.value.map(|v| v as f32).unwrap_or(0.3); // 30% for indeterminate

        let fill_style = StyleBuilder::new()
            .width_percent(fill_width)
            .height_percent(1.0)
            .build();

        let fill_visual = NodeVisual::default()
            .with_background(hex_to_rgba(props.variant.color()))
            .with_radius(height / 2.0);

        let fill = tree.new_visual_node(fill_style, fill_visual);

        let track = tree.new_visual_node_with_children(track_style, track_visual, &[fill]);
        children.push(track);

        tree.new_node_with_children(container_style, &children)
    }
}

// =============================================================================
// CIRCULAR PROGRESS
// =============================================================================

/// Circular progress properties
#[derive(Debug, Clone)]
pub struct CircularProgressProps {
    /// Current value (0.0 to 1.0 for determinate, None for indeterminate)
    pub value: Option<f64>,
    /// Size
    pub size: ProgressSize,
    /// Color variant
    pub variant: ProgressVariant,
    /// Show value in center
    pub show_value: bool,
    /// Custom label in center
    pub label: Option<String>,
}

impl Default for CircularProgressProps {
    fn default() -> Self {
        Self {
            value: None,
            size: ProgressSize::Medium,
            variant: ProgressVariant::Primary,
            show_value: false,
            label: None,
        }
    }
}

impl CircularProgressProps {
    /// Create a determinate circular progress
    pub fn new(value: f64) -> Self {
        Self {
            value: Some(value.clamp(0.0, 1.0)),
            ..Default::default()
        }
    }

    /// Create an indeterminate circular progress (spinner)
    pub fn indeterminate() -> Self {
        Self::default()
    }

    /// Set value
    pub fn value(mut self, value: f64) -> Self {
        self.value = Some(value.clamp(0.0, 1.0));
        self
    }

    /// Set size
    pub fn size(mut self, size: ProgressSize) -> Self {
        self.size = size;
        self
    }

    /// Set variant
    pub fn variant(mut self, variant: ProgressVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Show value in center
    pub fn with_value(mut self) -> Self {
        self.show_value = true;
        self
    }

    /// Set custom center label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }
}

/// Circular progress component
pub struct CircularProgress;

impl CircularProgress {
    /// Build a circular progress indicator
    pub fn build(tree: &mut LayoutTree, props: CircularProgressProps) -> NodeId {
        let size = props.size.circular_size();
        let stroke_width = props.size.stroke_width();

        let container_style = StyleBuilder::new()
            .size(size, size)
            .flex_row()
            .center()
            .build();

        // Outer ring (background)
        let outer_style = StyleBuilder::new()
            .size(size, size)
            .flex_row()
            .center()
            .build();

        let outer_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::TRANSPARENT))
            .with_border(hex_to_rgba(props.variant.bg_color()), stroke_width)
            .with_radius(size / 2.0);

        // For a proper circular progress, we would need arc rendering
        // This is a simplified visual representation

        // Inner circle (cutout for ring effect)
        let inner_size = size - (stroke_width * 2.0);
        let inner_style = StyleBuilder::new()
            .size(inner_size, inner_size)
            .flex_row()
            .center()
            .build();

        let inner_visual = NodeVisual::default()
            .with_background(hex_to_rgba(colors::BACKGROUND))
            .with_radius(inner_size / 2.0);

        let mut inner_children = Vec::new();

        // Center content
        if props.show_value || props.label.is_some() {
            let content_style = StyleBuilder::new()
                .flex_row()
                .center()
                .build();
            let content = tree.new_node(content_style);
            inner_children.push(content);
        }

        let inner = if inner_children.is_empty() {
            tree.new_visual_node(inner_style, inner_visual)
        } else {
            tree.new_visual_node_with_children(inner_style, inner_visual, &inner_children)
        };

        // Progress arc (simplified as a colored segment)
        // In a real implementation, this would use SVG-style arc rendering
        if let Some(value) = props.value {
            let progress_size = size;
            let progress_style = StyleBuilder::new()
                .size(progress_size, progress_size)
                .build();

            // Simplified: show a partial circle based on value
            // Real implementation would need proper arc rendering
            let progress_visual = NodeVisual::default()
                .with_border(hex_to_rgba(props.variant.color()), stroke_width)
                .with_radius(progress_size / 2.0);

            let progress = tree.new_visual_node(progress_style, progress_visual);

            tree.new_node_with_children(container_style, &[progress, inner])
        } else {
            let outer = tree.new_visual_node_with_children(outer_style, outer_visual, &[inner]);
            tree.new_node_with_children(container_style, &[outer])
        }
    }
}

// =============================================================================
// SPINNER COMPONENT
// =============================================================================

/// Simple loading spinner (convenience wrapper)
pub struct Spinner;

impl Spinner {
    /// Build a small spinner
    pub fn small(tree: &mut LayoutTree) -> NodeId {
        CircularProgress::build(tree, CircularProgressProps::indeterminate().size(ProgressSize::Small))
    }

    /// Build a medium spinner
    pub fn medium(tree: &mut LayoutTree) -> NodeId {
        CircularProgress::build(tree, CircularProgressProps::indeterminate().size(ProgressSize::Medium))
    }

    /// Build a large spinner
    pub fn large(tree: &mut LayoutTree) -> NodeId {
        CircularProgress::build(tree, CircularProgressProps::indeterminate().size(ProgressSize::Large))
    }

    /// Build a spinner with custom variant
    pub fn build(tree: &mut LayoutTree, size: ProgressSize, variant: ProgressVariant) -> NodeId {
        CircularProgress::build(
            tree,
            CircularProgressProps::indeterminate()
                .size(size)
                .variant(variant),
        )
    }
}

// =============================================================================
// PROGRESS STEPS
// =============================================================================

/// Step status
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum StepStatus {
    #[default]
    Pending,
    InProgress,
    Completed,
    Error,
}

/// Step definition
#[derive(Debug, Clone)]
pub struct Step {
    pub label: String,
    pub description: Option<String>,
    pub status: StepStatus,
}

impl Step {
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            description: None,
            status: StepStatus::Pending,
        }
    }

    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    pub fn status(mut self, status: StepStatus) -> Self {
        self.status = status;
        self
    }
}

/// Steps progress component
pub struct Steps;

impl Steps {
    /// Build a horizontal steps indicator
    pub fn build(tree: &mut LayoutTree, steps: &[Step], current: usize) -> NodeId {
        let container_style = StyleBuilder::new()
            .flex_row()
            .align_start()
            .width_percent(1.0)
            .build();

        let step_nodes: Vec<NodeId> = steps.iter()
            .enumerate()
            .map(|(i, step)| {
                let is_last = i == steps.len() - 1;
                Self::build_step(tree, step, i, current, is_last)
            })
            .collect();

        tree.new_node_with_children(container_style, &step_nodes)
    }

    fn build_step(
        tree: &mut LayoutTree,
        step: &Step,
        index: usize,
        current: usize,
        is_last: bool,
    ) -> NodeId {
        let step_style = StyleBuilder::new()
            .flex_row()
            .align_start()
            .flex_grow(if is_last { 0.0 } else { 1.0 })
            .build();

        // Step indicator
        let indicator_style = StyleBuilder::new()
            .flex_column()
            .align_center()
            .gap(spacing::SM)
            .build();

        // Circle
        let circle_size = 32.0;
        let circle_style = StyleBuilder::new()
            .size(circle_size, circle_size)
            .flex_row()
            .center()
            .build();

        let (bg_color, border_color) = match step.status {
            StepStatus::Completed => (colors::SUCCESS, colors::SUCCESS),
            StepStatus::InProgress => (colors::PRIMARY, colors::PRIMARY),
            StepStatus::Error => (colors::DANGER, colors::DANGER),
            StepStatus::Pending => {
                if index <= current {
                    (colors::PRIMARY, colors::PRIMARY)
                } else {
                    (colors::SURFACE, colors::BORDER)
                }
            }
        };

        let circle_visual = NodeVisual::default()
            .with_background(hex_to_rgba(if matches!(step.status, StepStatus::Completed | StepStatus::InProgress | StepStatus::Error) {
                bg_color
            } else {
                colors::SURFACE
            }))
            .with_border(hex_to_rgba(border_color), 2.0)
            .with_radius(circle_size / 2.0);

        let circle = tree.new_visual_node(circle_style, circle_visual);

        // Label
        let label_style = StyleBuilder::new()
            .flex_column()
            .align_center()
            .gap(spacing::XS)
            .build();
        let label_text = tree.new_node(StyleBuilder::new().build());

        let label = tree.new_node_with_children(label_style, &[label_text]);

        let indicator = tree.new_node_with_children(indicator_style, &[circle, label]);

        // Connector line (if not last)
        if is_last {
            tree.new_node_with_children(step_style, &[indicator])
        } else {
            let line_style = StyleBuilder::new()
                .flex_grow(1.0)
                .height(2.0)
                .margin(spacing::SM) // Simplified margin
                .build();

            let line_color = if index < current {
                colors::SUCCESS
            } else {
                colors::BORDER
            };

            let line_visual = NodeVisual::default()
                .with_background(hex_to_rgba(line_color));

            let line = tree.new_visual_node(line_style, line_visual);

            tree.new_node_with_children(step_style, &[indicator, line])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_size() {
        assert_eq!(ProgressSize::Small.height(), 4.0);
        assert_eq!(ProgressSize::Medium.height(), 8.0);
        assert_eq!(ProgressSize::Large.height(), 12.0);
    }

    #[test]
    fn test_linear_progress_build() {
        let mut tree = LayoutTree::new();
        let props = LinearProgressProps::new(0.5).with_label();
        let _node = LinearProgress::build(&mut tree, props);
    }

    #[test]
    fn test_indeterminate_progress() {
        let mut tree = LayoutTree::new();
        let props = LinearProgressProps::indeterminate();
        assert!(props.value.is_none());
        let _node = LinearProgress::build(&mut tree, props);
    }

    #[test]
    fn test_circular_progress_build() {
        let mut tree = LayoutTree::new();
        let props = CircularProgressProps::new(0.75).with_value();
        let _node = CircularProgress::build(&mut tree, props);
    }

    #[test]
    fn test_spinner_build() {
        let mut tree = LayoutTree::new();
        let _small = Spinner::small(&mut tree);
        let _medium = Spinner::medium(&mut tree);
        let _large = Spinner::large(&mut tree);
    }

    #[test]
    fn test_steps_build() {
        let mut tree = LayoutTree::new();
        let steps = vec![
            Step::new("Step 1").status(StepStatus::Completed),
            Step::new("Step 2").status(StepStatus::InProgress),
            Step::new("Step 3"),
        ];
        let _node = Steps::build(&mut tree, &steps, 1);
    }
}
