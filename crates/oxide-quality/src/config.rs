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
