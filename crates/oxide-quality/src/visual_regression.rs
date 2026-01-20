//! Visual Regression Testing Module
//!
//! Provides comprehensive visual regression testing capabilities for OxideKit applications.
//!
//! # Features
//!
//! - **Screenshot Capture**: Capture component and full-page screenshots
//! - **Image Comparison**: Multiple comparison algorithms (pixel diff, perceptual hash, SSIM)
//! - **Baseline Management**: Store, update, and version control baselines
//! - **Report Generation**: HTML reports with side-by-side diffs and overlays
//! - **CI Integration**: GitHub Actions and GitLab CI helpers
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_quality::visual_regression::{run_visual_regression, VisualRegressionConfig};
//! use std::path::Path;
//!
//! let config = VisualRegressionConfig::default();
//! let report = run_visual_regression(Path::new("./my-project"), &config)?;
//!
//! if !report.passed {
//!     println!("Visual regressions detected: {}", report.failed_count);
//!     // Generate HTML report
//!     generate_visual_report(&report, Path::new("./reports"))?;
//! }
//! ```

use crate::{
    QualityError, QualityResult,
    VisualRegressionConfig, VisualThresholds, ComparisonMethod,
    IgnoreRegion, CaptureSettings, Viewport,
};
use image::{DynamicImage, GenericImageView, ImageBuffer, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::Write;

// ============================================================================
// Types and Structures
// ============================================================================

/// Visual regression test report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualRegressionReport {
    /// Project path
    pub project: PathBuf,
    /// Overall pass/fail status
    pub passed: bool,
    /// Total number of comparisons
    pub total_count: usize,
    /// Number of passed comparisons
    pub passed_count: usize,
    /// Number of failed comparisons
    pub failed_count: usize,
    /// Number of new screenshots (no baseline)
    pub new_count: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Individual comparison results
    pub results: Vec<VisualRegressionResult>,
    /// Report generation timestamp
    pub timestamp: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Comparison method used
    pub comparison_method: String,
    /// Configuration snapshot
    pub config_snapshot: ConfigSnapshot,
}

impl VisualRegressionReport {
    /// Create a new empty report
    pub fn new(project: &Path, config: &VisualRegressionConfig) -> Self {
        Self {
            project: project.to_path_buf(),
            passed: true,
            total_count: 0,
            passed_count: 0,
            failed_count: 0,
            new_count: 0,
            warning_count: 0,
            results: Vec::new(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            duration_ms: 0,
            comparison_method: config.comparison_method.to_string(),
            config_snapshot: ConfigSnapshot::from_config(config),
        }
    }

    /// Add a comparison result
    pub fn add_result(&mut self, result: VisualRegressionResult) {
        self.total_count += 1;
        match result.status {
            ComparisonStatus::Passed => self.passed_count += 1,
            ComparisonStatus::Failed => {
                self.failed_count += 1;
                self.passed = false;
            }
            ComparisonStatus::New => self.new_count += 1,
            ComparisonStatus::Warning => {
                self.warning_count += 1;
                self.passed_count += 1; // Warnings still pass
            }
        }
        self.results.push(result);
    }

    /// Export to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get failing results only
    pub fn failures(&self) -> Vec<&VisualRegressionResult> {
        self.results.iter().filter(|r| r.status == ComparisonStatus::Failed).collect()
    }
}

/// Configuration snapshot for reproducibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigSnapshot {
    pub pixel_threshold_percent: f64,
    pub color_threshold: u8,
    pub hash_distance_threshold: u32,
    pub ssim_threshold: f64,
    pub comparison_method: String,
    pub anti_aliasing_tolerance: u32,
}

impl ConfigSnapshot {
    fn from_config(config: &VisualRegressionConfig) -> Self {
        Self {
            pixel_threshold_percent: config.thresholds.pixel_threshold_percent,
            color_threshold: config.thresholds.color_threshold,
            hash_distance_threshold: config.thresholds.hash_distance_threshold,
            ssim_threshold: config.thresholds.ssim_threshold,
            comparison_method: config.comparison_method.to_string(),
            anti_aliasing_tolerance: config.anti_aliasing_tolerance,
        }
    }
}

/// Result of a single visual comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualRegressionResult {
    /// Screenshot/component name
    pub name: String,
    /// Viewport used
    pub viewport: String,
    /// Comparison status
    pub status: ComparisonStatus,
    /// Detailed comparison statistics
    pub stats: Option<DiffStats>,
    /// Path to baseline image
    pub baseline_path: Option<PathBuf>,
    /// Path to actual image
    pub actual_path: Option<PathBuf>,
    /// Path to diff image
    pub diff_path: Option<PathBuf>,
    /// Error message if comparison failed
    pub error: Option<String>,
    /// Duration of comparison in milliseconds
    pub comparison_duration_ms: u64,
}

/// Comparison status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComparisonStatus {
    /// Images match within threshold
    Passed,
    /// Images differ beyond threshold
    Failed,
    /// No baseline exists (new screenshot)
    New,
    /// Within threshold but approaching limit
    Warning,
}

impl std::fmt::Display for ComparisonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonStatus::Passed => write!(f, "PASSED"),
            ComparisonStatus::Failed => write!(f, "FAILED"),
            ComparisonStatus::New => write!(f, "NEW"),
            ComparisonStatus::Warning => write!(f, "WARNING"),
        }
    }
}

/// Detailed diff statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffStats {
    /// Percentage of pixels that differ
    pub diff_percent: f64,
    /// Number of different pixels
    pub diff_pixel_count: u64,
    /// Total pixel count
    pub total_pixels: u64,
    /// Maximum color difference encountered
    pub max_color_diff: u32,
    /// Average color difference
    pub avg_color_diff: f64,
    /// Perceptual hash distance (if computed)
    pub hash_distance: Option<u32>,
    /// SSIM score (if computed)
    pub ssim_score: Option<f64>,
    /// Image dimensions
    pub dimensions: (u32, u32),
    /// Baseline dimensions (if different)
    pub baseline_dimensions: Option<(u32, u32)>,
}

impl DiffStats {
    /// Create stats for identical images
    pub fn identical(width: u32, height: u32) -> Self {
        Self {
            diff_percent: 0.0,
            diff_pixel_count: 0,
            total_pixels: (width as u64) * (height as u64),
            max_color_diff: 0,
            avg_color_diff: 0.0,
            hash_distance: Some(0),
            ssim_score: Some(1.0),
            dimensions: (width, height),
            baseline_dimensions: None,
        }
    }
}

/// Result of image comparison operation
#[derive(Debug, Clone)]
pub struct ComparisonResult {
    /// Whether images match within thresholds
    pub matches: bool,
    /// Detailed statistics
    pub stats: DiffStats,
    /// Generated diff image
    pub diff_image: Option<RgbaImage>,
    /// Warning (approaching threshold)
    pub is_warning: bool,
}

// ============================================================================
// Screenshot Capture
// ============================================================================

/// Screenshot capture utility
pub struct ScreenshotCapture {
    config: CaptureSettings,
}

impl ScreenshotCapture {
    /// Create a new screenshot capture utility
    pub fn new(config: CaptureSettings) -> Self {
        Self { config }
    }

