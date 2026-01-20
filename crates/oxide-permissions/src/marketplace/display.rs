//! Marketplace display components.

use serde::{Deserialize, Serialize};

use crate::attestation::{AttestationReport, BadgeRegistry, TrustLevel};
use crate::capabilities::CapabilityCategory;

/// Complete marketplace listing for an app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceListing {
    /// Application identifier.
    pub app_id: String,
    /// Application name.
    pub name: String,
    /// Application version.
    pub version: String,
    /// Publisher information.
    pub publisher: PublisherInfo,
    /// Trust and verification status.
    pub trust_status: TrustStatus,
    /// Security summary.
    pub security: SecuritySummary,
    /// Privacy summary.
    pub privacy: PrivacyOverview,
    /// Badges earned.
    pub badges: Vec<DisplayBadge>,
    /// Quick facts for display.
    pub quick_facts: Vec<QuickFact>,
}

/// Publisher information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    /// Publisher name.
    pub name: String,
    /// Publisher ID.
    pub id: Option<String>,
    /// Whether publisher is verified.
    pub verified: bool,
    /// Publisher website.
    pub website: Option<String>,
    /// Publisher support email.
    pub support_email: Option<String>,
}

/// Trust and verification status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrustStatus {
    /// Trust level.
    pub level: TrustLevel,
    /// Human-readable status.
    pub status_text: String,
    /// Whether build is verified.
    pub verified_build: bool,
    /// Last verification date.
    pub last_verified: Option<String>,
    /// Attestation report URL.
    pub attestation_url: Option<String>,
    /// Trust score (0-100).
    pub trust_score: u8,
}

/// Security summary for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySummary {
    /// Permission categories.
    pub permission_categories: Vec<PermissionCategoryInfo>,
    /// Total permission count.
    pub total_permissions: usize,
    /// Network policy status.
    pub network_status: NetworkStatus,
    /// Security score (0-100).
    pub security_score: u8,
    /// Security highlights.
    pub highlights: Vec<String>,
    /// Security concerns.
    pub concerns: Vec<String>,
}

/// Permission category information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionCategoryInfo {
    /// Category.
    pub category: String,
    /// Category icon.
    pub icon: String,
    /// Permissions in this category.
    pub permissions: Vec<PermissionInfo>,
    /// Risk level.
    pub risk_level: String,
}

/// Individual permission information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionInfo {
    /// Permission name.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Description.
    pub description: Option<String>,
    /// Why the app needs it.
    pub reason: Option<String>,
    /// Whether it's granted.
    pub status: String,
}

/// Network policy status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Whether app uses network.
    pub uses_network: bool,
    /// Whether allowlist is enforced.
    pub allowlist_enforced: bool,
    /// Status text.
    pub status_text: String,
    /// Allowed domains (if disclosed).
    pub allowed_domains: Vec<String>,
    /// Whether domains are fully disclosed.
    pub domains_disclosed: bool,
}

/// Privacy overview for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyOverview {
    /// Privacy score (0-100).
    pub privacy_score: u8,
    /// Privacy grade (A-F).
    pub privacy_grade: String,
    /// Data collection status.
    pub data_collection: DataCollectionStatus,
    /// Privacy highlights.
    pub highlights: Vec<String>,
    /// Privacy concerns.
    pub concerns: Vec<String>,
}

/// Data collection status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataCollectionStatus {
    /// Crash reporting.
    pub crash_reporting: CollectionItem,
    /// Analytics.
    pub analytics: CollectionItem,
    /// Telemetry.
    pub telemetry: CollectionItem,
}

/// Individual collection item status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItem {
    /// Item name.
    pub name: String,
    /// Whether enabled.
    pub enabled: bool,
    /// Whether requires consent.
    pub requires_consent: bool,
    /// Status text.
    pub status_text: String,
}

/// Badge for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayBadge {
    /// Badge ID.
    pub id: String,
    /// Badge name.
    pub name: String,
    /// Short description.
    pub description: String,
    /// Icon.
    pub icon: String,
    /// Tier (bronze, silver, gold, platinum).
    pub tier: String,
    /// Category.
    pub category: String,
}

/// Quick fact for display.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickFact {
    /// Fact icon.
    pub icon: String,
    /// Fact text.
    pub text: String,
    /// Whether it's positive.
    pub positive: bool,
}

