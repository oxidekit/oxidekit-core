//! Release configuration types and parsing

use crate::error::{ReleaseError, ReleaseResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Main release configuration loaded from oxide.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseConfig {
    /// Application name
    pub app_name: String,

    /// Application version
    pub version: String,

    /// Application identifier (reverse domain notation)
    pub app_id: String,

    /// Project root directory
    #[serde(skip)]
    pub project_root: PathBuf,

    /// Build configuration
    #[serde(default)]
    pub build: BuildConfig,

    /// Signing configuration
    #[serde(default)]
    pub signing: Option<SigningConfig>,

    /// Notarization configuration (macOS)
    #[serde(default)]
    pub notarization: Option<NotarizationConfig>,

    /// Packaging configuration
    #[serde(default)]
    pub packaging: Option<PackagingConfig>,

    /// GitHub release configuration
    #[serde(default)]
    pub github: Option<GitHubConfig>,

    /// Publishing configuration
    #[serde(default)]
    pub publish: Option<PublishConfig>,

    /// Update configuration
    #[serde(default)]
    pub update: Option<UpdateConfig>,
}

impl ReleaseConfig {
    /// Load configuration from a project directory
    pub fn from_project(path: impl AsRef<Path>) -> ReleaseResult<Self> {
        let project_root = path.as_ref().to_path_buf();
        let config_path = project_root.join("oxide.toml");

        if !config_path.exists() {
            return Err(ReleaseError::Config(format!(
                "oxide.toml not found in {}",
                project_root.display()
            )));
        }

        let content = std::fs::read_to_string(&config_path)?;
        let oxide_toml: OxideToml = toml::from_str(&content)?;

        let mut config = Self {
            app_name: oxide_toml.app.name,
            version: oxide_toml.app.version,
            app_id: oxide_toml
                .app
                .id
                .unwrap_or_else(|| format!("com.example.{}", oxide_toml.app.name)),
            project_root,
            build: oxide_toml.build.unwrap_or_default(),
            signing: oxide_toml.release.as_ref().and_then(|r| r.signing.clone()),
            notarization: oxide_toml
                .release
                .as_ref()
                .and_then(|r| r.notarization.clone()),
            packaging: oxide_toml
                .release
                .as_ref()
                .and_then(|r| r.packaging.clone()),
            github: oxide_toml.release.as_ref().and_then(|r| r.github.clone()),
            publish: oxide_toml.release.as_ref().and_then(|r| r.publish.clone()),
            update: oxide_toml.release.as_ref().and_then(|r| r.update.clone()),
        };

        // Apply environment variable overrides
        config.apply_env_overrides()?;

        Ok(config)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> ReleaseResult<()> {
        // Signing overrides
        if let Some(ref mut signing) = self.signing {
            if let Ok(val) = std::env::var("OXIDE_SIGNING_IDENTITY") {
                signing.identity = Some(val);
            }
            if let Ok(val) = std::env::var("OXIDE_SIGNING_CERTIFICATE") {
                signing.certificate_path = Some(PathBuf::from(val));
            }
            if let Ok(val) = std::env::var("OXIDE_SIGNING_PASSWORD") {
                signing.certificate_password = Some(val);
            }
        }

        // Notarization overrides
        if let Some(ref mut notarization) = self.notarization {
            if let Ok(val) = std::env::var("OXIDE_APPLE_TEAM_ID") {
                notarization.team_id = val;
            }
            if let Ok(val) = std::env::var("OXIDE_APPLE_ID") {
                notarization.apple_id = Some(val);
            }
            if let Ok(val) = std::env::var("OXIDE_APPLE_PASSWORD") {
                notarization.password = Some(val);
            }
            if let Ok(val) = std::env::var("OXIDE_APPLE_KEY_ID") {
                notarization.api_key_id = Some(val);
            }
            if let Ok(val) = std::env::var("OXIDE_APPLE_ISSUER_ID") {
                notarization.api_issuer_id = Some(val);
            }
        }

        // GitHub overrides
        if let Some(ref mut github) = self.github {
            if let Ok(val) = std::env::var("GITHUB_TOKEN") {
                github.token = Some(val);
            }
            if let Ok(val) = std::env::var("GITHUB_REPOSITORY") {
                if let Some((owner, repo)) = val.split_once('/') {
                    github.owner = owner.to_string();
                    github.repo = repo.to_string();
                }
            }
        }

        Ok(())
    }

    /// Get the output directory for release artifacts
    pub fn output_dir(&self) -> PathBuf {
        self.project_root.join("target").join("release-artifacts")
    }

    /// Get the build output directory
    pub fn build_output_dir(&self, target: &str) -> PathBuf {
        self.project_root
            .join("target")
            .join(target)
            .join("release")
    }
}

/// oxide.toml file structure
#[derive(Debug, Deserialize)]
struct OxideToml {
    app: AppSection,
    #[serde(default)]
    build: Option<BuildConfig>,
    #[serde(default)]
    release: Option<ReleaseSection>,
}

/// App section of oxide.toml
#[derive(Debug, Deserialize)]
struct AppSection {
    name: String,
    version: String,
    #[serde(default)]
    id: Option<String>,
}

/// Release section of oxide.toml
#[derive(Debug, Clone, Deserialize)]
struct ReleaseSection {
    #[serde(default)]
    signing: Option<SigningConfig>,
    #[serde(default)]
    notarization: Option<NotarizationConfig>,
    #[serde(default)]
    packaging: Option<PackagingConfig>,
    #[serde(default)]
    github: Option<GitHubConfig>,
    #[serde(default)]
    publish: Option<PublishConfig>,
    #[serde(default)]
    update: Option<UpdateConfig>,
}

/// Build configuration
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildConfig {
    /// Default build targets
    #[serde(default)]
    pub targets: Vec<String>,

    /// Additional cargo features to enable
    #[serde(default)]
    pub features: Vec<String>,

    /// Build profile (release, release-lto, etc.)
    #[serde(default = "default_profile")]
    pub profile: String,

    /// Strip debug symbols
    #[serde(default = "default_true")]
    pub strip: bool,

    /// Enable LTO
    #[serde(default)]
    pub lto: bool,

    /// Assets to bundle
    #[serde(default)]
    pub assets: Vec<AssetConfig>,

    /// Environment variables for build
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

fn default_profile() -> String {
    "release".to_string()
}

fn default_true() -> bool {
    true
}

/// Asset bundling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetConfig {
    /// Source path (relative to project root)
    pub src: String,

    /// Destination path in bundle
    pub dest: String,

    /// Glob patterns to include
    #[serde(default)]
    pub include: Vec<String>,

    /// Glob patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,
}

/// Code signing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Signing identity (macOS: certificate name, Windows: certificate thumbprint)
    #[serde(default)]
    pub identity: Option<String>,

    /// Path to signing certificate (Windows: .pfx file)
    #[serde(default)]
    pub certificate_path: Option<PathBuf>,

    /// Certificate password (Windows)
    #[serde(default)]
    pub certificate_password: Option<String>,

    /// Timestamp server URL
    #[serde(default)]
    pub timestamp_url: Option<String>,

    /// Entitlements file (macOS)
    #[serde(default)]
    pub entitlements: Option<PathBuf>,

    /// Enable hardened runtime (macOS)
    #[serde(default = "default_true")]
    pub hardened_runtime: bool,

    /// Additional signing flags
    #[serde(default)]
    pub flags: Vec<String>,
}

/// macOS notarization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotarizationConfig {
    /// Apple Developer Team ID
    pub team_id: String,

    /// Apple ID email (for password-based auth)
    #[serde(default)]
    pub apple_id: Option<String>,

    /// App-specific password (for password-based auth)
    #[serde(default)]
    pub password: Option<String>,

    /// API Key ID (for key-based auth)
    #[serde(default)]
    pub api_key_id: Option<String>,

    /// API Issuer ID (for key-based auth)
    #[serde(default)]
    pub api_issuer_id: Option<String>,

    /// Path to API key file (.p8)
    #[serde(default)]
    pub api_key_path: Option<PathBuf>,

    /// Timeout in seconds for notarization
    #[serde(default = "default_notarization_timeout")]
    pub timeout: u64,

    /// Staple the notarization ticket
    #[serde(default = "default_true")]
    pub staple: bool,
}

