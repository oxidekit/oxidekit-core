//! Quality gate configuration
//!
//! Defines configuration structures for all quality checks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Main quality gates configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Lint configuration
    #[serde(default)]
    pub lint: LintConfig,

    /// Accessibility configuration
    #[serde(default)]
    pub a11y: A11yConfig,

    /// Performance configuration
    #[serde(default)]
    pub perf: PerfConfig,

    /// Security configuration
    #[serde(default)]
    pub security: SecurityConfig,

    /// Bundle configuration
    #[serde(default)]
    pub bundle: BundleConfig,

    /// CI/CD configuration
    #[serde(default)]
    pub ci: CiConfig,

    /// Visual regression testing configuration
    #[serde(default)]
    pub visual_regression: VisualRegressionConfig,

    /// Paths to exclude from checks
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Whether to fail on warnings
    #[serde(default)]
    pub strict: bool,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            lint: LintConfig::default(),
            a11y: A11yConfig::default(),
            perf: PerfConfig::default(),
            security: SecurityConfig::default(),
            bundle: BundleConfig::default(),
            ci: CiConfig::default(),
            visual_regression: VisualRegressionConfig::default(),
            exclude: vec![
                "target/**".to_string(),
                "node_modules/**".to_string(),
                "**/*.min.js".to_string(),
            ],
            strict: false,
        }
    }
}

impl QualityConfig {
    /// Load configuration from a file
    pub fn from_file(path: &Path) -> Result<Self, crate::QualityError> {
        let content = std::fs::read_to_string(path)?;
        let config: Self = toml::from_str(&content)
            .map_err(|e| crate::QualityError::Config(e.to_string()))?;
        Ok(config)
    }

    /// Load configuration from project directory
    /// Looks for oxide-quality.toml, .oxide/quality.toml, or oxide.toml
    pub fn from_project(project_path: &Path) -> Self {
        let candidates = [
            project_path.join("oxide-quality.toml"),
            project_path.join(".oxide/quality.toml"),
            project_path.join("oxide.toml"),
        ];

        for candidate in &candidates {
            if candidate.exists() {
                if let Ok(config) = Self::from_file(candidate) {
                    tracing::info!("Loaded quality config from {:?}", candidate);
                    return config;
                }
            }
        }

        tracing::debug!("Using default quality config");
        Self::default()
    }

    /// Save configuration to a file
    pub fn save(&self, path: &Path) -> Result<(), crate::QualityError> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::QualityError::Config(e.to_string()))?;
        std::fs::write(path, content)?;
        Ok(())
    }
}

/// Lint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintConfig {
    /// Enable linting
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Rules to enable (empty = all)
    #[serde(default)]
    pub rules: Vec<String>,

    /// Rules to disable
    #[serde(default)]
    pub disable: Vec<String>,

    /// Rule severity overrides
    #[serde(default)]
    pub severity: HashMap<String, String>,

    /// File patterns to lint
    #[serde(default = "default_oui_patterns")]
    pub include: Vec<String>,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Maximum allowed warnings before failure
    #[serde(default)]
    pub max_warnings: Option<usize>,
}

impl Default for LintConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            rules: Vec::new(),
            disable: Vec::new(),
            severity: HashMap::new(),
            include: default_oui_patterns(),
            exclude: Vec::new(),
            max_warnings: None,
        }
    }
}

/// Accessibility configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yConfig {
    /// Enable accessibility checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// WCAG compliance level
    #[serde(default)]
    pub wcag_level: WcagLevel,

    /// Check keyboard navigation
    #[serde(default = "default_true")]
    pub check_keyboard: bool,

    /// Check focus visibility
    #[serde(default = "default_true")]
    pub check_focus: bool,

    /// Check color contrast
    #[serde(default = "default_true")]
    pub check_contrast: bool,

    /// Check ARIA roles and labels
    #[serde(default = "default_true")]
    pub check_aria: bool,

    /// Minimum contrast ratio for normal text
    #[serde(default = "default_contrast_normal")]
    pub min_contrast_normal: f64,

    /// Minimum contrast ratio for large text
    #[serde(default = "default_contrast_large")]
    pub min_contrast_large: f64,

    /// Components exempt from checks
    #[serde(default)]
    pub exempt: Vec<String>,
}

