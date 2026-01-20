//! Migration Report Generation
//!
//! Generates comprehensive migration reports in markdown and JSON formats,
//! including analysis summaries, token mappings, and migration recommendations.

use crate::analyzer::{AnalysisResult, Framework};
use crate::error::{MigrateResult, MigrationIssue};
use crate::mapper::MappingResult;
use crate::tokens::ExtractedTokens;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Complete migration report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationReport {
    /// Report metadata
    pub metadata: ReportMetadata,
    /// Source analysis summary
    pub analysis_summary: AnalysisSummary,
    /// Token extraction summary
    pub token_summary: TokenSummary,
    /// Component mapping summary
    pub mapping_summary: MappingSummary,
    /// Migration recommendations
    pub recommendations: Vec<Recommendation>,
    /// All issues found during migration
    pub issues: Vec<IssueEntry>,
    /// Confidence scores
    pub confidence: ConfidenceReport,
    /// Next steps/action items
    pub next_steps: Vec<ActionItem>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Tool version
    pub tool_version: String,
    /// Source path analyzed
    pub source_path: String,
    /// Source framework detected
    pub source_framework: String,
    /// Source framework version (if detected)
    pub source_version: Option<String>,
    /// Output directory (if conversion was performed)
    pub output_path: Option<String>,
}

/// Analysis summary for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisSummary {
    /// Framework detected
    pub framework: String,
    /// Framework version
    pub version: String,
    /// Framework detection confidence
    pub framework_confidence: f32,
    /// Files analyzed
    pub files_analyzed: FileCounts,
    /// Component inventory
    pub components: ComponentCounts,
    /// CSS variables found
    pub css_variables_count: usize,
    /// Colors detected
    pub colors_detected: usize,
    /// Fonts detected
    pub fonts_detected: usize,
}

/// File count summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileCounts {
    pub html: usize,
    pub css: usize,
    pub js: usize,
    pub total_bytes: u64,
}

/// Component count summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentCounts {
    pub total_types: usize,
    pub total_instances: usize,
    pub mappable: usize,
    pub manual_required: usize,
    pub unmappable: usize,
}

/// Token extraction summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSummary {
    /// Theme name generated
    pub theme_name: String,
    /// Is dark theme
    pub is_dark: bool,
    /// Color tokens extracted
    pub colors: ColorSummary,
    /// Typography tokens
    pub typography: TypographySummary,
    /// Spacing tokens
    pub spacing: SpacingSummary,
    /// Other design tokens
    pub other_tokens: OtherTokensSummary,
    /// Extraction confidence scores
    pub confidence: HashMap<String, f32>,
}

/// Color extraction summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColorSummary {
    /// Semantic colors extracted
    pub semantic_colors: Vec<String>,
    /// Custom colors extracted
    pub custom_colors: usize,
    /// Total unique colors
    pub total_colors: usize,
    /// Color mapping notes
    pub notes: Vec<String>,
}

/// Typography extraction summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TypographySummary {
    /// Font families detected
    pub font_families: Vec<String>,
    /// Font sizes in scale
    pub font_sizes: usize,
    /// Typography roles defined
    pub roles_defined: usize,
    /// Primary font
    pub primary_font: Option<String>,
}

/// Spacing extraction summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpacingSummary {
    /// Base spacing unit
    pub base_unit: f32,
    /// Spacing scale entries
    pub scale_entries: usize,
    /// Radius tokens
    pub radius_tokens: usize,
}

/// Other tokens summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OtherTokensSummary {
    /// Shadow tokens
    pub shadows: usize,
    /// Motion/animation tokens
    pub motion: usize,
    /// Custom tokens
    pub custom: usize,
}

/// Component mapping summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSummary {
    /// Layout pattern detected
    pub layout_pattern: String,
    /// Has sidebar
    pub has_sidebar: bool,
    /// Has navbar
    pub has_navbar: bool,
    /// Component mappings
    pub mappings: Vec<ComponentMappingEntry>,
    /// Design parts generated
    pub design_parts: Vec<DesignPartEntry>,
    /// Overall mapping confidence
    pub overall_confidence: f32,
}

/// Individual component mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMappingEntry {
    /// Source component type
    pub source: String,
    /// Target OxideKit component
    pub target: String,
    /// Occurrence count
    pub count: usize,
    /// Mapping confidence
    pub confidence: f32,
    /// Needs manual review
    pub needs_review: bool,
}

