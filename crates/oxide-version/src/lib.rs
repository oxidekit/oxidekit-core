//! OxideKit Versioning & Compatibility Enforcement
//!
//! Prevents ecosystem breakage by enforcing strict compatibility rules
//! across core, plugins, themes, and starters.
//!
//! # Design Principles
//!
//! - **No silent breakage, ever**
//! - Machine-enforced semver rules
//! - Upgrades should be boring, predictable, and safe
//! - Clear migration paths for breaking changes
//!
//! # Features
//!
//! - `full` - Enable all versioning features
//! - `migration` - Enable migration helpers
//! - `lockfile` - Enable lockfile tracking
//! - `breaking-detection` - Enable breaking change detection

mod semver;
mod constraint;
mod compatibility;
mod matrix;
mod error;
mod deprecation;

#[cfg(feature = "migration")]
mod migration;

#[cfg(feature = "lockfile")]
mod lockfile;

#[cfg(feature = "breaking-detection")]
mod breaking;

pub use semver::*;
pub use constraint::*;
pub use compatibility::*;
pub use matrix::*;
pub use error::*;
pub use deprecation::*;

#[cfg(feature = "migration")]
pub use migration::*;

#[cfg(feature = "lockfile")]
pub use lockfile::*;

#[cfg(feature = "breaking-detection")]
pub use breaking::*;

/// Prelude for convenient imports
pub mod prelude {
    pub use crate::{
        Version, VersionReq, VersionConstraint, PreRelease, BuildMetadata,
        VersionBump,
        Compatibility, CompatibilityResult, CompatibilityLevel,
        ComponentType, ComponentVersion, ComponentManifest,
        CompatibilityMatrix, MatrixEntry,
        VersionError,
        Deprecation, DeprecationLevel, DeprecationWarning,
        DeprecationRegistry, DeprecationRegistryBuilder,
    };

    #[cfg(feature = "migration")]
    pub use crate::{MigrationGuide, MigrationStep, MigrationPlan};

    #[cfg(feature = "lockfile")]
    pub use crate::{Lockfile, LockfileEntry, LockfileVersion};

    #[cfg(feature = "breaking-detection")]
    pub use crate::{BreakingChange, BreakingChangeType, ChangeAnalysis};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_parse() {
        let v = Version::parse("1.2.3").unwrap();
        assert_eq!(v.major, 1);
        assert_eq!(v.minor, 2);
        assert_eq!(v.patch, 3);
    }

    #[test]
    fn test_version_prerelease() {
        let v = Version::parse("1.0.0-alpha.1").unwrap();
        assert_eq!(v.pre, PreRelease::new("alpha.1").unwrap());
    }

    #[test]
    fn test_version_build() {
        let v = Version::parse("1.0.0+build.123").unwrap();
        assert_eq!(v.build, BuildMetadata::new("build.123"));
    }

    #[test]
    fn test_version_comparison() {
        let v1 = Version::parse("1.0.0").unwrap();
        let v2 = Version::parse("1.0.1").unwrap();
        let v3 = Version::parse("1.1.0").unwrap();
        let v4 = Version::parse("2.0.0").unwrap();

        assert!(v1 < v2);
        assert!(v2 < v3);
        assert!(v3 < v4);
    }

    #[test]
    fn test_version_req_match() {
        let req = VersionReq::parse(">=1.0.0, <2.0.0").unwrap();
        assert!(req.matches(&Version::parse("1.5.0").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
        assert!(!req.matches(&Version::parse("0.9.0").unwrap()));
    }

    #[test]
    fn test_caret_constraint() {
        let req = VersionReq::parse("^1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.9.9").unwrap()));
        assert!(!req.matches(&Version::parse("2.0.0").unwrap()));
    }

    #[test]
    fn test_tilde_constraint() {
        let req = VersionReq::parse("~1.2.3").unwrap();
        assert!(req.matches(&Version::parse("1.2.3").unwrap()));
        assert!(req.matches(&Version::parse("1.2.9").unwrap()));
        assert!(!req.matches(&Version::parse("1.3.0").unwrap()));
    }
}
