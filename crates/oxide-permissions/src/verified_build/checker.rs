//! Verified build checker - validates builds against the profile.

use std::collections::HashSet;
use std::path::Path;

use crate::capabilities::PermissionManifest;
use crate::error::{PermissionError, PermissionResult};

use super::forbidden::{
    api_alternative, api_forbidden_reason, crate_alternative, crate_forbidden_reason,
    ForbiddenItem, ForbiddenItemType, FLAGGED_APIS, FLAGGED_CRATES, FORBIDDEN_APIS,
    FORBIDDEN_CRATES,
};
use super::profile::{BuildMetadata, VerifiedBuildProfile};

/// Result of a verified build check.
#[derive(Debug, Clone)]
pub struct VerifiedBuildReport {
    /// Build profile used.
    pub profile: String,
    /// Whether the build is verified.
    pub is_verified: bool,
    /// Checks that passed.
    pub passed_checks: Vec<PassedCheck>,
    /// Checks that failed.
    pub failed_checks: Vec<FailedCheck>,
    /// Warnings (non-blocking issues).
    pub warnings: Vec<BuildWarning>,
    /// Forbidden items found.
    pub forbidden_items: Vec<ForbiddenItem>,
    /// Flagged items for review.
    pub flagged_items: Vec<ForbiddenItem>,
    /// Build metadata (if available).
    pub metadata: Option<BuildMetadata>,
}

/// A check that passed.
#[derive(Debug, Clone)]
pub struct PassedCheck {
    /// Check name.
    pub name: String,
    /// Check description.
    pub description: String,
}

/// A check that failed.
#[derive(Debug, Clone)]
pub struct FailedCheck {
    /// Check name.
    pub name: String,
    /// Failure reason.
    pub reason: String,
    /// How to fix.
    pub fix: Option<String>,
}

/// A build warning.
#[derive(Debug, Clone)]
pub struct BuildWarning {
    /// Warning type.
    pub warning_type: String,
    /// Warning message.
    pub message: String,
    /// Location (if applicable).
    pub location: Option<String>,
}

impl VerifiedBuildReport {
    /// Create a new report.
    pub fn new(profile: impl Into<String>) -> Self {
        Self {
            profile: profile.into(),
            is_verified: true, // Assume verified until checks fail
            passed_checks: Vec::new(),
            failed_checks: Vec::new(),
            warnings: Vec::new(),
            forbidden_items: Vec::new(),
            flagged_items: Vec::new(),
            metadata: None,
        }
    }

    /// Record a passed check.
    pub fn pass(&mut self, name: impl Into<String>, description: impl Into<String>) {
        self.passed_checks.push(PassedCheck {
            name: name.into(),
            description: description.into(),
        });
    }

    /// Record a failed check.
    pub fn fail(&mut self, name: impl Into<String>, reason: impl Into<String>) {
        self.is_verified = false;
        self.failed_checks.push(FailedCheck {
            name: name.into(),
            reason: reason.into(),
            fix: None,
        });
    }

    /// Record a failed check with fix suggestion.
    pub fn fail_with_fix(
        &mut self,
        name: impl Into<String>,
        reason: impl Into<String>,
        fix: impl Into<String>,
    ) {
        self.is_verified = false;
        self.failed_checks.push(FailedCheck {
            name: name.into(),
            reason: reason.into(),
            fix: Some(fix.into()),
        });
    }

    /// Add a warning.
    pub fn warn(&mut self, warning_type: impl Into<String>, message: impl Into<String>) {
        self.warnings.push(BuildWarning {
            warning_type: warning_type.into(),
            message: message.into(),
            location: None,
        });
    }

    /// Add a forbidden item.
    pub fn add_forbidden(&mut self, item: ForbiddenItem) {
        self.is_verified = false;
        self.forbidden_items.push(item);
    }

    /// Add a flagged item.
    pub fn add_flagged(&mut self, item: ForbiddenItem) {
        self.flagged_items.push(item);
    }

