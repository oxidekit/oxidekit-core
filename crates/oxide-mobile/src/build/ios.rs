//! iOS build output types.
//!
//! Defines iOS-specific build artifacts and configuration.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::{ArtifactType, BuildArtifact, BuildOutput};
use crate::target::IosArch;

/// iOS build output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosBuildOutput {
    /// Common build output info.
    pub common: BuildOutput,
    /// Path to the .app bundle.
    pub app_path: Option<PathBuf>,
    /// Path to the IPA file.
    pub ipa_path: Option<PathBuf>,
    /// Path to the dSYM debug symbols.
    pub dsym_path: Option<PathBuf>,
    /// Path to the xcarchive.
    pub archive_path: Option<PathBuf>,
    /// Bundle identifier.
    pub bundle_id: String,
    /// Bundle version.
    pub bundle_version: String,
    /// Architectures included.
    pub architectures: Vec<IosArch>,
    /// Minimum deployment target.
    pub deployment_target: String,
    /// Whether the app is signed.
    pub is_signed: bool,
    /// Signing identity used.
    pub signing_identity: Option<String>,
    /// Provisioning profile name.
    pub provisioning_profile: Option<String>,
    /// Team ID.
    pub team_id: Option<String>,
    /// Whether bitcode is enabled.
    pub bitcode_enabled: bool,
}

impl IosBuildOutput {
    /// Create a new iOS build output.
    pub fn new(common: BuildOutput, bundle_id: impl Into<String>) -> Self {
        Self {
            common,
            app_path: None,
            ipa_path: None,
            dsym_path: None,
            archive_path: None,
            bundle_id: bundle_id.into(),
            bundle_version: "1".into(),
            architectures: Vec::new(),
            deployment_target: "15.0".into(),
            is_signed: false,
            signing_identity: None,
            provisioning_profile: None,
            team_id: None,
            bitcode_enabled: false,
        }
    }

    /// Set the .app bundle path.
    pub fn with_app(mut self, path: impl Into<PathBuf>) -> Self {
        self.app_path = Some(path.into());
        self
    }

    /// Set the IPA path.
    pub fn with_ipa(mut self, path: impl Into<PathBuf>) -> Self {
        self.ipa_path = Some(path.into());
        self
    }

    /// Set the dSYM path.
    pub fn with_dsym(mut self, path: impl Into<PathBuf>) -> Self {
        self.dsym_path = Some(path.into());
        self
    }

    /// Set the archive path.
    pub fn with_archive(mut self, path: impl Into<PathBuf>) -> Self {
        self.archive_path = Some(path.into());
        self
    }

    /// Set signing information.
    pub fn with_signing(
        mut self,
        identity: impl Into<String>,
        profile: impl Into<String>,
        team_id: impl Into<String>,
    ) -> Self {
        self.is_signed = true;
        self.signing_identity = Some(identity.into());
        self.provisioning_profile = Some(profile.into());
        self.team_id = Some(team_id.into());
        self
    }

    /// Set deployment target.
    pub fn with_deployment_target(mut self, target: impl Into<String>) -> Self {
        self.deployment_target = target.into();
        self
    }

    /// Set architectures.
    pub fn with_architectures(mut self, archs: Vec<IosArch>) -> Self {
        self.architectures = archs;
        self
    }

    /// Get all build artifacts.
    pub fn artifacts(&self) -> Vec<BuildArtifact> {
        let mut artifacts = Vec::new();

        if let Some(app) = &self.app_path {
            artifacts.push(BuildArtifact::new(app.clone(), ArtifactType::App));
        }

        if let Some(ipa) = &self.ipa_path {
            artifacts.push(BuildArtifact::new(ipa.clone(), ArtifactType::Ipa));
        }

        if let Some(dsym) = &self.dsym_path {
            artifacts.push(BuildArtifact::new(dsym.clone(), ArtifactType::Dsym));
        }

        artifacts
    }

    /// Get the primary installable artifact.
    pub fn installable_artifact(&self) -> Option<&PathBuf> {
        // Prefer IPA for devices, .app for simulators
        self.ipa_path.as_ref().or(self.app_path.as_ref())
    }

    /// Get the distributable artifact (IPA).
    pub fn distributable_artifact(&self) -> Option<&PathBuf> {
        self.ipa_path.as_ref()
    }

    /// Check if this build is ready for App Store submission.
    pub fn is_store_ready(&self) -> bool {
        self.ipa_path.is_some()
            && self.is_signed
            && self.common.is_release()
            && self.team_id.is_some()
    }

    /// Check if this build is for the simulator.
    pub fn is_simulator_build(&self) -> bool {
        self.architectures.iter().any(|a| a.is_simulator())
    }

    /// Get the simctl install command.
    pub fn simctl_install_command(&self, device_id: &str) -> Option<String> {
        self.app_path.as_ref().map(|app| {
            format!("xcrun simctl install {} {}", device_id, app.display())
        })
    }

