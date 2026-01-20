//! Cross-platform packaging for OxideKit applications
//!
//! Creates platform-specific installers and packages:
//! - macOS: DMG, PKG
//! - Windows: MSI, MSIX
//! - Linux: AppImage, DEB, RPM

mod macos;
mod windows;
mod linux;

use crate::artifact::Artifact;
use crate::config::ReleaseConfig;
use crate::error::{ReleaseError, ReleaseResult};
use crate::TargetPlatform;
use std::path::PathBuf;

pub use macos::*;
pub use windows::*;
pub use linux::*;

/// Package format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageFormat {
    /// macOS DMG disk image
    Dmg,
    /// macOS PKG installer
    Pkg,
    /// Windows MSI installer
    Msi,
    /// Windows MSIX package
    Msix,
    /// Linux AppImage
    AppImage,
    /// Debian package
    Deb,
    /// RPM package
    Rpm,
    /// ZIP archive
    Zip,
    /// Tar.gz archive
    TarGz,
}

impl PackageFormat {
    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            Self::Dmg => "dmg",
            Self::Pkg => "pkg",
            Self::Msi => "msi",
            Self::Msix => "msix",
            Self::AppImage => "AppImage",
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::Zip => "zip",
            Self::TarGz => "tar.gz",
        }
    }

    /// Get the default formats for a platform
    pub fn defaults_for_platform(platform: TargetPlatform) -> Vec<Self> {
        if platform.is_macos() {
            vec![Self::Dmg]
        } else if platform.is_windows() {
            vec![Self::Msi]
        } else if platform.is_linux() {
            vec![Self::AppImage]
        } else {
            vec![Self::Zip]
        }
    }

    /// Check if format is supported on the current platform
    pub fn is_supported_on_current(&self) -> bool {
        match self {
            Self::Dmg | Self::Pkg => cfg!(target_os = "macos"),
            Self::Msi | Self::Msix => cfg!(target_os = "windows"),
            Self::AppImage | Self::Deb | Self::Rpm => cfg!(target_os = "linux"),
            Self::Zip | Self::TarGz => true,
        }
    }
}

impl std::fmt::Display for PackageFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dmg => write!(f, "DMG"),
            Self::Pkg => write!(f, "PKG"),
            Self::Msi => write!(f, "MSI"),
            Self::Msix => write!(f, "MSIX"),
            Self::AppImage => write!(f, "AppImage"),
            Self::Deb => write!(f, "DEB"),
            Self::Rpm => write!(f, "RPM"),
            Self::Zip => write!(f, "ZIP"),
            Self::TarGz => write!(f, "tar.gz"),
        }
    }
}

/// Packaging configuration
#[derive(Debug, Clone)]
pub struct PackagingConfig {
    /// Output directory for packages
    pub output_dir: PathBuf,
    /// Formats to create
    pub formats: Vec<PackageFormat>,
    /// App name
    pub app_name: String,
    /// App version
    pub version: String,
    /// App identifier
    pub app_id: String,
    /// Icon path
    pub icon: Option<PathBuf>,
    /// Description
    pub description: Option<String>,
    /// Copyright notice
    pub copyright: Option<String>,
    /// Publisher/vendor name
    pub publisher: Option<String>,
}

impl PackagingConfig {
    /// Create from release config
    pub fn from_release_config(config: &ReleaseConfig) -> Self {
        Self {
            output_dir: config.output_dir(),
            formats: vec![],
            app_name: config.app_name.clone(),
            version: config.version.clone(),
            app_id: config.app_id.clone(),
            icon: None,
            description: None,
            copyright: None,
            publisher: None,
        }
    }
}

/// Package an artifact
pub async fn package_artifact(
    artifact: &Artifact,
    config: &ReleaseConfig,
) -> ReleaseResult<Artifact> {
    let packaging_config = PackagingConfig::from_release_config(config);

    // Get formats for this platform
    let formats = PackageFormat::defaults_for_platform(artifact.platform);

    for format in formats {
        if !format.is_supported_on_current() {
            tracing::warn!(
                "Cannot create {} on current platform, skipping",
                format
            );
            continue;
        }

        let packaged = create_package(artifact, format, &packaging_config).await?;
        return Ok(packaged);
    }

    // If no packaging was done, return original artifact
    Ok(artifact.clone())
}

