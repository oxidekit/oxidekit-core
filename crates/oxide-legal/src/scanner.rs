//! License scanner for Rust dependencies
//!
//! Scans Cargo.toml and cargo metadata to analyze dependency licenses.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use cargo_metadata::{MetadataCommand, Package};
use serde::{Deserialize, Serialize};
use crate::error::{LegalError, LegalResult};
use crate::license::{License, LicenseCategory, LicenseCompatibility};

/// License scanner for analyzing project dependencies
#[derive(Debug)]
pub struct LicenseScanner {
    /// Path to the Cargo.toml or project directory
    manifest_path: PathBuf,
    /// Whether to include dev dependencies
    include_dev: bool,
    /// Whether to include build dependencies
    include_build: bool,
    /// Cache of parsed licenses
    license_cache: HashMap<String, License>,
}

impl LicenseScanner {
    /// Create a new scanner for the given manifest path
    pub fn new(manifest_path: impl AsRef<Path>) -> Self {
        let path = manifest_path.as_ref();
        let manifest_path = if path.is_dir() {
            path.join("Cargo.toml")
        } else {
            path.to_path_buf()
        };

        Self {
            manifest_path,
            include_dev: false,
            include_build: false,
            license_cache: HashMap::new(),
        }
    }

    /// Include dev dependencies in the scan
    pub fn with_dev_dependencies(mut self) -> Self {
        self.include_dev = true;
        self
    }

    /// Include build dependencies in the scan
    pub fn with_build_dependencies(mut self) -> Self {
        self.include_build = true;
        self
    }

    /// Scan all dependencies and return results
    pub fn scan(&mut self) -> LegalResult<ScanResult> {
        let metadata = MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            .exec()?;

        let mut dependencies = Vec::new();
        let mut summary = ScanSummary::default();

        // Get the root package
        let root_package = metadata
            .root_package()
            .ok_or_else(|| LegalError::CargoMetadataError("No root package found".to_string()))?;

        // Process all packages
        for package in &metadata.packages {
            // Skip the root package
            if package.id == root_package.id {
                continue;
            }

            // Check if this is a dependency of the root
            let is_direct = self.is_direct_dependency(package, root_package);

            let dep_license = self.analyze_package(package, is_direct)?;

            // Update summary
            summary.total_dependencies += 1;
            match dep_license.license.category {
                LicenseCategory::PublicDomain | LicenseCategory::Permissive => {
                    summary.permissive_count += 1;
                }
                LicenseCategory::WeakCopyleft => {
                    summary.weak_copyleft_count += 1;
                }
                LicenseCategory::StrongCopyleft | LicenseCategory::NetworkCopyleft => {
                    summary.copyleft_count += 1;
                }
                LicenseCategory::Proprietary => {
                    summary.proprietary_count += 1;
                }
                LicenseCategory::Unknown => {
                    summary.unknown_count += 1;
                }
            }

            if !dep_license.license.osi_approved {
                summary.non_osi_approved += 1;
            }

            dependencies.push(dep_license);
        }

        Ok(ScanResult {
            project_name: root_package.name.clone(),
            project_version: root_package.version.to_string(),
            project_license: root_package.license.clone(),
            dependencies,
            summary,
        })
    }

    /// Check if a package is a direct dependency
    fn is_direct_dependency(&self, package: &Package, root: &Package) -> bool {
        root.dependencies
            .iter()
            .any(|dep| dep.name == package.name)
    }

    /// Analyze a single package
    fn analyze_package(&mut self, package: &Package, is_direct: bool) -> LegalResult<DependencyLicense> {
        let license_str = package.license.as_deref().unwrap_or("Unknown");

        let license = if let Some(cached) = self.license_cache.get(license_str) {
            cached.clone()
        } else {
            let parsed = License::parse(license_str)?;
            self.license_cache.insert(license_str.to_string(), parsed.clone());
            parsed
        };

        // Try to find license file
        let license_file = self.find_license_file(&package.manifest_path);

        Ok(DependencyLicense {
            name: package.name.clone(),
            version: package.version.to_string(),
            license,
            license_file,
            authors: package.authors.clone(),
            repository: package.repository.clone(),
            description: package.description.clone(),
            is_direct,
        })
    }

    /// Find license file for a package
    fn find_license_file(&self, manifest_path: &cargo_metadata::camino::Utf8Path) -> Option<PathBuf> {
        let package_dir = manifest_path.parent()?;

        for name in &["LICENSE", "LICENSE.txt", "LICENSE.md", "LICENSE-MIT", "LICENSE-APACHE", "COPYING"] {
            let path = package_dir.join(name);
            if std::path::Path::new(path.as_str()).exists() {
                return Some(PathBuf::from(path.as_str()));
            }
        }

        None
    }

    /// Quick scan that only returns license identifiers
    pub fn quick_scan(&mut self) -> LegalResult<Vec<(String, String, String)>> {
        let metadata = MetadataCommand::new()
            .manifest_path(&self.manifest_path)
            .exec()?;

        let root_id = metadata
            .root_package()
            .map(|p| p.id.clone());

        let mut results = Vec::new();
        for package in &metadata.packages {
            if Some(&package.id) == root_id.as_ref() {
                continue;
            }

            results.push((
                package.name.clone(),
                package.version.to_string(),
                package.license.clone().unwrap_or_else(|| "Unknown".to_string()),
            ));
        }

        Ok(results)
    }
}

