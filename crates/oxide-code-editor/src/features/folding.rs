//! Code folding functionality.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};

/// A foldable range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FoldRange {
    /// Start line (0-indexed)
    pub start_line: usize,
    /// End line (0-indexed)
    pub end_line: usize,
    /// Kind of fold
    pub kind: FoldKind,
}

impl FoldRange {
    /// Create a new fold range
    pub fn new(start_line: usize, end_line: usize) -> Self {
        Self {
            start_line,
            end_line,
            kind: FoldKind::Block,
        }
    }

    /// Line count
    pub fn line_count(&self) -> usize {
        self.end_line.saturating_sub(self.start_line) + 1
    }
}

/// Alias for FoldRange
pub type FoldingRange = FoldRange;

/// Kind of foldable region
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FoldKind {
    /// Code block (braces, brackets)
    Block,
    /// Comment block
    Comment,
    /// Import statements
    Imports,
    /// Region marker
    Region,
}

/// Folding state manager
#[derive(Debug, Clone, Default)]
pub struct FoldState {
    /// Lines that are folded
    folded_lines: HashSet<usize>,
    /// Available fold ranges
    ranges: Vec<FoldRange>,
}

impl FoldState {
    /// Create new folding state
    pub fn new() -> Self {
        Self::default()
    }

    /// Set available fold ranges
    pub fn set_ranges(&mut self, ranges: Vec<FoldRange>) {
        self.ranges = ranges;
    }

    /// Toggle fold at line
    pub fn toggle(&mut self, line: usize) {
        if self.folded_lines.contains(&line) {
            self.unfold(line);
        } else {
            self.fold(line);
        }
    }

    /// Fold at line
    pub fn fold(&mut self, line: usize) {
        if let Some(range) = self.range_at(line) {
            self.folded_lines.insert(range.start_line);
        }
    }

    /// Unfold at line
    pub fn unfold(&mut self, line: usize) {
        self.folded_lines.remove(&line);
    }

    /// Unfold at line, returns true if something was unfolded
    pub fn unfold_at(&mut self, line: usize) -> bool {
        self.folded_lines.remove(&line)
    }

    /// Check if line is folded
    pub fn is_folded(&self, line: usize) -> bool {
        self.folded_lines.contains(&line)
    }

    /// Check if line is hidden (inside a fold)
    pub fn is_hidden(&self, line: usize) -> bool {
        for &folded_line in &self.folded_lines {
            if let Some(range) = self.range_at(folded_line) {
                if line > range.start_line && line <= range.end_line {
                    return true;
                }
            }
        }
        false
    }

    /// Get fold range at line
    pub fn range_at(&self, line: usize) -> Option<&FoldRange> {
        self.ranges.iter().find(|r| r.start_line == line)
    }

    /// Fold all
    pub fn fold_all(&mut self) {
        for range in &self.ranges {
            self.folded_lines.insert(range.start_line);
        }
    }

    /// Unfold all
    pub fn unfold_all(&mut self) {
        self.folded_lines.clear();
    }
}

/// Provider for fold ranges
pub trait FoldingProvider: Send + Sync {
    /// Get fold ranges for content
    fn get_fold_ranges(&self, content: &str) -> Vec<FoldRange>;

    /// Get fold range at a specific line from a rope document
    fn get_fold_range(&self, document: &ropey::Rope, line: usize) -> Option<FoldRange> {
        let content = document.to_string();
        let ranges = self.get_fold_ranges(&content);
        ranges.into_iter().find(|r| r.start_line == line)
    }

    /// Get all fold ranges from a rope document
    fn get_all_fold_ranges(&self, document: &ropey::Rope) -> Vec<FoldRange> {
        let content = document.to_string();
        self.get_fold_ranges(&content)
    }
}