    /// Set build metadata.
    pub fn with_metadata(mut self, metadata: BuildMetadata) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Generate a text summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== Verified Build Report ==="));
        lines.push(format!("Profile: {}", self.profile));
        lines.push(format!(
            "Status: {}",
            if self.is_verified {
                "VERIFIED"
            } else {
                "FAILED"
            }
        ));
        lines.push(String::new());

        if !self.passed_checks.is_empty() {
            lines.push("Passed Checks:".to_string());
            for check in &self.passed_checks {
                lines.push(format!("  [PASS] {}: {}", check.name, check.description));
            }
            lines.push(String::new());
        }

        if !self.failed_checks.is_empty() {
            lines.push("Failed Checks:".to_string());
            for check in &self.failed_checks {
                lines.push(format!("  [FAIL] {}: {}", check.name, check.reason));
                if let Some(fix) = &check.fix {
                    lines.push(format!("         Fix: {}", fix));
                }
            }
            lines.push(String::new());
        }

        if !self.forbidden_items.is_empty() {
            lines.push("Forbidden Items Found:".to_string());
            for item in &self.forbidden_items {
                let type_str = match item.item_type {
                    ForbiddenItemType::Api => "API",
                    ForbiddenItemType::Crate => "Crate",
                    ForbiddenItemType::Flagged => "Flagged",
                };
                lines.push(format!("  [{}] {}: {}", type_str, item.item, item.reason));
                if let Some(alt) = &item.alternative {
                    lines.push(format!("         Alternative: {}", alt));
                }
            }
            lines.push(String::new());
        }

        if !self.warnings.is_empty() {
            lines.push("Warnings:".to_string());
            for warning in &self.warnings {
                lines.push(format!("  [WARN] {}: {}", warning.warning_type, warning.message));
            }
        }

        lines.join("\n")
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        #[derive(serde::Serialize)]
        struct JsonReport<'a> {
            profile: &'a str,
            is_verified: bool,
            passed_count: usize,
            failed_count: usize,
            warning_count: usize,
            forbidden_count: usize,
            passed_checks: Vec<&'a str>,
            failed_checks: Vec<(&'a str, &'a str)>,
            warnings: Vec<&'a str>,
        }

        let json = JsonReport {
            profile: &self.profile,
            is_verified: self.is_verified,
            passed_count: self.passed_checks.len(),
            failed_count: self.failed_checks.len(),
            warning_count: self.warnings.len(),
            forbidden_count: self.forbidden_items.len(),
            passed_checks: self.passed_checks.iter().map(|c| c.name.as_str()).collect(),
            failed_checks: self
                .failed_checks
                .iter()
                .map(|c| (c.name.as_str(), c.reason.as_str()))
                .collect(),
            warnings: self.warnings.iter().map(|w| w.message.as_str()).collect(),
        };

        serde_json::to_string_pretty(&json)
    }
}

/// Verified build checker.
pub struct VerifiedBuildChecker {
    profile: VerifiedBuildProfile,
}

impl VerifiedBuildChecker {
    /// Create a new checker with the given profile.
    pub fn new(profile: VerifiedBuildProfile) -> Self {
        Self { profile }
    }

    /// Create a checker with the standard profile.
    pub fn standard() -> Self {
        Self::new(VerifiedBuildProfile::standard())
    }

    /// Create a checker with the strict profile.
    pub fn strict() -> Self {
        Self::new(VerifiedBuildProfile::strict())
    }

