//! Continuous Sync Engine
//!
//! Provides continuous synchronization between Figma and OxideKit projects:
//! - Watch for Figma file changes
//! - Pull updates automatically
//! - Handle conflicts intelligently
//! - Maintain sync state

use crate::api::{FigmaClient, FigmaConfig, FigmaUrlInfo};
use crate::diff::{DesignDiff, DiffResult};
use crate::error::{FigmaError, Result};
use crate::tokens::TokenExtractor;
use crate::translator::{TranslationResult, Translator};
use crate::types::FigmaFile;
use camino::{Utf8Path, Utf8PathBuf};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, error, info, warn};

/// Sync engine for continuous Figma synchronization
pub struct SyncEngine {
    client: FigmaClient,
    config: SyncConfig,
    state: SyncState,
}

/// Configuration for sync
#[derive(Debug, Clone)]
pub struct SyncConfig {
    /// Figma file key to sync
    pub file_key: String,

    /// Project directory
    pub project_dir: Utf8PathBuf,

    /// Output directory for generated files (relative to project)
    pub output_dir: Utf8PathBuf,

    /// Poll interval for checking updates
    pub poll_interval: Duration,

    /// Whether to auto-apply non-breaking changes
    pub auto_apply_safe: bool,

    /// Whether to download assets
    pub download_assets: bool,

    /// Assets output directory (relative to project)
    pub assets_dir: Utf8PathBuf,

    /// File patterns to ignore when checking for local changes
    pub ignore_patterns: Vec<String>,

    /// Whether to create backups before applying changes
    pub backup_enabled: bool,

    /// Maximum number of backups to keep
    pub max_backups: usize,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            file_key: String::new(),
            project_dir: Utf8PathBuf::from("."),
            output_dir: Utf8PathBuf::from("design"),
            poll_interval: Duration::from_secs(60),
            auto_apply_safe: false,
            download_assets: true,
            assets_dir: Utf8PathBuf::from("assets"),
            ignore_patterns: vec!["*.local.*".into(), "*.backup.*".into()],
            backup_enabled: true,
            max_backups: 5,
        }
    }
}

/// Sync state persisted between runs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SyncState {
    /// Last known Figma file version
    pub last_version: Option<String>,

    /// Last sync timestamp
    pub last_sync: Option<DateTime<Utc>>,

    /// Last Figma file modification timestamp
    pub last_modified: Option<String>,

    /// Hashes of generated files (for detecting local changes)
    pub file_hashes: HashMap<String, String>,

    /// Pending changes not yet applied
    pub pending_changes: Vec<PendingChange>,

    /// Files with local modifications
    pub locally_modified: Vec<String>,
}

/// A pending change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingChange {
    pub change_type: PendingChangeType,
    pub file_path: String,
    pub description: String,
    pub impact: String,
    pub detected_at: DateTime<Utc>,
}

/// Type of pending change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PendingChangeType {
    ThemeUpdate,
    TokenUpdate,
    ComponentUpdate,
    AssetUpdate,
    LayoutUpdate,
}

/// Result of sync operation
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Whether sync was successful
    pub success: bool,

    /// Whether there were changes
    pub has_changes: bool,

    /// Diff result (if changes detected)
    pub diff: Option<DiffResult>,

    /// Files updated
    pub files_updated: Vec<String>,

    /// Files with conflicts
    pub conflicts: Vec<SyncConflict>,

    /// Error message (if failed)
    pub error: Option<String>,

    /// Duration of sync
    pub duration: Duration,
}

/// A sync conflict
#[derive(Debug, Clone)]
pub struct SyncConflict {
    pub file_path: String,
    pub reason: ConflictReason,
    pub local_hash: String,
    pub remote_hash: String,
}

/// Reason for conflict
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConflictReason {
    LocalModification,
    BothModified,
    DeletedLocally,
}

impl SyncEngine {
    /// Create a new sync engine
    pub fn new(client: FigmaClient, config: SyncConfig) -> Self {
        Self {
            client,
            config,
            state: SyncState::default(),
        }
    }

    /// Load state from disk
    pub fn load_state(&mut self) -> Result<()> {
        let state_path = self.state_file_path();

        if state_path.exists() {
            let content = fs::read_to_string(&state_path)?;
            self.state = serde_json::from_str(&content)
                .map_err(|e| FigmaError::ParseError(e.to_string()))?;
            debug!("Loaded sync state from {}", state_path);
        }

        Ok(())
    }

