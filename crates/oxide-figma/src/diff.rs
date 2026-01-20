//! Design Diff and Comparison
//!
//! Compares Figma designs with existing OxideKit themes and components
//! to detect changes and generate patch plans.

use crate::error::Result;
use crate::tokens::ExtractedTokens;
use crate::types::Color;
use oxide_components::Theme;
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::HashMap;
use tracing::{debug, info};

/// Design diff engine
#[derive(Debug)]
pub struct DesignDiff {
    config: DiffConfig,
}

/// Configuration for diff
#[derive(Debug, Clone)]
pub struct DiffConfig {
    /// Color similarity threshold (0.0 - 1.0)
    pub color_threshold: f32,

    /// Spacing tolerance (pixels)
    pub spacing_tolerance: f32,

    /// Whether to include minor changes
    pub include_minor: bool,
}

impl Default for DiffConfig {
    fn default() -> Self {
        Self {
            color_threshold: 0.02,
            spacing_tolerance: 2.0,
            include_minor: false,
        }
    }
}

/// Result of design diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffResult {
    /// Color changes
    pub color_changes: Vec<ColorChange>,

    /// Spacing changes
    pub spacing_changes: Vec<SpacingChange>,

    /// Typography changes
    pub typography_changes: Vec<TypographyChange>,

    /// Radius changes
    pub radius_changes: Vec<RadiusChange>,

    /// Shadow changes
    pub shadow_changes: Vec<ShadowChange>,

    /// Component changes
    pub component_changes: Vec<ComponentChange>,

    /// Summary statistics
    pub summary: DiffSummary,
}

