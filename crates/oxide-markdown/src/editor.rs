//! Markdown editor component.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Editor mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EditorMode {
    /// Edit only
    #[default]
    Edit,
    /// Preview only
    Preview,
    /// Split view (edit + preview)
    Split,
}

/// Auto-save configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoSaveConfig {
    /// Enable auto-save
    pub enabled: bool,
    /// Interval
    pub interval: Duration,
}

impl Default for AutoSaveConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            interval: Duration::from_secs(30),
        }
    }
}

/// Keyboard shortcut
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyboardShortcut {
    /// Key code
    pub key: String,
    /// Ctrl/Cmd modifier
    pub ctrl: bool,
    /// Shift modifier
    pub shift: bool,
    /// Alt modifier
    pub alt: bool,
}

impl KeyboardShortcut {
    /// Create a new shortcut
    pub fn new(key: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            ctrl: false,
            shift: false,
            alt: false,
        }
    }

    /// With ctrl modifier
    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }

    /// With shift modifier
    pub fn shift(mut self) -> Self {
        self.shift = true;
        self
    }
}

/// Editor state
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EditorState {
    /// Current content
    pub content: String,
    /// Cursor position
    pub cursor: usize,
    /// Selection start
    pub selection_start: Option<usize>,
    /// Selection end
    pub selection_end: Option<usize>,
    /// Is modified
    pub modified: bool,
}

impl EditorState {
    /// Create new state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set content
    pub fn with_content(mut self, content: impl Into<String>) -> Self {
        self.content = content.into();
        self
    }

    /// Get selected text
    pub fn selected_text(&self) -> Option<&str> {
        match (self.selection_start, self.selection_end) {
            (Some(start), Some(end)) if start != end => {
                let (start, end) = if start < end { (start, end) } else { (end, start) };
                self.content.get(start..end)
            }
            _ => None,
        }
    }
}

/// Editor configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarkdownEditorConfig {
    /// Editor mode
    pub mode: EditorMode,
    /// Show toolbar
    pub toolbar: bool,
    /// Auto-save config
    pub auto_save: AutoSaveConfig,
    /// Line wrapping
    pub line_wrap: bool,
    /// Show line numbers
    pub line_numbers: bool,
}

impl MarkdownEditorConfig {
    /// Create new config
    pub fn new() -> Self {
        Self {
            toolbar: true,
            line_wrap: true,
            ..Default::default()
        }
    }
}

/// Markdown editor component
#[derive(Debug, Clone)]
pub struct MarkdownEditor {
    /// State
    pub state: EditorState,
    /// Configuration
    pub config: MarkdownEditorConfig,
}

impl MarkdownEditor {
    /// Create new editor
    pub fn new() -> Self {
        Self {
            state: EditorState::new(),
            config: MarkdownEditorConfig::new(),
        }
    }

    /// Set value
    pub fn value(mut self, content: impl Into<String>) -> Self {
        self.state.content = content.into();
        self
    }

    /// Set split view
    pub fn split_view(mut self, split: bool) -> Self {
        self.config.mode = if split { EditorMode::Split } else { EditorMode::Edit };
        self
    }

    /// Set toolbar
    pub fn toolbar(mut self, show: bool) -> Self {
        self.config.toolbar = show;
        self
    }

    /// Set auto-save
    pub fn auto_save(mut self, interval: Duration) -> Self {
        self.config.auto_save.enabled = true;
        self.config.auto_save.interval = interval;
        self
    }

    /// Insert text at cursor
    pub fn insert(&mut self, text: &str) {
        self.state.content.insert_str(self.state.cursor, text);
        self.state.cursor += text.len();
        self.state.modified = true;
    }

    /// Get content
    pub fn content(&self) -> &str {
        &self.state.content
    }
}

impl Default for MarkdownEditor {
    fn default() -> Self {
        Self::new()
    }
}
