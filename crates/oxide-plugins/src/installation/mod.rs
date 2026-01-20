//! Plugin installation from various sources.
//!
//! Supports three installation sources:
//!
//! - **Registry**: `oxide add ui.tables`
//! - **Git**: `oxide add git github.com/acme/plugin@v1.0.0`
//! - **Local**: `oxide add path ../my-plugin`

use std::path::{Path, PathBuf};
use std::fs;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::error::{PluginError, PluginResult};
use crate::manifest::{self, PluginManifest};
use crate::namespace::PluginId;
use crate::loader::PluginLoader;

/// Source from which to install a plugin.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstallSource {
    /// Install from the OxideKit registry.
    Registry {
        /// Version requirement (e.g., "^1.0.0", ">=0.5.0").
        version: Option<String>,
    },
    /// Install from a Git repository.
    Git {
        /// Repository URL.
        url: String,
        /// Git reference (tag or commit SHA, NOT branch).
        #[serde(rename = "ref")]
        git_ref: String,
    },
    /// Install from a local path.
    Path {
        /// Path to the plugin directory.
        path: PathBuf,
    },
}

impl InstallSource {
    /// Create a registry source.
    pub fn registry(version: Option<String>) -> Self {
        InstallSource::Registry { version }
    }

    /// Create a Git source.
    pub fn git(url: &str, git_ref: &str) -> PluginResult<Self> {
        // Validate ref is not a branch name (basic check)
        if git_ref == "main" || git_ref == "master" || git_ref == "develop" {
            return Err(PluginError::GitError(
                "Floating branch references are not allowed. Use a tag or commit SHA.".to_string()
            ));
        }

        Ok(InstallSource::Git {
            url: url.to_string(),
            git_ref: git_ref.to_string(),
        })
    }

    /// Create a path source.
    pub fn path<P: AsRef<Path>>(path: P) -> Self {
        InstallSource::Path {
            path: path.as_ref().to_path_buf(),
        }
    }

    /// Get the source type as a string.
    pub fn source_type(&self) -> &'static str {
        match self {
            InstallSource::Registry { .. } => "registry",
            InstallSource::Git { .. } => "git",
            InstallSource::Path { .. } => "path",
        }
    }
}

impl std::fmt::Display for InstallSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InstallSource::Registry { version } => {
                write!(f, "registry")?;
                if let Some(v) = version {
                    write!(f, "@{}", v)?;
                }
                Ok(())
            }
            InstallSource::Git { url, git_ref } => {
                write!(f, "git:{}@{}", url, git_ref)
            }
            InstallSource::Path { path } => {
                write!(f, "path:{}", path.display())
            }
        }
    }
}

/// Plugin installer.
#[derive(Debug)]
pub struct PluginInstaller {
    /// Project root directory.
    project_root: PathBuf,
    /// Installation directory.
    install_dir: PathBuf,
}

impl PluginInstaller {
    /// Create a new installer.
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        let root = project_root.as_ref().to_path_buf();
        let install_dir = root.join(".oxide/plugins");

