//! Touch-friendly UI inspector for mobile devices

use serde::{Deserialize, Serialize};

/// Mobile inspector overlay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileInspectorConfig {
    /// Enable shake-to-inspect gesture
    pub shake_to_inspect: bool,

    /// Multi-touch activation (e.g., 3-finger long press)
    pub activation_gesture: InspectorActivation,

    /// Overlay opacity
    pub overlay_opacity: f32,

    /// Show component bounds
    pub show_bounds: bool,

    /// Show safe areas
    pub show_safe_areas: bool,

    /// Show touch points
    pub show_touches: bool,
}

impl Default for MobileInspectorConfig {
    fn default() -> Self {
        Self {
            shake_to_inspect: true,
            activation_gesture: InspectorActivation::ThreeFingerLongPress,
            overlay_opacity: 0.8,
            show_bounds: true,
            show_safe_areas: true,
            show_touches: false,
        }
    }
}

/// How to activate the mobile inspector
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum InspectorActivation {
    /// Shake the device
    Shake,
    /// Long press with multiple fingers
    ThreeFingerLongPress,
    /// Volume button combination
    VolumeButtons,
    /// Debug menu item
    DebugMenu,
    /// Disabled
    Disabled,
}

impl Default for InspectorActivation {
    fn default() -> Self {
        Self::ThreeFingerLongPress
    }
}

/// Mobile inspector state
pub struct MobileInspector {
    config: MobileInspectorConfig,
    active: bool,
    selected_component: Option<ComponentId>,
    touch_points: Vec<TouchPoint>,
}

/// Unique identifier for a component
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentId(pub u64);

/// Touch point information
#[derive(Debug, Clone)]
pub struct TouchPoint {
    pub id: u64,
    pub x: f32,
    pub y: f32,
    pub phase: TouchPhase,
}

/// Phase of a touch event
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TouchPhase {
    Began,
    Moved,
    Ended,
    Cancelled,
}

impl MobileInspector {
    /// Create a new mobile inspector with the given configuration
    pub fn new(config: MobileInspectorConfig) -> Self {
        Self {
            config,
            active: false,
            selected_component: None,
            touch_points: Vec::new(),
        }
    }

    /// Activate the inspector
    pub fn activate(&mut self) {
        self.active = true;
        tracing::info!("[Inspector] Activated");
    }

    /// Deactivate the inspector
    pub fn deactivate(&mut self) {
        self.active = false;
        self.selected_component = None;
        tracing::info!("[Inspector] Deactivated");
    }

    /// Check if the inspector is active
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Toggle the inspector on/off
    pub fn toggle(&mut self) {
        if self.active {
            self.deactivate();
        } else {
            self.activate();
        }
    }

    /// Get the current configuration
    pub fn config(&self) -> &MobileInspectorConfig {
        &self.config
    }

    /// Update the configuration
    pub fn set_config(&mut self, config: MobileInspectorConfig) {
        self.config = config;
    }

    /// Select component at the given position
    pub fn select_at(&mut self, x: f32, y: f32) -> Option<ComponentId> {
        // Hit test to find component at position
        // Return selected component
        tracing::debug!("[Inspector] Select at ({}, {})", x, y);
        None
    }

    /// Get information about a component
    pub fn get_component_info(&self, _id: ComponentId) -> Option<ComponentInspectInfo> {
        // Return component details for inspection panel
        None
    }

    /// Get the currently selected component
    pub fn selected_component(&self) -> Option<ComponentId> {
        self.selected_component
    }

    /// Update touch points for visualization
    pub fn update_touches(&mut self, touches: &[TouchPoint]) {
        self.touch_points = touches.to_vec();
    }

    /// Get current touch points
    pub fn touch_points(&self) -> &[TouchPoint] {
        &self.touch_points
    }
}

impl Default for MobileInspector {
    fn default() -> Self {
        Self::new(MobileInspectorConfig::default())
    }
}

/// Information about a component for inspection
#[derive(Debug, Clone, Serialize)]
pub struct ComponentInspectInfo {
    pub id: u64,
    pub component_type: String,
    pub bounds: Bounds,
    pub props: Vec<PropInfo>,
    pub style: Vec<StyleInfo>,
    pub accessibility: Option<AccessibilityInfo>,
}

/// Component bounds
#[derive(Debug, Clone, Serialize)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Bounds {
    /// Create new bounds
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }

    /// Check if a point is inside the bounds
    pub fn contains(&self, x: f32, y: f32) -> bool {
        x >= self.x && x <= self.x + self.width && y >= self.y && y <= self.y + self.height
    }
}

/// Property information for inspection
#[derive(Debug, Clone, Serialize)]
pub struct PropInfo {
    pub name: String,
    pub value: String,
    pub editable: bool,
}

/// Style information for inspection
#[derive(Debug, Clone, Serialize)]
pub struct StyleInfo {
    pub property: String,
    pub value: String,
    pub source: String, // "inline", "theme", "computed"
}

/// Accessibility information for a component
#[derive(Debug, Clone, Serialize)]
pub struct AccessibilityInfo {
    pub label: Option<String>,
    pub hint: Option<String>,
    pub role: String,
    pub traits: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_inspector_config_default() {
        let config = MobileInspectorConfig::default();
        assert!(config.shake_to_inspect);
        assert_eq!(config.activation_gesture, InspectorActivation::ThreeFingerLongPress);
        assert_eq!(config.overlay_opacity, 0.8);
        assert!(config.show_bounds);
        assert!(config.show_safe_areas);
        assert!(!config.show_touches);
    }

    #[test]
    fn test_mobile_inspector_activation() {
        let mut inspector = MobileInspector::new(MobileInspectorConfig::default());
        assert!(!inspector.is_active());

        inspector.activate();
        assert!(inspector.is_active());

        inspector.deactivate();
        assert!(!inspector.is_active());
    }

    #[test]
    fn test_mobile_inspector_toggle() {
        let mut inspector = MobileInspector::default();
        assert!(!inspector.is_active());

        inspector.toggle();
        assert!(inspector.is_active());

        inspector.toggle();
        assert!(!inspector.is_active());
    }

    #[test]
    fn test_bounds_contains() {
        let bounds = Bounds::new(10.0, 20.0, 100.0, 50.0);
        assert!(bounds.contains(50.0, 40.0));
        assert!(bounds.contains(10.0, 20.0));
        assert!(!bounds.contains(5.0, 20.0));
        assert!(!bounds.contains(50.0, 100.0));
    }

    #[test]
    fn test_touch_points_update() {
        let mut inspector = MobileInspector::default();
        assert!(inspector.touch_points().is_empty());

        let touches = vec![
            TouchPoint { id: 1, x: 100.0, y: 200.0, phase: TouchPhase::Began },
            TouchPoint { id: 2, x: 150.0, y: 250.0, phase: TouchPhase::Moved },
        ];
        inspector.update_touches(&touches);
        assert_eq!(inspector.touch_points().len(), 2);
    }
}
