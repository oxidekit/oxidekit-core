//! Keyboard input handling for text editing
//!
//! Provides keyboard shortcut handling and key event processing.

use crate::cursor::{Cursor, CursorDirection, CursorUnit};
use crate::selection::SelectionRange;

/// Modifier keys state
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Modifiers {
    /// Shift key is pressed
    pub shift: bool,
    /// Control key is pressed (Ctrl on Windows/Linux, Control on macOS)
    pub ctrl: bool,
    /// Alt/Option key is pressed
    pub alt: bool,
    /// Meta/Super key is pressed (Cmd on macOS, Win on Windows)
    pub meta: bool,
}

impl Modifiers {
    /// Create a new modifiers state
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if no modifiers are pressed
    pub fn is_empty(&self) -> bool {
        !self.shift && !self.ctrl && !self.alt && !self.meta
    }

    /// Check if only shift is pressed
    pub fn is_shift_only(&self) -> bool {
        self.shift && !self.ctrl && !self.alt && !self.meta
    }

    /// Check if the primary modifier is pressed (Cmd on macOS, Ctrl elsewhere)
    /// For cross-platform shortcuts
    pub fn primary(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.meta
        } else {
            self.ctrl
        }
    }

    /// Check if the word modifier is pressed (Option on macOS, Ctrl elsewhere)
    /// Used for word-by-word navigation
    pub fn word_modifier(&self) -> bool {
        if cfg!(target_os = "macos") {
            self.alt
        } else {
            self.ctrl
        }
    }

    /// Check if Ctrl (or Cmd on macOS) and Shift are both pressed
    pub fn primary_shift(&self) -> bool {
        self.primary() && self.shift
    }
}

/// Virtual key codes for keyboard input
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyCode {
    // Letters
    A, B, C, D, E, F, G, H, I, J, K, L, M,
    N, O, P, Q, R, S, T, U, V, W, X, Y, Z,

    // Numbers
    Digit0, Digit1, Digit2, Digit3, Digit4,
    Digit5, Digit6, Digit7, Digit8, Digit9,

    // Navigation
    Left, Right, Up, Down,
    Home, End, PageUp, PageDown,

    // Editing
    Backspace, Delete, Enter, Tab,
    Space, Escape,

    // Function keys
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,

    // Other
    Insert,

    // Unknown/other key
    Unknown,
}

/// Keyboard event
#[derive(Debug, Clone)]
pub struct KeyEvent {
    /// The key that was pressed
    pub key: KeyCode,
    /// Modifier keys state
    pub modifiers: Modifiers,
    /// Text input associated with this key (if any)
    pub text: Option<String>,
    /// Whether this is a key repeat event
    pub is_repeat: bool,
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(key: KeyCode, modifiers: Modifiers) -> Self {
        Self {
            key,
            modifiers,
            text: None,
            is_repeat: false,
        }
    }

    /// Create a key event with text input
    pub fn with_text(key: KeyCode, modifiers: Modifiers, text: impl Into<String>) -> Self {
        Self {
            key,
            modifiers,
            text: Some(text.into()),
            is_repeat: false,
        }
    }

    /// Mark this event as a key repeat
    pub fn repeated(mut self) -> Self {
        self.is_repeat = true;
        self
    }
}

/// Actions that can result from keyboard input
#[derive(Debug, Clone, PartialEq)]
pub enum KeyAction {
    /// Insert text at cursor
    InsertText(String),
    /// Insert a newline
    InsertNewline,
    /// Insert a tab (or spaces)
    InsertTab,

    // Cursor movement
    /// Move cursor in a direction
    MoveCursor {
        direction: CursorDirection,
        unit: CursorUnit,
        extend_selection: bool,
    },

    // Selection
    /// Select all text
    SelectAll,

    // Editing
    /// Delete backward (backspace)
    DeleteBackward,
    /// Delete forward (delete key)
    DeleteForward,
    /// Delete word backward
    DeleteWordBackward,
    /// Delete word forward
    DeleteWordForward,
    /// Delete to line start
    DeleteToLineStart,
    /// Delete to line end
    DeleteToLineEnd,