    /// Save state to disk
    pub fn save_state(&self) -> Result<()> {
        let state_path = self.state_file_path();

        // Ensure parent directory exists
        if let Some(parent) = state_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(&self.state)
            .map_err(|e| FigmaError::SerializationError(e.to_string()))?;

        fs::write(&state_path, content)?;
        debug!("Saved sync state to {}", state_path);

        Ok(())
    }

    /// Get state file path
    fn state_file_path(&self) -> Utf8PathBuf {
        self.config.project_dir
            .join(&self.config.output_dir)
            .join(".figma-sync-state.json")
    }

    /// Check for updates without applying
    pub async fn check_for_updates(&mut self) -> Result<Option<DiffResult>> {
        info!(file_key = %self.config.file_key, "Checking for Figma updates");

        // Fetch current file
        let file = self.client.get_file(&self.config.file_key).await?;

        // Check if version changed
        let version_changed = self.state.last_version.as_ref() != Some(&file.version);
        let modified_changed = self.state.last_modified.as_ref() != Some(&file.last_modified);

        if !version_changed && !modified_changed {
            debug!("No changes detected");
            return Ok(None);
        }

        info!(
            old_version = ?self.state.last_version,
            new_version = %file.version,
            "Figma file has changed"
        );

        // Extract tokens and compare
        let extractor = TokenExtractor::new();
        let tokens = extractor.extract(&file)?;

        // Load current theme
        let theme_path = self.config.project_dir
            .join(&self.config.output_dir)
            .join("theme.generated.toml");

        if theme_path.exists() {
            let theme_content = fs::read_to_string(&theme_path)?;
            let current_theme = oxide_components::Theme::from_toml(&theme_content)?;

            let diff_engine = DesignDiff::new();
            let diff = diff_engine.compare_tokens(&tokens, &current_theme);

            if diff.has_changes() {
                return Ok(Some(diff));
            }
        }

        // No existing theme, treat as new
        Ok(None)
    }

    /// Perform a full sync
    pub async fn sync(&mut self) -> Result<SyncResult> {
        let start = std::time::Instant::now();
        info!(file_key = %self.config.file_key, "Starting Figma sync");

        // Load state
        let _ = self.load_state();

        // Fetch file
        let file = match self.client.get_file(&self.config.file_key).await {
            Ok(f) => f,
            Err(e) => {
                return Ok(SyncResult {
                    success: false,
                    has_changes: false,
                    diff: None,
                    files_updated: Vec::new(),
                    conflicts: Vec::new(),
                    error: Some(e.to_string()),
                    duration: start.elapsed(),
                });
            }
        };

        // Check for local modifications
        let conflicts = self.detect_conflicts()?;

        if !conflicts.is_empty() && !self.config.auto_apply_safe {
            warn!(
                conflict_count = conflicts.len(),
                "Detected local modifications, cannot auto-sync"
            );
            return Ok(SyncResult {
                success: false,
                has_changes: true,
                diff: None,
                files_updated: Vec::new(),
                conflicts,
                error: Some("Local modifications detected. Use 'oxide figma diff' to review.".into()),
                duration: start.elapsed(),
            });
        }

        // Translate file
        let translator = Translator::new();
        let result = translator.translate(&file)?;

        // Compare with existing
        let diff = self.compute_diff(&result)?;

        // Check if we should apply changes
        let should_apply = if !diff.has_changes() {
            false
        } else if diff.has_breaking_changes() && !self.config.auto_apply_safe {
            warn!("Breaking changes detected, requires manual review");
            false
        } else {
            true
        };

        let mut files_updated = Vec::new();

        if should_apply {
            // Create backup if enabled
            if self.config.backup_enabled {
                self.create_backup()?;
            }

            // Apply changes
            files_updated = self.apply_changes(&result)?;
        }

        // Update state
        self.state.last_version = Some(file.version.clone());
        self.state.last_modified = Some(file.last_modified.clone());
        self.state.last_sync = Some(Utc::now());
        self.update_file_hashes(&files_updated)?;
        self.save_state()?;

        let sync_result = SyncResult {
            success: true,
            has_changes: diff.has_changes(),
            diff: Some(diff),
            files_updated,
            conflicts: Vec::new(),
            error: None,
            duration: start.elapsed(),
        };

        info!(
            files_updated = sync_result.files_updated.len(),
            duration_ms = sync_result.duration.as_millis(),
            "Sync complete"
        );

        Ok(sync_result)
    }

