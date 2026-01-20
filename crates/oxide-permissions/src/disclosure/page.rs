//! Permission disclosure page generation.
//!
//! Generates structured data for a "Permissions & Privacy" page that
//! can be displayed in the app settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::capabilities::{
    Capability, CapabilityCategory, PermissionManifest, PermissionStatus, RiskLevel,
};
use crate::network::{NetworkMode, NetworkReport};

/// Complete disclosure page data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisclosurePage {
    /// App metadata.
    pub app: AppInfo,
    /// Permission sections.
    pub sections: Vec<PermissionSection>,
    /// Network policy section.
    pub network: Option<NetworkSection>,
    /// Privacy settings section.
    pub privacy: Option<PrivacySection>,
    /// Attestation/verification info.
    pub verification: Option<VerificationSection>,
}

/// App information for the disclosure page.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    /// App name.
    pub name: String,
    /// App version.
    pub version: String,
    /// Publisher/developer name.
    pub publisher: Option<String>,
    /// Whether the app is verified.
    pub is_verified: bool,
    /// App website.
    pub website: Option<String>,
    /// Privacy policy URL.
    pub privacy_policy_url: Option<String>,
}

/// A section of permissions grouped by category.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionSection {
    /// Category name.
    pub category: CapabilityCategory,
    /// Human-readable title.
    pub title: String,
    /// Section description.
    pub description: String,
    /// Risk level of this section.
    pub risk_level: RiskLevel,
    /// Individual permission items.
    pub items: Vec<PermissionItem>,
}

/// A single permission item.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionItem {
    /// Capability name.
    pub capability: String,
    /// Human-readable name.
    pub name: String,
    /// Description.
    pub description: String,
    /// Current status.
    pub status: PermissionStatus,
    /// Why the app needs this.
    pub reason: Option<String>,
    /// Whether it's granted.
    pub granted: bool,
    /// Icon for display.
    pub icon: String,
}

/// Network policy section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSection {
    /// Whether network is used.
    pub uses_network: bool,
    /// Whether allowlist is enforced.
    pub allowlist_enforced: bool,
    /// Network mode.
    pub mode: String,
    /// Allowed domains (if allowlist mode).
    pub allowed_domains: Vec<String>,
    /// Whether private ranges are blocked.
    pub blocks_private_ranges: bool,
    /// Whether HTTPS is required.
    pub requires_https: bool,
    /// Summary text.
    pub summary: String,
}

/// Privacy settings section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySection {
    /// Crash reporting status.
    pub crash_reporting: SettingStatus,
    /// Analytics status.
    pub analytics: SettingStatus,
    /// Telemetry status.
    pub telemetry: SettingStatus,
    /// Manual export status.
    pub manual_export: SettingStatus,
    /// Data retention info.
    pub data_retention: Option<String>,
}

/// Status of a privacy setting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettingStatus {
    /// Setting name.
    pub name: String,
    /// Whether enabled.
    pub enabled: bool,
    /// Whether user can control.
    pub user_controllable: bool,
    /// Description.
    pub description: String,
}

/// Verification/attestation section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationSection {
    /// Whether build is verified.
    pub verified_build: bool,
    /// Verification status text.
    pub status: String,
    /// Attestation URL if available.
    pub attestation_url: Option<String>,
    /// Last verification date.
    pub verified_date: Option<String>,
    /// Verification badges.
    pub badges: Vec<VerificationBadge>,
}

/// A verification badge.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationBadge {
    /// Badge name.
    pub name: String,
    /// Badge icon.
    pub icon: String,
    /// Whether earned.
    pub earned: bool,
    /// Description.
    pub description: String,
}

/// Builder for creating disclosure pages.
pub struct DisclosurePageBuilder {
    app: AppInfo,
    manifest: Option<PermissionManifest>,
    network_report: Option<NetworkReport>,
    granted_capabilities: HashMap<String, bool>,
}