    // Clipboard
    /// Copy selection
    Copy,
    /// Cut selection
    Cut,
    /// Paste from clipboard
    Paste,
    /// Paste without formatting
    PastePlain,

    // Undo/Redo
    /// Undo last action
    Undo,
    /// Redo last undone action
    Redo,

    // Other
    /// No action (event not handled)
    None,
    /// Escape (blur, cancel, etc.)
    Escape,
}

/// Keyboard shortcut handler
pub struct KeyboardHandler {
    /// Whether to use macOS-style shortcuts
    macos_shortcuts: bool,
}

impl KeyboardHandler {
    /// Create a new keyboard handler
    pub fn new() -> Self {
        Self {
            macos_shortcuts: cfg!(target_os = "macos"),
        }
    }

    /// Create a handler with explicit macOS mode
    pub fn with_macos_shortcuts(macos: bool) -> Self {
        Self {
            macos_shortcuts: macos,
        }
    }

    /// Handle a key event and return the corresponding action
    pub fn handle_key(&self, event: &KeyEvent) -> KeyAction {
        // Check for shortcuts first
        if let Some(action) = self.check_shortcuts(event) {
            return action;
        }

        // Handle navigation keys
        if let Some(action) = self.handle_navigation(event) {
            return action;
        }

        // Handle editing keys
        if let Some(action) = self.handle_editing(event) {
            return action;
        }

        // Handle text input
        if let Some(ref text) = event.text {
            if !text.is_empty() && event.modifiers.is_empty() || event.modifiers.is_shift_only() {
                return KeyAction::InsertText(text.clone());
            }
        }

        KeyAction::None
    }

    /// Check for keyboard shortcuts
    fn check_shortcuts(&self, event: &KeyEvent) -> Option<KeyAction> {
        let mods = &event.modifiers;

        // Primary shortcuts (Cmd/Ctrl)
        if mods.primary() && !mods.shift && !mods.alt {
            match event.key {
                KeyCode::A => return Some(KeyAction::SelectAll),
                KeyCode::C => return Some(KeyAction::Copy),
                KeyCode::X => return Some(KeyAction::Cut),
                KeyCode::V => return Some(KeyAction::Paste),
                KeyCode::Z => return Some(KeyAction::Undo),
                _ => {}
            }
        }

        // Primary + Shift shortcuts
        if mods.primary() && mods.shift && !mods.alt {
            match event.key {
                KeyCode::Z => return Some(KeyAction::Redo),
                KeyCode::V => return Some(KeyAction::PastePlain),
                _ => {}
            }
        }

        // macOS-specific: Ctrl+K = kill to end of line
        if self.macos_shortcuts && mods.ctrl && !mods.shift && !mods.alt && !mods.meta {
            if event.key == KeyCode::K {
                return Some(KeyAction::DeleteToLineEnd);
            }
        }

        // macOS-specific: Ctrl+H = backspace, Ctrl+D = delete forward
        if self.macos_shortcuts && mods.ctrl && !mods.shift && !mods.alt && !mods.meta {
            match event.key {
                KeyCode::H => return Some(KeyAction::DeleteBackward),
                KeyCode::D => return Some(KeyAction::DeleteForward),
                _ => {}
            }
        }

        None
    }