/// Design part entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPartEntry {
    /// Part tag
    pub tag: String,
    /// Part name
    pub name: String,
    /// Components included
    pub components: Vec<String>,
    /// Confidence
    pub confidence: f32,
}

/// Migration recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Priority (1-5, 1 is highest)
    pub priority: u8,
    /// Category
    pub category: RecommendationCategory,
    /// Estimated effort
    pub effort: String,
}

/// Recommendation categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RecommendationCategory {
    Design,
    Components,
    Performance,
    Compatibility,
    BestPractice,
}

impl std::fmt::Display for RecommendationCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecommendationCategory::Design => write!(f, "Design"),
            RecommendationCategory::Components => write!(f, "Components"),
            RecommendationCategory::Performance => write!(f, "Performance"),
            RecommendationCategory::Compatibility => write!(f, "Compatibility"),
            RecommendationCategory::BestPractice => write!(f, "Best Practice"),
        }
    }
}

/// Issue entry for the report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueEntry {
    /// Issue severity
    pub severity: String,
    /// Issue category
    pub category: String,
    /// Issue message
    pub message: String,
    /// Source file
    pub file: Option<String>,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Confidence report
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfidenceReport {
    /// Overall migration confidence
    pub overall: f32,
    /// Framework detection confidence
    pub framework: f32,
    /// Token extraction confidence
    pub tokens: f32,
    /// Component mapping confidence
    pub mapping: f32,
    /// Risk assessment
    pub risk_level: String,
}

/// Action item for next steps
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionItem {
    /// Action title
    pub title: String,
    /// Action description
    pub description: String,
    /// Priority (1-5)
    pub priority: u8,
    /// Estimated time
    pub estimated_time: String,
    /// Is blocking
    pub blocking: bool,
}

/// Report generator
pub struct ReportGenerator {
    /// Include detailed component mappings
    #[allow(dead_code)]
    include_details: bool,
    /// Include code snippets (reserved for future use)
    #[allow(dead_code)]
    include_snippets: bool,
}

impl ReportGenerator {
    /// Create a new report generator
    pub fn new() -> Self {
        Self {
            include_details: true,
            include_snippets: false,
        }
    }

    /// Enable/disable detailed output
    pub fn with_details(mut self, enabled: bool) -> Self {
        self.include_details = enabled;
        self
    }

    /// Generate a complete migration report
    pub fn generate(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
        source_path: impl AsRef<Path>,
        output_path: Option<&Path>,
    ) -> MigrateResult<MigrationReport> {
        let metadata = ReportMetadata {
            generated_at: Utc::now(),
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            source_path: source_path.as_ref().display().to_string(),
            source_framework: analysis.framework.to_string(),
            source_version: analysis.version.raw.clone(),
            output_path: output_path.map(|p| p.display().to_string()),
        };

        let analysis_summary = self.build_analysis_summary(analysis);
        let token_summary = self.build_token_summary(tokens);
        let mapping_summary = self.build_mapping_summary(analysis, mappings);
        let recommendations = self.generate_recommendations(analysis, tokens, mappings);
        let issues = self.collect_issues(analysis, tokens, mappings);
        let confidence = self.calculate_confidence(analysis, tokens, mappings);
        let next_steps = self.generate_next_steps(analysis, mappings, &confidence);

        Ok(MigrationReport {
            metadata,
            analysis_summary,
            token_summary,
            mapping_summary,
            recommendations,
            issues,
            confidence,
            next_steps,
        })
    }

    /// Build analysis summary
    fn build_analysis_summary(&self, analysis: &AnalysisResult) -> AnalysisSummary {
        AnalysisSummary {
            framework: analysis.framework.to_string(),
            version: analysis.version.to_string(),
            framework_confidence: analysis.framework_confidence,
            files_analyzed: FileCounts {
                html: analysis.files_analyzed.html_files,
                css: analysis.files_analyzed.css_files,
                js: analysis.files_analyzed.js_files,
                total_bytes: analysis.files_analyzed.total_bytes,
            },
            components: ComponentCounts {
                total_types: analysis.inventory.total_types,
                total_instances: analysis.inventory.total_instances,
                mappable: analysis.inventory.mappable_count,
                manual_required: analysis.inventory.manual_count,
                unmappable: analysis.inventory.unmappable_count,
            },
            css_variables_count: analysis.css_variables.len(),
            colors_detected: analysis.detected_colors.len(),
            fonts_detected: analysis.detected_fonts.len(),
        }
    }

