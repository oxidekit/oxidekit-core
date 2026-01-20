//! License types and classification
//!
//! Provides comprehensive license type detection and categorization
//! based on SPDX identifiers.

use serde::{Deserialize, Serialize};
use crate::error::{LegalError, LegalResult};

/// A software license
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct License {
    /// The SPDX identifier or custom identifier
    pub spdx_id: String,
    /// Parsed license type
    pub license_type: LicenseType,
    /// License category (permissive, copyleft, etc.)
    pub category: LicenseCategory,
    /// Whether this is a dual/multi license (OR expression)
    pub is_dual_license: bool,
    /// Component licenses if dual/multi licensed
    pub components: Vec<License>,
    /// OSI approved
    pub osi_approved: bool,
    /// FSF free
    pub fsf_free: bool,
}

impl License {
    /// Parse a license from an SPDX identifier
    pub fn parse(spdx: &str) -> LegalResult<Self> {
        let spdx = spdx.trim();

        // Handle OR expressions (dual licensing)
        if spdx.contains(" OR ") {
            let parts: Vec<&str> = spdx.split(" OR ").collect();
            let components: Vec<License> = parts
                .iter()
                .filter_map(|p| License::parse(p.trim()).ok())
                .collect();

            if components.is_empty() {
                return Err(LegalError::InvalidLicense(spdx.to_string()));
            }

            // Use the most permissive license category
            let category = components
                .iter()
                .map(|l| &l.category)
                .min()
                .cloned()
                .unwrap_or(LicenseCategory::Unknown);

            return Ok(License {
                spdx_id: spdx.to_string(),
                license_type: LicenseType::DualLicense,
                category,
                is_dual_license: true,
                components,
                osi_approved: true,
                fsf_free: true,
            });
        }

        // Handle AND expressions (all conditions apply)
        if spdx.contains(" AND ") {
            let parts: Vec<&str> = spdx.split(" AND ").collect();
            let components: Vec<License> = parts
                .iter()
                .filter_map(|p| License::parse(p.trim()).ok())
                .collect();

            if components.is_empty() {
                return Err(LegalError::InvalidLicense(spdx.to_string()));
            }

            // Use the most restrictive license category
            let category = components
                .iter()
                .map(|l| &l.category)
                .max()
                .cloned()
                .unwrap_or(LicenseCategory::Unknown);

            return Ok(License {
                spdx_id: spdx.to_string(),
                license_type: LicenseType::MultiLicense,
                category,
                is_dual_license: false,
                components,
                osi_approved: true,
                fsf_free: true,
            });
        }

        // Parse single license
        let (license_type, category, osi_approved, fsf_free) = match spdx.to_uppercase().as_str() {
            // Permissive licenses
            "MIT" => (LicenseType::Mit, LicenseCategory::Permissive, true, true),
            "MIT-0" => (LicenseType::Mit0, LicenseCategory::Permissive, true, true),
            "APACHE-2.0" | "APACHE" => (LicenseType::Apache2, LicenseCategory::Permissive, true, true),
            "BSD-2-CLAUSE" | "BSD-2" => (LicenseType::Bsd2, LicenseCategory::Permissive, true, true),
            "BSD-3-CLAUSE" | "BSD-3" => (LicenseType::Bsd3, LicenseCategory::Permissive, true, true),
            "ISC" => (LicenseType::Isc, LicenseCategory::Permissive, true, true),
            "UNLICENSE" | "UNLICENSED" => (LicenseType::Unlicense, LicenseCategory::PublicDomain, true, true),
            "CC0-1.0" | "CC0" => (LicenseType::Cc0, LicenseCategory::PublicDomain, true, true),
            "WTFPL" => (LicenseType::Wtfpl, LicenseCategory::PublicDomain, false, true),
            "ZLIB" => (LicenseType::Zlib, LicenseCategory::Permissive, true, true),
            "BSL-1.0" | "BOOST" => (LicenseType::Boost, LicenseCategory::Permissive, true, true),
            "0BSD" => (LicenseType::ZeroBsd, LicenseCategory::PublicDomain, true, true),

            // Weak copyleft
            "MPL-2.0" | "MPL" => (LicenseType::Mpl2, LicenseCategory::WeakCopyleft, true, true),
            "LGPL-2.1" | "LGPL-2.1-ONLY" | "LGPL-2.1-OR-LATER" =>
                (LicenseType::Lgpl21, LicenseCategory::WeakCopyleft, true, true),
            "LGPL-3.0" | "LGPL-3.0-ONLY" | "LGPL-3.0-OR-LATER" =>
                (LicenseType::Lgpl3, LicenseCategory::WeakCopyleft, true, true),
            "EPL-1.0" => (LicenseType::Epl1, LicenseCategory::WeakCopyleft, true, true),
            "EPL-2.0" => (LicenseType::Epl2, LicenseCategory::WeakCopyleft, true, true),
            "CDDL-1.0" => (LicenseType::Cddl, LicenseCategory::WeakCopyleft, true, false),

            // Strong copyleft
            "GPL-2.0" | "GPL-2.0-ONLY" | "GPL-2.0-OR-LATER" =>
                (LicenseType::Gpl2, LicenseCategory::StrongCopyleft, true, true),
            "GPL-3.0" | "GPL-3.0-ONLY" | "GPL-3.0-OR-LATER" =>
                (LicenseType::Gpl3, LicenseCategory::StrongCopyleft, true, true),
            "AGPL-3.0" | "AGPL-3.0-ONLY" | "AGPL-3.0-OR-LATER" =>
                (LicenseType::Agpl3, LicenseCategory::NetworkCopyleft, true, true),

            // Proprietary / Commercial
            "PROPRIETARY" | "COMMERCIAL" =>
                (LicenseType::Proprietary, LicenseCategory::Proprietary, false, false),

            // Unknown
            _ => {
                // Try to infer from common patterns
                let upper = spdx.to_uppercase();
                if upper.contains("MIT") {
                    (LicenseType::MitLike, LicenseCategory::Permissive, false, false)
                } else if upper.contains("BSD") {
                    (LicenseType::BsdLike, LicenseCategory::Permissive, false, false)
                } else if upper.contains("APACHE") {
                    (LicenseType::ApacheLike, LicenseCategory::Permissive, false, false)
                } else if upper.contains("GPL") {
                    (LicenseType::GplLike, LicenseCategory::StrongCopyleft, false, false)
                } else {
                    (LicenseType::Unknown, LicenseCategory::Unknown, false, false)
                }
            }
        };

        Ok(License {
            spdx_id: spdx.to_string(),
            license_type,
            category,
            is_dual_license: false,
            components: vec![],
            osi_approved,
            fsf_free,
        })
    }

