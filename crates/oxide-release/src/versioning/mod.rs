//! Version management and automation
//!
//! Handles semantic versioning, version bumping, and version file updates.

use crate::error::{ReleaseError, ReleaseResult};
use semver::Version;
use std::path::Path;

/// Version bump type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BumpType {
    /// Increment major version (x.0.0)
    Major,
    /// Increment minor version (x.y.0)
    Minor,
    /// Increment patch version (x.y.z)
    Patch,
    /// Set to specific version
    Set,
    /// Pre-release version (x.y.z-alpha.1)
    PreRelease,
    /// Remove pre-release (x.y.z-alpha.1 -> x.y.z)
    Release,
}

impl std::str::FromStr for BumpType {
    type Err = ReleaseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "major" => Ok(Self::Major),
            "minor" => Ok(Self::Minor),
            "patch" => Ok(Self::Patch),
            "prerelease" | "pre-release" | "pre" => Ok(Self::PreRelease),
            "release" => Ok(Self::Release),
            _ => Err(ReleaseError::InvalidVersion(format!(
                "Unknown bump type: {}. Use major, minor, patch, prerelease, or release",
                s
            ))),
        }
    }
}

/// Bump a version according to the specified type
pub fn bump_version(current: &Version, bump_type: BumpType) -> ReleaseResult<Version> {
    let mut new_version = current.clone();

    match bump_type {
        BumpType::Major => {
            new_version.major += 1;
            new_version.minor = 0;
            new_version.patch = 0;
            new_version.pre = semver::Prerelease::EMPTY;
        }
        BumpType::Minor => {
            new_version.minor += 1;
            new_version.patch = 0;
            new_version.pre = semver::Prerelease::EMPTY;
        }
        BumpType::Patch => {
            new_version.patch += 1;
            new_version.pre = semver::Prerelease::EMPTY;
        }
        BumpType::PreRelease => {
            // If already prerelease, increment it
            if !new_version.pre.is_empty() {
                new_version.pre = increment_prerelease(&new_version.pre)?;
            } else {
                // Start new prerelease
                new_version.patch += 1;
                new_version.pre = semver::Prerelease::new("alpha.1")?;
            }
        }
        BumpType::Release => {
            // Remove prerelease
            new_version.pre = semver::Prerelease::EMPTY;
        }
        BumpType::Set => {
            // This is handled separately
        }
    }

    new_version.build = semver::BuildMetadata::EMPTY;

    Ok(new_version)
}

/// Increment a prerelease version
fn increment_prerelease(pre: &semver::Prerelease) -> ReleaseResult<semver::Prerelease> {
    let pre_str = pre.as_str();

    // Try to parse as name.number format (e.g., alpha.1, beta.2)
    if let Some(dot_pos) = pre_str.rfind('.') {
        let name = &pre_str[..dot_pos];
        let num_str = &pre_str[dot_pos + 1..];

        if let Ok(num) = num_str.parse::<u64>() {
            let new_pre = format!("{}.{}", name, num + 1);
            return Ok(semver::Prerelease::new(&new_pre)?);
        }
    }

    // Try to parse as just a number
    if let Ok(num) = pre_str.parse::<u64>() {
        return Ok(semver::Prerelease::new(&(num + 1).to_string())?);
    }

    // Append .1
    let new_pre = format!("{}.1", pre_str);
    Ok(semver::Prerelease::new(&new_pre)?)
}

/// Set version with prerelease identifier
pub fn set_prerelease_version(
    base: &Version,
    prerelease: &str,
) -> ReleaseResult<Version> {
    let mut version = base.clone();
    version.pre = semver::Prerelease::new(prerelease)?;
    version.build = semver::BuildMetadata::EMPTY;
    Ok(version)
}

/// Set version with build metadata
pub fn set_build_metadata(
    version: &Version,
    build: &str,
) -> ReleaseResult<Version> {
    let mut version = version.clone();
    version.build = semver::BuildMetadata::new(build)?;
    Ok(version)
}