    /// Start continuous sync loop
    pub async fn start_continuous(&mut self) -> Result<()> {
        info!(
            poll_interval_secs = self.config.poll_interval.as_secs(),
            "Starting continuous sync"
        );

        let mut interval = interval(self.config.poll_interval);

        loop {
            interval.tick().await;

            match self.sync().await {
                Ok(result) => {
                    if result.has_changes {
                        info!(
                            files_updated = result.files_updated.len(),
                            "Sync detected changes"
                        );
                    }
                }
                Err(e) => {
                    error!(error = %e, "Sync failed");
                }
            }
        }
    }

    /// Pull latest changes (one-time sync)
    pub async fn pull(&mut self) -> Result<SyncResult> {
        self.sync().await
    }

    /// Detect conflicts with local files
    fn detect_conflicts(&self) -> Result<Vec<SyncConflict>> {
        let mut conflicts = Vec::new();

        for (path, expected_hash) in &self.state.file_hashes {
            let full_path = self.config.project_dir.join(path);

            if full_path.exists() {
                let content = fs::read_to_string(&full_path)?;
                let actual_hash = self.hash_content(&content);

                if actual_hash != *expected_hash {
                    conflicts.push(SyncConflict {
                        file_path: path.clone(),
                        reason: ConflictReason::LocalModification,
                        local_hash: actual_hash,
                        remote_hash: expected_hash.clone(),
                    });
                }
            }
        }

        Ok(conflicts)
    }

    /// Compute diff with existing files
    fn compute_diff(&self, result: &TranslationResult) -> Result<DiffResult> {
        let theme_path = self.config.project_dir
            .join(&self.config.output_dir)
            .join("theme.generated.toml");

        if theme_path.exists() {
            let existing = fs::read_to_string(&theme_path)?;
            let existing_theme = oxide_components::Theme::from_toml(&existing)?;

            let diff_engine = DesignDiff::new();
            Ok(diff_engine.compare_tokens(&result.tokens, &existing_theme))
        } else {
            // No existing theme
            Ok(DiffResult {
                color_changes: Vec::new(),
                spacing_changes: Vec::new(),
                typography_changes: Vec::new(),
                radius_changes: Vec::new(),
                shadow_changes: Vec::new(),
                component_changes: Vec::new(),
                summary: Default::default(),
            })
        }
    }

    /// Apply translation result to files
    fn apply_changes(&self, result: &TranslationResult) -> Result<Vec<String>> {
        let mut updated = Vec::new();

        let output_dir = self.config.project_dir.join(&self.config.output_dir);
        fs::create_dir_all(&output_dir)?;

        // Write theme
        let theme_path = output_dir.join("theme.generated.toml");
        let theme_content = result.theme.to_toml()?;
        fs::write(&theme_path, &theme_content)?;
        updated.push(theme_path.to_string());

        // Write typography
        if !result.typography_roles.is_empty() {
            let typo_path = output_dir.join("typography.generated.toml");
            let typo_content = toml::to_string_pretty(&result.typography_roles)?;
            fs::write(&typo_path, &typo_content)?;
            updated.push(typo_path.to_string());
        }

        // Write component mappings
        if !result.component_mappings.is_empty() {
            let components_path = output_dir.join("components.generated.json");
            let components_content = serde_json::to_string_pretty(&result.component_mappings)?;
            fs::write(&components_path, &components_content)?;
            updated.push(components_path.to_string());
        }

        // Write layouts
        if !result.layouts.is_empty() {
            let layouts_path = output_dir.join("layouts.generated.json");
            let layouts_content = serde_json::to_string_pretty(&result.layouts)?;
            fs::write(&layouts_path, &layouts_content)?;
            updated.push(layouts_path.to_string());
        }

        Ok(updated)
    }

