//! Mobile build configuration.
//!
//! Provides configuration types for mobile app builds including
//! app metadata, version information, and platform-specific settings.

use std::collections::HashMap;
use std::path::PathBuf;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use crate::target::{AndroidAbi, IosArch, MobilePlatform, MobileTarget};

/// Main mobile configuration for an OxideKit application.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileConfig {
    /// Application metadata.
    pub app: AppConfig,
    /// iOS-specific configuration.
    #[serde(default)]
    pub ios: IosConfig,
    /// Android-specific configuration.
    #[serde(default)]
    pub android: AndroidConfig,
}

impl MobileConfig {
    /// Create a new mobile configuration with defaults.
    pub fn new(name: impl Into<String>, bundle_id: impl Into<String>) -> Self {
        let name = name.into();
        let bundle_id = bundle_id.into();

        Self {
            app: AppConfig {
                name: name.clone(),
                bundle_id: bundle_id.clone(),
                version: "1.0.0".into(),
                build_number: 1,
                display_name: None,
            },
            ios: IosConfig::default(),
            android: AndroidConfig {
                package_name: bundle_id,
                ..Default::default()
            },
        }
    }

    /// Get configuration for a specific target.
    pub fn for_target(&self, target: MobileTarget) -> TargetConfig {
        match target.platform() {
            MobilePlatform::Ios => TargetConfig::Ios(&self.ios),
            MobilePlatform::Android => TargetConfig::Android(&self.android),
        }
    }

    /// Validate the configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        if self.app.name.is_empty() {
            errors.push("App name cannot be empty".into());
        }

        if self.app.bundle_id.is_empty() {
            errors.push("Bundle ID cannot be empty".into());
        }

        // Validate bundle ID format (reverse DNS)
        if !self.app.bundle_id.contains('.') {
            errors.push("Bundle ID should be in reverse DNS format (e.g., com.example.app)".into());
        }

        // Validate version format
        let parts: Vec<&str> = self.app.version.split('.').collect();
        if parts.len() != 3 || parts.iter().any(|p| p.parse::<u32>().is_err()) {
            errors.push("Version should be in semver format (e.g., 1.0.0)".into());
        }

        // Validate Android min SDK
        if self.android.min_sdk_version < 21 {
            errors.push("Android min SDK version must be at least 21".into());
        }

        // Validate iOS deployment target
        if self.ios.deployment_target.is_empty() {
            errors.push("iOS deployment target cannot be empty".into());
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for MobileConfig {
    fn default() -> Self {
        Self::new("MyApp", "com.example.myapp")
    }
}

/// Application metadata shared across platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    /// Internal app name (used for code generation).
    pub name: String,
    /// Bundle identifier (reverse DNS format).
    pub bundle_id: String,
    /// Version string (semver format: major.minor.patch).
    pub version: String,
    /// Build number (integer, incremented for each build).
    pub build_number: u32,
    /// Display name shown to users (defaults to name if not set).
    pub display_name: Option<String>,
}

impl AppConfig {
    /// Get the display name, falling back to the internal name.
    pub fn display_name(&self) -> &str {
        self.display_name.as_deref().unwrap_or(&self.name)
    }
}

/// iOS-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosConfig {
    /// Minimum iOS version (e.g., "15.0").
    pub deployment_target: String,
    /// Development team ID.
    pub team_id: Option<String>,
    /// Code signing identity.
    pub signing_identity: Option<String>,
    /// Provisioning profile name or UUID.
    pub provisioning_profile: Option<String>,
    /// Target architectures.
    #[serde(default = "IosConfig::default_architectures")]
    pub architectures: Vec<IosArch>,
    /// Device capabilities required.
    #[serde(default)]
    pub capabilities: Vec<String>,
    /// Info.plist overrides.
    #[serde(default)]
    pub info_plist: HashMap<String, serde_json::Value>,
    /// Additional entitlements.
    #[serde(default)]
    pub entitlements: HashMap<String, serde_json::Value>,
    /// App icon asset catalog name.
    #[serde(default = "IosConfig::default_app_icon")]
    pub app_icon: String,
    /// Launch storyboard name.
    pub launch_storyboard: Option<String>,
}

impl IosConfig {
    fn default_architectures() -> Vec<IosArch> {
        vec![IosArch::Arm64]
    }

    fn default_app_icon() -> String {
        "AppIcon".into()
    }
}

impl Default for IosConfig {
    fn default() -> Self {
        Self {
            deployment_target: "15.0".into(),
            team_id: None,
            signing_identity: None,
            provisioning_profile: None,
            architectures: Self::default_architectures(),
            capabilities: Vec::new(),
            info_plist: HashMap::new(),
            entitlements: HashMap::new(),
            app_icon: Self::default_app_icon(),
            launch_storyboard: None,
        }
    }
}

/// Android-specific configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidConfig {
    /// Package name (Java package format).
    pub package_name: String,
    /// Minimum SDK version.
    pub min_sdk_version: u32,
    /// Target SDK version.
    pub target_sdk_version: u32,
    /// Compile SDK version.
    pub compile_sdk_version: u32,
    /// Target ABIs.
    #[serde(default = "AndroidConfig::default_abis")]
    pub abis: Vec<AndroidAbi>,
    /// Android permissions required.
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Keystore path for signing.
    pub keystore_path: Option<Utf8PathBuf>,
    /// Keystore alias.
    pub keystore_alias: Option<String>,
    /// Gradle properties overrides.
    #[serde(default)]
    pub gradle_properties: HashMap<String, String>,
    /// AndroidManifest.xml additions.
    #[serde(default)]
    pub manifest_additions: HashMap<String, String>,
    /// Enable multidex.
    #[serde(default)]
    pub multidex: bool,
    /// Enable ProGuard/R8 minification.
    #[serde(default)]
    pub minify: bool,
}

