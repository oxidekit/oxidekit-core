//! Brand Asset Management
//!
//! Handles brand assets like logos, icons, and other visual elements.
//! Provides an asset pipeline for processing and generating icon sets.

use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::error::{BrandingError, BrandingResult};

/// A brand asset (logo, icon, image, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BrandAsset {
    /// Unique asset identifier
    pub id: String,

    /// Asset display name
    pub name: String,

    /// Asset type
    #[serde(rename = "type")]
    pub asset_type: AssetType,

    /// Path to the asset file (relative to brand pack root)
    pub path: PathBuf,

    /// SHA256 hash for integrity verification
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hash: Option<String>,

    /// Asset format (svg, png, etc.)
    pub format: AssetFormat,

    /// Asset variants (different sizes, colors, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<AssetVariant>,

    /// Usage guidelines
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidelines: Option<AssetGuidelines>,

    /// Metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

impl BrandAsset {
    /// Create a new brand asset
    pub fn new(id: impl Into<String>, name: impl Into<String>, path: impl Into<PathBuf>) -> Self {
        let path = path.into();
        let format = AssetFormat::from_path(&path);

        Self {
            id: id.into(),
            name: name.into(),
            asset_type: AssetType::Image,
            path,
            hash: None,
            format,
            variants: Vec::new(),
            guidelines: None,
            metadata: HashMap::new(),
        }
    }

    /// Set the asset type
    pub fn with_type(mut self, asset_type: AssetType) -> Self {
        self.asset_type = asset_type;
        self
    }

    /// Add a variant
    pub fn with_variant(mut self, variant: AssetVariant) -> Self {
        self.variants.push(variant);
        self
    }

    /// Set usage guidelines
    pub fn with_guidelines(mut self, guidelines: AssetGuidelines) -> Self {
        self.guidelines = Some(guidelines);
        self
    }

    /// Compute hash from file contents
    pub fn compute_hash(&mut self, content: &[u8]) {
        let mut hasher = Sha256::new();
        hasher.update(content);
        self.hash = Some(hex::encode(hasher.finalize()));
    }

    /// Verify asset integrity
    pub fn verify_integrity(&self, content: &[u8]) -> BrandingResult<()> {
        if let Some(expected) = &self.hash {
            let mut hasher = Sha256::new();
            hasher.update(content);
            let actual = hex::encode(hasher.finalize());

            if &actual != expected {
                return Err(BrandingError::IntegrityError {
                    path: self.path.display().to_string(),
                    expected: expected.clone(),
                    actual,
                });
            }
        }
        Ok(())
    }
}

/// Asset type categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    /// Primary logo
    Logo,
    /// Application icon
    Icon,
    /// Favicon
    Favicon,
    /// General image
    Image,
    /// Background pattern
    Pattern,
    /// Illustration
    Illustration,
    /// Font file
    Font,
    /// Video asset
    Video,
    /// Audio asset (jingles, etc.)
    Audio,
    /// Document template
    Template,
}

/// Asset format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AssetFormat {
    /// SVG vector format (preferred)
    Svg,
    /// PNG raster format
    #[default]
    Png,
    /// JPEG format
    Jpeg,
    /// WebP format
    Webp,
    /// ICO format (for favicons)
    Ico,
    /// ICNS format (macOS icons)
    Icns,
    /// PDF format
    Pdf,
    /// Font formats
    Woff,
    Woff2,
    Ttf,
    Otf,
    /// Video formats
    Mp4,
    Webm,
    /// Audio formats
    Mp3,
    Wav,
    /// Unknown format
    Unknown,
}

