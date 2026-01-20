//! User consent management.
//!
//! Tracks user consent decisions and provides persistence.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};

use crate::capabilities::Capability;
use crate::error::{PermissionError, PermissionResult};

/// User consent record for a capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsentRecord {
    /// The capability.
    pub capability: String,
    /// Whether consent was granted.
    pub granted: bool,
    /// When consent was recorded.
    pub timestamp: DateTime<Utc>,
    /// How consent was obtained (prompt, auto, etc.).
    pub method: ConsentMethod,
    /// Optional reason provided by user.
    pub user_note: Option<String>,
    /// Whether consent can be revoked.
    pub revocable: bool,
}

/// How consent was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConsentMethod {
    /// User responded to a prompt.
    Prompt,
    /// Automatically granted based on manifest.
    Auto,
    /// Granted by system policy.
    SystemPolicy,
    /// User granted through settings.
    Settings,
    /// Inherited from parent capability.
    Inherited,
}

/// Consent store for persisting user decisions.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConsentStore {
    /// App identifier.
    pub app_id: String,
    /// Consent records by capability.
    #[serde(default)]
    pub records: HashMap<String, ConsentRecord>,
    /// Whether first-launch consent was completed.
    #[serde(default)]
    pub first_launch_completed: bool,
    /// When the store was last updated.
    pub last_updated: Option<DateTime<Utc>>,
}

impl ConsentStore {
    /// Create a new consent store for an app.
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            records: HashMap::new(),
            first_launch_completed: false,
            last_updated: None,
        }
    }

    /// Load consent store from a file.
    pub fn load<P: AsRef<Path>>(path: P) -> PermissionResult<Self> {
        let content = std::fs::read_to_string(path.as_ref())?;
        serde_json::from_str(&content).map_err(|e| PermissionError::SerializationError(e.to_string()))
    }

    /// Save consent store to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> PermissionResult<()> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| PermissionError::SerializationError(e.to_string()))?;
        std::fs::write(path.as_ref(), content)?;
        Ok(())
    }

    /// Record a consent decision.
    pub fn record(
        &mut self,
        capability: impl Into<String>,
        granted: bool,
        method: ConsentMethod,
    ) -> &ConsentRecord {
        let capability = capability.into();
        let record = ConsentRecord {
            capability: capability.clone(),
            granted,
            timestamp: Utc::now(),
            method,
            user_note: None,
            revocable: method != ConsentMethod::SystemPolicy,
        };

        self.records.insert(capability.clone(), record);
        self.last_updated = Some(Utc::now());

        self.records.get(&capability).unwrap()
    }

    /// Record consent with a user note.
    pub fn record_with_note(
        &mut self,
        capability: impl Into<String>,
        granted: bool,
        method: ConsentMethod,
        note: impl Into<String>,
    ) -> &ConsentRecord {
        let capability = capability.into();
        let record = ConsentRecord {
            capability: capability.clone(),
            granted,
            timestamp: Utc::now(),
            method,
            user_note: Some(note.into()),
            revocable: method != ConsentMethod::SystemPolicy,
        };

        self.records.insert(capability.clone(), record);
        self.last_updated = Some(Utc::now());

        self.records.get(&capability).unwrap()
    }

    /// Get consent status for a capability.
    pub fn get(&self, capability: &str) -> Option<&ConsentRecord> {
        self.records.get(capability)
    }

    /// Check if consent is granted for a capability.
    pub fn is_granted(&self, capability: &str) -> Option<bool> {
        self.records.get(capability).map(|r| r.granted)
    }

    /// Check if consent is explicitly denied.
    pub fn is_denied(&self, capability: &str) -> bool {
        self.records
            .get(capability)
            .is_some_and(|r| !r.granted)
    }

    /// Revoke consent for a capability.
    pub fn revoke(&mut self, capability: &str) -> bool {
        if let Some(record) = self.records.get_mut(capability) {
            if record.revocable {
                record.granted = false;
                record.timestamp = Utc::now();
                record.method = ConsentMethod::Settings;
                self.last_updated = Some(Utc::now());
                return true;
            }
        }
        false
    }

    /// Remove a consent record.
    pub fn remove(&mut self, capability: &str) -> Option<ConsentRecord> {
        let record = self.records.remove(capability);
        if record.is_some() {
            self.last_updated = Some(Utc::now());
        }
        record
    }

    /// Mark first-launch consent as completed.
    pub fn complete_first_launch(&mut self) {
        self.first_launch_completed = true;
        self.last_updated = Some(Utc::now());
    }

    /// Check if a capability needs consent.
    pub fn needs_consent(&self, capability: &str) -> bool {
        !self.records.contains_key(capability)
    }

    /// Get all granted capabilities.
    pub fn granted_capabilities(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, r)| r.granted)
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get all denied capabilities.
    pub fn denied_capabilities(&self) -> Vec<&str> {
        self.records
            .iter()
            .filter(|(_, r)| !r.granted)
            .map(|(k, _)| k.as_str())
            .collect()
    }

    /// Get capabilities pending consent.
    pub fn pending_capabilities<'a>(&self, required: &'a [Capability]) -> Vec<&'a Capability> {
        required
            .iter()
            .filter(|cap| self.needs_consent(cap.as_str()))
            .collect()
    }

    /// Record batch consent (e.g., from first-launch dialog).
    pub fn record_batch(&mut self, decisions: &[(String, bool)], method: ConsentMethod) {
        for (capability, granted) in decisions {
            self.record(capability.clone(), *granted, method);
        }
    }

    /// Clear all records (for testing or reset).
    pub fn clear(&mut self) {
        self.records.clear();
        self.first_launch_completed = false;
        self.last_updated = Some(Utc::now());
    }

    /// Export consent data for backup.
    pub fn export(&self) -> PermissionResult<String> {
        serde_json::to_string_pretty(self)
            .map_err(|e| PermissionError::SerializationError(e.to_string()))
    }

    /// Generate a consent summary.
    pub fn summary(&self) -> ConsentSummary {
        let total = self.records.len();
        let granted = self.records.values().filter(|r| r.granted).count();
        let denied = total - granted;

        let by_method: HashMap<ConsentMethod, usize> = self.records.values().fold(
            HashMap::new(),
            |mut acc, r| {
                *acc.entry(r.method).or_insert(0) += 1;
                acc
            },
        );

        ConsentSummary {
            total,
            granted,
            denied,
            by_method,
            first_launch_completed: self.first_launch_completed,
            last_updated: self.last_updated,
        }
    }
}