impl Default for A11yConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            wcag_level: WcagLevel::AA,
            check_keyboard: true,
            check_focus: true,
            check_contrast: true,
            check_aria: true,
            min_contrast_normal: 4.5,
            min_contrast_large: 3.0,
            exempt: Vec::new(),
        }
    }
}

/// WCAG compliance levels
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum WcagLevel {
    /// Level A - minimum compliance
    A,
    /// Level AA - recommended compliance
    #[default]
    AA,
    /// Level AAA - highest compliance
    AAA,
}

impl std::fmt::Display for WcagLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WcagLevel::A => write!(f, "A"),
            WcagLevel::AA => write!(f, "AA"),
            WcagLevel::AAA => write!(f, "AAA"),
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfConfig {
    /// Enable performance checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Frame time budget in milliseconds (target 60fps = 16ms)
    #[serde(default = "default_frame_budget")]
    pub frame_budget_ms: f64,

    /// Layout pass time budget in milliseconds
    #[serde(default = "default_layout_budget")]
    pub layout_budget_ms: f64,

    /// Text shaping time budget in milliseconds
    #[serde(default = "default_text_budget")]
    pub text_budget_ms: f64,

    /// Maximum allocations per frame
    #[serde(default = "default_alloc_budget")]
    pub max_allocs_per_frame: usize,

    /// Memory usage budgets by app class
    #[serde(default)]
    pub memory_budgets: MemoryBudgets,

    /// Enable benchmark validation
    #[serde(default)]
    pub validate_benchmarks: bool,
}

impl Default for PerfConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            frame_budget_ms: 16.0,
            layout_budget_ms: 4.0,
            text_budget_ms: 2.0,
            max_allocs_per_frame: 1000,
            memory_budgets: MemoryBudgets::default(),
            validate_benchmarks: false,
        }
    }
}

/// Memory budgets for different app classes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryBudgets {
    /// Simple app (e.g., settings panel)
    #[serde(default = "default_mem_simple")]
    pub simple_mb: usize,
    /// Standard app (e.g., admin dashboard)
    #[serde(default = "default_mem_standard")]
    pub standard_mb: usize,
    /// Complex app (e.g., IDE, large data tables)
    #[serde(default = "default_mem_complex")]
    pub complex_mb: usize,
}

impl Default for MemoryBudgets {
    fn default() -> Self {
        Self {
            simple_mb: 64,
            standard_mb: 256,
            complex_mb: 512,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable security checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Audit dependencies
    #[serde(default = "default_true")]
    pub audit_deps: bool,

    /// Check for unsafe code usage
    #[serde(default = "default_true")]
    pub check_unsafe: bool,

    /// Allowed unsafe crates (if check_unsafe is enabled)
    #[serde(default)]
    pub allow_unsafe_crates: Vec<String>,

    /// Check permission declarations
    #[serde(default = "default_true")]
    pub check_permissions: bool,

    /// Required permissions for the project
    #[serde(default)]
    pub required_permissions: Vec<String>,

    /// Forbidden APIs
    #[serde(default)]
    pub forbidden_apis: Vec<String>,

    /// Maximum allowed vulnerabilities by severity
    #[serde(default)]
    pub max_vulnerabilities: VulnerabilityLimits,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            audit_deps: true,
            check_unsafe: true,
            allow_unsafe_crates: vec![
                "std".to_string(),
                "core".to_string(),
            ],
            check_permissions: true,
            required_permissions: Vec::new(),
            forbidden_apis: vec![
                "std::process::Command".to_string(),
                "std::env::set_var".to_string(),
            ],
            max_vulnerabilities: VulnerabilityLimits::default(),
        }
    }
}

/// Vulnerability severity limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VulnerabilityLimits {
    /// Maximum critical vulnerabilities (0 = none allowed)
    #[serde(default)]
    pub critical: usize,
    /// Maximum high severity vulnerabilities
    #[serde(default)]
    pub high: usize,
    /// Maximum medium severity vulnerabilities
    #[serde(default = "default_five")]
    pub medium: usize,
    /// Maximum low severity vulnerabilities
    #[serde(default = "default_ten")]
    pub low: usize,
}

impl Default for VulnerabilityLimits {
    fn default() -> Self {
        Self {
            critical: 0,
            high: 0,
            medium: 5,
            low: 10,
        }
    }
}

