//! Publishing system for extensions, themes, and design packs
//!
//! Handles validation, packaging, and uploading to the OxideKit marketplace.

use crate::error::{ReleaseError, ReleaseResult};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Publishable artifact types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PublishTarget {
    /// Extension/plugin
    Extension,
    /// Theme
    Theme,
    /// Design pack (icons, assets)
    DesignPack,
    /// UI component pack
    ComponentPack,
    /// Starter template
    Starter,
}

impl std::fmt::Display for PublishTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Extension => write!(f, "extension"),
            Self::Theme => write!(f, "theme"),
            Self::DesignPack => write!(f, "design-pack"),
            Self::ComponentPack => write!(f, "component-pack"),
            Self::Starter => write!(f, "starter"),
        }
    }
}

/// Publishing configuration
#[derive(Debug, Clone)]
pub struct PublishConfig {
    /// Marketplace API URL
    pub api_url: String,
    /// API token
    pub token: Option<String>,
    /// Publisher ID
    pub publisher_id: Option<String>,
    /// Dry run mode
    pub dry_run: bool,
}

impl Default for PublishConfig {
    fn default() -> Self {
        Self {
            api_url: "https://marketplace.oxidekit.com/api".to_string(),
            token: std::env::var("OXIDE_MARKETPLACE_TOKEN").ok(),
            publisher_id: std::env::var("OXIDE_PUBLISHER_ID").ok(),
            dry_run: false,
        }
    }
}

/// Manifest for publishable packages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageManifest {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Display name
    pub display_name: String,
    /// Description
    pub description: String,
    /// Package type
    #[serde(rename = "type")]
    pub package_type: PublishTarget,
    /// Publisher ID
    pub publisher: String,
    /// License
    pub license: String,
    /// Repository URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repository: Option<String>,
    /// Homepage URL
    #[serde(skip_serializing_if = "Option::is_none")]
    pub homepage: Option<String>,
    /// Keywords
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Categories
    #[serde(default)]
    pub categories: Vec<String>,
    /// OxideKit version compatibility
    pub oxide_version: String,
    /// Preview images
    #[serde(default)]
    pub previews: Vec<PreviewAsset>,
    /// Icon path
    #[serde(skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    /// Permissions required (for extensions)
    #[serde(default)]
    pub permissions: Vec<String>,
    /// Dependencies
    #[serde(default)]
    pub dependencies: std::collections::HashMap<String, String>,
}

/// Preview asset for marketplace listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreviewAsset {
    /// Asset type
    #[serde(rename = "type")]
    pub asset_type: PreviewType,
    /// File path or URL
    pub path: String,
    /// Caption
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
}

/// Type of preview asset
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PreviewType {
    /// Screenshot image
    Screenshot,
    /// Video
    Video,
    /// GIF animation
    Gif,
}

/// Validation result for a package
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub valid: bool,
    /// Errors (blocking)
    pub errors: Vec<ValidationError>,
    /// Warnings (non-blocking)
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a valid result
    pub fn valid() -> Self {
        Self {
            valid: true,
            errors: vec![],
            warnings: vec![],
        }
    }

    /// Add an error
    pub fn add_error(&mut self, code: &str, message: impl Into<String>) {
        self.valid = false;
        self.errors.push(ValidationError {
            code: code.to_string(),
            message: message.into(),
        });
    }

    /// Add a warning
    pub fn add_warning(&mut self, code: &str, message: impl Into<String>) {
        self.warnings.push(ValidationWarning {
            code: code.to_string(),
            message: message.into(),
        });
    }
}

/// Validation error
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
}

/// Validation warning
#[derive(Debug, Clone)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
}

