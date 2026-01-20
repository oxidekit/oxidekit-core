//! Component Inspector
//!
//! Provides component selection, property viewing, and source location tracking.
//! This module is available in all builds for basic inspection capabilities.

use oxide_components::{ComponentSpec, PropSpec};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Inspector state for managing component selection and display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectorState {
    /// Currently selected component ID
    pub selected: Option<String>,
    /// Currently hovered component ID
    pub hovered: Option<String>,
    /// Whether the inspector overlay is visible
    pub visible: bool,
    /// Show layout bounds for all components
    pub show_layout_bounds: bool,
    /// Show padding visualization
    pub show_padding: bool,
    /// Show margin visualization
    pub show_margin: bool,
    /// Show component labels
    pub show_labels: bool,
    /// Component breadcrumb path (from root to selected)
    pub breadcrumb: Vec<BreadcrumbItem>,
    /// Inspection mode
    pub mode: InspectionMode,
}

impl Default for InspectorState {
    fn default() -> Self {
        Self {
            selected: None,
            hovered: None,
            visible: false,
            show_layout_bounds: false,
            show_padding: false,
            show_margin: false,
            show_labels: true,
            breadcrumb: Vec::new(),
            mode: InspectionMode::Select,
        }
    }
}

impl InspectorState {
    /// Create a new inspector state
    pub fn new() -> Self {
        Self::default()
    }

    /// Toggle inspector visibility
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    /// Select a component by ID
    pub fn select(&mut self, id: impl Into<String>) {
        self.selected = Some(id.into());
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected = None;
        self.breadcrumb.clear();
    }

    /// Set hover target
    pub fn hover(&mut self, id: Option<String>) {
        self.hovered = id;
    }

    /// Update breadcrumb from component path
    pub fn set_breadcrumb(&mut self, items: Vec<BreadcrumbItem>) {
        self.breadcrumb = items;
    }

    /// Toggle layout bounds display
    pub fn toggle_layout_bounds(&mut self) {
        self.show_layout_bounds = !self.show_layout_bounds;
    }
}

/// Inspection mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum InspectionMode {
    /// Select components on click
    #[default]
    Select,
    /// Measure distances between elements
    Measure,
    /// Pick colors from the UI
    ColorPicker,
    /// View accessibility tree
    Accessibility,
}

/// Breadcrumb item for component path
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreadcrumbItem {
    /// Component ID
    pub id: String,
    /// Display name
    pub name: String,
    /// Component type (e.g., "Button", "Container")
    pub component_type: String,
}

/// Complete information about an inspected component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentInfo {
    /// Unique component instance ID
    pub id: String,
    /// Component type ID (e.g., "ui.Button")
    pub component_id: String,
    /// Display name
    pub name: String,
    /// Source file location
    pub source: Option<SourceLocation>,
    /// Current property values
    pub props: Vec<PropertyInfo>,
    /// Computed style values
    pub computed_styles: HashMap<String, StyleValue>,
    /// Resolved token references
    pub resolved_tokens: Vec<ResolvedToken>,
    /// Layout information
    pub layout: LayoutInfo,
    /// Typography information
    pub typography: Option<TypographyInfo>,
    /// Component state (hover, focus, etc.)
    pub state: ComponentState,
    /// Children count
    pub children_count: usize,
    /// Parent component ID
    pub parent_id: Option<String>,
    /// Component variant (if applicable)
    pub variant: Option<String>,
}

impl ComponentInfo {
    /// Create a basic component info
    pub fn new(id: impl Into<String>, component_id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            component_id: component_id.into(),
            name: String::new(),
            source: None,
            props: Vec::new(),
            computed_styles: HashMap::new(),
            resolved_tokens: Vec::new(),
            layout: LayoutInfo::default(),
            typography: None,
            state: ComponentState::default(),
            children_count: 0,
            parent_id: None,
            variant: None,
        }
    }

    /// Create info from a component spec
    pub fn from_spec(instance_id: impl Into<String>, spec: &ComponentSpec) -> Self {
        Self {
            id: instance_id.into(),
            component_id: spec.id.clone(),
            name: spec.name.clone(),
            source: None,
            props: spec
                .props
                .iter()
                .map(|p| PropertyInfo::from_spec(p))
                .collect(),
            computed_styles: HashMap::new(),
            resolved_tokens: Vec::new(),
            layout: LayoutInfo::default(),
            typography: None,
            state: ComponentState::default(),
            children_count: 0,
            parent_id: None,
            variant: None,
        }
    }
}

