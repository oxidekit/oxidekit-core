//! Export control compliance checks
//!
//! Helps ensure compliance with export control regulations
//! such as EAR (Export Administration Regulations) and ITAR.

use std::collections::HashSet;
use serde::{Deserialize, Serialize};
use crate::scanner::ScanResult;

/// Export control classifications
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum ExportClassification {
    /// Not export controlled (most open source)
    Ear99,
    /// Publicly available encryption (exemption)
    PubliclyAvailable,
    /// Encryption covered under 5D002 but eligible for TSU
    Tsu,
    /// Category 5 Part 2 (encryption)
    Cat5Part2,
    /// Requires individual export license
    LicenseRequired,
    /// Unknown classification
    Unknown,
}

impl ExportClassification {
    /// Get a description of the classification
    pub fn description(&self) -> &str {
        match self {
            ExportClassification::Ear99 => "EAR99 - Not controlled, no license required",
            ExportClassification::PubliclyAvailable => "Publicly available (open source) - Exemption applies",
            ExportClassification::Tsu => "TSU - Technology and Software Unrestricted",
            ExportClassification::Cat5Part2 => "Category 5 Part 2 - Encryption items, may require license",
            ExportClassification::LicenseRequired => "License Required - Consult legal",
            ExportClassification::Unknown => "Unknown - Requires manual review",
        }
    }

    /// Check if this classification allows unrestricted distribution
    pub fn allows_distribution(&self) -> bool {
        matches!(
            self,
            ExportClassification::Ear99
                | ExportClassification::PubliclyAvailable
                | ExportClassification::Tsu
        )
    }
}

/// Export restriction type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportRestriction {
    /// No restrictions
    None,
    /// Encryption technology
    Encryption,
    /// Dual-use items
    DualUse,
    /// Country restrictions
    CountryRestricted { countries: Vec<String> },
    /// End-use restrictions
    EndUseRestricted { restrictions: Vec<String> },
}

/// Export control checker
#[derive(Debug, Clone)]
pub struct ExportControl {
    /// Project classification
    classification: ExportClassification,
    /// Known encryption libraries
    encryption_packages: HashSet<String>,
    /// Sanctioned countries (ISO 3166-1 alpha-2)
    sanctioned_countries: HashSet<String>,
    /// Whether to treat as publicly available
    is_public: bool,
}

impl Default for ExportControl {
    fn default() -> Self {
        Self::new()
    }
}

impl ExportControl {
    /// Create a new export control checker
    pub fn new() -> Self {
        Self {
            classification: ExportClassification::Ear99,
            encryption_packages: Self::default_encryption_packages(),
            sanctioned_countries: Self::default_sanctioned_countries(),
            is_public: true,
        }
    }

