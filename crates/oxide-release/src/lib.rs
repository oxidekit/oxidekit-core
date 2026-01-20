//! OxideKit Release Automation System
//!
//! A comprehensive release, signing, packaging, and publishing system for OxideKit applications.
//! This crate handles the entire release lifecycle from version bumping to marketplace publishing.
//!
//! # Features
//!
//! - **Code Signing**: Platform-specific code signing for macOS and Windows
//! - **Notarization**: Apple notarization workflow for macOS apps
//! - **Packaging**: Cross-platform packaging (DMG, MSI, AppImage)
//! - **Changelog**: Automatic changelog generation from git history
//! - **Versioning**: Semantic versioning automation
//! - **GitHub Releases**: Automated GitHub release creation
//! - **Publishing**: Marketplace publishing for extensions, themes, and apps
//! - **Updates**: Update manifest generation for auto-updater
//!
//! # Example
//!
//! ```no_run
//! use oxide_release::{ReleaseConfig, ReleaseBuilder, ReleaseChannel};
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let config = ReleaseConfig::from_project(".")?;
//!
//!     let release = ReleaseBuilder::new(config)
//!         .channel(ReleaseChannel::Stable)
//!         .build_all_platforms()
//!         .sign()
//!         .notarize()
//!         .package()
//!         .generate_changelog()
//!         .create_github_release()
//!         .build()
//!         .await?;
//!
//!     println!("Release {} created successfully!", release.version);
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

mod config;
mod error;
mod artifact;
mod checksum;
mod doctor;

#[cfg(feature = "signing")]
pub mod signing;

#[cfg(feature = "notarization")]
pub mod notarization;

#[cfg(feature = "packaging")]
pub mod packaging;

#[cfg(feature = "changelog")]
pub mod changelog;

pub mod versioning;

#[cfg(feature = "github")]
pub mod github;

#[cfg(feature = "publish")]
pub mod publish;

#[cfg(feature = "update")]
pub mod update;

// Re-exports
pub use config::*;
pub use error::*;
pub use artifact::*;
pub use checksum::*;
pub use doctor::*;

/// Release channel for distribution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReleaseChannel {
    /// Stable release channel
    Stable,
    /// Beta release channel
    Beta,
    /// Nightly/development release channel
    Nightly,
    /// Custom channel
    #[serde(other)]
    Custom,
}

impl Default for ReleaseChannel {
    fn default() -> Self {
        Self::Stable
    }
}

impl std::fmt::Display for ReleaseChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Stable => write!(f, "stable"),
            Self::Beta => write!(f, "beta"),
            Self::Nightly => write!(f, "nightly"),
            Self::Custom => write!(f, "custom"),
        }
    }
}

impl std::str::FromStr for ReleaseChannel {
    type Err = ReleaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "stable" => Ok(Self::Stable),
            "beta" => Ok(Self::Beta),
            "nightly" => Ok(Self::Nightly),
            _ => Ok(Self::Custom),
        }
    }
}

/// Target platform for builds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetPlatform {
    /// macOS (Apple Silicon)
    MacOSArm64,
    /// macOS (Intel)
    MacOSX64,
    /// macOS Universal (both architectures)
    MacOSUniversal,
    /// Windows x64
    WindowsX64,
    /// Windows ARM64
    WindowsArm64,
    /// Linux x64
    LinuxX64,
    /// Linux ARM64
    LinuxArm64,
}