    /// Build token summary
    fn build_token_summary(&self, tokens: &ExtractedTokens) -> TokenSummary {
        let theme = &tokens.theme;
        let colors = &theme.tokens.color;

        let mut semantic_colors = Vec::new();
        if !colors.primary.value.is_empty() {
            semantic_colors.push("primary".into());
        }
        if !colors.secondary.value.is_empty() {
            semantic_colors.push("secondary".into());
        }
        if !colors.success.value.is_empty() {
            semantic_colors.push("success".into());
        }
        if !colors.warning.value.is_empty() {
            semantic_colors.push("warning".into());
        }
        if !colors.danger.value.is_empty() {
            semantic_colors.push("danger".into());
        }
        if !colors.info.value.is_empty() {
            semantic_colors.push("info".into());
        }
        if !colors.background.value.is_empty() {
            semantic_colors.push("background".into());
        }
        if !colors.text.value.is_empty() {
            semantic_colors.push("text".into());
        }

        TokenSummary {
            theme_name: theme.name.clone(),
            is_dark: theme.metadata.is_dark,
            colors: ColorSummary {
                semantic_colors,
                custom_colors: colors.custom.len(),
                total_colors: colors.custom.len() + 8, // 8 base semantic colors
                notes: tokens
                    .issues
                    .iter()
                    .filter(|i| i.category == crate::error::IssueCategory::ColorToken)
                    .map(|i| i.message.clone())
                    .collect(),
            },
            typography: TypographySummary {
                font_families: tokens.fonts.all_fonts.clone(),
                font_sizes: 7, // xs, sm, md, lg, xl, xxl, xxxl
                roles_defined: tokens.typography.roles.len(),
                primary_font: tokens.fonts.primary.clone(),
            },
            spacing: SpacingSummary {
                base_unit: theme.tokens.spacing.base,
                scale_entries: 6, // xs, sm, md, lg, xl, xxl
                radius_tokens: 6, // none, sm, md, lg, xl, full
            },
            other_tokens: OtherTokensSummary {
                shadows: 5, // none, sm, md, lg, xl
                motion: 4,  // durations
                custom: theme.tokens.custom.len(),
            },
            confidence: {
                let mut map = HashMap::new();
                map.insert("colors".into(), tokens.confidence.colors);
                map.insert("typography".into(), tokens.confidence.typography);
                map.insert("spacing".into(), tokens.confidence.spacing);
                map.insert("shadows".into(), tokens.confidence.shadows);
                map
            },
        }
    }

    /// Build mapping summary
    fn build_mapping_summary(
        &self,
        analysis: &AnalysisResult,
        mappings: &MappingResult,
    ) -> MappingSummary {
        MappingSummary {
            layout_pattern: mappings.layout.pattern.to_string(),
            has_sidebar: mappings.layout.sidebar.is_some(),
            has_navbar: mappings.layout.navbar.is_some(),
            mappings: mappings
                .components
                .iter()
                .map(|m| {
                    let count = analysis
                        .components
                        .iter()
                        .find(|c| c.component_type == m.source_type)
                        .map(|c| c.occurrences)
                        .unwrap_or(0);
                    ComponentMappingEntry {
                        source: format!("{:?}", m.source_type),
                        target: m.target_component.clone(),
                        count,
                        confidence: m.confidence,
                        needs_review: m.needs_review,
                    }
                })
                .collect(),
            design_parts: mappings
                .design_parts
                .iter()
                .map(|p| DesignPartEntry {
                    tag: p.tag.clone(),
                    name: p.name.clone(),
                    components: p.components.clone(),
                    confidence: p.confidence,
                })
                .collect(),
            overall_confidence: mappings.confidence,
        }
    }

