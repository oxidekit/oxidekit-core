//! # OxideKit Permissions & Attestation System
//!
//! A desktop-grade permissions and transparency model for OxideKit applications,
//! providing mobile-like permission disclosures with real enforcement.
//!
//! ## Overview
//!
//! This crate provides:
//!
//! - **Capability Definitions**: A canonical permission model with hierarchical capabilities
//! - **Permission Manifest**: Schema for declaring permissions in `oxide.toml`
//! - **Capability Firewall**: Runtime enforcement of declared permissions
//! - **Network Allowlist**: Enforce outbound network policy at runtime
//! - **Permission Disclosure**: UI components for displaying permissions to users
//! - **Verified Build Profile**: Policy-driven build hardening
//! - **Attestation Service**: Binary scanning and verification
//! - **Marketplace Badging**: Trust indicators for the registry
//!
//! ## Quick Start
//!
//! ### Declaring Permissions
//!
//! In your `oxide.toml`:
//!
//! ```toml
//! [permissions]
//! "native.filesystem" = ["filesystem.read"]
//! "native.network" = ["network.http"]
//!
//! [capabilities]
//! "filesystem.read" = { reason = "Save user preferences" }
//! "network.http" = { reason = "Fetch API data", domains = ["api.example.com"] }
//!
//! [network]
//! mode = "allowlist"
//! allow = ["api.example.com", "cdn.example.com"]
//! deny_private_ranges = true
//!
//! [privacy]
//! auto_crash_reporting = false
//! analytics_enabled = false
//! ```
//!
//! ### Using the Capability Firewall
//!
//! ```rust,ignore
//! use oxide_permissions::firewall::{CapabilityEnforcer, FirewallPolicy};
//! use oxide_permissions::capabilities::PermissionManifest;
//!
//! // Load manifest and create enforcer
//! let manifest = PermissionManifest::from_file("oxide.toml")?;
//! let enforcer = CapabilityEnforcer::from_manifest(&manifest);
//!
//! // Check capability before sensitive operation
//! enforcer.check("filesystem.read")?;
//!
//! // Or use a guard
//! let guard = enforcer.guard("network.http");
//! guard.execute(|| {
//!     // Make HTTP request
//! })?;
//! ```
//!
//! ### Network Allowlist Enforcement
//!
//! ```rust,ignore
//! use oxide_permissions::network::{NetworkEnforcer, NetworkPolicy, ConnectionRequest};
//!
//! // Create enforcer from policy
//! let policy = NetworkPolicy::strict_allowlist(vec![
//!     "api.example.com".to_string(),
//! ]);
//! let enforcer = NetworkEnforcer::new(policy);
//!
//! // Check connection before making request
//! let request = ConnectionRequest::new("api.example.com", 443, "https");
//! match enforcer.check(&request) {
//!     NetworkDecision::Allow => { /* proceed */ }
//!     NetworkDecision::Deny(reason) => { /* handle denial */ }
//!     NetworkDecision::Unknown => { /* policy not enforced */ }
//! }
//! ```
//!
//! ### Generating Attestation Reports
//!
//! ```rust,ignore
//! use oxide_permissions::attestation::{AttestationService, BinaryScanner};
//!
//! let service = AttestationService::default_service();
//!
//! // Upload binary for attestation
//! let result = service.upload("./dist/MyApp.exe")?;
//!
//! println!("Attestation ID: {}", result.attestation_id);
//! println!("Trust Level: {}", result.trust_level());
//! println!("Report: {}", result.report.summary());
//! ```
//!
//! ## Module Overview
//!
//! - [`capabilities`]: Core capability and permission types
//! - [`firewall`]: Runtime capability enforcement
//! - [`network`]: Network allowlist enforcement
//! - [`disclosure`]: Permission disclosure UI components
//! - [`verified_build`]: Verified build profile and checking
//! - [`attestation`]: Binary attestation service
//! - [`marketplace`]: Marketplace integration and badging
//!
//! ## Feature Flags
//!
//! - `async-runtime`: Enable async support with Tokio
//! - `network-enforcement`: Enable full network allowlist enforcement
//! - `attestation`: Enable attestation service with cryptographic verification
//! - `disclosure-ui`: Enable disclosure UI components
//! - `full`: Enable all features

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod capabilities;
pub mod disclosure;
pub mod error;
pub mod firewall;
pub mod marketplace;
pub mod network;