    /// Create backup of existing files
    fn create_backup(&self) -> Result<()> {
        let output_dir = self.config.project_dir.join(&self.config.output_dir);
        let backup_dir = output_dir.join(".backups");
        fs::create_dir_all(&backup_dir)?;

        let timestamp = Utc::now().format("%Y%m%d_%H%M%S");
        let backup_name = format!("backup_{}", timestamp);
        let backup_path = backup_dir.join(&backup_name);
        fs::create_dir_all(&backup_path)?;

        // Copy generated files
        for entry in fs::read_dir(&output_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() && path.extension().map_or(false, |e| e == "toml" || e == "json") {
                let file_name = path.file_name().unwrap().to_string_lossy();
                let dest = backup_path.join(file_name.as_ref());
                fs::copy(&path, dest)?;
            }
        }

        // Cleanup old backups
        self.cleanup_old_backups(&backup_dir)?;

        info!(?backup_path, "Created backup");

        Ok(())
    }

    /// Cleanup old backups
    fn cleanup_old_backups(&self, backup_dir: &Utf8Path) -> Result<()> {
        let mut backups: Vec<_> = fs::read_dir(backup_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_dir())
            .collect();

        if backups.len() > self.config.max_backups {
            // Sort by modification time (oldest first)
            backups.sort_by_key(|e| e.metadata().and_then(|m| m.modified()).ok());

            // Remove oldest backups
            let to_remove = backups.len() - self.config.max_backups;
            for entry in backups.into_iter().take(to_remove) {
                fs::remove_dir_all(entry.path())?;
                debug!("Removed old backup: {:?}", entry.path());
            }
        }

        Ok(())
    }

    /// Update file hashes in state
    fn update_file_hashes(&mut self, files: &[String]) -> Result<()> {
        for file_path in files {
            if let Ok(content) = fs::read_to_string(file_path) {
                let hash = self.hash_content(&content);

                // Store relative path
                let relative = file_path
                    .strip_prefix(self.config.project_dir.as_str())
                    .unwrap_or(file_path)
                    .trim_start_matches('/');

                self.state.file_hashes.insert(relative.to_string(), hash);
            }
        }
        Ok(())
    }

    /// Hash file content
    fn hash_content(&self, content: &str) -> String {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:016x}", hasher.finish())
    }
}

/// Sync configuration builder
#[derive(Debug, Default)]
pub struct SyncConfigBuilder {
    config: SyncConfig,
}

impl SyncConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn file_key(mut self, key: impl Into<String>) -> Self {
        self.config.file_key = key.into();
        self
    }

    pub fn project_dir(mut self, dir: impl Into<Utf8PathBuf>) -> Self {
        self.config.project_dir = dir.into();
        self
    }

    pub fn output_dir(mut self, dir: impl Into<Utf8PathBuf>) -> Self {
        self.config.output_dir = dir.into();
        self
    }

    pub fn poll_interval(mut self, interval: Duration) -> Self {
        self.config.poll_interval = interval;
        self
    }

    pub fn auto_apply_safe(mut self, auto: bool) -> Self {
        self.config.auto_apply_safe = auto;
        self
    }

    pub fn download_assets(mut self, download: bool) -> Self {
        self.config.download_assets = download;
        self
    }

    pub fn assets_dir(mut self, dir: impl Into<Utf8PathBuf>) -> Self {
        self.config.assets_dir = dir.into();
        self
    }

    pub fn backup_enabled(mut self, enabled: bool) -> Self {
        self.config.backup_enabled = enabled;
        self
    }

    pub fn build(self) -> SyncConfig {
        self.config
    }
}

// Implement humantime_serde for Duration
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let secs = duration.as_secs();
        format!("{}s", secs).serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let secs: u64 = s
            .trim_end_matches('s')
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Duration::from_secs(secs))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_config_builder() {
        let config = SyncConfigBuilder::new()
            .file_key("abc123")
            .project_dir("/my/project")
            .output_dir("generated")
            .poll_interval(Duration::from_secs(120))
            .auto_apply_safe(true)
            .build();

        assert_eq!(config.file_key, "abc123");
        assert_eq!(config.project_dir.as_str(), "/my/project");
        assert_eq!(config.output_dir.as_str(), "generated");
        assert_eq!(config.poll_interval.as_secs(), 120);
        assert!(config.auto_apply_safe);
    }

    #[test]
    fn test_hash_content() {
        let client = FigmaClient::new(FigmaConfig::with_token("test"));
        let config = SyncConfig::default();
        let engine = SyncEngine::new(client, config);

        let hash1 = engine.hash_content("hello world");
        let hash2 = engine.hash_content("hello world");
        let hash3 = engine.hash_content("different content");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
