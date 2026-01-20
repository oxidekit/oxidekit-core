//! OxideKit HTML Theme Migrator
//!
//! This crate provides migration tools for converting HTML themes from various
//! CSS frameworks (Bootstrap, Tailwind, ThemeForest templates, etc.) to OxideKit
//! design tokens, components, and project structures.
//!
//! # Overview
//!
//! The migration process consists of several phases:
//!
//! 1. **Analysis** - Detect the source CSS framework, inventory components, and
//!    calculate migration confidence
//! 2. **Token Extraction** - Extract design tokens (colors, typography, spacing,
//!    radii, shadows) from CSS variables and common patterns
//! 3. **Component Mapping** - Map source framework components to OxideKit
//!    equivalents with property transformations
//! 4. **Conversion** - Generate a complete OxideKit project with theme files,
//!    layouts, and placeholder pages
//! 5. **Reporting** - Generate comprehensive migration reports in markdown and JSON
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_migrate::{Migrator, MigrationConfig, StarterTemplate};
//! use std::path::Path;
//!
//! // Create migrator with custom configuration
//! let config = MigrationConfig {
//!     starter: StarterTemplate::AdminPanel,
//!     project_name: "my-migrated-app".into(),
//!     apply_theme: true,
//!     generate_placeholders: true,
//!     ..Default::default()
//! };
//!
//! let migrator = Migrator::new(config)?;
//!
//! // Run full migration
//! let result = migrator.migrate(Path::new("./my-bootstrap-template"))?;
//!
//! // Access migration results
//! println!("Migration confidence: {:.0}%", result.confidence * 100.0);
//! println!("Files generated: {}", result.output.files.len());
//! println!("TODOs remaining: {}", result.output.todos.len());
//!
//! // Generate and save report
//! let report = migrator.generate_report(&result)?;
//! std::fs::write("migration-report.md", report.to_markdown())?;
//! ```
//!
//! # CLI Usage
//!
//! The migrator can also be used via the OxideKit CLI:
//!
//! ```bash
//! # Analyze a template
//! oxide migrate analyze ./my-template
//!
//! # Extract tokens only
//! oxide migrate tokens ./my-template --out theme.generated.toml
//!
//! # Generate design pack parts
//! oxide migrate design ./my-template --out design.generated/
//!
//! # Full conversion
//! oxide migrate convert ./my-template --starter admin-panel --apply-theme theme.generated
//! ```
//!
//! # Supported Frameworks
//!
//! - Bootstrap (v4, v5)
//! - Tailwind CSS
//! - Bulma (partial)
//! - Foundation (partial)
//! - Custom/unknown (best-effort)
//!
//! # Architecture
//!
//! ```text
//! Source Template (HTML/CSS/ZIP)
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │   Analyzer  │  ← Framework detection, component inventory
//!    └─────────────┘
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │   Tokens    │  ← CSS variable extraction, token normalization
//!    └─────────────┘
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │   Mapper    │  ← Component mapping, layout detection
//!    └─────────────┘
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │  Converter  │  ← Project generation, file output
//!    └─────────────┘
//!         │
//!         ▼
//!    ┌─────────────┐
//!    │   Report    │  ← Migration report generation
//!    └─────────────┘
//!         │
//!         ▼
//! OxideKit Project Output
//! ```

pub mod analyzer;
pub mod converter;
pub mod error;
pub mod mapper;
pub mod report;
pub mod tokens;

pub use analyzer::{
    AnalysisResult, Analyzer, ComponentInventory, ComponentType, DetectedComponent,
    FileAnalysisSummary, Framework, FrameworkVersion,
};
pub use converter::{
    Converter, FileType, GeneratedFile, MigrationConfig, MigrationOutput, MigrationSummary,
    StarterTemplate, TodoCategory, TodoItem,
};
pub use error::{IssueCategory, MigrateError, MigrateResult, MigrationIssue, Severity};
pub use mapper::{
    ComponentMapper, ComponentMapping, DesignPart, LayoutMapping, LayoutPattern, MappingResult,
    NavbarConfig, SidebarConfig,
};
pub use report::{MigrationReport, ReportGenerator};
pub use tokens::{
    ExtractedFonts, ExtractedTokens, ExtractedTypography, TokenConfidence, TokenExtractor,
};

use std::path::Path;

/// Complete migration result
#[derive(Debug)]
pub struct MigrationResult {
    /// Analysis result
    pub analysis: AnalysisResult,
    /// Extracted tokens
    pub tokens: ExtractedTokens,
    /// Component mappings
    pub mappings: MappingResult,
    /// Generated output
    pub output: MigrationOutput,
    /// Overall migration confidence (0.0 to 1.0)
    pub confidence: f32,
}

/// Main migrator orchestrating the full migration process
pub struct Migrator {
    /// Migration configuration
    config: MigrationConfig,
    /// Framework analyzer
    analyzer: Analyzer,
    /// Token extractor
    token_extractor: TokenExtractor,
    /// Component mapper
    mapper: ComponentMapper,
    /// Project converter
    converter: Converter,
    /// Report generator
    report_generator: ReportGenerator,
}