    /// Get the ios-deploy install command.
    pub fn ios_deploy_command(&self) -> Option<String> {
        self.app_path.as_ref().map(|app| {
            format!("ios-deploy --bundle {}", app.display())
        })
    }
}

/// Xcode build configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XcodeBuildConfig {
    /// Project or workspace path.
    pub project_path: PathBuf,
    /// Scheme to build.
    pub scheme: String,
    /// Configuration (Debug/Release).
    pub configuration: String,
    /// SDK to use (iphoneos, iphonesimulator).
    pub sdk: String,
    /// Destination specifier.
    pub destination: Option<String>,
    /// Whether to use workspace.
    pub use_workspace: bool,
    /// Derived data path.
    pub derived_data_path: Option<PathBuf>,
    /// Archive path (for archiving).
    pub archive_path: Option<PathBuf>,
    /// Export options plist path.
    pub export_options_plist: Option<PathBuf>,
    /// Additional build settings.
    pub build_settings: Vec<(String, String)>,
    /// Whether to enable code signing.
    pub code_sign: bool,
}

impl Default for XcodeBuildConfig {
    fn default() -> Self {
        Self {
            project_path: PathBuf::from("App.xcodeproj"),
            scheme: "App".into(),
            configuration: "Debug".into(),
            sdk: "iphonesimulator".into(),
            destination: None,
            use_workspace: false,
            derived_data_path: None,
            archive_path: None,
            export_options_plist: None,
            build_settings: Vec::new(),
            code_sign: false,
        }
    }
}

impl XcodeBuildConfig {
    /// Create configuration for a simulator debug build.
    pub fn simulator_debug(scheme: impl Into<String>) -> Self {
        Self {
            scheme: scheme.into(),
            configuration: "Debug".into(),
            sdk: "iphonesimulator".into(),
            code_sign: false,
            ..Default::default()
        }
    }

    /// Create configuration for a device debug build.
    pub fn device_debug(scheme: impl Into<String>) -> Self {
        Self {
            scheme: scheme.into(),
            configuration: "Debug".into(),
            sdk: "iphoneos".into(),
            code_sign: true,
            ..Default::default()
        }
    }

    /// Create configuration for a release archive.
    pub fn release_archive(scheme: impl Into<String>) -> Self {
        Self {
            scheme: scheme.into(),
            configuration: "Release".into(),
            sdk: "iphoneos".into(),
            code_sign: true,
            ..Default::default()
        }
    }

    /// Set the project path.
    pub fn with_project(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_path = path.into();
        self.use_workspace = false;
        self
    }

    /// Set the workspace path.
    pub fn with_workspace(mut self, path: impl Into<PathBuf>) -> Self {
        self.project_path = path.into();
        self.use_workspace = true;
        self
    }

    /// Set a destination.
    pub fn with_destination(mut self, destination: impl Into<String>) -> Self {
        self.destination = Some(destination.into());
        self
    }

    /// Set derived data path.
    pub fn with_derived_data(mut self, path: impl Into<PathBuf>) -> Self {
        self.derived_data_path = Some(path.into());
        self
    }

    /// Set archive path.
    pub fn with_archive_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.archive_path = Some(path.into());
        self
    }

    /// Add a build setting.
    pub fn with_build_setting(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.build_settings.push((key.into(), value.into()));
        self
    }

    /// Build the xcodebuild command arguments.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Project or workspace
        if self.use_workspace {
            args.push("-workspace".into());
        } else {
            args.push("-project".into());
        }
        args.push(self.project_path.display().to_string());

        // Scheme
        args.push("-scheme".into());
        args.push(self.scheme.clone());

        // Configuration
        args.push("-configuration".into());
        args.push(self.configuration.clone());

        // SDK
        args.push("-sdk".into());
        args.push(self.sdk.clone());

        // Destination
        if let Some(dest) = &self.destination {
            args.push("-destination".into());
            args.push(dest.clone());
        }

        // Derived data
        if let Some(dd) = &self.derived_data_path {
            args.push("-derivedDataPath".into());
            args.push(dd.display().to_string());
        }

        // Build settings
        for (key, value) in &self.build_settings {
            args.push(format!("{}={}", key, value));
        }

        // Code signing
        if !self.code_sign {
            args.push("CODE_SIGN_IDENTITY=".into());
            args.push("CODE_SIGNING_REQUIRED=NO".into());
            args.push("CODE_SIGNING_ALLOWED=NO".into());
        }

        args
    }

    /// Build arguments for archiving.
    pub fn archive_args(&self) -> Vec<String> {
        let mut args = self.build_args();
        args.push("archive".into());

        if let Some(path) = &self.archive_path {
            args.push("-archivePath".into());
            args.push(path.display().to_string());
        }

        args
    }
}

