//! Keyboard Shortcuts
//!
//! Provides keyboard shortcuts for rapid dev editor navigation and control.
//! Supports customizable key bindings and modifier combinations.

use crate::ModifierKey;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A key combination for a shortcut
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct KeyBinding {
    /// Primary key (letter, number, or named key)
    pub key: Key,
    /// Required modifier keys
    pub modifiers: Vec<ModifierKey>,
}

impl KeyBinding {
    /// Create a simple key binding (no modifiers)
    pub fn key(key: Key) -> Self {
        Self {
            key,
            modifiers: Vec::new(),
        }
    }

    /// Create a key binding with modifiers
    pub fn with_modifiers(key: Key, modifiers: Vec<ModifierKey>) -> Self {
        Self { key, modifiers }
    }

    /// Create a Ctrl+key binding
    pub fn ctrl(key: Key) -> Self {
        Self::with_modifiers(key, vec![ModifierKey::Ctrl])
    }

    /// Create an Alt+key binding
    pub fn alt(key: Key) -> Self {
        Self::with_modifiers(key, vec![ModifierKey::Alt])
    }

    /// Create a Shift+key binding
    pub fn shift(key: Key) -> Self {
        Self::with_modifiers(key, vec![ModifierKey::Shift])
    }

    /// Create a Ctrl+Shift+key binding
    pub fn ctrl_shift(key: Key) -> Self {
        Self::with_modifiers(key, vec![ModifierKey::Ctrl, ModifierKey::Shift])
    }

    /// Format as display string
    pub fn to_display_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        for m in &self.modifiers {
            parts.push(match m {
                ModifierKey::Ctrl => "Ctrl".to_string(),
                ModifierKey::Alt => "Alt".to_string(),
                ModifierKey::Shift => "Shift".to_string(),
                ModifierKey::Meta => "Cmd".to_string(),
            });
        }
        parts.push(self.key.to_display_string());
        parts.join("+")
    }

    /// Check if this binding matches the given input
    pub fn matches(&self, key: &Key, modifiers: &[ModifierKey]) -> bool {
        if self.key != *key {
            return false;
        }

        // Check all required modifiers are present
        for m in &self.modifiers {
            if !modifiers.contains(m) {
                return false;
            }
        }

        // Check no extra modifiers are present
        for m in modifiers {
            if !self.modifiers.contains(m) {
                return false;
            }
        }

        true
    }
}

/// Key identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Key {
    /// Letter key (A-Z)
    Letter(char),
    /// Number key (0-9)
    Number(char),
    /// Function key (F1-F12)
    Function(u8),
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Special keys
    Enter,
    Escape,
    Tab,
    Space,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    /// Bracket keys
    BracketLeft,
    BracketRight,
    /// Punctuation
    Comma,
    Period,
    Slash,
    Backslash,
    Semicolon,
    Quote,
    Minus,
    Equals,
    Backquote,
}

impl Key {
    /// Create from a character
    pub fn from_char(c: char) -> Option<Self> {
        let c = c.to_ascii_uppercase();
        if c.is_ascii_alphabetic() {
            Some(Self::Letter(c))
        } else if c.is_ascii_digit() {
            Some(Self::Number(c))
        } else {
            match c {
                '[' => Some(Self::BracketLeft),
                ']' => Some(Self::BracketRight),
                ',' => Some(Self::Comma),
                '.' => Some(Self::Period),
                '/' => Some(Self::Slash),
                '\\' => Some(Self::Backslash),
                ';' => Some(Self::Semicolon),
                '\'' => Some(Self::Quote),
                '-' => Some(Self::Minus),
                '=' => Some(Self::Equals),
                '`' => Some(Self::Backquote),
                _ => None,
            }
        }
    }