    /// Default list of packages that contain encryption
    fn default_encryption_packages() -> HashSet<String> {
        [
            // Rust crypto crates
            "aes",
            "aes-gcm",
            "chacha20",
            "chacha20poly1305",
            "rsa",
            "ed25519",
            "ed25519-dalek",
            "x25519-dalek",
            "curve25519-dalek",
            "p256",
            "p384",
            "ring",
            "rustls",
            "native-tls",
            "openssl",
            "openssl-sys",
            "crypto",
            "cryptography",
            "sodiumoxide",
            "libsodium-sys",
            "argon2",
            "bcrypt",
            "scrypt",
            "sha2",
            "sha3",
            "blake2",
            "blake3",
            "hmac",
            "hkdf",
            "pbkdf2",
            "aead",
            "cipher",
            "block-cipher",
            "stream-cipher",
            "signature",
            "ecdsa",
            "dsa",
            "secp256k1",
            "k256",
            // Web/TLS
            "webpki",
            "rustls-webpki",
            "tokio-rustls",
            "hyper-rustls",
            "reqwest", // Uses TLS
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Default sanctioned countries (example - consult legal for actual list)
    fn default_sanctioned_countries() -> HashSet<String> {
        // Note: This is a simplified example. Real compliance requires
        // consulting with legal and regularly updating based on
        // current OFAC, BIS, and other regulatory requirements.
        [
            "CU", // Cuba
            "IR", // Iran
            "KP", // North Korea
            "SY", // Syria
            "RU", // Russia (various programs)
        ]
        .iter()
        .map(|s| s.to_string())
        .collect()
    }

    /// Set whether project is publicly available (open source)
    pub fn set_public(mut self, is_public: bool) -> Self {
        self.is_public = is_public;
        self.classification = if is_public {
            ExportClassification::PubliclyAvailable
        } else {
            ExportClassification::Unknown
        };
        self
    }

    /// Add an encryption package to track
    pub fn add_encryption_package(&mut self, name: &str) {
        self.encryption_packages.insert(name.to_string());
    }

    /// Analyze a scan result for export control concerns
    pub fn analyze(&self, scan: &ScanResult) -> ExportAnalysis {
        let mut encryption_deps = Vec::new();
        let mut concerns = Vec::new();
        let mut recommendations = Vec::new();

        // Check for encryption dependencies
        for dep in &scan.dependencies {
            if self.encryption_packages.contains(&dep.name) {
                encryption_deps.push(EncryptionDependency {
                    name: dep.name.clone(),
                    version: dep.version.clone(),
                    description: dep.description.clone(),
                });
            }
        }

        // Determine classification based on encryption usage
        let classification = if encryption_deps.is_empty() {
            ExportClassification::Ear99
        } else if self.is_public {
            // Open source encryption typically qualifies for TSU exception
            ExportClassification::PubliclyAvailable
        } else {
            ExportClassification::Cat5Part2
        };

        // Add concerns and recommendations
        if !encryption_deps.is_empty() {
            if self.is_public {
                concerns.push(ExportConcern {
                    severity: ConcernSeverity::Info,
                    area: "Encryption".to_string(),
                    description: format!(
                        "Project uses {} encryption dependencies but qualifies for publicly available exemption",
                        encryption_deps.len()
                    ),
                });

                recommendations.push(
                    "File TSU notification with BIS if not already done".to_string()
                );
                recommendations.push(
                    "Maintain source code availability to retain exemption".to_string()
                );
            } else {
                concerns.push(ExportConcern {
                    severity: ConcernSeverity::High,
                    area: "Encryption".to_string(),
                    description: format!(
                        "Project uses {} encryption dependencies and may require export license",
                        encryption_deps.len()
                    ),
                });

                recommendations.push(
                    "Consult with legal counsel regarding ECCN classification".to_string()
                );
                recommendations.push(
                    "Consider making project open source to qualify for exemption".to_string()
                );
            }
        }

        // Check for any potential dual-use concerns
        let has_network = scan.dependencies.iter().any(|d| {
            d.name.contains("net") || d.name.contains("http") || d.name.contains("socket")
        });

        if has_network && !encryption_deps.is_empty() {
            recommendations.push(
                "Document encryption usage for potential TSU filing".to_string()
            );
        }

        let requires_notification = !encryption_deps.is_empty() && self.is_public;
        let requires_license = !encryption_deps.is_empty() && !self.is_public;

        ExportAnalysis {
            classification,
            encryption_dependencies: encryption_deps,
            concerns,
            recommendations,
            is_public_source: self.is_public,
            requires_notification,
            requires_license,
        }
    }

    /// Check if distribution to a country is allowed
    pub fn check_country(&self, country_code: &str) -> CountryCheck {
        let code = country_code.to_uppercase();
        let is_sanctioned = self.sanctioned_countries.contains(&code);

        CountryCheck {
            country_code: code.clone(),
            is_allowed: !is_sanctioned,
            restrictions: if is_sanctioned {
                vec![format!("Distribution to {} is restricted under US sanctions", code)]
            } else {
                vec![]
            },
            required_actions: if is_sanctioned {
                vec!["Consult with legal before any distribution".to_string()]
            } else {
                vec![]
            },
        }
    }

    /// Generate TSU notification template
    pub fn generate_tsu_template(&self, project_name: &str, project_url: &str) -> String {
        format!(r#"
TECHNOLOGY AND SOFTWARE UNRESTRICTED (TSU) NOTIFICATION

Submitted pursuant to 15 CFR 742.15(b)

================================================================================

1. SUBMITTER INFORMATION:
   Name: [Your Name]
   Company: [Your Company]
   Address: [Your Address]
   Email: [Your Email]

2. PRODUCT INFORMATION:
   Name: {}
   Version: [Version]
   Type: Open Source Software
   URL: {}

3. ENCRYPTION INFORMATION:
   This software includes publicly available encryption source code.

   Encryption Algorithms Used:
   [List encryption algorithms]

   Key Lengths:
   [List key lengths]

4. DISTRIBUTION:
   The software is distributed via the Internet and is publicly available
   without restriction.

5. SOURCE CODE AVAILABILITY:
   Full source code is publicly available at: {}

================================================================================

Note: This is a template. Consult with legal counsel before submission.
Submit to: crypt@bis.doc.gov and enc@nsa.gov
"#, project_name, project_url, project_url)
    }
}

/// Result of export control analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportAnalysis {
    /// Determined classification
    pub classification: ExportClassification,
    /// Encryption dependencies found
    pub encryption_dependencies: Vec<EncryptionDependency>,
    /// Export control concerns
    pub concerns: Vec<ExportConcern>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Whether source is publicly available
    pub is_public_source: bool,
    /// Whether BIS notification is required
    pub requires_notification: bool,
    /// Whether export license may be required
    pub requires_license: bool,
}

impl ExportAnalysis {
    /// Check if project can be freely distributed
    pub fn can_distribute_freely(&self) -> bool {
        self.classification.allows_distribution()
    }

    /// Get a summary string
    pub fn summary(&self) -> String {
        let mut summary = format!(
            "Export Classification: {:?}\n",
            self.classification
        );

        summary.push_str(&format!(
            "Encryption Dependencies: {}\n",
            self.encryption_dependencies.len()
        ));

        if self.requires_notification {
            summary.push_str("Action Required: File TSU notification\n");
        }

        if self.requires_license {
            summary.push_str("Action Required: Obtain export license\n");
        }

        summary
    }
}

/// Encryption dependency information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionDependency {
    /// Package name
    pub name: String,
    /// Package version
    pub version: String,
    /// Description
    pub description: Option<String>,
}

/// Export control concern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportConcern {
    /// Severity level
    pub severity: ConcernSeverity,
    /// Area of concern
    pub area: String,
    /// Description
    pub description: String,
}