/// Bundle configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleConfig {
    /// Enable bundle checks
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum bundle size in bytes
    #[serde(default = "default_bundle_size")]
    pub max_size_bytes: u64,

    /// Maximum individual file size in bytes
    #[serde(default = "default_file_size")]
    pub max_file_size_bytes: u64,

    /// Track size changes
    #[serde(default = "default_true")]
    pub track_changes: bool,

    /// Size change threshold percentage for warning
    #[serde(default = "default_size_threshold")]
    pub warning_threshold_percent: f64,

    /// Size change threshold percentage for failure
    #[serde(default = "default_size_fail")]
    pub fail_threshold_percent: f64,

    /// Generate SBOM (Software Bill of Materials)
    #[serde(default)]
    pub generate_sbom: bool,

    /// Files to include in analysis
    #[serde(default)]
    pub include: Vec<String>,

    /// Files to exclude from analysis
    #[serde(default)]
    pub exclude: Vec<String>,
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_file_size_bytes: 1024 * 1024,  // 1 MB
            track_changes: true,
            warning_threshold_percent: 5.0,
            fail_threshold_percent: 20.0,
            generate_sbom: false,
            include: Vec::new(),
            exclude: vec![
                "*.map".to_string(),
                "*.pdb".to_string(),
            ],
        }
    }
}

/// CI/CD configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiConfig {
    /// Generate GitHub Actions workflow
    #[serde(default)]
    pub github_actions: bool,

    /// Generate GitLab CI config
    #[serde(default)]
    pub gitlab_ci: bool,

    /// Fail CI on warnings
    #[serde(default)]
    pub fail_on_warnings: bool,

    /// Upload reports as artifacts
    #[serde(default = "default_true")]
    pub upload_reports: bool,

    /// Comment on PRs
    #[serde(default = "default_true")]
    pub pr_comments: bool,

    /// Report format (json, markdown, text)
    #[serde(default)]
    pub report_format: ReportFormat,
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            github_actions: false,
            gitlab_ci: false,
            fail_on_warnings: false,
            upload_reports: true,
            pr_comments: true,
            report_format: ReportFormat::default(),
        }
    }
}

/// Report output format
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    #[default]
    Json,
    Markdown,
    Text,
}

/// Visual regression testing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualRegressionConfig {
    /// Enable visual regression testing
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Directory for baseline screenshots
    #[serde(default = "default_baseline_dir")]
    pub baseline_dir: String,

    /// Directory for actual screenshots during tests
    #[serde(default = "default_actual_dir")]
    pub actual_dir: String,

    /// Directory for diff output
    #[serde(default = "default_diff_dir")]
    pub diff_dir: String,

    /// Directory for HTML reports
    #[serde(default = "default_report_dir")]
    pub report_dir: String,

    /// Threshold configuration
    #[serde(default)]
    pub thresholds: VisualThresholds,

    /// Comparison method
    #[serde(default)]
    pub comparison_method: ComparisonMethod,

    /// Anti-aliasing tolerance (pixels)
    #[serde(default = "default_aa_tolerance")]
    pub anti_aliasing_tolerance: u32,

    /// Ignore regions (CSS selector patterns)
    #[serde(default)]
    pub ignore_regions: Vec<IgnoreRegion>,

    /// Update baselines automatically on failure (for development)
    #[serde(default)]
    pub auto_update_baselines: bool,

    /// Screenshot capture settings
    #[serde(default)]
    pub capture: CaptureSettings,

    /// File patterns to include for component screenshots
    #[serde(default = "default_screenshot_patterns")]
    pub include: Vec<String>,

    /// File patterns to exclude
    #[serde(default)]
    pub exclude: Vec<String>,

    /// Fail CI on visual regression
    #[serde(default = "default_true")]
    pub fail_on_regression: bool,

    /// Generate HTML report
    #[serde(default = "default_true")]
    pub generate_report: bool,
}

impl Default for VisualRegressionConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            baseline_dir: default_baseline_dir(),
            actual_dir: default_actual_dir(),
            diff_dir: default_diff_dir(),
            report_dir: default_report_dir(),
            thresholds: VisualThresholds::default(),
            comparison_method: ComparisonMethod::default(),
            anti_aliasing_tolerance: 2,
            ignore_regions: Vec::new(),
            auto_update_baselines: false,
            capture: CaptureSettings::default(),
            include: default_screenshot_patterns(),
            exclude: Vec::new(),
            fail_on_regression: true,
            generate_report: true,
        }
    }
}

