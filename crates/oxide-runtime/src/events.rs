//! Event system for OxideKit runtime
//!
//! Provides event handling, hit testing, and event dispatch to components.

use oxide_layout::{ComputedRect, LayoutTree, NodeId};
use std::collections::HashMap;

/// Mouse button types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

/// Event types that can be dispatched to components
#[derive(Debug, Clone)]
pub enum UiEvent {
    /// Mouse clicked on a component
    Click { x: f32, y: f32, button: MouseButton },
    /// Mouse double-clicked on a component
    DoubleClick { x: f32, y: f32, button: MouseButton },
    /// Mouse button pressed down
    MouseDown { x: f32, y: f32, button: MouseButton },
    /// Mouse button released
    MouseUp { x: f32, y: f32, button: MouseButton },
    /// Mouse entered a component's bounds
    MouseEnter { x: f32, y: f32 },
    /// Mouse left a component's bounds
    MouseLeave,
    /// Mouse moved within a component
    MouseMove { x: f32, y: f32 },
    /// Component received focus
    Focus,
    /// Component lost focus
    Blur,
    /// Key pressed while component has focus
    KeyDown { key: String, modifiers: Modifiers },
    /// Key released while component has focus
    KeyUp { key: String, modifiers: Modifiers },
    /// Text input while component has focus
    TextInput { text: String },
}

/// Keyboard modifiers
#[derive(Debug, Clone, Copy, Default)]
pub struct Modifiers {
    pub shift: bool,
    pub ctrl: bool,
    pub alt: bool,
    pub meta: bool,
}

/// An event handler callback type
pub type EventCallback = Box<dyn Fn(&UiEvent) + Send + Sync>;

/// Event handler definition with its associated action
#[derive(Debug, Clone)]
pub struct EventHandler {
    /// The event type this handler responds to
    pub event_type: EventType,
    /// The action to perform (stored as string for now, will be evaluated)
    pub action: HandlerAction,
}

/// Types of events that can be handled
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventType {
    Click,
    DoubleClick,
    MouseDown,
    MouseUp,
    MouseEnter,
    MouseLeave,
    MouseMove,
    Focus,
    Blur,
    KeyDown,
    KeyUp,
    TextInput,
}

/// Action to perform when event fires
#[derive(Debug, Clone)]
pub enum HandlerAction {
    /// Mutate state: field, operation, value
    StateMutation {
        field: String,
        op: MutationOp,
        value: ActionValue,
    },
    /// Call a function
    FunctionCall { name: String, args: Vec<ActionValue> },
    /// Navigate to a route
    Navigate { path: String },
    /// Raw expression (for complex cases)
    Raw(String),
}

/// Mutation operations
#[derive(Debug, Clone, Copy)]
pub enum MutationOp {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
    Toggle,
}

/// Values that can be used in actions
#[derive(Debug, Clone)]
pub enum ActionValue {
    Number(f64),
    String(String),
    Bool(bool),
}

/// Interactive state for a node
#[derive(Debug, Clone, Default)]
pub struct InteractiveState {
    /// Whether the mouse is currently over this node
    pub hovered: bool,
    /// Whether this node has focus
    pub focused: bool,
    /// Whether this node is being pressed
    pub pressed: bool,
    /// Whether this node is disabled
    pub disabled: bool,
}

/// Event manager handles all input events and dispatches to components
pub struct EventManager {
    /// Current mouse position in logical pixels
    pub mouse_position: (f32, f32),
    /// Currently hovered node (if any)
    pub hovered_node: Option<NodeId>,
    /// Currently focused node (if any)
    pub focused_node: Option<NodeId>,
    /// Node being pressed (mouse down but not yet up)
    pub pressed_node: Option<NodeId>,
    /// Event handlers registered for each node
    pub handlers: HashMap<NodeId, Vec<EventHandler>>,
    /// Interactive state for each node
    interactive_states: HashMap<NodeId, InteractiveState>,
    /// Last click time for double-click detection
    last_click_time: std::time::Instant,
    /// Last click position for double-click detection
    last_click_pos: (f32, f32),
    /// Double-click threshold in milliseconds
    double_click_threshold_ms: u64,
    /// Double-click distance threshold in pixels
    double_click_distance: f32,
}

