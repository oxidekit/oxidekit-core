//! NPM Build-Time Bundling Tooling
//!
//! **NOTE**: This uses Node.js ONLY as a build tool, not at runtime.
//!
//! # Purpose
//!
//! Allow users to use npm packages to build web widgets without requiring
//! Node at runtime. All outputs are bundled into the app as widget bundles.
//!
//! # Commands
//!
//! ```bash
//! oxide compat npm add <pkg>@<version>
//! oxide compat npm build
//! ```
//!
//! # Security
//!
//! - All outputs hashed and pinned in `extensions.lock`
//! - Build steps reproducible where feasible
//! - No runtime Node.js dependency

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during NPM operations
#[derive(Error, Debug)]
pub enum NpmError {
    /// Node.js not found
    #[error("Node.js not found. Please install Node.js for build-time bundling.")]
    NodeNotFound,

    /// NPM not found
    #[error("NPM not found. Please install NPM for build-time bundling.")]
    NpmNotFound,

    /// Package not found
    #[error("Package not found: {0}")]
    PackageNotFound(String),

    /// Version constraint not satisfiable
    #[error("Version constraint not satisfiable: {0}@{1}")]
    VersionNotSatisfiable(String, String),

    /// Build failed
    #[error("Build failed: {0}")]
    BuildFailed(String),

    /// Hash mismatch
    #[error("Hash mismatch for {package}: expected {expected}, got {actual}")]
    HashMismatch {
        package: String,
        expected: String,
        actual: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// Lockfile error
    #[error("Lockfile error: {0}")]
    LockfileError(String),
}

/// An NPM package reference
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NpmPackage {
    /// Package name
    pub name: String,
    /// Version constraint (semver)
    pub version: String,
    /// Registry URL (default: https://registry.npmjs.org)
    pub registry: Option<String>,
}

impl NpmPackage {
    /// Create a new package reference
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            registry: None,
        }
    }

    /// Parse from package@version string
    pub fn parse(spec: &str) -> Option<Self> {
        let parts: Vec<&str> = spec.rsplitn(2, '@').collect();
        match parts.as_slice() {
            [version, name] => Some(Self::new(name, version)),
            [name] => Some(Self::new(name, "latest")),
            _ => None,
        }
    }

    /// Get the package spec string
    pub fn spec(&self) -> String {
        format!("{}@{}", self.name, self.version)
    }
}

/// Resolved package information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedPackage {
    /// Package reference
    pub package: NpmPackage,
    /// Resolved version
    pub resolved_version: String,
    /// Tarball URL
    pub tarball_url: String,
    /// Tarball SHA512 hash
    pub tarball_hash: String,
    /// Dependencies
    pub dependencies: Vec<NpmPackage>,
}

/// Build artifact from NPM bundling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildArtifact {
    /// Artifact name
    pub name: String,
    /// Output file path (relative to build directory)
    pub path: PathBuf,
    /// Content hash
    pub hash: String,
    /// Size in bytes
    pub size: u64,
    /// Content type
    pub content_type: String,
    /// Source packages that produced this artifact
    pub source_packages: Vec<String>,
}

/// NPM lockfile for reproducible builds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmLockfile {
    /// Lockfile format version
    pub version: u32,
    /// Packages in the lockfile
    pub packages: HashMap<String, LockedPackage>,
    /// Build artifacts
    pub artifacts: Vec<BuildArtifact>,
    /// Build metadata
    pub build_metadata: BuildMetadata,
}

impl Default for NpmLockfile {
    fn default() -> Self {
        Self {
            version: 1,
            packages: HashMap::new(),
            artifacts: Vec::new(),
            build_metadata: BuildMetadata::default(),
        }
    }
}

impl NpmLockfile {
    /// Load lockfile from path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, NpmError> {
        let content = std::fs::read_to_string(path)?;
        let lockfile: Self = serde_json::from_str(&content)?;
        Ok(lockfile)
    }

    /// Save lockfile to path
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), NpmError> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add a locked package
    pub fn add_package(&mut self, package: LockedPackage) {
        self.packages.insert(package.name.clone(), package);
    }

    /// Get a locked package
    pub fn get_package(&self, name: &str) -> Option<&LockedPackage> {
        self.packages.get(name)
    }

    /// Add a build artifact
    pub fn add_artifact(&mut self, artifact: BuildArtifact) {
        self.artifacts.push(artifact);
    }

    /// Verify all artifacts
    pub fn verify_artifacts<P: AsRef<Path>>(&self, build_dir: P) -> Result<(), NpmError> {
        for artifact in &self.artifacts {
            let path = build_dir.as_ref().join(&artifact.path);
            if !path.exists() {
                return Err(NpmError::HashMismatch {
                    package: artifact.name.clone(),
                    expected: artifact.hash.clone(),
                    actual: "file not found".to_string(),
                });
            }

            let content = std::fs::read(&path)?;
            let hash = compute_sha256_bytes(&content);

            if hash != artifact.hash {
                return Err(NpmError::HashMismatch {
                    package: artifact.name.clone(),
                    expected: artifact.hash.clone(),
                    actual: hash,
                });
            }
        }
        Ok(())
    }
}

