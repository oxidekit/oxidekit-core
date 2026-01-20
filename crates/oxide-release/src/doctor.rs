//! Doctor command - validate release configuration and environment

use crate::config::ReleaseConfig;
use crate::error::{ReleaseError, ReleaseResult};
use std::collections::HashMap;
use std::process::Command;

/// Result of a doctor check
#[derive(Debug, Clone)]
pub struct CheckResult {
    /// Name of the check
    pub name: String,
    /// Category of the check
    pub category: CheckCategory,
    /// Status of the check
    pub status: CheckStatus,
    /// Detailed message
    pub message: String,
    /// Recommendations if failed
    pub recommendations: Vec<String>,
}

impl CheckResult {
    /// Create a passed check
    pub fn pass(name: impl Into<String>, category: CheckCategory, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category,
            status: CheckStatus::Pass,
            message: message.into(),
            recommendations: vec![],
        }
    }

    /// Create a warning check
    pub fn warn(
        name: impl Into<String>,
        category: CheckCategory,
        message: impl Into<String>,
        recommendations: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: CheckStatus::Warning,
            message: message.into(),
            recommendations,
        }
    }

    /// Create a failed check
    pub fn fail(
        name: impl Into<String>,
        category: CheckCategory,
        message: impl Into<String>,
        recommendations: Vec<String>,
    ) -> Self {
        Self {
            name: name.into(),
            category,
            status: CheckStatus::Fail,
            message: message.into(),
            recommendations,
        }
    }

    /// Create a skipped check
    pub fn skip(name: impl Into<String>, category: CheckCategory, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            category,
            status: CheckStatus::Skip,
            message: message.into(),
            recommendations: vec![],
        }
    }
}

/// Check status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckStatus {
    /// Check passed
    Pass,
    /// Check passed with warnings
    Warning,
    /// Check failed
    Fail,
    /// Check skipped
    Skip,
}

impl CheckStatus {
    /// Get the status indicator symbol
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Pass => "[OK]",
            Self::Warning => "[WARN]",
            Self::Fail => "[FAIL]",
            Self::Skip => "[SKIP]",
        }
    }

    /// Get ANSI color code for the status
    pub fn color(&self) -> &'static str {
        match self {
            Self::Pass => "\x1b[32m",    // Green
            Self::Warning => "\x1b[33m", // Yellow
            Self::Fail => "\x1b[31m",    // Red
            Self::Skip => "\x1b[90m",    // Gray
        }
    }
}

/// Check category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CheckCategory {
    /// Project configuration
    Project,
    /// Build tools
    BuildTools,
    /// Code signing (macOS)
    MacOSSigning,
    /// Code signing (Windows)
    WindowsSigning,
    /// Notarization
    Notarization,
    /// Packaging tools
    Packaging,
    /// GitHub
    GitHub,
    /// Publishing
    Publishing,
}

impl std::fmt::Display for CheckCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Project => write!(f, "Project"),
            Self::BuildTools => write!(f, "Build Tools"),
            Self::MacOSSigning => write!(f, "macOS Signing"),
            Self::WindowsSigning => write!(f, "Windows Signing"),
            Self::Notarization => write!(f, "Notarization"),
            Self::Packaging => write!(f, "Packaging"),
            Self::GitHub => write!(f, "GitHub"),
            Self::Publishing => write!(f, "Publishing"),
        }
    }
}

/// Doctor - validates release environment
pub struct Doctor {
    config: Option<ReleaseConfig>,
    checks: Vec<CheckResult>,
}

impl Doctor {
    /// Create a new doctor instance
    pub fn new() -> Self {
        Self {
            config: None,
            checks: vec![],
        }
    }

    /// Create a new doctor instance with config
    pub fn with_config(config: ReleaseConfig) -> Self {
        Self {
            config: Some(config),
            checks: vec![],
        }
    }

    /// Run all checks
    pub fn run_all(&mut self) -> &[CheckResult] {
        self.checks.clear();

        // Project checks
        self.check_project();

        // Build tools
        self.check_build_tools();

        // Platform-specific checks
        #[cfg(target_os = "macos")]
        {
            self.check_macos_signing();
            self.check_notarization();
        }

        #[cfg(target_os = "windows")]
        self.check_windows_signing();

        // Packaging tools
        self.check_packaging_tools();

        // GitHub
        self.check_github();

        &self.checks
    }

