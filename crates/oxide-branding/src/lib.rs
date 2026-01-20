//! OxideKit Design Governance & Branding System
//!
//! This crate provides comprehensive branding and design governance capabilities:
//!
//! - **Brand Packs**: Complete brand identity packages (logos, colors, assets)
//! - **App Packs**: Application-specific customizations that extend brand packs
//! - **White-label System**: Configuration for white-labeling applications
//! - **Design Token Governance**: Lock and protect critical design tokens
//! - **Brand Compliance**: Automated checking of brand guideline adherence
//! - **Asset Pipeline**: Logo/icon processing and generation
//! - **Theme Inheritance**: Brand-aware theme system with override rules
//!
//! # Architecture
//!
//! ```text
//! BrandPack (organization-level)
//!     |
//!     +-- AppPack (application-level, extends brand)
//!             |
//!             +-- Theme (runtime theme, respects locks)
//! ```
//!
//! # Example
//!
//! ```rust,ignore
//! use oxide_branding::{BrandPack, AppPack, BrandManager};
//!
//! // Load organization brand
//! let brand = BrandPack::from_file("acme-brand.toml")?;
//!
//! // Create app-specific customization
//! let app = AppPack::new("my-app")
//!     .extend_brand(&brand)
//!     .with_theme_overrides(custom_theme);
//!
//! // Check compliance
//! let manager = BrandManager::new(brand);
//! let issues = manager.check_compliance(&app)?;
//! ```

pub mod asset;
pub mod brand_pack;
pub mod app_pack;
pub mod white_label;
pub mod governance;
pub mod compliance;
pub mod export;
pub mod inheritance;
pub mod error;

pub use asset::*;
pub use brand_pack::*;
pub use app_pack::*;
pub use white_label::*;
pub use governance::*;
pub use compliance::*;
pub use export::*;
pub use inheritance::*;
pub use error::*;

/// Re-export for convenience
pub mod prelude {
    pub use crate::asset::{BrandAsset, AssetType, AssetPipeline, IconSize};
    pub use crate::brand_pack::{BrandPack, BrandIdentity, BrandColors, BrandTypography};
    pub use crate::app_pack::{AppPack, AppCustomization};
    pub use crate::white_label::{WhiteLabelConfig, WhiteLabelMode, WhiteLabelOverrides};
    pub use crate::governance::{TokenGovernance, TokenLock, GovernanceRule};
    pub use crate::error::LockLevel;
    pub use crate::compliance::{ComplianceChecker, ComplianceIssue, ComplianceLevel, ComplianceReport};
    pub use crate::export::{BrandExporter, ExportFormat, ExportOptions};
    pub use crate::inheritance::{ThemeInheritance, InheritanceChain, OverrideContext};
    pub use crate::error::{BrandingError, BrandingResult};
}

/// Brand manager - the main entry point for brand operations
#[derive(Debug)]
pub struct BrandManager {
    /// The loaded brand pack
    brand: BrandPack,

    /// Token governance rules
    governance: TokenGovernance,

    /// Compliance checker
    compliance: ComplianceChecker,

    /// Asset pipeline
    pipeline: AssetPipeline,
}

impl BrandManager {
    /// Create a new brand manager with the given brand pack
    pub fn new(brand: BrandPack) -> Self {
        let governance = TokenGovernance::from_brand(&brand);
        let compliance = ComplianceChecker::new(&brand);
        let pipeline = AssetPipeline::new();

        Self {
            brand,
            governance,
            compliance,
            pipeline,
        }
    }

    /// Get the brand pack
    pub fn brand(&self) -> &BrandPack {
        &self.brand
    }

    /// Get token governance
    pub fn governance(&self) -> &TokenGovernance {
        &self.governance
    }

    /// Get compliance checker
    pub fn compliance(&self) -> &ComplianceChecker {
        &self.compliance
    }

    /// Get asset pipeline
    pub fn pipeline(&self) -> &AssetPipeline {
        &self.pipeline
    }

    /// Check if a theme override is allowed
    pub fn can_override_token(&self, token_path: &str) -> bool {
        self.governance.can_override(token_path)
    }

    /// Check compliance of an app pack against brand guidelines
    pub fn check_app_compliance(&self, app: &AppPack) -> ComplianceReport {
        self.compliance.check_app_pack(app)
    }

    /// Create an app pack that extends this brand
    pub fn create_app_pack(&self, app_id: &str) -> AppPack {
        AppPack::new(app_id).extend_brand(&self.brand)
    }

    /// Generate icons from the brand logo
    #[cfg(feature = "image-processing")]
    pub fn generate_icons(&self, output_dir: &std::path::Path) -> BrandingResult<Vec<std::path::PathBuf>> {
        if let Some(logo) = self.brand.identity.primary_logo.as_ref() {
            self.pipeline.generate_icon_set(&logo.path, output_dir)
        } else {
            Err(BrandingError::MissingAsset("primary_logo".into()))
        }
    }

    /// Export brand assets
    pub fn export(&self, options: ExportOptions) -> BrandingResult<()> {
        let exporter = BrandExporter::new(&self.brand);
        exporter.export(options)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_brand_manager_creation() {
        let brand = BrandPack::default();
        let manager = BrandManager::new(brand);

        assert!(!manager.brand().identity.name.is_empty());
    }

    #[test]
    fn test_app_pack_creation() {
        let brand = BrandPack::default();
        let manager = BrandManager::new(brand);

        let app = manager.create_app_pack("test-app");
        assert_eq!(app.id, "test-app");
    }
}
