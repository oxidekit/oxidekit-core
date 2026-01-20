//! Translation suggestion engine
//!
//! Combines translation memory with fuzzy matching to provide
//! intelligent suggestions for translations.

use super::database::{MemoryEntry, TranslationMemory};
use super::fuzzy::{FuzzyMatch, FuzzyMatcher, MatchQuality};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Source of a translation suggestion
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionSource {
    /// Exact match from translation memory
    TranslationMemory,
    /// Fuzzy match from translation memory
    FuzzyMatch,
    /// Machine translation
    MachineTranslation,
    /// Glossary term
    Glossary,
    /// Previous version of the same key
    PreviousVersion,
}

impl SuggestionSource {
    /// Get human-readable name
    pub fn name(&self) -> &'static str {
        match self {
            SuggestionSource::TranslationMemory => "Translation Memory (Exact)",
            SuggestionSource::FuzzyMatch => "Translation Memory (Fuzzy)",
            SuggestionSource::MachineTranslation => "Machine Translation",
            SuggestionSource::Glossary => "Glossary",
            SuggestionSource::PreviousVersion => "Previous Version",
        }
    }

    /// Get confidence weight for this source
    pub fn confidence_weight(&self) -> f64 {
        match self {
            SuggestionSource::TranslationMemory => 1.0,
            SuggestionSource::FuzzyMatch => 0.9,
            SuggestionSource::Glossary => 0.95,
            SuggestionSource::PreviousVersion => 0.85,
            SuggestionSource::MachineTranslation => 0.7,
        }
    }
}

/// A translation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Suggestion {
    /// Suggested translation
    pub translation: String,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f64,
    /// Source of the suggestion
    pub source: SuggestionSource,
    /// Match quality (for fuzzy matches)
    pub quality: MatchQuality,
    /// Source text that was matched (for TM matches)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_source: Option<String>,
    /// Context information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Project the suggestion came from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Differences from original (for fuzzy)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub differences: Option<Vec<String>>,
}

impl Suggestion {
    /// Create a new suggestion
    pub fn new(translation: impl Into<String>, confidence: f64, source: SuggestionSource) -> Self {
        Self {
            translation: translation.into(),
            confidence: confidence.clamp(0.0, 1.0),
            source,
            quality: MatchQuality::from_score(confidence),
            matched_source: None,
            context: None,
            project: None,
            differences: None,
        }
    }

    /// Set matched source
    pub fn with_matched_source(mut self, source: impl Into<String>) -> Self {
        self.matched_source = Some(source.into());
        self
    }

    /// Set context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Check if this is a high-confidence suggestion
    pub fn is_high_confidence(&self) -> bool {
        self.confidence >= 0.9
    }
}

/// Configuration for the suggestion engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionConfig {
    /// Minimum confidence for suggestions
    pub min_confidence: f64,
    /// Maximum number of suggestions to return
    pub max_suggestions: usize,
    /// Whether to include fuzzy matches
    pub enable_fuzzy: bool,
    /// Minimum score for fuzzy matches
    pub fuzzy_threshold: f64,
    /// Whether to prefer project-specific matches
    pub prefer_same_project: bool,
    /// Boost for same-project matches
    pub same_project_boost: f64,
}

impl Default for SuggestionConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.5,
            max_suggestions: 5,
            enable_fuzzy: true,
            fuzzy_threshold: 0.7,
            prefer_same_project: true,
            same_project_boost: 0.1,
        }
    }
}

/// Engine for generating translation suggestions
pub struct SuggestionEngine<'a> {
    /// Translation memory to query
    memory: &'a TranslationMemory,
    /// Fuzzy matcher
    fuzzy: FuzzyMatcher,
    /// Configuration
    config: SuggestionConfig,
    /// Current project (for boosting)
    current_project: Option<String>,
    /// Glossary terms
    glossary: HashMap<String, String>,
}

