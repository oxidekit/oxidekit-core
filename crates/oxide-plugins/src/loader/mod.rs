//! Plugin loading and verification.
//!
//! Handles loading plugins, verifying their integrity, and preparing them
//! for execution.

use std::path::{Path, PathBuf};
use std::fs;
use sha2::{Sha256, Digest};
use tracing::{debug, warn};

use crate::error::{PluginError, PluginResult};
use crate::manifest::PluginManifest;
use crate::trust::TrustLevel;
use crate::{VerificationReport, VerificationStatus};
use crate::namespace::PluginId;

/// Plugin loader for loading and verifying plugins.
#[derive(Debug)]
pub struct PluginLoader {
    /// Project root directory.
    project_root: PathBuf,
}

impl PluginLoader {
    /// Create a new plugin loader.
    pub fn new<P: AsRef<Path>>(project_root: P) -> Self {
        Self {
            project_root: project_root.as_ref().to_path_buf(),
        }
    }

    /// Verify a plugin's integrity and security.
    pub fn verify(
        &self,
        manifest: &PluginManifest,
        install_path: &Path,
    ) -> PluginResult<VerificationReport> {
        let mut report = VerificationReport::new(manifest.plugin.id.clone());

        // 1. Manifest sanity checks
        self.verify_manifest(manifest, &mut report);

        // 2. File integrity checks
        self.verify_files(install_path, &mut report)?;

        // 3. Capability checks
        self.verify_capabilities(manifest, &mut report);

        // 4. Dependency audit
        self.verify_dependencies(manifest, &mut report);

        // 5. Build script check
        self.verify_build_script(manifest, install_path, &mut report);

        // 6. Static analysis (basic)
        self.verify_static_analysis(manifest, install_path, &mut report)?;

        Ok(report)
    }

    /// Verify manifest sanity.
    fn verify_manifest(&self, manifest: &PluginManifest, report: &mut VerificationReport) {
        // Check version is valid
        if manifest.plugin.version.major == 0 && manifest.plugin.version.minor == 0 {
            report.add_warning("Plugin version 0.0.x is typically for unstable releases".to_string());
        }

        // Check for deprecated plugins
        if manifest.plugin.deprecated {
            if let Some(replacement) = &manifest.plugin.replaced_by {
                report.add_warning(format!(
                    "Plugin is deprecated, consider using {} instead",
                    replacement
                ));
            } else {
                report.add_warning("Plugin is deprecated".to_string());
            }
        }

        // Check for required fields
        if manifest.plugin.repository.is_none() {
            report.add_warning("No repository URL specified".to_string());
        }

        // Check kind-specific config
        if !manifest.has_valid_kind_config() {
            report.add_check(
                "kind_config",
                false,
                Some("Missing kind-specific configuration".to_string()),
            );
        } else {
            report.add_check("kind_config", true, None);
        }

        report.add_check("manifest_valid", true, None);
    }

    /// Verify file integrity.
    fn verify_files(&self, install_path: &Path, report: &mut VerificationReport) -> PluginResult<()> {
        // Check plugin.toml exists
        let manifest_path = install_path.join("plugin.toml");
        if !manifest_path.exists() {
            report.add_check(
                "manifest_exists",
                false,
                Some("plugin.toml not found".to_string()),
            );
            return Ok(());
        }
        report.add_check("manifest_exists", true, None);

        // Check for suspicious files
        let suspicious_patterns = [
            ".exe",
            ".bat",
            ".cmd",
            ".ps1",
            ".sh", // Shell scripts need scrutiny
        ];

        for entry in walkdir::WalkDir::new(install_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if suspicious_patterns.iter().any(|p| p.contains(&ext_str)) {
                    report.add_warning(format!(
                        "Suspicious file type found: {}",
                        path.display()
                    ));
                }
            }
        }