/// Result of a license scan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanResult {
    /// Project name
    pub project_name: String,
    /// Project version
    pub project_version: String,
    /// Project's own license
    pub project_license: Option<String>,
    /// All dependency licenses
    pub dependencies: Vec<DependencyLicense>,
    /// Summary statistics
    pub summary: ScanSummary,
}

impl ScanResult {
    /// Get dependencies with a specific license category
    pub fn by_category(&self, category: LicenseCategory) -> Vec<&DependencyLicense> {
        self.dependencies
            .iter()
            .filter(|d| d.license.category == category)
            .collect()
    }

    /// Get dependencies with unknown licenses
    pub fn unknown_licenses(&self) -> Vec<&DependencyLicense> {
        self.by_category(LicenseCategory::Unknown)
    }

    /// Get copyleft dependencies (strong + network)
    pub fn copyleft_dependencies(&self) -> Vec<&DependencyLicense> {
        self.dependencies
            .iter()
            .filter(|d| {
                matches!(
                    d.license.category,
                    LicenseCategory::StrongCopyleft | LicenseCategory::NetworkCopyleft
                )
            })
            .collect()
    }

    /// Check compatibility with a target license
    pub fn check_compatibility(&self, target: &License) -> Vec<CompatibilityIssue> {
        let mut issues = Vec::new();

        for dep in &self.dependencies {
            let compat = LicenseCompatibility::check(&dep.license, target);
            if !compat.is_compatible {
                issues.push(CompatibilityIssue {
                    dependency: dep.name.clone(),
                    version: dep.version.clone(),
                    license: dep.license.spdx_id.clone(),
                    target_license: target.spdx_id.clone(),
                    notes: compat.notes,
                });
            }
        }

        issues
    }

    /// Get all unique licenses
    pub fn unique_licenses(&self) -> Vec<&License> {
        let mut seen = std::collections::HashSet::new();
        let mut licenses = Vec::new();

        for dep in &self.dependencies {
            if seen.insert(&dep.license.spdx_id) {
                licenses.push(&dep.license);
            }
        }

        licenses
    }

    /// Export to JSON
    pub fn to_json(&self) -> LegalResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Export to TOML
    pub fn to_toml(&self) -> LegalResult<String> {
        Ok(toml::to_string_pretty(self)?)
    }
}

/// License information for a dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyLicense {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Parsed license
    pub license: License,
    /// Path to license file (if found)
    pub license_file: Option<PathBuf>,
    /// Package authors
    pub authors: Vec<String>,
    /// Repository URL
    pub repository: Option<String>,
    /// Package description
    pub description: Option<String>,
    /// Whether this is a direct dependency
    pub is_direct: bool,
}

impl DependencyLicense {
    /// Get attribution text for this dependency
    pub fn attribution(&self) -> String {
        let mut text = format!("{} v{}\n", self.name, self.version);
        text.push_str(&format!("License: {}\n", self.license.spdx_id));

        if !self.authors.is_empty() {
            text.push_str(&format!("Authors: {}\n", self.authors.join(", ")));
        }

        if let Some(ref repo) = self.repository {
            text.push_str(&format!("Repository: {}\n", repo));
        }

        text
    }
}

/// Summary statistics from a scan
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ScanSummary {
    /// Total number of dependencies
    pub total_dependencies: usize,
    /// Number of permissive licenses
    pub permissive_count: usize,
    /// Number of weak copyleft licenses
    pub weak_copyleft_count: usize,
    /// Number of strong copyleft licenses
    pub copyleft_count: usize,
    /// Number of proprietary licenses
    pub proprietary_count: usize,
    /// Number of unknown licenses
    pub unknown_count: usize,
    /// Number of non-OSI-approved licenses
    pub non_osi_approved: usize,
}

impl ScanSummary {
    /// Check if the project has any copyleft dependencies
    pub fn has_copyleft(&self) -> bool {
        self.copyleft_count > 0 || self.weak_copyleft_count > 0
    }

    /// Check if the project has any unknown licenses
    pub fn has_unknown(&self) -> bool {
        self.unknown_count > 0
    }

    /// Check if the project is "clean" (all permissive)
    pub fn is_permissive_only(&self) -> bool {
        self.copyleft_count == 0
            && self.weak_copyleft_count == 0
            && self.proprietary_count == 0
            && self.unknown_count == 0
    }
}

/// A compatibility issue found during scanning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatibilityIssue {
    /// Dependency name
    pub dependency: String,
    /// Dependency version
    pub version: String,
    /// Dependency license
    pub license: String,
    /// Target license
    pub target_license: String,
    /// Notes about the incompatibility
    pub notes: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_creation() {
        let scanner = LicenseScanner::new(".");
        assert!(scanner.manifest_path.ends_with("Cargo.toml"));
    }

    #[test]
    fn test_scan_summary_default() {
        let summary = ScanSummary::default();
        assert!(!summary.has_copyleft());
        assert!(!summary.has_unknown());
        assert!(summary.is_permissive_only());
    }

    #[test]
    fn test_scan_summary_with_copyleft() {
        let summary = ScanSummary {
            copyleft_count: 1,
            ..Default::default()
        };
        assert!(summary.has_copyleft());
        assert!(!summary.is_permissive_only());
    }
}
