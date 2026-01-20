//! Capability Enforcer - The main runtime enforcement component.
//!
//! The enforcer is the central component that receives capability
//! check requests and returns enforcement decisions.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::capabilities::{Capability, CapabilityCategory, PermissionManifest, RiskLevel};
use crate::error::{PermissionError, PermissionResult};

use super::guard::{CapabilityBatch, CapabilityGuard};
use super::policy::{EnforcementMode, FirewallPolicy, PolicyDecision};

/// Statistics about capability checks.
#[derive(Debug, Clone, Default)]
pub struct EnforcementStats {
    /// Total number of checks performed.
    pub total_checks: u64,
    /// Number of allowed checks.
    pub allowed: u64,
    /// Number of denied checks.
    pub denied: u64,
    /// Number of checks requiring prompt.
    pub prompt_required: u64,
    /// Number of audit-only checks.
    pub audit_only: u64,
    /// Per-capability check counts.
    pub per_capability: HashMap<String, u64>,
}

/// Callback for capability check events.
pub type CheckCallback = Box<dyn Fn(&Capability, PolicyDecision) + Send + Sync>;

/// Callback for requesting user consent.
pub type ConsentCallback =
    Box<dyn Fn(&Capability) -> std::result::Result<bool, String> + Send + Sync>;

/// The capability enforcer handles all runtime permission enforcement.
pub struct CapabilityEnforcer {
    /// The firewall policy (mutable for runtime updates).
    policy: Arc<RwLock<FirewallPolicy>>,
    /// Enforcement statistics.
    stats: Arc<RwLock<EnforcementStats>>,
    /// Callback for check events (logging, monitoring).
    check_callback: Option<CheckCallback>,
    /// Callback for requesting user consent.
    consent_callback: Option<ConsentCallback>,
}

impl CapabilityEnforcer {
    /// Create a new capability enforcer with the given policy.
    pub fn new(policy: FirewallPolicy) -> Self {
        Self {
            policy: Arc::new(RwLock::new(policy)),
            stats: Arc::new(RwLock::new(EnforcementStats::default())),
            check_callback: None,
            consent_callback: None,
        }
    }

    /// Create an enforcer from a permission manifest.
    pub fn from_manifest(manifest: &PermissionManifest) -> Self {
        let policy = FirewallPolicy::from_manifest(manifest);
        Self::new(policy)
    }

    /// Create an enforcer with strict settings (no granted capabilities).
    pub fn strict() -> Self {
        Self::new(FirewallPolicy::strict())
    }

    /// Create an enforcer with permissive settings (log but allow).
    pub fn permissive() -> Self {
        Self::new(FirewallPolicy::permissive())
    }

    /// Set a callback for check events.
    pub fn on_check(mut self, callback: CheckCallback) -> Self {
        self.check_callback = Some(callback);
        self
    }

    /// Set a callback for consent requests.
    pub fn on_consent(mut self, callback: ConsentCallback) -> Self {
        self.consent_callback = Some(callback);
        self
    }

    /// Check a capability and enforce the policy.
    pub fn check(&self, capability: &str) -> PermissionResult<()> {
        let cap = Capability::new(capability);
        let decision = {
            let policy = self.policy.read().unwrap();
            policy.check(capability)
        };

        // Update statistics
        self.record_check(&cap, &decision);

        // Invoke callback if set
        if let Some(callback) = &self.check_callback {
            callback(&cap, decision.clone());
        }

        // Log the check
        match &decision {
            PolicyDecision::Allow => {
                tracing::trace!(capability = %cap, "Capability check: allowed");
            }
            PolicyDecision::Deny => {
                tracing::warn!(capability = %cap, "Capability check: denied");
            }
            PolicyDecision::RequirePrompt => {
                tracing::info!(capability = %cap, "Capability check: requires prompt");
            }
            PolicyDecision::AuditOnly => {
                tracing::debug!(capability = %cap, "Capability check: audit only");
            }
        }

        match decision {
            PolicyDecision::Allow | PolicyDecision::AuditOnly => Ok(()),
            PolicyDecision::Deny => Err(PermissionError::CapabilityDenied {
                capability: cap.to_string(),
            }),
            PolicyDecision::RequirePrompt => {
                // Try to get consent if callback is available
                if let Some(consent_callback) = &self.consent_callback {
                    match consent_callback(&cap) {
                        Ok(true) => {
                            // Record approval and allow
                            let mut policy = self.policy.write().unwrap();
                            policy.record_user_approval(capability);
                            Ok(())
                        }
                        Ok(false) => {
                            // Record denial and reject
                            let mut policy = self.policy.write().unwrap();
                            policy.record_user_denial(capability);
                            Err(PermissionError::CapabilityDenied {
                                capability: cap.to_string(),
                            })
                        }
                        Err(e) => Err(PermissionError::RuntimeEnforcementError(e)),
                    }
                } else {
                    // No consent callback, return error requiring prompt
                    Err(PermissionError::ConsentRequired(cap.to_string()))
                }
            }
        }
    }

