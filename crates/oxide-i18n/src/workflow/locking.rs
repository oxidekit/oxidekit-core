//! Translation key locking system
//!
//! Prevents conflicts when multiple translators work on the same keys.
//! Supports both file-based locks for local teams and optional server-based
//! locks for distributed teams.

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Error type for locking operations
#[derive(Debug, Error)]
pub enum LockError {
    /// Key is already locked by another user
    #[error("Key '{key}' is locked by {owner} until {expires}")]
    AlreadyLocked {
        key: String,
        owner: String,
        expires: DateTime<Utc>,
    },

    /// Lock not found
    #[error("No lock found for key '{key}'")]
    LockNotFound { key: String },

    /// Permission denied - not the lock owner
    #[error("Cannot release lock for key '{key}': owned by {owner}")]
    NotOwner { key: String, owner: String },

    /// Lock expired
    #[error("Lock for key '{key}' has expired")]
    LockExpired { key: String },

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Result type for locking operations
pub type LockResult<T> = Result<T, LockError>;

/// A lock on a translation key
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyLock {
    /// Unique lock ID
    pub id: Uuid,
    /// The locked key pattern (can be a glob)
    pub key_pattern: String,
    /// Who owns the lock
    pub owner: String,
    /// Owner's email (for contact)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_email: Option<String>,
    /// When the lock was acquired
    pub acquired_at: DateTime<Utc>,
    /// When the lock expires
    pub expires_at: DateTime<Utc>,
    /// Optional note about what's being worked on
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    /// Target locale being translated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locale: Option<String>,
    /// Hash of the source text when locked (to detect changes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_hash: Option<String>,
}

impl KeyLock {
    /// Create a new lock
    pub fn new(
        key_pattern: impl Into<String>,
        owner: impl Into<String>,
        duration: Duration,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            key_pattern: key_pattern.into(),
            owner: owner.into(),
            owner_email: None,
            acquired_at: now,
            expires_at: now + duration,
            note: None,
            locale: None,
            source_hash: None,
        }
    }

    /// Check if the lock is expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Check if a key matches this lock pattern
    pub fn matches(&self, key: &str) -> bool {
        if self.key_pattern.contains('*') {
            // Simple glob matching
            let pattern = self.key_pattern.replace('.', r"\.").replace('*', ".*");
            regex::Regex::new(&format!("^{}$", pattern))
                .map(|re| re.is_match(key))
                .unwrap_or(false)
        } else {
            self.key_pattern == key
        }
    }

    /// Extend the lock duration
    pub fn extend(&mut self, duration: Duration) {
        self.expires_at = Utc::now() + duration;
    }

    /// Set source hash for change detection
    pub fn with_source_hash(mut self, source: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(source.as_bytes());
        self.source_hash = Some(hex::encode(hasher.finalize()));
        self
    }
}

/// Status of a key's lock
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LockStatus {
    /// Key is not locked
    Unlocked,
    /// Key is locked by current user
    LockedBySelf(KeyLock),
    /// Key is locked by another user
    LockedByOther(KeyLock),
    /// Lock exists but has expired
    Expired(KeyLock),
}

impl LockStatus {
    /// Check if the key can be edited by the given user
    pub fn can_edit(&self, user: &str) -> bool {
        match self {
            LockStatus::Unlocked => true,
            LockStatus::LockedBySelf(_) => true,
            LockStatus::LockedByOther(_) => false,
            LockStatus::Expired(_) => true,
        }
    }
}

/// Manager for translation key locks
#[derive(Debug)]
pub struct LockManager {
    /// Path to the lock file
    lock_file: PathBuf,
    /// Current user identifier
    current_user: String,
    /// Default lock duration
    default_duration: Duration,
    /// In-memory lock cache
    locks: HashMap<Uuid, KeyLock>,
}

