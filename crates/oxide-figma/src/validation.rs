//! Validation and Quality Gates
//!
//! Validates generated output against:
//! - Component schemas
//! - Token usage (no hardcoded values)
//! - Accessibility checks (contrast, text size)
//! - Performance warnings (deep nesting, heavy layouts)

use crate::components::ComponentMapping;
use crate::layout::OxideLayout;
use crate::tokens::ExtractedTokens;
use crate::types::Color;
use oxide_components::Theme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};

/// Validator for generated output
#[derive(Debug)]
pub struct Validator {
    config: ValidatorConfig,
}

/// Configuration for validation
#[derive(Debug, Clone)]
pub struct ValidatorConfig {
    /// Minimum contrast ratio for text (WCAG AA = 4.5)
    pub min_contrast_ratio: f32,

    /// Minimum font size (px)
    pub min_font_size: f32,

    /// Maximum layout nesting depth
    pub max_nesting_depth: usize,

    /// Whether to enforce token usage
    pub enforce_tokens: bool,

    /// Whether to check accessibility
    pub check_accessibility: bool,

    /// Whether to check performance
    pub check_performance: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            min_contrast_ratio: 4.5, // WCAG AA
            min_font_size: 12.0,
            max_nesting_depth: 10,
            enforce_tokens: true,
            check_accessibility: true,
            check_performance: true,
        }
    }
}

/// Validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Validation errors (must fix)
    pub errors: Vec<ValidationIssue>,

    /// Validation warnings (should fix)
    pub warnings: Vec<ValidationIssue>,

    /// Validation info (suggestions)
    pub info: Vec<ValidationIssue>,

    /// Overall validation status
    pub status: ValidationStatus,

    /// Accessibility score (0-100)
    pub accessibility_score: u32,

    /// Performance score (0-100)
    pub performance_score: u32,
}

/// A validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Issue type
    pub issue_type: IssueType,

    /// Severity
    pub severity: Severity,

    /// Human-readable message
    pub message: String,

    /// Location (node ID, file path, etc.)
    pub location: Option<String>,

    /// Suggested fix
    pub suggestion: Option<String>,

    /// Related documentation URL
    pub docs_url: Option<String>,
}

/// Issue type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IssueType {
    // Accessibility
    InsufficientContrast,
    TextTooSmall,
    MissingAltText,
    MissingLabel,
    ColorAlone,

    // Token usage
    HardcodedColor,
    HardcodedSpacing,
    HardcodedFont,
    UndefinedToken,

    // Component
    InvalidProp,
    MissingRequiredProp,
    UnknownComponent,
    IncompatibleVariant,

    // Layout
    DeepNesting,
    LargeChildCount,
    MissingConstraints,
    OverlappingElements,

    // Performance
    LargeImageSize,
    TooManyShadows,
    ComplexGradient,
}

/// Issue severity
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
    Info,
}

/// Overall validation status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    Valid,
    ValidWithWarnings,
    Invalid,
}

impl Validator {
    /// Create a new validator
    pub fn new() -> Self {
        Self::with_config(ValidatorConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: ValidatorConfig) -> Self {
        Self { config }
    }

    /// Validate a theme
    pub fn validate_theme(&self, theme: &Theme) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        // Check color contrast
        if self.config.check_accessibility {
            self.check_color_contrast(theme, &mut errors, &mut warnings);
        }

        // Check token completeness
        self.check_token_completeness(theme, &mut warnings, &mut info);

        // Calculate scores
        let accessibility_score = self.calculate_accessibility_score(&errors, &warnings);
        let performance_score = 100; // Theme itself doesn't have performance issues

        let status = if !errors.is_empty() {
            ValidationStatus::Invalid
        } else if !warnings.is_empty() {
            ValidationStatus::ValidWithWarnings
        } else {
            ValidationStatus::Valid
        };

        ValidationReport {
            errors,
            warnings,
            info,
            status,
            accessibility_score,
            performance_score,
        }
    }

    /// Validate extracted tokens
    pub fn validate_tokens(&self, tokens: &ExtractedTokens) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        // Check for sufficient colors
        if tokens.colors.len() < 5 {
            warnings.push(ValidationIssue {
                issue_type: IssueType::UndefinedToken,
                severity: Severity::Warning,
                message: "Few colors extracted. Design may be missing color styles.".into(),
                location: None,
                suggestion: Some("Define color styles in Figma for consistent token generation.".into()),
                docs_url: None,
            });
        }

