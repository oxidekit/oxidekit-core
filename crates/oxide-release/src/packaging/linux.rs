//! Linux packaging: AppImage, DEB, and RPM creation

use crate::artifact::Artifact;
use crate::error::{ReleaseError, ReleaseResult};
use crate::TargetPlatform;
use std::path::{Path, PathBuf};
use std::process::Command;

/// AppImage creation configuration
#[derive(Debug, Clone)]
pub struct AppImageConfig {
    /// Application name
    pub app_name: String,
    /// Version string
    pub version: String,
    /// Output AppImage path
    pub output_path: PathBuf,
    /// Icon path (.png, preferably 256x256)
    pub icon: Option<PathBuf>,
    /// Desktop entry content (optional, will be generated if not provided)
    pub desktop_entry: Option<String>,
    /// Desktop categories
    pub categories: Vec<String>,
}

/// Create an AppImage
pub async fn create_appimage(binary_path: &Path, config: &AppImageConfig) -> ReleaseResult<Artifact> {
    if !binary_path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            binary_path.display().to_string(),
        ));
    }

    // Check for appimagetool
    let appimagetool = find_appimagetool()?;

    tracing::info!("Creating AppImage...");

    let temp_dir = tempfile::tempdir()?;
    let appdir = temp_dir.path().join(format!("{}.AppDir", config.app_name));

    // Create AppDir structure
    std::fs::create_dir_all(appdir.join("usr/bin"))?;
    std::fs::create_dir_all(appdir.join("usr/share/applications"))?;
    std::fs::create_dir_all(appdir.join("usr/share/icons/hicolor/256x256/apps"))?;

    // Copy binary
    let binary_dest = appdir.join("usr/bin").join(&config.app_name);
    std::fs::copy(binary_path, &binary_dest)?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary_dest, std::fs::Permissions::from_mode(0o755))?;
    }

    // Create or copy icon
    let icon_filename = format!("{}.png", config.app_name.to_lowercase());
    let icon_dest = appdir.join("usr/share/icons/hicolor/256x256/apps").join(&icon_filename);

    if let Some(ref icon) = config.icon {
        if icon.exists() {
            std::fs::copy(icon, &icon_dest)?;
        }
    }

    // Also place icon in root for AppImage
    if icon_dest.exists() {
        std::fs::copy(&icon_dest, appdir.join(&icon_filename))?;
    }

    // Create .desktop file
    let desktop_content = config.desktop_entry.clone().unwrap_or_else(|| {
        let categories = if config.categories.is_empty() {
            "Utility".to_string()
        } else {
            config.categories.join(";")
        };

        format!(
            r#"[Desktop Entry]
Type=Application
Name={name}
Exec={exec}
Icon={icon}
Categories={categories}
Terminal=false
"#,
            name = config.app_name,
            exec = config.app_name.to_lowercase(),
            icon = config.app_name.to_lowercase(),
            categories = categories,
        )
    });

    let desktop_path = appdir.join("usr/share/applications").join(format!("{}.desktop", config.app_name.to_lowercase()));
    std::fs::write(&desktop_path, &desktop_content)?;

    // Link desktop file to root
    std::fs::copy(&desktop_path, appdir.join(format!("{}.desktop", config.app_name.to_lowercase())))?;

    // Create AppRun script
    let apprun_content = format!(
        r#"#!/bin/bash
SELF=$(readlink -f "$0")
HERE=${{SELF%/*}}
export PATH="${{HERE}}/usr/bin:${{PATH}}"
export LD_LIBRARY_PATH="${{HERE}}/usr/lib:${{LD_LIBRARY_PATH}}"
exec "${{HERE}}/usr/bin/{}" "$@"
"#,
        config.app_name
    );

    let apprun_path = appdir.join("AppRun");
    std::fs::write(&apprun_path, apprun_content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&apprun_path, std::fs::Permissions::from_mode(0o755))?;
    }

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Run appimagetool
    let output = Command::new(&appimagetool)
        .arg(&appdir)
        .arg(&config.output_path)
        .env("ARCH", "x86_64")
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::packaging(format!(
            "appimagetool failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::LinuxX64,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Find appimagetool
fn find_appimagetool() -> ReleaseResult<PathBuf> {
    // Check PATH
    if let Ok(path) = which::which("appimagetool") {
        return Ok(path);
    }

    // Check common locations
    let locations = [
        "/usr/local/bin/appimagetool",
        "~/.local/bin/appimagetool",
        "/opt/appimagetool/appimagetool",
    ];

    for loc in locations {
        let path = PathBuf::from(shellexpand::tilde(loc).to_string());
        if path.exists() {
            return Ok(path);
        }
    }

    Err(ReleaseError::ToolNotFound("appimagetool".to_string()))
}

/// DEB package configuration
#[derive(Debug, Clone)]
pub struct DebConfig {
    /// Package name (lowercase, no spaces)
    pub package_name: String,
    /// Version string
    pub version: String,
    /// Maintainer (Name <email>)
    pub maintainer: String,
    /// Description
    pub description: String,
    /// Output DEB path
    pub output_path: PathBuf,
    /// Dependencies
    pub depends: Vec<String>,
    /// Section (e.g., "utils", "devel")
    pub section: String,
    /// Priority (optional, required, important, standard, extra)
    pub priority: String,
}

/// Create a DEB package
pub async fn create_deb(binary_path: &Path, config: &DebConfig) -> ReleaseResult<Artifact> {
    if !binary_path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            binary_path.display().to_string(),
        ));
    }

    // Check for dpkg-deb
    if which::which("dpkg-deb").is_err() {
        return Err(ReleaseError::ToolNotFound("dpkg-deb".to_string()));
    }

    tracing::info!("Creating DEB package...");

    let temp_dir = tempfile::tempdir()?;
    let pkg_dir = temp_dir.path().join(&config.package_name);

    // Create directory structure
    std::fs::create_dir_all(pkg_dir.join("DEBIAN"))?;
    std::fs::create_dir_all(pkg_dir.join("usr/bin"))?;
    std::fs::create_dir_all(pkg_dir.join("usr/share/applications"))?;

    // Copy binary
    let binary_dest = pkg_dir.join("usr/bin").join(&config.package_name);
    std::fs::copy(binary_path, &binary_dest)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&binary_dest, std::fs::Permissions::from_mode(0o755))?;
    }

    // Calculate installed size
    let metadata = std::fs::metadata(binary_path)?;
    let installed_size = metadata.len() / 1024; // KB

    // Create control file
    let depends = if config.depends.is_empty() {
        String::new()
    } else {
        format!("Depends: {}\n", config.depends.join(", "))
    };

    let control = format!(
        r#"Package: {package}
Version: {version}
Section: {section}
Priority: {priority}
Architecture: amd64
Installed-Size: {size}
Maintainer: {maintainer}
{depends}Description: {description}
"#,
        package = config.package_name,
        version = config.version,
        section = config.section,
        priority = config.priority,
        size = installed_size,
        maintainer = config.maintainer,
        depends = depends,
        description = config.description,
    );

    std::fs::write(pkg_dir.join("DEBIAN/control"), control)?;

    // Create .desktop file
    let desktop = format!(
        r#"[Desktop Entry]
Type=Application
Name={name}
Exec={exec}
Terminal=false
Categories=Utility;
"#,
        name = config.package_name,
        exec = config.package_name,
    );

    std::fs::write(
        pkg_dir.join(format!("usr/share/applications/{}.desktop", config.package_name)),
        desktop,
    )?;

    // Ensure output directory exists
    if let Some(parent) = config.output_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Build DEB
    let output = Command::new("dpkg-deb")
        .arg("--build")
        .arg("--root-owner-group")
        .arg(&pkg_dir)
        .arg(&config.output_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::packaging(format!(
            "dpkg-deb failed: {}",
            stderr.trim()
        )));
    }

    let mut artifact = Artifact::new(
        config.output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::LinuxX64,
        &config.output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// RPM package configuration
#[derive(Debug, Clone)]
pub struct RpmConfig {
    /// Package name
    pub name: String,
    /// Version string
    pub version: String,
    /// Release number
    pub release: String,
    /// Summary/description
    pub summary: String,
    /// License
    pub license: String,
    /// Output directory
    pub output_dir: PathBuf,
    /// Dependencies
    pub requires: Vec<String>,
}

/// Create an RPM package
pub async fn create_rpm(binary_path: &Path, config: &RpmConfig) -> ReleaseResult<Artifact> {
    if !binary_path.exists() {
        return Err(ReleaseError::ArtifactNotFound(
            binary_path.display().to_string(),
        ));
    }

    // Check for rpmbuild
    if which::which("rpmbuild").is_err() {
        return Err(ReleaseError::ToolNotFound("rpmbuild".to_string()));
    }

    tracing::info!("Creating RPM package...");

    let temp_dir = tempfile::tempdir()?;

    // Create rpmbuild directory structure
    let topdir = temp_dir.path().join("rpmbuild");
    for subdir in ["BUILD", "RPMS", "SOURCES", "SPECS", "SRPMS"] {
        std::fs::create_dir_all(topdir.join(subdir))?;
    }

    // Copy source to SOURCES
    let source_dir = topdir.join("SOURCES").join(&config.name);
    std::fs::create_dir_all(&source_dir)?;
    std::fs::copy(binary_path, source_dir.join(&config.name))?;

    // Create tarball
    let tarball_name = format!("{}-{}.tar.gz", config.name, config.version);
    let tarball_path = topdir.join("SOURCES").join(&tarball_name);

    let tar_output = Command::new("tar")
        .arg("-czf")
        .arg(&tarball_path)
        .arg("-C")
        .arg(topdir.join("SOURCES"))
        .arg(&config.name)
        .output()?;

    if !tar_output.status.success() {
        return Err(ReleaseError::packaging("Failed to create source tarball"));
    }

    // Create spec file
    let requires = if config.requires.is_empty() {
        String::new()
    } else {
        config.requires.iter()
            .map(|r| format!("Requires: {}", r))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let spec = format!(
        r#"Name:           {name}
Version:        {version}
Release:        {release}%{{?dist}}
Summary:        {summary}
License:        {license}
Source0:        {tarball}

{requires}

%description
{summary}

%prep
%setup -q

%install
mkdir -p %{{buildroot}}/usr/bin
install -m 755 {name} %{{buildroot}}/usr/bin/{name}

%files
/usr/bin/{name}

%changelog
* {date} Auto Generated <noreply@oxidekit.com> - {version}-{release}
- Initial package
"#,
        name = config.name,
        version = config.version,
        release = config.release,
        summary = config.summary,
        license = config.license,
        tarball = tarball_name,
        requires = requires,
        date = chrono::Utc::now().format("%a %b %d %Y"),
    );

    let spec_path = topdir.join("SPECS").join(format!("{}.spec", config.name));
    std::fs::write(&spec_path, spec)?;

    // Build RPM
    let output = Command::new("rpmbuild")
        .arg("--define")
        .arg(format!("_topdir {}", topdir.display()))
        .arg("-bb")
        .arg(&spec_path)
        .output()?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ReleaseError::packaging(format!(
            "rpmbuild failed: {}",
            stderr.trim()
        )));
    }

    // Find output RPM
    let rpms_dir = topdir.join("RPMS/x86_64");
    let rpm_file = std::fs::read_dir(&rpms_dir)?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().map(|ext| ext == "rpm").unwrap_or(false))
        .ok_or_else(|| ReleaseError::packaging("RPM file not found after build"))?;

    // Move to output directory
    std::fs::create_dir_all(&config.output_dir)?;
    let output_path = config.output_dir.join(rpm_file.file_name());
    std::fs::copy(rpm_file.path(), &output_path)?;

    let mut artifact = Artifact::new(
        output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::LinuxX64,
        output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}

/// Create a portable tar.gz archive for Linux
pub async fn create_portable_targz(
    binary_path: &Path,
    app_name: &str,
    version: &str,
    output_dir: &Path,
) -> ReleaseResult<Artifact> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let output_path = output_dir.join(format!("{}-{}-linux-x64.tar.gz", app_name, version));

    std::fs::create_dir_all(output_dir)?;

    let file = std::fs::File::create(&output_path)?;
    let enc = GzEncoder::new(file, Compression::default());
    let mut tar = Builder::new(enc);

    // Add binary
    tar.append_path_with_name(binary_path, app_name)?;

    tar.finish()?;

    let mut artifact = Artifact::new(
        output_path.file_name().unwrap().to_string_lossy(),
        TargetPlatform::LinuxX64,
        output_path,
    );
    artifact.update_metadata()?;

    Ok(artifact)
}
