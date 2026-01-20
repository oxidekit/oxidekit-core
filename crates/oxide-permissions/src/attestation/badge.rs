//! Attestation badges for marketplace display.

use serde::{Deserialize, Serialize};

/// Badge definitions for OxideKit marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeDefinition {
    /// Badge identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Short description.
    pub description: String,
    /// Long description with requirements.
    pub long_description: String,
    /// Icon identifier.
    pub icon: String,
    /// Badge category.
    pub category: BadgeCategory,
    /// Badge tier.
    pub tier: BadgeTier,
    /// Requirements to earn this badge.
    pub requirements: Vec<BadgeRequirement>,
}

/// Badge category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeCategory {
    /// Security-related badges.
    Security,
    /// Privacy-related badges.
    Privacy,
    /// Quality-related badges.
    Quality,
    /// Trust-related badges.
    Trust,
    /// Performance-related badges.
    Performance,
}

/// Badge tier (importance level).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BadgeTier {
    /// Basic tier.
    Bronze,
    /// Intermediate tier.
    Silver,
    /// Advanced tier.
    Gold,
    /// Highest tier.
    Platinum,
}

/// Requirement for earning a badge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BadgeRequirement {
    /// Requirement description.
    pub description: String,
    /// Type of check.
    pub check_type: String,
    /// Whether this requirement is mandatory.
    pub mandatory: bool,
}

/// Badge registry with all defined badges.
pub struct BadgeRegistry {
    badges: Vec<BadgeDefinition>,
}

impl BadgeRegistry {
    /// Create a new badge registry with default badges.
    pub fn new() -> Self {
        let mut registry = Self {
            badges: Vec::new(),
        };
        registry.register_defaults();
        registry
    }

