//! Figma to OxideKit Translator
//!
//! Main translation engine that orchestrates:
//! - Token extraction
//! - Component mapping
//! - Layout translation
//! - Theme generation
//! - Validation

use crate::components::{ComponentMapper, ComponentMapping, MapperConfig};
use crate::error::{FigmaError, Result};
use crate::layout::{LayoutTranslator, OxideLayout};
use crate::tokens::{ExtractedTokens, ExtractedTypography, TokenExtractor};
use crate::types::FigmaFile;
use crate::validation::{ValidationReport, Validator};
use camino::{Utf8Path, Utf8PathBuf};
use oxide_components::Theme;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use tracing::{debug, info, warn};

/// Figma to OxideKit translator
#[derive(Debug)]
pub struct Translator {
    config: TranslatorConfig,
    token_extractor: TokenExtractor,
    component_mapper: ComponentMapper,
    layout_translator: LayoutTranslator,
    validator: Validator,
}

/// Configuration for translation
#[derive(Debug, Clone)]
pub struct TranslatorConfig {
    /// Theme name
    pub theme_name: String,

    /// Whether this is a dark theme
    pub is_dark: bool,

    /// Whether to validate output
    pub validate: bool,

    /// Minimum component mapping confidence
    pub min_confidence: f32,

    /// Whether to generate layout files
    pub generate_layouts: bool,

    /// Whether to generate component mappings
    pub generate_components: bool,

    /// Whether to use semantic token naming
    pub semantic_naming: bool,
}

impl Default for TranslatorConfig {
    fn default() -> Self {
        Self {
            theme_name: "Figma Import".to_string(),
            is_dark: true,
            validate: true,
            min_confidence: 0.6,
            generate_layouts: true,
            generate_components: true,
            semantic_naming: true,
        }
    }
}

/// Result of translation
#[derive(Debug, Clone, Serialize)]
pub struct TranslationResult {
    /// Generated theme
    #[serde(skip_serializing)]
    pub theme: Theme,

    /// Extracted tokens (raw)
    #[serde(skip_serializing)]
    pub tokens: ExtractedTokens,

    /// Typography roles
    pub typography_roles: Vec<TypographyRoleMapping>,

    /// Component mappings
    pub component_mappings: Vec<ComponentMappingOutput>,

    /// Layout translations
    pub layouts: Vec<OxideLayout>,

    /// Validation report
    #[serde(skip)]
    pub validation: Option<ValidationReport>,

    /// Warnings and suggestions
    pub warnings: Vec<String>,

    /// Metadata
    pub metadata: TranslationMetadata,
}

impl TranslationResult {
    /// Export the theme to a TOML file
    pub fn export_theme(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let toml_content = toml::to_string_pretty(&self.theme)
            .map_err(|e| crate::error::FigmaError::ExportError(format!("Failed to serialize theme: {}", e)))?;
        std::fs::write(path, toml_content)?;
        Ok(())
    }

    /// Export the translation result as JSON
    pub fn export_json(&self, path: impl AsRef<std::path::Path>) -> Result<()> {
        let json_content = serde_json::to_string_pretty(self)
            .map_err(|e| crate::error::FigmaError::ExportError(format!("Failed to serialize result: {}", e)))?;
        std::fs::write(path, json_content)?;
        Ok(())
    }
}

/// Metadata about the translation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationMetadata {
    /// Source Figma file name
    pub source_file: String,

    /// Translation timestamp
    pub translated_at: String,

    /// Figma file version
    pub figma_version: String,

    /// Statistics
    pub stats: TranslationStats,
}

/// Translation statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TranslationStats {
    pub colors_extracted: usize,
    pub spacing_values: usize,
    pub typography_styles: usize,
    pub shadows_extracted: usize,
    pub components_mapped: usize,
    pub layouts_translated: usize,
    pub warnings_count: usize,
}

/// Typography role mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyRoleMapping {
    pub role: String,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub line_height: f32,
    pub letter_spacing: f32,
}

/// Component mapping output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMappingOutput {
    pub figma_name: String,
    pub figma_node_id: String,
    pub oxide_component: String,
    pub confidence: f32,
    pub props: HashMap<String, serde_json::Value>,
    pub warnings: Vec<String>,
}

impl Translator {
    /// Create a new translator with default config
    pub fn new() -> Self {
        Self::with_config(TranslatorConfig::default())
    }

