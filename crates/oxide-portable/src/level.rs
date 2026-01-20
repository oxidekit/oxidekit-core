//! Portability levels and classifications for APIs.
//!
//! Defines how APIs are categorized based on their cross-platform compatibility.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// The portability level of an API or feature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PortabilityLevel {
    /// Fully portable - works on all targets (desktop, web, mobile)
    Portable,
    /// Desktop only - requires desktop OS features
    DesktopOnly,
    /// Web only - requires browser/WASM environment
    WebOnly,
    /// Mobile only - requires iOS or Android
    MobileOnly,
    /// iOS only - requires iOS specifically
    IosOnly,
    /// Android only - requires Android specifically
    AndroidOnly,
    /// macOS only - requires macOS specifically
    MacosOnly,
    /// Windows only - requires Windows specifically
    WindowsOnly,
    /// Linux only - requires Linux specifically
    LinuxOnly,
    /// Native only - desktop + mobile, but not web
    NativeOnly,
    /// Experimental - may not work on all targets
    Experimental,
}

impl PortabilityLevel {
    /// Check if this level indicates a portable API.
    pub fn is_portable(&self) -> bool {
        matches!(self, PortabilityLevel::Portable)
    }

    /// Check if this API works on desktop targets.
    pub fn works_on_desktop(&self) -> bool {
        matches!(
            self,
            PortabilityLevel::Portable
                | PortabilityLevel::DesktopOnly
                | PortabilityLevel::MacosOnly
                | PortabilityLevel::WindowsOnly
                | PortabilityLevel::LinuxOnly
                | PortabilityLevel::NativeOnly
                | PortabilityLevel::Experimental
        )
    }

    /// Check if this API works on web targets.
    pub fn works_on_web(&self) -> bool {
        matches!(
            self,
            PortabilityLevel::Portable
                | PortabilityLevel::WebOnly
                | PortabilityLevel::Experimental
        )
    }

    /// Check if this API works on mobile targets.
    pub fn works_on_mobile(&self) -> bool {
        matches!(
            self,
            PortabilityLevel::Portable
                | PortabilityLevel::MobileOnly
                | PortabilityLevel::IosOnly
                | PortabilityLevel::AndroidOnly
                | PortabilityLevel::NativeOnly
                | PortabilityLevel::Experimental
        )
    }

    /// Get a human-readable description of this level.
    pub fn description(&self) -> &'static str {
        match self {
            PortabilityLevel::Portable => "Works on all platforms (desktop, web, mobile)",
            PortabilityLevel::DesktopOnly => "Only available on desktop platforms (macOS, Windows, Linux)",
            PortabilityLevel::WebOnly => "Only available on web (WASM/browser)",
            PortabilityLevel::MobileOnly => "Only available on mobile (iOS, Android)",
            PortabilityLevel::IosOnly => "Only available on iOS",
            PortabilityLevel::AndroidOnly => "Only available on Android",
            PortabilityLevel::MacosOnly => "Only available on macOS",
            PortabilityLevel::WindowsOnly => "Only available on Windows",
            PortabilityLevel::LinuxOnly => "Only available on Linux",
            PortabilityLevel::NativeOnly => "Available on native platforms (desktop + mobile), not web",
            PortabilityLevel::Experimental => "Experimental - platform support may vary",
        }
    }
}

impl fmt::Display for PortabilityLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PortabilityLevel::Portable => write!(f, "portable"),
            PortabilityLevel::DesktopOnly => write!(f, "desktop-only"),
            PortabilityLevel::WebOnly => write!(f, "web-only"),
            PortabilityLevel::MobileOnly => write!(f, "mobile-only"),
            PortabilityLevel::IosOnly => write!(f, "ios-only"),
            PortabilityLevel::AndroidOnly => write!(f, "android-only"),
            PortabilityLevel::MacosOnly => write!(f, "macos-only"),
            PortabilityLevel::WindowsOnly => write!(f, "windows-only"),
            PortabilityLevel::LinuxOnly => write!(f, "linux-only"),
            PortabilityLevel::NativeOnly => write!(f, "native-only"),
            PortabilityLevel::Experimental => write!(f, "experimental"),
        }
    }
}

impl Default for PortabilityLevel {
    fn default() -> Self {
        // Default to requiring explicit marking, treating as desktop-only
        // This ensures portable code must be explicitly marked
        PortabilityLevel::DesktopOnly
    }
}

