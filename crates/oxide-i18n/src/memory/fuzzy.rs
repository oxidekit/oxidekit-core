//! Fuzzy string matching for translation memory
//!
//! Uses various algorithms to find similar strings for translation suggestions.

use serde::{Deserialize, Serialize};
use strsim::{jaro_winkler, normalized_levenshtein, sorensen_dice};

/// Quality of a fuzzy match
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchQuality {
    /// 100% exact match
    Exact,
    /// Very high similarity (90-99%)
    VeryHigh,
    /// High similarity (75-89%)
    High,
    /// Medium similarity (50-74%)
    Medium,
    /// Low similarity (25-49%)
    Low,
    /// Very low similarity (<25%)
    VeryLow,
}

impl MatchQuality {
    /// Create from a similarity score (0.0 - 1.0)
    pub fn from_score(score: f64) -> Self {
        if score >= 1.0 {
            MatchQuality::Exact
        } else if score >= 0.9 {
            MatchQuality::VeryHigh
        } else if score >= 0.75 {
            MatchQuality::High
        } else if score >= 0.5 {
            MatchQuality::Medium
        } else if score >= 0.25 {
            MatchQuality::Low
        } else {
            MatchQuality::VeryLow
        }
    }

    /// Get minimum acceptable quality for suggestions
    pub fn is_acceptable(&self) -> bool {
        matches!(
            self,
            MatchQuality::Exact
                | MatchQuality::VeryHigh
                | MatchQuality::High
                | MatchQuality::Medium
        )
    }
}

/// A fuzzy match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyMatch {
    /// The matched source text
    pub matched_source: String,
    /// The translation of the matched text
    pub translation: String,
    /// Similarity score (0.0 - 1.0)
    pub score: f64,
    /// Match quality category
    pub quality: MatchQuality,
    /// Which algorithm produced this match
    pub algorithm: MatchAlgorithm,
    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
}

/// Algorithm used for matching
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchAlgorithm {
    /// Exact string match
    Exact,
    /// Levenshtein distance
    Levenshtein,
    /// Jaro-Winkler similarity
    JaroWinkler,
    /// Sorensen-Dice coefficient
    SorensenDice,
    /// Combined/weighted score
    Combined,
}

impl MatchAlgorithm {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            MatchAlgorithm::Exact => "Exact Match",
            MatchAlgorithm::Levenshtein => "Levenshtein Distance",
            MatchAlgorithm::JaroWinkler => "Jaro-Winkler",
            MatchAlgorithm::SorensenDice => "Sorensen-Dice",
            MatchAlgorithm::Combined => "Combined Score",
        }
    }
}

/// Configuration for fuzzy matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzyConfig {
    /// Minimum score to consider a match
    pub min_score: f64,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Weight for Levenshtein in combined score
    pub levenshtein_weight: f64,
    /// Weight for Jaro-Winkler in combined score
    pub jaro_winkler_weight: f64,
    /// Weight for Sorensen-Dice in combined score
    pub sorensen_dice_weight: f64,
    /// Whether to normalize text before matching
    pub normalize_text: bool,
    /// Whether to ignore case
    pub ignore_case: bool,
    /// Whether to ignore whitespace differences
    pub ignore_whitespace: bool,
}

impl Default for FuzzyConfig {
    fn default() -> Self {
        Self {
            min_score: 0.5,
            max_results: 10,
            levenshtein_weight: 0.3,
            jaro_winkler_weight: 0.4,
            sorensen_dice_weight: 0.3,
            normalize_text: true,
            ignore_case: true,
            ignore_whitespace: true,
        }
    }
}

/// Fuzzy string matcher
#[derive(Debug)]
pub struct FuzzyMatcher {
    config: FuzzyConfig,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default config
    pub fn new() -> Self {
        Self {
            config: FuzzyConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: FuzzyConfig) -> Self {
        Self { config }
    }

    /// Calculate similarity between two strings
    pub fn similarity(&self, a: &str, b: &str) -> f64 {
        let (a, b) = if self.config.normalize_text {
            (self.normalize(a), self.normalize(b))
        } else {
            (a.to_string(), b.to_string())
        };

        if a == b {
            return 1.0;
        }

        let lev = normalized_levenshtein(&a, &b);
        let jaro = jaro_winkler(&a, &b);
        let dice = sorensen_dice(&a, &b);

        // Weighted average
        let score = lev * self.config.levenshtein_weight
            + jaro * self.config.jaro_winkler_weight
            + dice * self.config.sorensen_dice_weight;

        score.clamp(0.0, 1.0)
    }

    /// Calculate individual algorithm scores
    pub fn detailed_similarity(&self, a: &str, b: &str) -> DetailedScore {
        let (a, b) = if self.config.normalize_text {
            (self.normalize(a), self.normalize(b))
        } else {
            (a.to_string(), b.to_string())
        };

        let exact = a == b;
        let levenshtein = normalized_levenshtein(&a, &b);
        let jaro_winkler = jaro_winkler(&a, &b);
        let sorensen_dice = sorensen_dice(&a, &b);

        let combined = levenshtein * self.config.levenshtein_weight
            + jaro_winkler * self.config.jaro_winkler_weight
            + sorensen_dice * self.config.sorensen_dice_weight;

        DetailedScore {
            exact,
            levenshtein,
            jaro_winkler,
            sorensen_dice,
            combined: combined.clamp(0.0, 1.0),
            edit_distance: strsim::levenshtein(&a, &b),
        }
    }

    /// Find matches in a list of candidates
    pub fn find_matches<'a>(
        &self,
        query: &str,
        candidates: impl Iterator<Item = (&'a str, &'a str)>,
    ) -> Vec<FuzzyMatch> {
        let mut matches: Vec<FuzzyMatch> = candidates
            .filter_map(|(source, translation)| {
                let score = self.similarity(query, source);
                if score >= self.config.min_score {
                    Some(FuzzyMatch {
                        matched_source: source.to_string(),
                        translation: translation.to_string(),
                        score,
                        quality: MatchQuality::from_score(score),
                        algorithm: if score >= 1.0 {
                            MatchAlgorithm::Exact
                        } else {
                            MatchAlgorithm::Combined
                        },
                        context: None,
                    })
                } else {
                    None
                }
            })
            .collect();

        // Sort by score (descending)
        matches.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        matches.truncate(self.config.max_results);

        matches
    }