/// Source file location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    /// File path
    pub file: String,
    /// Line number (1-indexed)
    pub line: u32,
    /// Column number (1-indexed)
    pub column: u32,
    /// Character offset from start of file
    pub offset: Option<usize>,
}

impl SourceLocation {
    pub fn new(file: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            file: file.into(),
            line,
            column,
            offset: None,
        }
    }

    /// Format as "file:line:column"
    pub fn to_string(&self) -> String {
        format!("{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Property information with current value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyInfo {
    /// Property name
    pub name: String,
    /// Property type
    pub prop_type: String,
    /// Current value (JSON)
    pub value: serde_json::Value,
    /// Default value (if different from current)
    pub default_value: Option<serde_json::Value>,
    /// Whether the property was explicitly set
    pub is_set: bool,
    /// Whether the property is required
    pub required: bool,
    /// Property description
    pub description: String,
    /// If this is a token reference, the token path
    pub token_ref: Option<String>,
}

impl PropertyInfo {
    /// Create from a PropSpec
    pub fn from_spec(spec: &PropSpec) -> Self {
        Self {
            name: spec.name.clone(),
            prop_type: format!("{:?}", spec.prop_type),
            value: spec.default.clone().unwrap_or(serde_json::Value::Null),
            default_value: spec.default.clone(),
            is_set: false,
            required: spec.required,
            description: spec.description.clone(),
            token_ref: None,
        }
    }
}

/// Computed style value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleValue {
    /// Raw value (e.g., "#3B82F6", "16px")
    pub raw: String,
    /// Computed numeric value (if applicable)
    pub computed: Option<f64>,
    /// Unit (px, %, em, etc.)
    pub unit: Option<String>,
    /// Token reference (if from a token)
    pub token: Option<String>,
    /// Whether this was inherited
    pub inherited: bool,
}

impl StyleValue {
    pub fn from_raw(raw: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            computed: None,
            unit: None,
            token: None,
            inherited: false,
        }
    }

    pub fn from_token(raw: impl Into<String>, token: impl Into<String>) -> Self {
        Self {
            raw: raw.into(),
            computed: None,
            unit: None,
            token: Some(token.into()),
            inherited: false,
        }
    }

    pub fn with_computed(mut self, value: f64, unit: impl Into<String>) -> Self {
        self.computed = Some(value);
        self.unit = Some(unit.into());
        self
    }
}

/// Resolved token information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedToken {
    /// Token path (e.g., "color.primary")
    pub path: String,
    /// Resolved value
    pub value: String,
    /// Category (color, spacing, typography, etc.)
    pub category: String,
    /// Where this token is used (property name)
    pub used_in: Vec<String>,
}

/// Layout information for a component
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LayoutInfo {
    /// Position X
    pub x: f32,
    /// Position Y
    pub y: f32,
    /// Width
    pub width: f32,
    /// Height
    pub height: f32,
    /// Padding (top, right, bottom, left)
    pub padding: [f32; 4],
    /// Margin (top, right, bottom, left)
    pub margin: [f32; 4],
    /// Border width
    pub border_width: f32,
    /// Corner radius
    pub corner_radius: f32,
    /// Display type
    pub display: String,
    /// Flex direction (if flex)
    pub flex_direction: Option<String>,
    /// Align items
    pub align_items: Option<String>,
    /// Justify content
    pub justify_content: Option<String>,
    /// Gap
    pub gap: Option<f32>,
}

impl LayoutInfo {
    /// Get content box dimensions
    pub fn content_box(&self) -> (f32, f32, f32, f32) {
        let x = self.x + self.padding[3] + self.border_width;
        let y = self.y + self.padding[0] + self.border_width;
        let w = self.width - self.padding[1] - self.padding[3] - 2.0 * self.border_width;
        let h = self.height - self.padding[0] - self.padding[2] - 2.0 * self.border_width;
        (x, y, w, h)
    }

    /// Get border box dimensions
    pub fn border_box(&self) -> (f32, f32, f32, f32) {
        (self.x, self.y, self.width, self.height)
    }

    /// Get margin box dimensions
    pub fn margin_box(&self) -> (f32, f32, f32, f32) {
        (
            self.x - self.margin[3],
            self.y - self.margin[0],
            self.width + self.margin[1] + self.margin[3],
            self.height + self.margin[0] + self.margin[2],
        )
    }
}

/// Typography information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyInfo {
    /// Typography role (body, heading, code, etc.)
    pub role: String,
    /// Font family
    pub font_family: String,
    /// Font size (px)
    pub font_size: f32,
    /// Font weight
    pub font_weight: u16,
    /// Line height
    pub line_height: f32,
    /// Letter spacing
    pub letter_spacing: f32,
    /// Text color
    pub color: String,
}