impl LockManager {
    /// Create a new lock manager
    pub fn new(
        lock_file: impl Into<PathBuf>,
        current_user: impl Into<String>,
    ) -> LockResult<Self> {
        let lock_file = lock_file.into();
        let locks = if lock_file.exists() {
            let content = fs::read_to_string(&lock_file)?;
            serde_json::from_str(&content)
                .map_err(|e| LockError::Serialization(e.to_string()))?
        } else {
            HashMap::new()
        };

        Ok(Self {
            lock_file,
            current_user: current_user.into(),
            default_duration: Duration::hours(4),
            locks,
        })
    }

    /// Set the default lock duration
    pub fn with_default_duration(mut self, duration: Duration) -> Self {
        self.default_duration = duration;
        self
    }

    /// Acquire a lock on a key
    pub fn lock(&mut self, key: impl Into<String>) -> LockResult<KeyLock> {
        self.lock_with_duration(key, self.default_duration)
    }

    /// Acquire a lock with a specific duration
    pub fn lock_with_duration(
        &mut self,
        key: impl Into<String>,
        duration: Duration,
    ) -> LockResult<KeyLock> {
        let key = key.into();

        // Check for existing locks
        self.cleanup_expired();

        for lock in self.locks.values() {
            if lock.matches(&key) && !lock.is_expired() {
                if lock.owner == self.current_user {
                    // Already locked by us, extend it
                    let mut lock = lock.clone();
                    lock.extend(duration);
                    self.locks.insert(lock.id, lock.clone());
                    self.save()?;
                    return Ok(lock);
                } else {
                    return Err(LockError::AlreadyLocked {
                        key: key.clone(),
                        owner: lock.owner.clone(),
                        expires: lock.expires_at,
                    });
                }
            }
        }

        // Create new lock
        let lock = KeyLock::new(key, &self.current_user, duration);
        self.locks.insert(lock.id, lock.clone());
        self.save()?;

        Ok(lock)
    }

    /// Release a lock
    pub fn unlock(&mut self, key: &str) -> LockResult<()> {
        let lock_id = self
            .locks
            .iter()
            .find(|(_, lock)| lock.matches(key) && !lock.is_expired())
            .map(|(id, lock)| {
                if lock.owner != self.current_user {
                    Err(LockError::NotOwner {
                        key: key.to_string(),
                        owner: lock.owner.clone(),
                    })
                } else {
                    Ok(*id)
                }
            })
            .transpose()?
            .ok_or_else(|| LockError::LockNotFound {
                key: key.to_string(),
            })?;

        self.locks.remove(&lock_id);
        self.save()?;

        Ok(())
    }

    /// Release a lock by ID
    pub fn unlock_by_id(&mut self, lock_id: Uuid) -> LockResult<()> {
        let lock = self
            .locks
            .get(&lock_id)
            .ok_or_else(|| LockError::LockNotFound {
                key: lock_id.to_string(),
            })?;

        if lock.owner != self.current_user {
            return Err(LockError::NotOwner {
                key: lock.key_pattern.clone(),
                owner: lock.owner.clone(),
            });
        }

        self.locks.remove(&lock_id);
        self.save()?;

        Ok(())
    }

    /// Get the lock status for a key
    pub fn status(&self, key: &str) -> LockStatus {
        for lock in self.locks.values() {
            if lock.matches(key) {
                if lock.is_expired() {
                    return LockStatus::Expired(lock.clone());
                } else if lock.owner == self.current_user {
                    return LockStatus::LockedBySelf(lock.clone());
                } else {
                    return LockStatus::LockedByOther(lock.clone());
                }
            }
        }
        LockStatus::Unlocked
    }

    /// Get all active locks
    pub fn active_locks(&self) -> Vec<&KeyLock> {
        self.locks
            .values()
            .filter(|lock| !lock.is_expired())
            .collect()
    }

    /// Get locks by owner
    pub fn locks_by_owner(&self, owner: &str) -> Vec<&KeyLock> {
        self.locks
            .values()
            .filter(|lock| lock.owner == owner && !lock.is_expired())
            .collect()
    }

