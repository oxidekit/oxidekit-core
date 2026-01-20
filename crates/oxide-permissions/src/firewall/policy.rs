//! Firewall policy configuration and management.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::capabilities::{Capability, PermissionManifest};

/// Firewall enforcement mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnforcementMode {
    /// Strict mode - all capability checks are enforced, violations are errors.
    #[default]
    Strict,
    /// Permissive mode - capability checks are enforced, violations are logged but allowed.
    Permissive,
    /// Audit mode - no enforcement, only logging.
    Audit,
    /// Disabled - no checking at all.
    Disabled,
}

/// Configuration for the capability firewall.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FirewallPolicy {
    /// Enforcement mode.
    #[serde(default)]
    pub mode: EnforcementMode,

    /// Set of granted capabilities.
    #[serde(default)]
    pub granted_capabilities: HashSet<String>,

    /// Whether to require explicit declaration for all capabilities.
    #[serde(default = "default_true")]
    pub require_declaration: bool,

    /// Whether to log all capability checks.
    #[serde(default)]
    pub log_all_checks: bool,

    /// Whether to allow runtime capability elevation (user prompts).
    #[serde(default = "default_true")]
    pub allow_runtime_elevation: bool,

    /// Capabilities that require user prompt before first use.
    #[serde(default)]
    pub prompt_required: HashSet<String>,

    /// Capabilities that have been user-approved at runtime.
    #[serde(default)]
    pub user_approved: HashSet<String>,

    /// Capabilities that have been user-denied at runtime.
    #[serde(default)]
    pub user_denied: HashSet<String>,
}

fn default_true() -> bool {
    true
}

impl Default for FirewallPolicy {
    fn default() -> Self {
        Self {
            mode: EnforcementMode::Strict,
            granted_capabilities: HashSet::new(),
            require_declaration: true,
            log_all_checks: false,
            allow_runtime_elevation: true,
            prompt_required: HashSet::new(),
            user_approved: HashSet::new(),
            user_denied: HashSet::new(),
        }
    }
}

impl FirewallPolicy {
    /// Create a new firewall policy with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a policy from a permission manifest.
    pub fn from_manifest(manifest: &PermissionManifest) -> Self {
        let mut policy = Self::new();

        // Grant all capabilities declared in the manifest
        for cap in manifest.all_capabilities() {
            policy.grant_capability(cap.as_str());
        }

        policy
    }

    /// Create a strict policy with no granted capabilities.
    pub fn strict() -> Self {
        Self {
            mode: EnforcementMode::Strict,
            allow_runtime_elevation: false,
            ..Default::default()
        }
    }

    /// Create a permissive policy that logs but doesn't block.
    pub fn permissive() -> Self {
        Self {
            mode: EnforcementMode::Permissive,
            ..Default::default()
        }
    }

    /// Create an audit-only policy.
    pub fn audit() -> Self {
        Self {
            mode: EnforcementMode::Audit,
            log_all_checks: true,
            ..Default::default()
        }
    }

    /// Grant a capability.
    pub fn grant_capability(&mut self, capability: &str) {
        self.granted_capabilities.insert(capability.to_string());
    }

    /// Revoke a capability.
    pub fn revoke_capability(&mut self, capability: &str) {
        self.granted_capabilities.remove(capability);
    }

    /// Check if a capability is granted.
    pub fn is_granted(&self, capability: &str) -> bool {
        // Check if explicitly denied by user
        if self.user_denied.contains(capability) {
            return false;
        }

        // Check if explicitly approved by user
        if self.user_approved.contains(capability) {
            return true;
        }

        // Check direct grant
        if self.granted_capabilities.contains(capability) {
            return true;
        }

        // Check parent capabilities (hierarchical matching)
        let cap = Capability::new(capability);
        for granted in &self.granted_capabilities {
            let granted_cap = Capability::new(granted.clone());
            if granted_cap.matches(&cap) {
                return true;
            }
        }

        false
    }

