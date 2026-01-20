//! Figma CLI commands
//!
//! Provides CLI interface for Figma translation:
//! - `oxide figma pull` - Pull and translate a Figma file
//! - `oxide figma sync` - Continuous sync with Figma
//! - `oxide figma diff` - Show changes from Figma
//! - `oxide figma import` - Import specific nodes

use anyhow::{Context, Result};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{info, warn};

/// Pull (translate) a Figma file to OxideKit
pub fn run_pull(
    url: &str,
    output: Option<&str>,
    dark: bool,
    name: Option<&str>,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        // Parse URL
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Pulling Figma file: {}", url_info.file_key);

        // Create client
        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        // Fetch file
        println!("Fetching file from Figma...");
        let file = client.get_file(&url_info.file_key).await
            .context("Failed to fetch Figma file")?;

        println!("File: {} (version {})", file.name, file.version);

        // Translate
        println!("Translating to OxideKit...");
        let translator = oxide_figma::prelude::TranslatorBuilder::new()
            .theme_name(name.unwrap_or(&file.name).to_string())
            .is_dark(dark)
            .build();

        let result = translator.translate(&file)
            .context("Translation failed")?;

        // Export
        let output_dir = output.unwrap_or("design");
        println!("Exporting to {}...", output_dir);
        let files = translator.export(&result, camino::Utf8Path::new(output_dir))
            .context("Export failed")?;

        println!("\nGenerated files:");
        for file in &files {
            println!("  - {}", file);
        }

        println!("\nTranslation summary:");
        println!("  Colors: {}", result.metadata.stats.colors_extracted);
        println!("  Spacing values: {}", result.metadata.stats.spacing_values);
        println!("  Typography styles: {}", result.metadata.stats.typography_styles);
        println!("  Components mapped: {}", result.metadata.stats.components_mapped);
        println!("  Layouts translated: {}", result.metadata.stats.layouts_translated);

        if result.metadata.stats.warnings_count > 0 {
            println!("\nWarnings ({}):", result.metadata.stats.warnings_count);
            for warning in &result.warnings {
                println!("  - {}", warning);
            }
        }

        if let Some(validation) = &result.validation {
            println!("\nValidation:");
            println!("  Accessibility score: {}/100", validation.accessibility_score);
            println!("  Performance score: {}/100", validation.performance_score);

            if !validation.errors.is_empty() {
                println!("  Errors: {}", validation.errors.len());
            }
            if !validation.warnings.is_empty() {
                println!("  Warnings: {}", validation.warnings.len());
            }
        }

        println!("\nDone! Theme exported to {}/theme.generated.toml", output_dir);

        Ok(())
    })
}

/// Start continuous sync with Figma
pub fn run_sync(
    url: &str,
    interval_secs: u64,
    auto_apply: bool,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Starting continuous sync with Figma file: {}", url_info.file_key);
        println!("Poll interval: {} seconds", interval_secs);
        println!("Auto-apply safe changes: {}", auto_apply);
        println!("\nPress Ctrl+C to stop.\n");

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        let sync_config = oxide_figma::prelude::SyncConfigBuilder::new()
            .file_key(&url_info.file_key)
            .project_dir(".")
            .output_dir("design")
            .poll_interval(Duration::from_secs(interval_secs))
            .auto_apply_safe(auto_apply)
            .build();

        let mut engine = oxide_figma::prelude::SyncEngine::new(client, sync_config);

        // Initial sync
        println!("Performing initial sync...");
        let result = engine.sync().await
            .context("Initial sync failed")?;

        if result.has_changes {
            println!("Initial sync found changes:");
            for file in &result.files_updated {
                println!("  Updated: {}", file);
            }
        } else {
            println!("Initial sync: No changes detected.");
        }

        // Start continuous sync
        println!("\nWatching for changes...");
        engine.start_continuous().await?;

        Ok(())
    })
}

/// Show diff between Figma and local files
pub fn run_diff(url: &str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Checking for changes from Figma...\n");

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        // Fetch current file
        let file = client.get_file(&url_info.file_key).await
            .context("Failed to fetch Figma file")?;

        // Extract tokens
        let extractor = oxide_figma::prelude::TokenExtractor::new();
        let tokens = extractor.extract(&file)
            .context("Failed to extract tokens")?;

        // Load existing theme
        let theme_path = "design/theme.generated.toml";
        if !std::path::Path::new(theme_path).exists() {
            println!("No existing theme found at {}.", theme_path);
            println!("Run 'oxide figma pull <url>' first to create one.");
            return Ok(());
        }

        let theme_content = std::fs::read_to_string(theme_path)
            .context("Failed to read existing theme")?;
        let existing_theme = oxide_components::theme::Theme::from_toml(&theme_content)
            .context("Failed to parse existing theme")?;

        // Compare
        let diff_engine = oxide_figma::prelude::DesignDiff::new();
        let diff = diff_engine.compare_tokens(&tokens, &existing_theme);

        if !diff.has_changes() {
            println!("No changes detected. Figma file matches local theme.");
            return Ok(());
        }

        println!("{}", diff.to_summary_string());

        if diff.has_breaking_changes() {
            println!("\n[!] Breaking changes detected. Review carefully before applying.");
        }

        println!("\nTo apply changes, run: oxide figma pull {}", url);

        Ok(())
    })
}

