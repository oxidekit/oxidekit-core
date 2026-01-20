//! Brand management CLI commands
//!
//! Commands for managing brand packs, checking compliance, and exporting assets.

use anyhow::{Context, Result};
use std::path::PathBuf;

use oxide_branding::{
    BrandPack, AppPack, BrandManager, ComplianceChecker,
    BrandExporter, ExportOptions, ExportFormat,
    WhiteLabelConfig, WhiteLabelBuilder, WhiteLabelMode,
    governance::GovernanceBuilder,
    compliance::ComplianceConfig,
    asset::IconPlatform,
};

/// Initialize a new brand pack
pub fn run_init(name: &str, output: Option<&str>) -> Result<()> {
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("brand.toml"));

    if output_path.exists() {
        anyhow::bail!("Brand file already exists at {:?}", output_path);
    }

    let brand = BrandPack::new(name);
    brand.save(&output_path)
        .context("Failed to save brand pack")?;

    println!("Created brand pack: {:?}", output_path);
    println!("\nNext steps:");
    println!("  1. Edit {} to customize your brand", output_path.display());
    println!("  2. Run 'oxide brand validate' to check your configuration");
    println!("  3. Run 'oxide brand export' to generate assets");

    Ok(())
}

/// Validate a brand pack
pub fn run_validate(path: Option<&str>) -> Result<()> {
    let path = path.unwrap_or("brand.toml");
    let brand = BrandPack::from_file(path)
        .context(format!("Failed to load brand pack from {}", path))?;

    println!("Brand Pack: {}", brand.identity.name);
    println!("Version: {}", brand.version);
    println!();

    // Validate brand pack
    match brand.validate() {
        Ok(()) => {
            println!("Validation: PASSED");
            println!();
            print_brand_summary(&brand);
        }
        Err(e) => {
            println!("Validation: FAILED");
            println!("Error: {}", e);
            std::process::exit(1);
        }
    }

    Ok(())
}

/// Check compliance of an app pack against a brand
pub fn run_check(brand_path: Option<&str>, app_path: Option<&str>) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");
    let app_path = app_path.unwrap_or("app.toml");

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let app = AppPack::from_file(app_path)
        .context(format!("Failed to load app pack from {}", app_path))?;

    let checker = ComplianceChecker::new(&brand);
    let report = checker.check_app_pack(&app);

    println!("{}", report.format());

    if !report.passed() {
        std::process::exit(1);
    }

    Ok(())
}

/// Check compliance of a white-label config
pub fn run_check_white_label(brand_path: Option<&str>, config_path: &str) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let config = WhiteLabelConfig::from_file(config_path)
        .context(format!("Failed to load white-label config from {}", config_path))?;

    let checker = ComplianceChecker::new(&brand);
    let report = checker.check_white_label(&config);

    println!("{}", report.format());

    if !report.passed() {
        std::process::exit(1);
    }

    Ok(())
}

/// Export brand assets
pub fn run_export(
    brand_path: Option<&str>,
    output: Option<&str>,
    format: Option<&str>,
) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");
    let output_dir = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("brand-export"));

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let export_format = match format.unwrap_or("brand-kit") {
        "brand-kit" => ExportFormat::BrandKit,
        "tokens" => ExportFormat::DesignTokens,
        "icons" => ExportFormat::IconSet,
        "colors" => ExportFormat::ColorPalette,
        "css" => ExportFormat::Css,
        "scss" => ExportFormat::Scss,
        "tailwind" => ExportFormat::TailwindConfig,
        "figma" => ExportFormat::FigmaTokens,
        "swift" => ExportFormat::SwiftUIColors,
        "android" => ExportFormat::AndroidColors,
        "style-dictionary" => ExportFormat::StyleDictionary,
        other => anyhow::bail!("Unknown export format: {}", other),
    };

    let options = ExportOptions {
        output_dir: output_dir.clone(),
        format: export_format,
        platforms: vec![IconPlatform::Web, IconPlatform::MacOS, IconPlatform::Windows],
        include_assets: true,
        generate_docs: true,
    };

    let exporter = BrandExporter::new(&brand);
    exporter.export(options)
        .context("Failed to export brand assets")?;

    println!("Exported brand assets to {:?}", output_dir);
    println!("Format: {:?}", export_format);

    Ok(())
}