/// Color change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorChange {
    pub name: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Spacing change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpacingChange {
    pub name: String,
    pub old_value: Option<f32>,
    pub new_value: f32,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Typography change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyChange {
    pub role: String,
    pub property: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Radius change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadiusChange {
    pub name: String,
    pub old_value: Option<f32>,
    pub new_value: f32,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Shadow change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShadowChange {
    pub name: String,
    pub property: String,
    pub old_value: Option<String>,
    pub new_value: String,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Component change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentChange {
    pub component: String,
    pub property: String,
    pub old_value: Option<String>,
    pub new_value: Option<String>,
    pub change_type: ChangeType,
    pub impact: ChangeImpact,
}

/// Type of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeType {
    Added,
    Modified,
    Removed,
}

/// Impact level of change
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeImpact {
    /// Minor change, unlikely to affect anything
    Minor,
    /// Moderate change, may affect some components
    Moderate,
    /// Major change, likely affects many components
    Major,
    /// Breaking change, will definitely affect components
    Breaking,
}

/// Summary of diff
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DiffSummary {
    pub total_changes: usize,
    pub additions: usize,
    pub modifications: usize,
    pub removals: usize,
    pub breaking_changes: usize,
    pub major_changes: usize,
    pub moderate_changes: usize,
    pub minor_changes: usize,
}

impl DesignDiff {
    /// Create a new diff engine
    pub fn new() -> Self {
        Self::with_config(DiffConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: DiffConfig) -> Self {
        Self { config }
    }

    /// Compare extracted tokens with existing theme
    pub fn compare_tokens(&self, tokens: &ExtractedTokens, theme: &Theme) -> DiffResult {
        info!("Comparing extracted tokens with theme");

        let mut result = DiffResult {
            color_changes: Vec::new(),
            spacing_changes: Vec::new(),
            typography_changes: Vec::new(),
            radius_changes: Vec::new(),
            shadow_changes: Vec::new(),
            component_changes: Vec::new(),
            summary: DiffSummary::default(),
        };

        // Compare colors
        self.compare_colors(tokens, theme, &mut result);

        // Compare spacing
        self.compare_spacing(tokens, theme, &mut result);

        // Compare radii
        self.compare_radii(tokens, theme, &mut result);

        // Compare shadows
        self.compare_shadows(tokens, theme, &mut result);

        // Compare typography
        self.compare_typography(tokens, theme, &mut result);

        // Calculate summary
        self.calculate_summary(&mut result);

        info!(
            total = result.summary.total_changes,
            breaking = result.summary.breaking_changes,
            "Diff complete"
        );

        result
    }

    /// Compare two themes
    pub fn compare_themes(&self, old_theme: &Theme, new_theme: &Theme) -> DiffResult {
        info!("Comparing themes");

        let mut result = DiffResult {
            color_changes: Vec::new(),
            spacing_changes: Vec::new(),
            typography_changes: Vec::new(),
            radius_changes: Vec::new(),
            shadow_changes: Vec::new(),
            component_changes: Vec::new(),
            summary: DiffSummary::default(),
        };

        // Compare colors
        self.compare_theme_colors(old_theme, new_theme, &mut result);

        // Compare spacing
        self.compare_theme_spacing(old_theme, new_theme, &mut result);

        // Compare radii
        self.compare_theme_radii(old_theme, new_theme, &mut result);

        // Calculate summary
        self.calculate_summary(&mut result);

        result
    }

    /// Compare TOML content
    pub fn compare_toml(&self, old_content: &str, new_content: &str) -> String {
        let diff = TextDiff::from_lines(old_content, new_content);
        let mut output = String::new();

        for change in diff.iter_all_changes() {
            let sign = match change.tag() {
                ChangeTag::Delete => "-",
                ChangeTag::Insert => "+",
                ChangeTag::Equal => " ",
            };
            output.push_str(&format!("{}{}", sign, change));
        }

        output
    }

    /// Compare colors
    fn compare_colors(&self, tokens: &ExtractedTokens, theme: &Theme, result: &mut DiffResult) {
        let theme_colors = self.extract_theme_colors(theme);

        for extracted in &tokens.colors {
            let new_hex = extracted.color.to_hex();

            if let Some(old_hex) = theme_colors.get(&extracted.name) {
                // Check if changed
                if !self.colors_similar_hex(old_hex, &new_hex) {
                    result.color_changes.push(ColorChange {
                        name: extracted.name.clone(),
                        old_value: Some(old_hex.clone()),
                        new_value: new_hex,
                        change_type: ChangeType::Modified,
                        impact: self.assess_color_impact(&extracted.name),
                    });
                }
            } else {
                // New color
                result.color_changes.push(ColorChange {
                    name: extracted.name.clone(),
                    old_value: None,
                    new_value: new_hex,
                    change_type: ChangeType::Added,
                    impact: ChangeImpact::Minor,
                });
            }
        }
    }

    /// Compare spacing
    fn compare_spacing(&self, tokens: &ExtractedTokens, theme: &Theme, result: &mut DiffResult) {
        let theme_spacing = [
            ("xs", theme.tokens.spacing.xs.value),
            ("sm", theme.tokens.spacing.sm.value),
            ("md", theme.tokens.spacing.md.value),
            ("lg", theme.tokens.spacing.lg.value),
            ("xl", theme.tokens.spacing.xl.value),
            ("xxl", theme.tokens.spacing.xxl.value),
        ];

        for (name, old_value) in theme_spacing {
            // Find corresponding new value
            let new_value = tokens.spacing.iter()
                .find(|v| self.spacing_matches_name(**v, name))
                .copied();

            if let Some(new) = new_value {
                if (old_value - new).abs() > self.config.spacing_tolerance {
                    result.spacing_changes.push(SpacingChange {
                        name: name.to_string(),
                        old_value: Some(old_value),
                        new_value: new,
                        change_type: ChangeType::Modified,
                        impact: ChangeImpact::Moderate,
                    });
                }
            }
        }
    }

    /// Compare radii
    fn compare_radii(&self, tokens: &ExtractedTokens, theme: &Theme, result: &mut DiffResult) {
        let theme_radii = [
            ("sm", theme.tokens.radius.sm),
            ("md", theme.tokens.radius.md),
            ("lg", theme.tokens.radius.lg),
            ("xl", theme.tokens.radius.xl),
        ];

        for (name, old_value) in theme_radii {
            if let Some(&new_value) = tokens.radii.iter()
                .find(|v| self.radius_matches_name(**v, name))
            {
                if (old_value - new_value).abs() > 0.5 {
                    result.radius_changes.push(RadiusChange {
                        name: name.to_string(),
                        old_value: Some(old_value),
                        new_value,
                        change_type: ChangeType::Modified,
                        impact: ChangeImpact::Moderate,
                    });
                }
            }
        }
    }

    /// Compare shadows
    fn compare_shadows(&self, tokens: &ExtractedTokens, theme: &Theme, result: &mut DiffResult) {
        for extracted in &tokens.shadows {
            // Find matching theme shadow
            let theme_shadow = match extracted.name.as_str() {
                "sm" => Some(&theme.tokens.shadow.sm),
                "md" => Some(&theme.tokens.shadow.md),
                "lg" => Some(&theme.tokens.shadow.lg),
                "xl" => Some(&theme.tokens.shadow.xl),
                _ => None,
            };

            if let Some(old) = theme_shadow {
                // Compare blur
                if (old.blur - extracted.blur).abs() > 1.0 {
                    result.shadow_changes.push(ShadowChange {
                        name: extracted.name.clone(),
                        property: "blur".to_string(),
                        old_value: Some(old.blur.to_string()),
                        new_value: extracted.blur.to_string(),
                        change_type: ChangeType::Modified,
                        impact: ChangeImpact::Minor,
                    });
                }

                // Compare offset
                if (old.y - extracted.y).abs() > 1.0 {
                    result.shadow_changes.push(ShadowChange {
                        name: extracted.name.clone(),
                        property: "y_offset".to_string(),
                        old_value: Some(old.y.to_string()),
                        new_value: extracted.y.to_string(),
                        change_type: ChangeType::Modified,
                        impact: ChangeImpact::Minor,
                    });
                }
            }
        }
    }

    /// Compare typography
    fn compare_typography(&self, tokens: &ExtractedTokens, theme: &Theme, result: &mut DiffResult) {
        for extracted in &tokens.typography {
            // Compare font families
            if extracted.role == "body" {
                let old_family = &theme.tokens.typography.font_family.sans;
                if !extracted.font_family.contains(old_family) && !old_family.contains(&extracted.font_family) {
                    result.typography_changes.push(TypographyChange {
                        role: extracted.role.clone(),
                        property: "font_family".to_string(),
                        old_value: Some(old_family.clone()),
                        new_value: extracted.font_family.clone(),
                        change_type: ChangeType::Modified,
                        impact: ChangeImpact::Major,
                    });
                }
            }
        }
    }

    /// Compare theme colors
    fn compare_theme_colors(&self, old_theme: &Theme, new_theme: &Theme, result: &mut DiffResult) {
        let old_colors = self.extract_theme_colors(old_theme);
        let new_colors = self.extract_theme_colors(new_theme);

        for (name, new_value) in &new_colors {
            if let Some(old_value) = old_colors.get(name) {
                if !self.colors_similar_hex(old_value, new_value) {
                    result.color_changes.push(ColorChange {
                        name: name.clone(),
                        old_value: Some(old_value.clone()),
                        new_value: new_value.clone(),
                        change_type: ChangeType::Modified,
                        impact: self.assess_color_impact(name),
                    });
                }
            } else {
                result.color_changes.push(ColorChange {
                    name: name.clone(),
                    old_value: None,
                    new_value: new_value.clone(),
                    change_type: ChangeType::Added,
                    impact: ChangeImpact::Minor,
                });
            }
        }

        // Check for removed colors
        for (name, old_value) in &old_colors {
            if !new_colors.contains_key(name) {
                result.color_changes.push(ColorChange {
                    name: name.clone(),
                    old_value: Some(old_value.clone()),
                    new_value: String::new(),
                    change_type: ChangeType::Removed,
                    impact: ChangeImpact::Breaking,
                });
            }
        }
    }

    /// Compare theme spacing
    fn compare_theme_spacing(&self, old_theme: &Theme, new_theme: &Theme, result: &mut DiffResult) {
        let old = &old_theme.tokens.spacing;
        let new = &new_theme.tokens.spacing;

        let comparisons = [
            ("xs", old.xs.value, new.xs.value),
            ("sm", old.sm.value, new.sm.value),
            ("md", old.md.value, new.md.value),
            ("lg", old.lg.value, new.lg.value),
            ("xl", old.xl.value, new.xl.value),
            ("xxl", old.xxl.value, new.xxl.value),
        ];

        for (name, old_val, new_val) in comparisons {
            if (old_val - new_val).abs() > self.config.spacing_tolerance {
                result.spacing_changes.push(SpacingChange {
                    name: name.to_string(),
                    old_value: Some(old_val),
                    new_value: new_val,
                    change_type: ChangeType::Modified,
                    impact: ChangeImpact::Moderate,
                });
            }
        }
    }

    /// Compare theme radii
    fn compare_theme_radii(&self, old_theme: &Theme, new_theme: &Theme, result: &mut DiffResult) {
        let old = &old_theme.tokens.radius;
        let new = &new_theme.tokens.radius;

        let comparisons = [
            ("sm", old.sm, new.sm),
            ("md", old.md, new.md),
            ("lg", old.lg, new.lg),
            ("xl", old.xl, new.xl),
        ];

        for (name, old_val, new_val) in comparisons {
            if (old_val - new_val).abs() > 0.5 {
                result.radius_changes.push(RadiusChange {
                    name: name.to_string(),
                    old_value: Some(old_val),
                    new_value: new_val,
                    change_type: ChangeType::Modified,
                    impact: ChangeImpact::Moderate,
                });
            }
        }
    }

    /// Extract color map from theme
    fn extract_theme_colors(&self, theme: &Theme) -> HashMap<String, String> {
        let mut colors = HashMap::new();
        let c = &theme.tokens.color;

        colors.insert("primary".into(), c.primary.value.clone());
        colors.insert("secondary".into(), c.secondary.value.clone());
        colors.insert("success".into(), c.success.value.clone());
        colors.insert("warning".into(), c.warning.value.clone());
        colors.insert("danger".into(), c.danger.value.clone());
        colors.insert("info".into(), c.info.value.clone());
        colors.insert("background".into(), c.background.value.clone());
        colors.insert("surface".into(), c.surface.value.clone());
        colors.insert("text".into(), c.text.value.clone());
        colors.insert("border".into(), c.border.value.clone());

        colors
    }

    /// Check if two hex colors are similar
    fn colors_similar_hex(&self, a: &str, b: &str) -> bool {
        let parse_hex = |s: &str| -> Option<(u8, u8, u8)> {
            let s = s.trim_start_matches('#');
            if s.len() != 6 {
                return None;
            }
            let r = u8::from_str_radix(&s[0..2], 16).ok()?;
            let g = u8::from_str_radix(&s[2..4], 16).ok()?;
            let b = u8::from_str_radix(&s[4..6], 16).ok()?;
            Some((r, g, b))
        };

        match (parse_hex(a), parse_hex(b)) {
            (Some((ar, ag, ab)), Some((br, bg, bb))) => {
                let threshold = (self.config.color_threshold * 255.0) as i16;
                (ar as i16 - br as i16).abs() <= threshold
                    && (ag as i16 - bg as i16).abs() <= threshold
                    && (ab as i16 - bb as i16).abs() <= threshold
            }
            _ => a == b,
        }
    }

    /// Assess impact of color change
    fn assess_color_impact(&self, name: &str) -> ChangeImpact {
        match name {
            "primary" | "background" | "text" => ChangeImpact::Major,
            "secondary" | "surface" | "border" => ChangeImpact::Moderate,
            _ => ChangeImpact::Minor,
        }
    }

    /// Check if spacing value matches name
    fn spacing_matches_name(&self, value: f32, name: &str) -> bool {
        let expected = match name {
            "xs" => 4.0,
            "sm" => 8.0,
            "md" => 16.0,
            "lg" => 24.0,
            "xl" => 32.0,
            "xxl" => 48.0,
            _ => return false,
        };
        (value - expected).abs() < self.config.spacing_tolerance * 2.0
    }

    /// Check if radius value matches name
    fn radius_matches_name(&self, value: f32, name: &str) -> bool {
        let expected = match name {
            "sm" => 4.0,
            "md" => 8.0,
            "lg" => 12.0,
            "xl" => 16.0,
            _ => return false,
        };
        (value - expected).abs() < 2.0
    }

    /// Calculate summary statistics
    fn calculate_summary(&self, result: &mut DiffResult) {
        let all_changes: Vec<(ChangeType, ChangeImpact)> = result
            .color_changes
            .iter()
            .map(|c| (c.change_type, c.impact))
            .chain(result.spacing_changes.iter().map(|c| (c.change_type, c.impact)))
            .chain(result.typography_changes.iter().map(|c| (c.change_type, c.impact)))
            .chain(result.radius_changes.iter().map(|c| (c.change_type, c.impact)))
            .chain(result.shadow_changes.iter().map(|c| (c.change_type, c.impact)))
            .chain(result.component_changes.iter().map(|c| (c.change_type, c.impact)))
            .collect();

        result.summary.total_changes = all_changes.len();
        result.summary.additions = all_changes.iter().filter(|(t, _)| *t == ChangeType::Added).count();
        result.summary.modifications = all_changes.iter().filter(|(t, _)| *t == ChangeType::Modified).count();
        result.summary.removals = all_changes.iter().filter(|(t, _)| *t == ChangeType::Removed).count();
        result.summary.breaking_changes = all_changes.iter().filter(|(_, i)| *i == ChangeImpact::Breaking).count();
        result.summary.major_changes = all_changes.iter().filter(|(_, i)| *i == ChangeImpact::Major).count();
        result.summary.moderate_changes = all_changes.iter().filter(|(_, i)| *i == ChangeImpact::Moderate).count();
        result.summary.minor_changes = all_changes.iter().filter(|(_, i)| *i == ChangeImpact::Minor).count();
    }
}

impl DiffResult {
    /// Check if there are any changes
    pub fn has_changes(&self) -> bool {
        self.summary.total_changes > 0
    }

    /// Check if there are breaking changes
    pub fn has_breaking_changes(&self) -> bool {
        self.summary.breaking_changes > 0
    }

    /// Get human-readable summary
    pub fn to_summary_string(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Design Diff Summary"));
        lines.push(format!("=================="));
        lines.push(format!("Total changes: {}", self.summary.total_changes));
        lines.push(format!("  Additions: {}", self.summary.additions));
        lines.push(format!("  Modifications: {}", self.summary.modifications));
        lines.push(format!("  Removals: {}", self.summary.removals));
        lines.push(format!(""));
        lines.push(format!("Impact breakdown:"));
        lines.push(format!("  Breaking: {}", self.summary.breaking_changes));
        lines.push(format!("  Major: {}", self.summary.major_changes));
        lines.push(format!("  Moderate: {}", self.summary.moderate_changes));
        lines.push(format!("  Minor: {}", self.summary.minor_changes));

        if !self.color_changes.is_empty() {
            lines.push(format!(""));
            lines.push(format!("Color changes:"));
            for change in &self.color_changes {
                let old = change.old_value.as_deref().unwrap_or("(new)");
                lines.push(format!(
                    "  {:?} {}: {} -> {}",
                    change.change_type, change.name, old, change.new_value
                ));
            }
        }

        if !self.spacing_changes.is_empty() {
            lines.push(format!(""));
            lines.push(format!("Spacing changes:"));
            for change in &self.spacing_changes {
                let old = change.old_value.map(|v| v.to_string()).unwrap_or_else(|| "(new)".into());
                lines.push(format!(
                    "  {:?} {}: {} -> {}",
                    change.change_type, change.name, old, change.new_value
                ));
            }
        }

        lines.join("\n")
    }
}

impl Default for DesignDiff {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_colors_similar_hex() {
        let diff = DesignDiff::new();
        assert!(diff.colors_similar_hex("#FF0000", "#FF0000"));
        assert!(diff.colors_similar_hex("#FF0000", "#FE0000")); // Close enough
        assert!(!diff.colors_similar_hex("#FF0000", "#00FF00")); // Different
    }

    #[test]
    fn test_diff_summary() {
        let result = DiffResult {
            color_changes: vec![ColorChange {
                name: "primary".into(),
                old_value: Some("#FF0000".into()),
                new_value: "#0000FF".into(),
                change_type: ChangeType::Modified,
                impact: ChangeImpact::Major,
            }],
            spacing_changes: Vec::new(),
            typography_changes: Vec::new(),
            radius_changes: Vec::new(),
            shadow_changes: Vec::new(),
            component_changes: Vec::new(),
            summary: DiffSummary {
                total_changes: 1,
                additions: 0,
                modifications: 1,
                removals: 0,
                breaking_changes: 0,
                major_changes: 1,
                moderate_changes: 0,
                minor_changes: 0,
            },
        };

        assert!(result.has_changes());
        assert!(!result.has_breaking_changes());
    }
}