/// A locked package entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LockedPackage {
    /// Package name
    pub name: String,
    /// Exact resolved version
    pub version: String,
    /// Tarball URL
    pub tarball_url: String,
    /// Tarball SHA512 hash
    pub tarball_hash: String,
    /// Integrity string (npm format)
    pub integrity: String,
    /// Resolved dependencies
    pub dependencies: HashMap<String, String>,
}

/// Build metadata
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildMetadata {
    /// Build timestamp
    pub timestamp: Option<String>,
    /// Node.js version used
    pub node_version: Option<String>,
    /// NPM version used
    pub npm_version: Option<String>,
    /// Build command
    pub build_command: Option<String>,
    /// Build environment
    pub environment: HashMap<String, String>,
}

/// NPM bundler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NpmBundlerConfig {
    /// Output directory for bundled assets
    pub output_dir: PathBuf,
    /// Temporary build directory
    pub build_dir: PathBuf,
    /// Entry point file
    pub entry_point: Option<String>,
    /// Output format (esm, cjs, iife)
    pub output_format: OutputFormat,
    /// Minify output
    pub minify: bool,
    /// Generate source maps
    pub source_maps: bool,
    /// External packages (not bundled)
    pub externals: Vec<String>,
    /// Target environment
    pub target: BundleTarget,
}

impl Default for NpmBundlerConfig {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("dist/widgets"),
            build_dir: PathBuf::from(".oxide/npm-build"),
            entry_point: None,
            output_format: OutputFormat::Iife,
            minify: true,
            source_maps: false,
            externals: Vec::new(),
            target: BundleTarget::Browser,
        }
    }
}

/// Output format for bundled JavaScript
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    /// ES Modules
    Esm,
    /// CommonJS
    Cjs,
    /// Immediately Invoked Function Expression (for browsers)
    Iife,
}

/// Bundle target environment
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BundleTarget {
    /// Browser environment
    Browser,
    /// Node.js environment (for build tools)
    Node,
}

/// NPM bundler for build-time package bundling
pub struct NpmBundler {
    /// Bundler configuration
    config: NpmBundlerConfig,
    /// Packages to bundle
    packages: Vec<NpmPackage>,
    /// Lockfile
    lockfile: NpmLockfile,
}

impl NpmBundler {
    /// Create a new bundler
    pub fn new(config: NpmBundlerConfig) -> Self {
        Self {
            config,
            packages: Vec::new(),
            lockfile: NpmLockfile::default(),
        }
    }

    /// Add a package to bundle
    pub fn add(&mut self, package: NpmPackage) {
        if !self.packages.contains(&package) {
            self.packages.push(package);
        }
    }

    /// Add a package by spec string (package@version)
    pub fn add_spec(&mut self, spec: &str) -> Result<(), NpmError> {
        let package =
            NpmPackage::parse(spec).ok_or_else(|| NpmError::PackageNotFound(spec.to_string()))?;
        self.add(package);
        Ok(())
    }

    /// Remove a package
    pub fn remove(&mut self, name: &str) {
        self.packages.retain(|p| p.name != name);
    }

    /// List packages
    pub fn packages(&self) -> &[NpmPackage] {
        &self.packages
    }