/// Validate a package before publishing
pub fn validate_package(manifest_path: &Path) -> ReleaseResult<ValidationResult> {
    let mut result = ValidationResult::valid();

    // Check manifest exists
    if !manifest_path.exists() {
        result.add_error("MANIFEST_NOT_FOUND", "Package manifest not found");
        return Ok(result);
    }

    // Read and parse manifest
    let content = std::fs::read_to_string(manifest_path)?;
    let manifest: PackageManifest = match toml::from_str(&content) {
        Ok(m) => m,
        Err(e) => {
            result.add_error("INVALID_MANIFEST", format!("Failed to parse manifest: {}", e));
            return Ok(result);
        }
    };

    // Validate required fields
    if manifest.name.is_empty() {
        result.add_error("MISSING_NAME", "Package name is required");
    }

    if manifest.version.is_empty() {
        result.add_error("MISSING_VERSION", "Package version is required");
    } else if semver::Version::parse(&manifest.version).is_err() {
        result.add_error("INVALID_VERSION", "Version must be valid semver");
    }

    if manifest.description.is_empty() {
        result.add_error("MISSING_DESCRIPTION", "Package description is required");
    }

    if manifest.publisher.is_empty() {
        result.add_error("MISSING_PUBLISHER", "Publisher ID is required");
    }

    if manifest.oxide_version.is_empty() {
        result.add_error("MISSING_COMPATIBILITY", "OxideKit version compatibility is required");
    }

    // Validate name format
    let name_re = regex::Regex::new(r"^[a-z][a-z0-9-]*$").unwrap();
    if !name_re.is_match(&manifest.name) {
        result.add_error(
            "INVALID_NAME",
            "Package name must be lowercase alphanumeric with hyphens",
        );
    }

    // Validate permissions for extensions
    if manifest.package_type == PublishTarget::Extension {
        for perm in &manifest.permissions {
            if !is_valid_permission(perm) {
                result.add_warning(
                    "UNKNOWN_PERMISSION",
                    format!("Unknown permission: {}", perm),
                );
            }
        }
    }

    // Check for icon
    if manifest.icon.is_none() {
        result.add_warning("MISSING_ICON", "Consider adding an icon for better visibility");
    }

    // Check for previews
    if manifest.previews.is_empty() {
        result.add_warning(
            "MISSING_PREVIEWS",
            "Consider adding preview screenshots for better visibility",
        );
    }

    // Check for keywords
    if manifest.keywords.is_empty() {
        result.add_warning(
            "MISSING_KEYWORDS",
            "Consider adding keywords for better discoverability",
        );
    }

    Ok(result)
}

/// Check if a permission is valid
fn is_valid_permission(perm: &str) -> bool {
    matches!(
        perm,
        "filesystem"
            | "network"
            | "clipboard"
            | "notifications"
            | "system-info"
            | "shell"
            | "window"
            | "storage"
    )
}

/// Package a directory for publishing
pub async fn create_package(
    source_dir: &Path,
    output_path: &Path,
) -> ReleaseResult<PathBuf> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    // Validate manifest exists
    let manifest_path = source_dir.join("oxide-package.toml");
    if !manifest_path.exists() {
        return Err(ReleaseError::ManifestValidation(
            "oxide-package.toml not found".to_string(),
        ));
    }

    // Create tar.gz
    let file = std::fs::File::create(output_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    // Add all files from source directory
    tar.append_dir_all(".", source_dir)?;
    tar.finish()?;

    Ok(output_path.to_path_buf())
}