/// Export options for IPA creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Export method (app-store, ad-hoc, enterprise, development).
    pub method: ExportMethod,
    /// Team ID.
    pub team_id: String,
    /// Signing style (automatic or manual).
    pub signing_style: SigningStyle,
    /// Provisioning profiles by bundle ID.
    pub provisioning_profiles: Vec<(String, String)>,
    /// Strip Swift symbols.
    pub strip_swift_symbols: bool,
    /// Upload to App Store Connect.
    pub upload_symbols: bool,
    /// Compile bitcode.
    pub compile_bitcode: bool,
}

impl ExportOptions {
    /// Create export options for App Store distribution.
    pub fn app_store(team_id: impl Into<String>) -> Self {
        Self {
            method: ExportMethod::AppStore,
            team_id: team_id.into(),
            signing_style: SigningStyle::Automatic,
            provisioning_profiles: Vec::new(),
            strip_swift_symbols: true,
            upload_symbols: true,
            compile_bitcode: false,
        }
    }

    /// Create export options for ad-hoc distribution.
    pub fn ad_hoc(team_id: impl Into<String>) -> Self {
        Self {
            method: ExportMethod::AdHoc,
            team_id: team_id.into(),
            signing_style: SigningStyle::Automatic,
            provisioning_profiles: Vec::new(),
            strip_swift_symbols: true,
            upload_symbols: false,
            compile_bitcode: false,
        }
    }

    /// Create export options for development.
    pub fn development(team_id: impl Into<String>) -> Self {
        Self {
            method: ExportMethod::Development,
            team_id: team_id.into(),
            signing_style: SigningStyle::Automatic,
            provisioning_profiles: Vec::new(),
            strip_swift_symbols: false,
            upload_symbols: false,
            compile_bitcode: false,
        }
    }

    /// Add a provisioning profile mapping.
    pub fn with_provisioning_profile(
        mut self,
        bundle_id: impl Into<String>,
        profile: impl Into<String>,
    ) -> Self {
        self.provisioning_profiles
            .push((bundle_id.into(), profile.into()));
        self
    }

    /// Set signing style to manual.
    pub fn with_manual_signing(mut self) -> Self {
        self.signing_style = SigningStyle::Manual;
        self
    }
}

/// IPA export method.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportMethod {
    /// App Store distribution.
    AppStore,
    /// Ad-hoc distribution.
    AdHoc,
    /// Enterprise distribution.
    Enterprise,
    /// Development distribution.
    Development,
}

impl ExportMethod {
    /// Get the plist value for this method.
    pub fn plist_value(&self) -> &'static str {
        match self {
            ExportMethod::AppStore => "app-store",
            ExportMethod::AdHoc => "ad-hoc",
            ExportMethod::Enterprise => "enterprise",
            ExportMethod::Development => "development",
        }
    }
}

/// Code signing style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
pub enum SigningStyle {
    /// Automatic signing managed by Xcode.
    #[default]
    Automatic,
    /// Manual signing with explicit profiles.
    Manual,
}

impl SigningStyle {
    /// Get the plist value for this style.
    pub fn plist_value(&self) -> &'static str {
        match self {
            SigningStyle::Automatic => "automatic",
            SigningStyle::Manual => "manual",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BuildProfile;
    use crate::target::MobileTarget;

    #[test]
    fn test_ios_build_output() {
        let common = BuildOutput::new(
            MobileTarget::IosDevice,
            BuildProfile::Release,
            "MyApp",
            "1.0.0",
            1,
            "/tmp/build",
        );

        let output = IosBuildOutput::new(common, "com.example.myapp")
            .with_app("/tmp/build/MyApp.app")
            .with_ipa("/tmp/build/MyApp.ipa")
            .with_dsym("/tmp/build/MyApp.app.dSYM")
            .with_signing("Apple Distribution", "MyApp Distribution", "TEAM123");

        assert!(output.is_signed);
        assert!(output.is_store_ready());
        assert_eq!(output.artifacts().len(), 3);
    }

    #[test]
    fn test_xcode_build_config() {
        let config = XcodeBuildConfig::release_archive("MyApp")
            .with_workspace("/path/to/MyApp.xcworkspace")
            .with_build_setting("PRODUCT_BUNDLE_IDENTIFIER", "com.example.myapp");

        assert_eq!(config.configuration, "Release");
        assert!(config.use_workspace);

        let args = config.build_args();
        assert!(args.contains(&"-workspace".into()));
        assert!(args.contains(&"MyApp".into()));
    }

    #[test]
    fn test_export_options() {
        let options = ExportOptions::app_store("TEAM123")
            .with_provisioning_profile("com.example.myapp", "MyApp Distribution Profile");

        assert!(matches!(options.method, ExportMethod::AppStore));
        assert_eq!(options.team_id, "TEAM123");
        assert_eq!(options.provisioning_profiles.len(), 1);
    }

    #[test]
    fn test_export_method_values() {
        assert_eq!(ExportMethod::AppStore.plist_value(), "app-store");
        assert_eq!(ExportMethod::AdHoc.plist_value(), "ad-hoc");
        assert_eq!(ExportMethod::Development.plist_value(), "development");
    }
}