impl TargetPlatform {
    /// Get the Rust target triple for this platform
    pub fn rust_target(&self) -> &'static str {
        match self {
            Self::MacOSArm64 => "aarch64-apple-darwin",
            Self::MacOSX64 => "x86_64-apple-darwin",
            Self::MacOSUniversal => "universal-apple-darwin",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
            Self::WindowsArm64 => "aarch64-pc-windows-msvc",
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64 => "aarch64-unknown-linux-gnu",
        }
    }

    /// Get the platform name for display
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::MacOSArm64 => "macOS (Apple Silicon)",
            Self::MacOSX64 => "macOS (Intel)",
            Self::MacOSUniversal => "macOS (Universal)",
            Self::WindowsX64 => "Windows (x64)",
            Self::WindowsArm64 => "Windows (ARM64)",
            Self::LinuxX64 => "Linux (x64)",
            Self::LinuxArm64 => "Linux (ARM64)",
        }
    }

    /// Get the current host platform
    pub fn current() -> Self {
        #[cfg(all(target_os = "macos", target_arch = "aarch64"))]
        return Self::MacOSArm64;

        #[cfg(all(target_os = "macos", target_arch = "x86_64"))]
        return Self::MacOSX64;

        #[cfg(all(target_os = "windows", target_arch = "x86_64"))]
        return Self::WindowsX64;

        #[cfg(all(target_os = "windows", target_arch = "aarch64"))]
        return Self::WindowsArm64;

        #[cfg(all(target_os = "linux", target_arch = "x86_64"))]
        return Self::LinuxX64;

        #[cfg(all(target_os = "linux", target_arch = "aarch64"))]
        return Self::LinuxArm64;

        #[cfg(not(any(
            all(target_os = "macos", target_arch = "aarch64"),
            all(target_os = "macos", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "x86_64"),
            all(target_os = "windows", target_arch = "aarch64"),
            all(target_os = "linux", target_arch = "x86_64"),
            all(target_os = "linux", target_arch = "aarch64"),
        )))]
        return Self::LinuxX64; // Default fallback
    }

    /// Check if this is a macOS target
    pub fn is_macos(&self) -> bool {
        matches!(
            self,
            Self::MacOSArm64 | Self::MacOSX64 | Self::MacOSUniversal
        )
    }

    /// Check if this is a Windows target
    pub fn is_windows(&self) -> bool {
        matches!(self, Self::WindowsX64 | Self::WindowsArm64)
    }

    /// Check if this is a Linux target
    pub fn is_linux(&self) -> bool {
        matches!(self, Self::LinuxX64 | Self::LinuxArm64)
    }
}

impl std::fmt::Display for TargetPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rust_target())
    }
}

/// Release builder for fluent API
pub struct ReleaseBuilder {
    config: ReleaseConfig,
    channel: ReleaseChannel,
    targets: Vec<TargetPlatform>,
    dry_run: bool,
    skip_signing: bool,
    skip_notarization: bool,
    skip_packaging: bool,
    skip_changelog: bool,
    skip_github: bool,
    notes: Option<String>,
    notes_mode: NotesMode,
}

/// Mode for release notes generation
#[derive(Debug, Clone, Copy, Default)]
pub enum NotesMode {
    /// Automatically generate from git history
    #[default]
    Auto,
    /// Use manually provided notes
    Manual,
    /// No release notes
    None,
}

impl ReleaseBuilder {
    /// Create a new release builder with the given configuration
    pub fn new(config: ReleaseConfig) -> Self {
        Self {
            config,
            channel: ReleaseChannel::Stable,
            targets: vec![TargetPlatform::current()],
            dry_run: false,
            skip_signing: false,
            skip_notarization: false,
            skip_packaging: false,
            skip_changelog: false,
            skip_github: false,
            notes: None,
            notes_mode: NotesMode::Auto,
        }
    }

    /// Set the release channel
    pub fn channel(mut self, channel: ReleaseChannel) -> Self {
        self.channel = channel;
        self
    }

    /// Set target platforms
    pub fn targets(mut self, targets: Vec<TargetPlatform>) -> Self {
        self.targets = targets;
        self
    }

    /// Build for the current platform only
    pub fn build_current_platform(mut self) -> Self {
        self.targets = vec![TargetPlatform::current()];
        self
    }

    /// Build for all supported platforms
    pub fn build_all_platforms(mut self) -> Self {
        self.targets = vec![
            TargetPlatform::MacOSArm64,
            TargetPlatform::MacOSX64,
            TargetPlatform::WindowsX64,
            TargetPlatform::LinuxX64,
        ];
        self
    }

