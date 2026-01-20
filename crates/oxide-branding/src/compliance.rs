//! Brand Compliance Checker
//!
//! Automated checking of brand guideline adherence for applications
//! and themes. Generates compliance reports with issues and recommendations.

use serde::{Deserialize, Serialize};

use crate::brand_pack::BrandPack;
use crate::app_pack::AppPack;
use crate::white_label::WhiteLabelConfig;
use crate::governance::TokenGovernance;

/// Brand compliance checker
#[derive(Debug)]
pub struct ComplianceChecker {
    /// Reference brand pack
    brand: BrandPack,

    /// Governance rules
    governance: TokenGovernance,

    /// Check configuration
    config: ComplianceConfig,
}

impl ComplianceChecker {
    /// Create a new compliance checker for a brand
    pub fn new(brand: &BrandPack) -> Self {
        Self {
            brand: brand.clone(),
            governance: TokenGovernance::from_brand(brand),
            config: ComplianceConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(brand: &BrandPack, config: ComplianceConfig) -> Self {
        Self {
            brand: brand.clone(),
            governance: TokenGovernance::from_brand(brand),
            config,
        }
    }

    /// Check compliance of an app pack
    pub fn check_app_pack(&self, app: &AppPack) -> ComplianceReport {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check color overrides
        for (name, color) in &app.colors.overrides {
            if let Some(brand_color) = self.get_brand_color(name) {
                if brand_color.locked {
                    issues.push(ComplianceIssue::new(
                        ComplianceLevel::Error,
                        format!("Locked color '{}' cannot be overridden", name),
                        format!("colors.{}", name),
                    ).with_suggestion(format!(
                        "Remove the override for '{}' or use a different color name",
                        name
                    )));
                } else if self.config.check_color_contrast {
                    // Check contrast compliance
                    if let Some(warning) = self.check_color_contrast(&color.value, name) {
                        warnings.push(warning);
                    }
                }
            }
        }

        // Check typography overrides
        if let Some(ref _font) = app.typography.primary_family {
            if self.brand.typography.primary_family.locked {
                issues.push(ComplianceIssue::new(
                    ComplianceLevel::Error,
                    "Primary font family is locked and cannot be overridden",
                    "typography.primary_family",
                ));
            }
        }

        // Check asset compliance
        if self.config.check_assets {
            self.check_asset_compliance(app, &mut issues, &mut warnings);
        }

        // Check theme compliance
        for (theme_name, theme) in &app.themes {
            self.check_theme_compliance(theme_name, theme, &mut issues, &mut warnings);
        }

        // Combine issues and warnings
        let mut all_issues = issues;
        all_issues.extend(warnings);

        ComplianceReport::new(all_issues)
    }

    /// Check compliance of a white-label configuration
    pub fn check_white_label(&self, config: &WhiteLabelConfig) -> ComplianceReport {
        let mut issues = Vec::new();

        // Check color overrides
        for (name, color) in &config.overrides.colors {
            if let Some(brand_color) = self.get_brand_color(name) {
                if brand_color.locked {
                    issues.push(ComplianceIssue::new(
                        ComplianceLevel::Error,
                        format!("Locked brand color '{}' cannot be overridden in white-label config", name),
                        format!("overrides.colors.{}", name),
                    ));
                }
            }

            // Validate color format
            if !self.is_valid_color(color) {
                issues.push(ComplianceIssue::new(
                    ComplianceLevel::Error,
                    format!("Invalid color format for '{}': {}", name, color),
                    format!("overrides.colors.{}", name),
                ).with_suggestion("Use hex format like #FF5500 or #FF550080"));
            }
        }

        // Check logo compliance
        if config.overrides.logos.primary.is_some() {
            if let Some(ref _logo) = self.brand.identity.primary_logo {
                if let Some(ref guidelines) = self.brand.guidelines.logo {
                    issues.push(ComplianceIssue::new(
                        ComplianceLevel::Warning,
                        "Logo override detected - ensure it meets brand guidelines",
                        "overrides.logos.primary",
                    ).with_details(format!(
                        "Minimum size: {:?}px, Clear space: {:?}%",
                        guidelines.min_size,
                        guidelines.clear_space.map(|s| s * 100.0)
                    )));
                }
            }
        }

        // Check identity compliance
        if self.config.check_naming {
            self.check_naming_compliance(&config.identity, &mut issues);
        }

        ComplianceReport::new(issues)
    }

    /// Quick compliance check - returns pass/fail
    pub fn is_compliant(&self, app: &AppPack) -> bool {
        let report = self.check_app_pack(app);
        report.passed()
    }

    fn get_brand_color(&self, name: &str) -> Option<&crate::brand_pack::BrandColor> {
        match name {
            "primary" => Some(&self.brand.colors.primary),
            "secondary" => Some(&self.brand.colors.secondary),
            "accent" => Some(&self.brand.colors.accent),
            _ => self.brand.colors.custom.get(name),
        }
    }

    fn check_color_contrast(&self, _color: &str, name: &str) -> Option<ComplianceIssue> {
        // Simplified contrast check - in production would calculate WCAG contrast ratios
        Some(ComplianceIssue::new(
            ComplianceLevel::Info,
            format!("Consider verifying contrast ratio for color '{}'", name),
            format!("colors.{}", name),
        ).with_suggestion("Ensure 4.5:1 contrast ratio for normal text, 3:1 for large text"))
    }

    fn check_asset_compliance(&self, app: &AppPack, issues: &mut Vec<ComplianceIssue>, _warnings: &mut Vec<ComplianceIssue>) {
        // Check app icon
        if app.assets.app_icon.is_none() && self.brand.identity.icon.is_none() {
            issues.push(ComplianceIssue::new(
                ComplianceLevel::Warning,
                "No app icon defined - consider adding one",
                "assets.app_icon",
            ));
        }

        // Check for required assets
        if self.config.required_assets.contains(&"splash".to_string()) {
            if app.assets.splash.is_none() {
                issues.push(ComplianceIssue::new(
                    ComplianceLevel::Warning,
                    "Splash screen asset is recommended but not defined",
                    "assets.splash",
                ));
            }
        }
    }

    fn check_theme_compliance(
        &self,
        theme_name: &str,
        theme: &crate::app_pack::ThemeOverrides,
        issues: &mut Vec<ComplianceIssue>,
        _warnings: &mut Vec<ComplianceIssue>,
    ) {
        for (token, _value) in &theme.colors {
            if !self.governance.can_override(token) {
                issues.push(ComplianceIssue::new(
                    ComplianceLevel::Error,
                    format!("Token '{}' is locked and cannot be overridden in theme '{}'", token, theme_name),
                    format!("themes.{}.colors.{}", theme_name, token),
                ));
            }
        }
    }

    fn check_naming_compliance(&self, identity: &crate::white_label::WhiteLabelIdentity, issues: &mut Vec<ComplianceIssue>) {
        // Check for brand name usage in white-label
        let brand_name = &self.brand.identity.name;
        if identity.app_name.to_lowercase().contains(&brand_name.to_lowercase()) {
            issues.push(ComplianceIssue::new(
                ComplianceLevel::Warning,
                format!("White-label app name contains original brand name '{}'", brand_name),
                "identity.app_name",
            ).with_suggestion("Consider using a unique name for white-label deployment"));
        }
    }

    fn is_valid_color(&self, color: &str) -> bool {
        if !color.starts_with('#') {
            return false;
        }

        let hex = &color[1..];
        if hex.len() != 6 && hex.len() != 8 {
            return false;
        }

        hex.chars().all(|c| c.is_ascii_hexdigit())
    }
}

/// Compliance check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Check color contrast ratios
    #[serde(default = "default_true")]
    pub check_color_contrast: bool,

    /// Check asset presence and guidelines
    #[serde(default = "default_true")]
    pub check_assets: bool,

    /// Check naming conventions
    #[serde(default)]
    pub check_naming: bool,

    /// Fail on warnings
    #[serde(default)]
    pub fail_on_warnings: bool,

    /// Required assets
    #[serde(default)]
    pub required_assets: Vec<String>,

    /// Custom validation rules
    #[serde(default)]
    pub custom_rules: Vec<CustomRule>,
}

fn default_true() -> bool {
    true
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            check_color_contrast: true,
            check_assets: true,
            check_naming: false,
            fail_on_warnings: false,
            required_assets: Vec::new(),
            custom_rules: Vec::new(),
        }
    }
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomRule {
    /// Rule name
    pub name: String,

    /// Token path pattern
    pub pattern: String,

    /// Validation type
    pub validation: ValidationType,

    /// Expected value or constraint
    pub expected: serde_json::Value,
}