/// Visual comparison thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualThresholds {
    /// Maximum percentage of different pixels allowed (0.0 - 100.0)
    #[serde(default = "default_pixel_threshold")]
    pub pixel_threshold_percent: f64,

    /// Maximum allowed pixel color difference (0 - 255)
    #[serde(default = "default_color_threshold")]
    pub color_threshold: u8,

    /// Perceptual hash distance threshold (0 = identical, higher = more different)
    #[serde(default = "default_hash_threshold")]
    pub hash_distance_threshold: u32,

    /// Structural similarity threshold (0.0 - 1.0, 1.0 = identical)
    #[serde(default = "default_ssim_threshold")]
    pub ssim_threshold: f64,

    /// Warning threshold (percentage of pixel_threshold_percent)
    #[serde(default = "default_warning_percent")]
    pub warning_at_percent: f64,
}

impl Default for VisualThresholds {
    fn default() -> Self {
        Self {
            pixel_threshold_percent: 0.1,     // 0.1% pixel difference allowed
            color_threshold: 5,                // 5 units color difference tolerance
            hash_distance_threshold: 5,        // Perceptual hash distance
            ssim_threshold: 0.99,              // 99% structural similarity required
            warning_at_percent: 50.0,          // Warn at 50% of threshold
        }
    }
}

/// Comparison method for visual regression
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonMethod {
    /// Pixel-by-pixel comparison
    #[default]
    PixelDiff,
    /// Perceptual hashing comparison
    PerceptualHash,
    /// Structural similarity index
    Ssim,
    /// Hybrid: uses multiple methods
    Hybrid,
}

impl std::fmt::Display for ComparisonMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComparisonMethod::PixelDiff => write!(f, "Pixel Diff"),
            ComparisonMethod::PerceptualHash => write!(f, "Perceptual Hash"),
            ComparisonMethod::Ssim => write!(f, "SSIM"),
            ComparisonMethod::Hybrid => write!(f, "Hybrid"),
        }
    }
}

/// Region to ignore during comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IgnoreRegion {
    /// Region name/description
    pub name: String,
    /// X coordinate (pixels or percentage)
    pub x: RegionCoord,
    /// Y coordinate (pixels or percentage)
    pub y: RegionCoord,
    /// Width (pixels or percentage)
    pub width: RegionCoord,
    /// Height (pixels or percentage)
    pub height: RegionCoord,
    /// CSS selector to auto-detect region (optional)
    #[serde(default)]
    pub selector: Option<String>,
}

/// Region coordinate value
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum RegionCoord {
    Pixels(u32),
    Percent(f64),
}

impl Default for RegionCoord {
    fn default() -> Self {
        RegionCoord::Pixels(0)
    }
}

impl RegionCoord {
    /// Convert to pixels given a dimension
    pub fn to_pixels(&self, dimension: u32) -> u32 {
        match self {
            RegionCoord::Pixels(px) => *px,
            RegionCoord::Percent(pct) => ((pct / 100.0) * dimension as f64) as u32,
        }
    }
}

/// Screenshot capture settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureSettings {
    /// Default viewport width
    #[serde(default = "default_viewport_width")]
    pub viewport_width: u32,
    /// Default viewport height
    #[serde(default = "default_viewport_height")]
    pub viewport_height: u32,
    /// Device pixel ratio (for retina displays)
    #[serde(default = "default_pixel_ratio")]
    pub device_pixel_ratio: f32,
    /// Wait time before capture (milliseconds)
    #[serde(default = "default_wait_ms")]
    pub wait_before_capture_ms: u64,
    /// Capture format
    #[serde(default)]
    pub format: ImageFormat,
    /// PNG compression level (0-9)
    #[serde(default = "default_compression")]
    pub compression_level: u32,
    /// Enable full-page screenshots
    #[serde(default)]
    pub full_page: bool,
    /// Multiple viewport sizes to test
    #[serde(default)]
    pub viewports: Vec<Viewport>,
}