        // Check for sufficient typography
        if tokens.typography.is_empty() {
            warnings.push(ValidationIssue {
                issue_type: IssueType::UndefinedToken,
                severity: Severity::Warning,
                message: "No typography styles found. Text may use inconsistent styles.".into(),
                location: None,
                suggestion: Some("Define text styles in Figma for typography tokens.".into()),
                docs_url: None,
            });
        }

        // Check for text size accessibility
        if self.config.check_accessibility {
            for typo in &tokens.typography {
                if typo.font_size < self.config.min_font_size {
                    warnings.push(ValidationIssue {
                        issue_type: IssueType::TextTooSmall,
                        severity: Severity::Warning,
                        message: format!(
                            "Text style '{}' has font size {}px, which may be too small.",
                            typo.role, typo.font_size
                        ),
                        location: typo.original_name.clone(),
                        suggestion: Some(format!(
                            "Consider increasing to at least {}px for readability.",
                            self.config.min_font_size
                        )),
                        docs_url: None,
                    });
                }
            }
        }

        let accessibility_score = self.calculate_accessibility_score(&errors, &warnings);
        let performance_score = 100;

        let status = if !errors.is_empty() {
            ValidationStatus::Invalid
        } else if !warnings.is_empty() {
            ValidationStatus::ValidWithWarnings
        } else {
            ValidationStatus::Valid
        };

