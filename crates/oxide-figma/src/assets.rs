//! Asset Downloading
//!
//! Downloads and manages assets from Figma:
//! - Icons (SVG, PNG)
//! - Images (fills, backgrounds)
//! - Exports (component renders)

use crate::api::{FigmaClient, FigmaConfig};
use crate::error::{FigmaError, Result};
use crate::types::*;
use camino::{Utf8Path, Utf8PathBuf};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Write;
use tokio::sync::Semaphore;
use tracing::{debug, info, warn};

/// Asset downloader for Figma
#[derive(Debug)]
pub struct AssetDownloader {
    client: FigmaClient,
    config: DownloaderConfig,
}

/// Configuration for asset downloading
#[derive(Debug, Clone)]
pub struct DownloaderConfig {
    /// Output directory for assets
    pub output_dir: Utf8PathBuf,

    /// Export format for icons
    pub icon_format: ExportFormat,

    /// Export scale for icons
    pub icon_scale: f32,

    /// Export format for images
    pub image_format: ExportFormat,

    /// Export scale for images
    pub image_scale: f32,

    /// Maximum concurrent downloads
    pub max_concurrent: usize,

    /// Whether to organize by type
    pub organize_by_type: bool,

    /// Whether to skip existing files
    pub skip_existing: bool,

    /// Whether to optimize SVGs
    pub optimize_svg: bool,
}

impl Default for DownloaderConfig {
    fn default() -> Self {
        Self {
            output_dir: Utf8PathBuf::from("assets"),
            icon_format: ExportFormat::Svg,
            icon_scale: 1.0,
            image_format: ExportFormat::Png,
            image_scale: 2.0, // 2x for retina
            max_concurrent: 4,
            organize_by_type: true,
            skip_existing: true,
            optimize_svg: true,
        }
    }
}

/// Result of asset download
#[derive(Debug, Clone)]
pub struct DownloadResult {
    /// Successfully downloaded assets
    pub downloaded: Vec<DownloadedAsset>,

    /// Failed downloads
    pub failed: Vec<FailedDownload>,

    /// Skipped (already exist)
    pub skipped: Vec<String>,

    /// Total bytes downloaded
    pub total_bytes: u64,
}

/// A successfully downloaded asset
#[derive(Debug, Clone)]
pub struct DownloadedAsset {
    /// Figma node ID
    pub node_id: String,

    /// Asset name
    pub name: String,

    /// Output path
    pub path: Utf8PathBuf,

    /// File size in bytes
    pub size: u64,

    /// Asset type
    pub asset_type: AssetType,
}

/// Asset type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssetType {
    Icon,
    Image,
    Component,
    ImageFill,
}

/// A failed download
#[derive(Debug, Clone)]
pub struct FailedDownload {
    /// Figma node ID
    pub node_id: String,

    /// Asset name
    pub name: String,

    /// Error message
    pub error: String,
}

/// Asset to be downloaded
#[derive(Debug, Clone)]
struct AssetRequest {
    node_id: String,
    name: String,
    asset_type: AssetType,
    format: ExportFormat,
    scale: f32,
}

impl AssetDownloader {
    /// Create a new asset downloader
    pub fn new(client: FigmaClient) -> Self {
        Self::with_config(client, DownloaderConfig::default())
    }

    /// Create with custom config
    pub fn with_config(client: FigmaClient, config: DownloaderConfig) -> Self {
        Self { client, config }
    }

    /// Download all assets from a Figma file
    pub async fn download_all(&self, file_key: &str, file: &FigmaFile) -> Result<DownloadResult> {
        info!(file_key, "Downloading assets from Figma file");

        // Collect all assets to download
        let requests = self.collect_asset_requests(file);

        info!(total_assets = requests.len(), "Found assets to download");

        // Create output directories
        self.ensure_directories()?;

        // Download in batches
        let result = self.download_assets(file_key, requests).await?;

        info!(
            downloaded = result.downloaded.len(),
            failed = result.failed.len(),
            skipped = result.skipped.len(),
            total_bytes = result.total_bytes,
            "Asset download complete"
        );

        Ok(result)
    }

    /// Download specific nodes as assets
    pub async fn download_nodes(
        &self,
        file_key: &str,
        node_ids: &[&str],
        format: ExportFormat,
        scale: f32,
    ) -> Result<DownloadResult> {
        let requests: Vec<AssetRequest> = node_ids
            .iter()
            .map(|id| AssetRequest {
                node_id: id.to_string(),
                name: id.to_string(),
                asset_type: AssetType::Component,
                format,
                scale,
            })
            .collect();

        self.ensure_directories()?;
        self.download_assets(file_key, requests).await
    }