    /// Handle navigation keys
    fn handle_navigation(&self, event: &KeyEvent) -> Option<KeyAction> {
        let mods = &event.modifiers;
        let extend = mods.shift;

        match event.key {
            KeyCode::Left => {
                let unit = if mods.word_modifier() {
                    CursorUnit::Word
                } else if mods.primary() {
                    CursorUnit::Line
                } else {
                    CursorUnit::Grapheme
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Left,
                    unit,
                    extend_selection: extend,
                })
            }
            KeyCode::Right => {
                let unit = if mods.word_modifier() {
                    CursorUnit::Word
                } else if mods.primary() {
                    CursorUnit::Line
                } else {
                    CursorUnit::Grapheme
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Right,
                    unit,
                    extend_selection: extend,
                })
            }
            KeyCode::Up => {
                let unit = if mods.primary() {
                    CursorUnit::Document
                } else {
                    CursorUnit::Line
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Up,
                    unit,
                    extend_selection: extend,
                })
            }
            KeyCode::Down => {
                let unit = if mods.primary() {
                    CursorUnit::Document
                } else {
                    CursorUnit::Line
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Down,
                    unit,
                    extend_selection: extend,
                })
            }
            KeyCode::Home => {
                let unit = if mods.primary() {
                    CursorUnit::Document
                } else {
                    CursorUnit::Line
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Left,
                    unit,
                    extend_selection: extend,
                })
            }
            KeyCode::End => {
                let unit = if mods.primary() {
                    CursorUnit::Document
                } else {
                    CursorUnit::Line
                };
                Some(KeyAction::MoveCursor {
                    direction: CursorDirection::Right,
                    unit,
                    extend_selection: extend,
                })
            }
            _ => None,
        }
    }

    /// Handle editing keys
    fn handle_editing(&self, event: &KeyEvent) -> Option<KeyAction> {
        let mods = &event.modifiers;

        match event.key {
            KeyCode::Backspace => {
                if mods.word_modifier() {
                    Some(KeyAction::DeleteWordBackward)
                } else if mods.primary() {
                    Some(KeyAction::DeleteToLineStart)
                } else {
                    Some(KeyAction::DeleteBackward)
                }
            }
            KeyCode::Delete => {
                if mods.word_modifier() {
                    Some(KeyAction::DeleteWordForward)
                } else if mods.primary() {
                    Some(KeyAction::DeleteToLineEnd)
                } else {
                    Some(KeyAction::DeleteForward)
                }
            }
            KeyCode::Enter => Some(KeyAction::InsertNewline),
            KeyCode::Tab => Some(KeyAction::InsertTab),
            KeyCode::Escape => Some(KeyAction::Escape),
            _ => None,
        }
    }
}

impl Default for KeyboardHandler {
    fn default() -> Self {
        Self::new()
    }
}