/// Generate icons from brand logo
pub fn run_generate_icons(
    brand_path: Option<&str>,
    output: Option<&str>,
    platform: Option<&str>,
) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");
    let output_dir = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("icons"));

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let platforms: Vec<IconPlatform> = match platform {
        Some("web") => vec![IconPlatform::Web],
        Some("macos") => vec![IconPlatform::MacOS],
        Some("windows") => vec![IconPlatform::Windows],
        Some("linux") => vec![IconPlatform::Linux],
        Some("ios") => vec![IconPlatform::IOS],
        Some("android") => vec![IconPlatform::Android],
        Some("all") | None => vec![
            IconPlatform::Web,
            IconPlatform::MacOS,
            IconPlatform::Windows,
            IconPlatform::Linux,
            IconPlatform::IOS,
            IconPlatform::Android,
        ],
        Some(other) => anyhow::bail!("Unknown platform: {}", other),
    };

    let options = ExportOptions {
        output_dir: output_dir.clone(),
        format: ExportFormat::IconSet,
        platforms,
        include_assets: true,
        generate_docs: false,
    };

    let exporter = BrandExporter::new(&brand);
    exporter.export(options)
        .context("Failed to generate icons")?;

    println!("Generated icons in {:?}", output_dir);

    Ok(())
}

/// Show locked tokens in a brand pack
pub fn run_show_locks(brand_path: Option<&str>) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let manager = BrandManager::new(brand);
    let governance = manager.governance();

    println!("Locked Tokens for {}", manager.brand().identity.name);
    println!("{}", "=".repeat(50));

    let locked = governance.locked_tokens();
    if locked.is_empty() {
        println!("\nNo tokens are locked.");
    } else {
        println!("\nLocked tokens ({}):", locked.len());
        for token in locked {
            if let Some(lock) = governance.get_lock(token) {
                println!("  - {} ({:?})", token, lock.level);
                if let Some(ref reason) = lock.reason {
                    println!("    Reason: {}", reason);
                }
            }
        }
    }

    Ok(())
}

/// Create a white-label configuration
pub fn run_create_white_label(
    name: &str,
    brand_path: Option<&str>,
    output: Option<&str>,
) -> Result<()> {
    let brand_path = brand_path.unwrap_or("brand.toml");
    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("white-label.toml"));

    let brand = BrandPack::from_file(brand_path)
        .context(format!("Failed to load brand pack from {}", brand_path))?;

    let config = WhiteLabelConfig::from_brand(&brand);
    config.save(&output_path)
        .context("Failed to save white-label config")?;

    println!("Created white-label config: {:?}", output_path);
    println!("Base brand: {}", brand.identity.name);
    println!("\nEdit the config to customize your white-label deployment.");

    Ok(())
}

/// Print brand pack summary
fn print_brand_summary(brand: &BrandPack) {
    println!("Summary");
    println!("{}", "-".repeat(30));
    println!("Name: {}", brand.identity.name);
    if !brand.identity.display_name.is_empty() {
        println!("Display Name: {}", brand.identity.display_name);
    }
    if let Some(ref tagline) = brand.identity.tagline {
        println!("Tagline: {}", tagline);
    }

    println!("\nColors:");
    println!("  Primary: {}", brand.colors.primary.value);
    println!("  Secondary: {}", brand.colors.secondary.value);
    println!("  Accent: {}", brand.colors.accent.value);
    if !brand.colors.custom.is_empty() {
        println!("  Custom colors: {}", brand.colors.custom.len());
    }

    println!("\nTypography:");
    println!("  Primary font: {}", brand.typography.primary_family.name);
    if let Some(ref mono) = brand.typography.mono_family {
        println!("  Mono font: {}", mono.name);
    }

    let locked_count = [
        brand.colors.primary.locked,
        brand.colors.secondary.locked,
        brand.colors.accent.locked,
        brand.typography.primary_family.locked,
    ].iter().filter(|&&x| x).count();

    if locked_count > 0 {
        println!("\nGovernance: {} tokens locked", locked_count);
    }

    if brand.identity.primary_logo.is_some() {
        println!("\nAssets: Primary logo defined");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_brand_init() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test-brand.toml");

        // This would need the actual run_init to be modified for testing
        // For now, just test the BrandPack creation
        let brand = BrandPack::new("Test Brand");
        brand.save(&path).unwrap();

        assert!(path.exists());

        let loaded = BrandPack::from_file(&path).unwrap();
        assert_eq!(loaded.identity.name, "Test Brand");
    }
}
