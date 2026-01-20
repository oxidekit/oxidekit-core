//! Marketplace registry integration.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::attestation::{AttestationReport, BadgeRegistry, TrustLevel};

use super::display::MarketplaceListing;

/// Marketplace registry entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryEntry {
    /// Application ID.
    pub app_id: String,
    /// Application name.
    pub name: String,
    /// Latest version.
    pub latest_version: String,
    /// Publisher ID.
    pub publisher_id: String,
    /// Trust level.
    pub trust_level: TrustLevel,
    /// Attestation ID (if verified).
    pub attestation_id: Option<String>,
    /// Badge IDs earned.
    pub badges: Vec<String>,
    /// Last updated timestamp.
    pub last_updated: String,
    /// Download count.
    pub downloads: u64,
}

/// Marketplace search filters.
#[derive(Debug, Clone, Default)]
pub struct SearchFilters {
    /// Minimum trust level.
    pub min_trust_level: Option<TrustLevel>,
    /// Required badges.
    pub required_badges: Vec<String>,
    /// Publisher ID filter.
    pub publisher_id: Option<String>,
    /// Category filter.
    pub category: Option<String>,
    /// Search query.
    pub query: Option<String>,
}

impl SearchFilters {
    /// Create new empty filters.
    pub fn new() -> Self {
        Self::default()
    }

    /// Require minimum trust level.
    pub fn min_trust(mut self, level: TrustLevel) -> Self {
        self.min_trust_level = Some(level);
        self
    }

    /// Require a specific badge.
    pub fn require_badge(mut self, badge: impl Into<String>) -> Self {
        self.required_badges.push(badge.into());
        self
    }

    /// Filter by publisher.
    pub fn by_publisher(mut self, publisher_id: impl Into<String>) -> Self {
        self.publisher_id = Some(publisher_id.into());
        self
    }

    /// Filter by category.
    pub fn in_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }

    /// Search by query.
    pub fn search(mut self, query: impl Into<String>) -> Self {
        self.query = Some(query.into());
        self
    }

    /// Check if an entry matches the filters.
    pub fn matches(&self, entry: &RegistryEntry) -> bool {
        // Check trust level
        if let Some(min_level) = &self.min_trust_level {
            if entry.trust_level < *min_level {
                return false;
            }
        }

        // Check required badges
        for badge in &self.required_badges {
            if !entry.badges.contains(badge) {
                return false;
            }
        }

        // Check publisher
        if let Some(publisher) = &self.publisher_id {
            if &entry.publisher_id != publisher {
                return false;
            }
        }

        // Check query
        if let Some(query) = &self.query {
            let query_lower = query.to_lowercase();
            if !entry.name.to_lowercase().contains(&query_lower)
                && !entry.app_id.to_lowercase().contains(&query_lower)
            {
                return false;
            }
        }

        true
    }
}

/// In-memory marketplace registry (for demonstration).
pub struct MarketplaceRegistry {
    entries: HashMap<String, RegistryEntry>,
    badge_registry: BadgeRegistry,
}

impl MarketplaceRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            badge_registry: BadgeRegistry::new(),
        }
    }

    /// Register an app from an attestation report.
    pub fn register_from_report(&mut self, report: &AttestationReport) -> RegistryEntry {
        let app_id = report
            .app
            .app_id
            .clone()
            .unwrap_or_else(|| format!("{}-{}", report.app.name.to_lowercase().replace(' ', "-"), report.app.version));

        let entry = RegistryEntry {
            app_id: app_id.clone(),
            name: report.app.name.clone(),
            latest_version: report.app.version.clone(),
            publisher_id: report.app.publisher.clone().unwrap_or_else(|| "unknown".to_string()),
            trust_level: report.trust_classification.level,
            attestation_id: Some(format!(
                "{}-{}",
                app_id,
                report.generated_at.timestamp()
            )),
            badges: report
                .badges
                .iter()
                .filter(|b| b.earned)
                .map(|b| b.id.clone())
                .collect(),
            last_updated: report.generated_at.to_rfc3339(),
            downloads: 0,
        };

        self.entries.insert(app_id, entry.clone());
        entry
    }

    /// Get an entry by app ID.
    pub fn get(&self, app_id: &str) -> Option<&RegistryEntry> {
        self.entries.get(app_id)
    }

    /// Search entries with filters.
    pub fn search(&self, filters: &SearchFilters) -> Vec<&RegistryEntry> {
        self.entries
            .values()
            .filter(|entry| filters.matches(entry))
            .collect()
    }

    /// Get all verified entries.
    pub fn verified_entries(&self) -> Vec<&RegistryEntry> {
        self.search(&SearchFilters::new().min_trust(TrustLevel::Verified))
    }

    /// Get entries with a specific badge.
    pub fn with_badge(&self, badge_id: &str) -> Vec<&RegistryEntry> {
        self.search(&SearchFilters::new().require_badge(badge_id))
    }

    /// Generate marketplace listing for an entry.
    pub fn get_listing(&self, app_id: &str, report: &AttestationReport) -> Option<MarketplaceListing> {
        self.entries.get(app_id)?;
        Some(MarketplaceListing::from_report(report, &self.badge_registry))
    }

    /// Get the badge registry.
    pub fn badge_registry(&self) -> &BadgeRegistry {
        &self.badge_registry
    }

    /// Get summary statistics.
    pub fn stats(&self) -> RegistryStats {
        let total = self.entries.len();
        let verified = self
            .entries
            .values()
            .filter(|e| e.trust_level >= TrustLevel::Verified)
            .count();

        let mut badge_counts: HashMap<String, usize> = HashMap::new();
        for entry in self.entries.values() {
            for badge in &entry.badges {
                *badge_counts.entry(badge.clone()).or_insert(0) += 1;
            }
        }

        RegistryStats {
            total_entries: total,
            verified_entries: verified,
            badge_distribution: badge_counts,
        }
    }
}

