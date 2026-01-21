//! Fuzzy Matching Algorithm
//!
//! Provides efficient fuzzy string matching with scoring for search and autocomplete.
//! Implements a modified Smith-Waterman algorithm optimized for user input matching.

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// Configuration for fuzzy matching behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyConfig {
    /// Score bonus for consecutive character matches
    pub consecutive_bonus: i32,
    /// Score bonus for matching at word boundaries (start of word, after separator)
    pub word_boundary_bonus: i32,
    /// Score bonus for matching at the start of the string
    pub start_bonus: i32,
    /// Score bonus for exact case match
    pub case_match_bonus: i32,
    /// Penalty for unmatched characters in the pattern
    pub unmatched_penalty: i32,
    /// Score bonus for camelCase boundary matches
    pub camel_case_bonus: i32,
    /// Minimum score threshold (matches below this are filtered out)
    pub min_score: i32,
    /// Whether to perform case-insensitive matching
    pub case_insensitive: bool,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            consecutive_bonus: 15,
            word_boundary_bonus: 10,
            start_bonus: 8,
            case_match_bonus: 3,
            unmatched_penalty: -1,
            camel_case_bonus: 8,
            min_score: 0,
            case_insensitive: true,
        }
    }
}

impl FuzzyConfig {
    /// Create a new fuzzy config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set consecutive match bonus
    pub fn consecutive_bonus(mut self, bonus: i32) -> Self {
        self.consecutive_bonus = bonus;
        self
    }

    /// Set word boundary bonus
    pub fn word_boundary_bonus(mut self, bonus: i32) -> Self {
        self.word_boundary_bonus = bonus;
        self
    }

    /// Set start bonus
    pub fn start_bonus(mut self, bonus: i32) -> Self {
        self.start_bonus = bonus;
        self
    }

    /// Set case match bonus
    pub fn case_match_bonus(mut self, bonus: i32) -> Self {
        self.case_match_bonus = bonus;
        self
    }

    /// Set minimum score threshold
    pub fn min_score(mut self, score: i32) -> Self {
        self.min_score = score;
        self
    }

    /// Set case sensitivity
    pub fn case_insensitive(mut self, insensitive: bool) -> Self {
        self.case_insensitive = insensitive;
        self
    }
}

/// Represents a matched range in the target string
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MatchRange {
    /// Start index (inclusive)
    pub start: usize,
    /// End index (exclusive)
    pub end: usize,
}

impl MatchRange {
    /// Create a new match range
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Get the length of the match
    pub fn len(&self) -> usize {
        self.end - self.start
    }

    /// Check if the range is empty
    pub fn is_empty(&self) -> bool {
        self.start >= self.end
    }
}

/// Result of a fuzzy match operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatch {
    /// The score of the match (higher is better)
    pub score: i32,
    /// The matched ranges in the target string
    pub ranges: Vec<MatchRange>,
    /// The original target string
    pub target: String,
    /// The pattern that was matched
    pub pattern: String,
}

impl FuzzyMatch {
    /// Create a new fuzzy match result
    pub fn new(score: i32, ranges: Vec<MatchRange>, target: String, pattern: String) -> Self {
        Self {
            score,
            ranges,
            target,
            pattern,
        }
    }

    /// Check if the match is valid (has at least one matched range)
    pub fn is_valid(&self) -> bool {
        !self.ranges.is_empty()
    }

    /// Get the matched substrings
    pub fn matched_parts(&self) -> Vec<&str> {
        self.ranges
            .iter()
            .filter_map(|r| self.target.get(r.start..r.end))
            .collect()
    }

    /// Get the highlighted string with markers around matched parts
    pub fn highlight(&self, start_marker: &str, end_marker: &str) -> String {
        if self.ranges.is_empty() {
            return self.target.clone();
        }

        let mut result = String::with_capacity(
            self.target.len() + self.ranges.len() * (start_marker.len() + end_marker.len()),
        );
        let mut last_end = 0;

        for range in &self.ranges {
            // Add unmatched part before this range
            if range.start > last_end {
                result.push_str(&self.target[last_end..range.start]);
            }
            // Add highlighted matched part
            result.push_str(start_marker);
            result.push_str(&self.target[range.start..range.end]);
            result.push_str(end_marker);
            last_end = range.end;
        }

        // Add remaining unmatched part
        if last_end < self.target.len() {
            result.push_str(&self.target[last_end..]);
        }

        result
    }
}

