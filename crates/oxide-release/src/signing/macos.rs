//! macOS code signing using codesign
//!
//! Provides code signing for macOS apps and binaries using Apple's codesign tool.
//! Supports:
//! - Developer ID Application certificates (for distribution)
//! - Developer ID Installer certificates (for PKG)
//! - Hardened Runtime
//! - Entitlements
//! - Deep signing (frameworks, helpers)

use crate::artifact::{Artifact, ArtifactSignature, SignatureAlgorithm};
use crate::config::SigningConfig;
use crate::error::{ReleaseError, ReleaseResult};
use crate::signing::{IdentityType, SigningIdentity};
use std::path::Path;
use std::process::Command;

/// Sign a macOS artifact
pub async fn sign_macos(
    artifact: &Artifact,
    config: &SigningConfig,
) -> ReleaseResult<ArtifactSignature> {
    let identity = config
        .identity
        .as_ref()
        .ok_or_else(|| ReleaseError::signing("No signing identity configured"))?;

    // Verify identity exists
    if !identity_exists(identity)? {
        return Err(ReleaseError::SigningIdentityNotFound(identity.clone()));
    }

    tracing::info!("Signing {} with identity: {}", artifact.name, identity);

    let mut cmd = Command::new("codesign");

    // Force re-sign
    cmd.arg("--force");

    // Sign with identity
    cmd.arg("--sign").arg(identity);

    // Timestamp (important for distribution)
    cmd.arg("--timestamp");

    // Deep sign (frameworks, plugins, helpers)
    cmd.arg("--deep");

    // Hardened runtime (required for notarization)
    if config.hardened_runtime {
        cmd.arg("--options").arg("runtime");
    }

    // Entitlements
    if let Some(ref entitlements) = config.entitlements {
        if entitlements.exists() {
            cmd.arg("--entitlements").arg(entitlements);
        } else {
            tracing::warn!("Entitlements file not found: {}", entitlements.display());
        }
    }

    // Additional flags
    for flag in &config.flags {
        cmd.arg(flag);
    }

    // The file to sign
    cmd.arg(&artifact.path);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::signing(format!(
            "codesign failed: {}",
            stderr.trim()
        )));
    }

    // Verify the signature
    verify_codesign(&artifact.path)?;

    Ok(ArtifactSignature {
        algorithm: SignatureAlgorithm::AppleCodesign,
        signer: identity.clone(),
        data: String::new(), // Apple signatures are embedded
        timestamp: Some(chrono::Utc::now()),
        certificate_chain: None,
    })
}

/// Verify a macOS code signature
pub async fn verify_macos_signature(artifact: &Artifact) -> ReleaseResult<bool> {
    verify_codesign(&artifact.path)
}

/// Internal: verify using codesign
fn verify_codesign(path: &Path) -> ReleaseResult<bool> {
    let output = Command::new("codesign")
        .args(["--verify", "--deep", "--strict"])
        .arg(path)
        .output()?;

    if output.status.success() {
        tracing::info!("Signature verified: {}", path.display());
        Ok(true)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("Signature verification failed: {}", stderr.trim());
        Ok(false)
    }
}

/// Check if a signing identity exists
fn identity_exists(identity: &str) -> ReleaseResult<bool> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.contains(identity))
}

/// List available macOS signing identities
pub fn list_macos_identities() -> ReleaseResult<Vec<SigningIdentity>> {
    let output = Command::new("security")
        .args(["find-identity", "-v", "-p", "codesigning"])
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::signing("Failed to list signing identities"));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut identities = Vec::new();

    for line in stdout.lines() {
        // Parse lines like:
        // 1) FINGERPRINT "Developer ID Application: Company Name (TEAMID)"
        if let Some(parsed) = parse_identity_line(line) {
            identities.push(parsed);
        }
    }

    Ok(identities)
}

/// Parse a security find-identity output line
fn parse_identity_line(line: &str) -> Option<SigningIdentity> {
    // Skip lines that don't contain identity info
    if !line.contains(')') || !line.contains('"') {
        return None;
    }

    // Extract fingerprint (40 hex chars)
    let fingerprint_start = line.find(|c: char| c.is_ascii_hexdigit())?;
    let fingerprint_part = &line[fingerprint_start..];
    let fingerprint_end = fingerprint_part
        .find(|c: char| !c.is_ascii_hexdigit())
        .unwrap_or(40);
    let fingerprint = &fingerprint_part[..fingerprint_end.min(40)];

    if fingerprint.len() != 40 {
        return None;
    }

    // Extract name between quotes
    let name_start = line.find('"')? + 1;
    let name_end = line.rfind('"')?;
    if name_end <= name_start {
        return None;
    }
    let name = &line[name_start..name_end];

    // Determine identity type
    let identity_type = if name.contains("Developer ID Application") {
        IdentityType::AppleDeveloperIdApplication
    } else if name.contains("Developer ID Installer") {
        IdentityType::AppleDeveloperIdInstaller
    } else if name.contains("Apple Development") || name.contains("iPhone Developer") {
        IdentityType::AppleDevelopment
    } else {
        IdentityType::Unknown
    };

    Some(SigningIdentity {
        name: name.to_string(),
        fingerprint: fingerprint.to_string(),
        is_valid: !line.contains("CSSMERR_TP_CERT_EXPIRED"),
        expires: None, // Would need to query certificate for this
        identity_type,
    })
}

