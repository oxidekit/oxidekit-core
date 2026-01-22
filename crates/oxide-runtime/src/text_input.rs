//! Text input integration for OxideKit runtime
//!
//! Provides simple single-line text input handling with cursor, selection, and clipboard support.

use oxide_layout::NodeId;
use oxide_text_edit::{ClipboardProvider, SystemClipboard};
use std::collections::HashMap;

/// Simple text position for single-line inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    /// Character offset (not byte offset)
    pub offset: usize,
}

impl TextPosition {
    /// Create a new position
    pub fn new(offset: usize) -> Self {
        Self { offset }
    }

    /// Create a zero position
    pub fn zero() -> Self {
        Self { offset: 0 }
    }
}

/// Selection range for single-line text inputs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Selection {
    /// Anchor position (where selection started)
    pub anchor: TextPosition,
    /// Focus position (current cursor position)
    pub focus: TextPosition,
}

impl Selection {
    /// Create a collapsed selection (cursor with no selection) at the given position
    pub fn new(position: TextPosition) -> Self {
        Self {
            anchor: position,
            focus: position,
        }
    }

    /// Create a selection range from anchor to focus
    pub fn new_range(anchor: TextPosition, focus: TextPosition) -> Self {
        Self { anchor, focus }
    }

    /// Check if selection has any range (not collapsed)
    pub fn has_selection(&self) -> bool {
        self.anchor.offset != self.focus.offset
    }

    /// Get the cursor position (focus)
    pub fn cursor(&self) -> TextPosition {
        self.focus
    }

    /// Extend selection to a new position
    pub fn extend_to(&mut self, position: TextPosition) {
        self.focus = position;
    }

    /// Get ordered positions (start, end)
    pub fn ordered_positions(&self) -> (TextPosition, TextPosition) {
        if self.anchor.offset <= self.focus.offset {
            (self.anchor, self.focus)
        } else {
            (self.focus, self.anchor)
        }
    }
}

/// Text input field state
#[derive(Debug)]
pub struct TextInputState {
    /// The text content
    pub text: String,
    /// Cursor position and selection
    pub selection: Selection,
    /// Whether the field is focused
    pub focused: bool,
    /// Whether the field is read-only
    pub readonly: bool,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Maximum length (0 = unlimited)
    pub max_length: usize,
    /// Password mode (mask characters)
    pub password: bool,
}

impl Default for TextInputState {
    fn default() -> Self {
        Self {
            text: String::new(),
            selection: Selection::new(TextPosition::zero()),
            focused: false,
            readonly: false,
            placeholder: None,
            max_length: 0,
            password: false,
        }
    }
}

impl TextInputState {
    /// Create a new text input state
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with initial text
    pub fn with_text(text: impl Into<String>) -> Self {
        let text = text.into();
        Self {
            text,
            ..Default::default()
        }
    }

    /// Get the display text (masked if password mode)
    pub fn display_text(&self) -> String {
        if self.password {
            "\u{2022}".repeat(self.text.chars().count())
        } else if self.text.is_empty() {
            self.placeholder.clone().unwrap_or_default()
        } else {
            self.text.clone()
        }
    }

    /// Insert text at cursor position
    pub fn insert(&mut self, text: &str) {
        if self.readonly {
            return;
        }

        // Delete selection first if any
        if self.selection.has_selection() {
            self.delete_selection();
        }

        // Check max length
        if self.max_length > 0 && self.text.chars().count() + text.chars().count() > self.max_length {
            let allowed = self.max_length.saturating_sub(self.text.chars().count());
            if allowed == 0 {
                return;
            }
            // Take only allowed characters
            let allowed_text: String = text.chars().take(allowed).collect();
            self.insert_at_cursor(&allowed_text);
        } else {
            self.insert_at_cursor(text);
        }
    }

    fn insert_at_cursor(&mut self, text: &str) {
        let pos = self.selection.cursor().offset;
        // Handle grapheme cluster boundaries properly
        let byte_pos = self.offset_to_byte_index(pos);
        self.text.insert_str(byte_pos, text);

        // Move cursor after inserted text
        let new_pos = pos + text.chars().count();
        self.selection = Selection::new(TextPosition::new(new_pos));
    }

    /// Delete character before cursor (backspace)
    pub fn backspace(&mut self) {
        if self.readonly {
            return;
        }

        if self.selection.has_selection() {
            self.delete_selection();
        } else {
            let pos = self.selection.cursor().offset;
            if pos > 0 {
                let byte_pos = self.offset_to_byte_index(pos);
                let prev_byte_pos = self.offset_to_byte_index(pos - 1);
                self.text.replace_range(prev_byte_pos..byte_pos, "");
                self.selection = Selection::new(TextPosition::new(pos - 1));
            }
        }
    }

    /// Delete character after cursor (delete key)
    pub fn delete(&mut self) {
        if self.readonly {
            return;
        }

        if self.selection.has_selection() {
            self.delete_selection();
        } else {
            let pos = self.selection.cursor().offset;
            let char_count = self.text.chars().count();
            if pos < char_count {
                let byte_pos = self.offset_to_byte_index(pos);
                let next_byte_pos = self.offset_to_byte_index(pos + 1);
                self.text.replace_range(byte_pos..next_byte_pos, "");
            }
        }
    }