impl AssetFormat {
    /// Detect format from file path
    pub fn from_path(path: &Path) -> Self {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| match ext.to_lowercase().as_str() {
                "svg" => AssetFormat::Svg,
                "png" => AssetFormat::Png,
                "jpg" | "jpeg" => AssetFormat::Jpeg,
                "webp" => AssetFormat::Webp,
                "ico" => AssetFormat::Ico,
                "icns" => AssetFormat::Icns,
                "pdf" => AssetFormat::Pdf,
                "woff" => AssetFormat::Woff,
                "woff2" => AssetFormat::Woff2,
                "ttf" => AssetFormat::Ttf,
                "otf" => AssetFormat::Otf,
                "mp4" => AssetFormat::Mp4,
                "webm" => AssetFormat::Webm,
                "mp3" => AssetFormat::Mp3,
                "wav" => AssetFormat::Wav,
                _ => AssetFormat::Unknown,
            })
            .unwrap_or(AssetFormat::Unknown)
    }

    /// Get the file extension for this format
    pub fn extension(&self) -> &'static str {
        match self {
            AssetFormat::Svg => "svg",
            AssetFormat::Png => "png",
            AssetFormat::Jpeg => "jpg",
            AssetFormat::Webp => "webp",
            AssetFormat::Ico => "ico",
            AssetFormat::Icns => "icns",
            AssetFormat::Pdf => "pdf",
            AssetFormat::Woff => "woff",
            AssetFormat::Woff2 => "woff2",
            AssetFormat::Ttf => "ttf",
            AssetFormat::Otf => "otf",
            AssetFormat::Mp4 => "mp4",
            AssetFormat::Webm => "webm",
            AssetFormat::Mp3 => "mp3",
            AssetFormat::Wav => "wav",
            AssetFormat::Unknown => "bin",
        }
    }
}

/// Asset variant (size, color, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetVariant {
    /// Variant name (e.g., "dark", "small", "monochrome")
    pub name: String,

    /// Path to variant file
    pub path: PathBuf,

    /// Variant-specific properties
    #[serde(flatten)]
    pub properties: VariantProperties,
}

/// Variant-specific properties
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VariantProperties {
    /// Width in pixels (for raster variants)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Height in pixels (for raster variants)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Scale factor (e.g., 2x, 3x)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f32>,

    /// Color mode
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mode: Option<ColorMode>,

    /// Theme (light/dark)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub theme: Option<ThemeMode>,
}

/// Color mode for asset variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ColorMode {
    Full,
    Monochrome,
    Grayscale,
    Knockout,
}

/// Theme mode for asset variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThemeMode {
    Light,
    Dark,
}

/// Asset usage guidelines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetGuidelines {
    /// Minimum size (in pixels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_size: Option<u32>,

    /// Required clear space (as percentage of asset size)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clear_space: Option<f32>,

    /// Allowed backgrounds
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_backgrounds: Vec<String>,

    /// Prohibited modifications
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prohibited: Vec<String>,

    /// Usage notes
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Default for AssetGuidelines {
    fn default() -> Self {
        Self {
            min_size: Some(32),
            clear_space: Some(0.1),
            allowed_backgrounds: vec![],
            prohibited: vec![
                "Do not stretch or distort".into(),
                "Do not change colors".into(),
                "Do not add effects".into(),
            ],
            notes: None,
        }
    }
}

/// Standard icon sizes for different platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct IconSize {
    /// Size in pixels (width = height)
    pub size: u32,

    /// Scale factor (1x, 2x, 3x)
    pub scale: u32,

    /// Platform this size is for
    pub platform: IconPlatform,
}

impl IconSize {
    pub const fn new(size: u32, scale: u32, platform: IconPlatform) -> Self {
        Self { size, scale, platform }
    }

