//! GitHub release automation
//!
//! Creates releases, uploads artifacts, and manages release metadata.

use crate::artifact::Artifact;
use crate::config::GitHubConfig;
use crate::error::{ReleaseError, ReleaseResult};
use crate::{Release, ReleaseChannel};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

/// Create a GitHub release
pub async fn create_release(
    release: &Release,
    config: &GitHubConfig,
    app_name: String,
) -> ReleaseResult<String> {
    let tag_name = format!("v{}", release.version);
    let release_name = config
        .name_template
        .as_ref()
        .map(|t| t.replace("{version}", &release.version).replace("{app}", &app_name))
        .unwrap_or_else(|| format!("{} v{}", app_name, release.version));

    let is_prerelease = release.channel != ReleaseChannel::Stable;

    // Prefer gh CLI if available
    if which::which("gh").is_ok() {
        return create_release_with_gh(release, config, &tag_name, &release_name, is_prerelease).await;
    }

    // Fall back to API
    create_release_with_api(release, config, &tag_name, &release_name, is_prerelease).await
}

/// Create release using gh CLI
async fn create_release_with_gh(
    release: &Release,
    config: &GitHubConfig,
    tag_name: &str,
    release_name: &str,
    is_prerelease: bool,
) -> ReleaseResult<String> {
    tracing::info!("Creating GitHub release using gh CLI...");

    let mut cmd = Command::new("gh");
    cmd.arg("release").arg("create").arg(tag_name);

    // Title
    cmd.arg("--title").arg(release_name);

    // Notes
    if let Some(ref notes) = release.changelog {
        cmd.arg("--notes").arg(notes);
    } else if config.generate_release_notes {
        cmd.arg("--generate-notes");
    } else {
        cmd.arg("--notes").arg("");
    }

    // Prerelease
    if is_prerelease && config.prerelease_for_channels {
        cmd.arg("--prerelease");
    }

    // Draft
    if config.draft {
        cmd.arg("--draft");
    }

    // Repository
    cmd.arg("--repo").arg(format!("{}/{}", config.owner, config.repo));

    // Add artifacts
    for artifact in &release.artifacts {
        if artifact.path.exists() {
            cmd.arg(&artifact.path);
        }
    }

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::github(format!(
            "gh release create failed: {}",
            stderr.trim()
        )));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let url = stdout.trim().to_string();

    tracing::info!("Release created: {}", url);
    Ok(url)
}

/// Create release using GitHub API
async fn create_release_with_api(
    release: &Release,
    config: &GitHubConfig,
    tag_name: &str,
    release_name: &str,
    is_prerelease: bool,
) -> ReleaseResult<String> {
    let token = config.token.as_ref().ok_or_else(|| {
        ReleaseError::github("GITHUB_TOKEN not set. Set it or install gh CLI.")
    })?;

    tracing::info!("Creating GitHub release using API...");

    // Use curl for API call to avoid adding reqwest dependency
    let api_url = format!(
        "https://api.github.com/repos/{}/{}/releases",
        config.owner, config.repo
    );

    let body = serde_json::json!({
        "tag_name": tag_name,
        "name": release_name,
        "body": release.changelog.as_deref().unwrap_or(""),
        "draft": config.draft,
        "prerelease": is_prerelease && config.prerelease_for_channels,
        "generate_release_notes": config.generate_release_notes && release.changelog.is_none(),
    });

    let output = Command::new("curl")
        .args(["-s", "-X", "POST"])
        .arg("-H").arg(format!("Authorization: Bearer {}", token))
        .arg("-H").arg("Accept: application/vnd.github+json")
        .arg("-H").arg("X-GitHub-Api-Version: 2022-11-28")
        .arg("-d").arg(body.to_string())
        .arg(&api_url)
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::github("API request failed"));
    }

    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    if let Some(url) = response.get("html_url").and_then(|v| v.as_str()) {
        // Upload artifacts
        if let Some(upload_url) = response.get("upload_url").and_then(|v| v.as_str()) {
            let upload_url = upload_url.replace("{?name,label}", "");
            for artifact in &release.artifacts {
                if artifact.path.exists() {
                    upload_artifact_api(&artifact, &upload_url, token).await?;
                }
            }
        }

        tracing::info!("Release created: {}", url);
        Ok(url.to_string())
    } else if let Some(message) = response.get("message").and_then(|v| v.as_str()) {
        Err(ReleaseError::github(message.to_string()))
    } else {
        Err(ReleaseError::github("Unknown API error"))
    }
}