impl Default for EventManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EventManager {
    pub fn new() -> Self {
        Self {
            mouse_position: (0.0, 0.0),
            hovered_node: None,
            focused_node: None,
            pressed_node: None,
            handlers: HashMap::new(),
            interactive_states: HashMap::new(),
            last_click_time: std::time::Instant::now(),
            last_click_pos: (0.0, 0.0),
            double_click_threshold_ms: 500,
            double_click_distance: 5.0,
        }
    }

    /// Register an event handler for a node
    pub fn register_handler(&mut self, node: NodeId, handler: EventHandler) {
        self.handlers.entry(node).or_default().push(handler);
    }

    /// Clear all handlers (called when UI is rebuilt)
    pub fn clear_handlers(&mut self) {
        self.handlers.clear();
    }

    /// Get interactive state for a node
    pub fn get_state(&self, node: NodeId) -> InteractiveState {
        self.interactive_states.get(&node).cloned().unwrap_or_default()
    }

    /// Handle mouse move event
    pub fn on_mouse_move(&mut self, x: f32, y: f32, tree: &LayoutTree, root: NodeId) -> Vec<(NodeId, UiEvent)> {
        let old_position = self.mouse_position;
        self.mouse_position = (x, y);

        let mut events = Vec::new();

        // Find node under mouse
        let hit_node = self.hit_test(x, y, tree, root);

        // Handle hover state changes
        if hit_node != self.hovered_node {
            // Mouse leave old node
            if let Some(old_node) = self.hovered_node {
                if let Some(state) = self.interactive_states.get_mut(&old_node) {
                    state.hovered = false;
                }
                events.push((old_node, UiEvent::MouseLeave));
            }

            // Mouse enter new node
            if let Some(new_node) = hit_node {
                self.interactive_states.entry(new_node).or_default().hovered = true;
                events.push((new_node, UiEvent::MouseEnter { x, y }));
            }

            self.hovered_node = hit_node;
        } else if let Some(node) = hit_node {
            // Mouse move within same node
            if old_position != (x, y) {
                events.push((node, UiEvent::MouseMove { x, y }));
            }
        }

        events
    }

    /// Handle mouse button down event
    pub fn on_mouse_down(&mut self, x: f32, y: f32, button: MouseButton, tree: &LayoutTree, root: NodeId) -> Vec<(NodeId, UiEvent)> {
        let mut events = Vec::new();

        let hit_node = self.hit_test(x, y, tree, root);

        if let Some(node) = hit_node {
            // Update pressed state
            self.pressed_node = Some(node);
            self.interactive_states.entry(node).or_default().pressed = true;

            events.push((node, UiEvent::MouseDown { x, y, button }));

            // Handle focus
            if self.focused_node != Some(node) {
                // Blur old focused node
                if let Some(old_focus) = self.focused_node {
                    if let Some(state) = self.interactive_states.get_mut(&old_focus) {
                        state.focused = false;
                    }
                    events.push((old_focus, UiEvent::Blur));
                }

                // Focus new node
                self.focused_node = Some(node);
                self.interactive_states.entry(node).or_default().focused = true;
                events.push((node, UiEvent::Focus));
            }
        }

        events
    }

    /// Handle mouse button up event
    pub fn on_mouse_up(&mut self, x: f32, y: f32, button: MouseButton, tree: &LayoutTree, root: NodeId) -> Vec<(NodeId, UiEvent)> {
        let mut events = Vec::new();

        let hit_node = self.hit_test(x, y, tree, root);

        // Clear pressed state
        if let Some(pressed) = self.pressed_node.take() {
            if let Some(state) = self.interactive_states.get_mut(&pressed) {
                state.pressed = false;
            }
            events.push((pressed, UiEvent::MouseUp { x, y, button }));

            // If released over the same node that was pressed, it's a click
            if hit_node == Some(pressed) {
                // Check for double-click
                let now = std::time::Instant::now();
                let time_diff = now.duration_since(self.last_click_time).as_millis() as u64;
                let pos_diff = ((x - self.last_click_pos.0).powi(2) + (y - self.last_click_pos.1).powi(2)).sqrt();

                if time_diff < self.double_click_threshold_ms && pos_diff < self.double_click_distance {
                    events.push((pressed, UiEvent::DoubleClick { x, y, button }));
                } else {
                    events.push((pressed, UiEvent::Click { x, y, button }));
                }

                self.last_click_time = now;
                self.last_click_pos = (x, y);
            }
        }

        events
    }