/// Categories of APIs with different portability characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ApiCategory {
    /// Core runtime APIs (always portable)
    Core,
    /// UI components
    Ui,
    /// Layout system
    Layout,
    /// Rendering
    Render,
    /// Input handling
    Input,
    /// File system operations
    FileSystem,
    /// Network operations
    Network,
    /// Window management
    Window,
    /// System integration
    System,
    /// Clipboard operations
    Clipboard,
    /// Notifications
    Notifications,
    /// Storage (persistent data)
    Storage,
    /// Authentication
    Auth,
    /// Sensors (camera, mic, gyro)
    Sensors,
    /// Platform-specific native APIs
    Native,
}

impl ApiCategory {
    /// Get the default portability level for this category.
    pub fn default_portability(&self) -> PortabilityLevel {
        match self {
            ApiCategory::Core => PortabilityLevel::Portable,
            ApiCategory::Ui => PortabilityLevel::Portable,
            ApiCategory::Layout => PortabilityLevel::Portable,
            ApiCategory::Render => PortabilityLevel::Portable,
            ApiCategory::Input => PortabilityLevel::Portable,
            ApiCategory::FileSystem => PortabilityLevel::NativeOnly,
            ApiCategory::Network => PortabilityLevel::Portable,
            ApiCategory::Window => PortabilityLevel::DesktopOnly,
            ApiCategory::System => PortabilityLevel::NativeOnly,
            ApiCategory::Clipboard => PortabilityLevel::Portable,
            ApiCategory::Notifications => PortabilityLevel::Portable,
            ApiCategory::Storage => PortabilityLevel::Portable,
            ApiCategory::Auth => PortabilityLevel::Portable,
            ApiCategory::Sensors => PortabilityLevel::NativeOnly,
            ApiCategory::Native => PortabilityLevel::DesktopOnly,
        }
    }

    /// Get categories that are fully portable.
    pub fn portable_categories() -> Vec<Self> {
        vec![
            ApiCategory::Core,
            ApiCategory::Ui,
            ApiCategory::Layout,
            ApiCategory::Render,
            ApiCategory::Input,
            ApiCategory::Network,
            ApiCategory::Clipboard,
            ApiCategory::Notifications,
            ApiCategory::Storage,
            ApiCategory::Auth,
        ]
    }

    /// Get categories that are desktop-only.
    pub fn desktop_only_categories() -> Vec<Self> {
        vec![
            ApiCategory::Window,
            ApiCategory::Native,
        ]
    }
}

impl fmt::Display for ApiCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ApiCategory::Core => write!(f, "core"),
            ApiCategory::Ui => write!(f, "ui"),
            ApiCategory::Layout => write!(f, "layout"),
            ApiCategory::Render => write!(f, "render"),
            ApiCategory::Input => write!(f, "input"),
            ApiCategory::FileSystem => write!(f, "filesystem"),
            ApiCategory::Network => write!(f, "network"),
            ApiCategory::Window => write!(f, "window"),
            ApiCategory::System => write!(f, "system"),
            ApiCategory::Clipboard => write!(f, "clipboard"),
            ApiCategory::Notifications => write!(f, "notifications"),
            ApiCategory::Storage => write!(f, "storage"),
            ApiCategory::Auth => write!(f, "auth"),
            ApiCategory::Sensors => write!(f, "sensors"),
            ApiCategory::Native => write!(f, "native"),
        }
    }
}

/// Constraint that must be satisfied for an API to work.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PortabilityConstraint {
    /// Required capabilities
    pub required_capabilities: HashSet<String>,
    /// Forbidden targets (API won't work on these)
    pub forbidden_targets: HashSet<String>,
    /// Allowed targets (if not empty, API only works on these)
    pub allowed_targets: HashSet<String>,
    /// Minimum Rust version required
    pub min_rust_version: Option<String>,
    /// Required features
    pub required_features: HashSet<String>,
}

impl Default for PortabilityConstraint {
    fn default() -> Self {
        Self {
            required_capabilities: HashSet::new(),
            forbidden_targets: HashSet::new(),
            allowed_targets: HashSet::new(),
            min_rust_version: None,
            required_features: HashSet::new(),
        }
    }
}