/// Upload an artifact using the API
async fn upload_artifact_api(
    artifact: &Artifact,
    upload_url: &str,
    token: &str,
) -> ReleaseResult<()> {
    let file_name = artifact.path.file_name().unwrap().to_string_lossy();
    let url = format!("{}?name={}", upload_url, file_name);

    tracing::info!("Uploading {}...", file_name);

    let output = Command::new("curl")
        .args(["-s", "-X", "POST"])
        .arg("-H").arg(format!("Authorization: Bearer {}", token))
        .arg("-H").arg("Accept: application/vnd.github+json")
        .arg("-H").arg(format!("Content-Type: {}", artifact.mime_type()))
        .arg("--data-binary").arg(format!("@{}", artifact.path.display()))
        .arg(&url)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("Failed to upload {}: {}", file_name, stderr.trim());
    }

    Ok(())
}

/// Upload additional artifacts to an existing release
pub async fn upload_artifacts(
    release_url: &str,
    artifacts: &[Artifact],
    config: &GitHubConfig,
) -> ReleaseResult<()> {
    // Use gh CLI if available
    if which::which("gh").is_ok() {
        for artifact in artifacts {
            if artifact.path.exists() {
                let output = Command::new("gh")
                    .args(["release", "upload"])
                    .arg(release_url.split('/').last().unwrap_or(""))
                    .arg(&artifact.path)
                    .arg("--repo").arg(format!("{}/{}", config.owner, config.repo))
                    .arg("--clobber")
                    .output()?;

                if !output.status.success() {
                    let stderr = String::from_utf8_lossy(&output.stderr);
                    tracing::warn!("Failed to upload {}: {}", artifact.name, stderr.trim());
                }
            }
        }
    }

    Ok(())
}

/// Delete a release
pub async fn delete_release(tag: &str, config: &GitHubConfig) -> ReleaseResult<()> {
    if which::which("gh").is_ok() {
        let output = Command::new("gh")
            .args(["release", "delete", tag])
            .arg("--repo").arg(format!("{}/{}", config.owner, config.repo))
            .arg("--yes")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReleaseError::github(format!(
                "Failed to delete release: {}",
                stderr.trim()
            )));
        }
    }

    Ok(())
}

/// List releases
pub async fn list_releases(config: &GitHubConfig, limit: usize) -> ReleaseResult<Vec<GitHubRelease>> {
    if which::which("gh").is_ok() {
        let output = Command::new("gh")
            .args(["release", "list"])
            .arg("--repo").arg(format!("{}/{}", config.owner, config.repo))
            .arg("--limit").arg(limit.to_string())
            .arg("--json").arg("tagName,name,publishedAt,isDraft,isPrerelease,url")
            .output()?;

        if output.status.success() {
            let releases: Vec<GitHubRelease> = serde_json::from_slice(&output.stdout)?;
            return Ok(releases);
        }
    }

    Ok(vec![])
}

/// GitHub release information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubRelease {
    /// Tag name
    #[serde(rename = "tagName")]
    pub tag_name: String,
    /// Release name
    pub name: String,
    /// Published timestamp
    #[serde(rename = "publishedAt")]
    pub published_at: Option<String>,
    /// Is draft
    #[serde(rename = "isDraft")]
    pub is_draft: bool,
    /// Is prerelease
    #[serde(rename = "isPrerelease")]
    pub is_prerelease: bool,
    /// Release URL
    pub url: String,
}