/// Component interaction state
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ComponentState {
    /// Is the component hovered
    pub hovered: bool,
    /// Is the component focused
    pub focused: bool,
    /// Is the component active/pressed
    pub active: bool,
    /// Is the component disabled
    pub disabled: bool,
    /// Is the component loading
    pub loading: bool,
    /// Custom state flags
    pub custom: HashMap<String, bool>,
}

impl ComponentState {
    /// Get the current state name for styling
    pub fn current_state_name(&self) -> &'static str {
        if self.disabled {
            "disabled"
        } else if self.active {
            "active"
        } else if self.focused {
            "focused"
        } else if self.hovered {
            "hovered"
        } else {
            "normal"
        }
    }
}

/// Hit testing result
#[derive(Debug, Clone)]
pub struct HitTestResult {
    /// Component ID that was hit
    pub component_id: String,
    /// Hit position relative to component
    pub local_x: f32,
    pub local_y: f32,
    /// Distance from the hit point to component center
    pub distance_to_center: f32,
}

/// Inspector for component introspection
pub struct Inspector {
    /// Current state
    pub state: InspectorState,
    /// Cached component info for selected component
    cached_info: Option<ComponentInfo>,
    /// Component registry reference (for looking up specs)
    component_specs: HashMap<String, ComponentSpec>,
}

impl Inspector {
    /// Create a new inspector
    pub fn new() -> Self {
        Self {
            state: InspectorState::default(),
            cached_info: None,
            component_specs: HashMap::new(),
        }
    }

    /// Register a component spec for inspection
    pub fn register_spec(&mut self, spec: ComponentSpec) {
        self.component_specs.insert(spec.id.clone(), spec);
    }

    /// Get spec for a component type
    pub fn get_spec(&self, component_id: &str) -> Option<&ComponentSpec> {
        self.component_specs.get(component_id)
    }

    /// Toggle inspector visibility
    pub fn toggle(&mut self) {
        self.state.toggle();
    }

    /// Handle component selection
    pub fn select(&mut self, info: ComponentInfo) {
        self.state.selected = Some(info.id.clone());
        self.cached_info = Some(info);
    }

    /// Get currently selected component info
    pub fn selected_info(&self) -> Option<&ComponentInfo> {
        self.cached_info.as_ref()
    }

    /// Clear selection
    pub fn clear(&mut self) {
        self.state.clear_selection();
        self.cached_info = None;
    }

    /// Perform hit test at coordinates
    pub fn hit_test(&self, _x: f32, _y: f32) -> Option<HitTestResult> {
        // This would be implemented by the runtime to check component bounds
        None
    }
}

impl Default for Inspector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inspector_state() {
        let mut state = InspectorState::new();
        assert!(!state.visible);

        state.toggle();
        assert!(state.visible);

        state.select("test-component");
        assert_eq!(state.selected, Some("test-component".to_string()));

        state.clear_selection();
        assert!(state.selected.is_none());
    }

    #[test]
    fn test_component_info() {
        let info = ComponentInfo::new("instance-1", "ui.Button");
        assert_eq!(info.id, "instance-1");
        assert_eq!(info.component_id, "ui.Button");
    }

    #[test]
    fn test_layout_info_boxes() {
        let layout = LayoutInfo {
            x: 100.0,
            y: 50.0,
            width: 200.0,
            height: 100.0,
            padding: [10.0, 15.0, 10.0, 15.0],
            margin: [5.0, 10.0, 5.0, 10.0],
            border_width: 2.0,
            ..Default::default()
        };

        let (bx, by, bw, bh) = layout.border_box();
        assert_eq!((bx, by, bw, bh), (100.0, 50.0, 200.0, 100.0));

        let (mx, my, mw, mh) = layout.margin_box();
        assert_eq!(mx, 90.0);
        assert_eq!(my, 45.0);
    }

    #[test]
    fn test_component_state() {
        let mut state = ComponentState::default();
        assert_eq!(state.current_state_name(), "normal");

        state.hovered = true;
        assert_eq!(state.current_state_name(), "hovered");

        state.focused = true;
        assert_eq!(state.current_state_name(), "focused");

        state.disabled = true;
        assert_eq!(state.current_state_name(), "disabled");
    }

    #[test]
    fn test_source_location() {
        let loc = SourceLocation::new("ui/app.oui", 42, 10);
        assert_eq!(loc.to_string(), "ui/app.oui:42:10");
    }
}