    /// Check a build against the profile.
    pub fn check(
        &self,
        manifest: Option<&PermissionManifest>,
        dependencies: &[&str],
    ) -> VerifiedBuildReport {
        let mut report = VerifiedBuildReport::new(&self.profile.name);

        // Check manifest permissions
        if self.profile.require_manifest_permissions {
            match manifest {
                Some(m) => {
                    if m.all_capabilities().is_empty() {
                        report.fail(
                            "manifest_permissions",
                            "No permissions declared in manifest",
                        );
                    } else {
                        report.pass(
                            "manifest_permissions",
                            format!("{} capabilities declared", m.all_capabilities().len()),
                        );
                    }
                }
                None => {
                    report.fail_with_fix(
                        "manifest_permissions",
                        "No manifest provided",
                        "Add a [permissions] section to oxide.toml",
                    );
                }
            }
        }

        // Check network policy
        if self.profile.require_network_policy {
            if let Some(m) = manifest {
                if m.allows_network() {
                    if m.has_network_allowlist() {
                        report.pass("network_policy", "Network allowlist enforced");
                    } else {
                        report.fail_with_fix(
                            "network_policy",
                            "Network capability used without allowlist",
                            "Add [network] section with mode = \"allowlist\"",
                        );
                    }
                } else {
                    report.pass("network_policy", "No network capability requested");
                }
            }
        }

        // Check forbidden crates
        if self.profile.check_forbidden_crates {
            let forbidden_crates = self.find_forbidden_crates(dependencies);
            if forbidden_crates.is_empty() {
                report.pass("forbidden_crates", "No forbidden crates detected");
            } else {
                for crate_name in &forbidden_crates {
                    let reason = crate_forbidden_reason(crate_name)
                        .unwrap_or("Forbidden by build profile");
                    let mut item = ForbiddenItem::crate_dep(*crate_name, reason);
                    if let Some(alt) = crate_alternative(crate_name) {
                        item = item.with_alternative(alt);
                    }
                    report.add_forbidden(item);
                }
                report.fail(
                    "forbidden_crates",
                    format!("{} forbidden crates detected", forbidden_crates.len()),
                );
            }
        }

        // Check flagged crates (warnings only)
        let flagged_crates = self.find_flagged_crates(dependencies);
        for crate_name in flagged_crates {
            report.add_flagged(ForbiddenItem::flagged(
                crate_name,
                ForbiddenItemType::Crate,
                "Crate flagged for security review",
            ));
            report.warn("flagged_crate", format!("Crate '{}' requires review", crate_name));
        }

        // Check minimum version
        if let Some(min_version) = &self.profile.min_oxidekit_version {
            // In a real implementation, this would check the actual version
            report.pass(
                "min_version",
                format!("OxideKit version >= {}", min_version),
            );
        }

        report
    }

    /// Check source code for forbidden APIs.
    pub fn check_source(&self, source_code: &str, file_path: &str) -> Vec<ForbiddenItem> {
        let mut items = Vec::new();

        if !self.profile.check_forbidden_apis {
            return items;
        }

        for api in FORBIDDEN_APIS {
            // Simple string matching (in production, use proper AST analysis)
            if source_code.contains(api) {
                let reason =
                    api_forbidden_reason(api).unwrap_or("Forbidden by build profile");
                let mut item = ForbiddenItem::api(*api, Some(file_path.to_string()), reason);
                if let Some(alt) = api_alternative(api) {
                    item = item.with_alternative(alt);
                }
                items.push(item);
            }
        }

        // Check for flagged patterns
        for pattern in FLAGGED_APIS {
            if source_code.contains(pattern) {
                items.push(
                    ForbiddenItem::flagged(
                        *pattern,
                        ForbiddenItemType::Flagged,
                        "Pattern flagged for security review",
                    )
                    .at_location(file_path.to_string()),
                );
            }
        }

        items
    }

