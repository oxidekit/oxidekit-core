//! # OxideKit Plugin System
//!
//! A complete plugin ecosystem for OxideKit that enables modular, safe-by-design
//! extensibility without the dependency chaos of npm or the security risks of
//! unrestricted native code execution.
//!
//! ## Core Concepts
//!
//! ### Plugin Categories
//!
//! - **UI**: Components, tables, forms, charts - no permissions by default
//! - **Native**: OS capabilities (filesystem, keychain, notifications)
//! - **Service**: App-level building blocks (auth, db, query cache)
//! - **Tooling**: Dev/build-time tools (generators, linters)
//! - **Theme**: Token packs, typography, icon sets
//! - **Design**: Admin shells, templates, layout kits
//!
//! ### Trust Levels
//!
//! - **Official**: Maintained by OxideKit org
//! - **Verified**: Identity verified + signed releases + capability review
//! - **Community**: Sandbox by default, clear warnings
//!
//! ### Installation Sources
//!
//! ```bash
//! oxide add ui.tables              # Registry
//! oxide add git github.com/acme/oxidekit-auth@v0.2.1  # GitHub
//! oxide add path ../my-plugin      # Local
//! ```
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use oxide_plugins::{PluginManager, PluginManifest, PluginCategory};
//!
//! // Load plugins from the project
//! let manager = PluginManager::new("./my-project")?;
//!
//! // Discover installed plugins
//! let plugins = manager.discover_plugins()?;
//!
//! // Install a plugin from registry
//! manager.install("ui.tables", InstallSource::Registry)?;
//!
//! // Verify a plugin's security
//! let report = manager.verify_plugin("ui.tables")?;
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod manifest;
pub mod discovery;
pub mod loader;
pub mod installation;
pub mod scaffold;
pub mod permissions;
pub mod sandbox;
pub mod trust;
pub mod registry;
pub mod mobile;
mod error;
mod lockfile;
mod namespace;

// Re-exports for convenient access
pub use manifest::{PluginManifest, PluginCategory, PluginKindConfig};
pub use discovery::PluginDiscovery;
pub use loader::PluginLoader;
pub use installation::{InstallSource, PluginInstaller};
pub use scaffold::{PluginScaffold, ScaffoldOptions};
pub use permissions::{Capability, Permission, PermissionSet};
pub use sandbox::SandboxConfig;
pub use trust::{TrustLevel, TrustPolicy};
pub use registry::RegistryClient;
pub use error::{PluginError, PluginResult};
pub use lockfile::{Lockfile, LockEntry};
pub use namespace::{PluginId, Namespace};

use std::path::{Path, PathBuf};
use std::collections::HashMap;

/// The main plugin manager that coordinates all plugin operations.
///
/// This is the primary entry point for working with the plugin system.
#[derive(Debug)]
pub struct PluginManager {
    /// Root directory of the project
    project_root: PathBuf,
    /// Plugin discovery subsystem
    discovery: PluginDiscovery,
    /// Plugin loader
    loader: PluginLoader,
    /// Plugin installer
    installer: PluginInstaller,
    /// Registry client
    registry: RegistryClient,
    /// Installed plugins cache
    plugins: HashMap<PluginId, LoadedPlugin>,
    /// Project lockfile
    lockfile: Lockfile,
}

/// Represents a loaded plugin with its manifest and runtime state.
#[derive(Debug, Clone)]
pub struct LoadedPlugin {
    /// The plugin's unique identifier
    pub id: PluginId,
    /// The plugin's manifest
    pub manifest: PluginManifest,
    /// Path to the plugin's installation directory
    pub install_path: PathBuf,
    /// Current trust level
    pub trust_level: TrustLevel,
    /// Whether the plugin is currently enabled
    pub enabled: bool,
}