    /// Download image fills from a file
    pub async fn download_image_fills(&self, file_key: &str) -> Result<DownloadResult> {
        info!(file_key, "Downloading image fills");

        let image_fills = self.client.get_image_fills(file_key).await?;

        if image_fills.error {
            return Err(FigmaError::ApiError("Failed to get image fills".into()));
        }

        let mut downloaded = Vec::new();
        let mut failed = Vec::new();
        let mut total_bytes = 0u64;

        self.ensure_directories()?;

        for (image_ref, url) in &image_fills.images {
            let output_path = self.get_output_path(image_ref, AssetType::ImageFill, ExportFormat::Png);

            if self.config.skip_existing && output_path.exists() {
                continue;
            }

            match self.download_single_url(url, &output_path).await {
                Ok(size) => {
                    total_bytes += size;
                    downloaded.push(DownloadedAsset {
                        node_id: image_ref.clone(),
                        name: image_ref.clone(),
                        path: output_path,
                        size,
                        asset_type: AssetType::ImageFill,
                    });
                }
                Err(e) => {
                    warn!(image_ref, error = %e, "Failed to download image fill");
                    failed.push(FailedDownload {
                        node_id: image_ref.clone(),
                        name: image_ref.clone(),
                        error: e.to_string(),
                    });
                }
            }
        }

        Ok(DownloadResult {
            downloaded,
            failed,
            skipped: Vec::new(),
            total_bytes,
        })
    }

    /// Collect asset requests from file
    fn collect_asset_requests(&self, file: &FigmaFile) -> Vec<AssetRequest> {
        let mut requests = Vec::new();
        let mut seen_ids = HashSet::new();

        // Walk document tree
        for page in &file.document.children {
            self.collect_from_node(page, &mut requests, &mut seen_ids);
        }

        requests
    }

    /// Collect assets from a node recursively
    fn collect_from_node(
        &self,
        node: &Node,
        requests: &mut Vec<AssetRequest>,
        seen: &mut HashSet<String>,
    ) {
        if seen.contains(&node.id) {
            return;
        }
        seen.insert(node.id.clone());

        // Check if this is an icon
        if self.is_icon(node) {
            requests.push(AssetRequest {
                node_id: node.id.clone(),
                name: self.sanitize_name(&node.name),
                asset_type: AssetType::Icon,
                format: self.config.icon_format,
                scale: self.config.icon_scale,
            });
        }

        // Check if this is an exportable image
        if self.is_exportable_image(node) {
            requests.push(AssetRequest {
                node_id: node.id.clone(),
                name: self.sanitize_name(&node.name),
                asset_type: AssetType::Image,
                format: self.config.image_format,
                scale: self.config.image_scale,
            });
        }

        // Check for explicit exports
        for export in &node.export_settings {
            let format = export.format;
            let scale = export.constraint
                .as_ref()
                .map(|c| {
                    if c.constraint_type == ExportConstraintType::Scale {
                        c.value
                    } else {
                        1.0
                    }
                })
                .unwrap_or(1.0);

            requests.push(AssetRequest {
                node_id: node.id.clone(),
                name: format!("{}{}", self.sanitize_name(&node.name), export.suffix),
                asset_type: AssetType::Component,
                format,
                scale,
            });
        }

        // Recurse into children
        for child in &node.children {
            self.collect_from_node(child, requests, seen);
        }
    }

    /// Check if node is an icon
    fn is_icon(&self, node: &Node) -> bool {
        let name_lower = node.name.to_lowercase();

        // Name-based detection
        if name_lower.contains("icon")
            || name_lower.starts_with("ic_")
            || name_lower.starts_with("ic-")
        {
            return true;
        }

        // Type-based detection (vectors, boolean ops)
        match node.node_type {
            NodeType::Vector | NodeType::BooleanOperation => {
                // Check size (icons are typically small)
                if let Some(bbox) = &node.absolute_bounding_box {
                    if bbox.width <= 64.0 && bbox.height <= 64.0 {
                        return true;
                    }
                }
            }
            NodeType::Frame | NodeType::Component => {
                // Component icons
                if let Some(bbox) = &node.absolute_bounding_box {
                    if bbox.width <= 64.0 && bbox.height <= 64.0 {
                        // Check if mostly vectors
                        let vector_count = self.count_vectors(node);
                        if vector_count > 0 {
                            return true;
                        }
                    }
                }
            }
            _ => {}
        }

        false
    }

    /// Count vector children
    fn count_vectors(&self, node: &Node) -> usize {
        let mut count = 0;
        for child in &node.children {
            match child.node_type {
                NodeType::Vector | NodeType::BooleanOperation => count += 1,
                _ => count += self.count_vectors(child),
            }
        }
        count
    }