impl Migrator {
    /// Create a new migrator with the given configuration
    pub fn new(config: MigrationConfig) -> MigrateResult<Self> {
        let converter = Converter::new(config.clone());

        Ok(Self {
            config,
            analyzer: Analyzer::new()?,
            token_extractor: TokenExtractor::new()?,
            mapper: ComponentMapper::new(),
            converter,
            report_generator: ReportGenerator::new(),
        })
    }

    /// Create a migrator with default configuration
    pub fn with_defaults() -> MigrateResult<Self> {
        Self::new(MigrationConfig::default())
    }

    /// Run the full migration process
    pub fn migrate(&self, source_path: &Path) -> MigrateResult<MigrationResult> {
        // Phase 1: Analyze source
        let analysis = self.analyzer.analyze(source_path)?;

        // Phase 2: Extract tokens
        let tokens = self.token_extractor.extract(&analysis)?;

        // Phase 3: Map components
        let mappings = self.mapper.map(&analysis)?;

        // Phase 4: Convert to OxideKit project
        let output = self.converter.convert(&analysis, &tokens, &mappings)?;

        // Calculate overall confidence
        let confidence =
            (analysis.migration_confidence + tokens.confidence.overall + mappings.confidence) / 3.0;

        Ok(MigrationResult {
            analysis,
            tokens,
            mappings,
            output,
            confidence,
        })
    }

    /// Analyze source only (without conversion)
    pub fn analyze(&self, source_path: &Path) -> MigrateResult<AnalysisResult> {
        self.analyzer.analyze(source_path)
    }

    /// Extract tokens from analysis
    pub fn extract_tokens(&self, analysis: &AnalysisResult) -> MigrateResult<ExtractedTokens> {
        self.token_extractor.extract(analysis)
    }

    /// Map components from analysis
    pub fn map_components(&self, analysis: &AnalysisResult) -> MigrateResult<MappingResult> {
        self.mapper.map(analysis)
    }

    /// Generate output from all phases
    pub fn generate_output(
        &self,
        analysis: &AnalysisResult,
        tokens: &ExtractedTokens,
        mappings: &MappingResult,
    ) -> MigrateResult<MigrationOutput> {
        self.converter.convert(analysis, tokens, mappings)
    }

    /// Write migration output to filesystem
    pub fn write_output(&self, output: &MigrationOutput) -> MigrateResult<()> {
        self.converter.write_output(output)
    }

    /// Generate migration report
    pub fn generate_report(
        &self,
        result: &MigrationResult,
        source_path: &Path,
    ) -> MigrateResult<MigrationReport> {
        self.report_generator.generate(
            &result.analysis,
            &result.tokens,
            &result.mappings,
            source_path,
            Some(&self.config.output_dir),
        )
    }

    /// Get the configuration
    pub fn config(&self) -> &MigrationConfig {
        &self.config
    }
}

/// Convenience function for quick analysis
pub fn analyze(source_path: &Path) -> MigrateResult<AnalysisResult> {
    let analyzer = Analyzer::new()?;
    analyzer.analyze(source_path)
}

/// Convenience function for quick token extraction
pub fn extract_tokens(source_path: &Path) -> MigrateResult<ExtractedTokens> {
    let analyzer = Analyzer::new()?;
    let analysis = analyzer.analyze(source_path)?;
    let extractor = TokenExtractor::new()?;
    extractor.extract(&analysis)
}

/// Convenience function for full migration with defaults
pub fn migrate(source_path: &Path, output_dir: &Path) -> MigrateResult<MigrationResult> {
    let config = MigrationConfig {
        output_dir: output_dir.to_path_buf(),
        ..Default::default()
    };
    let migrator = Migrator::new(config)?;
    migrator.migrate(source_path)
}

/// Re-export for convenience
pub mod prelude {
    pub use crate::analyzer::{AnalysisResult, Analyzer, ComponentType, Framework};
    pub use crate::converter::{Converter, MigrationConfig, MigrationOutput, StarterTemplate};
    pub use crate::error::{MigrateError, MigrateResult, MigrationIssue, Severity};
    pub use crate::mapper::{ComponentMapper, LayoutPattern, MappingResult};
    pub use crate::report::{MigrationReport, ReportGenerator};
    pub use crate::tokens::{ExtractedTokens, TokenExtractor};
    pub use crate::{analyze, extract_tokens, migrate, MigrationResult, Migrator};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migrator_creation() {
        let migrator = Migrator::with_defaults();
        assert!(migrator.is_ok());
    }

    #[test]
    fn test_config_defaults() {
        let config = MigrationConfig::default();
        assert_eq!(config.starter, StarterTemplate::AdminPanel);
        assert!(config.apply_theme);
        assert!(config.generate_placeholders);
    }

    #[test]
    fn test_starter_template_display() {
        assert_eq!(StarterTemplate::AdminPanel.to_string(), "admin-panel");
        assert_eq!(StarterTemplate::Minimal.to_string(), "minimal");
    }
}
