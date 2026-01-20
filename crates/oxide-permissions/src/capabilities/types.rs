//! Core capability type definitions.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A capability represents a specific permission or access right.
///
/// Capabilities follow a hierarchical namespace format:
/// `category.subcategory.action`
///
/// Examples:
/// - `filesystem.read`
/// - `filesystem.write`
/// - `network.http`
/// - `keychain.access`
/// - `camera.capture`
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Capability(String);

impl Capability {
    /// Create a new capability from a string.
    pub fn new(capability: impl Into<String>) -> Self {
        Self(capability.into())
    }

    /// Get the capability as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Get the category (first segment) of the capability.
    pub fn category(&self) -> &str {
        self.0.split('.').next().unwrap_or(&self.0)
    }

    /// Check if this capability is a parent of another.
    ///
    /// For example, `filesystem` is a parent of `filesystem.read`.
    pub fn is_parent_of(&self, other: &Capability) -> bool {
        other.0.starts_with(&self.0) && other.0.len() > self.0.len()
    }

    /// Check if this capability matches another (exact or parent).
    pub fn matches(&self, other: &Capability) -> bool {
        self.0 == other.0 || self.is_parent_of(other)
    }

    // Predefined capabilities for common operations

    /// Filesystem read capability
    pub const FILESYSTEM_READ: &'static str = "filesystem.read";
    /// Filesystem write capability
    pub const FILESYSTEM_WRITE: &'static str = "filesystem.write";
    /// Filesystem full access
    pub const FILESYSTEM_FULL: &'static str = "filesystem";

    /// Keychain access capability
    pub const KEYCHAIN_ACCESS: &'static str = "keychain.access";
    /// Keychain read-only
    pub const KEYCHAIN_READ: &'static str = "keychain.read";

    /// Network HTTP capability
    pub const NETWORK_HTTP: &'static str = "network.http";
    /// Network WebSocket capability
    pub const NETWORK_WEBSOCKET: &'static str = "network.websocket";
    /// Network full access
    pub const NETWORK_FULL: &'static str = "network";

    /// Camera capture capability
    pub const CAMERA_CAPTURE: &'static str = "camera.capture";
    /// Camera stream capability
    pub const CAMERA_STREAM: &'static str = "camera.stream";

    /// Microphone record capability
    pub const MICROPHONE_RECORD: &'static str = "microphone.record";
    /// Microphone stream capability
    pub const MICROPHONE_STREAM: &'static str = "microphone.stream";

    /// Screenshot capture capability
    pub const SCREENSHOT_CAPTURE: &'static str = "screenshot.capture";

    /// Clipboard read capability
    pub const CLIPBOARD_READ: &'static str = "clipboard.read";
    /// Clipboard write capability
    pub const CLIPBOARD_WRITE: &'static str = "clipboard.write";

    /// Background task capability
    pub const BACKGROUND_TASK: &'static str = "background.task";
    /// Background service capability
    pub const BACKGROUND_SERVICE: &'static str = "background.service";

    /// Notifications capability
    pub const NOTIFICATIONS: &'static str = "notifications";

    /// System info read capability
    pub const SYSTEM_INFO: &'static str = "system.info";

    /// Location access capability
    pub const LOCATION: &'static str = "location";
}

impl fmt::Display for Capability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Capability {
    type Err = crate::error::PermissionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Validate capability format
        if s.is_empty() {
            return Err(crate::error::PermissionError::InvalidCapability(
                "Capability cannot be empty".to_string(),
            ));
        }

        // Must start with a letter and contain only alphanumeric and dots
        let valid = s.chars().enumerate().all(|(i, c)| {
            if i == 0 {
                c.is_ascii_lowercase()
            } else {
                c.is_ascii_lowercase() || c.is_ascii_digit() || c == '.' || c == '_'
            }
        });

        if !valid {
            return Err(crate::error::PermissionError::InvalidCapability(format!(
                "Invalid capability format: '{}'. Must be lowercase alphanumeric with dots.",
                s
            )));
        }

        Ok(Self(s.to_string()))
    }
}