    /// Generate recommendations
    fn generate_recommendations(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Framework-specific recommendations
        match analysis.framework {
            Framework::Bootstrap => {
                recommendations.push(Recommendation {
                    title: "Bootstrap Grid Migration".into(),
                    description: "Bootstrap's 12-column grid maps well to OxideKit's Grid component. Consider using Flex for simpler layouts.".into(),
                    priority: 2,
                    category: RecommendationCategory::Components,
                    effort: "Low".into(),
                });
            }
            Framework::Tailwind => {
                recommendations.push(Recommendation {
                    title: "Tailwind Utility Migration".into(),
                    description: "Tailwind utility classes should be converted to OxideKit token-based styling. The theme file contains extracted tokens.".into(),
                    priority: 2,
                    category: RecommendationCategory::Design,
                    effort: "Medium".into(),
                });
            }
            Framework::Mixed => {
                recommendations.push(Recommendation {
                    title: "Consolidate CSS Frameworks".into(),
                    description: "Multiple CSS frameworks detected. Consider standardizing on OxideKit's native styling to avoid conflicts.".into(),
                    priority: 1,
                    category: RecommendationCategory::Design,
                    effort: "High".into(),
                });
            }
            _ => {}
        }

        // Color recommendations
        if tokens.confidence.colors < 0.7 {
            recommendations.push(Recommendation {
                title: "Review Color Tokens".into(),
                description: "Color extraction confidence is low. Review and customize colors in theme/theme.generated.toml.".into(),
                priority: 1,
                category: RecommendationCategory::Design,
                effort: "Medium".into(),
            });
        }

        // Typography recommendations
        if tokens.fonts.primary.is_none() {
            recommendations.push(Recommendation {
                title: "Define Primary Font".into(),
                description: "No primary font was detected. Add font configuration to ensure consistent typography.".into(),
                priority: 2,
                category: RecommendationCategory::Design,
                effort: "Low".into(),
            });
        }

        // Component recommendations
        let unmapped_percent = if analysis.inventory.total_instances > 0 {
            analysis.inventory.unmappable_count as f32 / analysis.inventory.total_instances as f32
        } else {
            0.0
        };

        if unmapped_percent > 0.1 {
            recommendations.push(Recommendation {
                title: "Custom Component Implementation".into(),
                description: format!(
                    "{:.0}% of components have no direct OxideKit equivalent. Consider implementing custom components or using compat.webview.",
                    unmapped_percent * 100.0
                ),
                priority: 1,
                category: RecommendationCategory::Components,
                effort: "High".into(),
            });
        }

        // Layout recommendations
        if mappings.layout.sidebar.is_some() && mappings.layout.navbar.is_some() {
            recommendations.push(Recommendation {
                title: "Dashboard Layout Optimization".into(),
                description: "Full dashboard layout detected. Consider using OxideKit's responsive layout utilities for mobile views.".into(),
                priority: 3,
                category: RecommendationCategory::Performance,
                effort: "Medium".into(),
            });
        }

        // Best practices
        recommendations.push(Recommendation {
            title: "Component Testing".into(),
            description: "Add visual regression tests for migrated components to ensure consistent rendering.".into(),
            priority: 3,
            category: RecommendationCategory::BestPractice,
            effort: "Medium".into(),
        });

        recommendations.push(Recommendation {
            title: "Accessibility Review".into(),
            description: "Review migrated components for accessibility compliance. OxideKit components have built-in ARIA support.".into(),
            priority: 2,
            category: RecommendationCategory::BestPractice,
            effort: "Medium".into(),
        });

        recommendations
    }

    /// Collect all issues
    fn collect_issues(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
    ) -> Vec<IssueEntry> {
        let mut issues = Vec::new();

        // Analysis issues
        for issue in &analysis.issues {
            issues.push(self.convert_issue(issue));
        }

        // Token issues
        for issue in &tokens.issues {
            issues.push(self.convert_issue(issue));
        }

        // Mapping issues
        for issue in &mappings.issues {
            issues.push(self.convert_issue(issue));
        }

        // Sort by severity (errors first)
        issues.sort_by(|a, b| {
            let severity_order = |s: &str| match s {
                "ERROR" => 0,
                "WARN" => 1,
                "INFO" => 2,
                _ => 3,
            };
            severity_order(&a.severity).cmp(&severity_order(&b.severity))
        });

        issues
    }

    fn convert_issue(&self, issue: &MigrationIssue) -> IssueEntry {
        IssueEntry {
            severity: issue.severity.to_string(),
            category: issue.category.to_string(),
            message: issue.message.clone(),
            file: issue.source_file.clone(),
            suggestion: issue.suggestion.clone(),
        }
    }

