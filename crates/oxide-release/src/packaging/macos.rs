//! macOS packaging: DMG and PKG creation

use crate::artifact::Artifact;
use crate::error::{ReleaseError, ReleaseResult};
use crate::TargetPlatform;
use std::path::{Path, PathBuf};
use std::process::Command;

/// DMG creation configuration
#[derive(Debug, Clone)]
pub struct DmgConfig {
    /// Application name
    pub app_name: String,
    /// Volume name (appears when mounted)
    pub volume_name: String,
    /// Output DMG path
    pub output_path: PathBuf,
    /// Background image path
    pub background_image: Option<PathBuf>,
    /// Icon size in pixels
    pub icon_size: u32,
    /// Window width
    pub window_width: u32,
    /// Window height
    pub window_height: u32,
}

/// Create a DMG disk image from an app bundle
pub async fn create_dmg(app_bundle: &Path, config: &DmgConfig) -> ReleaseResult<Artifact> {
    if !app_bundle.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            app_bundle.display().to_string(),
        ));
    }

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove existing DMG if present
    if config.output_path.exists() {
        std::fs::remove_file(&config.output_path)?;
    }

    // Check for create-dmg tool (preferred)
    if which::which("create-dmg").is_ok() {
        create_dmg_with_create_dmg(app_bundle, config).await
    } else {
        // Fall back to hdiutil
        create_dmg_with_hdiutil(app_bundle, config).await
    }
}

/// Create DMG using create-dmg tool (produces nicer DMGs)
async fn create_dmg_with_create_dmg(app_bundle: &Path, config: &DmgConfig) -> ReleaseResult<Artifact> {
    tracing::info!("Creating DMG using create-dmg...");

    let mut cmd = Command::new("create-dmg");

    cmd.arg("--volname").arg(&config.volume_name);
    cmd.arg("--window-pos").arg("200").arg("120");
    cmd.arg("--window-size")
        .arg(config.window_width.to_string())
        .arg(config.window_height.to_string());
    cmd.arg("--icon-size").arg(config.icon_size.to_string());

    // App icon position
    cmd.arg("--icon")
        .arg(&config.app_name)
        .arg("140")
        .arg("160");

    // Applications folder symlink
    cmd.arg("--hide-extension").arg(&config.app_name);
    cmd.arg("--app-drop-link").arg("400").arg("160");

    // Background
    if let Some(ref bg) = config.background_image {
        if bg.exists() {
            cmd.arg("--background").arg(bg);
        }
    }

    // Output path and source
    cmd.arg(&config.output_path);
    cmd.arg(app_bundle);

    let output = cmd.output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::packaging(format!(
            "create-dmg failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::MacOSUniversal,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Create DMG using hdiutil (built into macOS)
async fn create_dmg_with_hdiutil(app_bundle: &Path, config: &DmgConfig) -> ReleaseResult<Artifact> {
    tracing::info!("Creating DMG using hdiutil...");

    let temp_dir = tempfile::tempdir()?;
    let staging_dir = temp_dir.path().join("staging");
    std::fs::create_dir(&staging_dir)?;

    // Copy app to staging
    let app_name = app_bundle.file_name().unwrap();
    let staged_app = staging_dir.join(app_name);

    // Use ditto for proper app bundle copying
    let copy_output = Command::new("ditto")
        .arg(app_bundle)
        .arg(&staged_app)
        .output()?;

    if !copy_output.status.success() {
        return Err(ReleaseError::packaging("Failed to copy app bundle"));
    }

    // Create Applications symlink
    let apps_symlink = staging_dir.join("Applications");
    std::os::unix::fs::symlink("/Applications", &apps_symlink)?;

    // Create temporary DMG (uncompressed)
    let temp_dmg = temp_dir.path().join("temp.dmg");

    let create_output = Command::new("hdiutil")
        .args([
            "create",
            "-srcfolder",
            staging_dir.to_str().unwrap(),
            "-volname",
            &config.volume_name,
            "-fs",
            "HFS+",
            "-fsargs",
            "-c c=64,a=16,e=16",
            "-format",
            "UDRW",
        ])
        .arg(&temp_dmg)
        .output()?;

    if !create_output.status.success() {
        let stderr = String::from_utf8_lossy(&create_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "hdiutil create failed: {}",
            stderr.trim()
        )));
    }

    // Convert to compressed DMG
    let convert_output = Command::new("hdiutil")
        .args([
            "convert",
            temp_dmg.to_str().unwrap(),
            "-format",
            "UDZO",
            "-imagekey",
            "zlib-level=9",
            "-o",
        ])
        .arg(&config.output_path)
        .output()?;

    if !convert_output.status.success() {
        let stderr = String::from_utf8_lossy(&convert_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "hdiutil convert failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::MacOSUniversal,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// PKG creation configuration
#[derive(Debug, Clone)]
pub struct PkgConfig {
    /// Application name
    pub app_name: String,
    /// Bundle identifier
    pub identifier: String,
    /// Version string
    pub version: String,
    /// Output PKG path
    pub output_path: PathBuf,
    /// Install location
    pub install_location: PathBuf,
    /// Signing identity for PKG (Developer ID Installer)
    pub signing_identity: Option<String>,
}

/// Create a PKG installer
pub async fn create_pkg(app_bundle: &Path, config: &PkgConfig) -> ReleaseResult<Artifact> {
    if !app_bundle.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            app_bundle.display().to_string(),
        ));
    }

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    tracing::info!("Creating PKG installer...");

    let temp_dir = tempfile::tempdir()?;

    // Create component PKG
    let component_pkg = temp_dir.path().join("component.pkg");

    let mut pkgbuild = Command::new("pkgbuild");
    pkgbuild
        .arg("--root")
        .arg(app_bundle.parent().unwrap())
        .arg("--install-location")
        .arg(&config.install_location)
        .arg("--identifier")
        .arg(&config.identifier)
        .arg("--version")
        .arg(&config.version)
        .arg(&component_pkg);

    let pkgbuild_output = pkgbuild.output()?;

    if !pkgbuild_output.status.success() {
        let stderr = String::from_utf8_lossy(&pkgbuild_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "pkgbuild failed: {}",
            stderr.trim()
        )));
    }

    // Create distribution XML
    let distribution_xml = temp_dir.path().join("distribution.xml");
    let distribution_content = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<installer-gui-script minSpecVersion="2">
    <title>{}</title>
    <organization>{}</organization>
    <domains enable_localSystem="true"/>
    <options customize="never" require-scripts="false" hostArchitectures="arm64,x86_64"/>
    <choices-outline>
        <line choice="default">
            <line choice="{}.pkg"/>
        </line>
    </choices-outline>
    <choice id="default"/>
    <choice id="{}.pkg" visible="false">
        <pkg-ref id="{}"/>
    </choice>
    <pkg-ref id="{}" version="{}" onConclusion="none">component.pkg</pkg-ref>