impl From<&str> for Capability {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

/// Category of capabilities for grouping and display.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    /// Filesystem operations
    Filesystem,
    /// Keychain/secrets access
    Keychain,
    /// Network operations
    Network,
    /// Camera access
    Camera,
    /// Microphone access
    Microphone,
    /// Screenshot capture
    Screenshot,
    /// Clipboard operations
    Clipboard,
    /// Background execution
    Background,
    /// System notifications
    Notifications,
    /// System information
    System,
    /// Location services
    Location,
    /// Custom/plugin-defined
    Custom,
}

impl CapabilityCategory {
    /// Get the category from a capability string.
    pub fn from_capability(cap: &Capability) -> Self {
        match cap.category() {
            "filesystem" => Self::Filesystem,
            "keychain" => Self::Keychain,
            "network" => Self::Network,
            "camera" => Self::Camera,
            "microphone" => Self::Microphone,
            "screenshot" => Self::Screenshot,
            "clipboard" => Self::Clipboard,
            "background" => Self::Background,
            "notifications" => Self::Notifications,
            "system" => Self::System,
            "location" => Self::Location,
            _ => Self::Custom,
        }
    }

    /// Get a human-readable description of this category.
    pub fn description(&self) -> &'static str {
        match self {
            Self::Filesystem => "File System Access",
            Self::Keychain => "Keychain & Secrets",
            Self::Network => "Network Access",
            Self::Camera => "Camera Access",
            Self::Microphone => "Microphone Access",
            Self::Screenshot => "Screenshot Capture",
            Self::Clipboard => "Clipboard Access",
            Self::Background => "Background Execution",
            Self::Notifications => "System Notifications",
            Self::System => "System Information",
            Self::Location => "Location Services",
            Self::Custom => "Custom Capability",
        }
    }

    /// Get the risk level for this category.
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            Self::Filesystem => RiskLevel::High,
            Self::Keychain => RiskLevel::Critical,
            Self::Network => RiskLevel::High,
            Self::Camera => RiskLevel::High,
            Self::Microphone => RiskLevel::High,
            Self::Screenshot => RiskLevel::High,
            Self::Clipboard => RiskLevel::Medium,
            Self::Background => RiskLevel::Medium,
            Self::Notifications => RiskLevel::Low,
            Self::System => RiskLevel::Low,
            Self::Location => RiskLevel::High,
            Self::Custom => RiskLevel::Medium,
        }
    }
}

/// Risk level for a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    /// Low risk - minimal privacy/security impact
    Low,
    /// Medium risk - some privacy/security considerations
    Medium,
    /// High risk - significant privacy/security impact
    High,
    /// Critical risk - access to sensitive secrets or system resources
    Critical,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Medium => write!(f, "Medium"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

/// Permission grant status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionStatus {
    /// Permission is granted
    Granted,
    /// Permission is denied
    Denied,
    /// Permission requires user prompt
    RequiresPrompt,
    /// Permission status is unknown (not enforceable)
    Unknown,
}

impl fmt::Display for PermissionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Granted => write!(f, "Granted"),
            Self::Denied => write!(f, "Denied"),
            Self::RequiresPrompt => write!(f, "Requires Prompt"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_parsing() {
        let cap: Capability = "filesystem.read".parse().unwrap();
        assert_eq!(cap.as_str(), "filesystem.read");
        assert_eq!(cap.category(), "filesystem");
    }

    #[test]
    fn test_capability_matching() {
        let parent = Capability::new("filesystem");
        let child = Capability::new("filesystem.read");

        assert!(parent.is_parent_of(&child));
        assert!(parent.matches(&child));
        assert!(!child.is_parent_of(&parent));
    }

    #[test]
    fn test_invalid_capability() {
        assert!("".parse::<Capability>().is_err());
        assert!("123invalid".parse::<Capability>().is_err());
        assert!("Valid.Caps".parse::<Capability>().is_err()); // uppercase
    }

    #[test]
    fn test_category_from_capability() {
        let cap = Capability::new("network.http");
        assert_eq!(
            CapabilityCategory::from_capability(&cap),
            CapabilityCategory::Network
        );
    }

    #[test]
    fn test_risk_levels() {
        assert!(RiskLevel::Critical > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
    }
}