    /// Check multiple capabilities at once.
    pub fn check_all(&self, capabilities: &[&str]) -> PermissionResult<()> {
        for cap in capabilities {
            self.check(cap)?;
        }
        Ok(())
    }

    /// Create a guard for a capability.
    pub fn guard(&self, capability: impl Into<Capability>) -> CapabilityGuard {
        let policy = self.policy.read().unwrap().clone();
        CapabilityGuard::new(capability, Arc::new(policy))
    }

    /// Create a batch of capability checks.
    pub fn batch(&self) -> CapabilityBatch {
        let policy = self.policy.read().unwrap().clone();
        CapabilityBatch::new(Arc::new(policy))
    }

    /// Grant a capability at runtime.
    pub fn grant(&self, capability: &str) {
        let mut policy = self.policy.write().unwrap();
        policy.grant_capability(capability);
        tracing::info!(capability, "Runtime capability granted");
    }

    /// Revoke a capability at runtime.
    pub fn revoke(&self, capability: &str) {
        let mut policy = self.policy.write().unwrap();
        policy.revoke_capability(capability);
        tracing::info!(capability, "Runtime capability revoked");
    }

    /// Get the current enforcement mode.
    pub fn mode(&self) -> EnforcementMode {
        self.policy.read().unwrap().mode
    }

    /// Set the enforcement mode.
    pub fn set_mode(&self, mode: EnforcementMode) {
        let mut policy = self.policy.write().unwrap();
        policy.set_mode(mode);
        tracing::info!(?mode, "Enforcement mode changed");
    }

    /// Check if a capability is currently granted.
    pub fn is_granted(&self, capability: &str) -> bool {
        self.policy.read().unwrap().is_granted(capability)
    }

    /// Get all granted capabilities.
    pub fn granted_capabilities(&self) -> Vec<String> {
        self.policy
            .read()
            .unwrap()
            .granted_capabilities()
            .iter()
            .cloned()
            .collect()
    }

    /// Get enforcement statistics.
    pub fn stats(&self) -> EnforcementStats {
        self.stats.read().unwrap().clone()
    }