impl DisclosurePageBuilder {
    /// Create a new disclosure page builder.
    pub fn new(app_name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            app: AppInfo {
                name: app_name.into(),
                version: version.into(),
                publisher: None,
                is_verified: false,
                website: None,
                privacy_policy_url: None,
            },
            manifest: None,
            network_report: None,
            granted_capabilities: HashMap::new(),
        }
    }

    /// Set the publisher.
    pub fn publisher(mut self, publisher: impl Into<String>) -> Self {
        self.app.publisher = Some(publisher.into());
        self
    }

    /// Set verified status.
    pub fn verified(mut self, verified: bool) -> Self {
        self.app.is_verified = verified;
        self
    }

    /// Set the permission manifest.
    pub fn with_manifest(mut self, manifest: PermissionManifest) -> Self {
        self.manifest = Some(manifest);
        self
    }

    /// Set the network report.
    pub fn with_network_report(mut self, report: NetworkReport) -> Self {
        self.network_report = Some(report);
        self
    }

    /// Set capability grant status.
    pub fn set_granted(mut self, capability: &str, granted: bool) -> Self {
        self.granted_capabilities.insert(capability.to_string(), granted);
        self
    }

    /// Build the disclosure page.
    pub fn build(self) -> DisclosurePage {
        let mut sections = Vec::new();

        if let Some(manifest) = &self.manifest {
            // Group capabilities by category
            let by_category = manifest.capabilities_by_category();

            for (category, capabilities) in by_category {
                let items: Vec<PermissionItem> = capabilities
                    .iter()
                    .map(|cap| {
                        let granted = self
                            .granted_capabilities
                            .get(cap.as_str())
                            .copied()
                            .unwrap_or(true);

                        let reason = manifest.capability_reason(cap.as_str()).map(String::from);

                        PermissionItem {
                            capability: cap.to_string(),
                            name: format_capability_display_name(cap.as_str()),
                            description: get_capability_description(cap.as_str()),
                            status: if granted {
                                PermissionStatus::Granted
                            } else {
                                PermissionStatus::Denied
                            },
                            reason,
                            granted,
                            icon: icon_for_capability(cap.as_str()),
                        }
                    })
                    .collect();

                if !items.is_empty() {
                    sections.push(PermissionSection {
                        category,
                        title: category.description().to_string(),
                        description: get_category_description(category),
                        risk_level: category.risk_level(),
                        items,
                    });
                }
            }
        }

        // Sort sections by risk level (highest first)
        sections.sort_by(|a, b| b.risk_level.cmp(&a.risk_level));

        let network = self.network_report.as_ref().map(|report| NetworkSection {
            uses_network: report.total_connections > 0 || !report.allowed_domains.is_empty(),
            allowlist_enforced: report.is_enforced && report.mode == NetworkMode::Allowlist,
            mode: format!("{:?}", report.mode),
            allowed_domains: report.allowed_domains.clone(),
            blocks_private_ranges: report.deny_private_ranges,
            requires_https: report.require_https,
            summary: report.summary(),
        });

        let privacy = self.manifest.as_ref().and_then(|m| {
            m.privacy.as_ref().map(|p| PrivacySection {
                crash_reporting: SettingStatus {
                    name: "Automatic Crash Reporting".to_string(),
                    enabled: p.auto_crash_reporting,
                    user_controllable: true,
                    description: "Automatically send crash reports".to_string(),
                },
                analytics: SettingStatus {
                    name: "Analytics".to_string(),
                    enabled: p.analytics_enabled,
                    user_controllable: true,
                    description: "Collect usage analytics".to_string(),
                },
                telemetry: SettingStatus {
                    name: "Telemetry".to_string(),
                    enabled: !p.telemetry_requires_consent,
                    user_controllable: p.telemetry_requires_consent,
                    description: "Send anonymous usage data".to_string(),
                },
                manual_export: SettingStatus {
                    name: "Manual Diagnostics Export".to_string(),
                    enabled: p.manual_export_allowed,
                    user_controllable: false,
                    description: "Allow manual export of diagnostic data".to_string(),
                },
                data_retention: if p.data_retention_days > 0 {
                    Some(format!("{} days", p.data_retention_days))
                } else {
                    Some("No data retained".to_string())
                },
            })
        });

        let verification = Some(self.build_verification_section());

        DisclosurePage {
            app: self.app,
            sections,
            network,
            privacy,
            verification,
        }
    }

    fn build_verification_section(&self) -> VerificationSection {
        let mut badges = Vec::new();

        // Network allowlist badge
        let network_badge = if let Some(report) = &self.network_report {
            VerificationBadge {
                name: "Network Allowlist".to_string(),
                icon: "shield".to_string(),
                earned: report.is_enforced && report.mode == NetworkMode::Allowlist,
                description: if report.is_enforced {
                    "Network connections restricted to declared domains".to_string()
                } else {
                    "Network connections unrestricted".to_string()
                },
            }
        } else {
            VerificationBadge {
                name: "Network Allowlist".to_string(),
                icon: "shield".to_string(),
                earned: false,
                description: "Network policy unknown".to_string(),
            }
        };
        badges.push(network_badge);

        // Verified build badge
        badges.push(VerificationBadge {
            name: "Verified Build".to_string(),
            icon: "check-circle".to_string(),
            earned: self.app.is_verified,
            description: if self.app.is_verified {
                "Build verified by attestation service".to_string()
            } else {
                "Build not verified".to_string()
            },
        });

        // Privacy badge
        let privacy_badge = if let Some(manifest) = &self.manifest {
            if let Some(privacy) = &manifest.privacy {
                VerificationBadge {
                    name: "Privacy Conscious".to_string(),
                    icon: "eye-off".to_string(),
                    earned: !privacy.auto_crash_reporting && !privacy.analytics_enabled,
                    description: "No automatic data collection".to_string(),
                }
            } else {
                VerificationBadge {
                    name: "Privacy Conscious".to_string(),
                    icon: "eye-off".to_string(),
                    earned: false,
                    description: "Privacy settings not declared".to_string(),
                }
            }
        } else {
            VerificationBadge {
                name: "Privacy Conscious".to_string(),
                icon: "eye-off".to_string(),
                earned: false,
                description: "No manifest available".to_string(),
            }
        };
        badges.push(privacy_badge);

        VerificationSection {
            verified_build: self.app.is_verified,
            status: if self.app.is_verified {
                "Verified".to_string()
            } else {
                "Unverified".to_string()
            },
            attestation_url: None,
            verified_date: None,
            badges,
        }
    }
}