    /// Check if a capability requires a user prompt.
    pub fn requires_prompt(&self, capability: &str) -> bool {
        // Already approved or denied
        if self.user_approved.contains(capability) || self.user_denied.contains(capability) {
            return false;
        }

        // In the prompt required set
        if self.prompt_required.contains(capability) {
            return true;
        }

        // Check if capability is granted but runtime elevation is needed
        !self.is_granted(capability) && self.allow_runtime_elevation
    }

    /// Record user approval for a capability.
    pub fn record_user_approval(&mut self, capability: &str) {
        self.user_approved.insert(capability.to_string());
        self.user_denied.remove(capability);
    }

    /// Record user denial for a capability.
    pub fn record_user_denial(&mut self, capability: &str) {
        self.user_denied.insert(capability.to_string());
        self.user_approved.remove(capability);
    }

    /// Clear all user decisions (for testing or reset).
    pub fn clear_user_decisions(&mut self) {
        self.user_approved.clear();
        self.user_denied.clear();
    }

    /// Set the enforcement mode.
    pub fn set_mode(&mut self, mode: EnforcementMode) {
        self.mode = mode;
    }

    /// Mark a capability as requiring prompt.
    pub fn require_prompt_for(&mut self, capability: &str) {
        self.prompt_required.insert(capability.to_string());
    }

    /// Get all granted capabilities.
    pub fn granted_capabilities(&self) -> &HashSet<String> {
        &self.granted_capabilities
    }

    /// Get all capabilities that require prompts.
    pub fn capabilities_requiring_prompt(&self) -> &HashSet<String> {
        &self.prompt_required
    }
}

/// Result of a policy check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PolicyDecision {
    /// Capability is allowed.
    Allow,
    /// Capability is denied.
    Deny,
    /// Capability requires user prompt.
    RequirePrompt,
    /// Check was logged but not enforced (audit mode).
    AuditOnly,
}

impl FirewallPolicy {
    /// Check a capability and return a policy decision.
    pub fn check(&self, capability: &str) -> PolicyDecision {
        match self.mode {
            EnforcementMode::Disabled => PolicyDecision::Allow,
            EnforcementMode::Audit => {
                // Just log, always allow
                PolicyDecision::AuditOnly
            }
            EnforcementMode::Permissive | EnforcementMode::Strict => {
                if self.user_denied.contains(capability) {
                    PolicyDecision::Deny
                } else if self.is_granted(capability) {
                    PolicyDecision::Allow
                } else if self.requires_prompt(capability) {
                    PolicyDecision::RequirePrompt
                } else if self.mode == EnforcementMode::Permissive {
                    PolicyDecision::AuditOnly
                } else {
                    PolicyDecision::Deny
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_policy_grant_and_check() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");

        assert!(policy.is_granted("filesystem.read"));
        assert!(!policy.is_granted("filesystem.write"));
    }

    #[test]
    fn test_hierarchical_matching() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem");

        // Parent capability should match children
        assert!(policy.is_granted("filesystem"));
        assert!(policy.is_granted("filesystem.read"));
        assert!(policy.is_granted("filesystem.write"));
    }

    #[test]
    fn test_user_decisions() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("camera.capture");

        assert!(policy.is_granted("camera.capture"));

        policy.record_user_denial("camera.capture");
        assert!(!policy.is_granted("camera.capture"));

        policy.record_user_approval("camera.capture");
        assert!(policy.is_granted("camera.capture"));
    }

    #[test]
    fn test_policy_decisions() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");

        assert_eq!(policy.check("filesystem.read"), PolicyDecision::Allow);
        assert_eq!(policy.check("filesystem.write"), PolicyDecision::RequirePrompt);

        policy.set_mode(EnforcementMode::Audit);
        assert_eq!(policy.check("filesystem.write"), PolicyDecision::AuditOnly);
    }

    #[test]
    fn test_strict_policy() {
        let mut policy = FirewallPolicy::strict();
        assert_eq!(policy.check("filesystem.read"), PolicyDecision::Deny);

        policy.grant_capability("filesystem.read");
        assert_eq!(policy.check("filesystem.read"), PolicyDecision::Allow);
    }
}