    /// Reset enforcement statistics.
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = EnforcementStats::default();
    }

    /// Record a capability check in statistics.
    fn record_check(&self, capability: &Capability, decision: &PolicyDecision) {
        let mut stats = self.stats.write().unwrap();
        stats.total_checks += 1;

        match decision {
            PolicyDecision::Allow => stats.allowed += 1,
            PolicyDecision::Deny => stats.denied += 1,
            PolicyDecision::RequirePrompt => stats.prompt_required += 1,
            PolicyDecision::AuditOnly => stats.audit_only += 1,
        }

        *stats
            .per_capability
            .entry(capability.to_string())
            .or_insert(0) += 1;
    }

    /// Export the current policy configuration.
    pub fn export_policy(&self) -> FirewallPolicy {
        self.policy.read().unwrap().clone()
    }

    /// Import a policy configuration.
    pub fn import_policy(&self, policy: FirewallPolicy) {
        let mut current = self.policy.write().unwrap();
        *current = policy;
    }

    /// Get a summary of capability usage by category.
    pub fn usage_by_category(&self) -> HashMap<CapabilityCategory, u64> {
        let stats = self.stats.read().unwrap();
        let mut by_category: HashMap<CapabilityCategory, u64> = HashMap::new();

        for (cap_str, count) in &stats.per_capability {
            let cap = Capability::new(cap_str.clone());
            let category = CapabilityCategory::from_capability(&cap);
            *by_category.entry(category).or_insert(0) += count;
        }

        by_category
    }

    /// Get denied capabilities that were requested but not granted.
    pub fn denied_requests(&self) -> Vec<String> {
        let stats = self.stats.read().unwrap();
        let policy = self.policy.read().unwrap();

        stats
            .per_capability
            .keys()
            .filter(|cap| !policy.is_granted(cap))
            .cloned()
            .collect()
    }

    /// Generate a security report.
    pub fn security_report(&self) -> SecurityReport {
        let stats = self.stats.read().unwrap();
        let policy = self.policy.read().unwrap();

        let high_risk_granted: Vec<String> = policy
            .granted_capabilities()
            .iter()
            .filter(|cap| {
                let c = Capability::new(*cap);
                CapabilityCategory::from_capability(&c).risk_level() >= RiskLevel::High
            })
            .cloned()
            .collect();

        SecurityReport {
            enforcement_mode: policy.mode,
            total_capabilities_granted: policy.granted_capabilities().len(),
            high_risk_capabilities: high_risk_granted,
            total_checks: stats.total_checks,
            denied_checks: stats.denied,
            prompt_required_checks: stats.prompt_required,
        }
    }
}

impl Default for CapabilityEnforcer {
    fn default() -> Self {
        Self::new(FirewallPolicy::default())
    }
}

/// Security summary report.
#[derive(Debug, Clone)]
pub struct SecurityReport {
    /// Current enforcement mode.
    pub enforcement_mode: EnforcementMode,
    /// Total number of granted capabilities.
    pub total_capabilities_granted: usize,
    /// High-risk capabilities that are granted.
    pub high_risk_capabilities: Vec<String>,
    /// Total capability checks performed.
    pub total_checks: u64,
    /// Number of denied checks.
    pub denied_checks: u64,
    /// Number of checks requiring prompt.
    pub prompt_required_checks: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_enforcer_basic() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");

        let enforcer = CapabilityEnforcer::new(policy);

        assert!(enforcer.check("filesystem.read").is_ok());
        assert!(enforcer.check("filesystem.write").is_err());
    }

    #[test]
    fn test_enforcer_runtime_grant() {
        let enforcer = CapabilityEnforcer::strict();

        assert!(enforcer.check("filesystem.read").is_err());

        enforcer.grant("filesystem.read");
        assert!(enforcer.check("filesystem.read").is_ok());

        enforcer.revoke("filesystem.read");
        assert!(enforcer.check("filesystem.read").is_err());
    }

    #[test]
    fn test_enforcer_stats() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");
        policy.set_mode(EnforcementMode::Strict);

        let enforcer = CapabilityEnforcer::new(policy);

        let _ = enforcer.check("filesystem.read");
        let _ = enforcer.check("filesystem.write");
        let _ = enforcer.check("filesystem.read");

        let stats = enforcer.stats();
        assert_eq!(stats.total_checks, 3);
        assert_eq!(stats.allowed, 2);
        // filesystem.write was denied but requires prompt, not counted as denied
    }

    #[test]
    fn test_enforcer_from_manifest() {
        let manifest_str = r#"
[permissions]
"native.filesystem" = ["filesystem.read"]
"native.network" = ["network.http"]
"#;

        let manifest = PermissionManifest::from_str(manifest_str).unwrap();
        let enforcer = CapabilityEnforcer::from_manifest(&manifest);

        assert!(enforcer.is_granted("filesystem.read"));
        assert!(enforcer.is_granted("network.http"));
        assert!(!enforcer.is_granted("camera.capture"));
    }

    #[test]
    fn test_security_report() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");
        policy.grant_capability("keychain.access");

        let enforcer = CapabilityEnforcer::new(policy);
        let _ = enforcer.check("filesystem.read");
        let _ = enforcer.check("camera.capture");

        let report = enforcer.security_report();
        assert_eq!(report.total_capabilities_granted, 2);
        assert!(report.high_risk_capabilities.contains(&"filesystem.read".to_string()));
        assert!(report.high_risk_capabilities.contains(&"keychain.access".to_string()));
    }
}