    /// Get display string for the key
    pub fn to_display_string(&self) -> String {
        match self {
            Self::Letter(c) => c.to_string(),
            Self::Number(c) => c.to_string(),
            Self::Function(n) => format!("F{}", n),
            Self::Up => "Up".to_string(),
            Self::Down => "Down".to_string(),
            Self::Left => "Left".to_string(),
            Self::Right => "Right".to_string(),
            Self::Enter => "Enter".to_string(),
            Self::Escape => "Esc".to_string(),
            Self::Tab => "Tab".to_string(),
            Self::Space => "Space".to_string(),
            Self::Backspace => "Backspace".to_string(),
            Self::Delete => "Delete".to_string(),
            Self::Home => "Home".to_string(),
            Self::End => "End".to_string(),
            Self::PageUp => "PgUp".to_string(),
            Self::PageDown => "PgDn".to_string(),
            Self::BracketLeft => "[".to_string(),
            Self::BracketRight => "]".to_string(),
            Self::Comma => ",".to_string(),
            Self::Period => ".".to_string(),
            Self::Slash => "/".to_string(),
            Self::Backslash => "\\".to_string(),
            Self::Semicolon => ";".to_string(),
            Self::Quote => "'".to_string(),
            Self::Minus => "-".to_string(),
            Self::Equals => "=".to_string(),
            Self::Backquote => "`".to_string(),
        }
    }
}

/// Dev editor shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShortcutAction {
    // Inspector toggle
    ToggleInspector,
    ToggleLayoutBounds,
    ToggleProfiler,

    // Navigation
    SelectParent,
    SelectFirstChild,
    SelectNextSibling,
    SelectPrevSibling,
    ClearSelection,

    // Quick search
    QuickSearch,
    GoToSource,

    // Editing
    Undo,
    Redo,
    CopyAsCode,
    PasteComponent,
    DeleteComponent,

    // Context menu
    OpenContextMenu,
    CloseContextMenu,
    ExecuteAction,

    // View modes
    EditMode,
    InspectMode,
    MeasureMode,

    // State simulation
    SimulateHover,
    SimulateFocus,
    SimulateActive,
    ClearSimulation,

    // Panel control
    ShowTreePanel,
    ShowPropsPanel,
    ShowTokensPanel,
    HideAllPanels,

    // Zoom
    ZoomIn,
    ZoomOut,
    ZoomReset,
}

/// Shortcut configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShortcutConfig {
    /// Key bindings for each action
    pub bindings: HashMap<ShortcutAction, KeyBinding>,
    /// Enable/disable shortcuts
    pub enabled: bool,
    /// Help text for shortcuts
    pub help_text: HashMap<ShortcutAction, String>,
}