        Self {
            project_root: root,
            install_dir,
        }
    }

    /// Install a plugin from the given source.
    pub fn install(
        &self,
        plugin_id: &PluginId,
        source: &InstallSource,
    ) -> PluginResult<(PluginManifest, PathBuf)> {
        // Ensure install directory exists
        fs::create_dir_all(&self.install_dir)?;

        let target_dir = self.install_dir.join(plugin_id.to_dir_path());

        // Check if already exists
        if target_dir.exists() {
            return Err(PluginError::AlreadyInstalled(plugin_id.clone()));
        }

        match source {
            InstallSource::Registry { version } => {
                self.install_from_registry(plugin_id, version.as_deref(), &target_dir)
            }
            InstallSource::Git { url, git_ref } => {
                self.install_from_git(plugin_id, url, git_ref, &target_dir)
            }
            InstallSource::Path { path } => {
                self.install_from_path(plugin_id, path, &target_dir)
            }
        }
    }

    /// Install from the OxideKit registry.
    fn install_from_registry(
        &self,
        plugin_id: &PluginId,
        _version: Option<&str>,
        target_dir: &Path,
    ) -> PluginResult<(PluginManifest, PathBuf)> {
        // In a real implementation, this would:
        // 1. Query the registry API for the plugin
        // 2. Download the package
        // 3. Verify signatures
        // 4. Extract to target directory

        // For now, return an error since we don't have a real registry
        Err(PluginError::RegistryError(format!(
            "Registry installation not yet implemented for '{}'",
            plugin_id
        )))
    }

    /// Install from a Git repository.
    fn install_from_git(
        &self,
        plugin_id: &PluginId,
        url: &str,
        git_ref: &str,
        target_dir: &Path,
    ) -> PluginResult<(PluginManifest, PathBuf)> {
        info!("Installing {} from git: {}@{}", plugin_id, url, git_ref);

        // Validate the ref is not a floating branch
        if git_ref == "main" || git_ref == "master" || git_ref == "develop" {
            return Err(PluginError::GitError(
                "Floating branch references are not allowed. Use a tag or commit SHA.".to_string()
            ));
        }

        // In a real implementation, this would use git2 to:
        // 1. Clone the repository (shallow if possible)
        // 2. Checkout the specific ref
        // 3. Verify the commit matches the ref
        // 4. Copy to target directory

        // For now, return an error since we need git2 optional feature
        Err(PluginError::GitError(format!(
            "Git installation not yet implemented. Would install {}@{} to {:?}",
            url, git_ref, target_dir
        )))
    }

    /// Install from a local path.
    fn install_from_path(
        &self,
        plugin_id: &PluginId,
        source_path: &Path,
        target_dir: &Path,
    ) -> PluginResult<(PluginManifest, PathBuf)> {
        // Resolve the source path
        let source_path = if source_path.is_relative() {
            self.project_root.join(source_path)
        } else {
            source_path.to_path_buf()
        };

        // Check source exists
        if !source_path.exists() {
            return Err(PluginError::PluginNotFound(plugin_id.clone()));
        }

        // Check for plugin.toml
        let manifest_path = source_path.join("plugin.toml");
        if !manifest_path.exists() {
            return Err(PluginError::ManifestNotFound(manifest_path));
        }

        // Load and validate manifest
        let manifest = manifest::load_manifest(&manifest_path)?;

        // Verify plugin ID matches
        if manifest.plugin.id.full_name() != plugin_id.full_name() {
            return Err(PluginError::KindMismatch {
                expected: plugin_id.full_name().to_string(),
                actual: manifest.plugin.id.full_name().to_string(),
            });
        }

        // Create target directory
        fs::create_dir_all(target_dir)?;

        // Copy files
        self.copy_directory(&source_path, target_dir)?;

        info!("Installed {} from path: {:?}", plugin_id, source_path);

        Ok((manifest, target_dir.to_path_buf()))
    }

    /// Uninstall a plugin.
    pub fn uninstall(&self, install_path: &Path) -> PluginResult<()> {
        if !install_path.exists() {
            return Ok(()); // Already uninstalled
        }

        // Remove the directory
        fs::remove_dir_all(install_path)?;

        // Clean up empty parent directories
        let mut parent = install_path.parent();
        while let Some(p) = parent {
            if p.starts_with(&self.install_dir) && p != self.install_dir {
                if let Ok(entries) = fs::read_dir(p) {
                    if entries.count() == 0 {
                        fs::remove_dir(p).ok();
                    } else {
                        break;
                    }
                }
            }
            parent = p.parent();
        }

        Ok(())
    }

    /// Copy a directory recursively.
    fn copy_directory(&self, source: &Path, target: &Path) -> PluginResult<()> {
        for entry in walkdir::WalkDir::new(source)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let source_path = entry.path();
            let relative = source_path.strip_prefix(source)
                .map_err(|e| PluginError::IoError(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string()
                )))?;
            let target_path = target.join(relative);

            if entry.file_type().is_dir() {
                fs::create_dir_all(&target_path)?;
            } else {
                if let Some(parent) = target_path.parent() {
                    fs::create_dir_all(parent)?;
                }
                fs::copy(source_path, &target_path)?;
            }
        }

        Ok(())
    }

    /// Get the installation directory.
    pub fn install_directory(&self) -> &Path {
        &self.install_dir
    }
}