impl MarketplaceListing {
    /// Create a marketplace listing from an attestation report.
    pub fn from_report(report: &AttestationReport, badge_registry: &BadgeRegistry) -> Self {
        // Build permission categories
        let mut permission_categories = Vec::new();
        for (category, perms) in &report.permissions.by_category {
            let cat_enum = match category.as_str() {
                "Filesystem" => CapabilityCategory::Filesystem,
                "Keychain" => CapabilityCategory::Keychain,
                "Network" => CapabilityCategory::Network,
                "Camera" => CapabilityCategory::Camera,
                "Microphone" => CapabilityCategory::Microphone,
                "Screenshot" => CapabilityCategory::Screenshot,
                "Clipboard" => CapabilityCategory::Clipboard,
                "Background" => CapabilityCategory::Background,
                "Notifications" => CapabilityCategory::Notifications,
                "System" => CapabilityCategory::System,
                "Location" => CapabilityCategory::Location,
                _ => CapabilityCategory::Custom,
            };

            permission_categories.push(PermissionCategoryInfo {
                category: category.clone(),
                icon: icon_for_category(cat_enum),
                permissions: perms
                    .iter()
                    .map(|p| PermissionInfo {
                        name: p.clone(),
                        display_name: format_permission_name(p),
                        description: None,
                        reason: None,
                        status: "granted".to_string(),
                    })
                    .collect(),
                risk_level: format!("{:?}", cat_enum.risk_level()),
            });
        }

        // Build badges
        let badges: Vec<DisplayBadge> = report
            .badges
            .iter()
            .filter(|b| b.earned)
            .filter_map(|b| {
                badge_registry.get(&b.id).map(|def| DisplayBadge {
                    id: b.id.clone(),
                    name: def.name.clone(),
                    description: def.description.clone(),
                    icon: def.icon.clone(),
                    tier: format!("{:?}", def.tier).to_lowercase(),
                    category: format!("{:?}", def.category).to_lowercase(),
                })
            })
            .collect();

        // Build quick facts
        let mut quick_facts = Vec::new();

        if report.network.allowlist_enforced {
            quick_facts.push(QuickFact {
                icon: "shield".to_string(),
                text: "Network restricted to declared domains".to_string(),
                positive: true,
            });
        } else if report.network.uses_network {
            quick_facts.push(QuickFact {
                icon: "alert".to_string(),
                text: "Network access unrestricted".to_string(),
                positive: false,
            });
        } else {
            quick_facts.push(QuickFact {
                icon: "wifi-off".to_string(),
                text: "Works offline".to_string(),
                positive: true,
            });
        }

        if report.signature.is_valid {
            quick_facts.push(QuickFact {
                icon: "check".to_string(),
                text: "Signed by verified publisher".to_string(),
                positive: true,
            });
        }

        if report.privacy.privacy_score >= 80 {
            quick_facts.push(QuickFact {
                icon: "eye-off".to_string(),
                text: "Privacy-conscious design".to_string(),
                positive: true,
            });
        }

        // Calculate scores
        let trust_score = calculate_trust_score(report);
        let security_score = calculate_security_score(report);

        // Build security highlights and concerns
        let mut security_highlights = Vec::new();
        let mut security_concerns = Vec::new();

        if report.network.allowlist_enforced {
            security_highlights.push("Network allowlist enforced".to_string());
        } else if report.network.uses_network {
            security_concerns.push("Network access not restricted".to_string());
        }

        if report.signature.is_valid {
            security_highlights.push("Code signed".to_string());
        } else {
            security_concerns.push("Unsigned binary".to_string());
        }

        if !report.permissions.high_risk_capabilities.is_empty() {
            security_concerns.push(format!(
                "{} high-risk permissions",
                report.permissions.high_risk_capabilities.len()
            ));
        }

        // Build privacy highlights and concerns
        let mut privacy_highlights = Vec::new();
        let mut privacy_concerns = Vec::new();

        if !report.privacy.auto_crash_reporting {
            privacy_highlights.push("No automatic crash reporting".to_string());
        } else {
            privacy_concerns.push("Automatic crash reporting enabled".to_string());
        }

        if !report.privacy.analytics_enabled {
            privacy_highlights.push("No analytics collection".to_string());
        } else {
            privacy_concerns.push("Analytics collection enabled".to_string());
        }

        Self {
            app_id: report.app.app_id.clone().unwrap_or_default(),
            name: report.app.name.clone(),
            version: report.app.version.clone(),
            publisher: PublisherInfo {
                name: report.app.publisher.clone().unwrap_or_else(|| "Unknown".to_string()),
                id: None,
                verified: report.trust_classification.level >= TrustLevel::Verified,
                website: report.app.website.clone(),
                support_email: None,
            },
            trust_status: TrustStatus {
                level: report.trust_classification.level,
                status_text: format!("{}", report.trust_classification.level),
                verified_build: report.is_verified(),
                last_verified: Some(report.generated_at.to_rfc3339()),
                attestation_url: None,
                trust_score,
            },
            security: SecuritySummary {
                permission_categories,
                total_permissions: report.permissions.total_capabilities,
                network_status: NetworkStatus {
                    uses_network: report.network.uses_network,
                    allowlist_enforced: report.network.allowlist_enforced,
                    status_text: report.network.enforcement_status.clone(),
                    allowed_domains: report.network.allowed_domains.clone(),
                    domains_disclosed: report.network.allowlist_enforced,
                },
                security_score,
                highlights: security_highlights,
                concerns: security_concerns,
            },
            privacy: PrivacyOverview {
                privacy_score: report.privacy.privacy_score,
                privacy_grade: score_to_grade(report.privacy.privacy_score),
                data_collection: DataCollectionStatus {
                    crash_reporting: CollectionItem {
                        name: "Crash Reporting".to_string(),
                        enabled: report.privacy.auto_crash_reporting,
                        requires_consent: false,
                        status_text: if report.privacy.auto_crash_reporting {
                            "Automatic".to_string()
                        } else {
                            "Disabled".to_string()
                        },
                    },
                    analytics: CollectionItem {
                        name: "Analytics".to_string(),
                        enabled: report.privacy.analytics_enabled,
                        requires_consent: false,
                        status_text: if report.privacy.analytics_enabled {
                            "Enabled".to_string()
                        } else {
                            "Disabled".to_string()
                        },
                    },
                    telemetry: CollectionItem {
                        name: "Telemetry".to_string(),
                        enabled: !report.privacy.telemetry_requires_consent,
                        requires_consent: report.privacy.telemetry_requires_consent,
                        status_text: if report.privacy.telemetry_requires_consent {
                            "Consent required".to_string()
                        } else {
                            "Automatic".to_string()
                        },
                    },
                },
                highlights: privacy_highlights,
                concerns: privacy_concerns,
            },
            badges,
            quick_facts,
        }
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get the number of badges earned.
    pub fn badge_count(&self) -> usize {
        self.badges.len()
    }

    /// Check if the app meets a minimum trust level.
    pub fn meets_trust_level(&self, min_level: TrustLevel) -> bool {
        self.trust_status.level >= min_level
    }
}

fn icon_for_category(category: CapabilityCategory) -> String {
    match category {
        CapabilityCategory::Filesystem => "folder".to_string(),
        CapabilityCategory::Keychain => "key".to_string(),
        CapabilityCategory::Network => "globe".to_string(),
        CapabilityCategory::Camera => "camera".to_string(),
        CapabilityCategory::Microphone => "mic".to_string(),
        CapabilityCategory::Screenshot => "screenshot".to_string(),
        CapabilityCategory::Clipboard => "clipboard".to_string(),
        CapabilityCategory::Background => "clock".to_string(),
        CapabilityCategory::Notifications => "bell".to_string(),
        CapabilityCategory::System => "settings".to_string(),
        CapabilityCategory::Location => "location".to_string(),
        CapabilityCategory::Custom => "puzzle".to_string(),
    }
}

fn format_permission_name(name: &str) -> String {
    name.split('.')
        .map(|part| {
            let mut chars: Vec<char> = part.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn calculate_trust_score(report: &AttestationReport) -> u8 {
    let mut score: u8 = 50; // Base score

    if report.signature.is_valid {
        score = score.saturating_add(20);
    }

    if report.network.allowlist_enforced {
        score = score.saturating_add(15);
    }

    if report.permissions.all_have_reasons {
        score = score.saturating_add(10);
    }

    if report.is_verified() {
        score = score.saturating_add(5);
    }

    score.min(100)
}

fn calculate_security_score(report: &AttestationReport) -> u8 {
    let mut score: u8 = 100;

    // Deduct for high-risk permissions
    let high_risk_count = report.permissions.high_risk_capabilities.len();
    score = score.saturating_sub((high_risk_count * 5) as u8);

    // Deduct for unrestricted network
    if report.network.uses_network && !report.network.allowlist_enforced {
        score = score.saturating_sub(20);
    }

    // Deduct for missing signature
    if !report.signature.is_valid {
        score = score.saturating_sub(10);
    }

    // Deduct for critical permissions without reasons
    if !report.permissions.all_have_reasons && !report.permissions.high_risk_capabilities.is_empty()
    {
        score = score.saturating_sub(10);
    }

    score
}

fn score_to_grade(score: u8) -> String {
    match score {
        90..=100 => "A+".to_string(),
        80..=89 => "A".to_string(),
        70..=79 => "B".to_string(),
        60..=69 => "C".to_string(),
        50..=59 => "D".to_string(),
        _ => "F".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_score_to_grade() {
        assert_eq!(score_to_grade(95), "A+");
        assert_eq!(score_to_grade(85), "A");
        assert_eq!(score_to_grade(75), "B");
        assert_eq!(score_to_grade(65), "C");
        assert_eq!(score_to_grade(55), "D");
        assert_eq!(score_to_grade(40), "F");
    }

    #[test]
    fn test_format_permission_name() {
        assert_eq!(format_permission_name("filesystem.read"), "Filesystem Read");
        assert_eq!(format_permission_name("network.http"), "Network Http");
    }
}