impl Default for ShortcutConfig {
    fn default() -> Self {
        let mut bindings = HashMap::new();
        let mut help_text = HashMap::new();

        // Inspector toggles
        bindings.insert(
            ShortcutAction::ToggleInspector,
            KeyBinding::ctrl(Key::Letter('I')),
        );
        help_text.insert(
            ShortcutAction::ToggleInspector,
            "Toggle inspector panel".to_string(),
        );

        bindings.insert(
            ShortcutAction::ToggleLayoutBounds,
            KeyBinding::ctrl(Key::Letter('L')),
        );
        help_text.insert(
            ShortcutAction::ToggleLayoutBounds,
            "Toggle layout bounds overlay".to_string(),
        );

        bindings.insert(
            ShortcutAction::ToggleProfiler,
            KeyBinding::ctrl_shift(Key::Letter('P')),
        );
        help_text.insert(
            ShortcutAction::ToggleProfiler,
            "Toggle performance profiler".to_string(),
        );

        // Navigation
        bindings.insert(ShortcutAction::SelectParent, KeyBinding::key(Key::Escape));
        help_text.insert(
            ShortcutAction::SelectParent,
            "Select parent component".to_string(),
        );

        bindings.insert(ShortcutAction::SelectNextSibling, KeyBinding::key(Key::Down));
        help_text.insert(
            ShortcutAction::SelectNextSibling,
            "Select next sibling".to_string(),
        );

        bindings.insert(ShortcutAction::SelectPrevSibling, KeyBinding::key(Key::Up));
        help_text.insert(
            ShortcutAction::SelectPrevSibling,
            "Select previous sibling".to_string(),
        );

        bindings.insert(
            ShortcutAction::SelectFirstChild,
            KeyBinding::key(Key::Enter),
        );
        help_text.insert(
            ShortcutAction::SelectFirstChild,
            "Select first child".to_string(),
        );

        bindings.insert(
            ShortcutAction::ClearSelection,
            KeyBinding::shift(Key::Escape),
        );
        help_text.insert(ShortcutAction::ClearSelection, "Clear selection".to_string());

        // Quick search
        bindings.insert(
            ShortcutAction::QuickSearch,
            KeyBinding::ctrl(Key::Letter('K')),
        );
        help_text.insert(
            ShortcutAction::QuickSearch,
            "Open quick search".to_string(),
        );

        bindings.insert(
            ShortcutAction::GoToSource,
            KeyBinding::ctrl(Key::Letter('G')),
        );
        help_text.insert(ShortcutAction::GoToSource, "Go to source file".to_string());

        // Editing
        bindings.insert(ShortcutAction::Undo, KeyBinding::ctrl(Key::Letter('Z')));
        help_text.insert(ShortcutAction::Undo, "Undo last change".to_string());

        bindings.insert(
            ShortcutAction::Redo,
            KeyBinding::ctrl_shift(Key::Letter('Z')),
        );
        help_text.insert(ShortcutAction::Redo, "Redo last change".to_string());

        bindings.insert(
            ShortcutAction::CopyAsCode,
            KeyBinding::ctrl(Key::Letter('C')),
        );
        help_text.insert(
            ShortcutAction::CopyAsCode,
            "Copy component as code".to_string(),
        );

        bindings.insert(ShortcutAction::DeleteComponent, KeyBinding::key(Key::Delete));
        help_text.insert(
            ShortcutAction::DeleteComponent,
            "Delete selected component".to_string(),
        );

        // Context menu
        bindings.insert(ShortcutAction::OpenContextMenu, KeyBinding::key(Key::Space));
        help_text.insert(
            ShortcutAction::OpenContextMenu,
            "Open context menu".to_string(),
        );

        // View modes
        bindings.insert(ShortcutAction::InspectMode, KeyBinding::key(Key::Letter('I')));
        help_text.insert(
            ShortcutAction::InspectMode,
            "Switch to inspect mode".to_string(),
        );

        bindings.insert(ShortcutAction::EditMode, KeyBinding::key(Key::Letter('E')));
        help_text.insert(ShortcutAction::EditMode, "Switch to edit mode".to_string());

        bindings.insert(ShortcutAction::MeasureMode, KeyBinding::key(Key::Letter('M')));
        help_text.insert(
            ShortcutAction::MeasureMode,
            "Switch to measure mode".to_string(),
        );

        // State simulation
        bindings.insert(ShortcutAction::SimulateHover, KeyBinding::key(Key::Letter('H')));
        help_text.insert(ShortcutAction::SimulateHover, "Simulate hover state".to_string());

        bindings.insert(ShortcutAction::SimulateFocus, KeyBinding::key(Key::Letter('F')));
        help_text.insert(ShortcutAction::SimulateFocus, "Simulate focus state".to_string());

        bindings.insert(ShortcutAction::SimulateActive, KeyBinding::key(Key::Letter('A')));
        help_text.insert(ShortcutAction::SimulateActive, "Simulate active state".to_string());

        // Zoom
        bindings.insert(ShortcutAction::ZoomIn, KeyBinding::ctrl(Key::Equals));
        help_text.insert(ShortcutAction::ZoomIn, "Zoom in".to_string());

        bindings.insert(ShortcutAction::ZoomOut, KeyBinding::ctrl(Key::Minus));
        help_text.insert(ShortcutAction::ZoomOut, "Zoom out".to_string());

        bindings.insert(ShortcutAction::ZoomReset, KeyBinding::ctrl(Key::Number('0')));
        help_text.insert(ShortcutAction::ZoomReset, "Reset zoom".to_string());

        Self {
            bindings,
            enabled: true,
            help_text,
        }
    }
}