impl AndroidConfig {
    fn default_abis() -> Vec<AndroidAbi> {
        vec![AndroidAbi::Arm64V8a, AndroidAbi::ArmeabiV7a]
    }
}

impl Default for AndroidConfig {
    fn default() -> Self {
        Self {
            package_name: "com.example.myapp".into(),
            min_sdk_version: 24,
            target_sdk_version: 34,
            compile_sdk_version: 34,
            abis: Self::default_abis(),
            permissions: Vec::new(),
            keystore_path: None,
            keystore_alias: None,
            gradle_properties: HashMap::new(),
            manifest_additions: HashMap::new(),
            multidex: false,
            minify: false,
        }
    }
}

/// Platform-specific configuration reference.
#[derive(Debug, Clone, Copy)]
pub enum TargetConfig<'a> {
    /// iOS configuration.
    Ios(&'a IosConfig),
    /// Android configuration.
    Android(&'a AndroidConfig),
}

/// Build profile (debug vs release).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum BuildProfile {
    /// Debug build with debug symbols and no optimization.
    #[default]
    Debug,
    /// Release build with optimization and potentially stripped symbols.
    Release,
}

impl BuildProfile {
    /// Returns true if this is a debug build.
    pub fn is_debug(&self) -> bool {
        matches!(self, BuildProfile::Debug)
    }

    /// Returns true if this is a release build.
    pub fn is_release(&self) -> bool {
        matches!(self, BuildProfile::Release)
    }

    /// Get the Cargo profile name.
    pub fn cargo_profile(&self) -> &'static str {
        match self {
            BuildProfile::Debug => "debug",
            BuildProfile::Release => "release",
        }
    }
}

impl std::fmt::Display for BuildProfile {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.cargo_profile())
    }
}

/// Build options for mobile builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuildOptions {
    /// Target to build for.
    pub target: MobileTarget,
    /// Build profile.
    #[serde(default)]
    pub profile: BuildProfile,
    /// Output directory.
    pub output_dir: PathBuf,
    /// Enable verbose output.
    #[serde(default)]
    pub verbose: bool,
    /// Number of parallel jobs.
    pub jobs: Option<u32>,
    /// Additional environment variables.
    #[serde(default)]
    pub env: HashMap<String, String>,
    /// Additional Cargo features to enable.
    #[serde(default)]
    pub features: Vec<String>,
    /// Skip code signing (for debug builds).
    #[serde(default)]
    pub skip_signing: bool,
}

impl BuildOptions {
    /// Create new build options for a target.
    pub fn new(target: MobileTarget, output_dir: impl Into<PathBuf>) -> Self {
        Self {
            target,
            profile: BuildProfile::Debug,
            output_dir: output_dir.into(),
            verbose: false,
            jobs: None,
            env: HashMap::new(),
            features: Vec::new(),
            skip_signing: false,
        }
    }

    /// Set the build profile.
    pub fn with_profile(mut self, profile: BuildProfile) -> Self {
        self.profile = profile;
        self
    }

    /// Enable verbose output.
    pub fn verbose(mut self) -> Self {
        self.verbose = true;
        self
    }

    /// Skip code signing.
    pub fn skip_signing(mut self) -> Self {
        self.skip_signing = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_config_new() {
        let config = MobileConfig::new("TestApp", "com.test.app");
        assert_eq!(config.app.name, "TestApp");
        assert_eq!(config.app.bundle_id, "com.test.app");
        assert_eq!(config.android.package_name, "com.test.app");
    }

    #[test]
    fn test_mobile_config_validate() {
        let config = MobileConfig::new("TestApp", "com.test.app");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_mobile_config_validate_invalid() {
        let mut config = MobileConfig::default();
        config.app.bundle_id = "invalid".into(); // Missing dots
        config.app.version = "1.0".into(); // Invalid semver

        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors.iter().any(|e| e.contains("reverse DNS")));
        assert!(errors.iter().any(|e| e.contains("semver")));
    }

    #[test]
    fn test_build_profile() {
        assert!(BuildProfile::Debug.is_debug());
        assert!(!BuildProfile::Debug.is_release());
        assert!(BuildProfile::Release.is_release());
        assert_eq!(BuildProfile::Debug.cargo_profile(), "debug");
        assert_eq!(BuildProfile::Release.cargo_profile(), "release");
    }

    #[test]
    fn test_build_options() {
        let options = BuildOptions::new(MobileTarget::IosSimulator, "/tmp/build")
            .with_profile(BuildProfile::Release)
            .verbose()
            .skip_signing();

        assert_eq!(options.target, MobileTarget::IosSimulator);
        assert_eq!(options.profile, BuildProfile::Release);
        assert!(options.verbose);
        assert!(options.skip_signing);
    }

    #[test]
    fn test_app_config_display_name() {
        let mut app = AppConfig {
            name: "InternalName".into(),
            bundle_id: "com.test.app".into(),
            version: "1.0.0".into(),
            build_number: 1,
            display_name: None,
        };

        assert_eq!(app.display_name(), "InternalName");

        app.display_name = Some("My Cool App".into());
        assert_eq!(app.display_name(), "My Cool App");
    }

    #[test]
    fn test_ios_config_defaults() {
        let ios = IosConfig::default();
        assert_eq!(ios.deployment_target, "15.0");
        assert!(ios.architectures.contains(&IosArch::Arm64));
    }

    #[test]
    fn test_android_config_defaults() {
        let android = AndroidConfig::default();
        assert!(android.min_sdk_version >= 21);
        assert!(android.abis.contains(&AndroidAbi::Arm64V8a));
    }
}
