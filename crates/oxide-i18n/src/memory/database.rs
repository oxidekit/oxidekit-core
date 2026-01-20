//! Translation memory database
//!
//! Stores past translations for reuse and consistency.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Error type for memory operations
#[derive(Debug, Error)]
pub enum MemoryError {
    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Entry not found
    #[error("Entry not found: {0}")]
    NotFound(String),
}

/// Result type for memory operations
pub type MemoryResult<T> = Result<T, MemoryError>;

/// A single translation memory entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryEntry {
    /// Unique entry ID
    pub id: Uuid,
    /// Source text
    pub source: String,
    /// Source language code
    pub source_lang: String,
    /// Target text (translation)
    pub target: String,
    /// Target language code
    pub target_lang: String,
    /// Hash of source text for quick lookup
    pub source_hash: String,
    /// Context where this translation was used
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    /// Project this came from
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<String>,
    /// Who created/approved this translation
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    /// When the entry was created
    pub created_at: DateTime<Utc>,
    /// When the entry was last used
    pub last_used: DateTime<Utc>,
    /// Number of times this entry was used
    pub use_count: u32,
    /// Quality score (0.0 - 1.0)
    pub quality_score: f64,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
}

impl MemoryEntry {
    /// Create a new memory entry
    pub fn new(
        source: impl Into<String>,
        source_lang: impl Into<String>,
        target: impl Into<String>,
        target_lang: impl Into<String>,
    ) -> Self {
        let source = source.into();
        let source_hash = hash_text(&source);

        Self {
            id: Uuid::new_v4(),
            source,
            source_lang: source_lang.into(),
            target: target.into(),
            target_lang: target_lang.into(),
            source_hash,
            context: None,
            project: None,
            author: None,
            created_at: Utc::now(),
            last_used: Utc::now(),
            use_count: 0,
            quality_score: 1.0,
            tags: Vec::new(),
        }
    }

    /// Set context
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        self.context = Some(context.into());
        self
    }

    /// Set project
    pub fn with_project(mut self, project: impl Into<String>) -> Self {
        self.project = Some(project.into());
        self
    }

    /// Set author
    pub fn with_author(mut self, author: impl Into<String>) -> Self {
        self.author = Some(author.into());
        self
    }

    /// Set quality score
    pub fn with_quality(mut self, score: f64) -> Self {
        self.quality_score = score.clamp(0.0, 1.0);
        self
    }

    /// Mark as used
    pub fn mark_used(&mut self) {
        self.last_used = Utc::now();
        self.use_count += 1;
    }
}

/// Translation memory database
#[derive(Debug)]
pub struct TranslationMemory {
    /// Path to the memory database file
    db_path: PathBuf,
    /// Entries indexed by source hash
    entries_by_hash: HashMap<String, Vec<Uuid>>,
    /// Entries indexed by language pair
    entries_by_lang: HashMap<String, Vec<Uuid>>,
    /// All entries by ID
    entries: HashMap<Uuid, MemoryEntry>,
    /// Whether there are unsaved changes
    dirty: bool,
}

impl TranslationMemory {
    /// Create or load a translation memory database
    pub fn open(path: impl Into<PathBuf>) -> MemoryResult<Self> {
        let db_path = path.into();

        let mut tm = if db_path.exists() {
            let content = fs::read_to_string(&db_path)?;
            let entries: Vec<MemoryEntry> = serde_json::from_str(&content)
                .map_err(|e| MemoryError::Serialization(e.to_string()))?;

            let mut tm = Self {
                db_path,
                entries_by_hash: HashMap::new(),
                entries_by_lang: HashMap::new(),
                entries: HashMap::new(),
                dirty: false,
            };

            for entry in entries {
                tm.index_entry(&entry);
                tm.entries.insert(entry.id, entry);
            }

            tm
        } else {
            Self {
                db_path,
                entries_by_hash: HashMap::new(),
                entries_by_lang: HashMap::new(),
                entries: HashMap::new(),
                dirty: false,
            }
        };

        Ok(tm)
    }