/// Generate checksums file and upload to release
pub async fn upload_checksums(
    release_tag: &str,
    artifacts: &[Artifact],
    config: &GitHubConfig,
) -> ReleaseResult<()> {
    // Generate checksums content
    let mut content = String::new();
    for artifact in artifacts {
        if let Some(ref checksum) = artifact.checksum {
            let filename = artifact.path.file_name().unwrap().to_string_lossy();
            content.push_str(&format!("{}  {}\n", checksum, filename));
        }
    }

    if content.is_empty() {
        return Ok(());
    }

    // Write to temp file
    let temp_dir = tempfile::tempdir()?;
    let checksums_path = temp_dir.path().join("SHA256SUMS");
    std::fs::write(&checksums_path, &content)?;

    // Upload
    if which::which("gh").is_ok() {
        let output = Command::new("gh")
            .args(["release", "upload", release_tag])
            .arg(&checksums_path)
            .arg("--repo").arg(format!("{}/{}", config.owner, config.repo))
            .arg("--clobber")
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!("Failed to upload checksums: {}", stderr.trim());
        }
    }

    Ok(())
}

/// Edit an existing release
pub async fn edit_release(
    tag: &str,
    notes: Option<&str>,
    draft: Option<bool>,
    prerelease: Option<bool>,
    config: &GitHubConfig,
) -> ReleaseResult<()> {
    if which::which("gh").is_ok() {
        let mut cmd = Command::new("gh");
        cmd.args(["release", "edit", tag]);

        if let Some(notes) = notes {
            cmd.arg("--notes").arg(notes);
        }

        if let Some(draft) = draft {
            cmd.arg(if draft { "--draft" } else { "--draft=false" });
        }

        if let Some(prerelease) = prerelease {
            cmd.arg(if prerelease { "--prerelease" } else { "--prerelease=false" });
        }

        cmd.arg("--repo").arg(format!("{}/{}", config.owner, config.repo));

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReleaseError::github(format!(
                "Failed to edit release: {}",
                stderr.trim()
            )));
        }
    }

    Ok(())
}

/// Download release artifacts
pub async fn download_release_artifacts(
    tag: &str,
    output_dir: &Path,
    config: &GitHubConfig,
) -> ReleaseResult<Vec<std::path::PathBuf>> {
    std::fs::create_dir_all(output_dir)?;

    if which::which("gh").is_ok() {
        let output = Command::new("gh")
            .args(["release", "download", tag])
            .arg("--dir").arg(output_dir)
            .arg("--repo").arg(format!("{}/{}", config.owner, config.repo))
            .output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReleaseError::github(format!(
                "Failed to download artifacts: {}",
                stderr.trim()
            )));
        }
    }

    // List downloaded files
    let files: Vec<std::path::PathBuf> = std::fs::read_dir(output_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect();

    Ok(files)
}

/// Create release from a git tag (trigger CI workflow style)
pub fn create_release_dispatch(
    config: &GitHubConfig,
    workflow: &str,
    ref_name: &str,
    inputs: &serde_json::Value,
) -> ReleaseResult<()> {
    if which::which("gh").is_ok() {
        let mut cmd = Command::new("gh");
        cmd.args(["workflow", "run", workflow]);
        cmd.arg("--repo").arg(format!("{}/{}", config.owner, config.repo));
        cmd.arg("--ref").arg(ref_name);

        // Add inputs
        if let Some(obj) = inputs.as_object() {
            for (key, value) in obj {
                cmd.arg("-f").arg(format!(
                    "{}={}",
                    key,
                    value.as_str().unwrap_or(&value.to_string())
                ));
            }
        }

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ReleaseError::github(format!(
                "Failed to dispatch workflow: {}",
                stderr.trim()
            )));
        }
    }

    Ok(())
}