impl PartialEq for FuzzyMatch {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.target == other.target
    }
}

impl Eq for FuzzyMatch {}

impl PartialOrd for FuzzyMatch {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for FuzzyMatch {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher score is better, so reverse the comparison
        other.score.cmp(&self.score).then_with(|| self.target.cmp(&other.target))
    }
}

/// Fuzzy matcher for string matching
#[derive(Debug, Clone)]
pub struct FuzzyMatcher {
    config: FuzzyConfig,
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default configuration
    pub fn new() -> Self {
        Self {
            config: FuzzyConfig::default(),
        }
    }

    /// Create a fuzzy matcher with custom configuration
    pub fn with_config(config: FuzzyConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration
    pub fn config(&self) -> &FuzzyConfig {
        &self.config
    }

    /// Check if a character is a word boundary
    fn is_word_boundary(prev: Option<char>, current: char) -> bool {
        match prev {
            None => true, // Start of string
            Some(p) => {
                // After separator (space, underscore, dash, etc.)
                if p.is_whitespace() || p == '_' || p == '-' || p == '/' || p == '.' {
                    return true;
                }
                // CamelCase boundary (lowercase followed by uppercase)
                if p.is_lowercase() && current.is_uppercase() {
                    return true;
                }
                false
            }
        }
    }

    /// Match a pattern against a target string
    pub fn match_str(&self, pattern: &str, target: &str) -> Option<FuzzyMatch> {
        if pattern.is_empty() {
            return Some(FuzzyMatch::new(0, vec![], target.to_string(), pattern.to_string()));
        }

        if target.is_empty() {
            return None;
        }

        let pattern_chars: Vec<char> = if self.config.case_insensitive {
            pattern.to_lowercase().chars().collect()
        } else {
            pattern.chars().collect()
        };

        let target_chars: Vec<char> = target.chars().collect();
        let target_lower: Vec<char> = if self.config.case_insensitive {
            target.to_lowercase().chars().collect()
        } else {
            target_chars.clone()
        };

        let mut score = 0i32;
        let mut ranges: Vec<MatchRange> = Vec::new();
        let mut pattern_idx = 0;
        let mut consecutive = 0;
        let mut last_match_idx: Option<usize> = None;
        let mut current_range_start: Option<usize> = None;

        // Byte offset tracking for proper string slicing
        let mut byte_offsets: Vec<usize> = Vec::with_capacity(target_chars.len() + 1);
        let mut offset = 0;
        for c in &target_chars {
            byte_offsets.push(offset);
            offset += c.len_utf8();
        }
        byte_offsets.push(offset);

        for (target_idx, &target_char) in target_lower.iter().enumerate() {
            if pattern_idx >= pattern_chars.len() {
                break;
            }

            let pattern_char = pattern_chars[pattern_idx];

            if target_char == pattern_char {
                // Character matches
                let mut char_score = 1;

                // Consecutive bonus
                if let Some(last) = last_match_idx {
                    if target_idx == last + 1 {
                        consecutive += 1;
                        char_score += self.config.consecutive_bonus * consecutive.min(5);
                    } else {
                        consecutive = 0;
                    }
                }

                // Word boundary bonus
                let prev_char = if target_idx > 0 {
                    Some(target_chars[target_idx - 1])
                } else {
                    None
                };
                if Self::is_word_boundary(prev_char, target_chars[target_idx]) {
                    char_score += self.config.word_boundary_bonus;
                }

                // Start bonus
                if target_idx == 0 {
                    char_score += self.config.start_bonus;
                }

                // Case match bonus (only when case-insensitive matching is on)
                if self.config.case_insensitive
                    && target_chars[target_idx] == pattern.chars().nth(pattern_idx).unwrap_or(' ')
                {
                    char_score += self.config.case_match_bonus;
                }

                // CamelCase bonus
                if target_idx > 0 {
                    let prev = target_chars[target_idx - 1];
                    if prev.is_lowercase() && target_chars[target_idx].is_uppercase() {
                        char_score += self.config.camel_case_bonus;
                    }
                }

                score += char_score;

                // Track ranges
                if current_range_start.is_none() {
                    current_range_start = Some(byte_offsets[target_idx]);
                }

                // Check if next pattern char won't match next target char (end of consecutive)
                let is_last_pattern = pattern_idx + 1 >= pattern_chars.len();
                let next_target_matches = if !is_last_pattern && target_idx + 1 < target_lower.len()
                {
                    target_lower[target_idx + 1] == pattern_chars[pattern_idx + 1]
                } else {
                    false
                };

                if is_last_pattern || !next_target_matches {
                    // End current range if next won't match
                    if let Some(start) = current_range_start.take() {
                        ranges.push(MatchRange::new(start, byte_offsets[target_idx + 1]));
                    }
                }

                last_match_idx = Some(target_idx);
                pattern_idx += 1;
            } else {
                // No match - end current range if any
                if let Some(start) = current_range_start.take() {
                    if let Some(last) = last_match_idx {
                        ranges.push(MatchRange::new(start, byte_offsets[last + 1]));
                    }
                }
                score += self.config.unmatched_penalty;
            }
        }

        // Finalize any remaining range
        if let Some(start) = current_range_start {
            if let Some(last) = last_match_idx {
                ranges.push(MatchRange::new(start, byte_offsets[last + 1]));
            }
        }

        // Check if all pattern characters were matched
        if pattern_idx < pattern_chars.len() {
            return None;
        }

        // Consolidate adjacent ranges
        let consolidated_ranges = Self::consolidate_ranges(ranges);

        // Check minimum score
        if score < self.config.min_score {
            return None;
        }

        Some(FuzzyMatch::new(
            score,
            consolidated_ranges,
            target.to_string(),
            pattern.to_string(),
        ))
    }

    /// Consolidate adjacent ranges into single ranges
    fn consolidate_ranges(ranges: Vec<MatchRange>) -> Vec<MatchRange> {
        if ranges.is_empty() {
            return ranges;
        }

        let mut consolidated = Vec::with_capacity(ranges.len());
        let mut current = ranges[0];

        for range in ranges.into_iter().skip(1) {
            if range.start <= current.end {
                // Extend current range
                current = MatchRange::new(current.start, range.end.max(current.end));
            } else {
                consolidated.push(current);
                current = range;
            }
        }
        consolidated.push(current);

        consolidated
    }

    /// Match a pattern against multiple targets and return sorted results
    pub fn match_all<'a, I, S>(&self, pattern: &str, targets: I) -> Vec<FuzzyMatch>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let mut matches: Vec<FuzzyMatch> = targets
            .into_iter()
            .filter_map(|t| self.match_str(pattern, t.as_ref()))
            .collect();

        matches.sort();
        matches
    }