    /// Add a new entry to the memory
    pub fn add(&mut self, entry: MemoryEntry) -> Uuid {
        let id = entry.id;
        self.index_entry(&entry);
        self.entries.insert(id, entry);
        self.dirty = true;
        id
    }

    /// Add a translation pair
    pub fn add_translation(
        &mut self,
        source: impl Into<String>,
        source_lang: impl Into<String>,
        target: impl Into<String>,
        target_lang: impl Into<String>,
    ) -> Uuid {
        let entry = MemoryEntry::new(source, source_lang, target, target_lang);
        self.add(entry)
    }

    /// Get an entry by ID
    pub fn get(&self, id: Uuid) -> Option<&MemoryEntry> {
        self.entries.get(&id)
    }

    /// Get a mutable entry by ID
    pub fn get_mut(&mut self, id: Uuid) -> Option<&mut MemoryEntry> {
        self.dirty = true;
        self.entries.get_mut(&id)
    }

    /// Find exact matches for a source text
    pub fn find_exact(
        &self,
        source: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Vec<&MemoryEntry> {
        let hash = hash_text(source);
        let lang_key = lang_pair_key(source_lang, target_lang);

        self.entries_by_hash
            .get(&hash)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.entries.get(id))
                    .filter(|e| {
                        e.source == source
                            && e.source_lang == source_lang
                            && e.target_lang == target_lang
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Find entries for a language pair
    pub fn find_by_language(&self, source_lang: &str, target_lang: &str) -> Vec<&MemoryEntry> {
        let lang_key = lang_pair_key(source_lang, target_lang);

        self.entries_by_lang
            .get(&lang_key)
            .map(|ids| ids.iter().filter_map(|id| self.entries.get(id)).collect())
            .unwrap_or_default()
    }

    /// Get all entries
    pub fn all_entries(&self) -> impl Iterator<Item = &MemoryEntry> {
        self.entries.values()
    }

    /// Get entry count
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Get statistics
    pub fn stats(&self) -> MemoryStats {
        let mut languages: HashMap<String, usize> = HashMap::new();
        let mut quality_sum = 0.0;
        let mut total_uses = 0u64;

        for entry in self.entries.values() {
            let lang_key = lang_pair_key(&entry.source_lang, &entry.target_lang);
            *languages.entry(lang_key).or_default() += 1;
            quality_sum += entry.quality_score;
            total_uses += entry.use_count as u64;
        }

        let entry_count = self.entries.len();
        let avg_quality = if entry_count > 0 {
            quality_sum / entry_count as f64
        } else {
            0.0
        };

        MemoryStats {
            entry_count,
            language_pairs: languages,
            average_quality: avg_quality,
            total_uses,
        }
    }

    /// Remove an entry
    pub fn remove(&mut self, id: Uuid) -> Option<MemoryEntry> {
        if let Some(entry) = self.entries.remove(&id) {
            // Remove from hash index
            if let Some(ids) = self.entries_by_hash.get_mut(&entry.source_hash) {
                ids.retain(|&i| i != id);
            }
            // Remove from language index
            let lang_key = lang_pair_key(&entry.source_lang, &entry.target_lang);
            if let Some(ids) = self.entries_by_lang.get_mut(&lang_key) {
                ids.retain(|&i| i != id);
            }
            self.dirty = true;
            Some(entry)
        } else {
            None
        }
    }

    /// Remove entries older than a date
    pub fn prune_before(&mut self, date: DateTime<Utc>) -> usize {
        let to_remove: Vec<Uuid> = self
            .entries
            .iter()
            .filter(|(_, e)| e.last_used < date)
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.remove(id);
        }
        count
    }

    /// Remove entries with low quality
    pub fn prune_low_quality(&mut self, threshold: f64) -> usize {
        let to_remove: Vec<Uuid> = self
            .entries
            .iter()
            .filter(|(_, e)| e.quality_score < threshold)
            .map(|(id, _)| *id)
            .collect();

        let count = to_remove.len();
        for id in to_remove {
            self.remove(id);
        }
        count
    }

    /// Save the database
    pub fn save(&mut self) -> MemoryResult<()> {
        if !self.dirty {
            return Ok(());
        }

        if let Some(parent) = self.db_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let entries: Vec<&MemoryEntry> = self.entries.values().collect();
        let content = serde_json::to_string_pretty(&entries)
            .map_err(|e| MemoryError::Serialization(e.to_string()))?;

        fs::write(&self.db_path, content)?;
        self.dirty = false;
        Ok(())
    }

    /// Import entries from another memory
    pub fn import(&mut self, entries: Vec<MemoryEntry>) -> usize {
        let mut count = 0;
        for entry in entries {
            // Check for duplicates
            let existing = self.find_exact(&entry.source, &entry.source_lang, &entry.target_lang);
            if existing.iter().any(|e| e.target == entry.target) {
                continue;
            }
            self.add(entry);
            count += 1;
        }
        count
    }

    /// Export all entries
    pub fn export(&self) -> Vec<MemoryEntry> {
        self.entries.values().cloned().collect()
    }

    /// Index an entry for quick lookup
    fn index_entry(&mut self, entry: &MemoryEntry) {
        // Index by source hash
        self.entries_by_hash
            .entry(entry.source_hash.clone())
            .or_default()
            .push(entry.id);

        // Index by language pair
        let lang_key = lang_pair_key(&entry.source_lang, &entry.target_lang);
        self.entries_by_lang
            .entry(lang_key)
            .or_default()
            .push(entry.id);
    }
}

impl Drop for TranslationMemory {
    fn drop(&mut self) {
        // Auto-save on drop (ignore errors)
        let _ = self.save();
    }
}

/// Statistics about the translation memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total number of entries
    pub entry_count: usize,
    /// Entries by language pair
    pub language_pairs: HashMap<String, usize>,
    /// Average quality score
    pub average_quality: f64,
    /// Total number of uses
    pub total_uses: u64,
}

/// Create a hash of text for indexing
fn hash_text(text: &str) -> String {
    let normalized = text.trim().to_lowercase();
    let mut hasher = Sha256::new();
    hasher.update(normalized.as_bytes());
    hex::encode(hasher.finalize())
}

/// Create a key for a language pair
fn lang_pair_key(source: &str, target: &str) -> String {
    format!("{}>{}", source, target)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_memory_basic() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("memory.json");

        let mut tm = TranslationMemory::open(&db_path).unwrap();

        // Add entry
        let id = tm.add_translation("Hello", "en", "Hallo", "de");
        assert_eq!(tm.len(), 1);

        // Find exact match
        let matches = tm.find_exact("Hello", "en", "de");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].target, "Hallo");

        // Save and reload
        tm.save().unwrap();
        drop(tm);

        let tm2 = TranslationMemory::open(&db_path).unwrap();
        assert_eq!(tm2.len(), 1);
    }

    #[test]
    fn test_memory_stats() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("memory.json");

        let mut tm = TranslationMemory::open(&db_path).unwrap();

        tm.add_translation("Hello", "en", "Hallo", "de");
        tm.add_translation("World", "en", "Welt", "de");
        tm.add_translation("Hello", "en", "Bonjour", "fr");

        let stats = tm.stats();
        assert_eq!(stats.entry_count, 3);
        assert_eq!(stats.language_pairs.len(), 2);
    }

    #[test]
    fn test_memory_pruning() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("memory.json");

        let mut tm = TranslationMemory::open(&db_path).unwrap();

        // Add entry with low quality
        let entry = MemoryEntry::new("Test", "en", "Test", "de").with_quality(0.3);
        tm.add(entry);

        // Add normal entry
        tm.add_translation("Hello", "en", "Hallo", "de");

        assert_eq!(tm.len(), 2);

        // Prune low quality
        let pruned = tm.prune_low_quality(0.5);
        assert_eq!(pruned, 1);
        assert_eq!(tm.len(), 1);
    }
}