    /// Get all standard sizes for a platform
    pub fn standard_sizes(platform: IconPlatform) -> Vec<Self> {
        match platform {
            IconPlatform::Web => vec![
                Self::new(16, 1, platform),
                Self::new(32, 1, platform),
                Self::new(48, 1, platform),
                Self::new(64, 1, platform),
                Self::new(128, 1, platform),
                Self::new(192, 1, platform),
                Self::new(256, 1, platform),
                Self::new(512, 1, platform),
            ],
            IconPlatform::MacOS => vec![
                Self::new(16, 1, platform),
                Self::new(16, 2, platform),
                Self::new(32, 1, platform),
                Self::new(32, 2, platform),
                Self::new(128, 1, platform),
                Self::new(128, 2, platform),
                Self::new(256, 1, platform),
                Self::new(256, 2, platform),
                Self::new(512, 1, platform),
                Self::new(512, 2, platform),
            ],
            IconPlatform::Windows => vec![
                Self::new(16, 1, platform),
                Self::new(20, 1, platform),
                Self::new(24, 1, platform),
                Self::new(32, 1, platform),
                Self::new(40, 1, platform),
                Self::new(48, 1, platform),
                Self::new(64, 1, platform),
                Self::new(256, 1, platform),
            ],
            IconPlatform::Linux => vec![
                Self::new(16, 1, platform),
                Self::new(24, 1, platform),
                Self::new(32, 1, platform),
                Self::new(48, 1, platform),
                Self::new(64, 1, platform),
                Self::new(128, 1, platform),
                Self::new(256, 1, platform),
                Self::new(512, 1, platform),
            ],
            IconPlatform::IOS => vec![
                Self::new(20, 1, platform),
                Self::new(20, 2, platform),
                Self::new(20, 3, platform),
                Self::new(29, 1, platform),
                Self::new(29, 2, platform),
                Self::new(29, 3, platform),
                Self::new(40, 1, platform),
                Self::new(40, 2, platform),
                Self::new(40, 3, platform),
                Self::new(60, 2, platform),
                Self::new(60, 3, platform),
                Self::new(76, 1, platform),
                Self::new(76, 2, platform),
                Self::new(83, 2, platform),
                Self::new(1024, 1, platform),
            ],
            IconPlatform::Android => vec![
                Self::new(48, 1, platform),  // mdpi
                Self::new(72, 1, platform),  // hdpi
                Self::new(96, 1, platform),  // xhdpi
                Self::new(144, 1, platform), // xxhdpi
                Self::new(192, 1, platform), // xxxhdpi
                Self::new(512, 1, platform), // Play Store
            ],
        }
    }

    /// Get the actual pixel size (size * scale)
    pub fn pixel_size(&self) -> u32 {
        self.size * self.scale
    }
}

/// Platform for icon generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IconPlatform {
    Web,
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
}

/// Asset processing pipeline
#[derive(Debug, Default)]
pub struct AssetPipeline {
    /// Pipeline configuration
    pub config: PipelineConfig,
}

/// Pipeline configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineConfig {
    /// Output formats for generated assets
    #[serde(default)]
    pub output_formats: Vec<AssetFormat>,

    /// Quality for lossy compression (1-100)
    #[serde(default = "default_quality")]
    pub quality: u8,

    /// Whether to optimize output
    #[serde(default = "default_true")]
    pub optimize: bool,

    /// Generate retina (2x) variants
    #[serde(default = "default_true")]
    pub generate_retina: bool,

    /// Target platforms
    #[serde(default)]
    pub platforms: Vec<IconPlatform>,
}

fn default_quality() -> u8 {
    90
}

fn default_true() -> bool {
    true
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            output_formats: vec![AssetFormat::Png, AssetFormat::Webp],
            quality: 90,
            optimize: true,
            generate_retina: true,
            platforms: vec![IconPlatform::Web, IconPlatform::MacOS, IconPlatform::Windows],
        }
    }
}

impl AssetPipeline {
    /// Create a new asset pipeline with default config
    pub fn new() -> Self {
        Self {
            config: PipelineConfig::default(),
        }
    }

    /// Create a pipeline with custom config
    pub fn with_config(config: PipelineConfig) -> Self {
        Self { config }
    }

    /// Generate icon set from a source image
    #[cfg(feature = "image-processing")]
    pub fn generate_icon_set(
        &self,
        source: &Path,
        output_dir: &Path,
    ) -> BrandingResult<Vec<PathBuf>> {
        use image::GenericImageView;

        tracing::info!("Generating icon set from {:?}", source);

        // Load source image
        let img = image::open(source)?;
        let (width, height) = img.dimensions();

        if width != height {
            tracing::warn!("Source image is not square ({}x{}), icons may be distorted", width, height);
        }

        std::fs::create_dir_all(output_dir)?;

        let mut generated = Vec::new();

        for platform in &self.config.platforms {
            let sizes = IconSize::standard_sizes(*platform);

            for icon_size in sizes {
                let size = icon_size.pixel_size();
                let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);

                for format in &self.config.output_formats {
                    let filename = format!(
                        "icon-{}x{}@{}x.{}",
                        icon_size.size,
                        icon_size.size,
                        icon_size.scale,
                        format.extension()
                    );
                    let output_path = output_dir.join(&filename);

                    match format {
                        AssetFormat::Png => {
                            resized.save_with_format(&output_path, image::ImageFormat::Png)?;
                        }
                        AssetFormat::Webp => {
                            resized.save_with_format(&output_path, image::ImageFormat::WebP)?;
                        }
                        AssetFormat::Jpeg => {
                            resized.save_with_format(&output_path, image::ImageFormat::Jpeg)?;
                        }
                        _ => continue,
                    }

                    generated.push(output_path);
                }
            }
        }

