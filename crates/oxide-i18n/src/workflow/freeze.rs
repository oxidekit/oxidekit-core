//! String freeze workflow
//!
//! Manages the string freeze process for release preparation:
//! - Soft freeze: Strings can be added but changes need approval
//! - Hard freeze: No string changes allowed
//! - Release: Translations finalized

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Error type for freeze operations
#[derive(Debug, Error)]
pub enum FreezeError {
    /// Cannot modify strings during freeze
    #[error("Cannot {action} strings during {phase} freeze")]
    FreezeViolation { action: String, phase: String },

    /// Freeze file error
    #[error("Freeze file error: {0}")]
    FileError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for freeze operations
pub type FreezeResult<T> = Result<T, FreezeError>;

/// Phases of the string freeze workflow
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FreezePhase {
    /// Normal development - strings can be freely modified
    Development,
    /// Soft freeze - new strings need approval, changes discouraged
    SoftFreeze,
    /// Hard freeze - no string changes allowed
    HardFreeze,
    /// Release candidate - final translations being validated
    ReleaseCandidate,
    /// Released - version is final
    Released,
}

impl FreezePhase {
    /// Check if string additions are allowed
    pub fn allows_additions(&self) -> bool {
        matches!(self, FreezePhase::Development | FreezePhase::SoftFreeze)
    }

    /// Check if string modifications are allowed
    pub fn allows_modifications(&self) -> bool {
        matches!(self, FreezePhase::Development)
    }

    /// Check if string removals are allowed
    pub fn allows_removals(&self) -> bool {
        matches!(self, FreezePhase::Development)
    }

    /// Check if translation updates are allowed
    pub fn allows_translations(&self) -> bool {
        !matches!(self, FreezePhase::Released)
    }

    /// Get the next phase in the workflow
    pub fn next(&self) -> Option<FreezePhase> {
        match self {
            FreezePhase::Development => Some(FreezePhase::SoftFreeze),
            FreezePhase::SoftFreeze => Some(FreezePhase::HardFreeze),
            FreezePhase::HardFreeze => Some(FreezePhase::ReleaseCandidate),
            FreezePhase::ReleaseCandidate => Some(FreezePhase::Released),
            FreezePhase::Released => None,
        }
    }

    /// Get human-readable phase name
    pub fn display_name(&self) -> &'static str {
        match self {
            FreezePhase::Development => "Development",
            FreezePhase::SoftFreeze => "Soft Freeze",
            FreezePhase::HardFreeze => "Hard Freeze",
            FreezePhase::ReleaseCandidate => "Release Candidate",
            FreezePhase::Released => "Released",
        }
    }
}

impl Default for FreezePhase {
    fn default() -> Self {
        FreezePhase::Development
    }
}

/// String freeze status and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeStatus {
    /// Current phase
    pub phase: FreezePhase,
    /// Version being frozen
    pub version: String,
    /// When the freeze was initiated
    #[serde(skip_serializing_if = "Option::is_none")]
    pub started_at: Option<DateTime<Utc>>,
    /// When the freeze ends (for time-boxed freezes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ends_at: Option<DateTime<Utc>>,
    /// Approved exceptions during freeze
    #[serde(default)]
    pub exceptions: HashSet<String>,
    /// History of phase transitions
    #[serde(default)]
    pub history: Vec<PhaseTransition>,
    /// Notes about the freeze
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// A phase transition in the freeze history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhaseTransition {
    /// Phase transitioned from
    pub from: FreezePhase,
    /// Phase transitioned to
    pub to: FreezePhase,
    /// When the transition occurred
    pub timestamp: DateTime<Utc>,
    /// Who initiated the transition
    pub initiated_by: String,
    /// Reason for transition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

impl Default for FreezeStatus {
    fn default() -> Self {
        Self {
            phase: FreezePhase::Development,
            version: "0.0.0".to_string(),
            started_at: None,
            ends_at: None,
            exceptions: HashSet::new(),
            history: Vec::new(),
            notes: None,
        }
    }
}