    /// Delete selected text
    fn delete_selection(&mut self) {
        let (start, end) = self.selection.ordered_positions();
        let start_byte = self.offset_to_byte_index(start.offset);
        let end_byte = self.offset_to_byte_index(end.offset);
        self.text.replace_range(start_byte..end_byte, "");
        self.selection = Selection::new(start);
    }

    /// Move cursor left
    pub fn move_left(&mut self, extend_selection: bool) {
        let pos = self.selection.cursor();
        if pos.offset > 0 {
            let new_pos = TextPosition::new(pos.offset - 1);
            if extend_selection {
                self.selection.extend_to(new_pos);
            } else {
                self.selection = Selection::new(new_pos);
            }
        }
    }

    /// Move cursor right
    pub fn move_right(&mut self, extend_selection: bool) {
        let pos = self.selection.cursor();
        let char_count = self.text.chars().count();
        if pos.offset < char_count {
            let new_pos = TextPosition::new(pos.offset + 1);
            if extend_selection {
                self.selection.extend_to(new_pos);
            } else {
                self.selection = Selection::new(new_pos);
            }
        }
    }

    /// Move cursor to start
    pub fn move_to_start(&mut self, extend_selection: bool) {
        let new_pos = TextPosition::zero();
        if extend_selection {
            self.selection.extend_to(new_pos);
        } else {
            self.selection = Selection::new(new_pos);
        }
    }

    /// Move cursor to end
    pub fn move_to_end(&mut self, extend_selection: bool) {
        let char_count = self.text.chars().count();
        let new_pos = TextPosition::new(char_count);
        if extend_selection {
            self.selection.extend_to(new_pos);
        } else {
            self.selection = Selection::new(new_pos);
        }
    }

    /// Select all text
    pub fn select_all(&mut self) {
        let char_count = self.text.chars().count();
        self.selection = Selection::new_range(
            TextPosition::zero(),
            TextPosition::new(char_count),
        );
    }

    /// Get selected text
    pub fn selected_text(&self) -> &str {
        if !self.selection.has_selection() {
            return "";
        }
        let (start, end) = self.selection.ordered_positions();
        let start_byte = self.offset_to_byte_index(start.offset);
        let end_byte = self.offset_to_byte_index(end.offset);
        &self.text[start_byte..end_byte]
    }

    /// Convert character offset to byte index
    fn offset_to_byte_index(&self, offset: usize) -> usize {
        self.text
            .char_indices()
            .nth(offset)
            .map(|(i, _)| i)
            .unwrap_or(self.text.len())
    }

    /// Get cursor position for rendering
    pub fn cursor_offset(&self) -> usize {
        self.selection.cursor().offset
    }
}

/// Text input manager for handling multiple text fields
pub struct TextInputManager {
    /// Text input states by node ID
    inputs: HashMap<NodeId, TextInputState>,
    /// Currently focused input
    focused_input: Option<NodeId>,
    /// Clipboard provider (system clipboard)
    clipboard: Option<SystemClipboard>,
}

impl std::fmt::Debug for TextInputManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextInputManager")
            .field("inputs", &self.inputs)
            .field("focused_input", &self.focused_input)
            .field("clipboard", &self.clipboard.is_some())
            .finish()
    }
}

impl Default for TextInputManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInputManager {
    /// Create a new text input manager
    pub fn new() -> Self {
        Self {
            inputs: HashMap::new(),
            focused_input: None,
            clipboard: SystemClipboard::new().ok(),
        }
    }

    /// Register a text input node
    pub fn register(&mut self, node: NodeId, initial_text: &str) {
        self.inputs.insert(node, TextInputState::with_text(initial_text));
    }

    /// Unregister a text input node
    pub fn unregister(&mut self, node: NodeId) {
        self.inputs.remove(&node);
        if self.focused_input == Some(node) {
            self.focused_input = None;
        }
    }

    /// Focus a text input
    pub fn focus(&mut self, node: NodeId) {
        // Unfocus previous
        if let Some(prev) = self.focused_input {
            if let Some(input) = self.inputs.get_mut(&prev) {
                input.focused = false;
            }
        }

        // Focus new
        if let Some(input) = self.inputs.get_mut(&node) {
            input.focused = true;
            self.focused_input = Some(node);
        }
    }

    /// Unfocus current input
    pub fn blur(&mut self) {
        if let Some(node) = self.focused_input {
            if let Some(input) = self.inputs.get_mut(&node) {
                input.focused = false;
            }
        }
        self.focused_input = None;
    }

    /// Get the focused input state
    pub fn focused(&self) -> Option<&TextInputState> {
        self.focused_input.and_then(|n| self.inputs.get(&n))
    }

