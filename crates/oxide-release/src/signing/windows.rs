//! Windows code signing using signtool
//!
//! Provides Authenticode signing for Windows executables and installers.
//! Supports:
//! - Certificate files (.pfx)
//! - Certificate store
//! - Timestamping
//! - SHA-256 signing

use crate::artifact::{Artifact, ArtifactSignature, SignatureAlgorithm};
use crate::config::SigningConfig;
use crate::error::{ReleaseError, ReleaseResult};
use crate::signing::{IdentityType, SigningIdentity};
use std::path::Path;
use std::process::Command;

/// Default timestamp servers
const DEFAULT_TIMESTAMP_SERVERS: &[&str] = &[
    "http://timestamp.digicert.com",
    "http://timestamp.sectigo.com",
    "http://timestamp.comodoca.com",
    "http://tsa.starfieldtech.com",
];

/// Sign a Windows artifact
pub async fn sign_windows(
    artifact: &Artifact,
    config: &SigningConfig,
) -> ReleaseResult<ArtifactSignature> {
    // Find signtool
    let signtool = find_signtool()?;

    tracing::info!("Signing {} using signtool", artifact.name);

    let mut cmd = Command::new(&signtool);
    cmd.arg("sign");

    // Use SHA-256
    cmd.arg("/fd").arg("SHA256");

    // Certificate source
    if let Some(ref cert_path) = config.certificate_path {
        if !cert_path.exists() {
            return Err(ReleaseError::signing(format!(
                "Certificate file not found: {}",
                cert_path.display()
            )));
        }

        cmd.arg("/f").arg(cert_path);

        // Password
        if let Some(ref password) = config.certificate_password {
            cmd.arg("/p").arg(password);
        }
    } else if let Some(ref identity) = config.identity {
        // Use certificate from store by subject name
        cmd.arg("/n").arg(identity);
    } else {
        return Err(ReleaseError::signing(
            "No certificate or identity configured for Windows signing",
        ));
    }

    // Timestamp
    let timestamp_url = config
        .timestamp_url
        .as_deref()
        .unwrap_or(DEFAULT_TIMESTAMP_SERVERS[0]);
    cmd.arg("/tr").arg(timestamp_url);
    cmd.arg("/td").arg("SHA256");

    // Description (optional, shows in UAC prompt)
    cmd.arg("/d").arg(&artifact.name);

    // The file to sign
    cmd.arg(&artifact.path);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);

        // Try alternate timestamp servers on failure
        if stderr.contains("timestamp") || stdout.contains("timestamp") {
            tracing::warn!("Timestamp failed, trying alternate servers...");
            for &server in &DEFAULT_TIMESTAMP_SERVERS[1..] {
                if try_sign_with_timestamp(&signtool, artifact, config, server).await? {
                    return Ok(ArtifactSignature {
                        algorithm: SignatureAlgorithm::Authenticode,
                        signer: config
                            .identity
                            .clone()
                            .unwrap_or_else(|| "Certificate".to_string()),
                        data: String::new(),
                        timestamp: Some(chrono::Utc::now()),
                        certificate_chain: None,
                    });
                }
            }
        }

        return Err(ReleaseError::signing(format!(
            "signtool failed: {} {}",
            stdout.trim(),
            stderr.trim()
        )));
    }

    Ok(ArtifactSignature {
        algorithm: SignatureAlgorithm::Authenticode,
        signer: config
            .identity
            .clone()
            .unwrap_or_else(|| "Certificate".to_string()),
        data: String::new(),
        timestamp: Some(chrono::Utc::now()),
        certificate_chain: None,
    })
}

/// Try signing with a specific timestamp server
async fn try_sign_with_timestamp(
    signtool: &Path,
    artifact: &Artifact,
    config: &SigningConfig,
    timestamp_url: &str,
) -> ReleaseResult<bool> {
    let mut cmd = Command::new(signtool);
    cmd.arg("sign").arg("/fd").arg("SHA256");

    if let Some(ref cert_path) = config.certificate_path {
        cmd.arg("/f").arg(cert_path);
        if let Some(ref password) = config.certificate_password {
            cmd.arg("/p").arg(password);
        }
    } else if let Some(ref identity) = config.identity {
        cmd.arg("/n").arg(identity);
    }

    cmd.arg("/tr").arg(timestamp_url);
    cmd.arg("/td").arg("SHA256");
    cmd.arg(&artifact.path);

    let output = cmd.output()?;
    Ok(output.status.success())
}