</installer-gui-script>"#,
        config.app_name,
        config.identifier,
        config.identifier,
        config.identifier,
        config.identifier,
        config.identifier,
        config.version
    );

    std::fs::write(&distribution_xml, distribution_content)?;

    // Build final PKG with productbuild
    let mut productbuild = Command::new("productbuild");
    productbuild
        .arg("--distribution")
        .arg(&distribution_xml)
        .arg("--package-path")
        .arg(temp_dir.path());

    // Sign if identity provided
    if let Some(ref identity) = config.signing_identity {
        productbuild.arg("--sign").arg(identity);
    }

    productbuild.arg(&config.output_path);

    let productbuild_output = productbuild.output()?;

    if !productbuild_output.status.success() {
        let stderr = String::from_utf8_lossy(&productbuild_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "productbuild failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::MacOSUniversal,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Create an app bundle structure from a binary
pub async fn create_app_bundle(
    binary_path: &Path,
    app_name: &str,
    bundle_id: &str,
    version: &str,
    output_dir: &Path,
) -> ReleaseResult<PathBuf> {
    let bundle_path = output_dir.join(format!("{}.app", app_name));

    // Create directory structure
    let contents = bundle_path.join("Contents");
    let macos = contents.join("MacOS");
    let resources = contents.join("Resources");

    std::fs::create_dir_all(&macos)?;
    std::fs::create_dir_all(&resources)?;

    // Copy binary
    let binary_dest = macos.join(app_name);
    std::fs::copy(binary_path, &binary_dest)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary_dest, std::fs::Permissions::from_mode(0o755))?;
    }

    // Create Info.plist
    let info_plist = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>{}</string>
    <key>CFBundleIdentifier</key>
    <string>{}</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>{}</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>{}</string>
    <key>CFBundleVersion</key>
    <string>{}</string>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSSupportsAutomaticGraphicsSwitching</key>
    <true/>
</dict>
</plist>"#,
        app_name, bundle_id, app_name, version, version
    );

    std::fs::write(contents.join("Info.plist"), info_plist)?;

    // Create PkgInfo
    std::fs::write(contents.join("PkgInfo"), "APPL????")?;

    Ok(bundle_path)
}

/// Sign and notarize a DMG
pub async fn sign_and_notarize_dmg(
    dmg_path: &Path,
    signing_identity: &str,
    notarization_config: Option<&crate::config::NotarizationConfig>,
) -> ReleaseResult<()> {
    // Sign DMG
    let sign_output = Command::new("codesign")
        .args(["--force", "--sign"])
        .arg(signing_identity)
        .arg(dmg_path)
        .output()?;

    if !sign_output.status.success() {
        let stderr = String::from_utf8_lossy(&sign_output.stderr);
        return Err(ReleaseError::signing(format!(
            "Failed to sign DMG: {}",
            stderr.trim()
        )));
    }

    // Notarize if config provided
    #[cfg(feature = "notarization")]
    if let Some(config) = notarization_config {
        let artifact = Artifact::new(
            dmg_path.file_name().unwrap().to_string_lossy(),
            TargetPlatform::MacOSUniversal,
            dmg_path,
        );
        crate::notarization::notarize_artifact(&artifact, config).await?;
    }

    Ok(())
}