impl<'a> SuggestionEngine<'a> {
    /// Create a new suggestion engine
    pub fn new(memory: &'a TranslationMemory) -> Self {
        Self {
            memory,
            fuzzy: FuzzyMatcher::new(),
            config: SuggestionConfig::default(),
            current_project: None,
            glossary: HashMap::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(memory: &'a TranslationMemory, config: SuggestionConfig) -> Self {
        Self {
            memory,
            fuzzy: FuzzyMatcher::new(),
            config,
            current_project: None,
            glossary: HashMap::new(),
        }
    }

    /// Set the current project for boosting
    pub fn set_project(&mut self, project: impl Into<String>) {
        self.current_project = Some(project.into());
    }

    /// Add a glossary term
    pub fn add_glossary_term(&mut self, source: impl Into<String>, target: impl Into<String>) {
        self.glossary.insert(source.into(), target.into());
    }

    /// Load glossary from a map
    pub fn load_glossary(&mut self, glossary: HashMap<String, String>) {
        self.glossary.extend(glossary);
    }

    /// Get suggestions for a source text
    pub fn suggest(
        &self,
        source: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Vec<Suggestion> {
        let mut suggestions = Vec::new();

        // 1. Check glossary first
        for (term, translation) in &self.glossary {
            if source.to_lowercase().contains(&term.to_lowercase()) {
                suggestions.push(
                    Suggestion::new(translation.clone(), 0.95, SuggestionSource::Glossary)
                        .with_matched_source(term),
                );
            }
        }

        // 2. Check for exact matches in TM
        let exact_matches = self.memory.find_exact(source, source_lang, target_lang);
        for entry in exact_matches {
            let mut confidence = entry.quality_score;

            // Boost for same project
            if self.config.prefer_same_project {
                if let (Some(current), Some(entry_proj)) =
                    (&self.current_project, &entry.project)
                {
                    if current == entry_proj {
                        confidence = (confidence + self.config.same_project_boost).min(1.0);
                    }
                }
            }

            suggestions.push(
                Suggestion::new(&entry.target, confidence, SuggestionSource::TranslationMemory)
                    .with_matched_source(&entry.source)
                    .with_context(entry.context.clone().unwrap_or_default()),
            );
        }

        // 3. Find fuzzy matches if enabled
        if self.config.enable_fuzzy && suggestions.iter().all(|s| s.confidence < 1.0) {
            let all_entries: Vec<_> = self
                .memory
                .find_by_language(source_lang, target_lang)
                .into_iter()
                .map(|e| (e.source.as_str(), e.target.as_str(), e.quality_score, e.project.clone()))
                .collect();

            let candidates = all_entries
                .iter()
                .map(|(s, t, _, _)| (*s, *t));

            let fuzzy_matches = self.fuzzy.find_matches(source, candidates);

            for fm in fuzzy_matches {
                if fm.score >= self.config.fuzzy_threshold
                    && fm.quality.is_acceptable()
                {
                    let mut confidence = fm.score * SuggestionSource::FuzzyMatch.confidence_weight();

                    // Find the original entry for project boosting
                    if let Some((_, _, quality, project)) = all_entries
                        .iter()
                        .find(|(s, t, _, _)| *s == fm.matched_source && *t == fm.translation)
                    {
                        confidence *= quality;

                        if self.config.prefer_same_project {
                            if let (Some(current), Some(entry_proj)) = (&self.current_project, project) {
                                if current == entry_proj {
                                    confidence = (confidence + self.config.same_project_boost).min(1.0);
                                }
                            }
                        }
                    }

                    let mut suggestion = Suggestion::new(
                        &fm.translation,
                        confidence,
                        SuggestionSource::FuzzyMatch,
                    )
                    .with_matched_source(&fm.matched_source);

                    suggestion.quality = fm.quality;

                    // Calculate differences
                    if let Some(diffs) = calculate_differences(source, &fm.matched_source) {
                        suggestion.differences = Some(diffs);
                    }

                    suggestions.push(suggestion);
                }
            }
        }

        // Sort by confidence and deduplicate
        suggestions.sort_by(|a, b| {
            b.confidence
                .partial_cmp(&a.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Remove duplicate translations
        let mut seen = std::collections::HashSet::new();
        suggestions.retain(|s| seen.insert(s.translation.clone()));

        // Apply config limits
        suggestions.retain(|s| s.confidence >= self.config.min_confidence);
        suggestions.truncate(self.config.max_suggestions);

        suggestions
    }

    /// Get the best suggestion
    pub fn best_suggestion(
        &self,
        source: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Option<Suggestion> {
        self.suggest(source, source_lang, target_lang).into_iter().next()
    }

    /// Batch suggestions for multiple sources
    pub fn batch_suggest(
        &self,
        sources: &[&str],
        source_lang: &str,
        target_lang: &str,
    ) -> HashMap<String, Vec<Suggestion>> {
        sources
            .iter()
            .map(|source| {
                (
                    source.to_string(),
                    self.suggest(source, source_lang, target_lang),
                )
            })
            .collect()
    }
}

/// Calculate differences between two strings
fn calculate_differences(original: &str, matched: &str) -> Option<Vec<String>> {
    if original == matched {
        return None;
    }

    let mut diffs = Vec::new();

    // Simple word-level diff
    let orig_words: Vec<&str> = original.split_whitespace().collect();
    let match_words: Vec<&str> = matched.split_whitespace().collect();

    for word in &orig_words {
        if !match_words.contains(word) {
            diffs.push(format!("+{}", word));
        }
    }

    for word in &match_words {
        if !orig_words.contains(word) {
            diffs.push(format!("-{}", word));
        }
    }

    if diffs.is_empty() {
        None
    } else {
        Some(diffs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn create_test_memory() -> (tempfile::TempDir, TranslationMemory) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("memory.json");
        let mut tm = TranslationMemory::open(&db_path).unwrap();

        // Add test entries
        tm.add_translation("Hello", "en", "Hallo", "de");
        tm.add_translation("Hello world", "en", "Hallo Welt", "de");
        tm.add_translation("Good morning", "en", "Guten Morgen", "de");
        tm.add_translation("Welcome back", "en", "Willkommen zurück", "de");

        (dir, tm)
    }

    #[test]
    fn test_exact_suggestion() {
        let (_dir, tm) = create_test_memory();
        let engine = SuggestionEngine::new(&tm);

        let suggestions = engine.suggest("Hello", "en", "de");
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].translation, "Hallo");
        assert_eq!(suggestions[0].source, SuggestionSource::TranslationMemory);
    }

    #[test]
    fn test_fuzzy_suggestion() {
        let (_dir, tm) = create_test_memory();
        let engine = SuggestionEngine::new(&tm);

        let suggestions = engine.suggest("Hello World!", "en", "de");
        assert!(!suggestions.is_empty());
        // Should find "Hello world" -> "Hallo Welt" as fuzzy match
    }

    #[test]
    fn test_glossary() {
        let (_dir, tm) = create_test_memory();
        let mut engine = SuggestionEngine::new(&tm);

        engine.add_glossary_term("button", "Schaltfläche");

        let suggestions = engine.suggest("Click the button", "en", "de");
        assert!(suggestions.iter().any(|s| s.source == SuggestionSource::Glossary));
    }

    #[test]
    fn test_batch_suggest() {
        let (_dir, tm) = create_test_memory();
        let engine = SuggestionEngine::new(&tm);

        let results = engine.batch_suggest(&["Hello", "Good morning"], "en", "de");
        assert_eq!(results.len(), 2);
        assert!(!results["Hello"].is_empty());
        assert!(!results["Good morning"].is_empty());
    }
}