    /// Capture a screenshot from raw pixel data
    pub fn capture_from_pixels(
        &self,
        pixels: &[u8],
        width: u32,
        height: u32,
        name: &str,
        output_dir: &Path,
    ) -> QualityResult<PathBuf> {
        // Validate input
        let expected_size = (width as usize) * (height as usize) * 4;
        if pixels.len() != expected_size {
            return Err(QualityError::Config(format!(
                "Invalid pixel buffer size: expected {}, got {}",
                expected_size,
                pixels.len()
            )));
        }

        // Create image buffer
        let img: RgbaImage = ImageBuffer::from_raw(width, height, pixels.to_vec())
            .ok_or_else(|| QualityError::Config("Failed to create image buffer".to_string()))?;

        // Ensure output directory exists
        fs::create_dir_all(output_dir)?;

        // Generate output path
        let filename = format!("{}.{}", name, self.config.format.extension());
        let output_path = output_dir.join(&filename);

        // Save image
        self.save_image(&DynamicImage::ImageRgba8(img), &output_path)?;

        Ok(output_path)
    }

    /// Create a placeholder screenshot for testing
    pub fn create_placeholder(
        &self,
        width: u32,
        height: u32,
        color: [u8; 4],
        name: &str,
        output_dir: &Path,
    ) -> QualityResult<PathBuf> {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = Rgba(color);
        }

        fs::create_dir_all(output_dir)?;

        let filename = format!("{}.{}", name, self.config.format.extension());
        let output_path = output_dir.join(&filename);

        self.save_image(&DynamicImage::ImageRgba8(img), &output_path)?;

        Ok(output_path)
    }

    /// Save an image to disk
    fn save_image(&self, img: &DynamicImage, path: &Path) -> QualityResult<()> {
        img.save(path).map_err(|e| {
            QualityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save image: {}", e),
            ))
        })?;
        Ok(())
    }

    /// Load an image from disk
    pub fn load_image(path: &Path) -> QualityResult<DynamicImage> {
        image::open(path).map_err(|e| {
            QualityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to load image {}: {}", path.display(), e),
            ))
        })
    }

    /// Get configured viewports
    pub fn viewports(&self) -> &[Viewport] {
        &self.config.viewports
    }
}

impl Default for ScreenshotCapture {
    fn default() -> Self {
        Self::new(CaptureSettings::default())
    }
}

// ============================================================================
// Image Comparison
// ============================================================================

/// Image comparison engine
pub struct ImageComparison {
    thresholds: VisualThresholds,
    method: ComparisonMethod,
    aa_tolerance: u32,
    ignore_regions: Vec<IgnoreRegion>,
}

impl ImageComparison {
    /// Create a new image comparison engine
    pub fn new(config: &VisualRegressionConfig) -> Self {
        Self {
            thresholds: config.thresholds.clone(),
            method: config.comparison_method,
            aa_tolerance: config.anti_aliasing_tolerance,
            ignore_regions: config.ignore_regions.clone(),
        }
    }

    /// Compare two images
    pub fn compare(&self, baseline: &DynamicImage, actual: &DynamicImage) -> ComparisonResult {
        match self.method {
            ComparisonMethod::PixelDiff => self.pixel_diff(baseline, actual),
            ComparisonMethod::PerceptualHash => self.perceptual_hash_compare(baseline, actual),
            ComparisonMethod::Ssim => self.ssim_compare(baseline, actual),
            ComparisonMethod::Hybrid => self.hybrid_compare(baseline, actual),
        }
    }

    /// Pixel-by-pixel comparison
    fn pixel_diff(&self, baseline: &DynamicImage, actual: &DynamicImage) -> ComparisonResult {
        let baseline_rgba = baseline.to_rgba8();
        let actual_rgba = actual.to_rgba8();

        let (b_width, b_height) = baseline_rgba.dimensions();
        let (a_width, a_height) = actual_rgba.dimensions();

        // Handle dimension mismatch
        if b_width != a_width || b_height != a_height {
            return ComparisonResult {
                matches: false,
                stats: DiffStats {
                    diff_percent: 100.0,
                    diff_pixel_count: (b_width as u64 * b_height as u64).max(a_width as u64 * a_height as u64),
                    total_pixels: b_width as u64 * b_height as u64,
                    max_color_diff: 255 * 4,
                    avg_color_diff: 0.0,
                    hash_distance: None,
                    ssim_score: None,
                    dimensions: (a_width, a_height),
                    baseline_dimensions: Some((b_width, b_height)),
                },
                diff_image: None,
                is_warning: false,
            };
        }

        let total_pixels = (b_width as u64) * (b_height as u64);
        let mut diff_pixel_count: u64 = 0;
        let mut max_color_diff: u32 = 0;
        let mut total_color_diff: u64 = 0;

        // Create diff image
        let mut diff_img = RgbaImage::new(b_width, b_height);

        for y in 0..b_height {
            for x in 0..b_width {
                // Skip ignored regions
                if self.is_in_ignore_region(x, y, b_width, b_height) {
                    diff_img.put_pixel(x, y, Rgba([128, 128, 128, 128])); // Gray for ignored
                    continue;
                }

                let baseline_pixel = baseline_rgba.get_pixel(x, y);
                let actual_pixel = actual_rgba.get_pixel(x, y);

                let color_diff = self.calculate_pixel_diff(baseline_pixel, actual_pixel);

                if color_diff > self.thresholds.color_threshold as u32 {
                    // Check for anti-aliasing
                    if !self.is_anti_aliasing(x, y, &baseline_rgba, &actual_rgba) {
                        diff_pixel_count += 1;

                        // Mark diff pixel (red with intensity based on difference)
                        let intensity = ((color_diff as f32 / (255.0 * 4.0)) * 255.0) as u8;
                        diff_img.put_pixel(x, y, Rgba([255, 0, 0, intensity.max(128)]));
                    } else {
                        // Anti-aliased pixel (yellow)
                        diff_img.put_pixel(x, y, Rgba([255, 255, 0, 64]));
                    }
                } else {
                    // Matching pixel (copy from actual with reduced opacity)
                    let p = actual_pixel.0;
                    diff_img.put_pixel(x, y, Rgba([p[0], p[1], p[2], 32]));
                }

                max_color_diff = max_color_diff.max(color_diff);
                total_color_diff += color_diff as u64;
            }
        }

        let diff_percent = (diff_pixel_count as f64 / total_pixels as f64) * 100.0;
        let avg_color_diff = if total_pixels > 0 {
            total_color_diff as f64 / total_pixels as f64
        } else {
            0.0
        };

        let matches = diff_percent <= self.thresholds.pixel_threshold_percent;
        let is_warning = matches && diff_percent >= (self.thresholds.pixel_threshold_percent * self.thresholds.warning_at_percent / 100.0);

        ComparisonResult {
            matches,
            stats: DiffStats {
                diff_percent,
                diff_pixel_count,
                total_pixels,
                max_color_diff,
                avg_color_diff,
                hash_distance: None,
                ssim_score: None,
                dimensions: (a_width, a_height),
                baseline_dimensions: None,
            },
            diff_image: Some(diff_img),
            is_warning,
        }
    }

    /// Calculate pixel difference
    fn calculate_pixel_diff(&self, p1: &Rgba<u8>, p2: &Rgba<u8>) -> u32 {
        let r_diff = (p1.0[0] as i32 - p2.0[0] as i32).unsigned_abs();
        let g_diff = (p1.0[1] as i32 - p2.0[1] as i32).unsigned_abs();
        let b_diff = (p1.0[2] as i32 - p2.0[2] as i32).unsigned_abs();
        let a_diff = (p1.0[3] as i32 - p2.0[3] as i32).unsigned_abs();
        r_diff + g_diff + b_diff + a_diff
    }

