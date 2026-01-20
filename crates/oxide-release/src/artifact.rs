//! Release artifact types and utilities

use crate::{TargetPlatform, ReleaseError, ReleaseResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// A release artifact (binary, installer, package)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact name
    pub name: String,

    /// Target platform
    pub platform: TargetPlatform,

    /// Path to the artifact file
    pub path: PathBuf,

    /// SHA256 checksum
    pub checksum: Option<String>,

    /// Digital signature
    pub signature: Option<ArtifactSignature>,

    /// File size in bytes
    pub size: u64,
}

impl Artifact {
    /// Create a new artifact
    pub fn new(name: impl Into<String>, platform: TargetPlatform, path: impl Into<PathBuf>) -> Self {
        Self {
            name: name.into(),
            platform,
            path: path.into(),
            checksum: None,
            signature: None,
            size: 0,
        }
    }

    /// Get the artifact file extension
    pub fn extension(&self) -> Option<&str> {
        self.path.extension().and_then(|e| e.to_str())
    }

    /// Get the artifact MIME type
    pub fn mime_type(&self) -> &'static str {
        match self.extension() {
            Some("dmg") => "application/x-apple-diskimage",
            Some("pkg") => "application/x-newton-compatible-pkg",
            Some("msi") => "application/x-msi",
            Some("msix") => "application/msix",
            Some("exe") => "application/x-executable",
            Some("AppImage") => "application/x-executable",
            Some("deb") => "application/vnd.debian.binary-package",
            Some("rpm") => "application/x-rpm",
            Some("zip") => "application/zip",
            Some("tar") | Some("gz") | Some("tgz") => "application/gzip",
            _ => "application/octet-stream",
        }
    }

    /// Verify the artifact exists and matches its checksum
    pub fn verify(&self) -> ReleaseResult<bool> {
        if !self.path.exists() {
            return Err(ReleaseError::ArtifactNotFound(
                self.path.display().to_string(),
            ));
        }

        if let Some(ref expected) = self.checksum {
            let actual = crate::checksum::calculate_sha256(&self.path)?;
            if &actual != expected {
                return Err(ReleaseError::ChecksumMismatch {
                    expected: expected.clone(),
                    actual,
                });
            }
        }

        Ok(true)
    }

    /// Update the artifact's size and checksum from the actual file
    pub fn update_metadata(&mut self) -> ReleaseResult<()> {
        if !self.path.exists() {
            return Err(ReleaseError::ArtifactNotFound(
                self.path.display().to_string(),
            ));
        }

        let metadata = std::fs::metadata(&self.path)?;
        self.size = metadata.len();
        self.checksum = Some(crate::checksum::calculate_sha256(&self.path)?);

        Ok(())
    }

    /// Check if the artifact is signed
    pub fn is_signed(&self) -> bool {
        self.signature.is_some()
    }

    /// Create an artifact info for display
    pub fn info(&self) -> ArtifactInfo {
        ArtifactInfo {
            name: self.name.clone(),
            platform: self.platform.display_name().to_string(),
            path: self.path.display().to_string(),
            size: format_size(self.size),
            checksum: self.checksum.clone(),
            signed: self.is_signed(),
        }
    }
}

/// Artifact information for display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactInfo {
    /// Artifact name
    pub name: String,
    /// Platform name
    pub platform: String,
    /// File path
    pub path: String,
    /// Human-readable size
    pub size: String,
    /// Checksum
    pub checksum: Option<String>,
    /// Is signed
    pub signed: bool,
}

/// Digital signature for an artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactSignature {
    /// Signature algorithm
    pub algorithm: SignatureAlgorithm,

    /// Signer identity
    pub signer: String,

    /// Signature data (base64 encoded)
    pub data: String,

    /// Timestamp of signing
    pub timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Certificate chain (for verification)
    pub certificate_chain: Option<Vec<String>>,
}