    /// Check if node is an exportable image
    fn is_exportable_image(&self, node: &Node) -> bool {
        let name_lower = node.name.to_lowercase();

        // Name-based detection
        if name_lower.contains("image")
            || name_lower.contains("photo")
            || name_lower.contains("illustration")
            || name_lower.contains("graphic")
        {
            // Check if it has image fills
            for fill in &node.fills {
                if fill.paint_type == PaintType::Image {
                    return true;
                }
            }
        }

        false
    }

    /// Download assets in batches
    async fn download_assets(
        &self,
        file_key: &str,
        requests: Vec<AssetRequest>,
    ) -> Result<DownloadResult> {
        let mut downloaded = Vec::new();
        let mut failed = Vec::new();
        let mut skipped = Vec::new();
        let mut total_bytes = 0u64;

        if requests.is_empty() {
            return Ok(DownloadResult {
                downloaded,
                failed,
                skipped,
                total_bytes,
            });
        }

        // Group by format for batch API calls
        let mut by_format: HashMap<(ExportFormat, u32), Vec<AssetRequest>> = HashMap::new();
        for req in requests {
            let key = (req.format, (req.scale * 100.0) as u32);
            by_format.entry(key).or_default().push(req);
        }

        // Process each format group
        for ((format, scale_int), requests) in by_format {
            let scale = scale_int as f32 / 100.0;

            // Check for skipped files
            let mut to_download: Vec<AssetRequest> = Vec::new();
            for req in requests {
                let path = self.get_output_path(&req.name, req.asset_type, req.format);
                if self.config.skip_existing && path.exists() {
                    skipped.push(req.name.clone());
                } else {
                    to_download.push(req);
                }
            }

            if to_download.is_empty() {
                continue;
            }

            // Get image URLs from Figma API
            let node_ids: Vec<&str> = to_download.iter().map(|r| r.node_id.as_str()).collect();

            // Batch in groups of 50 (Figma API limit)
            for chunk in node_ids.chunks(50) {
                let images_response = self.client
                    .get_images(file_key, chunk, format, scale)
                    .await?;

                if let Some(err) = images_response.err {
                    warn!(error = %err, "Figma images API error");
                    continue;
                }

                // Download each image
                let semaphore = std::sync::Arc::new(Semaphore::new(self.config.max_concurrent));

                for req in &to_download {
                    if let Some(Some(url)) = images_response.images.get(&req.node_id) {
                        let path = self.get_output_path(&req.name, req.asset_type, req.format);

                        let _permit = semaphore.acquire().await.unwrap();

                        match self.download_single_url(url, &path).await {
                            Ok(size) => {
                                total_bytes += size;
                                downloaded.push(DownloadedAsset {
                                    node_id: req.node_id.clone(),
                                    name: req.name.clone(),
                                    path,
                                    size,
                                    asset_type: req.asset_type,
                                });
                            }
                            Err(e) => {
                                failed.push(FailedDownload {
                                    node_id: req.node_id.clone(),
                                    name: req.name.clone(),
                                    error: e.to_string(),
                                });
                            }
                        }
                    } else {
                        failed.push(FailedDownload {
                            node_id: req.node_id.clone(),
                            name: req.name.clone(),
                            error: "No URL returned from Figma".into(),
                        });
                    }
                }
            }
        }

        Ok(DownloadResult {
            downloaded,
            failed,
            skipped,
            total_bytes,
        })
    }

    /// Download a single URL to a file
    async fn download_single_url(&self, url: &str, path: &Utf8Path) -> Result<u64> {
        debug!(url, ?path, "Downloading asset");

        let bytes = self.client.download_image(url).await?;

        // Optimize SVG if enabled
        let bytes = if self.config.optimize_svg && path.extension() == Some("svg") {
            self.optimize_svg(&bytes)?
        } else {
            bytes
        };

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // Write file
        let mut file = fs::File::create(path)?;
        file.write_all(&bytes)?;

        Ok(bytes.len() as u64)
    }

    /// Get output path for an asset
    fn get_output_path(&self, name: &str, asset_type: AssetType, format: ExportFormat) -> Utf8PathBuf {
        let extension = match format {
            ExportFormat::Svg => "svg",
            ExportFormat::Png => "png",
            ExportFormat::Jpg => "jpg",
            ExportFormat::Pdf => "pdf",
        };

        let subdir = if self.config.organize_by_type {
            match asset_type {
                AssetType::Icon => "icons",
                AssetType::Image => "images",
                AssetType::Component => "components",
                AssetType::ImageFill => "fills",
            }
        } else {
            ""
        };

        if subdir.is_empty() {
            self.config.output_dir.join(format!("{}.{}", name, extension))
        } else {
            self.config.output_dir.join(subdir).join(format!("{}.{}", name, extension))
        }
    }