fn default_notarization_timeout() -> u64 {
    3600 // 1 hour
}

/// Packaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackagingConfig {
    /// macOS packaging options
    #[serde(default)]
    pub macos: Option<MacOSPackagingConfig>,

    /// Windows packaging options
    #[serde(default)]
    pub windows: Option<WindowsPackagingConfig>,

    /// Linux packaging options
    #[serde(default)]
    pub linux: Option<LinuxPackagingConfig>,
}

/// macOS-specific packaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacOSPackagingConfig {
    /// Create DMG installer
    #[serde(default = "default_true")]
    pub dmg: bool,

    /// Create PKG installer
    #[serde(default)]
    pub pkg: bool,

    /// DMG background image
    #[serde(default)]
    pub dmg_background: Option<PathBuf>,

    /// DMG icon positions
    #[serde(default)]
    pub dmg_icon_size: u32,

    /// App icon path
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// App category
    #[serde(default)]
    pub category: Option<String>,

    /// Minimum macOS version
    #[serde(default)]
    pub minimum_system_version: Option<String>,

    /// Copyright notice
    #[serde(default)]
    pub copyright: Option<String>,

    /// Frameworks to bundle
    #[serde(default)]
    pub frameworks: Vec<PathBuf>,
}

/// Windows-specific packaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowsPackagingConfig {
    /// Create MSI installer
    #[serde(default = "default_true")]
    pub msi: bool,

    /// Create MSIX package
    #[serde(default)]
    pub msix: bool,

    /// WiX toolset template
    #[serde(default)]
    pub wix_template: Option<PathBuf>,

    /// App icon path (.ico)
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Publisher name
    #[serde(default)]
    pub publisher: Option<String>,

    /// Install directory name
    #[serde(default)]
    pub install_dir: Option<String>,

    /// Add to PATH
    #[serde(default)]
    pub add_to_path: bool,

    /// Create desktop shortcut
    #[serde(default = "default_true")]
    pub desktop_shortcut: bool,

    /// Create start menu entry
    #[serde(default = "default_true")]
    pub start_menu: bool,

    /// File associations
    #[serde(default)]
    pub file_associations: Vec<FileAssociation>,
}