/// Validation types for custom rules
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    Equals,
    NotEquals,
    Contains,
    StartsWith,
    EndsWith,
    Matches,
    Range,
}

/// Compliance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    /// Issue severity level
    pub level: ComplianceLevel,

    /// Issue message
    pub message: String,

    /// Token/path this issue relates to
    pub path: String,

    /// Suggested fix
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggestion: Option<String>,

    /// Additional details
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,

    /// Related brand guideline
    #[serde(skip_serializing_if = "Option::is_none")]
    pub guideline: Option<String>,
}

impl ComplianceIssue {
    pub fn new(level: ComplianceLevel, message: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            path: path.into(),
            suggestion: None,
            details: None,
            guideline: None,
        }
    }

    pub fn with_suggestion(mut self, suggestion: impl Into<String>) -> Self {
        self.suggestion = Some(suggestion.into());
        self
    }

    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }

    pub fn with_guideline(mut self, guideline: impl Into<String>) -> Self {
        self.guideline = Some(guideline.into());
        self
    }

    pub fn is_error(&self) -> bool {
        matches!(self.level, ComplianceLevel::Error)
    }

    pub fn is_warning(&self) -> bool {
        matches!(self.level, ComplianceLevel::Warning)
    }
}

/// Compliance severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ComplianceLevel {
    /// Critical issue - blocks deployment
    Error,
    /// Warning - should be addressed
    Warning,
    /// Informational - best practice suggestion
    Info,
}

