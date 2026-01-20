//! Update management state

use anyhow::Result;
use chrono::{DateTime, Utc};
use semver::Version;
use serde::{Deserialize, Serialize};

/// Information about available updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    /// Current installed version
    pub current_version: String,

    /// Available OxideKit core updates
    pub core_update: Option<UpdateItem>,

    /// Available plugin updates
    pub plugin_updates: Vec<PluginUpdate>,

    /// Last check time
    pub last_check: DateTime<Utc>,

    /// Update check status
    pub status: UpdateCheckStatus,

    /// Error message if check failed
    pub error: Option<String>,
}

/// Status of update check
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum UpdateCheckStatus {
    #[default]
    Idle,
    Checking,
    Available,
    UpToDate,
    Error,
}

/// An available update item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateItem {
    /// Package name
    pub name: String,

    /// Current version
    pub current_version: String,

    /// New version
    pub new_version: String,

    /// Release date
    pub release_date: DateTime<Utc>,

    /// Release notes (markdown)
    pub release_notes: String,

    /// Download URL
    pub download_url: String,

    /// Download size in bytes
    pub download_size: u64,

    /// Whether this is a critical/security update
    pub critical: bool,

    /// Changelog URL
    pub changelog_url: Option<String>,
}

/// Plugin update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginUpdate {
    /// Plugin ID
    pub plugin_id: String,

    /// Plugin name
    pub plugin_name: String,

    /// Current version
    pub current_version: String,

    /// New version
    pub new_version: String,

    /// Update item details
    pub update: UpdateItem,
}

impl UpdateInfo {
    /// Create a new update info with current version
    pub fn new() -> Self {
        Self {
            current_version: env!("CARGO_PKG_VERSION").to_string(),
            core_update: None,
            plugin_updates: Vec::new(),
            last_check: Utc::now(),
            status: UpdateCheckStatus::Idle,
            error: None,
        }
    }

    /// Check if any updates are available
    pub fn has_updates(&self) -> bool {
        self.core_update.is_some() || !self.plugin_updates.is_empty()
    }

    /// Get total count of available updates
    pub fn available_count(&self) -> usize {
        let core = if self.core_update.is_some() { 1 } else { 0 };
        core + self.plugin_updates.len()
    }

    /// Check if there's a critical update
    pub fn has_critical_update(&self) -> bool {
        self.core_update.as_ref().map(|u| u.critical).unwrap_or(false)
            || self.plugin_updates.iter().any(|u| u.update.critical)
    }

    /// Get all updates as a flat list
    pub fn all_updates(&self) -> Vec<&UpdateItem> {
        let mut updates: Vec<&UpdateItem> = Vec::new();
        if let Some(core) = &self.core_update {
            updates.push(core);
        }
        for plugin in &self.plugin_updates {
            updates.push(&plugin.update);
        }
        updates
    }

    /// Total download size
    pub fn total_download_size(&self) -> u64 {
        let mut size = 0;
        if let Some(core) = &self.core_update {
            size += core.download_size;
        }
        for plugin in &self.plugin_updates {
            size += plugin.update.download_size;
        }
        size
    }
}

impl Default for UpdateInfo {
    fn default() -> Self {
        Self::new()
    }
}

/// Update checker service
pub struct UpdateChecker {
    /// Update server URL
    pub server_url: String,

    /// Update channel (stable, beta, nightly)
    pub channel: String,

    /// Include pre-releases
    pub include_prereleases: bool,
}

impl UpdateChecker {
    /// Create a new update checker with default settings
    pub fn new() -> Self {
        Self {
            server_url: "https://api.oxidekit.com/updates".to_string(),
            channel: "stable".to_string(),
            include_prereleases: false,
        }
    }

    /// Create with custom settings
    pub fn with_channel(mut self, channel: &str) -> Self {
        self.channel = channel.to_string();
        self
    }

    /// Include pre-releases
    pub fn include_prereleases(mut self, include: bool) -> Self {
        self.include_prereleases = include;
        self
    }

    /// Check for updates (async)
    pub async fn check(&self) -> Result<UpdateInfo> {
        let mut info = UpdateInfo::new();
        info.status = UpdateCheckStatus::Checking;

        // In a real implementation, this would make HTTP requests to the update server
        // For now, we'll simulate the check
        let check_result = self.perform_check().await;

        match check_result {
            Ok((core_update, plugin_updates)) => {
                info.core_update = core_update;
                info.plugin_updates = plugin_updates;
                info.status = if info.has_updates() {
                    UpdateCheckStatus::Available
                } else {
                    UpdateCheckStatus::UpToDate
                };
                info.last_check = Utc::now();
                info.error = None;
            }
            Err(e) => {
                info.status = UpdateCheckStatus::Error;
                info.error = Some(e.to_string());
            }
        }

        Ok(info)
    }

    /// Perform the actual update check
    async fn perform_check(&self) -> Result<(Option<UpdateItem>, Vec<PluginUpdate>)> {
        // Simulate network delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        // In production, this would:
        // 1. Query the update server for the latest version
        // 2. Compare with current version
        // 3. Return update info if available

        let current = Version::parse(env!("CARGO_PKG_VERSION"))?;

        // Simulate: check if there's a newer version (for demo purposes, always return none)
        // In production, this would compare against the server response
        let _latest = current.clone();

        // For now, return no updates (version is current)
        Ok((None, Vec::new()))
    }

    /// Download an update
    pub async fn download(&self, update: &UpdateItem, progress_callback: impl Fn(u64, u64)) -> Result<Vec<u8>> {
        // In production, this would download the update file with progress reporting
        let total = update.download_size;

        // Simulate download progress
        for i in 0..10 {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            progress_callback((i + 1) * total / 10, total);
        }

        // Return empty data (in production, this would be the actual update file)
        Ok(Vec::new())
    }

    /// Install a downloaded update
    pub async fn install(&self, _update: &UpdateItem, _data: &[u8]) -> Result<()> {
        // In production, this would:
        // 1. Verify the update signature
        // 2. Extract the update files
        // 3. Replace the current installation
        // 4. Request restart

        Ok(())
    }
}

impl Default for UpdateChecker {
    fn default() -> Self {
        Self::new()
    }
}

/// Format bytes as human-readable size
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_update_info() {
        let info = UpdateInfo::new();
        assert!(!info.has_updates());
        assert_eq!(info.available_count(), 0);
    }

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(500), "500 bytes");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[tokio::test]
    async fn test_update_checker() {
        let checker = UpdateChecker::new();
        let info = checker.check().await.unwrap();
        assert!(matches!(info.status, UpdateCheckStatus::UpToDate | UpdateCheckStatus::Available));
    }
}