    /// Ensure output directories exist
    fn ensure_directories(&self) -> Result<()> {
        fs::create_dir_all(&self.config.output_dir)?;

        if self.config.organize_by_type {
            fs::create_dir_all(self.config.output_dir.join("icons"))?;
            fs::create_dir_all(self.config.output_dir.join("images"))?;
            fs::create_dir_all(self.config.output_dir.join("components"))?;
            fs::create_dir_all(self.config.output_dir.join("fills"))?;
        }

        Ok(())
    }

    /// Sanitize filename
    fn sanitize_name(&self, name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '-' || c == '_' {
                    c.to_ascii_lowercase()
                } else if c == ' ' {
                    '-'
                } else {
                    '_'
                }
            })
            .collect::<String>()
            .trim_matches('_')
            .trim_matches('-')
            .to_string()
    }

    /// Optimize SVG content (basic optimization)
    fn optimize_svg(&self, bytes: &[u8]) -> Result<Vec<u8>> {
        // For production, integrate with SVGO or similar
        // For now, just do basic cleanup
        let svg_str = String::from_utf8_lossy(bytes);

        // Remove unnecessary whitespace
        let optimized = svg_str
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n");

        Ok(optimized.into_bytes())
    }
}

/// Builder for asset downloads
#[derive(Debug)]
pub struct AssetDownloadBuilder {
    config: DownloaderConfig,
    node_ids: Vec<String>,
    file_key: Option<String>,
}

impl AssetDownloadBuilder {
    /// Create new builder
    pub fn new() -> Self {
        Self {
            config: DownloaderConfig::default(),
            node_ids: Vec::new(),
            file_key: None,
        }
    }

    /// Set output directory
    pub fn output_dir(mut self, dir: impl Into<Utf8PathBuf>) -> Self {
        self.config.output_dir = dir.into();
        self
    }

    /// Set icon format
    pub fn icon_format(mut self, format: ExportFormat) -> Self {
        self.config.icon_format = format;
        self
    }

    /// Set icon scale
    pub fn icon_scale(mut self, scale: f32) -> Self {
        self.config.icon_scale = scale;
        self
    }

    /// Set image format
    pub fn image_format(mut self, format: ExportFormat) -> Self {
        self.config.image_format = format;
        self
    }

    /// Set image scale
    pub fn image_scale(mut self, scale: f32) -> Self {
        self.config.image_scale = scale;
        self
    }

    /// Set max concurrent downloads
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.config.max_concurrent = max;
        self
    }

    /// Set organize by type
    pub fn organize_by_type(mut self, organize: bool) -> Self {
        self.config.organize_by_type = organize;
        self
    }

    /// Set skip existing
    pub fn skip_existing(mut self, skip: bool) -> Self {
        self.config.skip_existing = skip;
        self
    }

    /// Add specific node IDs
    pub fn nodes(mut self, ids: Vec<String>) -> Self {
        self.node_ids = ids;
        self
    }

    /// Set file key
    pub fn file_key(mut self, key: impl Into<String>) -> Self {
        self.file_key = Some(key.into());
        self
    }

    /// Build the downloader
    pub fn build(self, client: FigmaClient) -> AssetDownloader {
        AssetDownloader::with_config(client, self.config)
    }

    /// Get the config
    pub fn config(&self) -> &DownloaderConfig {
        &self.config
    }
}

impl Default for AssetDownloadBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_name() {
        let config = FigmaConfig::with_token("test");
        let client = FigmaClient::new(config);
        let downloader = AssetDownloader::new(client);

        assert_eq!(downloader.sanitize_name("My Icon"), "my-icon");
        assert_eq!(downloader.sanitize_name("icon_name"), "icon_name");
        assert_eq!(downloader.sanitize_name("Icon/Category"), "icon_category");
    }

    #[test]
    fn test_get_output_path() {
        let config = FigmaConfig::with_token("test");
        let client = FigmaClient::new(config);
        let downloader = AssetDownloader::new(client);

        let path = downloader.get_output_path("my-icon", AssetType::Icon, ExportFormat::Svg);
        assert!(path.to_string().contains("icons"));
        assert!(path.to_string().ends_with(".svg"));
    }

    #[test]
    fn test_builder() {
        let builder = AssetDownloadBuilder::new()
            .output_dir("custom/assets")
            .icon_format(ExportFormat::Png)
            .icon_scale(2.0)
            .max_concurrent(8);

        assert_eq!(builder.config().output_dir.as_str(), "custom/assets");
        assert_eq!(builder.config().icon_format, ExportFormat::Png);
        assert_eq!(builder.config().icon_scale, 2.0);
        assert_eq!(builder.config().max_concurrent, 8);
    }
}
