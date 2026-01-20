//! macOS notarization using Apple's notarytool
//!
//! This module handles Apple notarization for macOS apps. Notarization is required
//! for apps distributed outside the Mac App Store on macOS 10.15+.
//!
//! Supports two authentication methods:
//! - Apple ID + App-specific password
//! - App Store Connect API Key (recommended for CI)

use crate::artifact::Artifact;
use crate::config::NotarizationConfig;
use crate::error::{ReleaseError, ReleaseResult};
use std::path::Path;
use std::process::Command;
use std::time::{Duration, Instant};

/// Notarize an artifact
pub async fn notarize_artifact(
    artifact: &Artifact,
    config: &NotarizationConfig,
) -> ReleaseResult<NotarizationResult> {
    if !artifact.path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            artifact.path.display().to_string(),
        ));
    }

    tracing::info!("Notarizing {}...", artifact.name);

    // Submit for notarization
    let submission_id = submit_for_notarization(&artifact.path, config).await?;
    tracing::info!("Submitted for notarization: {}", submission_id);

    // Wait for completion
    let result = wait_for_notarization(&submission_id, config).await?;

    // Staple if requested
    if config.staple && result.status == NotarizationStatus::Accepted {
        staple_artifact(&artifact.path).await?;
    }

    Ok(result)
}

/// Submit an artifact for notarization
async fn submit_for_notarization(
    path: &Path,
    config: &NotarizationConfig,
) -> ReleaseResult<String> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("notarytool");
    cmd.arg("submit");
    cmd.arg(path);

    // Add authentication
    add_auth_args(&mut cmd, config)?;

    // Output format
    cmd.arg("--output-format").arg("json");

    // Wait for completion
    cmd.arg("--wait");

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Submission failed: {}",
            stderr.trim()
        )));
    }

    // Parse JSON output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)?;

    let id = json["id"]
        .as_str()
        .ok_or_else(|| ReleaseError::notarization("No submission ID in response"))?;

    let status = json["status"]
        .as_str()
        .ok_or_else(|| ReleaseError::notarization("No status in response"))?;

    match status {
        "Accepted" => Ok(id.to_string()),
        "Invalid" | "Rejected" => {
            let message = json["message"].as_str().unwrap_or("Unknown reason");
            Err(ReleaseError::NotarizationRejected(message.to_string()))
        }
        _ => Ok(id.to_string()),
    }
}

/// Wait for notarization to complete (used if not using --wait)
async fn wait_for_notarization(
    submission_id: &str,
    config: &NotarizationConfig,
) -> ReleaseResult<NotarizationResult> {
    let timeout = Duration::from_secs(config.timeout);
    let start = Instant::now();
    let poll_interval = Duration::from_secs(30);

    loop {
        if start.elapsed() > timeout {
            return Err(ReleaseError::NotarizationTimeout(config.timeout));
        }

        let status = check_notarization_status(submission_id, config).await?;

        match status.status {
            NotarizationStatus::Accepted => {
                tracing::info!("Notarization accepted!");
                return Ok(status);
            }
            NotarizationStatus::Invalid | NotarizationStatus::Rejected => {
                return Err(ReleaseError::NotarizationRejected(
                    status.message.unwrap_or_else(|| "Unknown reason".to_string()),
                ));
            }
            NotarizationStatus::InProgress => {
                tracing::info!(
                    "Notarization in progress... ({}s elapsed)",
                    start.elapsed().as_secs()
                );
                tokio::time::sleep(poll_interval).await;
            }
        }
    }
}

/// Check notarization status
async fn check_notarization_status(
    submission_id: &str,
    config: &NotarizationConfig,
) -> ReleaseResult<NotarizationResult> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("notarytool");
    cmd.arg("info");
    cmd.arg(submission_id);

    add_auth_args(&mut cmd, config)?;

    cmd.arg("--output-format").arg("json");

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Status check failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)?;

    let status = match json["status"].as_str() {
        Some("Accepted") => NotarizationStatus::Accepted,
        Some("Invalid") => NotarizationStatus::Invalid,
        Some("Rejected") => NotarizationStatus::Rejected,
        _ => NotarizationStatus::InProgress,
    };

    Ok(NotarizationResult {
        submission_id: submission_id.to_string(),
        status,
        message: json["message"].as_str().map(String::from),
        log_url: None,
    })
}

/// Add authentication arguments to notarytool command
fn add_auth_args(cmd: &mut Command, config: &NotarizationConfig) -> ReleaseResult<()> {
    // Prefer API key authentication
    if let (Some(key_id), Some(issuer_id)) = (&config.api_key_id, &config.api_issuer_id) {
        cmd.arg("--key-id").arg(key_id);
        cmd.arg("--issuer").arg(issuer_id);

        if let Some(ref key_path) = config.api_key_path {
            cmd.arg("--key").arg(key_path);
        }
    } else if let (Some(apple_id), Some(password)) = (&config.apple_id, &config.password) {
        // Fall back to Apple ID auth
        cmd.arg("--apple-id").arg(apple_id);
        cmd.arg("--password").arg(password);
        cmd.arg("--team-id").arg(&config.team_id);
    } else {
        return Err(ReleaseError::notarization(
            "No authentication configured. Set API key or Apple ID credentials.",
        ));
    }

    Ok(())
}