impl FreezeStatus {
    /// Create a new freeze status for a version
    pub fn new(version: impl Into<String>) -> Self {
        Self {
            version: version.into(),
            ..Default::default()
        }
    }

    /// Check if a key has an exception
    pub fn has_exception(&self, key: &str) -> bool {
        self.exceptions.contains(key)
    }

    /// Add an exception for a key
    pub fn add_exception(&mut self, key: impl Into<String>) {
        self.exceptions.insert(key.into());
    }

    /// Remove an exception
    pub fn remove_exception(&mut self, key: &str) -> bool {
        self.exceptions.remove(key)
    }
}

/// Manager for string freeze workflow
#[derive(Debug)]
pub struct StringFreeze {
    /// Path to the freeze status file
    status_file: PathBuf,
    /// Current freeze status
    status: FreezeStatus,
    /// Current user
    current_user: String,
}

impl StringFreeze {
    /// Create or load a string freeze manager
    pub fn new(
        status_file: impl Into<PathBuf>,
        current_user: impl Into<String>,
    ) -> FreezeResult<Self> {
        let status_file = status_file.into();
        let status = if status_file.exists() {
            let content = fs::read_to_string(&status_file)?;
            serde_json::from_str(&content)
                .map_err(|e| FreezeError::FileError(e.to_string()))?
        } else {
            FreezeStatus::default()
        };

        Ok(Self {
            status_file,
            status,
            current_user: current_user.into(),
        })
    }

    /// Get current freeze status
    pub fn status(&self) -> &FreezeStatus {
        &self.status
    }

    /// Get current phase
    pub fn phase(&self) -> FreezePhase {
        self.status.phase
    }

    /// Start a freeze for a version
    pub fn start_freeze(&mut self, version: impl Into<String>) -> FreezeResult<()> {
        let version = version.into();
        let old_phase = self.status.phase;

        self.status = FreezeStatus::new(&version);
        self.status.phase = FreezePhase::SoftFreeze;
        self.status.started_at = Some(Utc::now());
        self.status.history.push(PhaseTransition {
            from: old_phase,
            to: FreezePhase::SoftFreeze,
            timestamp: Utc::now(),
            initiated_by: self.current_user.clone(),
            reason: Some(format!("Started freeze for version {}", version)),
        });

        self.save()
    }

    /// Advance to the next phase
    pub fn advance_phase(&mut self, reason: Option<String>) -> FreezeResult<FreezePhase> {
        let old_phase = self.status.phase;
        let new_phase = old_phase.next().ok_or_else(|| {
            FreezeError::FreezeViolation {
                action: "advance".to_string(),
                phase: "Released".to_string(),
            }
        })?;

        self.status.phase = new_phase;
        self.status.history.push(PhaseTransition {
            from: old_phase,
            to: new_phase,
            timestamp: Utc::now(),
            initiated_by: self.current_user.clone(),
            reason,
        });

        self.save()?;
        Ok(new_phase)
    }

    /// Go back to a previous phase (emergency rollback)
    pub fn rollback_to(&mut self, phase: FreezePhase, reason: String) -> FreezeResult<()> {
        let old_phase = self.status.phase;

        self.status.phase = phase;
        self.status.history.push(PhaseTransition {
            from: old_phase,
            to: phase,
            timestamp: Utc::now(),
            initiated_by: self.current_user.clone(),
            reason: Some(format!("ROLLBACK: {}", reason)),
        });

        self.save()
    }

    /// Reset to development phase
    pub fn reset(&mut self) -> FreezeResult<()> {
        let old_phase = self.status.phase;

        self.status.phase = FreezePhase::Development;
        self.status.started_at = None;
        self.status.ends_at = None;
        self.status.exceptions.clear();
        self.status.history.push(PhaseTransition {
            from: old_phase,
            to: FreezePhase::Development,
            timestamp: Utc::now(),
            initiated_by: self.current_user.clone(),
            reason: Some("Reset to development".to_string()),
        });

        self.save()
    }