    /// Check if a pixel is in an ignore region
    fn is_in_ignore_region(&self, x: u32, y: u32, width: u32, height: u32) -> bool {
        for region in &self.ignore_regions {
            let rx = region.x.to_pixels(width);
            let ry = region.y.to_pixels(height);
            let rw = region.width.to_pixels(width);
            let rh = region.height.to_pixels(height);

            if x >= rx && x < rx + rw && y >= ry && y < ry + rh {
                return true;
            }
        }
        false
    }

    /// Check if pixel difference is likely due to anti-aliasing
    /// Anti-aliasing is detected when a pixel is on an edge (has significantly different
    /// neighbors within the same image) - not when all pixels are uniformly different.
    fn is_anti_aliasing(&self, x: u32, y: u32, baseline: &RgbaImage, actual: &RgbaImage) -> bool {
        let (width, height) = baseline.dimensions();
        let tolerance = self.aa_tolerance;

        let baseline_center = baseline.get_pixel(x, y);
        let actual_center = actual.get_pixel(x, y);

        // For anti-aliasing, we need the pixel to be on an edge in at least one of the images.
        // Check if this pixel has significantly different neighbors in baseline or actual.
        let mut baseline_is_edge = false;
        let mut actual_is_edge = false;

        // Check surrounding pixels
        for dy in 0..=2 {
            for dx in 0..=2 {
                if dx == 1 && dy == 1 {
                    continue; // Skip center pixel
                }

                let nx = x.saturating_sub(1).saturating_add(dx);
                let ny = y.saturating_sub(1).saturating_add(dy);

                if nx >= width || ny >= height {
                    continue;
                }

                let baseline_neighbor = baseline.get_pixel(nx, ny);
                let actual_neighbor = actual.get_pixel(nx, ny);

                // Check if this pixel is on an edge in baseline
                let baseline_internal_diff = self.calculate_pixel_diff(baseline_center, baseline_neighbor);
                if baseline_internal_diff > (tolerance * 20) as u32 {
                    baseline_is_edge = true;
                }

                // Check if this pixel is on an edge in actual
                let actual_internal_diff = self.calculate_pixel_diff(actual_center, actual_neighbor);
                if actual_internal_diff > (tolerance * 20) as u32 {
                    actual_is_edge = true;
                }
            }
        }

        // Only consider it anti-aliasing if the pixel is on an edge in at least one image
        // This prevents solid color changes from being flagged as anti-aliasing
        baseline_is_edge || actual_is_edge
    }

    /// Perceptual hash comparison
    fn perceptual_hash_compare(&self, baseline: &DynamicImage, actual: &DynamicImage) -> ComparisonResult {
        let baseline_hash = self.compute_perceptual_hash(baseline);
        let actual_hash = self.compute_perceptual_hash(actual);

        let distance = self.hamming_distance(&baseline_hash, &actual_hash);
        let matches = distance <= self.thresholds.hash_distance_threshold;
        let is_warning = matches && distance >= (self.thresholds.hash_distance_threshold / 2);

        let (width, height) = baseline.dimensions();

        ComparisonResult {
            matches,
            stats: DiffStats {
                diff_percent: (distance as f64 / 64.0) * 100.0, // 64-bit hash
                diff_pixel_count: distance as u64,
                total_pixels: 64,
                max_color_diff: 0,
                avg_color_diff: 0.0,
                hash_distance: Some(distance),
                ssim_score: None,
                dimensions: (width, height),
                baseline_dimensions: None,
            },
            diff_image: None,
            is_warning,
        }
    }

    /// Compute perceptual hash (dHash algorithm)
    fn compute_perceptual_hash(&self, img: &DynamicImage) -> [u8; 8] {
        // Resize to 9x8 for dHash
        let resized = img.resize_exact(9, 8, image::imageops::FilterType::Lanczos3);
        let grayscale = resized.to_luma8();

        let mut hash = [0u8; 8];
        let mut bit_idx = 0;

        for y in 0..8 {
            for x in 0..8 {
                let left = grayscale.get_pixel(x, y).0[0];
                let right = grayscale.get_pixel(x + 1, y).0[0];

                if left > right {
                    let byte_idx = bit_idx / 8;
                    let bit_pos = 7 - (bit_idx % 8);
                    hash[byte_idx] |= 1 << bit_pos;
                }
                bit_idx += 1;
            }
        }

        hash
    }

    /// Calculate Hamming distance between two hashes
    fn hamming_distance(&self, h1: &[u8; 8], h2: &[u8; 8]) -> u32 {
        let mut distance = 0;
        for i in 0..8 {
            distance += (h1[i] ^ h2[i]).count_ones();
        }
        distance
    }

    /// SSIM-based comparison (simplified)
    fn ssim_compare(&self, baseline: &DynamicImage, actual: &DynamicImage) -> ComparisonResult {
        let ssim_score = self.compute_ssim(baseline, actual);
        let matches = ssim_score >= self.thresholds.ssim_threshold;
        let is_warning = matches && ssim_score <= (self.thresholds.ssim_threshold + (1.0 - self.thresholds.ssim_threshold) * 0.5);

        let (width, height) = baseline.dimensions();

        ComparisonResult {
            matches,
            stats: DiffStats {
                diff_percent: (1.0 - ssim_score) * 100.0,
                diff_pixel_count: 0,
                total_pixels: (width as u64) * (height as u64),
                max_color_diff: 0,
                avg_color_diff: 0.0,
                hash_distance: None,
                ssim_score: Some(ssim_score),
                dimensions: (width, height),
                baseline_dimensions: None,
            },
            diff_image: None,
            is_warning,
        }
    }

    /// Compute SSIM (simplified implementation)
    fn compute_ssim(&self, img1: &DynamicImage, img2: &DynamicImage) -> f64 {
        let gray1 = img1.to_luma8();
        let gray2 = img2.to_luma8();

        let (w1, h1) = gray1.dimensions();
        let (w2, h2) = gray2.dimensions();

        if w1 != w2 || h1 != h2 {
            return 0.0;
        }

        let n = (w1 as f64) * (h1 as f64);

        // Calculate means
        let mut sum1: f64 = 0.0;
        let mut sum2: f64 = 0.0;

        for (p1, p2) in gray1.pixels().zip(gray2.pixels()) {
            sum1 += p1.0[0] as f64;
            sum2 += p2.0[0] as f64;
        }

        let mean1 = sum1 / n;
        let mean2 = sum2 / n;

        // Calculate variances and covariance
        let mut var1: f64 = 0.0;
        let mut var2: f64 = 0.0;
        let mut covar: f64 = 0.0;

        for (p1, p2) in gray1.pixels().zip(gray2.pixels()) {
            let d1 = p1.0[0] as f64 - mean1;
            let d2 = p2.0[0] as f64 - mean2;
            var1 += d1 * d1;
            var2 += d2 * d2;
            covar += d1 * d2;
        }

        var1 /= n - 1.0;
        var2 /= n - 1.0;
        covar /= n - 1.0;

        // SSIM constants
        let c1 = 6.5025;  // (0.01 * 255)^2
        let c2 = 58.5225; // (0.03 * 255)^2

        // SSIM formula
        let numerator = (2.0 * mean1 * mean2 + c1) * (2.0 * covar + c2);
        let denominator = (mean1 * mean1 + mean2 * mean2 + c1) * (var1 + var2 + c2);

        numerator / denominator
    }