/// Import specific elements from Figma
pub fn run_import(
    url: &str,
    output: Option<&str>,
    format: &str,
    scale: f32,
) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Importing from Figma...");

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        // Determine export format
        let export_format = match format.to_lowercase().as_str() {
            "svg" => oxide_figma::types::ExportFormat::Svg,
            "png" => oxide_figma::types::ExportFormat::Png,
            "jpg" | "jpeg" => oxide_figma::types::ExportFormat::Jpg,
            "pdf" => oxide_figma::types::ExportFormat::Pdf,
            _ => {
                return Err(anyhow::anyhow!("Unsupported format: {}. Use svg, png, jpg, or pdf.", format));
            }
        };

        // Fetch file
        let file = client.get_file(&url_info.file_key).await
            .context("Failed to fetch Figma file")?;

        // Create asset downloader
        let output_dir = output.unwrap_or("assets");
        let downloader = oxide_figma::prelude::AssetDownloadBuilder::new()
            .output_dir(output_dir)
            .icon_format(export_format)
            .icon_scale(scale)
            .image_format(export_format)
            .image_scale(scale)
            .build(client);

        // Download assets
        println!("Downloading assets...");
        let result = downloader.download_all(&url_info.file_key, &file).await
            .context("Failed to download assets")?;

        println!("\nDownload complete:");
        println!("  Downloaded: {} files ({} bytes)", result.downloaded.len(), result.total_bytes);
        println!("  Skipped: {} files (already exist)", result.skipped.len());
        println!("  Failed: {} files", result.failed.len());

        if !result.failed.is_empty() {
            println!("\nFailed downloads:");
            for failed in &result.failed {
                println!("  - {}: {}", failed.name, failed.error);
            }
        }

        if !result.downloaded.is_empty() {
            println!("\nDownloaded assets:");
            for asset in result.downloaded.iter().take(10) {
                println!("  - {}", asset.path);
            }
            if result.downloaded.len() > 10 {
                println!("  ... and {} more", result.downloaded.len() - 10);
            }
        }

        Ok(())
    })
}

/// Download image fills
pub fn run_download_images(url: &str, output: Option<&str>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Downloading image fills from Figma...");

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        let output_dir = output.unwrap_or("assets");
        let downloader = oxide_figma::prelude::AssetDownloadBuilder::new()
            .output_dir(output_dir)
            .build(client);

        let result = downloader.download_image_fills(&url_info.file_key).await
            .context("Failed to download image fills")?;

        println!("\nDownload complete:");
        println!("  Downloaded: {} images", result.downloaded.len());

        Ok(())
    })
}

/// Get Figma file info
pub fn run_info(url: &str) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        println!("Fetching Figma file info...\n");

        let file = client.get_file(&url_info.file_key).await
            .context("Failed to fetch Figma file")?;

        println!("File: {}", file.name);
        println!("Version: {}", file.version);
        println!("Last modified: {}", file.last_modified);
        println!("Schema version: {}", file.schema_version);

        if !file.thumbnail_url.is_empty() {
            println!("Thumbnail: {}", file.thumbnail_url);
        }

        println!("\nPages ({}):", file.document.children.len());
        for page in &file.document.children {
            println!("  - {} (ID: {})", page.name, page.id);
            println!("    Frames: {}", page.children.len());
        }

        println!("\nComponents: {}", file.components.len());
        println!("Component sets: {}", file.component_sets.len());
        println!("Styles: {}", file.styles.len());

        if !file.styles.is_empty() {
            println!("\nStyles by type:");
            let mut style_counts = std::collections::HashMap::new();
            for style in file.styles.values() {
                *style_counts.entry(format!("{:?}", style.style_type)).or_insert(0) += 1;
            }
            for (style_type, count) in style_counts {
                println!("  {}: {}", style_type, count);
            }
        }

        Ok(())
    })
}

/// Export design pack from Figma
pub fn run_export_design(url: &str, output: Option<&str>) -> Result<()> {
    let rt = tokio::runtime::Runtime::new()?;

    rt.block_on(async {
        let url_info = oxide_figma::FigmaClient::parse_url(url)
            .context("Failed to parse Figma URL")?;

        println!("Exporting design pack from Figma...");

        let config = oxide_figma::FigmaConfig::from_env()
            .context("FIGMA_TOKEN environment variable not set")?;
        let client = oxide_figma::FigmaClient::new(config);

        // Fetch file
        let file = client.get_file(&url_info.file_key).await
            .context("Failed to fetch Figma file")?;

        // Translate
        let translator = oxide_figma::prelude::TranslatorBuilder::new()
            .theme_name(&file.name)
            .is_dark(true)
            .build();

        let result = translator.translate(&file)
            .context("Translation failed")?;

        // Create design pack directory
        let default_output = format!("design.{}", to_kebab_case(&file.name));
        let output_dir = output.unwrap_or(&default_output);
        std::fs::create_dir_all(output_dir)?;

        // Export all files
        translator.export(&result, camino::Utf8Path::new(output_dir))?;

        // Download assets
        let downloader = oxide_figma::prelude::AssetDownloadBuilder::new()
            .output_dir(format!("{}/assets", output_dir))
            .build(client);

        let assets = downloader.download_all(&url_info.file_key, &file).await
            .context("Failed to download assets")?;

        println!("\nDesign pack exported to: {}", output_dir);
        println!("  Theme: theme.generated.toml");
        println!("  Typography: typography.generated.toml");
        println!("  Components: components.generated.json");
        println!("  Layouts: layouts.generated.json");
        println!("  Assets: {} files", assets.downloaded.len());

        Ok(())
    })
}

/// Convert string to kebab-case
fn to_kebab_case(s: &str) -> String {
    s.chars()
        .map(|c| {
            if c.is_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}