/// Summary of consent status.
#[derive(Debug, Clone)]
pub struct ConsentSummary {
    /// Total capabilities with consent records.
    pub total: usize,
    /// Number of granted.
    pub granted: usize,
    /// Number of denied.
    pub denied: usize,
    /// Breakdown by method.
    pub by_method: HashMap<ConsentMethod, usize>,
    /// First launch completed.
    pub first_launch_completed: bool,
    /// Last update time.
    pub last_updated: Option<DateTime<Utc>>,
}

/// First-launch consent flow.
pub struct FirstLaunchConsent {
    /// App identifier.
    app_id: String,
    /// Capabilities requiring consent.
    capabilities: Vec<Capability>,
    /// User decisions.
    decisions: Vec<(String, bool)>,
}

impl FirstLaunchConsent {
    /// Create a new first-launch consent flow.
    pub fn new(app_id: impl Into<String>) -> Self {
        Self {
            app_id: app_id.into(),
            capabilities: Vec::new(),
            decisions: Vec::new(),
        }
    }

    /// Add a capability requiring consent.
    pub fn require(mut self, capability: impl Into<Capability>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Add multiple capabilities.
    pub fn require_all(mut self, capabilities: Vec<Capability>) -> Self {
        self.capabilities.extend(capabilities);
        self
    }

    /// Get capabilities requiring decisions.
    pub fn pending_capabilities(&self) -> &[Capability] {
        &self.capabilities
    }

    /// Record a decision.
    pub fn decide(&mut self, capability: &str, granted: bool) {
        self.decisions.push((capability.to_string(), granted));
    }

    /// Record decisions for all capabilities at once (accept all).
    pub fn accept_all(&mut self) {
        for cap in &self.capabilities {
            self.decisions.push((cap.to_string(), true));
        }
    }

    /// Record decisions to deny all.
    pub fn deny_all(&mut self) {
        for cap in &self.capabilities {
            self.decisions.push((cap.to_string(), false));
        }
    }

    /// Apply decisions to a consent store.
    pub fn apply_to(&self, store: &mut ConsentStore) {
        store.record_batch(&self.decisions, ConsentMethod::Prompt);
        store.complete_first_launch();
    }

    /// Check if all capabilities have decisions.
    pub fn is_complete(&self) -> bool {
        let decided: std::collections::HashSet<_> =
            self.decisions.iter().map(|(c, _)| c.as_str()).collect();
        self.capabilities
            .iter()
            .all(|c| decided.contains(c.as_str()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_consent_store() {
        let mut store = ConsentStore::new("test-app");

        store.record("filesystem.read", true, ConsentMethod::Prompt);
        store.record("camera.capture", false, ConsentMethod::Prompt);

        assert!(store.is_granted("filesystem.read").unwrap());
        assert!(!store.is_granted("camera.capture").unwrap());
        assert!(store.needs_consent("network.http"));
    }

    #[test]
    fn test_consent_revocation() {
        let mut store = ConsentStore::new("test-app");

        store.record("filesystem.read", true, ConsentMethod::Prompt);
        assert!(store.is_granted("filesystem.read").unwrap());

        assert!(store.revoke("filesystem.read"));
        assert!(!store.is_granted("filesystem.read").unwrap());
    }

    #[test]
    fn test_system_policy_not_revocable() {
        let mut store = ConsentStore::new("test-app");

        store.record("system.required", true, ConsentMethod::SystemPolicy);
        assert!(!store.revoke("system.required"));
        assert!(store.is_granted("system.required").unwrap());
    }

    #[test]
    fn test_first_launch_consent() {
        let mut consent = FirstLaunchConsent::new("test-app")
            .require("filesystem.read")
            .require("network.http");

        assert!(!consent.is_complete());

        consent.decide("filesystem.read", true);
        consent.decide("network.http", true);

        assert!(consent.is_complete());

        let mut store = ConsentStore::new("test-app");
        consent.apply_to(&mut store);

        assert!(store.first_launch_completed);
        assert!(store.is_granted("filesystem.read").unwrap());
    }

    #[test]
    fn test_consent_summary() {
        let mut store = ConsentStore::new("test-app");

        store.record("a", true, ConsentMethod::Prompt);
        store.record("b", true, ConsentMethod::Prompt);
        store.record("c", false, ConsentMethod::Prompt);

        let summary = store.summary();
        assert_eq!(summary.total, 3);
        assert_eq!(summary.granted, 2);
        assert_eq!(summary.denied, 1);
    }
}