impl Default for CaptureSettings {
    fn default() -> Self {
        Self {
            viewport_width: 1280,
            viewport_height: 720,
            device_pixel_ratio: 1.0,
            wait_before_capture_ms: 100,
            format: ImageFormat::Png,
            compression_level: 6,
            full_page: false,
            viewports: vec![
                Viewport::desktop(),
                Viewport::tablet(),
                Viewport::mobile(),
            ],
        }
    }
}

/// Viewport configuration for responsive testing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Viewport {
    /// Viewport name
    pub name: String,
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// Device pixel ratio
    #[serde(default = "default_pixel_ratio")]
    pub device_pixel_ratio: f32,
}

impl Viewport {
    /// Desktop viewport (1280x720)
    pub fn desktop() -> Self {
        Self {
            name: "desktop".to_string(),
            width: 1280,
            height: 720,
            device_pixel_ratio: 1.0,
        }
    }

    /// Tablet viewport (768x1024)
    pub fn tablet() -> Self {
        Self {
            name: "tablet".to_string(),
            width: 768,
            height: 1024,
            device_pixel_ratio: 2.0,
        }
    }

    /// Mobile viewport (375x667)
    pub fn mobile() -> Self {
        Self {
            name: "mobile".to_string(),
            width: 375,
            height: 667,
            device_pixel_ratio: 3.0,
        }
    }

    /// Create a custom viewport
    pub fn custom(name: &str, width: u32, height: u32) -> Self {
        Self {
            name: name.to_string(),
            width,
            height,
            device_pixel_ratio: 1.0,
        }
    }
}

/// Image format for screenshots
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImageFormat {
    #[default]
    Png,
    Jpeg,
    WebP,
}

impl ImageFormat {
    /// Get file extension
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::WebP => "webp",
        }
    }
}

// Default value helpers
fn default_true() -> bool { true }
fn default_oui_patterns() -> Vec<String> { vec!["**/*.oui".to_string()] }
fn default_contrast_normal() -> f64 { 4.5 }
fn default_contrast_large() -> f64 { 3.0 }
fn default_frame_budget() -> f64 { 16.0 }
fn default_layout_budget() -> f64 { 4.0 }
fn default_text_budget() -> f64 { 2.0 }
fn default_alloc_budget() -> usize { 1000 }
fn default_mem_simple() -> usize { 64 }
fn default_mem_standard() -> usize { 256 }
fn default_mem_complex() -> usize { 512 }
fn default_five() -> usize { 5 }
fn default_ten() -> usize { 10 }
fn default_bundle_size() -> u64 { 10 * 1024 * 1024 }
fn default_file_size() -> u64 { 1024 * 1024 }
fn default_size_threshold() -> f64 { 5.0 }
fn default_size_fail() -> f64 { 20.0 }
fn default_baseline_dir() -> String { ".visual-regression/baseline".to_string() }
fn default_actual_dir() -> String { ".visual-regression/actual".to_string() }
fn default_diff_dir() -> String { ".visual-regression/diff".to_string() }
fn default_report_dir() -> String { ".visual-regression/reports".to_string() }
fn default_pixel_threshold() -> f64 { 0.1 }
fn default_color_threshold() -> u8 { 5 }
fn default_hash_threshold() -> u32 { 5 }
fn default_ssim_threshold() -> f64 { 0.99 }
fn default_warning_percent() -> f64 { 50.0 }
fn default_aa_tolerance() -> u32 { 2 }
fn default_viewport_width() -> u32 { 1280 }
fn default_viewport_height() -> u32 { 720 }
fn default_pixel_ratio() -> f32 { 1.0 }
fn default_wait_ms() -> u64 { 100 }
fn default_compression() -> u32 { 6 }
fn default_screenshot_patterns() -> Vec<String> { vec!["**/*.oui".to_string(), "**/*.component.rs".to_string()] }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = QualityConfig::default();
        assert!(config.lint.enabled);
        assert!(config.a11y.enabled);
        assert_eq!(config.a11y.wcag_level, WcagLevel::AA);
    }

    #[test]
    fn test_config_serialization() {
        let config = QualityConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[lint]"));
        assert!(toml_str.contains("[a11y]"));
    }

    #[test]
    fn test_wcag_level_display() {
        assert_eq!(WcagLevel::A.to_string(), "A");
        assert_eq!(WcagLevel::AA.to_string(), "AA");
        assert_eq!(WcagLevel::AAA.to_string(), "AAA");
    }
}