/// Update version in oxide.toml
pub fn update_oxide_toml_version(project_root: &Path, new_version: &Version) -> ReleaseResult<()> {
    let oxide_toml_path = project_root.join("oxide.toml");

    if !oxide_toml_path.exists() {
        return Err(ReleaseError::config("oxide.toml not found"));
    }

    let content = std::fs::read_to_string(&oxide_toml_path)?;
    let updated = update_toml_version(&content, &new_version.to_string())?;
    std::fs::write(&oxide_toml_path, updated)?;

    Ok(())
}

/// Update version in Cargo.toml
pub fn update_cargo_toml_version(project_root: &Path, new_version: &Version) -> ReleaseResult<()> {
    let cargo_toml_path = project_root.join("Cargo.toml");

    if !cargo_toml_path.exists() {
        return Err(ReleaseError::config("Cargo.toml not found"));
    }

    let content = std::fs::read_to_string(&cargo_toml_path)?;
    let updated = update_toml_version(&content, &new_version.to_string())?;
    std::fs::write(&cargo_toml_path, updated)?;

    Ok(())
}

/// Update version string in TOML content
fn update_toml_version(content: &str, new_version: &str) -> ReleaseResult<String> {
    let version_re = regex::Regex::new(r#"^(\s*version\s*=\s*)"([^"]+)"(.*)$"#).unwrap();

    let updated: Vec<String> = content
        .lines()
        .map(|line| {
            if version_re.is_match(line) {
                version_re
                    .replace(line, format!(r#"$1"{}"$3"#, new_version))
                    .to_string()
            } else {
                line.to_string()
            }
        })
        .collect();

    Ok(updated.join("\n"))
}

/// Update version in all relevant files
pub fn update_all_versions(project_root: &Path, new_version: &Version) -> ReleaseResult<Vec<String>> {
    let mut updated_files = Vec::new();

    // Update oxide.toml
    let oxide_toml = project_root.join("oxide.toml");
    if oxide_toml.exists() {
        update_oxide_toml_version(project_root, new_version)?;
        updated_files.push("oxide.toml".to_string());
    }

    // Update Cargo.toml
    let cargo_toml = project_root.join("Cargo.toml");
    if cargo_toml.exists() {
        update_cargo_toml_version(project_root, new_version)?;
        updated_files.push("Cargo.toml".to_string());
    }

    // Update package.json if it exists
    let package_json = project_root.join("package.json");
    if package_json.exists() {
        update_package_json_version(&package_json, new_version)?;
        updated_files.push("package.json".to_string());
    }

    Ok(updated_files)
}

/// Update version in package.json
fn update_package_json_version(path: &Path, new_version: &Version) -> ReleaseResult<()> {
    let content = std::fs::read_to_string(path)?;
    let mut json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(obj) = json.as_object_mut() {
        obj.insert(
            "version".to_string(),
            serde_json::Value::String(new_version.to_string()),
        );
    }

    let updated = serde_json::to_string_pretty(&json)?;
    std::fs::write(path, updated)?;

    Ok(())
}

/// Read current version from oxide.toml
pub fn read_current_version(project_root: &Path) -> ReleaseResult<Version> {
    let oxide_toml_path = project_root.join("oxide.toml");

    if !oxide_toml_path.exists() {
        return Err(ReleaseError::config("oxide.toml not found"));
    }

    let content = std::fs::read_to_string(&oxide_toml_path)?;
    let toml: toml::Value = toml::from_str(&content)?;

    let version_str = toml
        .get("app")
        .and_then(|a| a.get("version"))
        .and_then(|v| v.as_str())
        .ok_or_else(|| ReleaseError::config("version not found in oxide.toml"))?;

    Ok(Version::parse(version_str)?)
}

/// Create a git tag for the version
pub fn create_version_tag(
    repo_path: &Path,
    version: &Version,
    message: Option<&str>,
) -> ReleaseResult<()> {
    let repo = git2::Repository::open(repo_path)?;
    let tag_name = format!("v{}", version);

    let head = repo.head()?.peel_to_commit()?;
    let sig = repo.signature()?;

    let msg = message.unwrap_or(&format!("Release {}", tag_name));

    repo.tag(&tag_name, head.as_object(), &sig, msg, false)?;

    tracing::info!("Created tag: {}", tag_name);
    Ok(())
}

/// Push tag to remote
pub fn push_tag(repo_path: &Path, tag_name: &str, remote: &str) -> ReleaseResult<()> {
    use std::process::Command;

    let output = Command::new("git")
        .current_dir(repo_path)
        .args(["push", remote, tag_name])
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::Git(format!(
            "Failed to push tag: {}",
            stderr.trim()
        )));
    }

    Ok(())
}

/// Version history entry
#[derive(Debug, Clone)]
pub struct VersionHistoryEntry {
    /// Version
    pub version: Version,
    /// Tag name
    pub tag: String,
    /// Release date
    pub date: chrono::DateTime<chrono::Utc>,
    /// Commit hash
    pub commit: String,
}

/// Get version history from git tags
pub fn get_version_history(repo_path: &Path) -> ReleaseResult<Vec<VersionHistoryEntry>> {
    let repo = git2::Repository::open(repo_path)?;
    let mut history = Vec::new();

    let tags = repo.tag_names(None)?;
    for tag_name in tags.iter().flatten() {
        // Parse version from tag
        let version_str = tag_name.trim_start_matches('v');
        if let Ok(version) = Version::parse(version_str) {
            // Get tag info
            if let Ok(tag_ref) = repo.find_reference(&format!("refs/tags/{}", tag_name)) {
                if let Ok(commit) = tag_ref.peel_to_commit() {
                    let timestamp = commit.time().seconds();
                    let date = chrono::DateTime::from_timestamp(timestamp, 0)
                        .unwrap_or_else(chrono::Utc::now);

                    history.push(VersionHistoryEntry {
                        version,
                        tag: tag_name.to_string(),
                        date,
                        commit: commit.id().to_string()[..7].to_string(),
                    });
                }
            }
        }
    }

    // Sort by version descending
    history.sort_by(|a, b| b.version.cmp(&a.version));

    Ok(history)
}

/// Check if version already exists as a tag
pub fn version_tag_exists(repo_path: &Path, version: &Version) -> ReleaseResult<bool> {
    let repo = git2::Repository::open(repo_path)?;
    let tag_name = format!("v{}", version);

    Ok(repo.find_reference(&format!("refs/tags/{}", tag_name)).is_ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bump_major() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = bump_version(&v, BumpType::Major).unwrap();
        assert_eq!(bumped.to_string(), "2.0.0");
    }

    #[test]
    fn test_bump_minor() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = bump_version(&v, BumpType::Minor).unwrap();
        assert_eq!(bumped.to_string(), "1.3.0");
    }

    #[test]
    fn test_bump_patch() {
        let v = Version::parse("1.2.3").unwrap();
        let bumped = bump_version(&v, BumpType::Patch).unwrap();
        assert_eq!(bumped.to_string(), "1.2.4");
    }

    #[test]
    fn test_bump_prerelease() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        let bumped = bump_version(&v, BumpType::PreRelease).unwrap();
        assert_eq!(bumped.to_string(), "1.2.3-alpha.2");
    }

    #[test]
    fn test_bump_release() {
        let v = Version::parse("1.2.3-alpha.1").unwrap();
        let bumped = bump_version(&v, BumpType::Release).unwrap();
        assert_eq!(bumped.to_string(), "1.2.3");
    }

    #[test]
    fn test_update_toml_version() {
        let content = r#"
[app]
name = "test"
version = "1.0.0"
"#;
        let updated = update_toml_version(content, "2.0.0").unwrap();
        assert!(updated.contains(r#"version = "2.0.0""#));
    }
}
