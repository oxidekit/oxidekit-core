//! Update manifest generation for auto-updater
//!
//! Generates signed update manifests that apps can use to check for and download updates.

use crate::artifact::Artifact;
use crate::error::{ReleaseError, ReleaseResult};
use crate::{Release, ReleaseChannel, TargetPlatform};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Update manifest for auto-updater clients
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateManifest {
    /// Manifest version
    pub manifest_version: u32,
    /// App name
    pub app_name: String,
    /// Current version
    pub version: String,
    /// Release channel
    pub channel: ReleaseChannel,
    /// Release date
    pub release_date: chrono::DateTime<chrono::Utc>,
    /// Minimum version required to update (for delta updates)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,
    /// Release notes/changelog
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
    /// Platform-specific update info
    pub platforms: HashMap<String, PlatformUpdate>,
    /// Manifest signature
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

impl UpdateManifest {
    /// Create manifest from a release
    pub fn from_release(release: &Release) -> Self {
        let mut platforms = HashMap::new();

        for artifact in &release.artifacts {
            let platform_key = artifact.platform.rust_target().to_string();
            platforms.insert(
                platform_key,
                PlatformUpdate {
                    url: String::new(), // Set by caller
                    checksum: artifact.checksum.clone(),
                    size: artifact.size,
                    signature: artifact.signature.as_ref().map(|s| s.data.clone()),
                },
            );
        }

        Self {
            manifest_version: 1,
            app_name: String::new(), // Set by caller
            version: release.version.clone(),
            channel: release.channel,
            release_date: release.created_at,
            minimum_version: None,
            notes: release.changelog.clone(),
            platforms,
            signature: None,
        }
    }

    /// Set app name
    pub fn with_app_name(mut self, name: impl Into<String>) -> Self {
        self.app_name = name.into();
        self
    }

    /// Set download URLs for platforms
    pub fn with_urls(mut self, base_url: &str) -> Self {
        for (platform, update) in &mut self.platforms {
            update.url = format!("{}/{}", base_url, platform);
        }
        self
    }

    /// Set minimum version requirement
    pub fn with_minimum_version(mut self, version: impl Into<String>) -> Self {
        self.minimum_version = Some(version.into());
        self
    }

    /// Sign the manifest
    pub fn sign(mut self, private_key: &[u8]) -> ReleaseResult<Self> {
        let content = self.signable_content()?;
        let signature = sign_content(&content, private_key)?;
        self.signature = Some(signature);
        Ok(self)
    }

    /// Get content to sign (excludes signature field)
    fn signable_content(&self) -> ReleaseResult<Vec<u8>> {
        let mut manifest = self.clone();
        manifest.signature = None;
        Ok(serde_json::to_vec(&manifest)?)
    }

    /// Verify manifest signature
    pub fn verify(&self, public_key: &[u8]) -> ReleaseResult<bool> {
        let Some(ref signature) = self.signature else {
            return Ok(false);
        };

        let content = self.signable_content()?;
        verify_signature(&content, signature, public_key)
    }

    /// Save manifest to file
    pub fn save(&self, path: &Path) -> ReleaseResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Load manifest from file
    pub fn load(path: &Path) -> ReleaseResult<Self> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content)?)
    }
}

/// Platform-specific update information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformUpdate {
    /// Download URL
    pub url: String,
    /// SHA256 checksum
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checksum: Option<String>,
    /// File size in bytes
    pub size: u64,
    /// Signature of the artifact
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,
}

/// Sign content using Ed25519
fn sign_content(content: &[u8], _private_key: &[u8]) -> ReleaseResult<String> {
    // In production, use ed25519-dalek or similar
    // For now, return a placeholder
    let hash = crate::checksum::calculate_sha256_bytes(content);
    Ok(format!("ed25519:{}", hash))
}

/// Verify Ed25519 signature
fn verify_signature(content: &[u8], signature: &str, _public_key: &[u8]) -> ReleaseResult<bool> {
    // In production, use ed25519-dalek or similar
    // For now, verify placeholder format
    if let Some(hash) = signature.strip_prefix("ed25519:") {
        let computed = crate::checksum::calculate_sha256_bytes(content);
        return Ok(hash == computed);
    }
    Ok(false)
}

/// Update channel manifest (lists latest version per channel)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelManifest {
    /// Manifest version
    pub manifest_version: u32,
    /// App name
    pub app_name: String,
    /// Available channels with their latest versions
    pub channels: HashMap<String, ChannelInfo>,
}

/// Information about an update channel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelInfo {
    /// Latest version in this channel
    pub latest_version: String,
    /// URL to the update manifest for this version
    pub manifest_url: String,
    /// Whether this channel is recommended
    #[serde(default)]
    pub recommended: bool,
}

