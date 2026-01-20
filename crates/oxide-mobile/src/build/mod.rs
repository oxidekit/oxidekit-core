//! Build output types for mobile platforms.
//!
//! Defines the output artifacts from mobile builds including APK/AAB for Android
//! and IPA for iOS.

pub mod android;
pub mod ios;

use std::path::PathBuf;

use camino::Utf8PathBuf;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

pub use android::AndroidBuildOutput;
pub use ios::IosBuildOutput;

use crate::config::BuildProfile;
use crate::target::MobileTarget;

/// Common build output information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOutput {
    /// Target platform.
    pub target: MobileTarget,
    /// Build profile used.
    pub profile: BuildProfile,
    /// App name.
    pub app_name: String,
    /// App version.
    pub version: String,
    /// Build number.
    pub build_number: u32,
    /// Build timestamp.
    pub timestamp: DateTime<Utc>,
    /// Total build duration in seconds.
    pub duration_secs: f64,
    /// Output directory.
    pub output_dir: Utf8PathBuf,
}

impl BuildOutput {
    /// Create a new build output.
    pub fn new(
        target: MobileTarget,
        profile: BuildProfile,
        app_name: impl Into<String>,
        version: impl Into<String>,
        build_number: u32,
        output_dir: impl Into<Utf8PathBuf>,
    ) -> Self {
        Self {
            target,
            profile,
            app_name: app_name.into(),
            version: version.into(),
            build_number,
            timestamp: Utc::now(),
            duration_secs: 0.0,
            output_dir: output_dir.into(),
        }
    }

    /// Set the build duration.
    pub fn with_duration(mut self, duration_secs: f64) -> Self {
        self.duration_secs = duration_secs;
        self
    }

    /// Get the version string with build number.
    pub fn full_version(&self) -> String {
        format!("{} ({})", self.version, self.build_number)
    }

    /// Check if this is a release build.
    pub fn is_release(&self) -> bool {
        self.profile.is_release()
    }
}

/// Build artifact with path and size information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Path to the artifact.
    pub path: PathBuf,
    /// Size in bytes.
    pub size_bytes: u64,
    /// SHA256 hash of the artifact.
    pub sha256: Option<String>,
    /// Type of artifact.
    pub artifact_type: ArtifactType,
}

impl BuildArtifact {
    /// Create a new build artifact.
    pub fn new(path: impl Into<PathBuf>, artifact_type: ArtifactType) -> Self {
        Self {
            path: path.into(),
            size_bytes: 0,
            sha256: None,
            artifact_type,
        }
    }

    /// Set the file size.
    pub fn with_size(mut self, size_bytes: u64) -> Self {
        self.size_bytes = size_bytes;
        self
    }

    /// Set the SHA256 hash.
    pub fn with_hash(mut self, sha256: impl Into<String>) -> Self {
        self.sha256 = Some(sha256.into());
        self
    }

    /// Get the file name.
    pub fn file_name(&self) -> Option<&str> {
        self.path.file_name()?.to_str()
    }

    /// Get the file extension.
    pub fn extension(&self) -> Option<&str> {
        self.path.extension()?.to_str()
    }

    /// Get a human-readable size string.
    pub fn size_string(&self) -> String {
        let size = self.size_bytes as f64;
        if size < 1024.0 {
            format!("{} B", self.size_bytes)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else if size < 1024.0 * 1024.0 * 1024.0 {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", size / (1024.0 * 1024.0 * 1024.0))
        }
    }
}

/// Type of build artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ArtifactType {
    /// Android APK file.
    Apk,
    /// Android App Bundle (AAB) file.
    Aab,
    /// iOS App file.
    App,
    /// iOS IPA file.
    Ipa,
    /// dSYM debug symbols (iOS).
    Dsym,
    /// Proguard mapping file (Android).
    ProguardMapping,
    /// Native debug symbols.
    DebugSymbols,
    /// Build log.
    BuildLog,
    /// Other artifact type.
    Other,
}

impl ArtifactType {
    /// Get the typical file extension for this artifact type.
    pub fn extension(&self) -> &'static str {
        match self {
            ArtifactType::Apk => "apk",
            ArtifactType::Aab => "aab",
            ArtifactType::App => "app",
            ArtifactType::Ipa => "ipa",
            ArtifactType::Dsym => "dSYM",
            ArtifactType::ProguardMapping => "txt",
            ArtifactType::DebugSymbols => "zip",
            ArtifactType::BuildLog => "log",
            ArtifactType::Other => "",
        }
    }

    /// Determine artifact type from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "apk" => ArtifactType::Apk,
            "aab" => ArtifactType::Aab,
            "app" => ArtifactType::App,
            "ipa" => ArtifactType::Ipa,
            "dsym" => ArtifactType::Dsym,
            "log" => ArtifactType::BuildLog,
            _ => ArtifactType::Other,
        }
    }

    /// Returns true if this is an installable artifact.
    pub fn is_installable(&self) -> bool {
        matches!(
            self,
            ArtifactType::Apk | ArtifactType::App | ArtifactType::Ipa
        )
    }

    /// Returns true if this is a distributable artifact.
    pub fn is_distributable(&self) -> bool {
        matches!(self, ArtifactType::Apk | ArtifactType::Aab | ArtifactType::Ipa)
    }
}

