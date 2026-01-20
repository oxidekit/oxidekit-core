//! Translation memory and suggestions
//!
//! Provides translation memory (TM) capabilities:
//! - Store and retrieve past translations
//! - Fuzzy matching for similar strings
//! - Suggestions for new translations
//! - Machine translation integration (optional)

pub mod database;
pub mod fuzzy;
pub mod suggestions;

pub use database::{TranslationMemory, MemoryEntry, MemoryStats};
pub use fuzzy::{FuzzyMatcher, FuzzyMatch, MatchQuality, extract_placeholders};
pub use suggestions::{SuggestionEngine, Suggestion, SuggestionSource};