    /// Hybrid comparison using multiple methods
    fn hybrid_compare(&self, baseline: &DynamicImage, actual: &DynamicImage) -> ComparisonResult {
        let pixel_result = self.pixel_diff(baseline, actual);
        let hash_result = self.perceptual_hash_compare(baseline, actual);
        let ssim_result = self.ssim_compare(baseline, actual);

        // All methods must pass for hybrid to pass
        let matches = pixel_result.matches && hash_result.matches && ssim_result.matches;
        let is_warning = matches && (pixel_result.is_warning || hash_result.is_warning || ssim_result.is_warning);

        let mut stats = pixel_result.stats.clone();
        stats.hash_distance = hash_result.stats.hash_distance;
        stats.ssim_score = ssim_result.stats.ssim_score;

        ComparisonResult {
            matches,
            stats,
            diff_image: pixel_result.diff_image,
            is_warning,
        }
    }
}

// ============================================================================
// Baseline Management
// ============================================================================

/// Baseline screenshot manager
pub struct BaselineManager {
    baseline_dir: PathBuf,
    actual_dir: PathBuf,
    diff_dir: PathBuf,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new(config: &VisualRegressionConfig, project_path: &Path) -> Self {
        Self {
            baseline_dir: project_path.join(&config.baseline_dir),
            actual_dir: project_path.join(&config.actual_dir),
            diff_dir: project_path.join(&config.diff_dir),
        }
    }

    /// Initialize directories
    pub fn init(&self) -> QualityResult<()> {
        fs::create_dir_all(&self.baseline_dir)?;
        fs::create_dir_all(&self.actual_dir)?;
        fs::create_dir_all(&self.diff_dir)?;
        Ok(())
    }

    /// Check if baseline exists for a screenshot
    pub fn has_baseline(&self, name: &str, viewport: &str) -> bool {
        self.baseline_path(name, viewport).exists()
    }

    /// Get baseline image path
    pub fn baseline_path(&self, name: &str, viewport: &str) -> PathBuf {
        self.baseline_dir.join(format!("{}_{}.png", name, viewport))
    }

    /// Get actual image path
    pub fn actual_path(&self, name: &str, viewport: &str) -> PathBuf {
        self.actual_dir.join(format!("{}_{}.png", name, viewport))
    }

    /// Get diff image path
    pub fn diff_path(&self, name: &str, viewport: &str) -> PathBuf {
        self.diff_dir.join(format!("{}_{}_diff.png", name, viewport))
    }

    /// Load baseline image
    pub fn load_baseline(&self, name: &str, viewport: &str) -> QualityResult<DynamicImage> {
        let path = self.baseline_path(name, viewport);
        ScreenshotCapture::load_image(&path)
    }

    /// Save baseline image
    pub fn save_baseline(&self, name: &str, viewport: &str, image: &DynamicImage) -> QualityResult<PathBuf> {
        let path = self.baseline_path(name, viewport);
        image.save(&path).map_err(|e| {
            QualityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save baseline: {}", e),
            ))
        })?;
        Ok(path)
    }

    /// Save actual image
    pub fn save_actual(&self, name: &str, viewport: &str, image: &DynamicImage) -> QualityResult<PathBuf> {
        let path = self.actual_path(name, viewport);
        image.save(&path).map_err(|e| {
            QualityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save actual: {}", e),
            ))
        })?;
        Ok(path)
    }

    /// Save diff image
    pub fn save_diff(&self, name: &str, viewport: &str, image: &RgbaImage) -> QualityResult<PathBuf> {
        let path = self.diff_path(name, viewport);
        image.save(&path).map_err(|e| {
            QualityError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to save diff: {}", e),
            ))
        })?;
        Ok(path)
    }

    /// Update baseline from actual
    pub fn update_baseline(&self, name: &str, viewport: &str) -> QualityResult<()> {
        let actual = self.actual_path(name, viewport);
        let baseline = self.baseline_path(name, viewport);

        if actual.exists() {
            fs::copy(&actual, &baseline)?;
            tracing::info!("Updated baseline: {}", baseline.display());
        }
        Ok(())
    }

    /// List all baselines
    pub fn list_baselines(&self) -> QualityResult<Vec<String>> {
        let mut baselines = Vec::new();

        if self.baseline_dir.exists() {
            for entry in fs::read_dir(&self.baseline_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map_or(false, |e| e == "png") {
                    if let Some(name) = path.file_stem().and_then(|s| s.to_str()) {
                        baselines.push(name.to_string());
                    }
                }
            }
        }

        Ok(baselines)
    }

    /// Clean actual and diff directories
    pub fn clean(&self) -> QualityResult<()> {
        if self.actual_dir.exists() {
            fs::remove_dir_all(&self.actual_dir)?;
        }
        if self.diff_dir.exists() {
            fs::remove_dir_all(&self.diff_dir)?;
        }
        self.init()
    }

    /// Compute hash of baseline for versioning
    pub fn baseline_hash(&self, name: &str, viewport: &str) -> QualityResult<String> {
        let path = self.baseline_path(name, viewport);
        let data = fs::read(&path)?;

        let mut hasher = Sha256::new();
        hasher.update(&data);
        let result = hasher.finalize();

        Ok(base64::Engine::encode(&base64::engine::general_purpose::STANDARD, result))
    }
}

// ============================================================================
// Report Generation
// ============================================================================

/// HTML report generator for visual regression results
pub struct VisualReportGenerator {
    report_dir: PathBuf,
}

impl VisualReportGenerator {
    /// Create a new report generator
    pub fn new(report_dir: &Path) -> Self {
        Self {
            report_dir: report_dir.to_path_buf(),
        }
    }

    /// Generate HTML report
    pub fn generate(&self, report: &VisualRegressionReport) -> QualityResult<PathBuf> {
        fs::create_dir_all(&self.report_dir)?;

        let html = self.render_html(report);
        let report_path = self.report_dir.join("visual-regression-report.html");

        let mut file = fs::File::create(&report_path)?;
        file.write_all(html.as_bytes())?;

        // Also save JSON report
        let json_path = self.report_dir.join("visual-regression-report.json");
        let json = report.to_json().map_err(|e| {
            QualityError::Config(format!("Failed to serialize report: {}", e))
        })?;
        fs::write(&json_path, json)?;

        tracing::info!("Generated visual regression report: {}", report_path.display());

        Ok(report_path)
    }