    /// Get mutable focused input state
    pub fn focused_mut(&mut self) -> Option<&mut TextInputState> {
        self.focused_input.and_then(|n| self.inputs.get_mut(&n))
    }

    /// Get input state by node
    pub fn get(&self, node: NodeId) -> Option<&TextInputState> {
        self.inputs.get(&node)
    }

    /// Get mutable input state by node
    pub fn get_mut(&mut self, node: NodeId) -> Option<&mut TextInputState> {
        self.inputs.get_mut(&node)
    }

    /// Handle text input event
    pub fn on_text_input(&mut self, text: &str) {
        if let Some(input) = self.focused_mut() {
            input.insert(text);
        }
    }

    /// Handle key down event
    pub fn on_key_down(&mut self, key: &str, shift: bool, ctrl: bool, meta: bool) -> bool {
        let Some(input) = self.focused_mut() else {
            return false;
        };

        let cmd_or_ctrl = ctrl || meta;

        match key {
            "Backspace" => {
                input.backspace();
                true
            }
            "Delete" => {
                input.delete();
                true
            }
            "ArrowLeft" | "Left" => {
                input.move_left(shift);
                true
            }
            "ArrowRight" | "Right" => {
                input.move_right(shift);
                true
            }
            "Home" => {
                input.move_to_start(shift);
                true
            }
            "End" => {
                input.move_to_end(shift);
                true
            }
            "a" | "A" if cmd_or_ctrl => {
                input.select_all();
                true
            }
            "c" | "C" if cmd_or_ctrl => {
                self.copy();
                true
            }
            "x" | "X" if cmd_or_ctrl => {
                self.cut();
                true
            }
            "v" | "V" if cmd_or_ctrl => {
                self.paste();
                true
            }
            _ => false,
        }
    }

    /// Copy selected text to clipboard
    pub fn copy(&mut self) {
        let Some(input) = self.focused() else {
            return;
        };

        let text = input.selected_text();
        if text.is_empty() {
            return;
        }

        if let Some(clipboard) = &self.clipboard {
            let _ = clipboard.write_text(text);
        }
    }

    /// Cut selected text to clipboard
    pub fn cut(&mut self) {
        // First copy
        if let Some(input) = self.focused() {
            let text = input.selected_text().to_string();
            if !text.is_empty() {
                if let Some(clipboard) = &self.clipboard {
                    let _ = clipboard.write_text(&text);
                }
            }
        }

        // Then delete
        if let Some(input) = self.focused_mut() {
            if input.selection.has_selection() {
                input.backspace();
            }
        }
    }

    /// Paste from clipboard
    pub fn paste(&mut self) {
        let text = self.clipboard.as_ref().and_then(|c| c.read_text().ok().flatten());

        if let (Some(text), Some(input)) = (text, self.focused_mut()) {
            input.insert(&text);
        }
    }

    /// Get all registered input nodes
    pub fn nodes(&self) -> impl Iterator<Item = &NodeId> {
        self.inputs.keys()
    }

    /// Clear all inputs
    pub fn clear(&mut self) {
        self.inputs.clear();
        self.focused_input = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_text_input_insert() {
        let mut input = TextInputState::new();
        input.insert("Hello");
        assert_eq!(input.text, "Hello");
        assert_eq!(input.cursor_offset(), 5);
    }

    #[test]
    fn test_text_input_backspace() {
        let mut input = TextInputState::with_text("Hello");
        input.selection = Selection::new(TextPosition::new(5));
        input.backspace();
        assert_eq!(input.text, "Hell");
    }

    #[test]
    fn test_text_input_delete() {
        let mut input = TextInputState::with_text("Hello");
        input.selection = Selection::new(TextPosition::new(0));
        input.delete();
        assert_eq!(input.text, "ello");
    }

    #[test]
    fn test_text_input_selection() {
        let mut input = TextInputState::with_text("Hello World");
        input.select_all();
        assert_eq!(input.selected_text(), "Hello World");
    }

    #[test]
    fn test_text_input_password_mode() {
        let mut input = TextInputState::with_text("secret");
        input.password = true;
        assert_eq!(input.display_text(), "••••••");
    }

    #[test]
    fn test_text_input_max_length() {
        let mut input = TextInputState::new();
        input.max_length = 5;
        input.insert("Hello World");
        assert_eq!(input.text, "Hello");
    }

    #[test]
    fn test_text_input_readonly() {
        let mut input = TextInputState::with_text("readonly");
        input.readonly = true;
        input.insert("more");
        assert_eq!(input.text, "readonly"); // Should not change
    }

    #[test]
    fn test_manager_focus() {
        let mut manager = TextInputManager::new();
        let node1 = NodeId::new(1);
        let node2 = NodeId::new(2);

        manager.register(node1, "Field 1");
        manager.register(node2, "Field 2");

        manager.focus(node1);
        assert!(manager.get(node1).unwrap().focused);
        assert!(!manager.get(node2).unwrap().focused);

        manager.focus(node2);
        assert!(!manager.get(node1).unwrap().focused);
        assert!(manager.get(node2).unwrap().focused);
    }
}