    /// Check project configuration
    fn check_project(&mut self) {
        // Check oxide.toml exists
        if let Some(ref config) = self.config {
            self.checks.push(CheckResult::pass(
                "oxide.toml",
                CheckCategory::Project,
                format!("Found: {} v{}", config.app_name, config.version),
            ));

            // Check app ID format
            if config.app_id.contains('.') && config.app_id.split('.').count() >= 2 {
                self.checks.push(CheckResult::pass(
                    "App ID",
                    CheckCategory::Project,
                    format!("Valid reverse-domain format: {}", config.app_id),
                ));
            } else {
                self.checks.push(CheckResult::warn(
                    "App ID",
                    CheckCategory::Project,
                    format!("App ID '{}' should use reverse-domain notation", config.app_id),
                    vec!["Use format: com.company.appname".to_string()],
                ));
            }

            // Check version format
            if semver::Version::parse(&config.version).is_ok() {
                self.checks.push(CheckResult::pass(
                    "Version",
                    CheckCategory::Project,
                    format!("Valid semver: {}", config.version),
                ));
            } else {
                self.checks.push(CheckResult::fail(
                    "Version",
                    CheckCategory::Project,
                    format!("Invalid semver: {}", config.version),
                    vec!["Use format: MAJOR.MINOR.PATCH (e.g., 1.0.0)".to_string()],
                ));
            }
        } else {
            self.checks.push(CheckResult::fail(
                "oxide.toml",
                CheckCategory::Project,
                "Not found in current directory".to_string(),
                vec!["Run 'oxide init' to create a project".to_string()],
            ));
        }
    }

    /// Check build tools availability
    fn check_build_tools(&mut self) {
        // Check cargo
        if which::which("cargo").is_ok() {
            if let Ok(output) = Command::new("cargo").arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                self.checks.push(CheckResult::pass(
                    "cargo",
                    CheckCategory::BuildTools,
                    version.trim().to_string(),
                ));
            }
        } else {
            self.checks.push(CheckResult::fail(
                "cargo",
                CheckCategory::BuildTools,
                "Not found".to_string(),
                vec!["Install Rust from https://rustup.rs".to_string()],
            ));
        }