    /// Check if this license is compatible with another license
    pub fn compatible_with(&self, other: &License) -> bool {
        LicenseCompatibility::check(self, other).is_compatible
    }

    /// Check if this license allows commercial use
    pub fn allows_commercial(&self) -> bool {
        !matches!(self.category, LicenseCategory::Proprietary)
    }

    /// Check if this license requires attribution
    pub fn requires_attribution(&self) -> bool {
        !matches!(
            self.license_type,
            LicenseType::Unlicense
                | LicenseType::Cc0
                | LicenseType::ZeroBsd
                | LicenseType::Wtfpl
        )
    }

    /// Check if this license requires source disclosure
    pub fn requires_source_disclosure(&self) -> bool {
        matches!(
            self.category,
            LicenseCategory::StrongCopyleft
                | LicenseCategory::NetworkCopyleft
        )
    }

    /// Check if this license has file-level copyleft (weak copyleft)
    pub fn has_file_copyleft(&self) -> bool {
        matches!(self.category, LicenseCategory::WeakCopyleft)
    }

    /// Get a human-readable name for the license
    pub fn display_name(&self) -> &str {
        match self.license_type {
            LicenseType::Mit => "MIT License",
            LicenseType::Mit0 => "MIT No Attribution",
            LicenseType::Apache2 => "Apache License 2.0",
            LicenseType::Bsd2 => "BSD 2-Clause License",
            LicenseType::Bsd3 => "BSD 3-Clause License",
            LicenseType::Isc => "ISC License",
            LicenseType::Unlicense => "The Unlicense",
            LicenseType::Cc0 => "Creative Commons Zero v1.0",
            LicenseType::Wtfpl => "WTFPL",
            LicenseType::Zlib => "zlib License",
            LicenseType::Boost => "Boost Software License 1.0",
            LicenseType::ZeroBsd => "Zero-Clause BSD",
            LicenseType::Mpl2 => "Mozilla Public License 2.0",
            LicenseType::Lgpl21 => "GNU Lesser GPL v2.1",
            LicenseType::Lgpl3 => "GNU Lesser GPL v3.0",
            LicenseType::Epl1 => "Eclipse Public License 1.0",
            LicenseType::Epl2 => "Eclipse Public License 2.0",
            LicenseType::Cddl => "Common Development and Distribution License",
            LicenseType::Gpl2 => "GNU GPL v2.0",
            LicenseType::Gpl3 => "GNU GPL v3.0",
            LicenseType::Agpl3 => "GNU Affero GPL v3.0",
            LicenseType::Proprietary => "Proprietary License",
            LicenseType::DualLicense => "Dual License",
            LicenseType::MultiLicense => "Multi License",
            LicenseType::MitLike => "MIT-like License",
            LicenseType::BsdLike => "BSD-like License",
            LicenseType::ApacheLike => "Apache-like License",
            LicenseType::GplLike => "GPL-like License",
            LicenseType::Unknown => "Unknown License",
        }
    }
}