    /// Enable dry run mode (no actual changes)
    pub fn dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    /// Skip code signing
    pub fn skip_signing(mut self) -> Self {
        self.skip_signing = true;
        self
    }

    /// Skip notarization
    pub fn skip_notarization(mut self) -> Self {
        self.skip_notarization = true;
        self
    }

    /// Skip packaging
    pub fn skip_packaging(mut self) -> Self {
        self.skip_packaging = true;
        self
    }

    /// Skip changelog generation
    pub fn skip_changelog(mut self) -> Self {
        self.skip_changelog = true;
        self
    }

    /// Skip GitHub release
    pub fn skip_github(mut self) -> Self {
        self.skip_github = true;
        self
    }

    /// Set manual release notes
    pub fn notes(mut self, notes: impl Into<String>) -> Self {
        self.notes = Some(notes.into());
        self.notes_mode = NotesMode::Manual;
        self
    }

    /// Set release notes mode
    pub fn notes_mode(mut self, mode: NotesMode) -> Self {
        self.notes_mode = mode;
        self
    }

    /// Execute the release process
    pub async fn build(self) -> Result<Release, ReleaseError> {
        let mut release = Release {
            version: self.config.version.clone(),
            channel: self.channel,
            artifacts: Vec::new(),
            changelog: None,
            github_release_url: None,
            created_at: chrono::Utc::now(),
        };

        tracing::info!(
            "Starting release {} on {} channel",
            release.version,
            release.channel
        );

        if self.dry_run {
            tracing::info!("Dry run mode - no actual changes will be made");
        }

        // Build for each target
        for target in &self.targets {
            tracing::info!("Building for {}", target.display_name());

            if !self.dry_run {
                // Actual build logic would go here
                let artifact = Artifact {
                    name: format!("{}-{}-{}", self.config.app_name, release.version, target),
                    platform: *target,
                    path: std::path::PathBuf::from(format!(
                        "target/{}/release/{}",
                        target.rust_target(),
                        self.config.app_name
                    )),
                    checksum: None,
                    signature: None,
                    size: 0,
                };
                release.artifacts.push(artifact);
            }
        }

        // Sign artifacts
        #[cfg(feature = "signing")]
        if !self.skip_signing && !self.dry_run {
            tracing::info!("Signing artifacts...");
            for artifact in &mut release.artifacts {
                if let Some(ref signing_config) = self.config.signing {
                    if let Ok(sig) = signing::sign_artifact(artifact, signing_config).await {
                        artifact.signature = Some(sig);
                    }
                }
            }
        }

        // Notarize (macOS only)
        #[cfg(feature = "notarization")]
        if !self.skip_notarization && !self.dry_run {
            for artifact in &release.artifacts {
                if artifact.platform.is_macos() {
                    if let Some(ref notarization_config) = self.config.notarization {
                        tracing::info!("Notarizing {}...", artifact.name);
                        notarization::notarize_artifact(artifact, notarization_config).await?;
                    }
                }
            }
        }

        // Package artifacts
        #[cfg(feature = "packaging")]
        if !self.skip_packaging && !self.dry_run {
            tracing::info!("Packaging artifacts...");
            for artifact in &mut release.artifacts {
                let packaged = packaging::package_artifact(artifact, &self.config).await?;
                *artifact = packaged;
            }
        }

        // Generate checksums
        if !self.dry_run {
            for artifact in &mut release.artifacts {
                if artifact.path.exists() {
                    artifact.checksum = Some(checksum::calculate_sha256(&artifact.path)?);
                    artifact.size = std::fs::metadata(&artifact.path)
                        .map(|m| m.len())
                        .unwrap_or(0);
                }
            }
        }

        // Generate changelog
        #[cfg(feature = "changelog")]
        if !self.skip_changelog && !self.dry_run {
            tracing::info!("Generating changelog...");
            let changelog = match self.notes_mode {
                NotesMode::Auto => changelog::generate_changelog(&self.config).await?,
                NotesMode::Manual => self.notes.unwrap_or_default(),
                NotesMode::None => String::new(),
            };
            release.changelog = Some(changelog);
        }

        // Create GitHub release
        #[cfg(feature = "github")]
        if !self.skip_github && !self.dry_run {
            if let Some(ref github_config) = self.config.github {
                tracing::info!("Creating GitHub release...");
                let url =
                    github::create_release(&release, github_config, self.config.app_name.clone())
                        .await?;
                release.github_release_url = Some(url);
            }
        }

        tracing::info!("Release {} completed successfully!", release.version);
        Ok(release)
    }
}