/// Parse an install specifier string.
///
/// Formats:
/// - `ui.tables` -> Registry
/// - `ui.tables@1.0.0` -> Registry with version
/// - `git github.com/acme/plugin@v1.0.0` -> Git
/// - `path ../my-plugin` -> Local path
pub fn parse_install_specifier(spec: &str) -> PluginResult<(PluginId, InstallSource)> {
    if spec.starts_with("git ") {
        // Git source
        let rest = &spec[4..];
        let (url, git_ref) = rest.rsplit_once('@')
            .ok_or_else(|| PluginError::InvalidPluginId(
                "Git source requires @ref (e.g., git github.com/acme/plugin@v1.0.0)".to_string()
            ))?;

        // Try to infer plugin ID from URL
        let plugin_id = infer_plugin_id_from_url(url)?;
        let source = InstallSource::git(url, git_ref)?;

        Ok((plugin_id, source))
    } else if spec.starts_with("path ") {
        // Path source
        let path = &spec[5..];
        let path = PathBuf::from(path);

        // Load manifest to get plugin ID
        let manifest_path = path.join("plugin.toml");
        if manifest_path.exists() {
            let manifest = manifest::load_manifest(&manifest_path)?;
            Ok((manifest.plugin.id.clone(), InstallSource::path(&path)))
        } else {
            Err(PluginError::ManifestNotFound(manifest_path))
        }
    } else {
        // Registry source (possibly with version)
        let (id_str, version) = if let Some((id, ver)) = spec.rsplit_once('@') {
            (id, Some(ver.to_string()))
        } else {
            (spec, None)
        };

        let plugin_id = PluginId::parse(id_str)?;
        let source = InstallSource::registry(version);

        Ok((plugin_id, source))
    }
}

/// Try to infer a plugin ID from a Git URL.
fn infer_plugin_id_from_url(url: &str) -> PluginResult<PluginId> {
    // Extract the repo name from the URL
    let name = url
        .trim_end_matches(".git")
        .rsplit('/')
        .next()
        .ok_or_else(|| PluginError::InvalidPluginId(
            format!("Cannot infer plugin ID from URL: {}", url)
        ))?;

    // Try to parse as plugin ID (e.g., "oxidekit-ui-tables" -> "ui.tables")
    if let Some(stripped) = name.strip_prefix("oxidekit-") {
        let parts: Vec<&str> = stripped.splitn(2, '-').collect();
        if parts.len() == 2 {
            return PluginId::parse(&format!("{}.{}", parts[0], parts[1]));
        }
    }

    // Fallback: use the name directly with a generic namespace
    Err(PluginError::InvalidPluginId(format!(
        "Cannot infer plugin ID from '{}'. Please specify the plugin ID explicitly.",
        name
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_install_source_display() {
        let registry = InstallSource::registry(Some("1.0.0".to_string()));
        assert_eq!(registry.to_string(), "registry@1.0.0");

        let git = InstallSource::git("github.com/acme/plugin", "v1.0.0").unwrap();
        assert_eq!(git.to_string(), "git:github.com/acme/plugin@v1.0.0");

        let path = InstallSource::path("/my/plugin");
        assert_eq!(path.to_string(), "path:/my/plugin");
    }

    #[test]
    fn test_git_source_rejects_branches() {
        assert!(InstallSource::git("github.com/test", "main").is_err());
        assert!(InstallSource::git("github.com/test", "master").is_err());
        assert!(InstallSource::git("github.com/test", "develop").is_err());

        // Tags and SHAs should work
        assert!(InstallSource::git("github.com/test", "v1.0.0").is_ok());
        assert!(InstallSource::git("github.com/test", "abc123def").is_ok());
    }

    #[test]
    fn test_parse_install_specifier() {
        // Registry
        let (id, source) = parse_install_specifier("ui.tables").unwrap();
        assert_eq!(id.full_name(), "ui.tables");
        assert!(matches!(source, InstallSource::Registry { version: None }));

        // Registry with version
        let (id, source) = parse_install_specifier("ui.tables@1.0.0").unwrap();
        assert_eq!(id.full_name(), "ui.tables");
        assert!(matches!(source, InstallSource::Registry { version: Some(_) }));
    }
}