        ValidationReport {
            errors,
            warnings,
            info,
            status,
            accessibility_score,
            performance_score,
        }
    }

    /// Validate component mappings
    pub fn validate_components(&self, mappings: &[ComponentMapping]) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        for mapping in mappings {
            // Check confidence
            if mapping.confidence.value() < 0.5 {
                warnings.push(ValidationIssue {
                    issue_type: IssueType::UnknownComponent,
                    severity: Severity::Warning,
                    message: format!(
                        "Low confidence mapping ({:.0}%) for '{}' -> '{}'",
                        mapping.confidence.value() * 100.0,
                        mapping.figma_name,
                        mapping.component.name()
                    ),
                    location: Some(mapping.figma_node_id.clone()),
                    suggestion: Some("Review mapping and consider manual adjustment.".into()),
                    docs_url: None,
                });
            }

            // Add existing warnings
            for w in &mapping.warnings {
                warnings.push(ValidationIssue {
                    issue_type: IssueType::UnknownComponent,
                    severity: Severity::Warning,
                    message: w.clone(),
                    location: Some(mapping.figma_node_id.clone()),
                    suggestion: None,
                    docs_url: None,
                });
            }
        }

        let status = if !errors.is_empty() {
            ValidationStatus::Invalid
        } else if !warnings.is_empty() {
            ValidationStatus::ValidWithWarnings
        } else {
            ValidationStatus::Valid
        };

        ValidationReport {
            errors,
            warnings,
            info,
            status,
            accessibility_score: 100,
            performance_score: 100,
        }
    }

    /// Validate layouts
    pub fn validate_layouts(&self, layouts: &[OxideLayout]) -> ValidationReport {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut info = Vec::new();

        for layout in layouts {
            if self.config.check_performance {
                self.check_layout_performance(layout, 0, &mut warnings);
            }
        }

        let performance_score = self.calculate_performance_score(&warnings);

        let status = if !errors.is_empty() {
            ValidationStatus::Invalid
        } else if !warnings.is_empty() {
            ValidationStatus::ValidWithWarnings
        } else {
            ValidationStatus::Valid
        };

        ValidationReport {
            errors,
            warnings,
            info,
            status,
            accessibility_score: 100,
            performance_score,
        }
    }

    /// Check color contrast
    fn check_color_contrast(
        &self,
        theme: &Theme,
        errors: &mut Vec<ValidationIssue>,
        warnings: &mut Vec<ValidationIssue>,
    ) {
        let colors = &theme.tokens.color;

        // Check text on background
        self.check_contrast_pair(
            "text",
            &colors.text.value,
            "background",
            &colors.background.value,
            errors,
            warnings,
        );

        // Check text on surface
        self.check_contrast_pair(
            "text",
            &colors.text.value,
            "surface",
            &colors.surface.value,
            errors,
            warnings,
        );

        // Check primary contrast
        if let Some(contrast) = &colors.primary.contrast {
            self.check_contrast_pair(
                "primary contrast",
                contrast,
                "primary",
                &colors.primary.value,
                errors,
                warnings,
            );
        }
    }

    /// Check contrast between two colors
    fn check_contrast_pair(
        &self,
        fg_name: &str,
        fg_color: &str,
        bg_name: &str,
        bg_color: &str,
        errors: &mut Vec<ValidationIssue>,
        warnings: &mut Vec<ValidationIssue>,
    ) {
        let fg = match self.parse_color(fg_color) {
            Some(c) => c,
            None => return,
        };

        let bg = match self.parse_color(bg_color) {
            Some(c) => c,
            None => return,
        };

        let ratio = self.calculate_contrast_ratio(&fg, &bg);

        if ratio < 3.0 {
            errors.push(ValidationIssue {
                issue_type: IssueType::InsufficientContrast,
                severity: Severity::Error,
                message: format!(
                    "Contrast ratio {:.2}:1 between {} and {} fails WCAG requirements.",
                    ratio, fg_name, bg_name
                ),
                location: Some(format!("{} on {}", fg_name, bg_name)),
                suggestion: Some("Increase color difference for better readability.".into()),
                docs_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum".into()),
            });
        } else if ratio < self.config.min_contrast_ratio {
            warnings.push(ValidationIssue {
                issue_type: IssueType::InsufficientContrast,
                severity: Severity::Warning,
                message: format!(
                    "Contrast ratio {:.2}:1 between {} and {} is below WCAG AA ({:.1}:1).",
                    ratio, fg_name, bg_name, self.config.min_contrast_ratio
                ),
                location: Some(format!("{} on {}", fg_name, bg_name)),
                suggestion: Some("Consider increasing contrast for better accessibility.".into()),
                docs_url: Some("https://www.w3.org/WAI/WCAG21/Understanding/contrast-minimum".into()),
            });
        }
    }

    /// Parse hex color
    fn parse_color(&self, hex: &str) -> Option<Color> {
        let hex = hex.trim_start_matches('#');
        if hex.len() != 6 {
            return None;
        }

        let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
        let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
        let b = u8::from_str_radix(&hex[4..6], 16).ok()?;

        Some(Color {
            r: r as f32 / 255.0,
            g: g as f32 / 255.0,
            b: b as f32 / 255.0,
            a: 1.0,
        })
    }

    /// Calculate contrast ratio between two colors
    fn calculate_contrast_ratio(&self, fg: &Color, bg: &Color) -> f32 {
        let l1 = self.relative_luminance(fg);
        let l2 = self.relative_luminance(bg);

        let lighter = l1.max(l2);
        let darker = l1.min(l2);

        (lighter + 0.05) / (darker + 0.05)
    }

    /// Calculate relative luminance
    fn relative_luminance(&self, color: &Color) -> f32 {
        let r = self.srgb_to_linear(color.r);
        let g = self.srgb_to_linear(color.g);
        let b = self.srgb_to_linear(color.b);

        0.2126 * r + 0.7152 * g + 0.0722 * b
    }

    /// Convert sRGB to linear
    fn srgb_to_linear(&self, value: f32) -> f32 {
        if value <= 0.03928 {
            value / 12.92
        } else {
            ((value + 0.055) / 1.055).powf(2.4)
        }
    }

    /// Check token completeness
    fn check_token_completeness(
        &self,
        theme: &Theme,
        warnings: &mut Vec<ValidationIssue>,
        info: &mut Vec<ValidationIssue>,
    ) {
        let colors = &theme.tokens.color;

        // Check for default/empty colors
        if colors.primary.value.is_empty() {
            warnings.push(ValidationIssue {
                issue_type: IssueType::UndefinedToken,
                severity: Severity::Warning,
                message: "Primary color is not defined.".into(),
                location: Some("tokens.color.primary".into()),
                suggestion: Some("Define a primary color for the theme.".into()),
                docs_url: None,
            });
        }

        if colors.background.value.is_empty() {
            warnings.push(ValidationIssue {
                issue_type: IssueType::UndefinedToken,
                severity: Severity::Warning,
                message: "Background color is not defined.".into(),
                location: Some("tokens.color.background".into()),
                suggestion: Some("Define a background color for the theme.".into()),
                docs_url: None,
            });
        }

        // Info about missing optional colors
        if colors.success.value.is_empty() {
            info.push(ValidationIssue {
                issue_type: IssueType::UndefinedToken,
                severity: Severity::Info,
                message: "Success color is not defined. Will use default.".into(),
                location: Some("tokens.color.success".into()),
                suggestion: None,
                docs_url: None,
            });
        }
    }

    /// Check layout performance
    fn check_layout_performance(
        &self,
        layout: &OxideLayout,
        depth: usize,
        warnings: &mut Vec<ValidationIssue>,
    ) {
        // Check nesting depth
        if depth > self.config.max_nesting_depth {
            warnings.push(ValidationIssue {
                issue_type: IssueType::DeepNesting,
                severity: Severity::Warning,
                message: format!(
                    "Layout nesting depth ({}) exceeds recommended maximum ({}).",
                    depth, self.config.max_nesting_depth
                ),
                location: layout.figma_node_id.clone(),
                suggestion: Some("Consider flattening the layout structure.".into()),
                docs_url: None,
            });
        }

        // Check child count
        if layout.children.len() > 50 {
            warnings.push(ValidationIssue {
                issue_type: IssueType::LargeChildCount,
                severity: Severity::Warning,
                message: format!(
                    "Layout has {} children, which may impact performance.",
                    layout.children.len()
                ),
                location: layout.figma_node_id.clone(),
                suggestion: Some("Consider using virtualization for large lists.".into()),
                docs_url: None,
            });
        }

        // Recurse into children
        for child in &layout.children {
            self.check_layout_performance(child, depth + 1, warnings);
        }
    }

    /// Calculate accessibility score
    fn calculate_accessibility_score(
        &self,
        errors: &[ValidationIssue],
        warnings: &[ValidationIssue],
    ) -> u32 {
        let error_penalty = errors.iter()
            .filter(|e| matches!(e.issue_type, IssueType::InsufficientContrast | IssueType::TextTooSmall))
            .count() as u32 * 20;

        let warning_penalty = warnings.iter()
            .filter(|w| matches!(w.issue_type, IssueType::InsufficientContrast | IssueType::TextTooSmall))
            .count() as u32 * 5;

        100u32.saturating_sub(error_penalty).saturating_sub(warning_penalty)
    }

    /// Calculate performance score
    fn calculate_performance_score(&self, warnings: &[ValidationIssue]) -> u32 {
        let penalty = warnings.iter()
            .filter(|w| matches!(
                w.issue_type,
                IssueType::DeepNesting | IssueType::LargeChildCount | IssueType::TooManyShadows
            ))
            .count() as u32 * 10;

        100u32.saturating_sub(penalty)
    }
}