impl ShortcutConfig {
    /// Create a new shortcut config
    pub fn new() -> Self {
        Self::default()
    }

    /// Set a custom binding for an action
    pub fn set_binding(&mut self, action: ShortcutAction, binding: KeyBinding) {
        self.bindings.insert(action, binding);
    }

    /// Get binding for an action
    pub fn get_binding(&self, action: ShortcutAction) -> Option<&KeyBinding> {
        self.bindings.get(&action)
    }

    /// Find action for a key input
    pub fn find_action(&self, key: &Key, modifiers: &[ModifierKey]) -> Option<ShortcutAction> {
        if !self.enabled {
            return None;
        }

        for (action, binding) in &self.bindings {
            if binding.matches(key, modifiers) {
                return Some(*action);
            }
        }
        None
    }

    /// Get help text for an action
    pub fn get_help(&self, action: ShortcutAction) -> Option<&String> {
        self.help_text.get(&action)
    }

    /// Generate help text for all shortcuts
    pub fn generate_help(&self) -> String {
        let mut lines = Vec::new();
        lines.push("Keyboard Shortcuts".to_string());
        lines.push("==================".to_string());

        let categories = [
            ("Inspector", vec![
                ShortcutAction::ToggleInspector,
                ShortcutAction::ToggleLayoutBounds,
                ShortcutAction::ToggleProfiler,
            ]),
            ("Navigation", vec![
                ShortcutAction::SelectParent,
                ShortcutAction::SelectNextSibling,
                ShortcutAction::SelectPrevSibling,
                ShortcutAction::SelectFirstChild,
                ShortcutAction::ClearSelection,
            ]),
            ("Search", vec![
                ShortcutAction::QuickSearch,
                ShortcutAction::GoToSource,
            ]),
            ("Editing", vec![
                ShortcutAction::Undo,
                ShortcutAction::Redo,
                ShortcutAction::CopyAsCode,
                ShortcutAction::DeleteComponent,
            ]),
            ("Modes", vec![
                ShortcutAction::InspectMode,
                ShortcutAction::EditMode,
                ShortcutAction::MeasureMode,
            ]),
            ("Simulation", vec![
                ShortcutAction::SimulateHover,
                ShortcutAction::SimulateFocus,
                ShortcutAction::SimulateActive,
            ]),
        ];

        for (name, actions) in categories {
            lines.push(String::new());
            lines.push(format!("{}:", name));
            for action in actions {
                if let Some(binding) = self.get_binding(action) {
                    let help = self.get_help(action).map(|s| s.as_str()).unwrap_or("");
                    lines.push(format!(
                        "  {:15} - {}",
                        binding.to_display_string(),
                        help
                    ));
                }
            }
        }

        lines.join("\n")
    }
}

/// Shortcut handler for processing keyboard input
#[derive(Debug)]
pub struct ShortcutHandler {
    /// Configuration
    pub config: ShortcutConfig,
    /// Currently pressed modifiers
    pressed_modifiers: Vec<ModifierKey>,
    /// Whether shortcuts are temporarily disabled (e.g., during text input)
    suspended: bool,
}

impl Default for ShortcutHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl ShortcutHandler {
    /// Create a new shortcut handler
    pub fn new() -> Self {
        Self {
            config: ShortcutConfig::default(),
            pressed_modifiers: Vec::new(),
            suspended: false,
        }
    }

    /// Create with custom config
    pub fn with_config(config: ShortcutConfig) -> Self {
        Self {
            config,
            pressed_modifiers: Vec::new(),
            suspended: false,
        }
    }