        // Check rustc
        if which::which("rustc").is_ok() {
            if let Ok(output) = Command::new("rustc").arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                self.checks.push(CheckResult::pass(
                    "rustc",
                    CheckCategory::BuildTools,
                    version.trim().to_string(),
                ));
            }
        } else {
            self.checks.push(CheckResult::fail(
                "rustc",
                CheckCategory::BuildTools,
                "Not found".to_string(),
                vec!["Install Rust from https://rustup.rs".to_string()],
            ));
        }

        // Check git
        if which::which("git").is_ok() {
            if let Ok(output) = Command::new("git").arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                self.checks.push(CheckResult::pass(
                    "git",
                    CheckCategory::BuildTools,
                    version.trim().to_string(),
                ));
            }
        } else {
            self.checks.push(CheckResult::warn(
                "git",
                CheckCategory::BuildTools,
                "Not found".to_string(),
                vec!["Install git for version control and changelog generation".to_string()],
            ));
        }
    }

    /// Check macOS signing setup
    #[cfg(target_os = "macos")]
    fn check_macos_signing(&mut self) {
        // Check codesign
        if which::which("codesign").is_ok() {
            self.checks.push(CheckResult::pass(
                "codesign",
                CheckCategory::MacOSSigning,
                "Available".to_string(),
            ));
        } else {
            self.checks.push(CheckResult::fail(
                "codesign",
                CheckCategory::MacOSSigning,
                "Not found".to_string(),
                vec!["Install Xcode Command Line Tools: xcode-select --install".to_string()],
            ));
        }

        // Check for signing identities
        if let Ok(output) = Command::new("security")
            .args(["find-identity", "-v", "-p", "codesigning"])
            .output()
        {
            let output_str = String::from_utf8_lossy(&output.stdout);
            let identities: Vec<&str> = output_str
                .lines()
                .filter(|l| l.contains("Developer ID Application") || l.contains("Apple Development"))
                .collect();

            if identities.is_empty() {
                self.checks.push(CheckResult::warn(
                    "Signing Identity",
                    CheckCategory::MacOSSigning,
                    "No valid signing identities found".to_string(),
                    vec![
                        "Install a Developer ID Application certificate from Apple Developer Portal".to_string(),
                        "For testing, use 'Apple Development' certificate".to_string(),
                    ],
                ));
            } else {
                for identity in identities.iter().take(3) {
                    self.checks.push(CheckResult::pass(
                        "Signing Identity",
                        CheckCategory::MacOSSigning,
                        identity.trim().to_string(),
                    ));
                }
            }
        }

        // Check configured identity
        if let Some(ref config) = self.config {
            if let Some(ref signing) = config.signing {
                if let Some(ref identity) = signing.identity {
                    // Verify the identity exists
                    if let Ok(output) = Command::new("security")
                        .args(["find-identity", "-v", "-p", "codesigning"])
                        .output()
                    {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if output_str.contains(identity) {
                            self.checks.push(CheckResult::pass(
                                "Configured Identity",
                                CheckCategory::MacOSSigning,
                                format!("Found: {}", identity),
                            ));
                        } else {
                            self.checks.push(CheckResult::fail(
                                "Configured Identity",
                                CheckCategory::MacOSSigning,
                                format!("Not found: {}", identity),
                                vec![
                                    "Check the identity name in oxide.toml".to_string(),
                                    "Run 'security find-identity -v -p codesigning' to list available".to_string(),
                                ],
                            ));
                        }
                    }
                }
            }
        }
    }

    /// Check notarization setup
    #[cfg(target_os = "macos")]
    fn check_notarization(&mut self) {
        // Check notarytool/altool
        if which::which("xcrun").is_ok() {
            if let Ok(output) = Command::new("xcrun")
                .args(["notarytool", "--version"])
                .output()
            {
                if output.status.success() {
                    self.checks.push(CheckResult::pass(
                        "notarytool",
                        CheckCategory::Notarization,
                        "Available via xcrun".to_string(),
                    ));
                } else {
                    self.checks.push(CheckResult::warn(
                        "notarytool",
                        CheckCategory::Notarization,
                        "Not available".to_string(),
                        vec!["Install Xcode 13+ for notarytool support".to_string()],
                    ));
                }
            }
        }

        // Check credentials
        if let Some(ref config) = self.config {
            if let Some(ref notarization) = config.notarization {
                // Check for API key auth
                if notarization.api_key_id.is_some() && notarization.api_issuer_id.is_some() {
                    if let Some(ref key_path) = notarization.api_key_path {
                        if key_path.exists() {
                            self.checks.push(CheckResult::pass(
                                "Notarization Credentials",
                                CheckCategory::Notarization,
                                "API key authentication configured".to_string(),
                            ));
                        } else {
                            self.checks.push(CheckResult::fail(
                                "Notarization Credentials",
                                CheckCategory::Notarization,
                                format!("API key file not found: {}", key_path.display()),
                                vec!["Download API key from App Store Connect".to_string()],
                            ));
                        }
                    }
                } else if notarization.apple_id.is_some() && notarization.password.is_some() {
                    self.checks.push(CheckResult::pass(
                        "Notarization Credentials",
                        CheckCategory::Notarization,
                        "Apple ID authentication configured".to_string(),
                    ));
                } else {
                    self.checks.push(CheckResult::warn(
                        "Notarization Credentials",
                        CheckCategory::Notarization,
                        "No credentials configured".to_string(),
                        vec![
                            "Set OXIDE_APPLE_ID and OXIDE_APPLE_PASSWORD for Apple ID auth".to_string(),
                            "Or use API key auth with OXIDE_APPLE_KEY_ID and OXIDE_APPLE_ISSUER_ID".to_string(),
                        ],
                    ));
                }
            }
        }
    }

    /// Check Windows signing setup
    #[cfg(target_os = "windows")]
    fn check_windows_signing(&mut self) {
        // Check signtool
        if which::which("signtool").is_ok() {
            self.checks.push(CheckResult::pass(
                "signtool",
                CheckCategory::WindowsSigning,
                "Available".to_string(),
            ));
        } else {
            self.checks.push(CheckResult::warn(
                "signtool",
                CheckCategory::WindowsSigning,
                "Not found in PATH".to_string(),
                vec![
                    "Install Windows SDK".to_string(),
                    "Add SDK bin directory to PATH".to_string(),
                ],
            ));
        }

        // Check configured certificate
        if let Some(ref config) = self.config {
            if let Some(ref signing) = config.signing {
                if let Some(ref cert_path) = signing.certificate_path {
                    if cert_path.exists() {
                        self.checks.push(CheckResult::pass(
                            "Signing Certificate",
                            CheckCategory::WindowsSigning,
                            format!("Found: {}", cert_path.display()),
                        ));
                    } else {
                        self.checks.push(CheckResult::fail(
                            "Signing Certificate",
                            CheckCategory::WindowsSigning,
                            format!("Not found: {}", cert_path.display()),
                            vec!["Provide valid .pfx certificate file".to_string()],
                        ));
                    }
                }
            }
        }
    }

    /// Check packaging tools
    fn check_packaging_tools(&mut self) {
        #[cfg(target_os = "macos")]
        {
            // Check create-dmg or hdiutil
            if which::which("create-dmg").is_ok() {
                self.checks.push(CheckResult::pass(
                    "create-dmg",
                    CheckCategory::Packaging,
                    "Available".to_string(),
                ));
            } else if which::which("hdiutil").is_ok() {
                self.checks.push(CheckResult::pass(
                    "hdiutil",
                    CheckCategory::Packaging,
                    "Available (built-in DMG creation)".to_string(),
                ));
            }

            // Check productbuild for PKG
            if which::which("productbuild").is_ok() {
                self.checks.push(CheckResult::pass(
                    "productbuild",
                    CheckCategory::Packaging,
                    "Available (PKG creation)".to_string(),
                ));
            }
        }

        #[cfg(target_os = "windows")]
        {
            // Check WiX Toolset
            if which::which("candle").is_ok() && which::which("light").is_ok() {
                self.checks.push(CheckResult::pass(
                    "WiX Toolset",
                    CheckCategory::Packaging,
                    "Available (MSI creation)".to_string(),
                ));
            } else {
                self.checks.push(CheckResult::warn(
                    "WiX Toolset",
                    CheckCategory::Packaging,
                    "Not found".to_string(),
                    vec!["Install WiX Toolset from https://wixtoolset.org".to_string()],
                ));
            }
        }

        #[cfg(target_os = "linux")]
        {
            // Check appimagetool
            if which::which("appimagetool").is_ok() {
                self.checks.push(CheckResult::pass(
                    "appimagetool",
                    CheckCategory::Packaging,
                    "Available (AppImage creation)".to_string(),
                ));
            } else {
                self.checks.push(CheckResult::warn(
                    "appimagetool",
                    CheckCategory::Packaging,
                    "Not found".to_string(),
                    vec!["Download from https://appimage.github.io/appimagetool".to_string()],
                ));
            }

            // Check dpkg-deb
            if which::which("dpkg-deb").is_ok() {
                self.checks.push(CheckResult::pass(
                    "dpkg-deb",
                    CheckCategory::Packaging,
                    "Available (DEB creation)".to_string(),
                ));
            }

            // Check rpmbuild
            if which::which("rpmbuild").is_ok() {
                self.checks.push(CheckResult::pass(
                    "rpmbuild",
                    CheckCategory::Packaging,
                    "Available (RPM creation)".to_string(),
                ));
            }
        }
    }

    /// Check GitHub setup
    fn check_github(&mut self) {
        // Check gh CLI
        if which::which("gh").is_ok() {
            if let Ok(output) = Command::new("gh").arg("--version").output() {
                let version = String::from_utf8_lossy(&output.stdout);
                self.checks.push(CheckResult::pass(
                    "gh CLI",
                    CheckCategory::GitHub,
                    version.lines().next().unwrap_or("installed").to_string(),
                ));

                // Check auth status
                if let Ok(auth_output) = Command::new("gh").args(["auth", "status"]).output() {
                    if auth_output.status.success() {
                        self.checks.push(CheckResult::pass(
                            "gh auth",
                            CheckCategory::GitHub,
                            "Authenticated".to_string(),
                        ));
                    } else {
                        self.checks.push(CheckResult::warn(
                            "gh auth",
                            CheckCategory::GitHub,
                            "Not authenticated".to_string(),
                            vec!["Run 'gh auth login' to authenticate".to_string()],
                        ));
                    }
                }
            }
        } else {
            self.checks.push(CheckResult::warn(
                "gh CLI",
                CheckCategory::GitHub,
                "Not installed".to_string(),
                vec![
                    "Install from https://cli.github.com".to_string(),
                    "Or use GITHUB_TOKEN environment variable".to_string(),
                ],
            ));
        }

        // Check GITHUB_TOKEN
        if std::env::var("GITHUB_TOKEN").is_ok() {
            self.checks.push(CheckResult::pass(
                "GITHUB_TOKEN",
                CheckCategory::GitHub,
                "Set".to_string(),
            ));
        } else {
            self.checks.push(CheckResult::skip(
                "GITHUB_TOKEN",
                CheckCategory::GitHub,
                "Not set (optional if using gh CLI)".to_string(),
            ));
        }
    }

    /// Get all check results
    pub fn results(&self) -> &[CheckResult] {
        &self.checks
    }

    /// Get results grouped by category
    pub fn results_by_category(&self) -> HashMap<CheckCategory, Vec<&CheckResult>> {
        let mut map: HashMap<CheckCategory, Vec<&CheckResult>> = HashMap::new();
        for check in &self.checks {
            map.entry(check.category).or_default().push(check);
        }
        map
    }

    /// Check if all required checks passed
    pub fn all_passed(&self) -> bool {
        !self.checks.iter().any(|c| c.status == CheckStatus::Fail)
    }

    /// Get all failed checks
    pub fn failures(&self) -> Vec<&CheckResult> {
        self.checks
            .iter()
            .filter(|c| c.status == CheckStatus::Fail)
            .collect()
    }

    /// Get all warnings
    pub fn warnings(&self) -> Vec<&CheckResult> {
        self.checks
            .iter()
            .filter(|c| c.status == CheckStatus::Warning)
            .collect()
    }

    /// Format results for display
    pub fn format_results(&self, colored: bool) -> String {
        let mut output = String::new();
        let reset = if colored { "\x1b[0m" } else { "" };

        let by_category = self.results_by_category();
        let categories = [
            CheckCategory::Project,
            CheckCategory::BuildTools,
            CheckCategory::MacOSSigning,
            CheckCategory::WindowsSigning,
            CheckCategory::Notarization,
            CheckCategory::Packaging,
            CheckCategory::GitHub,
            CheckCategory::Publishing,
        ];

        for category in categories {
            if let Some(checks) = by_category.get(&category) {
                output.push_str(&format!("\n{}\n", category));
                output.push_str(&"-".repeat(40));
                output.push('\n');

                for check in checks {
                    let color = if colored { check.status.color() } else { "" };
                    output.push_str(&format!(
                        "{}{}{} {}: {}\n",
                        color,
                        check.status.symbol(),
                        reset,
                        check.name,
                        check.message
                    ));

                    for rec in &check.recommendations {
                        output.push_str(&format!("      -> {}\n", rec));
                    }
                }
            }
        }

        // Summary
        output.push_str(&format!("\n{}\n", "=".repeat(40)));
        let passed = self.checks.iter().filter(|c| c.status == CheckStatus::Pass).count();
        let warned = self.checks.iter().filter(|c| c.status == CheckStatus::Warning).count();
        let failed = self.checks.iter().filter(|c| c.status == CheckStatus::Fail).count();
        let skipped = self.checks.iter().filter(|c| c.status == CheckStatus::Skip).count();

        output.push_str(&format!(
            "Summary: {} passed, {} warnings, {} failed, {} skipped\n",
            passed, warned, failed, skipped
        ));

        if failed > 0 {
            output.push_str("\nFix the failed checks before releasing.\n");
        } else if warned > 0 {
            output.push_str("\nAll required checks passed. Consider addressing warnings.\n");
        } else {
            output.push_str("\nAll checks passed! Ready to release.\n");
        }

        output
    }
}

impl Default for Doctor {
    fn default() -> Self {
        Self::new()
    }
}

/// Run doctor checks and return error if critical checks fail
pub fn run_doctor(config: Option<ReleaseConfig>) -> ReleaseResult<Doctor> {
    let mut doctor = match config {
        Some(c) => Doctor::with_config(c),
        None => Doctor::new(),
    };

    doctor.run_all();

    if !doctor.all_passed() {
        let failures = doctor.failures();
        let mut recommendations = Vec::new();
        for f in failures {
            recommendations.extend(f.recommendations.clone());
        }

        return Err(ReleaseError::doctor_failed(
            "Some required checks failed",
            recommendations,
        ));
    }

    Ok(doctor)
}