    /// Handle keyboard events
    pub fn on_key_down(&mut self, key: String, modifiers: Modifiers) -> Vec<(NodeId, UiEvent)> {
        let mut events = Vec::new();

        if let Some(focused) = self.focused_node {
            events.push((focused, UiEvent::KeyDown { key, modifiers }));
        }

        events
    }

    /// Handle key up events
    pub fn on_key_up(&mut self, key: String, modifiers: Modifiers) -> Vec<(NodeId, UiEvent)> {
        let mut events = Vec::new();

        if let Some(focused) = self.focused_node {
            events.push((focused, UiEvent::KeyUp { key, modifiers }));
        }

        events
    }

    /// Handle text input events
    pub fn on_text_input(&mut self, text: String) -> Vec<(NodeId, UiEvent)> {
        let mut events = Vec::new();

        if let Some(focused) = self.focused_node {
            events.push((focused, UiEvent::TextInput { text }));
        }

        events
    }

    /// Perform hit testing to find the node at a given position
    /// Returns the topmost (last rendered) node that contains the point
    pub fn hit_test(&self, x: f32, y: f32, tree: &LayoutTree, root: NodeId) -> Option<NodeId> {
        let mut hit_node: Option<NodeId> = None;

        // Traverse tree to find all nodes containing the point
        // The last one in traversal order is the topmost
        tree.traverse(root, |node, rect, _visual| {
            if self.point_in_rect(x, y, &rect) {
                // Check if this node has handlers registered (is interactive)
                // For now, mark all nodes as potentially hittable
                hit_node = Some(node);
            }
        });

        hit_node
    }

    /// Check if a point is inside a rectangle
    fn point_in_rect(&self, x: f32, y: f32, rect: &ComputedRect) -> bool {
        x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
    }

    /// Get handlers for a node
    pub fn get_handlers(&self, node: NodeId) -> Option<&Vec<EventHandler>> {
        self.handlers.get(&node)
    }

    /// Dispatch events to their handlers
    /// Returns a list of actions to execute
    pub fn dispatch_events(&self, events: &[(NodeId, UiEvent)]) -> Vec<(NodeId, &EventHandler)> {
        let mut actions = Vec::new();

        for (node, event) in events {
            if let Some(handlers) = self.handlers.get(node) {
                let event_type = event_to_type(event);
                for handler in handlers {
                    if handler.event_type == event_type {
                        actions.push((*node, handler));
                    }
                }
            }
        }

        actions
    }
}

/// Convert a UiEvent to its EventType
fn event_to_type(event: &UiEvent) -> EventType {
    match event {
        UiEvent::Click { .. } => EventType::Click,
        UiEvent::DoubleClick { .. } => EventType::DoubleClick,
        UiEvent::MouseDown { .. } => EventType::MouseDown,
        UiEvent::MouseUp { .. } => EventType::MouseUp,
        UiEvent::MouseEnter { .. } => EventType::MouseEnter,
        UiEvent::MouseLeave => EventType::MouseLeave,
        UiEvent::MouseMove { .. } => EventType::MouseMove,
        UiEvent::Focus => EventType::Focus,
        UiEvent::Blur => EventType::Blur,
        UiEvent::KeyDown { .. } => EventType::KeyDown,
        UiEvent::KeyUp { .. } => EventType::KeyUp,
        UiEvent::TextInput { .. } => EventType::TextInput,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_point_in_rect() {
        let manager = EventManager::new();
        let rect = ComputedRect { x: 10.0, y: 10.0, width: 100.0, height: 50.0 };

        assert!(manager.point_in_rect(50.0, 30.0, &rect)); // Inside
        assert!(manager.point_in_rect(10.0, 10.0, &rect)); // Top-left corner
        assert!(!manager.point_in_rect(5.0, 30.0, &rect));  // Left of rect
        assert!(!manager.point_in_rect(120.0, 30.0, &rect)); // Right of rect
        assert!(!manager.point_in_rect(50.0, 5.0, &rect));  // Above rect
        assert!(!manager.point_in_rect(50.0, 70.0, &rect)); // Below rect
    }

    #[test]
    fn test_double_click_detection() {
        // Double-click detection is time-based, so we just test the structure
        let manager = EventManager::new();
        assert_eq!(manager.double_click_threshold_ms, 500);
        assert_eq!(manager.double_click_distance, 5.0);
    }

    #[test]
    fn test_interactive_state_default() {
        let state = InteractiveState::default();
        assert!(!state.hovered);
        assert!(!state.focused);
        assert!(!state.pressed);
        assert!(!state.disabled);
    }
}