        tracing::info!("Generated {} icon files", generated.len());
        Ok(generated)
    }

    /// Generate icon set (stub for non-image-processing builds)
    #[cfg(not(feature = "image-processing"))]
    pub fn generate_icon_set(
        &self,
        _source: &Path,
        _output_dir: &Path,
    ) -> BrandingResult<Vec<PathBuf>> {
        Err(BrandingError::PipelineError(
            "Image processing feature not enabled. Enable 'image-processing' feature.".into()
        ))
    }

    /// Generate favicon files (ICO + PNG)
    #[cfg(feature = "image-processing")]
    pub fn generate_favicons(
        &self,
        source: &Path,
        output_dir: &Path,
    ) -> BrandingResult<Vec<PathBuf>> {
        tracing::info!("Generating favicons from {:?}", source);

        let img = image::open(source)?;
        std::fs::create_dir_all(output_dir)?;

        let mut generated = Vec::new();

        // Generate PNG favicons
        let favicon_sizes = [16, 32, 48, 64, 128, 180, 192, 512];
        for size in favicon_sizes {
            let resized = img.resize_exact(size, size, image::imageops::FilterType::Lanczos3);
            let path = output_dir.join(format!("favicon-{}x{}.png", size, size));
            resized.save_with_format(&path, image::ImageFormat::Png)?;
            generated.push(path);
        }

        // Generate apple-touch-icon
        let apple_icon = img.resize_exact(180, 180, image::imageops::FilterType::Lanczos3);
        let apple_path = output_dir.join("apple-touch-icon.png");
        apple_icon.save_with_format(&apple_path, image::ImageFormat::Png)?;
        generated.push(apple_path);

        tracing::info!("Generated {} favicon files", generated.len());
        Ok(generated)
    }

    /// Generate favicons (stub for non-image-processing builds)
    #[cfg(not(feature = "image-processing"))]
    pub fn generate_favicons(
        &self,
        _source: &Path,
        _output_dir: &Path,
    ) -> BrandingResult<Vec<PathBuf>> {
        Err(BrandingError::PipelineError(
            "Image processing feature not enabled. Enable 'image-processing' feature.".into()
        ))
    }

    /// Verify all assets in a brand pack
    pub fn verify_assets(&self, assets: &[BrandAsset], base_path: &Path) -> BrandingResult<()> {
        for asset in assets {
            let full_path = base_path.join(&asset.path);

            if !full_path.exists() {
                return Err(BrandingError::MissingAsset(
                    asset.path.display().to_string()
                ));
            }

            if asset.hash.is_some() {
                let content = std::fs::read(&full_path)?;
                asset.verify_integrity(&content)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_format_detection() {
        assert_eq!(AssetFormat::from_path(Path::new("logo.svg")), AssetFormat::Svg);
        assert_eq!(AssetFormat::from_path(Path::new("icon.png")), AssetFormat::Png);
        assert_eq!(AssetFormat::from_path(Path::new("photo.JPEG")), AssetFormat::Jpeg);
    }

    #[test]
    fn test_asset_hash() {
        let mut asset = BrandAsset::new("logo", "Logo", "logo.svg");
        asset.compute_hash(b"test content");
        assert!(asset.hash.is_some());
        assert!(asset.verify_integrity(b"test content").is_ok());
        assert!(asset.verify_integrity(b"wrong content").is_err());
    }

    #[test]
    fn test_icon_sizes() {
        let web_sizes = IconSize::standard_sizes(IconPlatform::Web);
        assert!(!web_sizes.is_empty());

        let macos_sizes = IconSize::standard_sizes(IconPlatform::MacOS);
        assert!(macos_sizes.iter().any(|s| s.scale == 2));
    }
}