#[cfg(feature = "attestation")]
pub mod attestation;

#[cfg(not(feature = "attestation"))]
pub mod attestation;

pub mod verified_build;

// Re-exports for convenient access
pub use capabilities::{
    Capability, CapabilityCategory, CapabilityRegistry, PermissionManifest, PermissionStatus,
    RiskLevel,
};
pub use disclosure::{ConsentStore, DisclosurePage, DisclosurePageBuilder, PermissionPrompt, PromptBuilder};
pub use error::{PermissionError, PermissionResult};
pub use firewall::{CapabilityEnforcer, CapabilityGuard, EnforcementMode, FirewallPolicy};
pub use network::{
    ConnectionRequest, NetworkDecision, NetworkDenyReason, NetworkEnforcer, NetworkMode,
    NetworkPolicy,
};

pub use attestation::{
    AttestationReport, AttestationService, AttestationStatus, BadgeRegistry, BinaryScanner,
    TrustLevel,
};

pub use marketplace::{EnterprisePolicy, MarketplaceListing, MarketplaceRegistry, SearchFilters};
pub use verified_build::{
    BuildMetadata, VerifiedBuildChecker, VerifiedBuildProfile, VerifiedBuildReport,
};

/// Version of the permissions system.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check if a capability is a known OxideKit capability.
pub fn is_known_capability(capability: &str) -> bool {
    CapabilityRegistry::global().is_registered(capability)
}

/// Get information about a capability.
pub fn capability_info(capability: &str) -> Option<&'static capabilities::RegisteredCapability> {
    CapabilityRegistry::global().get(capability)
}

/// Create a default capability enforcer (strict mode).
pub fn default_enforcer() -> CapabilityEnforcer {
    CapabilityEnforcer::strict()
}

/// Create an enforcer from a manifest file.
pub fn enforcer_from_file(
    manifest_path: impl AsRef<std::path::Path>,
) -> PermissionResult<CapabilityEnforcer> {
    let manifest = PermissionManifest::from_file(manifest_path)?;
    Ok(CapabilityEnforcer::from_manifest(&manifest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_known_capability() {
        assert!(is_known_capability("filesystem.read"));
        assert!(is_known_capability("network.http"));
        assert!(!is_known_capability("nonexistent.capability"));
    }

    #[test]
    fn test_capability_info() {
        let info = capability_info("filesystem.read");
        assert!(info.is_some());

        let info = info.unwrap();
        assert_eq!(info.capability.as_str(), "filesystem.read");
    }

    #[test]
    fn test_default_enforcer() {
        let enforcer = default_enforcer();
        assert_eq!(enforcer.mode(), EnforcementMode::Strict);
    }

    #[test]
    fn test_integration() {
        // Create a manifest
        let manifest_str = r#"
[permissions]
"native.filesystem" = ["filesystem.read"]
"native.network" = ["network.http"]

[capabilities]
"filesystem.read" = { reason = "Save preferences" }
"network.http" = { reason = "API calls" }

[network]
mode = "allowlist"
allow = ["api.example.com"]
deny_private_ranges = true

[privacy]
auto_crash_reporting = false
analytics_enabled = false
"#;

        let manifest = PermissionManifest::from_str(manifest_str).unwrap();

        // Create enforcer
        let enforcer = CapabilityEnforcer::from_manifest(&manifest);

        // Check capabilities
        assert!(enforcer.is_granted("filesystem.read"));
        assert!(enforcer.is_granted("network.http"));
        assert!(!enforcer.is_granted("camera.capture"));

        // Check network policy
        let network_enforcer = NetworkEnforcer::new(manifest.network.clone().unwrap())
            .with_dns_resolution(false);

        let allowed = ConnectionRequest::new("api.example.com", 443, "https");
        assert_eq!(network_enforcer.check(&allowed), NetworkDecision::Allow);

        let denied = ConnectionRequest::new("evil.com", 443, "https");
        assert!(matches!(
            network_enforcer.check(&denied),
            NetworkDecision::Deny(_)
        ));
    }
}