/// Known license types
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum LicenseType {
    // Permissive
    Mit,
    Mit0,
    Apache2,
    Bsd2,
    Bsd3,
    Isc,
    Unlicense,
    Cc0,
    Wtfpl,
    Zlib,
    Boost,
    ZeroBsd,

    // Weak copyleft
    Mpl2,
    Lgpl21,
    Lgpl3,
    Epl1,
    Epl2,
    Cddl,

    // Strong copyleft
    Gpl2,
    Gpl3,
    Agpl3,

    // Proprietary
    Proprietary,

    // Composite
    DualLicense,
    MultiLicense,

    // Inferred
    MitLike,
    BsdLike,
    ApacheLike,
    GplLike,

    // Unknown
    Unknown,
}

/// License category for compatibility checking
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "kebab-case")]
pub enum LicenseCategory {
    /// Public domain - no restrictions
    PublicDomain = 0,
    /// Permissive - attribution required, few restrictions
    Permissive = 1,
    /// Weak copyleft - file-level copyleft
    WeakCopyleft = 2,
    /// Strong copyleft - project-level copyleft
    StrongCopyleft = 3,
    /// Network copyleft - includes network use
    NetworkCopyleft = 4,
    /// Proprietary - not open source
    Proprietary = 5,
    /// Unknown - could not determine
    Unknown = 6,
}

impl LicenseCategory {
    /// Get a human-readable description
    pub fn description(&self) -> &str {
        match self {
            LicenseCategory::PublicDomain => "No restrictions, public domain equivalent",
            LicenseCategory::Permissive => "Permissive, attribution typically required",
            LicenseCategory::WeakCopyleft => "File-level copyleft, changes to files must be shared",
            LicenseCategory::StrongCopyleft => "Strong copyleft, derivative works must use same license",
            LicenseCategory::NetworkCopyleft => "Network copyleft, network use triggers copyleft",
            LicenseCategory::Proprietary => "Proprietary, not open source",
            LicenseCategory::Unknown => "Unknown license category",
        }
    }

    /// Check if this category is generally safe for commercial use
    pub fn commercial_safe(&self) -> bool {
        matches!(
            self,
            LicenseCategory::PublicDomain
                | LicenseCategory::Permissive
                | LicenseCategory::WeakCopyleft
        )
    }
}

/// Result of license compatibility check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseCompatibility {
    /// Whether the licenses are compatible
    pub is_compatible: bool,
    /// Compatibility notes/warnings
    pub notes: Vec<String>,
    /// Required actions for compliance
    pub requirements: Vec<String>,
}