/// Concern severity level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConcernSeverity {
    /// Informational
    Info,
    /// Low concern
    Low,
    /// Medium concern
    Medium,
    /// High concern - requires action
    High,
    /// Critical - must address before distribution
    Critical,
}

/// Country distribution check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CountryCheck {
    /// ISO country code
    pub country_code: String,
    /// Whether distribution is allowed
    pub is_allowed: bool,
    /// Restrictions that apply
    pub restrictions: Vec<String>,
    /// Required actions
    pub required_actions: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scanner::ScanSummary;

    fn create_test_scan() -> ScanResult {
        ScanResult {
            project_name: "test-project".to_string(),
            project_version: "1.0.0".to_string(),
            project_license: Some("MIT".to_string()),
            dependencies: vec![],
            summary: ScanSummary::default(),
        }
    }

    #[test]
    fn test_no_encryption() {
        let control = ExportControl::new().set_public(true);
        let scan = create_test_scan();
        let analysis = control.analyze(&scan);

        assert_eq!(analysis.classification, ExportClassification::Ear99);
        assert!(analysis.encryption_dependencies.is_empty());
    }

    #[test]
    fn test_public_exemption() {
        let control = ExportControl::new().set_public(true);
        assert_eq!(control.classification, ExportClassification::PubliclyAvailable);
    }

    #[test]
    fn test_sanctioned_country() {
        let control = ExportControl::new();
        let check = control.check_country("KP");

        assert!(!check.is_allowed);
        assert!(!check.restrictions.is_empty());
    }

    #[test]
    fn test_allowed_country() {
        let control = ExportControl::new();
        let check = control.check_country("DE");

        assert!(check.is_allowed);
        assert!(check.restrictions.is_empty());
    }
}