    /// Get current user's locks
    pub fn my_locks(&self) -> Vec<&KeyLock> {
        self.locks_by_owner(&self.current_user)
    }

    /// Release all locks owned by current user
    pub fn unlock_all_mine(&mut self) -> LockResult<usize> {
        let my_lock_ids: Vec<Uuid> = self
            .locks
            .iter()
            .filter(|(_, lock)| lock.owner == self.current_user)
            .map(|(id, _)| *id)
            .collect();

        let count = my_lock_ids.len();
        for id in my_lock_ids {
            self.locks.remove(&id);
        }
        self.save()?;

        Ok(count)
    }

    /// Cleanup expired locks
    pub fn cleanup_expired(&mut self) -> usize {
        let expired: Vec<Uuid> = self
            .locks
            .iter()
            .filter(|(_, lock)| lock.is_expired())
            .map(|(id, _)| *id)
            .collect();

        let count = expired.len();
        for id in expired {
            self.locks.remove(&id);
        }

        if count > 0 {
            let _ = self.save();
        }

        count
    }

    /// Force release a lock (admin operation)
    pub fn force_unlock(&mut self, key: &str) -> LockResult<()> {
        let lock_ids: Vec<Uuid> = self
            .locks
            .iter()
            .filter(|(_, lock)| lock.matches(key))
            .map(|(id, _)| *id)
            .collect();

        if lock_ids.is_empty() {
            return Err(LockError::LockNotFound {
                key: key.to_string(),
            });
        }

        for id in lock_ids {
            self.locks.remove(&id);
        }
        self.save()?;

        Ok(())
    }

    /// Save locks to file
    fn save(&self) -> LockResult<()> {
        if let Some(parent) = self.lock_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.locks)
            .map_err(|e| LockError::Serialization(e.to_string()))?;
        fs::write(&self.lock_file, content)?;
        Ok(())
    }

    /// Reload locks from file
    pub fn reload(&mut self) -> LockResult<()> {
        if self.lock_file.exists() {
            let content = fs::read_to_string(&self.lock_file)?;
            self.locks = serde_json::from_str(&content)
                .map_err(|e| LockError::Serialization(e.to_string()))?;
        }
        Ok(())
    }
}

/// Lock file format for serialization
#[derive(Debug, Serialize, Deserialize)]
pub struct LockFile {
    /// Version of the lock file format
    pub version: String,
    /// Active locks
    pub locks: Vec<KeyLock>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_key_lock_matching() {
        let lock = KeyLock::new("auth.*", "alice", Duration::hours(1));
        assert!(lock.matches("auth.login"));
        assert!(lock.matches("auth.logout"));
        assert!(!lock.matches("common.button"));
    }

    #[test]
    fn test_lock_manager() {
        let dir = tempdir().unwrap();
        let lock_file = dir.path().join("locks.json");

        let mut manager = LockManager::new(&lock_file, "alice").unwrap();

        // Lock a key
        let lock = manager.lock("auth.login").unwrap();
        assert_eq!(lock.owner, "alice");

        // Check status
        let status = manager.status("auth.login");
        assert!(matches!(status, LockStatus::LockedBySelf(_)));

        // Try to lock same key with different user
        let mut manager2 = LockManager::new(&lock_file, "bob").unwrap();
        let result = manager2.lock("auth.login");
        assert!(result.is_err());

        // Release lock
        manager.unlock("auth.login").unwrap();

        // Now bob can lock
        manager2.reload().unwrap();
        let lock2 = manager2.lock("auth.login").unwrap();
        assert_eq!(lock2.owner, "bob");
    }

    #[test]
    fn test_lock_expiration() {
        let mut lock = KeyLock::new("test", "alice", Duration::seconds(-1));
        assert!(lock.is_expired());

        lock.extend(Duration::hours(1));
        assert!(!lock.is_expired());
    }
}