    /// Render HTML report
    fn render_html(&self, report: &VisualRegressionReport) -> String {
        let status_class = if report.passed { "passed" } else { "failed" };
        let status_text = if report.passed { "PASSED" } else { "FAILED" };

        let results_html: String = report.results.iter().map(|r| {
            self.render_result_card(r)
        }).collect();

        format!(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Visual Regression Report - OxideKit</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: #0d1117;
            color: #c9d1d9;
            line-height: 1.6;
            padding: 20px;
        }}
        .container {{
            max-width: 1400px;
            margin: 0 auto;
        }}
        header {{
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            padding: 24px;
            margin-bottom: 24px;
        }}
        h1 {{
            font-size: 24px;
            margin-bottom: 16px;
        }}
        .summary {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
            gap: 16px;
            margin-top: 16px;
        }}
        .stat {{
            background: #21262d;
            border-radius: 6px;
            padding: 16px;
            text-align: center;
        }}
        .stat-value {{
            font-size: 32px;
            font-weight: bold;
        }}
        .stat-label {{
            font-size: 12px;
            color: #8b949e;
            text-transform: uppercase;
        }}
        .status {{
            display: inline-block;
            padding: 4px 12px;
            border-radius: 16px;
            font-weight: 600;
            font-size: 14px;
        }}
        .status.passed {{
            background: #238636;
            color: white;
        }}
        .status.failed {{
            background: #da3633;
            color: white;
        }}
        .status.warning {{
            background: #9e6a03;
            color: white;
        }}
        .status.new {{
            background: #1f6feb;
            color: white;
        }}
        .results {{
            display: grid;
            gap: 16px;
        }}
        .result-card {{
            background: #161b22;
            border: 1px solid #30363d;
            border-radius: 8px;
            overflow: hidden;
        }}
        .result-card.failed {{
            border-color: #da3633;
        }}
        .result-card.warning {{
            border-color: #9e6a03;
        }}
        .result-header {{
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 16px;
            border-bottom: 1px solid #30363d;
        }}
        .result-name {{
            font-weight: 600;
            font-size: 16px;
        }}
        .result-body {{
            padding: 16px;
        }}
        .image-comparison {{
            display: grid;
            grid-template-columns: repeat(3, 1fr);
            gap: 16px;
        }}
        .image-panel {{
            text-align: center;
        }}
        .image-panel h4 {{
            margin-bottom: 8px;
            font-size: 14px;
            color: #8b949e;
        }}
        .image-panel img {{
            max-width: 100%;
            border-radius: 4px;
            border: 1px solid #30363d;
            background: #21262d;
        }}
        .stats-grid {{
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
            gap: 12px;
            margin-top: 16px;
            padding-top: 16px;
            border-top: 1px solid #30363d;
        }}
        .stats-item {{
            font-size: 14px;
        }}
        .stats-item .label {{
            color: #8b949e;
        }}
        .stats-item .value {{
            font-weight: 600;
        }}
        .diff-overlay {{
            position: relative;
            cursor: pointer;
        }}
        .diff-overlay img {{
            transition: opacity 0.3s;
        }}
        .diff-overlay:hover .diff-img {{
            opacity: 0.5;
        }}
        .error-message {{
            background: #da3633;
            color: white;
            padding: 12px;
            border-radius: 4px;
            margin-top: 12px;
        }}
        footer {{
            margin-top: 24px;
            text-align: center;
            color: #8b949e;
            font-size: 14px;
        }}
        .filter-bar {{
            display: flex;
            gap: 8px;
            margin-bottom: 16px;
        }}
        .filter-btn {{
            padding: 8px 16px;
            border: 1px solid #30363d;
            border-radius: 6px;
            background: #21262d;
            color: #c9d1d9;
            cursor: pointer;
            font-size: 14px;
        }}
        .filter-btn:hover {{
            background: #30363d;
        }}
        .filter-btn.active {{
            background: #1f6feb;
            border-color: #1f6feb;
        }}
    </style>
</head>
<body>
    <div class="container">
        <header>
            <h1>Visual Regression Report</h1>
            <span class="status {status_class}">{status_text}</span>

            <div class="summary">
                <div class="stat">
                    <div class="stat-value">{total}</div>
                    <div class="stat-label">Total</div>
                </div>
                <div class="stat">
                    <div class="stat-value" style="color: #3fb950;">{passed}</div>
                    <div class="stat-label">Passed</div>
                </div>
                <div class="stat">
                    <div class="stat-value" style="color: #da3633;">{failed}</div>
                    <div class="stat-label">Failed</div>
                </div>
                <div class="stat">
                    <div class="stat-value" style="color: #9e6a03;">{warnings}</div>
                    <div class="stat-label">Warnings</div>
                </div>
                <div class="stat">
                    <div class="stat-value" style="color: #1f6feb;">{new}</div>
                    <div class="stat-label">New</div>
                </div>
            </div>

            <div style="margin-top: 16px; color: #8b949e; font-size: 14px;">
                <div>Project: <code>{project}</code></div>
                <div>Generated: {timestamp}</div>
                <div>Method: {method}</div>
                <div>Duration: {duration}ms</div>
            </div>
        </header>

        <div class="filter-bar">
            <button class="filter-btn active" onclick="filterResults('all')">All</button>
            <button class="filter-btn" onclick="filterResults('failed')">Failed</button>
            <button class="filter-btn" onclick="filterResults('warning')">Warnings</button>
            <button class="filter-btn" onclick="filterResults('new')">New</button>
            <button class="filter-btn" onclick="filterResults('passed')">Passed</button>
        </div>

        <div class="results">
            {results}
        </div>

        <footer>
            <p>Generated by OxideKit Visual Regression Testing</p>
        </footer>
    </div>

    <script>
        function filterResults(status) {{
            document.querySelectorAll('.filter-btn').forEach(btn => {{
                btn.classList.remove('active');
                if (btn.textContent.toLowerCase() === status || (status === 'all' && btn.textContent === 'All')) {{
                    btn.classList.add('active');
                }}
            }});

            document.querySelectorAll('.result-card').forEach(card => {{
                if (status === 'all') {{
                    card.style.display = 'block';
                }} else {{
                    const cardStatus = card.dataset.status;
                    card.style.display = cardStatus === status ? 'block' : 'none';
                }}
            }});
        }}
    </script>
