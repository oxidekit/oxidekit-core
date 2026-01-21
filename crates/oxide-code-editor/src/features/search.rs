//! Find and replace functionality.

use crate::Range;
use serde::{Deserialize, Serialize};

/// Find options
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FindOptions {
    /// Case sensitive search
    pub case_sensitive: bool,
    /// Whole word only
    pub whole_word: bool,
    /// Use regex
    pub regex: bool,
    /// Search in selection only
    pub in_selection: bool,
}

impl FindOptions {
    /// Create new find options
    pub fn new() -> Self {
        Self::default()
    }

    /// Set case sensitive
    pub fn case_sensitive(mut self, value: bool) -> Self {
        self.case_sensitive = value;
        self
    }

    /// Set whole word
    pub fn whole_word(mut self, value: bool) -> Self {
        self.whole_word = value;
        self
    }

    /// Set regex mode
    pub fn regex(mut self, value: bool) -> Self {
        self.regex = value;
        self
    }
}

/// A search match
#[derive(Debug, Clone)]
pub struct SearchMatch {
    /// Range of the match
    pub range: Range,
    /// Matched text
    pub text: String,
    /// Index of this match
    pub index: usize,
}

impl SearchMatch {
    /// Create a new search match
    pub fn new(range: Range, text: String, index: usize) -> Self {
        Self { range, text, index }
    }
}

/// Result of a find operation
#[derive(Debug, Clone, Default)]
pub struct FindResult {
    /// All matches
    pub matches: Vec<SearchMatch>,
    /// Current match index
    pub current_index: Option<usize>,
}

impl FindResult {
    /// Create new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Total match count
    pub fn count(&self) -> usize {
        self.matches.len()
    }

    /// Is empty
    pub fn is_empty(&self) -> bool {
        self.matches.is_empty()
    }

    /// Get current match
    pub fn current(&self) -> Option<&SearchMatch> {
        self.current_index.and_then(|i| self.matches.get(i))
    }

    /// Go to next match
    pub fn next(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = Some(match self.current_index {
            Some(i) => (i + 1) % self.matches.len(),
            None => 0,
        });
        self.current()
    }

    /// Go to previous match
    pub fn previous(&mut self) -> Option<&SearchMatch> {
        if self.matches.is_empty() {
            return None;
        }
        self.current_index = Some(match self.current_index {
            Some(i) => i.checked_sub(1).unwrap_or(self.matches.len() - 1),
            None => self.matches.len() - 1,
        });
        self.current()
    }
}

/// Search and replace manager
#[derive(Debug, Clone, Default)]
pub struct SearchReplace {
    /// Search query
    pub query: String,
    /// Replacement text
    pub replacement: String,
    /// Find options
    pub options: FindOptions,
    /// Current results
    pub results: FindResult,
}

impl SearchReplace {
    /// Create new search/replace
    pub fn new() -> Self {
        Self::default()
    }

    /// Set query
    pub fn query(mut self, query: impl Into<String>) -> Self {
        self.query = query.into();
        self
    }

    /// Set replacement
    pub fn replacement(mut self, replacement: impl Into<String>) -> Self {
        self.replacement = replacement.into();
        self
    }

    /// Set options
    pub fn options(mut self, options: FindOptions) -> Self {
        self.options = options;
        self
    }

    /// Clear results
    pub fn clear(&mut self) {
        self.results = FindResult::new();
    }

    /// Find all occurrences in a rope document
    pub fn find(&mut self, document: &ropey::Rope, query: &str) -> Vec<crate::Range> {
        self.query = query.to_string();
        let content = document.to_string();
        let mut matches = Vec::new();

        if query.is_empty() {
            return matches;
        }

        let search_content = if self.options.case_sensitive {
            content.clone()
        } else {
            content.to_lowercase()
        };

        let search_query = if self.options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        let mut offset = 0;
        while let Some(idx) = search_content[offset..].find(&search_query) {
            let start_idx = offset + idx;
            let end_idx = start_idx + query.len();

            // Convert character index to position
            let start_pos = self.char_index_to_position(&content, start_idx);
            let end_pos = self.char_index_to_position(&content, end_idx);

            matches.push(crate::Range::new(start_pos, end_pos));
            offset = start_idx + 1;
        }

        // Update results
        self.results.matches = matches
            .iter()
            .enumerate()
            .map(|(i, range)| SearchMatch::new(*range, query.to_string(), i))
            .collect();

        if !matches.is_empty() {
            self.results.current_index = Some(0);
        }

        matches
    }

    /// Find with specific options
    pub fn find_with_options(
        &mut self,
        document: &ropey::Rope,
        query: &str,
        options: &FindOptions,
    ) -> Result<Vec<crate::Range>, crate::EditorError> {
        self.options = options.clone();
        Ok(self.find(document, query))
    }

    /// Convert character index to position
    fn char_index_to_position(&self, content: &str, char_idx: usize) -> crate::Position {
        let mut line = 0;
        let mut col = 0;

        for (i, c) in content.chars().enumerate() {
            if i == char_idx {
                break;
            }
            if c == '\n' {
                line += 1;
                col = 0;
            } else {
                col += 1;
            }
        }

        crate::Position::new(line, col)
    }
}