    /// Normalize text for comparison
    fn normalize(&self, text: &str) -> String {
        let mut result = text.to_string();

        if self.config.ignore_case {
            result = result.to_lowercase();
        }

        if self.config.ignore_whitespace {
            result = result.split_whitespace().collect::<Vec<_>>().join(" ");
        }

        result.trim().to_string()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed similarity scores
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailedScore {
    /// Whether it's an exact match
    pub exact: bool,
    /// Normalized Levenshtein similarity
    pub levenshtein: f64,
    /// Jaro-Winkler similarity
    pub jaro_winkler: f64,
    /// Sorensen-Dice coefficient
    pub sorensen_dice: f64,
    /// Combined weighted score
    pub combined: f64,
    /// Raw edit distance
    pub edit_distance: usize,
}

/// Segment text for better matching
pub fn segment_text(text: &str) -> Vec<String> {
    // Split on common delimiters while preserving placeholders
    let mut segments = Vec::new();
    let mut current = String::new();
    let mut in_placeholder = false;

    for c in text.chars() {
        if c == '{' {
            if !current.is_empty() {
                segments.push(current.clone());
                current.clear();
            }
            in_placeholder = true;
            current.push(c);
        } else if c == '}' {
            current.push(c);
            if in_placeholder {
                segments.push(current.clone());
                current.clear();
                in_placeholder = false;
            }
        } else if !in_placeholder && (c == '.' || c == '!' || c == '?' || c == ',' || c == ';') {
            current.push(c);
            segments.push(current.clone());
            current.clear();
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        segments.push(current);
    }

    segments
}

/// Extract placeholders from text
pub fn extract_placeholders(text: &str) -> Vec<String> {
    let re = regex::Regex::new(r"\{([^}]+)\}").unwrap();
    re.captures_iter(text)
        .map(|cap| cap[0].to_string())
        .collect()
}

/// Check if two texts have compatible placeholders
pub fn placeholders_compatible(source: &str, translation: &str) -> bool {
    let source_ph: std::collections::HashSet<_> = extract_placeholders(source).into_iter().collect();
    let trans_ph: std::collections::HashSet<_> = extract_placeholders(translation).into_iter().collect();
    source_ph == trans_ph
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_similarity() {
        let matcher = FuzzyMatcher::new();

        // Exact match
        assert_eq!(matcher.similarity("hello", "hello"), 1.0);

        // Very similar
        let score = matcher.similarity("hello", "helo");
        assert!(score > 0.8);

        // Different
        let score = matcher.similarity("hello", "world");
        assert!(score < 0.5);
    }

    #[test]
    fn test_fuzzy_matching() {
        let matcher = FuzzyMatcher::new();

        let candidates = vec![
            ("Hello world", "Hallo Welt"),
            ("Hello there", "Hallo da"),
            ("Goodbye", "Auf Wiedersehen"),
        ];

        let matches =
            matcher.find_matches("Hello world!", candidates.iter().map(|(s, t)| (*s, *t)));

        assert!(!matches.is_empty());
        assert_eq!(matches[0].matched_source, "Hello world");
    }

    #[test]
    fn test_placeholder_extraction() {
        let text = "Hello {name}, you have {count} messages";
        let placeholders = extract_placeholders(text);
        assert_eq!(placeholders.len(), 2);
        assert!(placeholders.contains(&"{name}".to_string()));
        assert!(placeholders.contains(&"{count}".to_string()));
    }

    #[test]
    fn test_placeholder_compatibility() {
        assert!(placeholders_compatible(
            "Hello {name}",
            "Hallo {name}"
        ));
        assert!(!placeholders_compatible(
            "Hello {name}",
            "Hallo {user}"
        ));
    }

    #[test]
    fn test_match_quality() {
        assert_eq!(MatchQuality::from_score(1.0), MatchQuality::Exact);
        assert_eq!(MatchQuality::from_score(0.95), MatchQuality::VeryHigh);
        assert_eq!(MatchQuality::from_score(0.8), MatchQuality::High);
        assert_eq!(MatchQuality::from_score(0.6), MatchQuality::Medium);
        assert_eq!(MatchQuality::from_score(0.3), MatchQuality::Low);
        assert_eq!(MatchQuality::from_score(0.1), MatchQuality::VeryLow);
    }
}