impl ChannelManifest {
    /// Create a new channel manifest
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            manifest_version: 1,
            app_name: app_name.into(),
            channels: HashMap::new(),
        }
    }

    /// Add a channel
    pub fn add_channel(
        mut self,
        channel: ReleaseChannel,
        version: impl Into<String>,
        manifest_url: impl Into<String>,
    ) -> Self {
        self.channels.insert(
            channel.to_string(),
            ChannelInfo {
                latest_version: version.into(),
                manifest_url: manifest_url.into(),
                recommended: channel == ReleaseChannel::Stable,
            },
        );
        self
    }

    /// Save to file
    pub fn save(&self, path: &Path) -> ReleaseResult<()> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Generate update manifest for a release
pub fn generate_update_manifest(
    release: &Release,
    app_name: &str,
    download_base_url: &str,
) -> UpdateManifest {
    let mut manifest = UpdateManifest::from_release(release).with_app_name(app_name);

    // Set download URLs
    for artifact in &release.artifacts {
        let platform_key = artifact.platform.rust_target().to_string();
        if let Some(platform_update) = manifest.platforms.get_mut(&platform_key) {
            let filename = artifact.path.file_name().unwrap().to_string_lossy();
            platform_update.url = format!("{}/{}", download_base_url, filename);
        }
    }

    manifest
}

/// Update server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateServerConfig {
    /// Base URL for updates
    pub base_url: String,
    /// Path to store manifests
    pub manifest_path: String,
    /// Enable anti-downgrade protection
    #[serde(default)]
    pub anti_downgrade: bool,
    /// Minimum supported version (for anti-downgrade)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,
}

/// Publish update manifest to server
pub async fn publish_update_manifest(
    manifest: &UpdateManifest,
    config: &UpdateServerConfig,
) -> ReleaseResult<String> {
    let manifest_url = format!(
        "{}/manifests/{}/{}.json",
        config.base_url, manifest.channel, manifest.version
    );

    // In production, upload via HTTP
    tracing::info!("Update manifest would be published to: {}", manifest_url);

    Ok(manifest_url)
}

/// Check for updates (client-side)
pub async fn check_for_updates(
    current_version: &str,
    channel: ReleaseChannel,
    manifest_url: &str,
) -> ReleaseResult<Option<UpdateInfo>> {
    // Fetch manifest
    let output = std::process::Command::new("curl")
        .args(["-s"])
        .arg(manifest_url)
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::UpdateMetadata("Failed to fetch manifest".to_string()));
    }

    let manifest: UpdateManifest = serde_json::from_slice(&output.stdout)?;

    // Compare versions
    let current = semver::Version::parse(current_version)?;
    let latest = semver::Version::parse(&manifest.version)?;

    if latest > current {
        let platform = TargetPlatform::current();
        let platform_key = platform.rust_target();

        if let Some(platform_update) = manifest.platforms.get(platform_key) {
            return Ok(Some(UpdateInfo {
                version: manifest.version,
                channel: manifest.channel,
                download_url: platform_update.url.clone(),
                size: platform_update.size,
                checksum: platform_update.checksum.clone(),
                notes: manifest.notes,
            }));
        }
    }

    Ok(None)
}

/// Update information for the client
#[derive(Debug, Clone)]
pub struct UpdateInfo {
    /// New version
    pub version: String,
    /// Release channel
    pub channel: ReleaseChannel,
    /// Download URL
    pub download_url: String,
    /// File size
    pub size: u64,
    /// Checksum
    pub checksum: Option<String>,
    /// Release notes
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_channel_manifest() {
        let manifest = ChannelManifest::new("test-app")
            .add_channel(
                ReleaseChannel::Stable,
                "1.0.0",
                "https://example.com/stable/1.0.0.json",
            )
            .add_channel(
                ReleaseChannel::Beta,
                "1.1.0-beta.1",
                "https://example.com/beta/1.1.0-beta.1.json",
            );

        assert_eq!(manifest.channels.len(), 2);
        assert!(manifest.channels.get("stable").unwrap().recommended);
        assert!(!manifest.channels.get("beta").unwrap().recommended);
    }

    #[test]
    fn test_update_manifest_serialization() {
        let manifest = UpdateManifest {
            manifest_version: 1,
            app_name: "test".to_string(),
            version: "1.0.0".to_string(),
            channel: ReleaseChannel::Stable,
            release_date: chrono::Utc::now(),
            minimum_version: None,
            notes: Some("Test release".to_string()),
            platforms: HashMap::new(),
            signature: None,
        };

        let json = serde_json::to_string(&manifest).unwrap();
        let parsed: UpdateManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.app_name, "test");
        assert_eq!(parsed.version, "1.0.0");
    }
}