    /// Calculate confidence report
    fn calculate_confidence(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
    ) -> ConfidenceReport {
        let overall = (analysis.migration_confidence + tokens.confidence.overall + mappings.confidence) / 3.0;

        let risk_level = if overall >= 0.8 {
            "Low - Migration should proceed smoothly"
        } else if overall >= 0.6 {
            "Medium - Some manual adjustments expected"
        } else if overall >= 0.4 {
            "High - Significant manual work required"
        } else {
            "Very High - Consider manual migration"
        };

        ConfidenceReport {
            overall,
            framework: analysis.framework_confidence,
            tokens: tokens.confidence.overall,
            mapping: mappings.confidence,
            risk_level: risk_level.into(),
        }
    }

    /// Generate next steps
    fn generate_next_steps(
        &self,
        analysis: &AnalysisResult,
        _mappings: &MappingResult,
        confidence: &ConfidenceReport,
    ) -> Vec<ActionItem> {
        let mut steps = Vec::new();

        steps.push(ActionItem {
            title: "Review Generated Theme".into(),
            description: "Open theme/theme.generated.toml and verify colors, spacing, and typography match your design.".into(),
            priority: 1,
            estimated_time: "30 min".into(),
            blocking: false,
        });

        if confidence.tokens < 0.7 {
            steps.push(ActionItem {
                title: "Customize Design Tokens".into(),
                description: "Token extraction confidence is low. Manually adjust token values in the theme file.".into(),
                priority: 1,
                estimated_time: "1-2 hours".into(),
                blocking: true,
            });
        }

        steps.push(ActionItem {
            title: "Implement Navigation".into(),
            description: "Set up routing and navigation based on the detected layout pattern.".into(),
            priority: 1,
            estimated_time: "1-2 hours".into(),
            blocking: true,
        });

        if analysis.inventory.unmappable_count > 0 {
            steps.push(ActionItem {
                title: "Handle Unmapped Components".into(),
                description: format!(
                    "Implement or find alternatives for {} components without OxideKit equivalents.",
                    analysis.inventory.unmappable_count
                ),
                priority: 2,
                estimated_time: "2-4 hours".into(),
                blocking: false,
            });
        }

        steps.push(ActionItem {
            title: "Connect Data Sources".into(),
            description: "Replace placeholder data with actual API calls and state management.".into(),
            priority: 2,
            estimated_time: "2-4 hours".into(),
            blocking: true,
        });

        steps.push(ActionItem {
            title: "Test All Pages".into(),
            description: "Verify all migrated pages render correctly and function as expected.".into(),
            priority: 2,
            estimated_time: "1-2 hours".into(),
            blocking: false,
        });

        steps.push(ActionItem {
            title: "Mobile Responsiveness".into(),
            description: "Test and adjust layouts for mobile viewports.".into(),
            priority: 3,
            estimated_time: "1-2 hours".into(),
            blocking: false,
        });

        steps
    }

    /// Export report to JSON
    pub fn to_json(&self, report: &MigrationReport) -> MigrateResult<String> {
        Ok(serde_json::to_string_pretty(report)?)
    }

