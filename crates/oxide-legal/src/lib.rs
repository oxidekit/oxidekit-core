//! OxideKit Legal & Business Hygiene
//!
//! Prevents ecosystem collapse due to legal ambiguity by providing
//! comprehensive license scanning, SBOM generation, and compliance tools.
//!
//! # Design Principles
//!
//! - **Clear legal standing** - Know your dependencies' licenses
//! - **Automated compliance** - Machine-enforced license checks
//! - **Transparency** - Full software bill of materials
//! - **Safe contributions** - CLA/DCO verification
//!
//! # Features
//!
//! - `full` - Enable all legal features
//! - `sbom` - Enable SBOM generation
//! - `cla` - Enable CLA checking
//! - `export-control` - Enable export control checks

mod error;
mod license;
mod scanner;
mod report;
mod policy;

#[cfg(feature = "sbom")]
mod sbom;

#[cfg(feature = "cla")]
mod cla;

#[cfg(feature = "export-control")]
mod export_control;

pub use error::*;
pub use license::*;
pub use scanner::*;
pub use report::*;
pub use policy::*;

#[cfg(feature = "sbom")]
pub use sbom::*;

#[cfg(feature = "cla")]
pub use cla::*;

#[cfg(feature = "export-control")]
pub use export_control::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        LegalError, LegalResult,
        License, LicenseType, LicenseCategory, LicenseCompatibility,
        LicenseScanner, ScanResult, DependencyLicense,
        ComplianceReport, ReportFormat, ComplianceStatus,
        LicensePolicy, PolicyRule, PolicyAction,
    };

    #[cfg(feature = "sbom")]
    pub use crate::{Sbom, SbomComponent, SbomFormat, SbomGenerator};

    #[cfg(feature = "cla")]
    pub use crate::{ClaChecker, ClaStatus, ContributorAgreement};

    #[cfg(feature = "export-control")]
    pub use crate::{ExportControl, ExportClassification, ExportRestriction};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_license_parse() {
        let license = License::parse("MIT").unwrap();
        assert_eq!(license.license_type, LicenseType::Mit);
        assert_eq!(license.category, LicenseCategory::Permissive);
    }

    #[test]
    fn test_license_compatibility() {
        let mit = License::parse("MIT").unwrap();
        let apache = License::parse("Apache-2.0").unwrap();

        assert!(mit.compatible_with(&apache));
    }

    #[test]
    fn test_dual_license() {
        let dual = License::parse("MIT OR Apache-2.0").unwrap();
        assert_eq!(dual.category, LicenseCategory::Permissive);
    }
}