impl std::fmt::Display for ArtifactType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            ArtifactType::Apk => "APK",
            ArtifactType::Aab => "AAB",
            ArtifactType::App => "App",
            ArtifactType::Ipa => "IPA",
            ArtifactType::Dsym => "dSYM",
            ArtifactType::ProguardMapping => "ProGuard Mapping",
            ArtifactType::DebugSymbols => "Debug Symbols",
            ArtifactType::BuildLog => "Build Log",
            ArtifactType::Other => "Other",
        };
        write!(f, "{}", name)
    }
}

/// Build status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BuildStatus {
    /// Build is pending.
    Pending,
    /// Build is in progress.
    InProgress,
    /// Build succeeded.
    Succeeded,
    /// Build failed.
    Failed,
    /// Build was cancelled.
    Cancelled,
}

impl BuildStatus {
    /// Returns true if the build has finished (success or failure).
    pub fn is_finished(&self) -> bool {
        matches!(
            self,
            BuildStatus::Succeeded | BuildStatus::Failed | BuildStatus::Cancelled
        )
    }

    /// Returns true if the build succeeded.
    pub fn succeeded(&self) -> bool {
        matches!(self, BuildStatus::Succeeded)
    }
}

/// Build step for progress tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildStep {
    /// Step name.
    pub name: String,
    /// Step description.
    pub description: String,
    /// Step status.
    pub status: BuildStatus,
    /// Duration in seconds.
    pub duration_secs: Option<f64>,
    /// Error message if failed.
    pub error: Option<String>,
}

impl BuildStep {
    /// Create a new build step.
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            status: BuildStatus::Pending,
            duration_secs: None,
            error: None,
        }
    }

    /// Mark the step as in progress.
    pub fn start(&mut self) {
        self.status = BuildStatus::InProgress;
    }

    /// Mark the step as succeeded.
    pub fn succeed(&mut self, duration_secs: f64) {
        self.status = BuildStatus::Succeeded;
        self.duration_secs = Some(duration_secs);
    }

    /// Mark the step as failed.
    pub fn fail(&mut self, error: impl Into<String>) {
        self.status = BuildStatus::Failed;
        self.error = Some(error.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_artifact_size_string() {
        let artifact = BuildArtifact::new("/path/to/app.apk", ArtifactType::Apk);

        let small = artifact.clone().with_size(500);
        assert_eq!(small.size_string(), "500 B");

        let kb = artifact.clone().with_size(1536);
        assert_eq!(kb.size_string(), "1.5 KB");

        let mb = artifact.clone().with_size(15 * 1024 * 1024);
        assert_eq!(mb.size_string(), "15.0 MB");
    }

    #[test]
    fn test_artifact_type_from_extension() {
        assert_eq!(ArtifactType::from_extension("apk"), ArtifactType::Apk);
        assert_eq!(ArtifactType::from_extension("aab"), ArtifactType::Aab);
        assert_eq!(ArtifactType::from_extension("ipa"), ArtifactType::Ipa);
        assert_eq!(ArtifactType::from_extension("xyz"), ArtifactType::Other);
    }

    #[test]
    fn test_artifact_type_properties() {
        assert!(ArtifactType::Apk.is_installable());
        assert!(ArtifactType::Ipa.is_installable());
        assert!(!ArtifactType::Aab.is_installable());

        assert!(ArtifactType::Apk.is_distributable());
        assert!(ArtifactType::Aab.is_distributable());
        assert!(!ArtifactType::Dsym.is_distributable());
    }

    #[test]
    fn test_build_status() {
        assert!(BuildStatus::Succeeded.is_finished());
        assert!(BuildStatus::Failed.is_finished());
        assert!(!BuildStatus::InProgress.is_finished());

        assert!(BuildStatus::Succeeded.succeeded());
        assert!(!BuildStatus::Failed.succeeded());
    }

    #[test]
    fn test_build_step() {
        let mut step = BuildStep::new("compile", "Compiling Rust code");
        assert!(matches!(step.status, BuildStatus::Pending));

        step.start();
        assert!(matches!(step.status, BuildStatus::InProgress));

        step.succeed(5.5);
        assert!(matches!(step.status, BuildStatus::Succeeded));
        assert_eq!(step.duration_secs, Some(5.5));
    }
}