    /// Export report to Markdown
    pub fn to_markdown(&self, report: &MigrationReport) -> String {
        let mut md = String::new();

        // Header
        md.push_str("# OxideKit Migration Report\n\n");
        md.push_str(&format!(
            "Generated: {}\n",
            report.metadata.generated_at.format("%Y-%m-%d %H:%M:%S UTC")
        ));
        md.push_str(&format!("Tool Version: {}\n\n", report.metadata.tool_version));

        // Overview
        md.push_str("## Overview\n\n");
        md.push_str(&format!(
            "- **Source**: {}\n",
            report.metadata.source_path
        ));
        md.push_str(&format!(
            "- **Framework**: {} {}\n",
            report.metadata.source_framework,
            report.metadata.source_version.as_deref().unwrap_or("")
        ));
        md.push_str(&format!(
            "- **Overall Confidence**: {:.0}%\n",
            report.confidence.overall * 100.0
        ));
        md.push_str(&format!("- **Risk Level**: {}\n\n", report.confidence.risk_level));

        // Analysis Summary
        md.push_str("## Source Analysis\n\n");
        md.push_str("### Files Analyzed\n\n");
        md.push_str(&format!(
            "| Type | Count |\n|------|-------|\n| HTML | {} |\n| CSS | {} |\n| JS/TS | {} |\n\n",
            report.analysis_summary.files_analyzed.html,
            report.analysis_summary.files_analyzed.css,
            report.analysis_summary.files_analyzed.js
        ));

        md.push_str("### Components Detected\n\n");
        md.push_str(&format!(
            "- Total types: {}\n",
            report.analysis_summary.components.total_types
        ));
        md.push_str(&format!(
            "- Total instances: {}\n",
            report.analysis_summary.components.total_instances
        ));
        md.push_str(&format!(
            "- Auto-mappable: {}\n",
            report.analysis_summary.components.mappable
        ));
        md.push_str(&format!(
            "- Manual review needed: {}\n",
            report.analysis_summary.components.manual_required
        ));
        md.push_str(&format!(
            "- No equivalent: {}\n\n",
            report.analysis_summary.components.unmappable
        ));

        // Token Summary
        md.push_str("## Design Tokens\n\n");
        md.push_str(&format!(
            "- Theme: {} ({})\n",
            report.token_summary.theme_name,
            if report.token_summary.is_dark { "Dark" } else { "Light" }
        ));
        md.push_str(&format!(
            "- Colors: {} semantic, {} custom\n",
            report.token_summary.colors.semantic_colors.len(),
            report.token_summary.colors.custom_colors
        ));
        md.push_str(&format!(
            "- Typography: {} fonts, {} roles\n",
            report.token_summary.typography.font_families.len(),
            report.token_summary.typography.roles_defined
        ));
        md.push_str(&format!(
            "- Spacing base: {}px\n\n",
            report.token_summary.spacing.base_unit
        ));

        // Mapping Summary
        md.push_str("## Component Mappings\n\n");
        md.push_str(&format!(
            "Layout Pattern: **{}**\n\n",
            report.mapping_summary.layout_pattern
        ));

        if !report.mapping_summary.mappings.is_empty() {
            md.push_str("| Source | Target | Count | Confidence | Review |\n");
            md.push_str("|--------|--------|-------|------------|--------|\n");
            for mapping in &report.mapping_summary.mappings {
                md.push_str(&format!(
                    "| {} | {} | {} | {:.0}% | {} |\n",
                    mapping.source,
                    mapping.target,
                    mapping.count,
                    mapping.confidence * 100.0,
                    if mapping.needs_review { "Yes" } else { "No" }
                ));
            }
            md.push_str("\n");
        }

        // Recommendations
        md.push_str("## Recommendations\n\n");
        for rec in &report.recommendations {
            md.push_str(&format!(
                "### {} (Priority {})\n\n",
                rec.title, rec.priority
            ));
            md.push_str(&format!("{}\n\n", rec.description));
            md.push_str(&format!(
                "- Category: {}\n- Effort: {}\n\n",
                rec.category, rec.effort
            ));
        }

        // Issues
        if !report.issues.is_empty() {
            md.push_str("## Issues\n\n");
            let errors: Vec<_> = report.issues.iter().filter(|i| i.severity == "ERROR").collect();
            let warnings: Vec<_> = report.issues.iter().filter(|i| i.severity == "WARN").collect();

            if !errors.is_empty() {
                md.push_str("### Errors\n\n");
                for issue in errors {
                    md.push_str(&format!("- **[{}]** {}\n", issue.category, issue.message));
                    if let Some(ref suggestion) = issue.suggestion {
                        md.push_str(&format!("  - Suggestion: {}\n", suggestion));
                    }
                }
                md.push_str("\n");
            }

            if !warnings.is_empty() {
                md.push_str("### Warnings\n\n");
                for issue in warnings {
                    md.push_str(&format!("- **[{}]** {}\n", issue.category, issue.message));
                }
                md.push_str("\n");
            }
        }

        // Next Steps
        md.push_str("## Next Steps\n\n");
        for (i, step) in report.next_steps.iter().enumerate() {
            md.push_str(&format!(
                "{}. **{}** ({}){}\n   {}\n\n",
                i + 1,
                step.title,
                step.estimated_time,
                if step.blocking { " - BLOCKING" } else { "" },
                step.description
            ));
        }

        // Confidence Details
        md.push_str("## Confidence Details\n\n");
        md.push_str(&format!(
            "| Category | Confidence |\n|----------|------------|\n"
        ));
        md.push_str(&format!(
            "| Framework Detection | {:.0}% |\n",
            report.confidence.framework * 100.0
        ));
        md.push_str(&format!(
            "| Token Extraction | {:.0}% |\n",
            report.confidence.tokens * 100.0
        ));
        md.push_str(&format!(
            "| Component Mapping | {:.0}% |\n",
            report.confidence.mapping * 100.0
        ));
        md.push_str(&format!(
            "| **Overall** | **{:.0}%** |\n\n",
            report.confidence.overall * 100.0
        ));

        md.push_str("---\n\n");
        md.push_str("*Generated by OxideKit Migrate Tool*\n");

        md
    }
}