/// Create a package in a specific format
pub async fn create_package(
    artifact: &Artifact,
    format: PackageFormat,
    config: &PackagingConfig,
) -> ReleaseResult<Artifact> {
    tracing::info!("Creating {} package for {}", format, artifact.name);

    match format {
        PackageFormat::Dmg => {
            #[cfg(target_os = "macos")]
            {
                let dmg_config = DmgConfig {
                    app_name: config.app_name.clone(),
                    volume_name: format!("{} {}", config.app_name, config.version),
                    output_path: config
                        .output_dir
                        .join(format!("{}-{}.dmg", config.app_name, config.version)),
                    background_image: None,
                    icon_size: 128,
                    window_width: 600,
                    window_height: 400,
                };
                return macos::create_dmg(&artifact.path, &dmg_config).await;
            }
            #[cfg(not(target_os = "macos"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "DMG can only be created on macOS".to_string(),
            ));
        }
        PackageFormat::Pkg => {
            #[cfg(target_os = "macos")]
            {
                let pkg_config = PkgConfig {
                    app_name: config.app_name.clone(),
                    identifier: config.app_id.clone(),
                    version: config.version.clone(),
                    output_path: config
                        .output_dir
                        .join(format!("{}-{}.pkg", config.app_name, config.version)),
                    install_location: PathBuf::from("/Applications"),
                    signing_identity: None,
                };
                return macos::create_pkg(&artifact.path, &pkg_config).await;
            }
            #[cfg(not(target_os = "macos"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "PKG can only be created on macOS".to_string(),
            ));
        }
        PackageFormat::Msi => {
            #[cfg(target_os = "windows")]
            {
                let msi_config = MsiConfig {
                    app_name: config.app_name.clone(),
                    version: config.version.clone(),
                    manufacturer: config.publisher.clone().unwrap_or_else(|| "Unknown".to_string()),
                    upgrade_code: uuid::Uuid::new_v5(
                        &uuid::Uuid::NAMESPACE_DNS,
                        config.app_id.as_bytes(),
                    ),
                    output_path: config
                        .output_dir
                        .join(format!("{}-{}.msi", config.app_name, config.version)),
                    icon: config.icon.clone(),
                    license_file: None,
                };
                return windows::create_msi(&artifact.path, &msi_config).await;
            }
            #[cfg(not(target_os = "windows"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "MSI can only be created on Windows".to_string(),
            ));
        }
        PackageFormat::AppImage => {
            #[cfg(target_os = "linux")]
            {
                let appimage_config = AppImageConfig {
                    app_name: config.app_name.clone(),
                    version: config.version.clone(),
                    output_path: config
                        .output_dir
                        .join(format!("{}-{}.AppImage", config.app_name, config.version)),
                    icon: config.icon.clone(),
                    desktop_entry: None,
                    categories: vec!["Utility".to_string()],
                };
                return linux::create_appimage(&artifact.path, &appimage_config).await;
            }
            #[cfg(not(target_os = "linux"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "AppImage can only be created on Linux".to_string(),
            ));
        }
        PackageFormat::Deb => {
            #[cfg(target_os = "linux")]
            {
                let deb_config = DebConfig {
                    package_name: config.app_name.to_lowercase().replace(' ', "-"),
                    version: config.version.clone(),
                    maintainer: config
                        .publisher
                        .clone()
                        .unwrap_or_else(|| "Unknown <unknown@example.com>".to_string()),
                    description: config
                        .description
                        .clone()
                        .unwrap_or_else(|| "OxideKit application".to_string()),
                    output_path: config.output_dir.join(format!(
                        "{}_{}_amd64.deb",
                        config.app_name.to_lowercase().replace(' ', "-"),
                        config.version
                    )),
                    depends: vec![],
                    section: "utils".to_string(),
                    priority: "optional".to_string(),
                };
                return linux::create_deb(&artifact.path, &deb_config).await;
            }
            #[cfg(not(target_os = "linux"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "DEB can only be created on Linux".to_string(),
            ));
        }
        PackageFormat::Rpm => {
            #[cfg(target_os = "linux")]
            {
                let rpm_config = RpmConfig {
                    name: config.app_name.to_lowercase().replace(' ', "-"),
                    version: config.version.clone(),
                    release: "1".to_string(),
                    summary: config
                        .description
                        .clone()
                        .unwrap_or_else(|| "OxideKit application".to_string()),
                    license: "MIT".to_string(),
                    output_dir: config.output_dir.clone(),
                    requires: vec![],
                };
                return linux::create_rpm(&artifact.path, &rpm_config).await;
            }
            #[cfg(not(target_os = "linux"))]
            return Err(ReleaseError::UnsupportedPackageFormat(
                "RPM can only be created on Linux".to_string(),
            ));
        }
        PackageFormat::Zip => {
            return create_zip_archive(artifact, config).await;
        }
        PackageFormat::TarGz => {
            return create_targz_archive(artifact, config).await;
        }
        _ => {
            return Err(ReleaseError::UnsupportedPackageFormat(format!(
                "{} not yet implemented",
                format
            )));
        }
    }
}

/// Create a ZIP archive
async fn create_zip_archive(
    artifact: &Artifact,
    config: &PackagingConfig,
) -> ReleaseResult<Artifact> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let output_path = config
        .output_dir
        .join(format!("{}-{}.zip", config.app_name, config.version));

    std::fs::create_dir_all(&config.output_dir)?;

    let file = std::fs::File::create(&output_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .unix_permissions(0o755);

    let file_name = artifact
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    zip.start_file(file_name.as_ref(), options)?;

    let content = std::fs::read(&artifact.path)?;
    zip.write_all(&content)?;

    zip.finish()?;

    let mut packaged = Artifact::new(
        format!("{}-{}.zip", config.app_name, config.version),
        artifact.platform,
        output_path,
    );
    packaged.update_metadata()?;

    Ok(packaged)
}

/// Create a tar.gz archive
async fn create_targz_archive(
    artifact: &Artifact,
    config: &PackagingConfig,
) -> ReleaseResult<Artifact> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let output_path = config
        .output_dir
        .join(format!("{}-{}.tar.gz", config.app_name, config.version));

    std::fs::create_dir_all(&config.output_dir)?;

    let file = std::fs::File::create(&output_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    let file_name = artifact
        .path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    tar.append_path_with_name(&artifact.path, file_name.as_ref())?;

    tar.finish()?;

    let mut packaged = Artifact::new(
        format!("{}-{}.tar.gz", config.app_name, config.version),
        artifact.platform,
        output_path,
    );
    packaged.update_metadata()?;

    Ok(packaged)
}