impl PluginManager {
    /// Create a new plugin manager for the given project root.
    ///
    /// # Arguments
    ///
    /// * `project_root` - Path to the OxideKit project root
    ///
    /// # Returns
    ///
    /// A new `PluginManager` instance or an error if initialization fails.
    pub fn new<P: AsRef<Path>>(project_root: P) -> PluginResult<Self> {
        let project_root = project_root.as_ref().to_path_buf();

        // Ensure project root exists
        if !project_root.exists() {
            return Err(PluginError::ProjectNotFound(project_root));
        }

        // Load or create lockfile
        let lockfile_path = project_root.join("extensions.lock");
        let lockfile = if lockfile_path.exists() {
            Lockfile::load(&lockfile_path)?
        } else {
            Lockfile::new()
        };

        Ok(Self {
            project_root: project_root.clone(),
            discovery: PluginDiscovery::new(&project_root),
            loader: PluginLoader::new(&project_root),
            installer: PluginInstaller::new(&project_root),
            registry: RegistryClient::default(),
            plugins: HashMap::new(),
            lockfile,
        })
    }

    /// Discover all plugins in the project.
    ///
    /// Scans the project's plugin directories and returns information about
    /// all installed plugins.
    pub fn discover_plugins(&mut self) -> PluginResult<Vec<LoadedPlugin>> {
        let manifests = self.discovery.scan()?;
        let mut plugins = Vec::new();

        for (path, manifest) in manifests {
            let id = manifest.id().clone();
            let trust_level = self.determine_trust_level(&manifest);

            let plugin = LoadedPlugin {
                id: id.clone(),
                manifest,
                install_path: path,
                trust_level,
                enabled: true,
            };

            self.plugins.insert(id, plugin.clone());
            plugins.push(plugin);
        }

        Ok(plugins)
    }

    /// Install a plugin from the specified source.
    ///
    /// # Arguments
    ///
    /// * `plugin_id` - The plugin identifier (e.g., "ui.tables")
    /// * `source` - Where to install from (Registry, Git, or Local path)
    ///
    /// # Returns
    ///
    /// The installed plugin's manifest on success.
    pub fn install(&mut self, plugin_id: &str, source: InstallSource) -> PluginResult<PluginManifest> {
        let id = PluginId::parse(plugin_id)?;

        // Check if already installed
        if self.plugins.contains_key(&id) {
            return Err(PluginError::AlreadyInstalled(id));
        }

        // Install the plugin
        let (manifest, install_path) = self.installer.install(&id, &source)?;

        // Update lockfile
        let lock_entry = LockEntry::from_install(&id, &source, &manifest)?;
        self.lockfile.add_entry(lock_entry);
        self.lockfile.save(&self.project_root.join("extensions.lock"))?;

        // Add to loaded plugins
        let trust_level = self.determine_trust_level(&manifest);
        let plugin = LoadedPlugin {
            id: id.clone(),
            manifest: manifest.clone(),
            install_path,
            trust_level,
            enabled: true,
        };
        self.plugins.insert(id, plugin);

        Ok(manifest)
    }

    /// Uninstall a plugin.
    pub fn uninstall(&mut self, plugin_id: &str) -> PluginResult<()> {
        let id = PluginId::parse(plugin_id)?;

        // Check if installed
        let plugin = self.plugins.remove(&id)
            .ok_or_else(|| PluginError::NotInstalled(id.clone()))?;

        // Remove from filesystem
        self.installer.uninstall(&plugin.install_path)?;

        // Update lockfile
        self.lockfile.remove_entry(&id);
        self.lockfile.save(&self.project_root.join("extensions.lock"))?;

        Ok(())
    }

    /// Verify a plugin's security and compatibility.
    pub fn verify_plugin(&self, plugin_id: &str) -> PluginResult<VerificationReport> {
        let id = PluginId::parse(plugin_id)?;

        let plugin = self.plugins.get(&id)
            .ok_or_else(|| PluginError::NotInstalled(id.clone()))?;

        self.loader.verify(&plugin.manifest, &plugin.install_path)
    }