    /// Create with custom config
    pub fn with_config(config: TranslatorConfig) -> Self {
        let mapper_config = MapperConfig {
            min_confidence: config.min_confidence,
            suggest_new_components: true,
            ai_assisted: false,
            strict: false,
        };

        Self {
            config,
            token_extractor: TokenExtractor::new(),
            component_mapper: ComponentMapper::with_config(mapper_config),
            layout_translator: LayoutTranslator::new(),
            validator: Validator::new(),
        }
    }

    /// Translate a Figma file to OxideKit
    pub fn translate(&self, file: &FigmaFile) -> Result<TranslationResult> {
        info!(file_name = %file.name, "Translating Figma file to OxideKit");

        let mut warnings = Vec::new();

        // Extract tokens
        let tokens = self.token_extractor.extract(file)?;

        debug!(
            colors = tokens.colors.len(),
            spacing = tokens.spacing.len(),
            typography = tokens.typography.len(),
            "Tokens extracted"
        );

        // Generate theme
        let theme = self.token_extractor.to_theme(
            &tokens,
            &self.config.theme_name,
            self.config.is_dark,
        );

        // Map typography roles
        let typography_roles = self.map_typography_roles(&tokens.typography);

        // Map components
        let component_mappings = if self.config.generate_components {
            self.map_components(file, &mut warnings)?
        } else {
            Vec::new()
        };

        // Translate layouts
        let layouts = if self.config.generate_layouts {
            self.translate_layouts(file)?
        } else {
            Vec::new()
        };

        // Validate
        let validation = if self.config.validate {
            let report = self.validator.validate_theme(&theme);
            if !report.is_valid() {
                for error in &report.errors {
                    warnings.push(format!("Validation error: {}", error.message));
                }
            }
            Some(report)
        } else {
            None
        };

        let stats = TranslationStats {
            colors_extracted: tokens.colors.len(),
            spacing_values: tokens.spacing.len(),
            typography_styles: tokens.typography.len(),
            shadows_extracted: tokens.shadows.len(),
            components_mapped: component_mappings.len(),
            layouts_translated: layouts.len(),
            warnings_count: warnings.len(),
        };

        let result = TranslationResult {
            theme,
            tokens,
            typography_roles,
            component_mappings,
            layouts,
            validation,
            warnings,
            metadata: TranslationMetadata {
                source_file: file.name.clone(),
                translated_at: chrono::Utc::now().to_rfc3339(),
                figma_version: file.version.clone(),
                stats,
            },
        };

        info!(
            colors = result.metadata.stats.colors_extracted,
            components = result.metadata.stats.components_mapped,
            layouts = result.metadata.stats.layouts_translated,
            warnings = result.metadata.stats.warnings_count,
            "Translation complete"
        );

        Ok(result)
    }

    /// Map typography styles to roles
    fn map_typography_roles(&self, typography: &[ExtractedTypography]) -> Vec<TypographyRoleMapping> {
        let mut roles = Vec::new();
        let mut seen_roles = std::collections::HashSet::new();

        for style in typography {
            // Only include each role once (use the most common/first found)
            if seen_roles.contains(&style.role) {
                continue;
            }
            seen_roles.insert(style.role.clone());

            roles.push(TypographyRoleMapping {
                role: style.role.clone(),
                font_family: style.font_family.clone(),
                font_size: style.font_size,
                font_weight: style.font_weight,
                line_height: style.line_height,
                letter_spacing: style.letter_spacing,
            });
        }

        roles
    }

    /// Map Figma components to OxideKit components
    fn map_components(
        &self,
        file: &FigmaFile,
        warnings: &mut Vec<String>,
    ) -> Result<Vec<ComponentMappingOutput>> {
        let mappings = self.component_mapper.map_file(file)?;

        let mut outputs = Vec::new();

        for mapping in mappings {
            // Convert props to JSON values
            let mut props = HashMap::new();
            for (key, value) in mapping.props {
                let json_value = match value {
                    crate::components::PropValue::String(s) => serde_json::Value::String(s),
                    crate::components::PropValue::Number(n) => {
                        serde_json::Value::Number(serde_json::Number::from_f64(n).unwrap())
                    }
                    crate::components::PropValue::Boolean(b) => serde_json::Value::Bool(b),
                    crate::components::PropValue::Enum(e) => serde_json::Value::String(e),
                    crate::components::PropValue::Token(t) => {
                        serde_json::Value::String(format!("{{{}}}", t))
                    }
                };
                props.insert(key, json_value);
            }

            // Collect warnings
            for w in &mapping.warnings {
                warnings.push(w.clone());
            }

            outputs.push(ComponentMappingOutput {
                figma_name: mapping.figma_name,
                figma_node_id: mapping.figma_node_id,
                oxide_component: mapping.component.name().to_string(),
                confidence: mapping.confidence.value(),
                props,
                warnings: mapping.warnings,
            });
        }

        Ok(outputs)
    }