    /// Match pattern against targets with custom scoring
    pub fn match_with_scorer<'a, T, F>(
        &self,
        pattern: &str,
        items: &'a [T],
        text_fn: F,
    ) -> Vec<(&'a T, FuzzyMatch)>
    where
        F: Fn(&T) -> &str,
    {
        let mut results: Vec<(&'a T, FuzzyMatch)> = items
            .iter()
            .filter_map(|item| {
                self.match_str(pattern, text_fn(item))
                    .map(|m| (item, m))
            })
            .collect();

        results.sort_by(|a, b| a.1.cmp(&b.1));
        results
    }
}

/// Exact prefix matching for autocomplete scenarios
#[derive(Debug, Clone)]
pub struct PrefixMatcher {
    case_insensitive: bool,
}

impl Default for PrefixMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl PrefixMatcher {
    /// Create a new prefix matcher
    pub fn new() -> Self {
        Self {
            case_insensitive: true,
        }
    }

    /// Set case sensitivity
    pub fn case_insensitive(mut self, insensitive: bool) -> Self {
        self.case_insensitive = insensitive;
        self
    }

    /// Check if target starts with pattern
    pub fn matches(&self, pattern: &str, target: &str) -> bool {
        if self.case_insensitive {
            target.to_lowercase().starts_with(&pattern.to_lowercase())
        } else {
            target.starts_with(pattern)
        }
    }

    /// Get match range if pattern matches
    pub fn match_range(&self, pattern: &str, target: &str) -> Option<MatchRange> {
        if self.matches(pattern, target) {
            Some(MatchRange::new(0, pattern.len()))
        } else {
            None
        }
    }

    /// Filter targets by prefix
    pub fn filter<'a, I, S>(&self, pattern: &str, targets: I) -> Vec<S>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        targets
            .into_iter()
            .filter(|t| self.matches(pattern, t.as_ref()))
            .collect()
    }
}