    /// Suspend shortcut handling (e.g., during text input)
    pub fn suspend(&mut self) {
        self.suspended = true;
    }

    /// Resume shortcut handling
    pub fn resume(&mut self) {
        self.suspended = false;
    }

    /// Check if suspended
    pub fn is_suspended(&self) -> bool {
        self.suspended
    }

    /// Handle modifier key press
    pub fn modifier_pressed(&mut self, modifier: ModifierKey) {
        if !self.pressed_modifiers.contains(&modifier) {
            self.pressed_modifiers.push(modifier);
        }
    }

    /// Handle modifier key release
    pub fn modifier_released(&mut self, modifier: ModifierKey) {
        self.pressed_modifiers.retain(|m| *m != modifier);
    }

    /// Clear all pressed modifiers
    pub fn clear_modifiers(&mut self) {
        self.pressed_modifiers.clear();
    }

    /// Get currently pressed modifiers
    pub fn pressed_modifiers(&self) -> &[ModifierKey] {
        &self.pressed_modifiers
    }

    /// Handle key press and return matching action
    pub fn handle_key(&self, key: Key) -> Option<ShortcutAction> {
        if self.suspended {
            return None;
        }
        self.config.find_action(&key, &self.pressed_modifiers)
    }

    /// Check if a specific modifier is pressed
    pub fn is_modifier_pressed(&self, modifier: ModifierKey) -> bool {
        self.pressed_modifiers.contains(&modifier)
    }

    /// Check if Ctrl is pressed
    pub fn is_ctrl_pressed(&self) -> bool {
        self.is_modifier_pressed(ModifierKey::Ctrl)
    }

    /// Check if Alt is pressed
    pub fn is_alt_pressed(&self) -> bool {
        self.is_modifier_pressed(ModifierKey::Alt)
    }

    /// Check if Shift is pressed
    pub fn is_shift_pressed(&self) -> bool {
        self.is_modifier_pressed(ModifierKey::Shift)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_binding() {
        let binding = KeyBinding::ctrl(Key::Letter('I'));
        assert_eq!(binding.to_display_string(), "Ctrl+I");

        assert!(binding.matches(&Key::Letter('I'), &[ModifierKey::Ctrl]));
        assert!(!binding.matches(&Key::Letter('I'), &[]));
        assert!(!binding.matches(&Key::Letter('J'), &[ModifierKey::Ctrl]));
    }

    #[test]
    fn test_key_from_char() {
        assert_eq!(Key::from_char('a'), Some(Key::Letter('A')));
        assert_eq!(Key::from_char('5'), Some(Key::Number('5')));
        assert_eq!(Key::from_char('['), Some(Key::BracketLeft));
    }

    #[test]
    fn test_shortcut_config() {
        let config = ShortcutConfig::default();

        let action = config.find_action(&Key::Letter('I'), &[ModifierKey::Ctrl]);
        assert_eq!(action, Some(ShortcutAction::ToggleInspector));

        let action = config.find_action(&Key::Up, &[]);
        assert_eq!(action, Some(ShortcutAction::SelectPrevSibling));
    }

    #[test]
    fn test_shortcut_handler() {
        let mut handler = ShortcutHandler::new();

        handler.modifier_pressed(ModifierKey::Ctrl);
        let action = handler.handle_key(Key::Letter('I'));
        assert_eq!(action, Some(ShortcutAction::ToggleInspector));

        handler.suspend();
        let action = handler.handle_key(Key::Letter('I'));
        assert!(action.is_none());

        handler.resume();
        let action = handler.handle_key(Key::Letter('I'));
        assert_eq!(action, Some(ShortcutAction::ToggleInspector));
    }

    #[test]
    fn test_help_generation() {
        let config = ShortcutConfig::default();
        let help = config.generate_help();
        assert!(help.contains("Keyboard Shortcuts"));
        assert!(help.contains("Ctrl+I"));
    }
}