</body>
</html>"#,
            status_class = status_class,
            status_text = status_text,
            total = report.total_count,
            passed = report.passed_count,
            failed = report.failed_count,
            warnings = report.warning_count,
            new = report.new_count,
            project = report.project.display(),
            timestamp = report.timestamp,
            method = report.comparison_method,
            duration = report.duration_ms,
            results = results_html,
        )
    }

    /// Render a single result card
    fn render_result_card(&self, result: &VisualRegressionResult) -> String {
        let status_class = match result.status {
            ComparisonStatus::Passed => "passed",
            ComparisonStatus::Failed => "failed",
            ComparisonStatus::Warning => "warning",
            ComparisonStatus::New => "new",
        };

        let images_html = if let (Some(baseline), Some(actual), Some(diff)) =
            (&result.baseline_path, &result.actual_path, &result.diff_path) {
            format!(r#"
                <div class="image-comparison">
                    <div class="image-panel">
                        <h4>Baseline</h4>
                        <img src="{baseline}" alt="Baseline">
                    </div>
                    <div class="image-panel">
                        <h4>Actual</h4>
                        <img src="{actual}" alt="Actual">
                    </div>
                    <div class="image-panel diff-overlay">
                        <h4>Diff</h4>
                        <img src="{diff}" alt="Diff" class="diff-img">
                    </div>
                </div>
            "#,
                baseline = baseline.display(),
                actual = actual.display(),
                diff = diff.display(),
            )
        } else if let Some(actual) = &result.actual_path {
            format!(r#"
                <div class="image-comparison">
                    <div class="image-panel">
                        <h4>New Screenshot</h4>
                        <img src="{}" alt="New screenshot">
                    </div>
                </div>
            "#, actual.display())
        } else {
            String::new()
        };

        let stats_html = if let Some(stats) = &result.stats {
            format!(r#"
                <div class="stats-grid">
                    <div class="stats-item">
                        <span class="label">Diff: </span>
                        <span class="value">{diff_percent:.4}%</span>
                    </div>
                    <div class="stats-item">
                        <span class="label">Pixels: </span>
                        <span class="value">{diff_pixels}/{total_pixels}</span>
                    </div>
                    <div class="stats-item">
                        <span class="label">Max Diff: </span>
                        <span class="value">{max_diff}</span>
                    </div>
                    <div class="stats-item">
                        <span class="label">Dimensions: </span>
                        <span class="value">{width}x{height}</span>
                    </div>
                    {hash_html}
                    {ssim_html}
                </div>
            "#,
                diff_percent = stats.diff_percent,
                diff_pixels = stats.diff_pixel_count,
                total_pixels = stats.total_pixels,
                max_diff = stats.max_color_diff,
                width = stats.dimensions.0,
                height = stats.dimensions.1,
                hash_html = stats.hash_distance.map_or(String::new(), |d| {
                    format!(r#"<div class="stats-item"><span class="label">Hash Distance: </span><span class="value">{}</span></div>"#, d)
                }),
                ssim_html = stats.ssim_score.map_or(String::new(), |s| {
                    format!(r#"<div class="stats-item"><span class="label">SSIM: </span><span class="value">{:.4}</span></div>"#, s)
                }),
            )
        } else {
            String::new()
        };

        let error_html = result.error.as_ref().map_or(String::new(), |e| {
            format!(r#"<div class="error-message">{}</div>"#, e)
        });

        format!(r#"
            <div class="result-card {status_class}" data-status="{status_class}">
                <div class="result-header">
                    <span class="result-name">{name} ({viewport})</span>
                    <span class="status {status_class}">{status}</span>
                </div>
                <div class="result-body">
                    {images}
                    {stats}
                    {error}
                </div>
            </div>
        "#,
            status_class = status_class,
            name = result.name,
            viewport = result.viewport,
            status = result.status,
            images = images_html,
            stats = stats_html,
            error = error_html,
        )
    }
}

// ============================================================================
// CI Integration Helpers
// ============================================================================

/// Generate GitHub Actions workflow for visual regression testing
pub fn generate_github_actions_visual() -> String {
    r#"# OxideKit Visual Regression Testing
# Generated by oxide-quality

name: Visual Regression

on:
  pull_request:
    branches: [main, master]

jobs:
  visual-regression:
    name: Visual Regression Tests
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: dtolnay/rust-action@stable

      - name: Cache cargo
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/bin/
            ~/.cargo/registry/index/
            ~/.cargo/registry/cache/
            ~/.cargo/git/db/
            target/
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install oxide CLI
        run: cargo install --path crates/oxide-cli

      - name: Checkout baseline branch
        uses: actions/checkout@v4
        with:
          ref: ${{ github.base_ref }}
          path: baseline

      - name: Run visual regression tests
        run: |
          oxide visual-regression test \
            --baseline-dir baseline/.visual-regression/baseline \
            --output-dir .visual-regression/reports

      - name: Upload visual regression report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: visual-regression-report
          path: .visual-regression/reports/
          retention-days: 14

      - name: Comment on PR
        if: failure() && github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(
              fs.readFileSync('.visual-regression/reports/visual-regression-report.json', 'utf8')
            );

            const failures = report.results
              .filter(r => r.status === 'failed')
              .map(r => `- **${r.name}** (${r.viewport}): ${r.stats?.diff_percent?.toFixed(4)}% diff`)
              .join('\n');

            const body = `## Visual Regression Test Failed

            **${report.failed_count}** visual regressions detected.

            ### Failed Screenshots
            ${failures}

            [View full report](${process.env.GITHUB_SERVER_URL}/${process.env.GITHUB_REPOSITORY}/actions/runs/${process.env.GITHUB_RUN_ID})

            <details>
            <summary>Update baselines</summary>

            To update baselines, run:
            \`\`\`bash
            oxide visual-regression update
            git add .visual-regression/baseline/
            git commit -m "chore: update visual regression baselines"
            \`\`\`
            </details>`;

            await github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
"#.to_string()
}

/// Generate GitLab CI config for visual regression testing
pub fn generate_gitlab_ci_visual() -> String {
    r#"# OxideKit Visual Regression Testing
# Generated by oxide-quality

visual-regression:
  stage: test
  image: rust:latest
  variables:
    CARGO_HOME: $CI_PROJECT_DIR/.cargo
  cache:
    key: $CI_COMMIT_REF_SLUG
    paths:
      - .cargo/
      - target/
  script:
    - cargo install --path crates/oxide-cli
    - |
      # Fetch baseline from main branch
      git fetch origin main:baseline-ref
      git worktree add ../baseline baseline-ref
    - oxide visual-regression test --baseline-dir ../baseline/.visual-regression/baseline --output-dir .visual-regression/reports
  artifacts:
    when: always
    paths:
      - .visual-regression/reports/
    expire_in: 14 days
    reports:
      junit: .visual-regression/reports/junit.xml
  only:
    - merge_requests
  allow_failure: true
"#.to_string()
}

/// Exit codes for CI integration
pub mod exit_codes {
    /// All tests passed
    pub const SUCCESS: i32 = 0;
    /// Visual regressions detected
    pub const REGRESSIONS_FOUND: i32 = 1;
    /// New screenshots detected (no baseline)
    pub const NEW_SCREENSHOTS: i32 = 2;
    /// Configuration or runtime error
    pub const ERROR: i32 = 3;
}

// ============================================================================
// Main API Functions
// ============================================================================

/// Run visual regression tests
pub fn run_visual_regression(
    project_path: &Path,
    config: &VisualRegressionConfig,
) -> QualityResult<VisualRegressionReport> {
    let start = std::time::Instant::now();
    let mut report = VisualRegressionReport::new(project_path, config);

    if !config.enabled {
        tracing::info!("Visual regression testing is disabled");
        return Ok(report);
    }

    let baseline_manager = BaselineManager::new(config, project_path);
    baseline_manager.init()?;

    let comparison = ImageComparison::new(config);

    // Get list of screenshots to test
    let screenshots = discover_screenshots(project_path, config)?;

    for (name, viewport, actual_image) in screenshots {
        let result = run_single_comparison(
            &name,
            &viewport,
            &actual_image,
            &baseline_manager,
            &comparison,
            config,
        )?;
        report.add_result(result);
    }

    report.duration_ms = start.elapsed().as_millis() as u64;

    // Generate HTML report if enabled
    if config.generate_report {
        let report_dir = project_path.join(&config.report_dir);
        let generator = VisualReportGenerator::new(&report_dir);
        generator.generate(&report)?;
    }

    Ok(report)
}

/// Run comparison for a single screenshot
fn run_single_comparison(
    name: &str,
    viewport: &str,
    actual_image: &DynamicImage,
    baseline_manager: &BaselineManager,
    comparison: &ImageComparison,
    config: &VisualRegressionConfig,
) -> QualityResult<VisualRegressionResult> {
    let start = std::time::Instant::now();

    // Save actual image
    let actual_path = baseline_manager.save_actual(name, viewport, actual_image)?;

    // Check if baseline exists
    if !baseline_manager.has_baseline(name, viewport) {
        // New screenshot
        if config.auto_update_baselines {
            baseline_manager.save_baseline(name, viewport, actual_image)?;
        }

        return Ok(VisualRegressionResult {
            name: name.to_string(),
            viewport: viewport.to_string(),
            status: ComparisonStatus::New,
            stats: None,
            baseline_path: None,
            actual_path: Some(actual_path),
            diff_path: None,
            error: None,
            comparison_duration_ms: start.elapsed().as_millis() as u64,
        });
    }

    // Load baseline and compare
    let baseline_image = baseline_manager.load_baseline(name, viewport)?;
    let comparison_result = comparison.compare(&baseline_image, actual_image);

    // Save diff image if available
    let diff_path = if let Some(ref diff_img) = comparison_result.diff_image {
        Some(baseline_manager.save_diff(name, viewport, diff_img)?)
    } else {
        None
    };

    // Determine status
    let status = if comparison_result.matches {
        if comparison_result.is_warning {
            ComparisonStatus::Warning
        } else {
            ComparisonStatus::Passed
        }
    } else {
        // Auto-update baselines if configured
        if config.auto_update_baselines {
            baseline_manager.save_baseline(name, viewport, actual_image)?;
        }
        ComparisonStatus::Failed
    };

    Ok(VisualRegressionResult {
        name: name.to_string(),
        viewport: viewport.to_string(),
        status,
        stats: Some(comparison_result.stats),
        baseline_path: Some(baseline_manager.baseline_path(name, viewport)),
        actual_path: Some(actual_path),
        diff_path,
        error: None,
        comparison_duration_ms: start.elapsed().as_millis() as u64,
    })
}

/// Discover screenshots to test
fn discover_screenshots(
    project_path: &Path,
    config: &VisualRegressionConfig,
) -> QualityResult<Vec<(String, String, DynamicImage)>> {
    let mut screenshots = Vec::new();
    let actual_dir = project_path.join(&config.actual_dir);

    if !actual_dir.exists() {
        return Ok(screenshots);
    }

    for entry in walkdir::WalkDir::new(&actual_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "png" || ext == "jpg" || ext == "webp" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    // Parse name and viewport from filename (format: name_viewport.ext)
                    let parts: Vec<&str> = stem.rsplitn(2, '_').collect();
                    let (name, viewport) = if parts.len() == 2 {
                        (parts[1].to_string(), parts[0].to_string())
                    } else {
                        (stem.to_string(), "default".to_string())
                    };

                    match image::open(path) {
                        Ok(img) => screenshots.push((name, viewport, img)),
                        Err(e) => {
                            tracing::warn!("Failed to load image {}: {}", path.display(), e);
                        }
                    }
                }
            }
        }
    }

    Ok(screenshots)
}

