//! Autocomplete functionality.

use serde::{Deserialize, Serialize};

/// Kind of completion item
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompletionKind {
    /// Keyword
    Keyword,
    /// Variable
    Variable,
    /// Function
    Function,
    /// Method
    Method,
    /// Field
    Field,
    /// Class/Type
    Class,
    /// Module
    Module,
    /// Property
    Property,
    /// Constant
    Constant,
    /// Snippet
    Snippet,
    /// Text
    Text,
}

/// A completion item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletionItem {
    /// Display label
    pub label: String,
    /// Kind of item
    pub kind: CompletionKind,
    /// Text to insert
    pub insert_text: String,
    /// Detail text
    pub detail: Option<String>,
    /// Documentation
    pub documentation: Option<String>,
}

impl CompletionItem {
    /// Create a new completion item
    pub fn new(label: impl Into<String>, kind: CompletionKind) -> Self {
        let label = label.into();
        Self {
            insert_text: label.clone(),
            label,
            kind,
            detail: None,
            documentation: None,
        }
    }
}

/// Trigger for autocompletion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompletionTrigger {
    /// Manual invocation
    Manual,
    /// Triggered by character
    Character(char),
    /// Triggered by typing
    Automatic,
}

/// Completion provider trait
pub trait CompletionProvider: Send + Sync {
    /// Get completions at position
    fn get_completions(&self, content: &str, position: usize) -> Vec<CompletionItem>;
}

/// Autocomplete system
#[derive(Debug, Default)]
pub struct AutoComplete {
    /// Whether autocomplete is active
    pub active: bool,
    /// Current items
    pub items: Vec<CompletionItem>,
    /// Selected index
    pub selected_index: usize,
}

impl AutoComplete {
    /// Create new autocomplete
    pub fn new() -> Self {
        Self::default()
    }

    /// Show completions
    pub fn show(&mut self, items: Vec<CompletionItem>) {
        self.items = items;
        self.active = !self.items.is_empty();
        self.selected_index = 0;
    }

    /// Hide completions
    pub fn hide(&mut self) {
        self.active = false;
        self.items.clear();
    }

    /// Select next item
    pub fn select_next(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.items.len();
        }
    }

    /// Select previous item
    pub fn select_previous(&mut self) {
        if !self.items.is_empty() {
            self.selected_index = self.selected_index.checked_sub(1).unwrap_or(self.items.len() - 1);
        }
    }

    /// Get selected item
    pub fn selected(&self) -> Option<&CompletionItem> {
        self.items.get(self.selected_index)
    }
}