impl LicenseCompatibility {
    /// Check compatibility between two licenses
    pub fn check(source: &License, target: &License) -> Self {
        let mut notes = Vec::new();
        let mut requirements = Vec::new();

        // Handle dual licenses - check if any combination works
        if source.is_dual_license {
            for component in &source.components {
                let result = Self::check(component, target);
                if result.is_compatible {
                    return result;
                }
            }
        }

        if target.is_dual_license {
            for component in &target.components {
                let result = Self::check(source, component);
                if result.is_compatible {
                    return result;
                }
            }
        }

        // Category-based compatibility
        let is_compatible = match (&source.category, &target.category) {
            // Public domain is compatible with everything
            (LicenseCategory::PublicDomain, _) => true,
            (_, LicenseCategory::PublicDomain) => true,

            // Permissive licenses are generally compatible
            (LicenseCategory::Permissive, LicenseCategory::Permissive) => true,
            (LicenseCategory::Permissive, _) => {
                notes.push("Permissive license used with more restrictive license".to_string());
                true
            }

            // Weak copyleft
            (LicenseCategory::WeakCopyleft, LicenseCategory::Permissive) => true,
            (LicenseCategory::WeakCopyleft, LicenseCategory::WeakCopyleft) => {
                requirements.push("Changes to licensed files must be shared".to_string());
                true
            }
            (LicenseCategory::WeakCopyleft, LicenseCategory::StrongCopyleft) => {
                notes.push("Weak copyleft may be subsumed by strong copyleft".to_string());
                true
            }

            // Strong copyleft - requires careful handling
            (LicenseCategory::StrongCopyleft, LicenseCategory::Permissive) => true,
            (LicenseCategory::StrongCopyleft, LicenseCategory::WeakCopyleft) => {
                requirements.push("Entire project may need to be GPL-licensed".to_string());
                true
            }
            (LicenseCategory::StrongCopyleft, LicenseCategory::StrongCopyleft) => {
                // Check GPL version compatibility
                if source.license_type == target.license_type {
                    requirements.push("Project must use the same GPL license".to_string());
                    true
                } else {
                    notes.push("Different GPL versions may not be compatible".to_string());
                    false
                }
            }

            // Proprietary is generally incompatible with copyleft
            (LicenseCategory::Proprietary, LicenseCategory::StrongCopyleft)
            | (LicenseCategory::Proprietary, LicenseCategory::NetworkCopyleft) => {
                notes.push("Proprietary license incompatible with copyleft".to_string());
                false
            }
            (LicenseCategory::StrongCopyleft, LicenseCategory::Proprietary)
            | (LicenseCategory::NetworkCopyleft, LicenseCategory::Proprietary) => {
                notes.push("Copyleft license incompatible with proprietary".to_string());
                false
            }
            (LicenseCategory::Proprietary, _) => {
                notes.push("Proprietary license - check terms carefully".to_string());
                true
            }
            (_, LicenseCategory::Proprietary) => {
                notes.push("Using proprietary dependency - check terms".to_string());
                true
            }

            // Network copyleft is the most restrictive
            (LicenseCategory::NetworkCopyleft, _) => {
                requirements.push("AGPL requires source disclosure for network use".to_string());
                true
            }
            (_, LicenseCategory::NetworkCopyleft) => {
                requirements.push("Using AGPL dependency triggers network copyleft".to_string());
                true
            }

            // Unknown requires manual review
            (LicenseCategory::Unknown, _) | (_, LicenseCategory::Unknown) => {
                notes.push("Unknown license - requires manual review".to_string());
                false
            }
        };

        // Add attribution requirements
        if source.requires_attribution() {
            requirements.push(format!("Attribution required for {}", source.spdx_id));
        }
        if target.requires_attribution() && source != target {
            requirements.push(format!("Attribution required for {}", target.spdx_id));
        }

        LicenseCompatibility {
            is_compatible,
            notes,
            requirements,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_mit() {
        let license = License::parse("MIT").unwrap();
        assert_eq!(license.license_type, LicenseType::Mit);
        assert_eq!(license.category, LicenseCategory::Permissive);
        assert!(license.osi_approved);
    }

    #[test]
    fn test_parse_apache() {
        let license = License::parse("Apache-2.0").unwrap();
        assert_eq!(license.license_type, LicenseType::Apache2);
        assert_eq!(license.category, LicenseCategory::Permissive);
    }

    #[test]
    fn test_parse_dual_license() {
        let license = License::parse("MIT OR Apache-2.0").unwrap();
        assert!(license.is_dual_license);
        assert_eq!(license.components.len(), 2);
        assert_eq!(license.category, LicenseCategory::Permissive);
    }

    #[test]
    fn test_parse_gpl() {
        let license = License::parse("GPL-3.0").unwrap();
        assert_eq!(license.license_type, LicenseType::Gpl3);
        assert_eq!(license.category, LicenseCategory::StrongCopyleft);
        assert!(license.requires_source_disclosure());
    }

    #[test]
    fn test_compatibility_permissive() {
        let mit = License::parse("MIT").unwrap();
        let apache = License::parse("Apache-2.0").unwrap();
        assert!(mit.compatible_with(&apache));
    }

    #[test]
    fn test_compatibility_copyleft() {
        let mit = License::parse("MIT").unwrap();
        let gpl = License::parse("GPL-3.0").unwrap();
        // MIT can be used in GPL projects
        assert!(mit.compatible_with(&gpl));
    }
}