impl Default for MarketplaceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics.
#[derive(Debug, Clone)]
pub struct RegistryStats {
    /// Total number of entries.
    pub total_entries: usize,
    /// Number of verified entries.
    pub verified_entries: usize,
    /// Badge distribution.
    pub badge_distribution: HashMap<String, usize>,
}

/// Enterprise policy for marketplace usage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterprisePolicy {
    /// Minimum required trust level.
    pub min_trust_level: TrustLevel,
    /// Required badges for approval.
    pub required_badges: Vec<String>,
    /// Blocked publishers.
    pub blocked_publishers: Vec<String>,
    /// Maximum risk level allowed.
    pub max_risk_level: String,
    /// Require network allowlist.
    pub require_network_allowlist: bool,
    /// Require code signing.
    pub require_code_signing: bool,
}

impl Default for EnterprisePolicy {
    fn default() -> Self {
        Self {
            min_trust_level: TrustLevel::Verified,
            required_badges: vec!["network_allowlist".to_string()],
            blocked_publishers: Vec::new(),
            max_risk_level: "high".to_string(),
            require_network_allowlist: true,
            require_code_signing: true,
        }
    }
}

impl EnterprisePolicy {
    /// Create a strict policy.
    pub fn strict() -> Self {
        Self {
            min_trust_level: TrustLevel::Verified,
            required_badges: vec![
                "network_allowlist".to_string(),
                "verified_build".to_string(),
                "privacy_conscious".to_string(),
            ],
            blocked_publishers: Vec::new(),
            max_risk_level: "medium".to_string(),
            require_network_allowlist: true,
            require_code_signing: true,
        }
    }

    /// Create a permissive policy.
    pub fn permissive() -> Self {
        Self {
            min_trust_level: TrustLevel::Basic,
            required_badges: Vec::new(),
            blocked_publishers: Vec::new(),
            max_risk_level: "critical".to_string(),
            require_network_allowlist: false,
            require_code_signing: false,
        }
    }

    /// Check if an entry complies with the policy.
    pub fn check_compliance(&self, entry: &RegistryEntry) -> PolicyCompliance {
        let mut violations = Vec::new();

        // Check trust level
        if entry.trust_level < self.min_trust_level {
            violations.push(format!(
                "Trust level {} below minimum {}",
                entry.trust_level, self.min_trust_level
            ));
        }

        // Check required badges
        for badge in &self.required_badges {
            if !entry.badges.contains(badge) {
                violations.push(format!("Missing required badge: {}", badge));
            }
        }

        // Check blocked publishers
        if self.blocked_publishers.contains(&entry.publisher_id) {
            violations.push(format!("Publisher {} is blocked", entry.publisher_id));
        }

        // Check network allowlist requirement
        if self.require_network_allowlist && !entry.badges.contains(&"network_allowlist".to_string())
        {
            violations.push("Network allowlist not enforced".to_string());
        }

        // Check code signing requirement
        if self.require_code_signing && !entry.badges.contains(&"signed".to_string()) {
            violations.push("Code not signed".to_string());
        }

        PolicyCompliance {
            compliant: violations.is_empty(),
            violations,
        }
    }
}

/// Result of policy compliance check.
#[derive(Debug, Clone)]
pub struct PolicyCompliance {
    /// Whether the entry is compliant.
    pub compliant: bool,
    /// List of violations.
    pub violations: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_filters() {
        let entry = RegistryEntry {
            app_id: "test-app".to_string(),
            name: "Test App".to_string(),
            latest_version: "1.0.0".to_string(),
            publisher_id: "acme".to_string(),
            trust_level: TrustLevel::Verified,
            attestation_id: Some("test-123".to_string()),
            badges: vec!["network_allowlist".to_string()],
            last_updated: "2024-01-01".to_string(),
            downloads: 100,
        };

        // Matching filters
        let filters = SearchFilters::new()
            .min_trust(TrustLevel::Basic)
            .require_badge("network_allowlist");
        assert!(filters.matches(&entry));

        // Non-matching trust level
        let filters = SearchFilters::new().min_trust(TrustLevel::Official);
        assert!(!filters.matches(&entry));

        // Non-matching badge
        let filters = SearchFilters::new().require_badge("verified_build");
        assert!(!filters.matches(&entry));
    }

    #[test]
    fn test_enterprise_policy() {
        let policy = EnterprisePolicy::default();

        let compliant_entry = RegistryEntry {
            app_id: "good-app".to_string(),
            name: "Good App".to_string(),
            latest_version: "1.0.0".to_string(),
            publisher_id: "trusted".to_string(),
            trust_level: TrustLevel::Verified,
            attestation_id: Some("test-123".to_string()),
            badges: vec!["network_allowlist".to_string(), "signed".to_string()],
            last_updated: "2024-01-01".to_string(),
            downloads: 100,
        };

        let compliance = policy.check_compliance(&compliant_entry);
        assert!(compliance.compliant);

        let non_compliant_entry = RegistryEntry {
            app_id: "bad-app".to_string(),
            name: "Bad App".to_string(),
            latest_version: "1.0.0".to_string(),
            publisher_id: "unknown".to_string(),
            trust_level: TrustLevel::Unknown,
            attestation_id: None,
            badges: Vec::new(),
            last_updated: "2024-01-01".to_string(),
            downloads: 0,
        };

        let compliance = policy.check_compliance(&non_compliant_entry);
        assert!(!compliance.compliant);
        assert!(!compliance.violations.is_empty());
    }
}