/// Verify a Windows code signature
pub async fn verify_windows_signature(artifact: &Artifact) -> ReleaseResult<bool> {
    let signtool = find_signtool()?;

    let output = Command::new(&signtool)
        .arg("verify")
        .arg("/pa") // Use default Authenticode policy
        .arg(&artifact.path)
        .output()?;

    if output.status.success() {
        tracing::info!("Signature verified: {}", artifact.path.display());
        Ok(true)
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        tracing::warn!("Signature verification failed: {}", stdout.trim());
        Ok(false)
    }
}

/// Find signtool.exe
fn find_signtool() -> ReleaseResult<std::path::PathBuf> {
    // First check PATH
    if let Ok(path) = which::which("signtool") {
        return Ok(path);
    }

    // Search Windows SDK locations
    let sdk_paths = [
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files (x86)\Windows Kits\8.1\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];

    for sdk_path in sdk_paths {
        let path = std::path::Path::new(sdk_path);
        if path.exists() {
            // Find the latest version
            if let Ok(entries) = std::fs::read_dir(path) {
                let mut versions: Vec<_> = entries
                    .filter_map(|e| e.ok())
                    .filter(|e| e.path().is_dir())
                    .collect();

                // Sort by version (descending)
                versions.sort_by(|a, b| b.file_name().cmp(&a.file_name()));

                for version in versions {
                    // Try x64 first
                    let signtool = version.path().join("x64").join("signtool.exe");
                    if signtool.exists() {
                        return Ok(signtool);
                    }

                    // Then x86
                    let signtool = version.path().join("x86").join("signtool.exe");
                    if signtool.exists() {
                        return Ok(signtool);
                    }
                }
            }
        }
    }

    Err(ReleaseError::ToolNotFound("signtool.exe".to_string()))
}

/// List Windows certificates available for signing
pub fn list_windows_certificates() -> ReleaseResult<Vec<SigningIdentity>> {
    let mut identities = Vec::new();

    // Use certutil to list certificates
    let output = Command::new("certutil")
        .args(["-store", "My"])
        .output()?;

    if !output.status.success() {
        // Try without admin privileges using PowerShell
        let ps_output = Command::new("powershell")
            .args([
                "-Command",
                "Get-ChildItem -Path Cert:\\CurrentUser\\My | Format-List Subject, Thumbprint, NotAfter",
            ])
            .output()?;

        if ps_output.status.success() {
            let stdout = String::from_utf8_lossy(&ps_output.stdout);
            identities.extend(parse_powershell_certs(&stdout));
        }
    } else {
        let stdout = String::from_utf8_lossy(&output.stdout);
        identities.extend(parse_certutil_output(&stdout));
    }

    Ok(identities)
}

/// Parse certutil output
fn parse_certutil_output(output: &str) -> Vec<SigningIdentity> {
    let mut identities = Vec::new();
    let mut current_name = None;
    let mut current_fingerprint = None;

    for line in output.lines() {
        let line = line.trim();

        if line.starts_with("Subject:") {
            if let Some(cn_start) = line.find("CN=") {
                let cn = &line[cn_start + 3..];
                let cn_end = cn.find(',').unwrap_or(cn.len());
                current_name = Some(cn[..cn_end].to_string());
            }
        } else if line.starts_with("Cert Hash(sha1):") {
            let hash = line.replace("Cert Hash(sha1):", "").replace(' ', "");
            current_fingerprint = Some(hash.trim().to_string());
        }

        // When we have both, create identity
        if let (Some(name), Some(fingerprint)) = (&current_name, &current_fingerprint) {
            identities.push(SigningIdentity {
                name: name.clone(),
                fingerprint: fingerprint.clone(),
                is_valid: true,
                expires: None,
                identity_type: IdentityType::WindowsAuthenticode,
            });
            current_name = None;
            current_fingerprint = None;
        }
    }

    identities
}

/// Parse PowerShell certificate output
fn parse_powershell_certs(output: &str) -> Vec<SigningIdentity> {
    let mut identities = Vec::new();
    let mut current_name = None;
    let mut current_fingerprint = None;

    for line in output.lines() {
        let line = line.trim();

        if let Some(subject) = line.strip_prefix("Subject    :") {
            let subject = subject.trim();
            if let Some(cn_start) = subject.find("CN=") {
                let cn = &subject[cn_start + 3..];
                let cn_end = cn.find(',').unwrap_or(cn.len());
                current_name = Some(cn[..cn_end].to_string());
            }
        } else if let Some(thumbprint) = line.strip_prefix("Thumbprint :") {
            current_fingerprint = Some(thumbprint.trim().to_string());
        }

        if let (Some(name), Some(fingerprint)) = (&current_name, &current_fingerprint) {
            identities.push(SigningIdentity {
                name: name.clone(),
                fingerprint: fingerprint.clone(),
                is_valid: true,
                expires: None,
                identity_type: IdentityType::WindowsAuthenticode,
            });
            current_name = None;
            current_fingerprint = None;
        }
    }

    identities
}