fn format_capability_display_name(capability: &str) -> String {
    let registry = crate::capabilities::CapabilityRegistry::global();
    if let Some(registered) = registry.get(capability) {
        registered.name.clone()
    } else {
        capability
            .split('.')
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
}

fn get_capability_description(capability: &str) -> String {
    let registry = crate::capabilities::CapabilityRegistry::global();
    if let Some(registered) = registry.get(capability) {
        registered.description.clone()
    } else {
        format!("Access to {}", capability)
    }
}

fn icon_for_capability(capability: &str) -> String {
    let cap = Capability::new(capability);
    let category = CapabilityCategory::from_capability(&cap);

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

fn get_category_description(category: CapabilityCategory) -> String {
    match category {
        CapabilityCategory::Filesystem => {
            "Access to read and write files on your computer.".to_string()
        }
        CapabilityCategory::Keychain => {
            "Access to passwords and secure credentials stored on your system.".to_string()
        }
        CapabilityCategory::Network => {
            "Access to connect to the internet and external services.".to_string()
        }
        CapabilityCategory::Camera => "Access to capture photos and video.".to_string(),
        CapabilityCategory::Microphone => "Access to record audio.".to_string(),
        CapabilityCategory::Screenshot => "Access to capture screenshots.".to_string(),
        CapabilityCategory::Clipboard => {
            "Access to read and write clipboard contents.".to_string()
        }
        CapabilityCategory::Background => {
            "Ability to run tasks when the app is not in focus.".to_string()
        }
        CapabilityCategory::Notifications => {
            "Ability to display system notifications.".to_string()
        }
        CapabilityCategory::System => "Access to system information.".to_string(),
        CapabilityCategory::Location => "Access to your location.".to_string(),
        CapabilityCategory::Custom => "Custom application capabilities.".to_string(),
    }
}

impl DisclosurePage {
    /// Render as JSON for API responses.
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Generate a text summary.
    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== {} v{} ===", self.app.name, self.app.version));

        if let Some(publisher) = &self.app.publisher {
            lines.push(format!("Publisher: {}", publisher));
        }

        lines.push(format!(
            "Verified: {}",
            if self.app.is_verified { "Yes" } else { "No" }
        ));
        lines.push(String::new());

        // Permissions
        lines.push("PERMISSIONS".to_string());
        lines.push("-----------".to_string());

        for section in &self.sections {
            lines.push(format!("\n[{}]", section.title));
            for item in &section.items {
                let status = if item.granted { "+" } else { "-" };
                let reason = item
                    .reason
                    .as_ref()
                    .map(|r| format!(" - {}", r))
                    .unwrap_or_default();
                lines.push(format!(" {} {}{}", status, item.name, reason));
            }
        }

        // Network
        if let Some(network) = &self.network {
            lines.push(String::new());
            lines.push("NETWORK".to_string());
            lines.push("-------".to_string());
            lines.push(format!("Mode: {}", network.mode));
            if network.allowlist_enforced {
                lines.push(format!("Allowed domains: {}", network.allowed_domains.join(", ")));
            } else {
                lines.push("Domains: Unknown (not enforced)".to_string());
            }
        }

        // Privacy
        if let Some(privacy) = &self.privacy {
            lines.push(String::new());
            lines.push("PRIVACY".to_string());
            lines.push("-------".to_string());
            lines.push(format!(
                "Crash Reporting: {}",
                if privacy.crash_reporting.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            ));
            lines.push(format!(
                "Analytics: {}",
                if privacy.analytics.enabled {
                    "Enabled"
                } else {
                    "Disabled"
                }
            ));
        }

        // Verification
        if let Some(verification) = &self.verification {
            lines.push(String::new());
            lines.push("VERIFICATION".to_string());
            lines.push("------------".to_string());
            for badge in &verification.badges {
                let status = if badge.earned { "[x]" } else { "[ ]" };
                lines.push(format!("{} {}: {}", status, badge.name, badge.description));
            }
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disclosure_page_builder() {
        let manifest_str = r#"
[capabilities]
"filesystem.read" = { reason = "Save preferences" }
"network.http" = { reason = "API calls" }

[privacy]
auto_crash_reporting = false
analytics_enabled = false
"#;

        let manifest = PermissionManifest::from_str(manifest_str).unwrap();

        let page = DisclosurePageBuilder::new("TestApp", "1.0.0")
            .publisher("Test Publisher")
            .with_manifest(manifest)
            .build();

        assert_eq!(page.app.name, "TestApp");
        assert!(!page.sections.is_empty());
        assert!(page.privacy.is_some());
    }

    #[test]
    fn test_disclosure_page_rendering() {
        let page = DisclosurePageBuilder::new("TestApp", "1.0.0")
            .verified(true)
            .build();

        let text = page.render_text();
        assert!(text.contains("TestApp"));
        assert!(text.contains("Verified: Yes"));
    }
}
