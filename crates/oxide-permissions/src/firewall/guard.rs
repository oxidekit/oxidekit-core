//! Capability guards for protecting sensitive operations.
//!
//! Guards are used to wrap sensitive operations and ensure capability
//! checks are performed before allowing access.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::capabilities::Capability;
use crate::error::{PermissionError, PermissionResult};

use super::policy::{FirewallPolicy, PolicyDecision};

/// A guard that protects a capability-gated operation.
#[derive(Debug, Clone)]
pub struct CapabilityGuard {
    /// The capability being guarded.
    capability: Capability,
    /// Reference to the firewall policy.
    policy: Arc<FirewallPolicy>,
}

impl CapabilityGuard {
    /// Create a new capability guard.
    pub fn new(capability: impl Into<Capability>, policy: Arc<FirewallPolicy>) -> Self {
        Self {
            capability: capability.into(),
            policy,
        }
    }

    /// Check if the guarded capability is allowed.
    pub fn check(&self) -> PermissionResult<()> {
        match self.policy.check(self.capability.as_str()) {
            PolicyDecision::Allow | PolicyDecision::AuditOnly => Ok(()),
            PolicyDecision::Deny => Err(PermissionError::CapabilityDenied {
                capability: self.capability.to_string(),
            }),
            PolicyDecision::RequirePrompt => Err(PermissionError::ConsentRequired(
                self.capability.to_string(),
            )),
        }
    }

    /// Execute a closure if the capability is allowed.
    pub fn execute<F, T>(&self, f: F) -> PermissionResult<T>
    where
        F: FnOnce() -> T,
    {
        self.check()?;
        Ok(f())
    }

    /// Execute a fallible closure if the capability is allowed.
    pub fn try_execute<F, T, E>(&self, f: F) -> PermissionResult<Result<T, E>>
    where
        F: FnOnce() -> Result<T, E>,
    {
        self.check()?;
        Ok(f())
    }

    /// Get the capability being guarded.
    pub fn capability(&self) -> &Capability {
        &self.capability
    }
}

/// A scoped guard that logs entry and exit.
pub struct ScopedGuard {
    capability: Capability,
    entered: bool,
}

impl ScopedGuard {
    /// Enter a guarded scope.
    pub fn enter(capability: impl Into<Capability>, policy: &FirewallPolicy) -> PermissionResult<Self> {
        let capability = capability.into();

        match policy.check(capability.as_str()) {
            PolicyDecision::Allow | PolicyDecision::AuditOnly => {
                tracing::debug!(capability = %capability, "Entering guarded scope");
                Ok(Self {
                    capability,
                    entered: true,
                })
            }
            PolicyDecision::Deny => Err(PermissionError::CapabilityDenied {
                capability: capability.to_string(),
            }),
            PolicyDecision::RequirePrompt => Err(PermissionError::ConsentRequired(
                capability.to_string(),
            )),
        }
    }

    /// Check if the scope was successfully entered.
    pub fn is_entered(&self) -> bool {
        self.entered
    }

    /// Get the capability this scope guards.
    pub fn capability(&self) -> &Capability {
        &self.capability
    }
}

impl Drop for ScopedGuard {
    fn drop(&mut self) {
        if self.entered {
            tracing::debug!(capability = %self.capability, "Exiting guarded scope");
        }
    }
}

/// Async capability guard for async operations.
pub struct AsyncCapabilityGuard {
    capability: Capability,
    policy: Arc<FirewallPolicy>,
}

impl AsyncCapabilityGuard {
    /// Create a new async capability guard.
    pub fn new(capability: impl Into<Capability>, policy: Arc<FirewallPolicy>) -> Self {
        Self {
            capability: capability.into(),
            policy,
        }
    }

    /// Check if the guarded capability is allowed.
    pub fn check(&self) -> PermissionResult<()> {
        match self.policy.check(self.capability.as_str()) {
            PolicyDecision::Allow | PolicyDecision::AuditOnly => Ok(()),
            PolicyDecision::Deny => Err(PermissionError::CapabilityDenied {
                capability: self.capability.to_string(),
            }),
            PolicyDecision::RequirePrompt => Err(PermissionError::ConsentRequired(
                self.capability.to_string(),
            )),
        }
    }