    /// Translate Figma layouts
    fn translate_layouts(&self, file: &FigmaFile) -> Result<Vec<OxideLayout>> {
        self.layout_translator.translate_file(file)
    }

    /// Export translation result to files
    pub fn export(&self, result: &TranslationResult, output_dir: &Utf8Path) -> Result<Vec<String>> {
        info!(?output_dir, "Exporting translation result");

        fs::create_dir_all(output_dir)?;

        let mut files = Vec::new();

        // Export theme
        let theme_path = output_dir.join("theme.generated.toml");
        let theme_content = result.theme.to_toml()?;
        fs::write(&theme_path, &theme_content)?;
        files.push(theme_path.to_string());

        // Export typography roles
        if !result.typography_roles.is_empty() {
            let typo_path = output_dir.join("typography.generated.toml");
            let typo_content = self.format_typography_toml(&result.typography_roles)?;
            fs::write(&typo_path, typo_content)?;
            files.push(typo_path.to_string());
        }

        // Export component mappings
        if !result.component_mappings.is_empty() {
            let components_path = output_dir.join("components.generated.json");
            let components_content = serde_json::to_string_pretty(&result.component_mappings)?;
            fs::write(&components_path, components_content)?;
            files.push(components_path.to_string());
        }

        // Export layouts
        if !result.layouts.is_empty() {
            let layouts_path = output_dir.join("layouts.generated.json");
            let layouts_content = serde_json::to_string_pretty(&result.layouts)?;
            fs::write(&layouts_path, layouts_content)?;
            files.push(layouts_path.to_string());
        }

        // Export metadata
        let meta_path = output_dir.join("figma-import.json");
        let meta_content = serde_json::to_string_pretty(&result.metadata)?;
        fs::write(&meta_path, meta_content)?;
        files.push(meta_path.to_string());

        info!(file_count = files.len(), "Export complete");

        Ok(files)
    }

    /// Format typography as TOML
    fn format_typography_toml(&self, roles: &[TypographyRoleMapping]) -> Result<String> {
        let mut output = String::new();
        output.push_str("# Generated typography roles from Figma\n\n");

        for role in roles {
            output.push_str(&format!("[{}]\n", role.role));
            output.push_str(&format!("font_family = \"{}\"\n", role.font_family));
            output.push_str(&format!("font_size = {}\n", role.font_size));
            output.push_str(&format!("font_weight = {}\n", role.font_weight));
            output.push_str(&format!("line_height = {:.2}\n", role.line_height));
            output.push_str(&format!("letter_spacing = {:.3}\n", role.letter_spacing));
            output.push_str("\n");
        }

        Ok(output)
    }
}

impl Default for Translator {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for translator configuration
#[derive(Debug, Default)]
pub struct TranslatorBuilder {
    config: TranslatorConfig,
}

impl TranslatorBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme_name(mut self, name: impl Into<String>) -> Self {
        self.config.theme_name = name.into();
        self
    }

    pub fn is_dark(mut self, dark: bool) -> Self {
        self.config.is_dark = dark;
        self
    }

    pub fn validate(mut self, validate: bool) -> Self {
        self.config.validate = validate;
        self
    }

    pub fn min_confidence(mut self, confidence: f32) -> Self {
        self.config.min_confidence = confidence;
        self
    }

    pub fn generate_layouts(mut self, generate: bool) -> Self {
        self.config.generate_layouts = generate;
        self
    }

    pub fn generate_components(mut self, generate: bool) -> Self {
        self.config.generate_components = generate;
        self
    }

    pub fn semantic_naming(mut self, semantic: bool) -> Self {
        self.config.semantic_naming = semantic;
        self
    }

    pub fn build(self) -> Translator {
        Translator::with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translator_builder() {
        let translator = TranslatorBuilder::new()
            .theme_name("My Theme")
            .is_dark(false)
            .min_confidence(0.8)
            .build();

        assert_eq!(translator.config.theme_name, "My Theme");
        assert!(!translator.config.is_dark);
        assert_eq!(translator.config.min_confidence, 0.8);
    }

    #[test]
    fn test_typography_role_mapping() {
        let role = TypographyRoleMapping {
            role: "heading".to_string(),
            font_family: "Inter".to_string(),
            font_size: 24.0,
            font_weight: 700,
            line_height: 1.2,
            letter_spacing: 0.0,
        };

        assert_eq!(role.role, "heading");
        assert_eq!(role.font_size, 24.0);
    }
}