/// A completed release
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Release {
    /// Release version
    pub version: String,
    /// Release channel
    pub channel: ReleaseChannel,
    /// Release artifacts
    pub artifacts: Vec<Artifact>,
    /// Changelog content
    pub changelog: Option<String>,
    /// GitHub release URL
    pub github_release_url: Option<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl Release {
    /// Get artifacts for a specific platform
    pub fn artifacts_for_platform(&self, platform: TargetPlatform) -> Vec<&Artifact> {
        self.artifacts
            .iter()
            .filter(|a| a.platform == platform)
            .collect()
    }

    /// Generate a release manifest for auto-updater
    #[cfg(feature = "update")]
    pub fn to_update_manifest(&self) -> update::UpdateManifest {
        update::UpdateManifest::from_release(self)
    }
}

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        Artifact, Checksum, NotesMode, Release, ReleaseBuilder, ReleaseChannel, ReleaseConfig,
        ReleaseError, TargetPlatform,
    };

    #[cfg(feature = "signing")]
    pub use crate::signing::{SigningConfig, SigningIdentity};

    #[cfg(feature = "notarization")]
    pub use crate::notarization::NotarizationConfig;

    #[cfg(feature = "packaging")]
    pub use crate::packaging::{PackageFormat, PackagingConfig};

    #[cfg(feature = "changelog")]
    pub use crate::changelog::{ChangelogConfig, ChangelogEntry};

    #[cfg(feature = "github")]
    pub use crate::github::GitHubConfig;

    #[cfg(feature = "publish")]
    pub use crate::publish::{PublishConfig, PublishTarget};

    #[cfg(feature = "update")]
    pub use crate::update::UpdateManifest;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_release_channel_parsing() {
        assert_eq!(
            "stable".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Stable
        );
        assert_eq!(
            "beta".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Beta
        );
        assert_eq!(
            "nightly".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Nightly
        );
        assert_eq!(
            "custom".parse::<ReleaseChannel>().unwrap(),
            ReleaseChannel::Custom
        );
    }

    #[test]
    fn test_target_platform_current() {
        let current = TargetPlatform::current();
        // Just verify it returns something valid
        assert!(!current.rust_target().is_empty());
        assert!(!current.display_name().is_empty());
    }

    #[test]
    fn test_target_platform_classification() {
        assert!(TargetPlatform::MacOSArm64.is_macos());
        assert!(TargetPlatform::MacOSX64.is_macos());
        assert!(TargetPlatform::MacOSUniversal.is_macos());
        assert!(!TargetPlatform::MacOSArm64.is_windows());
        assert!(!TargetPlatform::MacOSArm64.is_linux());

        assert!(TargetPlatform::WindowsX64.is_windows());
        assert!(TargetPlatform::WindowsArm64.is_windows());
        assert!(!TargetPlatform::WindowsX64.is_macos());
        assert!(!TargetPlatform::WindowsX64.is_linux());

        assert!(TargetPlatform::LinuxX64.is_linux());
        assert!(TargetPlatform::LinuxArm64.is_linux());
        assert!(!TargetPlatform::LinuxX64.is_macos());
        assert!(!TargetPlatform::LinuxX64.is_windows());
    }
}