/// Contains matching for substring search
#[derive(Debug, Clone)]
pub struct ContainsMatcher {
    case_insensitive: bool,
}

impl Default for ContainsMatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainsMatcher {
    /// Create a new contains matcher
    pub fn new() -> Self {
        Self {
            case_insensitive: true,
        }
    }

    /// Set case sensitivity
    pub fn case_insensitive(mut self, insensitive: bool) -> Self {
        self.case_insensitive = insensitive;
        self
    }

    /// Check if target contains pattern
    pub fn matches(&self, pattern: &str, target: &str) -> bool {
        if self.case_insensitive {
            target.to_lowercase().contains(&pattern.to_lowercase())
        } else {
            target.contains(pattern)
        }
    }

    /// Find all match ranges
    pub fn find_ranges(&self, pattern: &str, target: &str) -> Vec<MatchRange> {
        if pattern.is_empty() || target.is_empty() {
            return vec![];
        }

        let search_target = if self.case_insensitive {
            target.to_lowercase()
        } else {
            target.to_string()
        };
        let search_pattern = if self.case_insensitive {
            pattern.to_lowercase()
        } else {
            pattern.to_string()
        };

        search_target
            .match_indices(&search_pattern)
            .map(|(idx, matched)| MatchRange::new(idx, idx + matched.len()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_match_exact() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("test", "test").unwrap();
        assert!(result.score > 0);
        assert_eq!(result.ranges.len(), 1);
    }

    #[test]
    fn test_fuzzy_match_prefix() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("te", "test").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_fuzzy_match_scattered() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("tst", "test").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_fuzzy_match_case_insensitive() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("TEST", "test").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_fuzzy_match_no_match() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("xyz", "test");
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_empty_pattern() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("", "test").unwrap();
        assert_eq!(result.score, 0);
        assert!(result.ranges.is_empty());
    }

    #[test]
    fn test_fuzzy_match_empty_target() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("test", "");
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_word_boundary_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.match_str("gc", "getCommand").unwrap();
        let result2 = matcher.match_str("gc", "ageCount").unwrap();
        // Word boundary match should score higher
        assert!(result1.score > result2.score);
    }

