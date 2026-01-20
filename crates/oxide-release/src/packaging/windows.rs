//! Windows packaging: MSI and MSIX creation

use crate::artifact::Artifact;
use crate::error::{ReleaseError, ReleaseResult};
use crate::TargetPlatform;
use std::path::{Path, PathBuf};
use std::process::Command;
use uuid::Uuid;

/// MSI creation configuration
#[derive(Debug, Clone)]
pub struct MsiConfig {
    /// Application name
    pub app_name: String,
    /// Version string (must be x.y.z format)
    pub version: String,
    /// Manufacturer/vendor name
    pub manufacturer: String,
    /// Upgrade code UUID (consistent across versions)
    pub upgrade_code: Uuid,
    /// Output MSI path
    pub output_path: PathBuf,
    /// Icon path (.ico)
    pub icon: Option<PathBuf>,
    /// License file (RTF format)
    pub license_file: Option<PathBuf>,
}

/// Create an MSI installer using WiX Toolset
pub async fn create_msi(binary_path: &Path, config: &MsiConfig) -> ReleaseResult<Artifact> {
    if !binary_path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            binary_path.display().to_string(),
        ));
    }

    // Check for WiX tools
    if which::which("candle").is_err() || which::which("light").is_err() {
        return Err(ReleaseError::ToolNotFound(
            "WiX Toolset (candle/light)".to_string(),
        ));
    }

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    tracing::info!("Creating MSI installer using WiX...");

    let temp_dir = tempfile::tempdir()?;
    let wxs_path = temp_dir.path().join("installer.wxs");
    let wixobj_path = temp_dir.path().join("installer.wixobj");

    // Generate product code (unique per version)
    let product_code = Uuid::new_v5(&config.upgrade_code, config.version.as_bytes());

    // Generate WiX source
    let wxs_content = generate_wix_source(binary_path, config, &product_code)?;
    std::fs::write(&wxs_path, wxs_content)?;

    // Compile WiX source
    let candle_output = Command::new("candle")
        .arg("-nologo")
        .arg("-arch").arg("x64")
        .arg("-out").arg(&wixobj_path)
        .arg(&wxs_path)
        .output()?;

    if !candle_output.status.success() {
        let stderr = String::from_utf8_lossy(&candle_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "WiX candle failed: {}",
            stderr.trim()
        )));
    }

    // Link to MSI
    let light_output = Command::new("light")
        .arg("-nologo")
        .arg("-ext").arg("WixUIExtension")
        .arg("-out").arg(&config.output_path)
        .arg(&wixobj_path)
        .output()?;

    if !light_output.status.success() {
        let stderr = String::from_utf8_lossy(&light_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "WiX light failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::WindowsX64,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Generate WiX source file
fn generate_wix_source(
    binary_path: &Path,
    config: &MsiConfig,
    product_code: &Uuid,
) -> ReleaseResult<String> {
    let binary_name = binary_path
        .file_name()
        .unwrap()
        .to_string_lossy();

    let exe_name = if binary_name.ends_with(".exe") {
        binary_name.to_string()
    } else {
        format!("{}.exe", binary_name)
    };

    let wxs = format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<Wix xmlns="http://schemas.microsoft.com/wix/2006/wi">
    <Product
        Id="{product_code}"
        Name="{app_name}"
        Language="1033"
        Version="{version}"
        Manufacturer="{manufacturer}"
        UpgradeCode="{upgrade_code}">

        <Package
            InstallerVersion="500"
            Compressed="yes"
            InstallScope="perMachine"
            Platform="x64" />

        <MajorUpgrade
            DowngradeErrorMessage="A newer version of [ProductName] is already installed."
            AllowSameVersionUpgrades="yes" />

        <MediaTemplate EmbedCab="yes" />

        <Feature Id="ProductFeature" Title="{app_name}" Level="1">
            <ComponentGroupRef Id="ProductComponents" />
            <ComponentRef Id="ApplicationShortcut" />
        </Feature>

        <Directory Id="TARGETDIR" Name="SourceDir">
            <Directory Id="ProgramFiles64Folder">
                <Directory Id="INSTALLFOLDER" Name="{app_name}">
                    <Component Id="MainExecutable" Guid="*">
                        <File Id="MainExeFile" Source="{binary_path}" KeyPath="yes" />
                    </Component>
                </Directory>
            </Directory>
            <Directory Id="ProgramMenuFolder">
                <Directory Id="ApplicationProgramsFolder" Name="{app_name}" />
            </Directory>
            <Directory Id="DesktopFolder" Name="Desktop" />
        </Directory>

        <DirectoryRef Id="ApplicationProgramsFolder">
            <Component Id="ApplicationShortcut" Guid="*">
                <Shortcut Id="ApplicationStartMenuShortcut"
                    Name="{app_name}"
                    Description="{app_name}"
                    Target="[INSTALLFOLDER]{exe_name}"
                    WorkingDirectory="INSTALLFOLDER" />
                <RemoveFolder Id="CleanUpShortCut" Directory="ApplicationProgramsFolder" On="uninstall" />
                <RegistryValue Root="HKCU" Key="Software\\{manufacturer}\\{app_name}"
                    Name="installed" Type="integer" Value="1" KeyPath="yes" />
            </Component>
        </DirectoryRef>

        <ComponentGroup Id="ProductComponents" Directory="INSTALLFOLDER">
            <ComponentRef Id="MainExecutable" />
        </ComponentGroup>

        <UI>
            <UIRef Id="WixUI_Minimal" />
        </UI>
    </Product>
</Wix>"#,
        product_code = product_code,
        app_name = config.app_name,
        version = config.version,
        manufacturer = config.manufacturer,
        upgrade_code = config.upgrade_code,
        binary_path = binary_path.display(),
        exe_name = exe_name,
    );

    Ok(wxs)
}

/// MSIX creation configuration
#[derive(Debug, Clone)]
pub struct MsixConfig {
    /// Application name
    pub app_name: String,
    /// Publisher DN (e.g., "CN=Company Name")
    pub publisher: String,
    /// Version string (must be x.y.z.w format)
    pub version: String,
    /// Output MSIX path
    pub output_path: PathBuf,
    /// Icon path (.png, 150x150 for tiles)
    pub icon: Option<PathBuf>,
    /// Description
    pub description: Option<String>,
}

/// Create an MSIX package
pub async fn create_msix(binary_path: &Path, config: &MsixConfig) -> ReleaseResult<Artifact> {
    if !binary_path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            binary_path.display().to_string(),
        ));
    }

    // Check for makeappx
    if which::which("makeappx").is_err() {
        return Err(ReleaseError::ToolNotFound("makeappx".to_string()));
    }

    tracing::info!("Creating MSIX package...");

    let temp_dir = tempfile::tempdir()?;
    let package_dir = temp_dir.path().join("package");
    std::fs::create_dir_all(&package_dir)?;

    // Copy binary
    let exe_name = format!("{}.exe", config.app_name);
    let exe_dest = package_dir.join(&exe_name);
    std::fs::copy(binary_path, &exe_dest)?;

    // Create AppxManifest.xml
    let manifest = generate_appx_manifest(config, &exe_name)?;
    std::fs::write(package_dir.join("AppxManifest.xml"), manifest)?;

    // Copy/create assets
    create_msix_assets(&package_dir, config)?;

    // Create MSIX
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let makeappx_output = Command::new("makeappx")
        .arg("pack")
        .arg("/d").arg(&package_dir)
        .arg("/p").arg(&config.output_path)
        .arg("/o") // Overwrite
        .output()?;

    if !makeappx_output.status.success() {
        let stderr = String::from_utf8_lossy(&makeappx_output.stderr);
        return Err(ReleaseError::packaging(format!(
            "makeappx failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::WindowsX64,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Generate AppxManifest.xml
fn generate_appx_manifest(config: &MsixConfig, exe_name: &str) -> ReleaseResult<String> {
    let identity_name = config.app_name.replace(' ', "");
    let description = config.description.as_deref().unwrap_or(&config.app_name);

    let manifest = format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<Package
    xmlns="http://schemas.microsoft.com/appx/manifest/foundation/windows10"
    xmlns:uap="http://schemas.microsoft.com/appx/manifest/uap/windows10"
    xmlns:rescap="http://schemas.microsoft.com/appx/manifest/foundation/windows10/restrictedcapabilities"
    IgnorableNamespaces="uap rescap">

    <Identity
        Name="{identity_name}"
        Publisher="{publisher}"
        Version="{version}"
        ProcessorArchitecture="x64" />

    <Properties>
        <DisplayName>{app_name}</DisplayName>
        <PublisherDisplayName>{publisher_display}</PublisherDisplayName>
        <Logo>Assets\StoreLogo.png</Logo>
    </Properties>

    <Dependencies>
        <TargetDeviceFamily Name="Windows.Desktop" MinVersion="10.0.17763.0" MaxVersionTested="10.0.22621.0" />
    </Dependencies>

    <Resources>
        <Resource Language="en-us" />
    </Resources>

    <Applications>
        <Application Id="App" Executable="{exe_name}" EntryPoint="Windows.FullTrustApplication">
            <uap:VisualElements
                DisplayName="{app_name}"
                Description="{description}"
                BackgroundColor="transparent"
                Square150x150Logo="Assets\Square150x150Logo.png"
                Square44x44Logo="Assets\Square44x44Logo.png">
                <uap:DefaultTile Wide310x150Logo="Assets\Wide310x150Logo.png" />
            </uap:VisualElements>
        </Application>
    </Applications>

    <Capabilities>
        <rescap:Capability Name="runFullTrust" />
    </Capabilities>
</Package>"#,
        identity_name = identity_name,
        publisher = config.publisher,
        publisher_display = config.publisher.trim_start_matches("CN="),
        version = config.version,
        app_name = config.app_name,
        exe_name = exe_name,
        description = description,
    );

    Ok(manifest)
}

/// Create MSIX asset placeholders
fn create_msix_assets(package_dir: &Path, _config: &MsixConfig) -> ReleaseResult<()> {
    let assets_dir = package_dir.join("Assets");
    std::fs::create_dir_all(&assets_dir)?;

    // Create placeholder PNGs (1x1 transparent)
    // In production, these would be actual icons from config
    let placeholder_png = include_bytes!("../../placeholder_icon.png").to_vec();

    // If placeholder doesn't exist, create a minimal valid PNG
    let png_data = if placeholder_png.is_empty() {
        // Minimal 1x1 transparent PNG
        vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01,
            0x08, 0x06, 0x00, 0x00, 0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00,
            0x0A, 0x49, 0x44, 0x41, 0x54, 0x78, 0x9C, 0x63, 0x00, 0x01, 0x00, 0x00,
            0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00, 0x00, 0x00, 0x00, 0x49,
            0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ]
    } else {
        placeholder_png
    };

    std::fs::write(assets_dir.join("StoreLogo.png"), &png_data)?;
    std::fs::write(assets_dir.join("Square150x150Logo.png"), &png_data)?;
    std::fs::write(assets_dir.join("Square44x44Logo.png"), &png_data)?;
    std::fs::write(assets_dir.join("Wide310x150Logo.png"), &png_data)?;

    Ok(())
}

/// Sign an MSI/MSIX package
pub async fn sign_windows_package(
    package_path: &Path,
    signing_config: &crate::config::SigningConfig,
) -> ReleaseResult<()> {
    let artifact = Artifact::new(
        package_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::WindowsX64,
        package_path,
    );

    crate::signing::windows::sign_windows(&artifact, signing_config).await?;
    Ok(())
}

/// Create portable ZIP for Windows (no installer)
pub async fn create_portable_zip(
    binary_path: &Path,
    app_name: &str,
    version: &str,
    output_dir: &Path,
) -> ReleaseResult<Artifact> {
    use std::io::Write;
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    let output_path = output_dir.join(format!("{}-{}-portable.zip", app_name, version));

    std::fs::create_dir_all(output_dir)?;

    let file = std::fs::File::create(&output_path)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    // Add executable
    let exe_name = format!("{}.exe", app_name);
    zip.start_file(&exe_name, options)?;
    let content = std::fs::read(binary_path)?;
    zip.write_all(&content)?;

    zip.finish()?;

    let mut artifact = Artifact::new(
        output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::WindowsX64,
        output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}