    /// Get a plugin by ID.
    pub fn get_plugin(&self, plugin_id: &str) -> PluginResult<&LoadedPlugin> {
        let id = PluginId::parse(plugin_id)?;
        self.plugins.get(&id)
            .ok_or_else(|| PluginError::NotInstalled(id))
    }

    /// List all installed plugins.
    pub fn list_plugins(&self) -> Vec<&LoadedPlugin> {
        self.plugins.values().collect()
    }

    /// Get the project root path.
    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    /// Determine the trust level for a plugin based on its manifest and publisher.
    fn determine_trust_level(&self, manifest: &PluginManifest) -> TrustLevel {
        // Check publisher against known official/verified publishers
        if manifest.plugin.publisher.starts_with("oxidekit") {
            TrustLevel::Official
        } else if self.is_verified_publisher(&manifest.plugin.publisher) {
            TrustLevel::Verified
        } else {
            TrustLevel::Community
        }
    }

    /// Check if a publisher is in the verified list.
    fn is_verified_publisher(&self, _publisher: &str) -> bool {
        // In a real implementation, this would check against a registry
        // or local cache of verified publishers
        false
    }
}

/// Report from plugin verification.
#[derive(Debug, Clone)]
pub struct VerificationReport {
    /// Plugin ID
    pub plugin_id: PluginId,
    /// Overall verification status
    pub status: VerificationStatus,
    /// Individual check results
    pub checks: Vec<VerificationCheck>,
    /// Warnings that don't fail verification
    pub warnings: Vec<String>,
    /// Errors that fail verification
    pub errors: Vec<String>,
}

/// Status of plugin verification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerificationStatus {
    /// Plugin passed all checks
    Passed,
    /// Plugin passed with warnings
    PassedWithWarnings,
    /// Plugin failed verification
    Failed,
}

/// Individual verification check result.
#[derive(Debug, Clone)]
pub struct VerificationCheck {
    /// Name of the check
    pub name: String,
    /// Whether the check passed
    pub passed: bool,
    /// Optional message
    pub message: Option<String>,
}

impl VerificationReport {
    /// Create a new verification report.
    pub fn new(plugin_id: PluginId) -> Self {
        Self {
            plugin_id,
            status: VerificationStatus::Passed,
            checks: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    /// Add a check result.
    pub fn add_check(&mut self, name: &str, passed: bool, message: Option<String>) {
        self.checks.push(VerificationCheck {
            name: name.to_string(),
            passed,
            message,
        });

        if !passed {
            self.status = VerificationStatus::Failed;
        }
    }

    /// Add a warning.
    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
        if self.status == VerificationStatus::Passed {
            self.status = VerificationStatus::PassedWithWarnings;
        }
    }

    /// Add an error.
    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
        self.status = VerificationStatus::Failed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_plugin_id_parsing() {
        let id = PluginId::parse("ui.tables").unwrap();
        assert_eq!(id.namespace().as_str(), "ui");
        assert_eq!(id.name(), "tables");

        let nested = PluginId::parse("theme.admin.modern.dark").unwrap();
        assert_eq!(nested.namespace().as_str(), "theme");
        assert_eq!(nested.name(), "admin.modern.dark");
    }

    #[test]
    fn test_plugin_manager_creation() {
        let dir = tempdir().unwrap();
        let manager = PluginManager::new(dir.path());
        assert!(manager.is_ok());
    }

    #[test]
    fn test_verification_report() {
        let id = PluginId::parse("ui.test").unwrap();
        let mut report = VerificationReport::new(id);

        report.add_check("manifest_valid", true, None);
        assert_eq!(report.status, VerificationStatus::Passed);

        report.add_warning("Minor issue".to_string());
        assert_eq!(report.status, VerificationStatus::PassedWithWarnings);

        report.add_check("permissions_valid", false, Some("Invalid permission".to_string()));
        assert_eq!(report.status, VerificationStatus::Failed);
    }
}