impl Default for ReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::AnalysisResult;
    use crate::mapper::MappingResult;
    use crate::tokens::ExtractedTokens;

    fn create_test_analysis() -> AnalysisResult {
        let mut analysis = AnalysisResult::default();
        analysis.framework = Framework::Bootstrap;
        analysis.framework_confidence = 0.9;
        analysis.migration_confidence = 0.85;
        analysis.inventory.total_types = 10;
        analysis.inventory.total_instances = 50;
        analysis.inventory.mappable_count = 40;
        analysis
    }

    fn create_test_tokens() -> ExtractedTokens {
        use oxide_components::theme::Theme;

        ExtractedTokens {
            theme: Theme::dark(),
            typography: Default::default(),
            fonts: Default::default(),
            name_mapping: HashMap::new(),
            issues: Vec::new(),
            confidence: crate::tokens::TokenConfidence {
                colors: 0.8,
                typography: 0.7,
                spacing: 0.9,
                radius: 0.85,
                shadows: 0.75,
                overall: 0.8,
            },
        }
    }

    fn create_test_mappings() -> MappingResult {
        MappingResult {
            layout: Default::default(),
            components: Vec::new(),
            design_parts: Vec::new(),
            issues: Vec::new(),
            confidence: 0.85,
        }
    }

    #[test]
    fn test_report_generation() {
        let generator = ReportGenerator::new();
        let analysis = create_test_analysis();
        let tokens = create_test_tokens();
        let mappings = create_test_mappings();

        let report = generator
            .generate(&analysis, &tokens, &mappings, "/test/path", None)
            .unwrap();

        assert_eq!(report.metadata.source_framework, "Bootstrap");
        assert!(report.confidence.overall > 0.5);
        assert!(!report.recommendations.is_empty());
        assert!(!report.next_steps.is_empty());
    }

    #[test]
    fn test_markdown_export() {
        let generator = ReportGenerator::new();
        let analysis = create_test_analysis();
        let tokens = create_test_tokens();
        let mappings = create_test_mappings();

        let report = generator
            .generate(&analysis, &tokens, &mappings, "/test/path", None)
            .unwrap();

        let md = generator.to_markdown(&report);

        assert!(md.contains("# OxideKit Migration Report"));
        assert!(md.contains("Bootstrap"));
        assert!(md.contains("## Recommendations"));
        assert!(md.contains("## Next Steps"));
    }

    #[test]
    fn test_json_export() {
        let generator = ReportGenerator::new();
        let analysis = create_test_analysis();
        let tokens = create_test_tokens();
        let mappings = create_test_mappings();

        let report = generator
            .generate(&analysis, &tokens, &mappings, "/test/path", None)
            .unwrap();

        let json = generator.to_json(&report).unwrap();

        assert!(json.contains("\"source_framework\""));
        assert!(json.contains("Bootstrap"));
    }

    #[test]
    fn test_confidence_calculation() {
        let generator = ReportGenerator::new();
        let analysis = create_test_analysis();
        let tokens = create_test_tokens();
        let mappings = create_test_mappings();

        let confidence = generator.calculate_confidence(&analysis, &tokens, &mappings);

        assert!(confidence.overall > 0.0);
        assert!(confidence.overall <= 1.0);
        assert!(!confidence.risk_level.is_empty());
    }

    #[test]
    fn test_recommendation_generation() {
        let generator = ReportGenerator::new();
        let analysis = create_test_analysis();
        let tokens = create_test_tokens();
        let mappings = create_test_mappings();

        let recommendations = generator.generate_recommendations(&analysis, &tokens, &mappings);

        assert!(!recommendations.is_empty());
        // Should have at least accessibility and testing recommendations
        assert!(recommendations.iter().any(|r| r.category == RecommendationCategory::BestPractice));
    }
}
