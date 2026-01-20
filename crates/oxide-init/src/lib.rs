//! OxideKit Project Initialization
//!
//! First-class project initialization system for OxideKit featuring:
//!
//! - Interactive wizard with guided setup
//! - Comprehensive `oxide.toml` manifest generation
//! - License selection with template generation
//! - Permissions allowlist configuration
//! - Git/GitHub integration
//! - Non-interactive mode for CI/automation
//!
//! # Example
//!
//! ```ignore
//! use oxide_init::{InitWizard, InitConfig};
//!
//! // Interactive mode
//! let wizard = InitWizard::new();
//! let config = wizard.run()?;
//! config.generate("my-app", ".")?;
//!
//! // Non-interactive mode
//! let config = InitConfig::builder()
//!     .name("my-app")
//!     .app_id("com.example.myapp")
//!     .license(License::Mit)
//!     .build()?;
//! config.generate(".", false)?;
//! ```

mod manifest;
mod license;
mod permissions;
mod wizard;
mod generator;
mod validation;
mod templates;

pub use manifest::*;
pub use license::*;
pub use permissions::*;
pub use wizard::*;
pub use generator::*;
pub use validation::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        // Manifest
        OxideManifest, AppSection, AuthorSection, MaintainerSection,
        RepositorySection, LicenseSection, PermissionsSection, PolicySection,
        WindowSection, DevSection, BuildSection, ExtensionsSection,

        // License
        License, LicenseTemplate,

        // Permissions
        CapabilityBundle, Capability, TrustLevel,

        // Wizard
        InitWizard, InitConfig, InitConfigBuilder, ProjectType, ThemeSelection,

        // Generator
        ProjectGenerator, GeneratedProject,

        // Validation
        ManifestValidator, ValidationResult, ValidationIssue,
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_creation() {
        let manifest = OxideManifest::new("test-app", "com.example.test");
        assert_eq!(manifest.app.name, "test-app");
        assert_eq!(manifest.app.id, "com.example.test");
        assert_eq!(manifest.app.version, "0.1.0");
    }

    #[test]
    fn test_license_template() {
        let license = License::Mit;
        let template = license.template("Test Project", "2024");
        assert!(template.contains("MIT License"));
    }
}