/// Update baselines from actual screenshots
pub fn update_baselines(
    project_path: &Path,
    config: &VisualRegressionConfig,
    names: Option<&[String]>,
) -> QualityResult<Vec<String>> {
    let baseline_manager = BaselineManager::new(config, project_path);
    baseline_manager.init()?;

    let mut updated = Vec::new();
    let actual_dir = project_path.join(&config.actual_dir);

    if !actual_dir.exists() {
        return Ok(updated);
    }

    for entry in walkdir::WalkDir::new(&actual_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();
        if let Some(ext) = path.extension() {
            if ext == "png" || ext == "jpg" || ext == "webp" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    let parts: Vec<&str> = stem.rsplitn(2, '_').collect();
                    let (name, viewport) = if parts.len() == 2 {
                        (parts[1].to_string(), parts[0].to_string())
                    } else {
                        (stem.to_string(), "default".to_string())
                    };

                    // Filter by name if specified
                    if let Some(filter) = names {
                        if !filter.iter().any(|n| n == &name || n == stem) {
                            continue;
                        }
                    }

                    baseline_manager.update_baseline(&name, &viewport)?;
                    updated.push(format!("{}_{}", name, viewport));
                }
            }
        }
    }

    Ok(updated)
}

/// Generate visual regression report
pub fn generate_visual_report(
    report: &VisualRegressionReport,
    output_dir: &Path,
) -> QualityResult<PathBuf> {
    let generator = VisualReportGenerator::new(output_dir);
    generator.generate(report)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{RegionCoord, ImageFormat};
    use tempfile::TempDir;

    fn create_test_image(width: u32, height: u32, color: [u8; 4]) -> DynamicImage {
        let mut img = RgbaImage::new(width, height);
        for pixel in img.pixels_mut() {
            *pixel = Rgba(color);
        }
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_visual_regression_report_new() {
        let config = VisualRegressionConfig::default();
        let report = VisualRegressionReport::new(Path::new("/test"), &config);

        assert!(report.passed);
        assert_eq!(report.total_count, 0);
        assert_eq!(report.failed_count, 0);
    }

    #[test]
    fn test_visual_regression_report_add_result() {
        let config = VisualRegressionConfig::default();
        let mut report = VisualRegressionReport::new(Path::new("/test"), &config);

        report.add_result(VisualRegressionResult {
            name: "test".to_string(),
            viewport: "desktop".to_string(),
            status: ComparisonStatus::Passed,
            stats: None,
            baseline_path: None,
            actual_path: None,
            diff_path: None,
            error: None,
            comparison_duration_ms: 100,
        });

        assert_eq!(report.total_count, 1);
        assert_eq!(report.passed_count, 1);
        assert!(report.passed);

        report.add_result(VisualRegressionResult {
            name: "test2".to_string(),
            viewport: "desktop".to_string(),
            status: ComparisonStatus::Failed,
            stats: None,
            baseline_path: None,
            actual_path: None,
            diff_path: None,
            error: None,
            comparison_duration_ms: 100,
        });

        assert_eq!(report.total_count, 2);
        assert_eq!(report.failed_count, 1);
        assert!(!report.passed);
    }

    #[test]
    fn test_comparison_status_display() {
        assert_eq!(ComparisonStatus::Passed.to_string(), "PASSED");
        assert_eq!(ComparisonStatus::Failed.to_string(), "FAILED");
        assert_eq!(ComparisonStatus::New.to_string(), "NEW");
        assert_eq!(ComparisonStatus::Warning.to_string(), "WARNING");
    }

    #[test]
    fn test_diff_stats_identical() {
        let stats = DiffStats::identical(100, 100);
        assert_eq!(stats.diff_percent, 0.0);
        assert_eq!(stats.diff_pixel_count, 0);
        assert_eq!(stats.total_pixels, 10000);
    }

    #[test]
    fn test_region_coord_to_pixels() {
        let pixels = RegionCoord::Pixels(100);
        assert_eq!(pixels.to_pixels(1000), 100);

        let percent = RegionCoord::Percent(10.0);
        assert_eq!(percent.to_pixels(1000), 100);
    }

    #[test]
    fn test_image_comparison_identical() {
        let config = VisualRegressionConfig::default();
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [255, 0, 0, 255]);
        let img2 = create_test_image(100, 100, [255, 0, 0, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(result.matches);
        assert_eq!(result.stats.diff_percent, 0.0);
    }

    #[test]
    fn test_image_comparison_different() {
        let config = VisualRegressionConfig::default();
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [255, 0, 0, 255]);
        let img2 = create_test_image(100, 100, [0, 255, 0, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(!result.matches);
        assert!(result.stats.diff_percent > 0.0);
        assert_eq!(result.stats.diff_pixel_count, 10000);
    }

    #[test]
    fn test_image_comparison_dimension_mismatch() {
        let config = VisualRegressionConfig::default();
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [255, 0, 0, 255]);
        let img2 = create_test_image(200, 200, [255, 0, 0, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(!result.matches);
        assert_eq!(result.stats.diff_percent, 100.0);
    }

    #[test]
    fn test_perceptual_hash_identical() {
        let config = VisualRegressionConfig {
            comparison_method: ComparisonMethod::PerceptualHash,
            ..Default::default()
        };
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [255, 0, 0, 255]);
        let img2 = create_test_image(100, 100, [255, 0, 0, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(result.matches);
        assert_eq!(result.stats.hash_distance, Some(0));
    }

    #[test]
    fn test_ssim_identical() {
        let config = VisualRegressionConfig {
            comparison_method: ComparisonMethod::Ssim,
            ..Default::default()
        };
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [128, 128, 128, 255]);
        let img2 = create_test_image(100, 100, [128, 128, 128, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(result.matches);
        assert!(result.stats.ssim_score.unwrap() >= 0.99);
    }

    #[test]
    fn test_hybrid_comparison() {
        let config = VisualRegressionConfig {
            comparison_method: ComparisonMethod::Hybrid,
            ..Default::default()
        };
        let comparison = ImageComparison::new(&config);

        let img1 = create_test_image(100, 100, [128, 128, 128, 255]);
        let img2 = create_test_image(100, 100, [128, 128, 128, 255]);

        let result = comparison.compare(&img1, &img2);
        assert!(result.matches);
        assert!(result.stats.hash_distance.is_some());
        assert!(result.stats.ssim_score.is_some());
    }

    #[test]
    fn test_baseline_manager_paths() {
        let config = VisualRegressionConfig::default();
        let manager = BaselineManager::new(&config, Path::new("/project"));

        let baseline = manager.baseline_path("button", "desktop");
        assert!(baseline.to_string_lossy().contains("button_desktop.png"));

        let actual = manager.actual_path("button", "desktop");
        assert!(actual.to_string_lossy().contains("button_desktop.png"));

        let diff = manager.diff_path("button", "desktop");
        assert!(diff.to_string_lossy().contains("button_desktop_diff.png"));
    }

    #[test]
    fn test_baseline_manager_init() {
        let temp_dir = TempDir::new().unwrap();
        let config = VisualRegressionConfig::default();
        let manager = BaselineManager::new(&config, temp_dir.path());

        assert!(manager.init().is_ok());
        assert!(temp_dir.path().join(&config.baseline_dir).exists());
        assert!(temp_dir.path().join(&config.actual_dir).exists());
        assert!(temp_dir.path().join(&config.diff_dir).exists());
    }

    #[test]
    fn test_baseline_manager_save_and_load() {
        let temp_dir = TempDir::new().unwrap();
        let config = VisualRegressionConfig::default();
        let manager = BaselineManager::new(&config, temp_dir.path());
        manager.init().unwrap();

        let img = create_test_image(100, 100, [255, 0, 0, 255]);
        manager.save_baseline("test", "desktop", &img).unwrap();

        assert!(manager.has_baseline("test", "desktop"));
        let loaded = manager.load_baseline("test", "desktop").unwrap();
        assert_eq!(loaded.dimensions(), (100, 100));
    }

    #[test]
    fn test_screenshot_capture_placeholder() {
        let temp_dir = TempDir::new().unwrap();
        let capture = ScreenshotCapture::default();

        let path = capture.create_placeholder(
            100, 100,
            [255, 0, 0, 255],
            "test",
            temp_dir.path(),
        ).unwrap();

        assert!(path.exists());
        let img = image::open(&path).unwrap();
        assert_eq!(img.dimensions(), (100, 100));
    }

    #[test]
    fn test_visual_report_generator() {
        let temp_dir = TempDir::new().unwrap();
        let config = VisualRegressionConfig::default();
        let mut report = VisualRegressionReport::new(Path::new("/test"), &config);

        report.add_result(VisualRegressionResult {
            name: "button".to_string(),
            viewport: "desktop".to_string(),
            status: ComparisonStatus::Passed,
            stats: Some(DiffStats::identical(100, 100)),
            baseline_path: Some(PathBuf::from("/test/baseline.png")),
            actual_path: Some(PathBuf::from("/test/actual.png")),
            diff_path: Some(PathBuf::from("/test/diff.png")),
            error: None,
            comparison_duration_ms: 50,
        });

        let generator = VisualReportGenerator::new(temp_dir.path());
        let report_path = generator.generate(&report).unwrap();

        assert!(report_path.exists());
        let html = fs::read_to_string(&report_path).unwrap();
        assert!(html.contains("Visual Regression Report"));
        assert!(html.contains("button"));
    }

    #[test]
    fn test_github_actions_generation() {
        let workflow = generate_github_actions_visual();
        assert!(workflow.contains("Visual Regression"));
        assert!(workflow.contains("oxide visual-regression"));
    }

    #[test]
    fn test_gitlab_ci_generation() {
        let ci = generate_gitlab_ci_visual();
        assert!(ci.contains("visual-regression"));
        assert!(ci.contains("oxide visual-regression"));
    }

    #[test]
    fn test_visual_thresholds_default() {
        let thresholds = VisualThresholds::default();
        assert_eq!(thresholds.pixel_threshold_percent, 0.1);
        assert_eq!(thresholds.color_threshold, 5);
        assert_eq!(thresholds.hash_distance_threshold, 5);
        assert_eq!(thresholds.ssim_threshold, 0.99);
    }

    #[test]
    fn test_comparison_method_display() {
        assert_eq!(ComparisonMethod::PixelDiff.to_string(), "Pixel Diff");
        assert_eq!(ComparisonMethod::PerceptualHash.to_string(), "Perceptual Hash");
        assert_eq!(ComparisonMethod::Ssim.to_string(), "SSIM");
        assert_eq!(ComparisonMethod::Hybrid.to_string(), "Hybrid");
    }

    #[test]
    fn test_viewport_presets() {
        let desktop = Viewport::desktop();
        assert_eq!(desktop.width, 1280);
        assert_eq!(desktop.height, 720);

        let tablet = Viewport::tablet();
        assert_eq!(tablet.width, 768);
        assert_eq!(tablet.height, 1024);

        let mobile = Viewport::mobile();
        assert_eq!(mobile.width, 375);
        assert_eq!(mobile.height, 667);
    }

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_ignore_region() {
        let config = VisualRegressionConfig {
            ignore_regions: vec![
                IgnoreRegion {
                    name: "dynamic-area".to_string(),
                    x: RegionCoord::Pixels(10),
                    y: RegionCoord::Pixels(10),
                    width: RegionCoord::Pixels(50),
                    height: RegionCoord::Pixels(50),
                    selector: None,
                },
            ],
            ..Default::default()
        };
        let comparison = ImageComparison::new(&config);

        // Point inside region
        assert!(comparison.is_in_ignore_region(25, 25, 100, 100));
        // Point outside region
        assert!(!comparison.is_in_ignore_region(70, 70, 100, 100));
    }

    #[test]
    fn test_update_baselines() {
        let temp_dir = TempDir::new().unwrap();
        let config = VisualRegressionConfig::default();

        // Create actual directory with test image
        let actual_dir = temp_dir.path().join(&config.actual_dir);
        fs::create_dir_all(&actual_dir).unwrap();

        let img = create_test_image(100, 100, [255, 0, 0, 255]);
        img.save(actual_dir.join("button_desktop.png")).unwrap();

        let updated = update_baselines(temp_dir.path(), &config, None).unwrap();
        assert!(!updated.is_empty());

        // Verify baseline was created
        let baseline_dir = temp_dir.path().join(&config.baseline_dir);
        assert!(baseline_dir.join("button_desktop.png").exists());
    }

    #[test]
    fn test_config_snapshot() {
        let config = VisualRegressionConfig::default();
        let snapshot = ConfigSnapshot::from_config(&config);

        assert_eq!(snapshot.pixel_threshold_percent, config.thresholds.pixel_threshold_percent);
        assert_eq!(snapshot.color_threshold, config.thresholds.color_threshold);
    }

    #[test]
    fn test_report_failures_filter() {
        let config = VisualRegressionConfig::default();
        let mut report = VisualRegressionReport::new(Path::new("/test"), &config);

        report.add_result(VisualRegressionResult {
            name: "passed".to_string(),
            viewport: "desktop".to_string(),
            status: ComparisonStatus::Passed,
            stats: None,
            baseline_path: None,
            actual_path: None,
            diff_path: None,
            error: None,
            comparison_duration_ms: 0,
        });

        report.add_result(VisualRegressionResult {
            name: "failed".to_string(),
            viewport: "desktop".to_string(),
            status: ComparisonStatus::Failed,
            stats: None,
            baseline_path: None,
            actual_path: None,
            diff_path: None,
            error: None,
            comparison_duration_ms: 0,
        });

        let failures = report.failures();
        assert_eq!(failures.len(), 1);
        assert_eq!(failures[0].name, "failed");
    }
}