/// Staple the notarization ticket to the artifact
pub async fn staple_artifact(path: &Path) -> ReleaseResult<()> {
    tracing::info!("Stapling notarization ticket to {}", path.display());

    let output = Command::new("xcrun")
        .args(["stapler", "staple"])
        .arg(path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Stapling failed: {}",
            stderr.trim()
        )));
    }

    // Verify stapling
    let verify_output = Command::new("xcrun")
        .args(["stapler", "validate"])
        .arg(path)
        .output()?;

    if !verify_output.status.success() {
        tracing::warn!("Stapling verification failed, but ticket was applied");
    }

    Ok(())
}

/// Check if an artifact is already notarized
pub async fn is_notarized(path: &Path) -> ReleaseResult<bool> {
    let output = Command::new("spctl")
        .args(["-a", "-v", "--type", "install"])
        .arg(path)
        .output()?;

    // spctl outputs to stderr
    let stderr = String::from_utf8_lossy(&output.stderr);
    Ok(stderr.contains("accepted") || stderr.contains("notarized"))
}

/// Get the notarization log for a submission
pub async fn get_notarization_log(
    submission_id: &str,
    config: &NotarizationConfig,
) -> ReleaseResult<String> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("notarytool");
    cmd.arg("log");
    cmd.arg(submission_id);

    add_auth_args(&mut cmd, config)?;

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Failed to get log: {}",
            stderr.trim()
        )));
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

/// List recent notarization submissions
pub async fn list_submissions(
    config: &NotarizationConfig,
    limit: usize,
) -> ReleaseResult<Vec<NotarizationSubmission>> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("notarytool");
    cmd.arg("history");

    add_auth_args(&mut cmd, config)?;

    cmd.arg("--output-format").arg("json");

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Failed to list history: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)?;

    let mut submissions = Vec::new();
    if let Some(history) = json["notarizationHistory"].as_array() {
        for entry in history.iter().take(limit) {
            submissions.push(NotarizationSubmission {
                id: entry["id"].as_str().unwrap_or("").to_string(),
                name: entry["name"].as_str().unwrap_or("").to_string(),
                status: match entry["status"].as_str() {
                    Some("Accepted") => NotarizationStatus::Accepted,
                    Some("Invalid") => NotarizationStatus::Invalid,
                    Some("Rejected") => NotarizationStatus::Rejected,
                    _ => NotarizationStatus::InProgress,
                },
                created_date: entry["createdDate"].as_str().map(String::from),
            });
        }
    }

    Ok(submissions)
}

/// Notarization status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotarizationStatus {
    /// Notarization in progress
    InProgress,
    /// Notarization accepted
    Accepted,
    /// Notarization rejected
    Rejected,
    /// Package was invalid
    Invalid,
}

impl std::fmt::Display for NotarizationStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InProgress => write!(f, "In Progress"),
            Self::Accepted => write!(f, "Accepted"),
            Self::Rejected => write!(f, "Rejected"),
            Self::Invalid => write!(f, "Invalid"),
        }
    }
}

/// Notarization result
#[derive(Debug, Clone)]
pub struct NotarizationResult {
    /// Submission ID
    pub submission_id: String,
    /// Status
    pub status: NotarizationStatus,
    /// Status message
    pub message: Option<String>,
    /// URL to notarization log
    pub log_url: Option<String>,
}

/// Notarization submission history entry
#[derive(Debug, Clone)]
pub struct NotarizationSubmission {
    /// Submission ID
    pub id: String,
    /// Package name
    pub name: String,
    /// Status
    pub status: NotarizationStatus,
    /// Created date
    pub created_date: Option<String>,
}

/// Store API credentials for notarization
pub async fn store_credentials(
    profile_name: &str,
    config: &NotarizationConfig,
) -> ReleaseResult<()> {
    let mut cmd = Command::new("xcrun");
    cmd.arg("notarytool");
    cmd.arg("store-credentials");
    cmd.arg(profile_name);

    if let (Some(key_id), Some(issuer_id)) = (&config.api_key_id, &config.api_issuer_id) {
        cmd.arg("--key-id").arg(key_id);
        cmd.arg("--issuer").arg(issuer_id);
        if let Some(ref key_path) = config.api_key_path {
            cmd.arg("--key").arg(key_path);
        }
    } else if let (Some(apple_id), Some(password)) = (&config.apple_id, &config.password) {
        cmd.arg("--apple-id").arg(apple_id);
        cmd.arg("--password").arg(password);
        cmd.arg("--team-id").arg(&config.team_id);
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::notarization(format!(
            "Failed to store credentials: {}",
            stderr.trim()
        )));
    }

    tracing::info!("Credentials stored as profile: {}", profile_name);
    Ok(())
}

/// Create a notarization-ready ZIP for submission
pub async fn create_notarization_zip(
    source: &Path,
    output: &Path,
) -> ReleaseResult<()> {
    let output_cmd = Command::new("ditto")
        .args(["-c", "-k", "--keepParent"])
        .arg(source)
        .arg(output)
        .output()?;

    if !output_cmd.status.success() {
        let stderr = String::from_utf8_lossy(&output_cmd.stderr);
        return Err(ReleaseError::notarization(format!(
            "Failed to create ZIP: {}",
            stderr.trim()
        )));
    }

    Ok(())
}