    /// Check if Node.js is available
    pub fn check_node() -> Result<String, NpmError> {
        let output = std::process::Command::new("node")
            .arg("--version")
            .output()
            .map_err(|_| NpmError::NodeNotFound)?;

        if !output.status.success() {
            return Err(NpmError::NodeNotFound);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Check if NPM is available
    pub fn check_npm() -> Result<String, NpmError> {
        let output = std::process::Command::new("npm")
            .arg("--version")
            .output()
            .map_err(|_| NpmError::NpmNotFound)?;

        if !output.status.success() {
            return Err(NpmError::NpmNotFound);
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    }

    /// Install packages (npm install)
    pub fn install(&mut self) -> Result<(), NpmError> {
        // Check prerequisites
        let node_version = Self::check_node()?;
        let npm_version = Self::check_npm()?;

        // Create build directory
        std::fs::create_dir_all(&self.config.build_dir)?;

        // Generate package.json
        let package_json = self.generate_package_json();
        let package_json_path = self.config.build_dir.join("package.json");
        std::fs::write(&package_json_path, serde_json::to_string_pretty(&package_json)?)?;

        // Run npm install
        let output = std::process::Command::new("npm")
            .arg("install")
            .arg("--production")
            .current_dir(&self.config.build_dir)
            .output()?;

        if !output.status.success() {
            return Err(NpmError::BuildFailed(
                String::from_utf8_lossy(&output.stderr).to_string(),
            ));
        }

        // Update metadata
        self.lockfile.build_metadata.node_version = Some(node_version);
        self.lockfile.build_metadata.npm_version = Some(npm_version);

        // Parse npm lockfile and update our lockfile
        self.parse_npm_lockfile()?;

        Ok(())
    }

    /// Generate package.json for the bundle
    fn generate_package_json(&self) -> serde_json::Value {
        let mut deps = serde_json::Map::new();
        for package in &self.packages {
            deps.insert(package.name.clone(), serde_json::json!(package.version));
        }

        serde_json::json!({
            "name": "oxide-npm-bundle",
            "version": "0.0.0",
            "private": true,
            "dependencies": deps
        })
    }

    /// Parse npm package-lock.json and update our lockfile
    fn parse_npm_lockfile(&mut self) -> Result<(), NpmError> {
        let lockfile_path = self.config.build_dir.join("package-lock.json");
        if !lockfile_path.exists() {
            return Ok(());
        }

        let content = std::fs::read_to_string(&lockfile_path)?;
        let npm_lock: serde_json::Value = serde_json::from_str(&content)?;

        // Extract packages from npm lockfile
        if let Some(packages) = npm_lock.get("packages").and_then(|p| p.as_object()) {
            for (path, info) in packages {
                // Skip the root package
                if path.is_empty() {
                    continue;
                }

                // Extract package name from path (node_modules/package-name)
                let name = path
                    .strip_prefix("node_modules/")
                    .unwrap_or(path)
                    .to_string();

                if let Some(info_obj) = info.as_object() {
                    let locked = LockedPackage {
                        name: name.clone(),
                        version: info_obj
                            .get("version")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        tarball_url: info_obj
                            .get("resolved")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        tarball_hash: "".to_string(),
                        integrity: info_obj
                            .get("integrity")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        dependencies: HashMap::new(),
                    };

                    self.lockfile.add_package(locked);
                }
            }
        }

        Ok(())
    }

    /// Build the bundle
    pub fn build(&mut self) -> Result<Vec<BuildArtifact>, NpmError> {
        // For now, we'll use a simple approach: copy installed packages
        // In a real implementation, we'd use esbuild or rollup

        let mut artifacts = Vec::new();

        // Create output directory
        std::fs::create_dir_all(&self.config.output_dir)?;

        // Generate bundle entry point
        let entry = self.generate_entry_point();
        let entry_path = self.config.build_dir.join("entry.js");
        std::fs::write(&entry_path, &entry)?;

        // In a real implementation, we'd run a bundler here
        // For now, just copy the entry point as a "bundle"
        let output_name = "bundle.js";
        let output_path = self.config.output_dir.join(output_name);

        // Simulate bundling by copying
        std::fs::copy(&entry_path, &output_path)?;

        let content = std::fs::read(&output_path)?;
        let artifact = BuildArtifact {
            name: output_name.to_string(),
            path: PathBuf::from(output_name),
            hash: compute_sha256_bytes(&content),
            size: content.len() as u64,
            content_type: "application/javascript".to_string(),
            source_packages: self.packages.iter().map(|p| p.spec()).collect(),
        };

        artifacts.push(artifact.clone());
        self.lockfile.add_artifact(artifact);

        // Update timestamp
        self.lockfile.build_metadata.timestamp =
            Some(chrono::Utc::now().to_rfc3339());

        Ok(artifacts)
    }

    /// Generate entry point that re-exports all packages
    fn generate_entry_point(&self) -> String {
        let mut code = String::new();
        code.push_str("// Auto-generated by oxide-compat NPM bundler\n\n");

        for package in &self.packages {
            let safe_name = package.name.replace('-', "_").replace('/', "_");
            code.push_str(&format!(
                "import * as {} from '{}';\n",
                safe_name, package.name
            ));
        }

        code.push_str("\nexport {\n");
        for package in &self.packages {
            let safe_name = package.name.replace('-', "_").replace('/', "_");
            code.push_str(&format!("  {},\n", safe_name));
        }
        code.push_str("};\n");

        code
    }

    /// Save lockfile
    pub fn save_lockfile<P: AsRef<Path>>(&self, path: P) -> Result<(), NpmError> {
        self.lockfile.save(path)
    }

    /// Load lockfile
    pub fn load_lockfile<P: AsRef<Path>>(&mut self, path: P) -> Result<(), NpmError> {
        self.lockfile = NpmLockfile::load(path)?;
        Ok(())
    }

    /// Clean build directory
    pub fn clean(&self) -> Result<(), NpmError> {
        if self.config.build_dir.exists() {
            std::fs::remove_dir_all(&self.config.build_dir)?;
        }
        Ok(())
    }
}

/// Compute SHA256 hash of bytes
fn compute_sha256_bytes(data: &[u8]) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Builder for NpmBundler
pub struct NpmBundlerBuilder {
    config: NpmBundlerConfig,
    packages: Vec<NpmPackage>,
}

impl NpmBundlerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: NpmBundlerConfig::default(),
            packages: Vec::new(),
        }
    }