    /// Check if an action is allowed
    pub fn check_action(&self, action: &StringAction, key: &str) -> FreezeResult<()> {
        // Check for exception
        if self.status.has_exception(key) {
            return Ok(());
        }

        let allowed = match action {
            StringAction::Add => self.status.phase.allows_additions(),
            StringAction::Modify => self.status.phase.allows_modifications(),
            StringAction::Remove => self.status.phase.allows_removals(),
            StringAction::Translate => self.status.phase.allows_translations(),
        };

        if allowed {
            Ok(())
        } else {
            Err(FreezeError::FreezeViolation {
                action: action.display_name().to_string(),
                phase: self.status.phase.display_name().to_string(),
            })
        }
    }

    /// Add an exception for a key
    pub fn add_exception(&mut self, key: impl Into<String>) -> FreezeResult<()> {
        self.status.add_exception(key);
        self.save()
    }

    /// Remove an exception
    pub fn remove_exception(&mut self, key: &str) -> FreezeResult<bool> {
        let removed = self.status.remove_exception(key);
        self.save()?;
        Ok(removed)
    }

    /// Save status to file
    fn save(&self) -> FreezeResult<()> {
        if let Some(parent) = self.status_file.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(&self.status)
            .map_err(|e| FreezeError::FileError(e.to_string()))?;
        fs::write(&self.status_file, content)?;
        Ok(())
    }

    /// Reload from file
    pub fn reload(&mut self) -> FreezeResult<()> {
        if self.status_file.exists() {
            let content = fs::read_to_string(&self.status_file)?;
            self.status = serde_json::from_str(&content)
                .map_err(|e| FreezeError::FileError(e.to_string()))?;
        }
        Ok(())
    }
}

/// Actions that can be performed on strings
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StringAction {
    /// Adding a new string
    Add,
    /// Modifying an existing source string
    Modify,
    /// Removing a string
    Remove,
    /// Adding or updating a translation
    Translate,
}

impl StringAction {
    /// Get human-readable name
    pub fn display_name(&self) -> &'static str {
        match self {
            StringAction::Add => "add",
            StringAction::Modify => "modify",
            StringAction::Remove => "remove",
            StringAction::Translate => "translate",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_freeze_phases() {
        assert!(FreezePhase::Development.allows_additions());
        assert!(FreezePhase::Development.allows_modifications());
        assert!(FreezePhase::Development.allows_removals());

        assert!(FreezePhase::SoftFreeze.allows_additions());
        assert!(!FreezePhase::SoftFreeze.allows_modifications());
        assert!(!FreezePhase::SoftFreeze.allows_removals());

        assert!(!FreezePhase::HardFreeze.allows_additions());
        assert!(!FreezePhase::HardFreeze.allows_modifications());
    }

    #[test]
    fn test_freeze_workflow() {
        let dir = tempdir().unwrap();
        let status_file = dir.path().join("freeze.json");

        let mut freeze = StringFreeze::new(&status_file, "release-manager").unwrap();

        // Start in development
        assert_eq!(freeze.phase(), FreezePhase::Development);

        // Start freeze
        freeze.start_freeze("1.0.0").unwrap();
        assert_eq!(freeze.phase(), FreezePhase::SoftFreeze);

        // Advance phases
        freeze.advance_phase(Some("Ready for hard freeze".to_string())).unwrap();
        assert_eq!(freeze.phase(), FreezePhase::HardFreeze);

        // Check action restrictions
        assert!(freeze
            .check_action(&StringAction::Translate, "any.key")
            .is_ok());
        assert!(freeze.check_action(&StringAction::Add, "any.key").is_err());

        // Add exception
        freeze.add_exception("emergency.fix").unwrap();
        assert!(freeze
            .check_action(&StringAction::Add, "emergency.fix")
            .is_ok());
    }
}