        report.add_check("file_integrity", true, None);
        Ok(())
    }

    /// Verify capabilities are appropriate.
    fn verify_capabilities(&self, manifest: &PluginManifest, report: &mut VerificationReport) {
        let capabilities = manifest.required_capabilities();
        let category = manifest.plugin.kind;
        let allowed = category.allowed_capabilities();

        for cap_str in &capabilities {
            if let Ok(cap) = cap_str.parse::<crate::permissions::Capability>() {
                if !allowed.contains(&cap) {
                    report.add_check(
                        "capability_allowed",
                        false,
                        Some(format!(
                            "Capability '{}' not allowed for {} plugins",
                            cap_str, category
                        )),
                    );
                    return;
                }
            }
        }

        // Check for high-risk capability combinations
        let has_spawn = capabilities.iter().any(|c| c == "process.spawn");
        let has_network = capabilities.iter().any(|c| c.starts_with("network."));
        let has_fs_write = capabilities.iter().any(|c| c == "filesystem.write");

        if has_spawn && has_network {
            report.add_warning(
                "Plugin requests both process spawn and network access - high risk combination"
                    .to_string(),
            );
        }

        if has_spawn && has_fs_write {
            report.add_warning(
                "Plugin requests both process spawn and filesystem write - high risk combination"
                    .to_string(),
            );
        }

        report.add_check("capabilities_valid", true, None);
    }

    /// Verify dependencies.
    fn verify_dependencies(&self, manifest: &PluginManifest, report: &mut VerificationReport) {
        // Check for self-dependency
        for dep in &manifest.dependencies.plugins {
            if dep.id == manifest.plugin.id.full_name() {
                report.add_check(
                    "no_self_dependency",
                    false,
                    Some("Plugin depends on itself".to_string()),
                );
                return;
            }
        }

        // Check dependency count (too many deps is a warning)
        let total_deps = manifest.dependencies.plugins.len()
            + manifest.dependencies.optional.len()
            + manifest.dependencies.peer.len();

        if total_deps > 20 {
            report.add_warning(format!(
                "Plugin has {} dependencies - consider reducing",
                total_deps
            ));
        }

        report.add_check("dependencies_valid", true, None);
    }

    /// Verify build script safety.
    fn verify_build_script(
        &self,
        manifest: &PluginManifest,
        install_path: &Path,
        report: &mut VerificationReport,
    ) {
        if let Some(build) = &manifest.build {
            if build.has_build_script {
                // Build scripts are high risk
                report.add_warning("Plugin has a build script - requires manual review".to_string());

                // Check if build script exists
                if let Some(script_path) = &build.build_script {
                    let full_path = install_path.join(script_path);
                    if !full_path.exists() {
                        report.add_check(
                            "build_script_exists",
                            false,
                            Some("Declared build script not found".to_string()),
                        );
                        return;
                    }
                }
            }
        }

        // Check for hidden build.rs in Rust plugins
        let build_rs = install_path.join("build.rs");
        if build_rs.exists() {
            report.add_warning("Plugin contains build.rs - may execute code during build".to_string());
        }

        report.add_check("build_script_safe", true, None);
    }

    /// Basic static analysis.
    fn verify_static_analysis(
        &self,
        manifest: &PluginManifest,
        install_path: &Path,
        report: &mut VerificationReport,
    ) -> PluginResult<()> {
        // Check for suspicious code patterns in Rust files
        let suspicious_patterns = [
            "std::process::Command",     // Process execution
            "std::env::var",             // Environment variable access
            "std::fs::remove",           // File deletion
            "unsafe {",                  // Unsafe code blocks
            "extern \"C\"",              // FFI
            "dlopen",                    // Dynamic loading
            "LoadLibrary",               // Windows dynamic loading
        ];

        let mut found_suspicious = Vec::new();

        for entry in walkdir::WalkDir::new(install_path)
            .max_depth(5)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.extension().map(|e| e == "rs").unwrap_or(false) {
                if let Ok(content) = fs::read_to_string(path) {
                    for pattern in &suspicious_patterns {
                        if content.contains(pattern) {
                            found_suspicious.push(format!(
                                "Found '{}' in {}",
                                pattern,
                                path.file_name().unwrap_or_default().to_string_lossy()
                            ));
                        }
                    }
                }
            }
        }

        if !found_suspicious.is_empty() {
            for finding in &found_suspicious {
                report.add_warning(finding.clone());
            }
        }

        report.add_check("static_analysis", true, None);
        Ok(())
    }

    /// Calculate the hash of a file.
    pub fn hash_file(path: &Path) -> PluginResult<String> {
        let content = fs::read(path)?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();
        Ok(hex::encode(result))
    }

    /// Calculate the hash of a directory.
    pub fn hash_directory(path: &Path) -> PluginResult<String> {
        let mut hasher = Sha256::new();

        let mut entries: Vec<_> = walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        // Sort for deterministic hashing
        entries.sort_by_key(|e| e.path().to_path_buf());

        for entry in entries {
            let file_path = entry.path();
            let relative = file_path.strip_prefix(path).unwrap_or(file_path);

            // Include path in hash
            hasher.update(relative.to_string_lossy().as_bytes());

            // Include file content
            if let Ok(content) = fs::read(file_path) {
                hasher.update(&content);
            }
        }

        let result = hasher.finalize();
        Ok(hex::encode(result))
    }
}

/// Plugin load state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadState {
    /// Plugin is not loaded.
    Unloaded,
    /// Plugin is being loaded.
    Loading,
    /// Plugin is loaded and ready.
    Loaded,
    /// Plugin failed to load.
    Failed,
}

/// Runtime information about a loaded plugin.
#[derive(Debug)]
pub struct LoadedPluginInfo {
    /// Plugin ID.
    pub id: PluginId,
    /// Load state.
    pub state: LoadState,
    /// Trust level.
    pub trust_level: TrustLevel,
    /// Path to the plugin.
    pub path: PathBuf,
    /// Hash of the plugin directory.
    pub hash: Option<String>,
    /// Load time in milliseconds.
    pub load_time_ms: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::io::Write;

    #[test]
    fn test_hash_file() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.txt");
        let mut file = fs::File::create(&file_path).unwrap();
        file.write_all(b"test content").unwrap();

        let hash = PluginLoader::hash_file(&file_path).unwrap();
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex length
    }

    #[test]
    fn test_hash_directory() {
        let dir = tempdir().unwrap();

        // Create some files
        fs::write(dir.path().join("a.txt"), "content a").unwrap();
        fs::write(dir.path().join("b.txt"), "content b").unwrap();

        let hash1 = PluginLoader::hash_directory(dir.path()).unwrap();
        assert!(!hash1.is_empty());

        // Same content should produce same hash
        let hash2 = PluginLoader::hash_directory(dir.path()).unwrap();
        assert_eq!(hash1, hash2);

        // Changing content should change hash
        fs::write(dir.path().join("a.txt"), "modified content").unwrap();
        let hash3 = PluginLoader::hash_directory(dir.path()).unwrap();
        assert_ne!(hash1, hash3);
    }
}
