//! Android build output types.
//!
//! Defines Android-specific build artifacts and configuration.

use std::path::PathBuf;

use camino::Utf8PathBuf;
use serde::{Deserialize, Serialize};

use super::{ArtifactType, BuildArtifact, BuildOutput};
use crate::target::AndroidAbi;

/// Android build output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidBuildOutput {
    /// Common build output info.
    pub common: BuildOutput,
    /// Path to the APK file (for debug or direct install).
    pub apk_path: Option<PathBuf>,
    /// Path to the AAB file (for Play Store distribution).
    pub aab_path: Option<PathBuf>,
    /// Paths to native libraries by ABI.
    pub native_libs: Vec<(AndroidAbi, PathBuf)>,
    /// Path to the ProGuard mapping file (if minification enabled).
    pub proguard_mapping: Option<PathBuf>,
    /// Path to the debug symbols.
    pub debug_symbols: Option<PathBuf>,
    /// Package name.
    pub package_name: String,
    /// Version code.
    pub version_code: u32,
    /// ABIs included in the build.
    pub abis: Vec<AndroidAbi>,
    /// Whether the APK is signed.
    pub is_signed: bool,
    /// Signing key alias used.
    pub signing_key_alias: Option<String>,
    /// Minimum SDK version.
    pub min_sdk: u32,
    /// Target SDK version.
    pub target_sdk: u32,
}

impl AndroidBuildOutput {
    /// Create a new Android build output.
    pub fn new(common: BuildOutput, package_name: impl Into<String>) -> Self {
        Self {
            common,
            apk_path: None,
            aab_path: None,
            native_libs: Vec::new(),
            proguard_mapping: None,
            debug_symbols: None,
            package_name: package_name.into(),
            version_code: 1,
            abis: Vec::new(),
            is_signed: false,
            signing_key_alias: None,
            min_sdk: 24,
            target_sdk: 34,
        }
    }

    /// Set the APK path.
    pub fn with_apk(mut self, path: impl Into<PathBuf>) -> Self {
        self.apk_path = Some(path.into());
        self
    }

    /// Set the AAB path.
    pub fn with_aab(mut self, path: impl Into<PathBuf>) -> Self {
        self.aab_path = Some(path.into());
        self
    }

    /// Add a native library.
    pub fn with_native_lib(mut self, abi: AndroidAbi, path: impl Into<PathBuf>) -> Self {
        self.native_libs.push((abi, path.into()));
        self
    }

    /// Set signing information.
    pub fn with_signing(mut self, key_alias: impl Into<String>) -> Self {
        self.is_signed = true;
        self.signing_key_alias = Some(key_alias.into());
        self
    }

    /// Set SDK versions.
    pub fn with_sdk_versions(mut self, min_sdk: u32, target_sdk: u32) -> Self {
        self.min_sdk = min_sdk;
        self.target_sdk = target_sdk;
        self
    }

    /// Get all build artifacts.
    pub fn artifacts(&self) -> Vec<BuildArtifact> {
        let mut artifacts = Vec::new();

        if let Some(apk) = &self.apk_path {
            artifacts.push(BuildArtifact::new(apk.clone(), ArtifactType::Apk));
        }

        if let Some(aab) = &self.aab_path {
            artifacts.push(BuildArtifact::new(aab.clone(), ArtifactType::Aab));
        }

        if let Some(mapping) = &self.proguard_mapping {
            artifacts.push(BuildArtifact::new(mapping.clone(), ArtifactType::ProguardMapping));
        }

        if let Some(symbols) = &self.debug_symbols {
            artifacts.push(BuildArtifact::new(symbols.clone(), ArtifactType::DebugSymbols));
        }

        artifacts
    }

    /// Get the primary installable artifact (APK).
    pub fn installable_artifact(&self) -> Option<&PathBuf> {
        self.apk_path.as_ref()
    }

    /// Get the distributable artifact (AAB preferred, APK fallback).
    pub fn distributable_artifact(&self) -> Option<&PathBuf> {
        self.aab_path.as_ref().or(self.apk_path.as_ref())
    }

    /// Check if this build is ready for Play Store submission.
    pub fn is_store_ready(&self) -> bool {
        self.aab_path.is_some() && self.is_signed && self.common.is_release()
    }

    /// Get the ADB install command.
    pub fn adb_install_command(&self) -> Option<String> {
        self.apk_path.as_ref().map(|apk| {
            format!("adb install -r {}", apk.display())
        })
    }
}

/// Android build configuration for Gradle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradleBuildConfig {
    /// Gradle wrapper path.
    pub gradle_wrapper: Utf8PathBuf,
    /// Build type (debug/release).
    pub build_type: String,
    /// Product flavor (if any).
    pub product_flavor: Option<String>,
    /// Additional Gradle properties.
    pub properties: Vec<(String, String)>,
    /// Gradle tasks to execute.
    pub tasks: Vec<String>,
    /// Whether to run in offline mode.
    pub offline: bool,
    /// Number of parallel workers.
    pub parallel_workers: Option<u32>,
    /// Gradle daemon memory (e.g., "2g").
    pub daemon_memory: Option<String>,
}