/// Linux-specific packaging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinuxPackagingConfig {
    /// Create AppImage
    #[serde(default = "default_true")]
    pub appimage: bool,

    /// Create DEB package
    #[serde(default)]
    pub deb: bool,

    /// Create RPM package
    #[serde(default)]
    pub rpm: bool,

    /// App icon path
    #[serde(default)]
    pub icon: Option<PathBuf>,

    /// Desktop entry categories
    #[serde(default)]
    pub categories: Vec<String>,

    /// Desktop entry keywords
    #[serde(default)]
    pub keywords: Vec<String>,

    /// Package dependencies (deb)
    #[serde(default)]
    pub deb_depends: Vec<String>,

    /// Package dependencies (rpm)
    #[serde(default)]
    pub rpm_requires: Vec<String>,

    /// AppImage runtime arch
    #[serde(default)]
    pub appimage_arch: Option<String>,
}

/// File association configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileAssociation {
    /// File extension (without dot)
    pub ext: String,

    /// MIME type
    #[serde(default)]
    pub mime_type: Option<String>,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Icon path
    #[serde(default)]
    pub icon: Option<PathBuf>,
}

/// GitHub release configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubConfig {
    /// Repository owner
    pub owner: String,

    /// Repository name
    pub repo: String,

    /// GitHub token (usually from env)
    #[serde(default)]
    pub token: Option<String>,

    /// Create draft release
    #[serde(default)]
    pub draft: bool,

    /// Mark as prerelease for non-stable channels
    #[serde(default = "default_true")]
    pub prerelease_for_channels: bool,

    /// Generate release notes automatically
    #[serde(default = "default_true")]
    pub generate_release_notes: bool,

    /// Release name template
    #[serde(default)]
    pub name_template: Option<String>,

    /// Release body template
    #[serde(default)]
    pub body_template: Option<String>,
}

/// Publishing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishConfig {
    /// OxideKit marketplace configuration
    #[serde(default)]
    pub marketplace: Option<MarketplaceConfig>,

    /// crates.io configuration
    #[serde(default)]
    pub crates_io: Option<CratesIoConfig>,
}

/// OxideKit marketplace configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceConfig {
    /// Marketplace API endpoint
    #[serde(default = "default_marketplace_url")]
    pub url: String,

    /// API token (usually from env)
    #[serde(default)]
    pub token: Option<String>,

    /// Publisher ID
    #[serde(default)]
    pub publisher_id: Option<String>,
}

fn default_marketplace_url() -> String {
    "https://marketplace.oxidekit.com/api".to_string()
}

/// crates.io configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CratesIoConfig {
    /// Publish to crates.io
    #[serde(default)]
    pub enabled: bool,

    /// API token (usually from env)
    #[serde(default)]
    pub token: Option<String>,
}

/// Update/auto-updater configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    /// Update server URL
    pub url: String,

    /// Public key for signature verification
    #[serde(default)]
    pub public_key: Option<String>,

    /// Private key path for signing updates
    #[serde(default)]
    pub private_key_path: Option<PathBuf>,

    /// Supported update channels
    #[serde(default)]
    pub channels: Vec<String>,

    /// Enable anti-downgrade protection
    #[serde(default)]
    pub anti_downgrade: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_build_config() {
        let config = BuildConfig::default();
        assert!(config.targets.is_empty());
        assert!(config.features.is_empty());
        assert_eq!(config.profile, "release");
        assert!(config.strip);
    }

    #[test]
    fn test_signing_config_deserialization() {
        let toml = r#"
            identity = "Developer ID Application: Example Inc"
            hardened_runtime = true
        "#;

        let config: SigningConfig = toml::from_str(toml).unwrap();
        assert_eq!(
            config.identity,
            Some("Developer ID Application: Example Inc".to_string())
        );
        assert!(config.hardened_runtime);
    }
}