    /// Set output directory
    pub fn output_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.output_dir = path.as_ref().to_path_buf();
        self
    }

    /// Set build directory
    pub fn build_dir<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.build_dir = path.as_ref().to_path_buf();
        self
    }

    /// Set entry point
    pub fn entry_point(mut self, entry: &str) -> Self {
        self.config.entry_point = Some(entry.to_string());
        self
    }

    /// Set output format
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.config.output_format = format;
        self
    }

    /// Enable minification
    pub fn minify(mut self, minify: bool) -> Self {
        self.config.minify = minify;
        self
    }

    /// Enable source maps
    pub fn source_maps(mut self, source_maps: bool) -> Self {
        self.config.source_maps = source_maps;
        self
    }

    /// Add external package
    pub fn external(mut self, name: &str) -> Self {
        self.config.externals.push(name.to_string());
        self
    }

    /// Set target environment
    pub fn target(mut self, target: BundleTarget) -> Self {
        self.config.target = target;
        self
    }

    /// Add a package
    pub fn package(mut self, package: NpmPackage) -> Self {
        self.packages.push(package);
        self
    }

    /// Add package by spec
    pub fn package_spec(mut self, spec: &str) -> Self {
        if let Some(pkg) = NpmPackage::parse(spec) {
            self.packages.push(pkg);
        }
        self
    }

    /// Build the bundler
    pub fn build(self) -> NpmBundler {
        let mut bundler = NpmBundler::new(self.config);
        for pkg in self.packages {
            bundler.add(pkg);
        }
        bundler
    }
}

impl Default for NpmBundlerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_package_parse() {
        let pkg = NpmPackage::parse("lodash@4.17.21").unwrap();
        assert_eq!(pkg.name, "lodash");
        assert_eq!(pkg.version, "4.17.21");

        let pkg = NpmPackage::parse("lodash").unwrap();
        assert_eq!(pkg.name, "lodash");
        assert_eq!(pkg.version, "latest");

        let pkg = NpmPackage::parse("@types/node@18.0.0").unwrap();
        assert_eq!(pkg.name, "@types/node");
        assert_eq!(pkg.version, "18.0.0");
    }

    #[test]
    fn test_package_spec() {
        let pkg = NpmPackage::new("lodash", "4.17.21");
        assert_eq!(pkg.spec(), "lodash@4.17.21");
    }

    #[test]
    fn test_bundler_builder() {
        let bundler = NpmBundlerBuilder::new()
            .output_dir("dist")
            .minify(true)
            .source_maps(false)
            .package_spec("lodash@4.17.21")
            .package_spec("dayjs@1.11.0")
            .build();

        assert_eq!(bundler.packages().len(), 2);
    }

    #[test]
    fn test_lockfile_serialization() {
        let mut lockfile = NpmLockfile::default();
        lockfile.add_package(LockedPackage {
            name: "test".to_string(),
            version: "1.0.0".to_string(),
            tarball_url: "https://example.com/test-1.0.0.tgz".to_string(),
            tarball_hash: "abc123".to_string(),
            integrity: "sha512-abc123".to_string(),
            dependencies: HashMap::new(),
        });

        let json = serde_json::to_string(&lockfile).unwrap();
        let parsed: NpmLockfile = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.packages.len(), 1);
        assert!(parsed.packages.contains_key("test"));
    }
}