/// Sign a macOS app bundle
pub async fn sign_app_bundle(
    bundle_path: &Path,
    config: &SigningConfig,
) -> ReleaseResult<()> {
    let identity = config
        .identity
        .as_ref()
        .ok_or_else(|| ReleaseError::signing("No signing identity configured"))?;

    // Sign frameworks first (deep signing)
    let frameworks_path = bundle_path.join("Contents/Frameworks");
    if frameworks_path.exists() {
        for entry in std::fs::read_dir(&frameworks_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.extension().map(|e| e == "framework").unwrap_or(false)
                || path.extension().map(|e| e == "dylib").unwrap_or(false)
            {
                sign_component(&path, identity, config).await?;
            }
        }
    }

    // Sign helpers
    let helpers_path = bundle_path.join("Contents/Helpers");
    if helpers_path.exists() {
        for entry in std::fs::read_dir(&helpers_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() || path.extension().map(|e| e == "app").unwrap_or(false) {
                sign_component(&path, identity, config).await?;
            }
        }
    }

    // Sign main executable
    let main_executable = bundle_path.join("Contents/MacOS");
    if main_executable.exists() {
        for entry in std::fs::read_dir(&main_executable)? {
            let entry = entry?;
            sign_component(&entry.path(), identity, config).await?;
        }
    }

    // Sign the bundle itself
    let artifact = Artifact::new(
        bundle_path.file_name().unwrap().to_string_lossy(),
        crate::TargetPlatform::MacOSArm64,
        bundle_path,
    );
    sign_macos(&artifact, config).await?;

    Ok(())
}

/// Sign a single component
async fn sign_component(
    path: &Path,
    identity: &str,
    config: &SigningConfig,
) -> ReleaseResult<()> {
    tracing::debug!("Signing component: {}", path.display());

    let mut cmd = Command::new("codesign");
    cmd.arg("--force")
        .arg("--sign")
        .arg(identity)
        .arg("--timestamp");

    if config.hardened_runtime {
        cmd.arg("--options").arg("runtime");
    }

    cmd.arg(path);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::signing(format!(
            "Failed to sign {}: {}",
            path.display(),
            stderr.trim()
        )));
    }

    Ok(())
}

/// Get code signing requirements for an artifact
pub fn get_signing_requirements(path: &Path) -> ReleaseResult<String> {
    let output = Command::new("codesign")
        .args(["-d", "-r", "-"])
        .arg(path)
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(ReleaseError::signing(format!(
            "Failed to get requirements: {}",
            stderr
        )))
    }
}

/// Display signing information for an artifact
pub fn display_signing_info(path: &Path) -> ReleaseResult<SigningInfo> {
    let output = Command::new("codesign")
        .args(["-dvvv"])
        .arg(path)
        .output()?;

    // codesign outputs to stderr for -d
    let info = String::from_utf8_lossy(&output.stderr);

    let mut signing_info = SigningInfo::default();

    for line in info.lines() {
        if let Some(value) = line.strip_prefix("Authority=") {
            signing_info.authorities.push(value.to_string());
        } else if let Some(value) = line.strip_prefix("TeamIdentifier=") {
            signing_info.team_id = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Timestamp=") {
            signing_info.timestamp = Some(value.to_string());
        } else if let Some(value) = line.strip_prefix("Identifier=") {
            signing_info.identifier = Some(value.to_string());
        } else if line.contains("flags=") {
            if line.contains("runtime") {
                signing_info.hardened_runtime = true;
            }
        }
    }

    Ok(signing_info)
}

/// Signing information for display
#[derive(Debug, Clone, Default)]
pub struct SigningInfo {
    /// Certificate authority chain
    pub authorities: Vec<String>,
    /// Team identifier
    pub team_id: Option<String>,
    /// Bundle/code identifier
    pub identifier: Option<String>,
    /// Signing timestamp
    pub timestamp: Option<String>,
    /// Hardened runtime enabled
    pub hardened_runtime: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_identity_line() {
        let line = r#"  1) ABC123DEF456789012345678901234567890ABCD "Developer ID Application: Example Inc (TEAMID)""#;
        let identity = parse_identity_line(line).unwrap();

        assert_eq!(identity.name, "Developer ID Application: Example Inc (TEAMID)");
        assert_eq!(identity.fingerprint, "ABC123DEF456789012345678901234567890ABCD");
        assert_eq!(identity.identity_type, IdentityType::AppleDeveloperIdApplication);
    }

    #[test]
    fn test_parse_invalid_line() {
        assert!(parse_identity_line("invalid line").is_none());
        assert!(parse_identity_line("").is_none());
        assert!(parse_identity_line("0 valid identities found").is_none());
    }
}