/// Publish a package to the marketplace
pub async fn publish_package(
    package_path: &Path,
    config: &PublishConfig,
) -> ReleaseResult<PublishResult> {
    // Validate package
    let manifest_path = package_path.join("oxide-package.toml");
    let validation = validate_package(&manifest_path)?;

    if !validation.valid {
        return Err(ReleaseError::ManifestValidation(
            validation.errors.iter()
                .map(|e| format!("{}: {}", e.code, e.message))
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }

    // Check for token
    let token = config.token.as_ref().ok_or_else(|| {
        ReleaseError::publish("OXIDE_MARKETPLACE_TOKEN not set")
    })?;

    if config.dry_run {
        tracing::info!("Dry run - would publish package");
        return Ok(PublishResult {
            success: true,
            package_url: None,
            message: "Dry run successful".to_string(),
        });
    }

    // Create package archive
    let temp_dir = tempfile::tempdir()?;
    let archive_path = temp_dir.path().join("package.tar.gz");
    create_package(package_path, &archive_path).await?;

    // Upload to marketplace
    let upload_url = format!("{}/packages/upload", config.api_url);

    let output = std::process::Command::new("curl")
        .args(["-s", "-X", "POST"])
        .arg("-H").arg(format!("Authorization: Bearer {}", token))
        .arg("-F").arg(format!("package=@{}", archive_path.display()))
        .arg(&upload_url)
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::publish("Upload failed"));
    }

    let response: serde_json::Value = serde_json::from_slice(&output.stdout)?;

    if let Some(url) = response.get("url").and_then(|v| v.as_str()) {
        Ok(PublishResult {
            success: true,
            package_url: Some(url.to_string()),
            message: "Package published successfully".to_string(),
        })
    } else if let Some(error) = response.get("error").and_then(|v| v.as_str()) {
        Err(ReleaseError::publish(error.to_string()))
    } else {
        Err(ReleaseError::publish("Unknown error"))
    }
}

/// Result of publishing
#[derive(Debug, Clone)]
pub struct PublishResult {
    /// Whether publishing succeeded
    pub success: bool,
    /// URL of the published package
    pub package_url: Option<String>,
    /// Status message
    pub message: String,
}

/// Unpublish/yank a package version
pub async fn unpublish_package(
    name: &str,
    version: &str,
    config: &PublishConfig,
) -> ReleaseResult<()> {
    let token = config.token.as_ref().ok_or_else(|| {
        ReleaseError::publish("OXIDE_MARKETPLACE_TOKEN not set")
    })?;

    let url = format!("{}/packages/{}/versions/{}", config.api_url, name, version);

    let output = std::process::Command::new("curl")
        .args(["-s", "-X", "DELETE"])
        .arg("-H").arg(format!("Authorization: Bearer {}", token))
        .arg(&url)
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::publish("Unpublish failed"));
    }

    Ok(())
}

/// Search packages in the marketplace
pub async fn search_packages(
    query: &str,
    config: &PublishConfig,
) -> ReleaseResult<Vec<PackageInfo>> {
    let url = format!("{}/packages/search?q={}", config.api_url, query);

    let output = std::process::Command::new("curl")
        .args(["-s"])
        .arg(&url)
        .output()?;

    if !output.status.success() {
        return Err(ReleaseError::publish("Search failed"));
    }

    let results: Vec<PackageInfo> = serde_json::from_slice(&output.stdout)?;
    Ok(results)
}

/// Package information from marketplace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PackageInfo {
    /// Package name
    pub name: String,
    /// Display name
    pub display_name: String,
    /// Version
    pub version: String,
    /// Description
    pub description: String,
    /// Publisher
    pub publisher: String,
    /// Download count
    pub downloads: u64,
    /// Rating (0-5)
    pub rating: f32,
}

/// Initialize a new package manifest
pub fn init_package_manifest(
    path: &Path,
    package_type: PublishTarget,
    name: &str,
) -> ReleaseResult<()> {
    let manifest = PackageManifest {
        name: name.to_lowercase().replace(' ', "-"),
        version: "0.1.0".to_string(),
        display_name: name.to_string(),
        description: format!("An OxideKit {}", package_type),
        package_type,
        publisher: "your-publisher-id".to_string(),
        license: "MIT".to_string(),
        repository: None,
        homepage: None,
        keywords: vec![],
        categories: vec![],
        oxide_version: ">=0.1.0".to_string(),
        previews: vec![],
        icon: None,
        permissions: vec![],
        dependencies: std::collections::HashMap::new(),
    };

    let content = toml::to_string_pretty(&manifest)?;
    let manifest_path = path.join("oxide-package.toml");
    std::fs::write(&manifest_path, content)?;

    tracing::info!("Created {}", manifest_path.display());
    Ok(())
}