/// Signature algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SignatureAlgorithm {
    /// Apple codesign
    AppleCodesign,
    /// Windows Authenticode
    Authenticode,
    /// GPG signature
    Gpg,
    /// Ed25519 signature (for updates)
    Ed25519,
}

impl std::fmt::Display for SignatureAlgorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppleCodesign => write!(f, "Apple Codesign"),
            Self::Authenticode => write!(f, "Windows Authenticode"),
            Self::Gpg => write!(f, "GPG"),
            Self::Ed25519 => write!(f, "Ed25519"),
        }
    }
}

/// Artifact collection for a release
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArtifactCollection {
    /// All artifacts
    pub artifacts: Vec<Artifact>,
}

impl ArtifactCollection {
    /// Create a new empty collection
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an artifact
    pub fn add(&mut self, artifact: Artifact) {
        self.artifacts.push(artifact);
    }

    /// Get artifacts for a specific platform
    pub fn for_platform(&self, platform: TargetPlatform) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.platform == platform).collect()
    }

    /// Get all macOS artifacts
    pub fn macos(&self) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.platform.is_macos()).collect()
    }

    /// Get all Windows artifacts
    pub fn windows(&self) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.platform.is_windows()).collect()
    }

    /// Get all Linux artifacts
    pub fn linux(&self) -> Vec<&Artifact> {
        self.artifacts.iter().filter(|a| a.platform.is_linux()).collect()
    }

    /// Verify all artifacts
    pub fn verify_all(&self) -> ReleaseResult<()> {
        for artifact in &self.artifacts {
            artifact.verify()?;
        }
        Ok(())
    }

    /// Generate checksums file content
    pub fn checksums_file(&self) -> String {
        let mut content = String::new();
        for artifact in &self.artifacts {
            if let Some(ref checksum) = artifact.checksum {
                content.push_str(&format!(
                    "{}  {}\n",
                    checksum,
                    artifact.path.file_name().unwrap_or_default().to_string_lossy()
                ));
            }
        }
        content
    }

    /// Total size of all artifacts
    pub fn total_size(&self) -> u64 {
        self.artifacts.iter().map(|a| a.size).sum()
    }
}

impl IntoIterator for ArtifactCollection {
    type Item = Artifact;
    type IntoIter = std::vec::IntoIter<Artifact>;

    fn into_iter(self) -> Self::IntoIter {
        self.artifacts.into_iter()
    }
}

impl<'a> IntoIterator for &'a ArtifactCollection {
    type Item = &'a Artifact;
    type IntoIter = std::slice::Iter<'a, Artifact>;

    fn into_iter(self) -> Self::IntoIter {
        self.artifacts.iter()
    }
}

/// Format file size for human readability
fn format_size(bytes: u64) -> String {
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
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1536), "1.50 KB");
        assert_eq!(format_size(1048576), "1.00 MB");
        assert_eq!(format_size(1073741824), "1.00 GB");
    }

    #[test]
    fn test_artifact_mime_type() {
        let artifact = Artifact::new("test", TargetPlatform::MacOSArm64, "test.dmg");
        assert_eq!(artifact.mime_type(), "application/x-apple-diskimage");

        let artifact = Artifact::new("test", TargetPlatform::WindowsX64, "test.msi");
        assert_eq!(artifact.mime_type(), "application/x-msi");

        let artifact = Artifact::new("test", TargetPlatform::LinuxX64, "test.AppImage");
        assert_eq!(artifact.mime_type(), "application/x-executable");
    }

    #[test]
    fn test_artifact_collection() {
        let mut collection = ArtifactCollection::new();
        collection.add(Artifact::new("mac", TargetPlatform::MacOSArm64, "test.dmg"));
        collection.add(Artifact::new("win", TargetPlatform::WindowsX64, "test.msi"));
        collection.add(Artifact::new("linux", TargetPlatform::LinuxX64, "test.AppImage"));

        assert_eq!(collection.macos().len(), 1);
        assert_eq!(collection.windows().len(), 1);
        assert_eq!(collection.linux().len(), 1);
    }
}
