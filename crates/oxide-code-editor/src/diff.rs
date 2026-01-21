//! Diff view for comparing code.

use serde::{Deserialize, Serialize};

/// Type of change in a diff
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiffChange {
    /// Line was added
    Added,
    /// Line was removed
    Removed,
    /// Line was modified
    Modified,
    /// Line is unchanged
    Unchanged,
}

/// A line in the diff view
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffLine {
    /// Line number in the old file (if present)
    pub old_line: Option<usize>,
    /// Line number in the new file (if present)
    pub new_line: Option<usize>,
    /// Content of the line
    pub content: String,
    /// Type of change
    pub change: DiffChange,
}

impl DiffLine {
    /// Create a new diff line
    pub fn new(content: impl Into<String>, change: DiffChange) -> Self {
        Self {
            old_line: None,
            new_line: None,
            content: content.into(),
            change,
        }
    }
}

/// Configuration for the diff view
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DiffViewConfig {
    /// Show line numbers
    pub line_numbers: bool,
    /// Show inline changes
    pub inline_changes: bool,
    /// Side-by-side view
    pub side_by_side: bool,
    /// Context lines around changes
    pub context_lines: usize,
}

/// Diff view component
#[derive(Debug, Clone)]
pub struct DiffView {
    /// Lines in the diff
    pub lines: Vec<DiffLine>,
    /// Configuration
    pub config: DiffViewConfig,
}

impl DiffView {
    /// Create a new diff view
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            config: DiffViewConfig::default(),
        }
    }

    /// Set the lines
    pub fn lines(mut self, lines: Vec<DiffLine>) -> Self {
        self.lines = lines;
        self
    }

    /// Set configuration
    pub fn config(mut self, config: DiffViewConfig) -> Self {
        self.config = config;
        self
    }

    /// Get added line count
    pub fn added_count(&self) -> usize {
        self.lines.iter().filter(|l| l.change == DiffChange::Added).count()
    }

    /// Get removed line count
    pub fn removed_count(&self) -> usize {
        self.lines.iter().filter(|l| l.change == DiffChange::Removed).count()
    }
}

impl Default for DiffView {
    fn default() -> Self {
        Self::new()
    }
}