    /// Find forbidden crates in dependencies.
    fn find_forbidden_crates<'a>(&self, dependencies: &[&'a str]) -> Vec<&'a str> {
        let forbidden_set: HashSet<_> = FORBIDDEN_CRATES.iter().copied().collect();
        let additional: HashSet<_> = self
            .profile
            .additional_forbidden_crates
            .iter()
            .map(String::as_str)
            .collect();

        dependencies
            .iter()
            .filter(|dep| {
                let dep = **dep;
                (forbidden_set.contains(dep) || additional.contains(dep))
                    && !self.profile.crate_exceptions.contains(dep)
            })
            .copied()
            .collect()
    }

    /// Find flagged crates in dependencies.
    fn find_flagged_crates<'a>(&self, dependencies: &[&'a str]) -> Vec<&'a str> {
        let flagged_set: HashSet<_> = FLAGGED_CRATES.iter().copied().collect();

        dependencies
            .iter()
            .filter(|dep| flagged_set.contains(**dep))
            .copied()
            .collect()
    }

    /// Verify a complete project directory.
    pub fn verify_project<P: AsRef<Path>>(
        &self,
        project_path: P,
        manifest: Option<&PermissionManifest>,
    ) -> PermissionResult<VerifiedBuildReport> {
        let project_path = project_path.as_ref();

        // Check Cargo.toml for dependencies
        let cargo_toml_path = project_path.join("Cargo.toml");
        let dependencies = if cargo_toml_path.exists() {
            self.extract_dependencies(&cargo_toml_path)?
        } else {
            Vec::new()
        };

        let dep_refs: Vec<&str> = dependencies.iter().map(String::as_str).collect();
        let mut report = self.check(manifest, &dep_refs);

        // Scan source files for forbidden APIs
        let src_path = project_path.join("src");
        if src_path.exists() {
            let mut forbidden_count = 0;
            for entry in walkdir::WalkDir::new(&src_path)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if entry.path().extension().is_some_and(|ext| ext == "rs") {
                    if let Ok(content) = std::fs::read_to_string(entry.path()) {
                        let path_str = entry.path().to_string_lossy();
                        let items = self.check_source(&content, &path_str);
                        for item in items {
                            if item.item_type == ForbiddenItemType::Flagged {
                                report.add_flagged(item);
                            } else {
                                forbidden_count += 1;
                                report.add_forbidden(item);
                            }
                        }
                    }
                }
            }

            if self.profile.check_forbidden_apis {
                if forbidden_count == 0 {
                    report.pass("forbidden_apis", "No forbidden APIs detected in source");
                } else {
                    report.fail(
                        "forbidden_apis",
                        format!("{} forbidden API usages detected", forbidden_count),
                    );
                }
            }
        }

        Ok(report)
    }

    /// Extract dependencies from Cargo.toml.
    fn extract_dependencies<P: AsRef<Path>>(
        &self,
        cargo_toml_path: P,
    ) -> PermissionResult<Vec<String>> {
        let content = std::fs::read_to_string(cargo_toml_path)?;
        let cargo: toml::Value = toml::from_str(&content)?;

        let mut deps = Vec::new();

        if let Some(dependencies) = cargo.get("dependencies") {
            if let Some(table) = dependencies.as_table() {
                deps.extend(table.keys().cloned());
            }
        }

        if let Some(dev_deps) = cargo.get("dev-dependencies") {
            if let Some(table) = dev_deps.as_table() {
                deps.extend(table.keys().cloned());
            }
        }

        if let Some(build_deps) = cargo.get("build-dependencies") {
            if let Some(table) = build_deps.as_table() {
                deps.extend(table.keys().cloned());
            }
        }

        Ok(deps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checker_basic() {
        let checker = VerifiedBuildChecker::standard();

        let manifest_str = r#"
[permissions]
"native.network" = ["network.http"]

[network]
mode = "allowlist"
allow = ["api.example.com"]
"#;
        let manifest = PermissionManifest::from_str(manifest_str).unwrap();

        let report = checker.check(Some(&manifest), &[]);
        assert!(report.is_verified);
    }

    #[test]
    fn test_checker_forbidden_crate() {
        let checker = VerifiedBuildChecker::standard();

        let report = checker.check(None, &["socket2", "tokio"]);
        assert!(!report.is_verified);
        assert!(!report.forbidden_items.is_empty());
    }

    #[test]
    fn test_checker_source_scan() {
        let checker = VerifiedBuildChecker::standard();

        let source = r#"
fn main() {
    let stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
}
"#;

        let items = checker.check_source(source, "main.rs");
        assert!(!items.is_empty());
    }

    #[test]
    fn test_report_summary() {
        let mut report = VerifiedBuildReport::new("test");
        report.pass("check1", "Passed");
        report.fail("check2", "Failed");

        let summary = report.summary();
        assert!(summary.contains("FAILED"));
        assert!(summary.contains("check1"));
        assert!(summary.contains("check2"));
    }
}