    #[test]
    fn test_fuzzy_match_camel_case() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("gc", "getCommand").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_fuzzy_match_consecutive_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.match_str("abc", "abcdef").unwrap();
        let result2 = matcher.match_str("adf", "abcdef").unwrap();
        // Consecutive match should score higher
        assert!(result1.score > result2.score);
    }

    #[test]
    fn test_fuzzy_match_start_bonus() {
        let matcher = FuzzyMatcher::new();
        let result1 = matcher.match_str("te", "test").unwrap();
        let result2 = matcher.match_str("st", "test").unwrap();
        // Start match should score higher
        assert!(result1.score > result2.score);
    }

    #[test]
    fn test_fuzzy_match_highlight() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("test", "testing").unwrap();
        let highlighted = result.highlight("<b>", "</b>");
        assert_eq!(highlighted, "<b>test</b>ing");
    }

    #[test]
    fn test_fuzzy_match_all() {
        let matcher = FuzzyMatcher::new();
        let targets = vec!["test", "testing", "contest", "attest", "best"];
        let results = matcher.match_all("test", targets);
        assert_eq!(results.len(), 4); // "best" doesn't match
        assert_eq!(results[0].target, "test"); // Exact match should be first
    }

    #[test]
    fn test_fuzzy_match_sorting() {
        let matcher = FuzzyMatcher::new();
        let targets = vec!["testing", "test", "Contest"];
        let results = matcher.match_all("test", targets);
        // "test" (exact match) should come before "testing" (prefix) before "Contest"
        assert_eq!(results[0].target, "test");
    }

    #[test]
    fn test_fuzzy_config_min_score() {
        let config = FuzzyConfig::new().min_score(100);
        let matcher = FuzzyMatcher::with_config(config);
        let result = matcher.match_str("t", "test");
        // Single char match unlikely to reach score of 100
        assert!(result.is_none());
    }

    #[test]
    fn test_fuzzy_match_unicode() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("cafe", "cafe").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_prefix_matcher() {
        let matcher = PrefixMatcher::new();
        assert!(matcher.matches("te", "test"));
        assert!(matcher.matches("TE", "test"));
        assert!(!matcher.matches("st", "test"));
    }

    #[test]
    fn test_prefix_matcher_case_sensitive() {
        let matcher = PrefixMatcher::new().case_insensitive(false);
        assert!(matcher.matches("te", "test"));
        assert!(!matcher.matches("TE", "test"));
    }

    #[test]
    fn test_contains_matcher() {
        let matcher = ContainsMatcher::new();
        assert!(matcher.matches("es", "test"));
        assert!(matcher.matches("ES", "test"));
        assert!(!matcher.matches("xyz", "test"));
    }

    #[test]
    fn test_contains_matcher_find_ranges() {
        let matcher = ContainsMatcher::new();
        let ranges = matcher.find_ranges("test", "this is a test string with test");
        assert_eq!(ranges.len(), 2);
        assert_eq!(ranges[0].start, 10);
        // "this is a test string with test"
        //  0123456789012345678901234567890
        //            ^         ^
        //           10        27
        assert_eq!(ranges[1].start, 27);
    }

    #[test]
    fn test_match_range() {
        let range = MatchRange::new(5, 10);
        assert_eq!(range.len(), 5);
        assert!(!range.is_empty());

        let empty_range = MatchRange::new(5, 5);
        assert!(empty_range.is_empty());
    }

    #[test]
    fn test_fuzzy_match_with_scorer() {
        #[derive(Debug)]
        struct Item {
            name: String,
            id: u32,
        }

        let items = vec![
            Item { name: "Apple".to_string(), id: 1 },
            Item { name: "Banana".to_string(), id: 2 },
            Item { name: "Apricot".to_string(), id: 3 },
        ];

        let matcher = FuzzyMatcher::new();
        let results = matcher.match_with_scorer("ap", &items, |item| &item.name);
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0.name, "Apple");
    }

    #[test]
    fn test_fuzzy_match_consolidate_ranges() {
        let ranges = vec![
            MatchRange::new(0, 2),
            MatchRange::new(2, 4),
            MatchRange::new(6, 8),
        ];
        let consolidated = FuzzyMatcher::consolidate_ranges(ranges);
        assert_eq!(consolidated.len(), 2);
        assert_eq!(consolidated[0].start, 0);
        assert_eq!(consolidated[0].end, 4);
    }

    #[test]
    fn test_fuzzy_match_partial_pattern() {
        let matcher = FuzzyMatcher::new();
        // Pattern "fltr" should match "filter"
        let result = matcher.match_str("fltr", "filter").unwrap();
        assert!(result.score > 0);
    }

    #[test]
    fn test_fuzzy_match_underscore_boundary() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("gc", "get_command").unwrap();
        assert!(result.score > 0);
        // Word boundary after underscore should give bonus
    }

    #[test]
    fn test_fuzzy_match_slash_boundary() {
        let matcher = FuzzyMatcher::new();
        let result = matcher.match_str("uf", "user/file").unwrap();
        assert!(result.score > 0);
    }
}