    /// Execute an async operation if the capability is allowed.
    pub async fn execute<F, Fut, T>(&self, f: F) -> PermissionResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = T>,
    {
        self.check()?;
        Ok(f().await)
    }

    /// Execute a fallible async operation if the capability is allowed.
    pub async fn try_execute<F, Fut, T, E>(&self, f: F) -> PermissionResult<Result<T, E>>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
    {
        self.check()?;
        Ok(f().await)
    }
}

/// Type alias for boxed async closures.
pub type BoxedAsyncFn<T> = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = T> + Send>> + Send>;

/// A batch of capability checks.
#[derive(Debug)]
pub struct CapabilityBatch {
    capabilities: Vec<Capability>,
    policy: Arc<FirewallPolicy>,
}

impl CapabilityBatch {
    /// Create a new capability batch.
    pub fn new(policy: Arc<FirewallPolicy>) -> Self {
        Self {
            capabilities: Vec::new(),
            policy,
        }
    }

    /// Add a capability to check.
    pub fn require(mut self, capability: impl Into<Capability>) -> Self {
        self.capabilities.push(capability.into());
        self
    }

    /// Check all capabilities in the batch.
    pub fn check_all(&self) -> PermissionResult<()> {
        for cap in &self.capabilities {
            match self.policy.check(cap.as_str()) {
                PolicyDecision::Allow | PolicyDecision::AuditOnly => continue,
                PolicyDecision::Deny => {
                    return Err(PermissionError::CapabilityDenied {
                        capability: cap.to_string(),
                    });
                }
                PolicyDecision::RequirePrompt => {
                    return Err(PermissionError::ConsentRequired(cap.to_string()));
                }
            }
        }
        Ok(())
    }

    /// Get all denied capabilities.
    pub fn denied_capabilities(&self) -> Vec<&Capability> {
        self.capabilities
            .iter()
            .filter(|cap| {
                matches!(
                    self.policy.check(cap.as_str()),
                    PolicyDecision::Deny | PolicyDecision::RequirePrompt
                )
            })
            .collect()
    }
}

/// Macro for creating capability guards inline.
#[macro_export]
macro_rules! require_capability {
    ($policy:expr, $cap:expr) => {{
        let guard = $crate::firewall::CapabilityGuard::new($cap, ::std::sync::Arc::clone(&$policy));
        guard.check()
    }};
}

/// Macro for creating scoped capability guards.
#[macro_export]
macro_rules! guarded_scope {
    ($policy:expr, $cap:expr) => {
        $crate::firewall::ScopedGuard::enter($cap, &$policy)?
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_policy() -> Arc<FirewallPolicy> {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");
        policy.grant_capability("network.http");
        Arc::new(policy)
    }

    #[test]
    fn test_capability_guard_allowed() {
        let policy = test_policy();
        let guard = CapabilityGuard::new("filesystem.read", policy);

        assert!(guard.check().is_ok());

        let result = guard.execute(|| 42);
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_capability_guard_denied() {
        let policy = test_policy();
        let guard = CapabilityGuard::new("camera.capture", policy);

        let result = guard.check();
        assert!(result.is_err());
    }

    #[test]
    fn test_scoped_guard() {
        let mut policy = FirewallPolicy::new();
        policy.grant_capability("filesystem.read");

        let guard = ScopedGuard::enter("filesystem.read", &policy).unwrap();
        assert!(guard.is_entered());
        // Guard drops at end of scope
    }

    #[test]
    fn test_capability_batch() {
        let policy = test_policy();
        let batch = CapabilityBatch::new(Arc::clone(&policy))
            .require("filesystem.read")
            .require("network.http");

        assert!(batch.check_all().is_ok());

        let batch_with_denied = CapabilityBatch::new(policy)
            .require("filesystem.read")
            .require("camera.capture");

        let denied = batch_with_denied.denied_capabilities();
        assert_eq!(denied.len(), 1);
        assert_eq!(denied[0].as_str(), "camera.capture");
    }
}