impl Default for GradleBuildConfig {
    fn default() -> Self {
        Self {
            gradle_wrapper: Utf8PathBuf::from("./gradlew"),
            build_type: "debug".into(),
            product_flavor: None,
            properties: Vec::new(),
            tasks: vec!["assembleDebug".into()],
            offline: false,
            parallel_workers: None,
            daemon_memory: Some("2g".into()),
        }
    }
}

impl GradleBuildConfig {
    /// Create configuration for a debug build.
    pub fn debug() -> Self {
        Self::default()
    }

    /// Create configuration for a release build.
    pub fn release() -> Self {
        Self {
            build_type: "release".into(),
            tasks: vec!["assembleRelease".into()],
            ..Default::default()
        }
    }

    /// Create configuration for building an AAB.
    pub fn bundle_release() -> Self {
        Self {
            build_type: "release".into(),
            tasks: vec!["bundleRelease".into()],
            ..Default::default()
        }
    }

    /// Set product flavor.
    pub fn with_flavor(mut self, flavor: impl Into<String>) -> Self {
        self.product_flavor = Some(flavor.into());
        self
    }

    /// Add a Gradle property.
    pub fn with_property(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.properties.push((key.into(), value.into()));
        self
    }

    /// Build the Gradle command arguments.
    pub fn build_args(&self) -> Vec<String> {
        let mut args = Vec::new();

        // Add tasks
        args.extend(self.tasks.clone());

        // Add properties
        for (key, value) in &self.properties {
            args.push(format!("-P{}={}", key, value));
        }

        // Add flags
        if self.offline {
            args.push("--offline".into());
        }

        if let Some(workers) = self.parallel_workers {
            args.push(format!("--parallel"));
            args.push(format!("-Dorg.gradle.workers.max={}", workers));
        }

        if let Some(memory) = &self.daemon_memory {
            args.push(format!("-Dorg.gradle.jvmargs=-Xmx{}", memory));
        }

        args
    }
}

/// Android keystore configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeystoreConfig {
    /// Path to the keystore file.
    pub path: PathBuf,
    /// Keystore password.
    #[serde(skip_serializing)]
    pub store_password: String,
    /// Key alias.
    pub key_alias: String,
    /// Key password.
    #[serde(skip_serializing)]
    pub key_password: String,
}

impl KeystoreConfig {
    /// Create a new keystore configuration.
    pub fn new(
        path: impl Into<PathBuf>,
        store_password: impl Into<String>,
        key_alias: impl Into<String>,
        key_password: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            store_password: store_password.into(),
            key_alias: key_alias.into(),
            key_password: key_password.into(),
        }
    }

    /// Create configuration for the debug keystore.
    pub fn debug_keystore() -> Self {
        // Default Android debug keystore
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self {
            path: PathBuf::from(home).join(".android/debug.keystore"),
            store_password: "android".into(),
            key_alias: "androiddebugkey".into(),
            key_password: "android".into(),
        }
    }

    /// Get Gradle signing properties.
    pub fn gradle_properties(&self) -> Vec<(String, String)> {
        vec![
            ("RELEASE_STORE_FILE".into(), self.path.display().to_string()),
            ("RELEASE_STORE_PASSWORD".into(), self.store_password.clone()),
            ("RELEASE_KEY_ALIAS".into(), self.key_alias.clone()),
            ("RELEASE_KEY_PASSWORD".into(), self.key_password.clone()),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::BuildProfile;
    use crate::target::MobileTarget;

    #[test]
    fn test_android_build_output() {
        let common = BuildOutput::new(
            MobileTarget::AndroidDevice,
            BuildProfile::Release,
            "MyApp",
            "1.0.0",
            1,
            "/tmp/build",
        );

        let output = AndroidBuildOutput::new(common, "com.example.myapp")
            .with_apk("/tmp/build/app-release.apk")
            .with_aab("/tmp/build/app-release.aab")
            .with_signing("release-key")
            .with_sdk_versions(24, 34);

        assert!(output.is_signed);
        assert!(output.is_store_ready());
        assert_eq!(output.artifacts().len(), 2);
    }

    #[test]
    fn test_gradle_build_config() {
        let config = GradleBuildConfig::release()
            .with_flavor("production")
            .with_property("versionCode", "42");

        assert_eq!(config.build_type, "release");
        assert_eq!(config.product_flavor, Some("production".into()));

        let args = config.build_args();
        assert!(args.contains(&"assembleRelease".into()));
        assert!(args.iter().any(|a| a.contains("versionCode")));
    }

    #[test]
    fn test_keystore_config() {
        let debug = KeystoreConfig::debug_keystore();
        assert_eq!(debug.store_password, "android");
        assert_eq!(debug.key_alias, "androiddebugkey");

        let props = debug.gradle_properties();
        assert_eq!(props.len(), 4);
    }
}