/// Execute a cursor movement action
pub fn execute_cursor_movement(
    cursor: &mut Cursor,
    selection: &mut SelectionRange,
    text: &str,
    direction: CursorDirection,
    unit: CursorUnit,
    extend_selection: bool,
) {
    if extend_selection {
        // If we're starting a selection, set the anchor
        if selection.is_collapsed() {
            selection.anchor = cursor.position();
        }

        // Move cursor
        cursor.move_by(text, direction, unit);

        // Update selection focus
        selection.focus = cursor.position();
    } else {
        // Not extending selection
        if !selection.is_collapsed() {
            // If there's a selection, collapse to the appropriate end
            match direction {
                CursorDirection::Left | CursorDirection::Up => {
                    cursor.set_position(selection.start());
                }
                CursorDirection::Right | CursorDirection::Down => {
                    cursor.set_position(selection.end());
                }
            }
            *selection = SelectionRange::collapsed(cursor.position());
        } else {
            // No selection, just move cursor
            cursor.move_by(text, direction, unit);
            *selection = SelectionRange::collapsed(cursor.position());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_modifiers_empty() {
        let mods = Modifiers::new();
        assert!(mods.is_empty());
        assert!(!mods.shift);
        assert!(!mods.ctrl);
        assert!(!mods.alt);
        assert!(!mods.meta);
    }

    #[test]
    fn test_modifiers_shift_only() {
        let mods = Modifiers {
            shift: true,
            ..Default::default()
        };
        assert!(mods.is_shift_only());
        assert!(!mods.is_empty());
    }

    #[test]
    fn test_key_event_new() {
        let event = KeyEvent::new(KeyCode::A, Modifiers::default());
        assert_eq!(event.key, KeyCode::A);
        assert!(event.modifiers.is_empty());
        assert!(event.text.is_none());
        assert!(!event.is_repeat);
    }

    #[test]
    fn test_key_event_with_text() {
        let event = KeyEvent::with_text(KeyCode::A, Modifiers::default(), "a");
        assert_eq!(event.text, Some("a".to_string()));
    }

    #[test]
    fn test_keyboard_handler_text_input() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::with_text(KeyCode::A, Modifiers::default(), "a");
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::InsertText("a".to_string()));
    }

    #[test]
    fn test_keyboard_handler_copy() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::C, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Copy);
    }

    #[test]
    fn test_keyboard_handler_paste() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::V, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Paste);
    }

    #[test]
    fn test_keyboard_handler_cut() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::X, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Cut);
    }

    #[test]
    fn test_keyboard_handler_undo() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::Z, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Undo);
    }

    #[test]
    fn test_keyboard_handler_redo() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            shift: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::Z, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Redo);
    }

    #[test]
    fn test_keyboard_handler_select_all() {
        let handler = KeyboardHandler::with_macos_shortcuts(false);
        let mods = Modifiers {
            ctrl: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::A, mods);
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::SelectAll);
    }

    #[test]
    fn test_keyboard_handler_navigation() {
        let handler = KeyboardHandler::new();

        // Left arrow
        let event = KeyEvent::new(KeyCode::Left, Modifiers::default());
        let action = handler.handle_key(&event);
        assert!(matches!(action, KeyAction::MoveCursor {
            direction: CursorDirection::Left,
            unit: CursorUnit::Grapheme,
            extend_selection: false,
        }));

        // Shift+Right arrow (extend selection)
        let mods = Modifiers {
            shift: true,
            ..Default::default()
        };
        let event = KeyEvent::new(KeyCode::Right, mods);
        let action = handler.handle_key(&event);
        assert!(matches!(action, KeyAction::MoveCursor {
            direction: CursorDirection::Right,
            unit: CursorUnit::Grapheme,
            extend_selection: true,
        }));
    }

    #[test]
    fn test_keyboard_handler_backspace() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::new(KeyCode::Backspace, Modifiers::default());
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::DeleteBackward);
    }

    #[test]
    fn test_keyboard_handler_delete() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::new(KeyCode::Delete, Modifiers::default());
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::DeleteForward);
    }

    #[test]
    fn test_keyboard_handler_enter() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::new(KeyCode::Enter, Modifiers::default());
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::InsertNewline);
    }

    #[test]
    fn test_keyboard_handler_tab() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::new(KeyCode::Tab, Modifiers::default());
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::InsertTab);
    }

    #[test]
    fn test_keyboard_handler_escape() {
        let handler = KeyboardHandler::new();
        let event = KeyEvent::new(KeyCode::Escape, Modifiers::default());
        let action = handler.handle_key(&event);
        assert_eq!(action, KeyAction::Escape);
    }

    #[test]
    fn test_execute_cursor_movement_simple() {
        let text = "Hello World";
        let mut cursor = Cursor::at_offset(0);
        let mut selection = SelectionRange::collapsed_at(0);

        execute_cursor_movement(
            &mut cursor,
            &mut selection,
            text,
            CursorDirection::Right,
            CursorUnit::Grapheme,
            false,
        );

        assert_eq!(cursor.offset(), 1);
        assert!(selection.is_collapsed());
    }

    #[test]
    fn test_execute_cursor_movement_extend() {
        let text = "Hello World";
        let mut cursor = Cursor::at_offset(0);
        let mut selection = SelectionRange::collapsed_at(0);

        execute_cursor_movement(
            &mut cursor,
            &mut selection,
            text,
            CursorDirection::Right,
            CursorUnit::Grapheme,
            true,
        );

        assert_eq!(cursor.offset(), 1);
        assert!(!selection.is_collapsed());
        assert_eq!(selection.anchor.offset, 0);
        assert_eq!(selection.focus.offset, 1);
    }

    #[test]
    fn test_execute_cursor_movement_collapse() {
        let text = "Hello World";
        let mut cursor = Cursor::at_offset(5);
        let mut selection = SelectionRange::new(
            crate::selection::TextPosition::from_offset(0),
            crate::selection::TextPosition::from_offset(5),
        );

        // Moving right without extend should collapse to end
        execute_cursor_movement(
            &mut cursor,
            &mut selection,
            text,
            CursorDirection::Right,
            CursorUnit::Grapheme,
            false,
        );

        assert_eq!(cursor.offset(), 5);
        assert!(selection.is_collapsed());
    }
}
