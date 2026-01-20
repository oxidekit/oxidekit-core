//! Code signing for macOS and Windows
//!
//! This module provides platform-specific code signing functionality.
//! - macOS: Uses `codesign` with Developer ID certificates
//! - Windows: Uses `signtool` with Authenticode certificates

mod macos;
mod windows;

use crate::artifact::{Artifact, ArtifactSignature, SignatureAlgorithm};
use crate::config::SigningConfig;
use crate::error::{ReleaseError, ReleaseResult};

pub use macos::*;
pub use windows::*;

/// Sign an artifact using platform-appropriate signing
pub async fn sign_artifact(
    artifact: &Artifact,
    config: &SigningConfig,
) -> ReleaseResult<ArtifactSignature> {
    if !artifact.path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            artifact.path.display().to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        if artifact.platform.is_macos() {
            return macos::sign_macos(artifact, config).await;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if artifact.platform.is_windows() {
            return windows::sign_windows(artifact, config).await;
        }
    }

    // Cross-platform signing not supported for the target
    Err(ReleaseError::signing(format!(
        "Cannot sign {} artifact on current platform",
        artifact.platform.display_name()
    )))
}

/// Verify an artifact's signature
pub async fn verify_signature(artifact: &Artifact) -> ReleaseResult<bool> {
    if !artifact.path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            artifact.path.display().to_string(),
        ));
    }

    #[cfg(target_os = "macos")]
    {
        if artifact.platform.is_macos() {
            return macos::verify_macos_signature(artifact).await;
        }
    }

    #[cfg(target_os = "windows")]
    {
        if artifact.platform.is_windows() {
            return windows::verify_windows_signature(artifact).await;
        }
    }

    // Cannot verify on this platform
    Ok(false)
}

/// Signing identity information
#[derive(Debug, Clone)]
pub struct SigningIdentity {
    /// Identity name/common name
    pub name: String,
    /// Certificate fingerprint/thumbprint
    pub fingerprint: String,
    /// Whether the identity is valid (not expired)
    pub is_valid: bool,
    /// Expiration date
    pub expires: Option<chrono::DateTime<chrono::Utc>>,
    /// Identity type
    pub identity_type: IdentityType,
}

/// Type of signing identity
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityType {
    /// Apple Developer ID Application (for distribution)
    AppleDeveloperIdApplication,
    /// Apple Developer ID Installer (for PKG)
    AppleDeveloperIdInstaller,
    /// Apple Development (for testing)
    AppleDevelopment,
    /// Windows Authenticode (EV or OV)
    WindowsAuthenticode,
    /// Self-signed (testing only)
    SelfSigned,
    /// Unknown
    Unknown,
}

impl std::fmt::Display for IdentityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::AppleDeveloperIdApplication => write!(f, "Developer ID Application"),
            Self::AppleDeveloperIdInstaller => write!(f, "Developer ID Installer"),
            Self::AppleDevelopment => write!(f, "Apple Development"),
            Self::WindowsAuthenticode => write!(f, "Windows Authenticode"),
            Self::SelfSigned => write!(f, "Self-Signed"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// List available signing identities for the current platform
pub fn list_signing_identities() -> ReleaseResult<Vec<SigningIdentity>> {
    #[cfg(target_os = "macos")]
    {
        return macos::list_macos_identities();
    }

    #[cfg(target_os = "windows")]
    {
        return windows::list_windows_certificates();
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        Ok(vec![])
    }
}

/// Find the best signing identity matching the given name
pub fn find_signing_identity(name: &str) -> ReleaseResult<Option<SigningIdentity>> {
    let identities = list_signing_identities()?;

    // First try exact match
    if let Some(identity) = identities.iter().find(|i| i.name == name) {
        return Ok(Some(identity.clone()));
    }

    // Then try partial match
    if let Some(identity) = identities.iter().find(|i| i.name.contains(name)) {
        return Ok(Some(identity.clone()));
    }

    // Try fingerprint match
    if let Some(identity) = identities.iter().find(|i| i.fingerprint.starts_with(name)) {
        return Ok(Some(identity.clone()));
    }

    Ok(None)
}

/// Signing result with details
#[derive(Debug, Clone)]
pub struct SigningResult {
    /// Whether signing succeeded
    pub success: bool,
    /// Signature details
    pub signature: Option<ArtifactSignature>,
    /// Signing identity used
    pub identity: String,
    /// Timestamp server used
    pub timestamp_server: Option<String>,
    /// Any warnings
    pub warnings: Vec<String>,
}

impl SigningResult {
    /// Create a successful signing result
    pub fn success(signature: ArtifactSignature, identity: impl Into<String>) -> Self {
        Self {
            success: true,
            signature: Some(signature),
            identity: identity.into(),
            timestamp_server: None,
            warnings: vec![],
        }
    }

    /// Add timestamp server info
    pub fn with_timestamp(mut self, server: impl Into<String>) -> Self {
        self.timestamp_server = Some(server.into());
        self
    }

    /// Add a warning
    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}