/// Compliance check report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// All issues found
    pub issues: Vec<ComplianceIssue>,

    /// Overall status
    pub status: ComplianceStatus,

    /// Summary counts
    pub summary: ComplianceSummary,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl ComplianceReport {
    pub fn new(issues: Vec<ComplianceIssue>) -> Self {
        let error_count = issues.iter().filter(|i| i.is_error()).count();
        let warning_count = issues.iter().filter(|i| i.is_warning()).count();
        let info_count = issues.iter().filter(|i| matches!(i.level, ComplianceLevel::Info)).count();

        let status = if error_count > 0 {
            ComplianceStatus::Failed
        } else if warning_count > 0 {
            ComplianceStatus::PassedWithWarnings
        } else {
            ComplianceStatus::Passed
        };

        Self {
            issues,
            status,
            summary: ComplianceSummary {
                total: error_count + warning_count + info_count,
                errors: error_count,
                warnings: warning_count,
                info: info_count,
            },
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn passed(&self) -> bool {
        matches!(self.status, ComplianceStatus::Passed | ComplianceStatus::PassedWithWarnings)
    }

    pub fn errors(&self) -> Vec<&ComplianceIssue> {
        self.issues.iter().filter(|i| i.is_error()).collect()
    }

    pub fn warnings(&self) -> Vec<&ComplianceIssue> {
        self.issues.iter().filter(|i| i.is_warning()).collect()
    }

    /// Format report as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("\nBrand Compliance Report\n"));
        output.push_str(&format!("========================\n"));
        output.push_str(&format!("Status: {:?}\n", self.status));
        output.push_str(&format!(
            "Issues: {} errors, {} warnings, {} info\n\n",
            self.summary.errors, self.summary.warnings, self.summary.info
        ));

        if !self.issues.is_empty() {
            for issue in &self.issues {
                let level = match issue.level {
                    ComplianceLevel::Error => "ERROR",
                    ComplianceLevel::Warning => "WARNING",
                    ComplianceLevel::Info => "INFO",
                };
                output.push_str(&format!("[{}] {}\n", level, issue.message));
                output.push_str(&format!("  Path: {}\n", issue.path));
                if let Some(ref suggestion) = issue.suggestion {
                    output.push_str(&format!("  Suggestion: {}\n", suggestion));
                }
                output.push('\n');
            }
        } else {
            output.push_str("No issues found. All brand guidelines are satisfied.\n");
        }

        output
    }

    /// Convert to JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

/// Compliance status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    Passed,
    PassedWithWarnings,
    Failed,
}

/// Summary of compliance issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceSummary {
    pub total: usize,
    pub errors: usize,
    pub warnings: usize,
    pub info: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::brand_pack::BrandColor;

    #[test]
    fn test_compliance_checker_creation() {
        let brand = BrandPack::default();
        let checker = ComplianceChecker::new(&brand);
        assert!(checker.config.check_color_contrast);
    }

    #[test]
    fn test_compliant_app() {
        let brand = BrandPack::default();
        let app = AppPack::new("test-app");

        let checker = ComplianceChecker::new(&brand);
        let report = checker.check_app_pack(&app);

        assert!(report.passed());
    }

    #[test]
    fn test_locked_color_violation() {
        let mut brand = BrandPack::default();
        brand.colors.primary = BrandColor::new("#FF0000").locked();

        let mut app = AppPack::new("test-app");
        app.colors.overrides.insert("primary".into(), BrandColor::new("#00FF00"));

        let checker = ComplianceChecker::new(&brand);
        let report = checker.check_app_pack(&app);

        assert!(!report.passed());
        assert!(report.summary.errors > 0);
    }

    #[test]
    fn test_report_format() {
        let issues = vec![
            ComplianceIssue::new(ComplianceLevel::Error, "Test error", "test.path"),
            ComplianceIssue::new(ComplianceLevel::Warning, "Test warning", "test.path2"),
        ];

        let report = ComplianceReport::new(issues);
        let formatted = report.format();

        assert!(formatted.contains("ERROR"));
        assert!(formatted.contains("WARNING"));
        assert!(formatted.contains("Test error"));
    }

    #[test]
    fn test_valid_color() {
        let brand = BrandPack::default();
        let checker = ComplianceChecker::new(&brand);

        assert!(checker.is_valid_color("#FF5500"));
        assert!(checker.is_valid_color("#FF550080"));
        assert!(!checker.is_valid_color("FF5500"));
        assert!(!checker.is_valid_color("#GG5500"));
        assert!(!checker.is_valid_color("#FFF"));
    }
}