impl ValidationReport {
    /// Check if validation passed
    pub fn is_valid(&self) -> bool {
        matches!(self.status, ValidationStatus::Valid | ValidationStatus::ValidWithWarnings)
    }

    /// Check if there are any issues
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }

    /// Get human-readable summary
    pub fn to_summary_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Validation Report"));
        lines.push(format!("================="));
        lines.push(format!("Status: {:?}", self.status));
        lines.push(format!("Accessibility Score: {}/100", self.accessibility_score));
        lines.push(format!("Performance Score: {}/100", self.performance_score));
        lines.push(format!(""));
        lines.push(format!("Issues:"));
        lines.push(format!("  Errors: {}", self.errors.len()));
        lines.push(format!("  Warnings: {}", self.warnings.len()));
        lines.push(format!("  Info: {}", self.info.len()));

        if !self.errors.is_empty() {
            lines.push(format!(""));
            lines.push(format!("Errors:"));
            for error in &self.errors {
                lines.push(format!("  - {}", error.message));
            }
        }

        if !self.warnings.is_empty() {
            lines.push(format!(""));
            lines.push(format!("Warnings:"));
            for warning in &self.warnings {
                lines.push(format!("  - {}", warning.message));
            }
        }

        lines.join("\n")
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contrast_ratio() {
        let validator = Validator::new();

        // Black on white should be high contrast
        let black = Color { r: 0.0, g: 0.0, b: 0.0, a: 1.0 };
        let white = Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 };
        let ratio = validator.calculate_contrast_ratio(&black, &white);
        assert!(ratio > 20.0);

        // Same color should be 1:1
        let gray = Color { r: 0.5, g: 0.5, b: 0.5, a: 1.0 };
        let ratio = validator.calculate_contrast_ratio(&gray, &gray);
        assert!((ratio - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_parse_color() {
        let validator = Validator::new();

        let color = validator.parse_color("#FF0000").unwrap();
        assert!((color.r - 1.0).abs() < 0.01);
        assert!((color.g - 0.0).abs() < 0.01);
        assert!((color.b - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_validation_report() {
        let report = ValidationReport {
            errors: Vec::new(),
            warnings: vec![ValidationIssue {
                issue_type: IssueType::InsufficientContrast,
                severity: Severity::Warning,
                message: "Test warning".into(),
                location: None,
                suggestion: None,
                docs_url: None,
            }],
            info: Vec::new(),
            status: ValidationStatus::ValidWithWarnings,
            accessibility_score: 95,
            performance_score: 100,
        };

        assert!(report.is_valid());
        assert!(report.has_issues());
    }
}