impl PortabilityConstraint {
    /// Create a new empty constraint.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a constraint that requires specific capabilities.
    pub fn requires_capabilities<I, S>(capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            required_capabilities: capabilities.into_iter().map(Into::into).collect(),
            ..Default::default()
        }
    }

    /// Add a required capability.
    pub fn require_capability(mut self, capability: impl Into<String>) -> Self {
        self.required_capabilities.insert(capability.into());
        self
    }

    /// Forbid specific targets.
    pub fn forbid_target(mut self, target: impl Into<String>) -> Self {
        self.forbidden_targets.insert(target.into());
        self
    }

    /// Allow only specific targets.
    pub fn allow_target(mut self, target: impl Into<String>) -> Self {
        self.allowed_targets.insert(target.into());
        self
    }

    /// Require a minimum Rust version.
    pub fn require_rust_version(mut self, version: impl Into<String>) -> Self {
        self.min_rust_version = Some(version.into());
        self
    }

    /// Require a feature to be enabled.
    pub fn require_feature(mut self, feature: impl Into<String>) -> Self {
        self.required_features.insert(feature.into());
        self
    }

    /// Check if constraints are satisfied for a target.
    pub fn is_satisfied_for(&self, target: &crate::Target) -> bool {
        // Check forbidden targets
        if self.forbidden_targets.contains(target.triple()) {
            return false;
        }

        // Check allowed targets (if specified)
        if !self.allowed_targets.is_empty() && !self.allowed_targets.contains(target.triple()) {
            return false;
        }

        // Check required capabilities
        for capability in &self.required_capabilities {
            if !target.has_capability(capability) {
                return false;
            }
        }

        true
    }

    /// Merge with another constraint.
    pub fn merge(&mut self, other: &PortabilityConstraint) {
        self.required_capabilities.extend(other.required_capabilities.iter().cloned());
        self.forbidden_targets.extend(other.forbidden_targets.iter().cloned());
        self.allowed_targets.extend(other.allowed_targets.iter().cloned());

        if self.min_rust_version.is_none() {
            self.min_rust_version = other.min_rust_version.clone();
        }

        self.required_features.extend(other.required_features.iter().cloned());
    }
}

/// Predefined constraints for common scenarios.
pub mod constraints {
    use super::*;

    /// Constraint for portable APIs.
    pub fn portable() -> PortabilityConstraint {
        PortabilityConstraint::new()
    }

    /// Constraint for desktop-only APIs.
    pub fn desktop_only() -> PortabilityConstraint {
        PortabilityConstraint::new()
            .require_capability("native_windows")
            .forbid_target("wasm32-unknown-unknown")
    }

    /// Constraint for native-only APIs (no web).
    pub fn native_only() -> PortabilityConstraint {
        PortabilityConstraint::new()
            .require_capability("filesystem")
            .forbid_target("wasm32-unknown-unknown")
    }

    /// Constraint for APIs requiring filesystem.
    pub fn requires_filesystem() -> PortabilityConstraint {
        PortabilityConstraint::requires_capabilities(["filesystem"])
    }

    /// Constraint for APIs requiring GPU.
    pub fn requires_gpu() -> PortabilityConstraint {
        PortabilityConstraint::requires_capabilities(["gpu"])
    }

    /// Constraint for APIs requiring threading.
    pub fn requires_threads() -> PortabilityConstraint {
        PortabilityConstraint::requires_capabilities(["threads"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::targets;

    #[test]
    fn test_portability_levels() {
        assert!(PortabilityLevel::Portable.is_portable());
        assert!(!PortabilityLevel::DesktopOnly.is_portable());

        assert!(PortabilityLevel::Portable.works_on_desktop());
        assert!(PortabilityLevel::Portable.works_on_web());
        assert!(PortabilityLevel::Portable.works_on_mobile());

        assert!(PortabilityLevel::DesktopOnly.works_on_desktop());
        assert!(!PortabilityLevel::DesktopOnly.works_on_web());
    }

    #[test]
    fn test_api_categories() {
        assert_eq!(ApiCategory::Core.default_portability(), PortabilityLevel::Portable);
        assert_eq!(ApiCategory::Window.default_portability(), PortabilityLevel::DesktopOnly);
        assert_eq!(ApiCategory::FileSystem.default_portability(), PortabilityLevel::NativeOnly);
    }

    #[test]
    fn test_constraints() {
        let desktop = targets::macos_arm64();
        let web = targets::web_wasm32();

        let desktop_constraint = constraints::desktop_only();
        assert!(desktop_constraint.is_satisfied_for(&desktop));
        assert!(!desktop_constraint.is_satisfied_for(&web));

        let portable = constraints::portable();
        assert!(portable.is_satisfied_for(&desktop));
        assert!(portable.is_satisfied_for(&web));
    }
}