/// Get signing information for a Windows executable
pub fn get_signature_info(path: &Path) -> ReleaseResult<Option<WindowsSignatureInfo>> {
    let signtool = find_signtool()?;

    let output = Command::new(&signtool)
        .arg("verify")
        .arg("/v") // Verbose
        .arg("/pa")
        .arg(path)
        .output()?;

    if !output.status.success() {
        return Ok(None);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut info = WindowsSignatureInfo::default();

    for line in stdout.lines() {
        let line = line.trim();
        if line.starts_with("Issued to:") {
            info.issued_to = Some(line.replace("Issued to:", "").trim().to_string());
        } else if line.starts_with("Issued by:") {
            info.issued_by = Some(line.replace("Issued by:", "").trim().to_string());
        } else if line.starts_with("Expires:") {
            info.expires = Some(line.replace("Expires:", "").trim().to_string());
        } else if line.contains("Timestamp:") {
            info.timestamp = Some(line.replace("Timestamp:", "").trim().to_string());
        }
    }

    Ok(Some(info))
}

/// Windows signature information
#[derive(Debug, Clone, Default)]
pub struct WindowsSignatureInfo {
    /// Certificate issued to
    pub issued_to: Option<String>,
    /// Certificate issued by
    pub issued_by: Option<String>,
    /// Certificate expiration
    pub expires: Option<String>,
    /// Signing timestamp
    pub timestamp: Option<String>,
}

/// Add a dual signature (SHA-1 + SHA-256) for older Windows compatibility
pub async fn dual_sign_windows(
    artifact: &Artifact,
    config: &SigningConfig,
) -> ReleaseResult<()> {
    let signtool = find_signtool()?;
    let timestamp_url = config
        .timestamp_url
        .as_deref()
        .unwrap_or(DEFAULT_TIMESTAMP_SERVERS[0]);

    // First signature: SHA-1 (for Windows 7/Vista)
    let mut cmd1 = Command::new(&signtool);
    cmd1.arg("sign").arg("/fd").arg("SHA1");

    if let Some(ref cert_path) = config.certificate_path {
        cmd1.arg("/f").arg(cert_path);
        if let Some(ref password) = config.certificate_password {
            cmd1.arg("/p").arg(password);
        }
    } else if let Some(ref identity) = config.identity {
        cmd1.arg("/n").arg(identity);
    }

    // Use older timestamp format for SHA-1
    cmd1.arg("/t").arg(timestamp_url);
    cmd1.arg(&artifact.path);

    let output1 = cmd1.output()?;
    if !output1.status.success() {
        tracing::warn!("SHA-1 signing failed, continuing with SHA-256 only");
    }

    // Second signature: SHA-256 (appended)
    let mut cmd2 = Command::new(&signtool);
    cmd2.arg("sign").arg("/fd").arg("SHA256").arg("/as"); // /as = append signature

    if let Some(ref cert_path) = config.certificate_path {
        cmd2.arg("/f").arg(cert_path);
        if let Some(ref password) = config.certificate_password {
            cmd2.arg("/p").arg(password);
        }
    } else if let Some(ref identity) = config.identity {
        cmd2.arg("/n").arg(identity);
    }

    cmd2.arg("/tr").arg(timestamp_url);
    cmd2.arg("/td").arg("SHA256");
    cmd2.arg(&artifact.path);

    let output2 = cmd2.output()?;
    if !output2.status.success() {
        let stderr = String::from_utf8_lossy(&output2.stderr);
        return Err(ReleaseError::signing(format!(
            "SHA-256 signing failed: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_certutil_output() {
        let output = r#"
================ Certificate 0 ================
Serial Number: 1234567890abcdef
Issuer: CN=DigiCert, O=DigiCert Inc
Subject: CN=Example Inc, O=Example Inc
Cert Hash(sha1): AB CD EF 12 34 56 78 90 AB CD EF 12 34 56 78 90 AB CD EF 12
"#;

        let identities = parse_certutil_output(output);
        assert_eq!(identities.len(), 1);
        assert_eq!(identities[0].name, "Example Inc");
    }
}