    /// Register default OxideKit badges.
    fn register_defaults(&mut self) {
        // Network Allowlist Badge
        self.badges.push(BadgeDefinition {
            id: "network_allowlist".to_string(),
            name: "Network Allowlist".to_string(),
            description: "Network connections restricted to declared domains".to_string(),
            long_description: "This application enforces a network allowlist policy, meaning \
                              it can only connect to pre-declared domains. This prevents \
                              unexpected data exfiltration and provides transparency about \
                              network activity."
                .to_string(),
            icon: "shield".to_string(),
            category: BadgeCategory::Security,
            tier: BadgeTier::Gold,
            requirements: vec![
                BadgeRequirement {
                    description: "Network mode set to allowlist".to_string(),
                    check_type: "manifest_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "At least one domain declared".to_string(),
                    check_type: "manifest_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "Private IP ranges blocked".to_string(),
                    check_type: "manifest_check".to_string(),
                    mandatory: false,
                },
            ],
        });

        // Verified Build Badge
        self.badges.push(BadgeDefinition {
            id: "verified_build".to_string(),
            name: "Verified Build".to_string(),
            description: "Build verified by OxideKit attestation service".to_string(),
            long_description: "This application was built using the OxideKit verified build \
                              profile and passed all security checks. The build process \
                              ensures no forbidden APIs or dependencies are used."
                .to_string(),
            icon: "check-circle".to_string(),
            category: BadgeCategory::Security,
            tier: BadgeTier::Platinum,
            requirements: vec![
                BadgeRequirement {
                    description: "Build passes verified build profile".to_string(),
                    check_type: "build_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "No forbidden crates detected".to_string(),
                    check_type: "dependency_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "No forbidden APIs detected".to_string(),
                    check_type: "source_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // Privacy Conscious Badge
        self.badges.push(BadgeDefinition {
            id: "privacy_conscious".to_string(),
            name: "Privacy Conscious".to_string(),
            description: "Minimal data collection and strong privacy controls".to_string(),
            long_description: "This application has strong privacy controls with no automatic \
                              data collection. Crash reporting and analytics are disabled or \
                              require explicit user consent."
                .to_string(),
            icon: "eye-off".to_string(),
            category: BadgeCategory::Privacy,
            tier: BadgeTier::Gold,
            requirements: vec![
                BadgeRequirement {
                    description: "No automatic crash reporting".to_string(),
                    check_type: "privacy_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "No analytics collection".to_string(),
                    check_type: "privacy_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "Privacy score >= 80".to_string(),
                    check_type: "privacy_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // Signed Badge
        self.badges.push(BadgeDefinition {
            id: "signed".to_string(),
            name: "Signed".to_string(),
            description: "Binary signed with verified identity".to_string(),
            long_description: "This application binary is cryptographically signed, allowing \
                              verification of its authenticity and integrity. The signature \
                              proves the binary hasn't been tampered with since signing."
                .to_string(),
            icon: "pen-tool".to_string(),
            category: BadgeCategory::Trust,
            tier: BadgeTier::Silver,
            requirements: vec![
                BadgeRequirement {
                    description: "Valid code signature".to_string(),
                    check_type: "signature_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // Well Documented Badge
        self.badges.push(BadgeDefinition {
            id: "documented".to_string(),
            name: "Well Documented".to_string(),
            description: "All permissions have documented reasons".to_string(),
            long_description: "This application provides clear documentation for all requested \
                              permissions, explaining why each capability is needed."
                .to_string(),
            icon: "book".to_string(),
            category: BadgeCategory::Quality,
            tier: BadgeTier::Bronze,
            requirements: vec![
                BadgeRequirement {
                    description: "All capabilities have reason field".to_string(),
                    check_type: "manifest_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // Minimal Permissions Badge
        self.badges.push(BadgeDefinition {
            id: "minimal_permissions".to_string(),
            name: "Minimal Permissions".to_string(),
            description: "Requests only essential permissions".to_string(),
            long_description: "This application follows the principle of least privilege, \
                              requesting only the minimum permissions necessary for its \
                              functionality."
                .to_string(),
            icon: "lock".to_string(),
            category: BadgeCategory::Security,
            tier: BadgeTier::Silver,
            requirements: vec![
                BadgeRequirement {
                    description: "No critical-risk permissions".to_string(),
                    check_type: "permission_check".to_string(),
                    mandatory: true,
                },
                BadgeRequirement {
                    description: "Less than 5 total permissions".to_string(),
                    check_type: "permission_check".to_string(),
                    mandatory: false,
                },
            ],
        });

        // No Network Badge
        self.badges.push(BadgeDefinition {
            id: "no_network".to_string(),
            name: "Offline Capable".to_string(),
            description: "Works completely offline".to_string(),
            long_description: "This application does not require any network access and works \
                              completely offline. Your data stays on your device."
                .to_string(),
            icon: "wifi-off".to_string(),
            category: BadgeCategory::Privacy,
            tier: BadgeTier::Gold,
            requirements: vec![
                BadgeRequirement {
                    description: "No network capability requested".to_string(),
                    check_type: "permission_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // HTTPS Only Badge
        self.badges.push(BadgeDefinition {
            id: "https_only".to_string(),
            name: "HTTPS Only".to_string(),
            description: "All connections use secure HTTPS".to_string(),
            long_description: "This application enforces HTTPS for all network connections, \
                              ensuring data is encrypted in transit."
                .to_string(),
            icon: "lock".to_string(),
            category: BadgeCategory::Security,
            tier: BadgeTier::Silver,
            requirements: vec![
                BadgeRequirement {
                    description: "require_https enabled in network policy".to_string(),
                    check_type: "manifest_check".to_string(),
                    mandatory: true,
                },
            ],
        });

        // Official Badge
        self.badges.push(BadgeDefinition {
            id: "official".to_string(),
            name: "Official".to_string(),
            description: "Published by OxideKit organization".to_string(),
            long_description: "This is an official OxideKit application or plugin, maintained \
                              by the OxideKit core team."
                .to_string(),
            icon: "award".to_string(),
            category: BadgeCategory::Trust,
            tier: BadgeTier::Platinum,
            requirements: vec![
                BadgeRequirement {
                    description: "Published by oxidekit org".to_string(),
                    check_type: "publisher_check".to_string(),
                    mandatory: true,
                },
            ],
        });
    }

    /// Get all badge definitions.
    pub fn all(&self) -> &[BadgeDefinition] {
        &self.badges
    }

    /// Get a badge by ID.
    pub fn get(&self, id: &str) -> Option<&BadgeDefinition> {
        self.badges.iter().find(|b| b.id == id)
    }

    /// Get badges by category.
    pub fn by_category(&self, category: BadgeCategory) -> Vec<&BadgeDefinition> {
        self.badges.iter().filter(|b| b.category == category).collect()
    }

    /// Get badges by tier.
    pub fn by_tier(&self, tier: BadgeTier) -> Vec<&BadgeDefinition> {
        self.badges.iter().filter(|b| b.tier == tier).collect()
    }
}

impl Default for BadgeRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Marketplace display information for an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceDisplay {
    /// App name.
    pub app_name: String,
    /// App version.
    pub app_version: String,
    /// Publisher name.
    pub publisher: Option<String>,
    /// Earned badges.
    pub badges: Vec<EarnedBadge>,
    /// Permission summary.
    pub permission_summary: PermissionSummaryDisplay,
    /// Network summary.
    pub network_summary: NetworkSummaryDisplay,
    /// Trust level.
    pub trust_level: String,
    /// Attestation URL (if available).
    pub attestation_url: Option<String>,
    /// Last verified date.
    pub last_verified: Option<String>,
}

/// An earned badge for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EarnedBadge {
    /// Badge ID.
    pub id: String,
    /// Badge name.
    pub name: String,
    /// Short description.
    pub description: String,
    /// Icon.
    pub icon: String,
    /// Tier.
    pub tier: BadgeTier,
}

/// Permission summary for marketplace display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSummaryDisplay {
    /// List of permission categories used.
    pub categories: Vec<String>,
    /// Number of permissions.
    pub count: usize,
    /// Highest risk level.
    pub max_risk: String,
}

/// Network summary for marketplace display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSummaryDisplay {
    /// Whether network is used.
    pub uses_network: bool,
    /// Whether allowlist is enforced.
    pub allowlist_enforced: bool,
    /// Status text.
    pub status: String,
    /// Allowed domains (if public).
    pub domains: Vec<String>,
}

impl MarketplaceDisplay {
    /// Create from an attestation report.
    pub fn from_report(
        report: &super::report::AttestationReport,
        registry: &BadgeRegistry,
    ) -> Self {
        let badges: Vec<EarnedBadge> = report
            .badges
            .iter()
            .filter(|b| b.earned)
            .filter_map(|b| {
                registry.get(&b.id).map(|def| EarnedBadge {
                    id: b.id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    icon: def.icon.clone(),
                    tier: def.tier,
                })
            })
            .collect();

        let permission_categories: Vec<String> = report
            .permissions
            .by_category
            .keys()
            .cloned()
            .collect();

        Self {
            app_name: report.app.name.clone(),
            app_version: report.app.version.clone(),
            publisher: report.app.publisher.clone(),
            badges,
            permission_summary: PermissionSummaryDisplay {
                categories: permission_categories,
                count: report.permissions.total_capabilities,
                max_risk: format!("{:?}", report.permissions.max_risk_level),
            },
            network_summary: NetworkSummaryDisplay {
                uses_network: report.network.uses_network,
                allowlist_enforced: report.network.allowlist_enforced,
                status: report.network.enforcement_status.clone(),
                domains: report.network.allowed_domains.clone(),
            },
            trust_level: report.trust_classification.level.to_string(),
            attestation_url: None,
            last_verified: Some(report.generated_at.to_rfc3339()),
        }
    }

    /// Serialize to JSON for API responses.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_badge_registry() {
        let registry = BadgeRegistry::new();

        assert!(!registry.all().is_empty());
        assert!(registry.get("network_allowlist").is_some());
        assert!(registry.get("nonexistent").is_none());
    }

    #[test]
    fn test_badge_categories() {
        let registry = BadgeRegistry::new();

        let security_badges = registry.by_category(BadgeCategory::Security);
        assert!(!security_badges.is_empty());
    }

    #[test]
    fn test_badge_tiers() {
        let registry = BadgeRegistry::new();

        let platinum_badges = registry.by_tier(BadgeTier::Platinum);
        assert!(!platinum_badges.is_empty());
        assert!(platinum_badges.iter().any(|b| b.id == "verified_build"));
    }

    #[test]
    fn test_tier_ordering() {
        assert!(BadgeTier::Platinum > BadgeTier::Gold);
        assert!(BadgeTier::Gold > BadgeTier::Silver);
        assert!(BadgeTier::Silver > BadgeTier::Bronze);
    }
}
