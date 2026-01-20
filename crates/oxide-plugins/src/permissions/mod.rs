//! Plugin permissions and capability model.
//!
//! Plugins must declare the capabilities they require, and the app must
//! explicitly allow them in `oxide.toml`. This prevents plugins from
//! silently accessing dangerous OS resources.
//!
//! # Example
//!
//! In `plugin.toml`:
//! ```toml
//! [native]
//! capabilities = ["filesystem.read", "filesystem.write"]
//! ```
//!
//! In `oxide.toml`:
//! ```toml
//! [plugins.native.keychain]
//! allow = ["filesystem.read", "keychain.access"]
//! ```

use std::collections::HashSet;
use std::str::FromStr;
use serde::{Deserialize, Serialize};

use crate::error::{PluginError, PluginResult};

/// A capability that a plugin may request.
///
/// Capabilities are hierarchical permissions that grant access to
/// specific OS or app resources.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Capability {
    // Filesystem capabilities
    /// Read files from the filesystem.
    #[serde(rename = "filesystem.read")]
    FilesystemRead,
    /// Write files to the filesystem.
    #[serde(rename = "filesystem.write")]
    FilesystemWrite,

    // Keychain capabilities
    /// Access the system keychain.
    #[serde(rename = "keychain.access")]
    KeychainAccess,

    // Network capabilities
    /// Make HTTP/HTTPS requests.
    #[serde(rename = "network.http")]
    NetworkHttp,
    /// Use WebSocket connections.
    #[serde(rename = "network.websocket")]
    NetworkWebsocket,

    // Notification capabilities
    /// Send system notifications.
    #[serde(rename = "notifications.send")]
    NotificationsSend,

    // Process capabilities
    /// Spawn external processes.
    #[serde(rename = "process.spawn")]
    ProcessSpawn,

    // Clipboard capabilities
    /// Read from the clipboard.
    #[serde(rename = "clipboard.read")]
    ClipboardRead,
    /// Write to the clipboard.
    #[serde(rename = "clipboard.write")]
    ClipboardWrite,

    // System capabilities
    /// Access the system tray.
    #[serde(rename = "system.tray")]
    SystemTray,
    /// Use the auto-updater.
    #[serde(rename = "system.auto_updater")]
    AutoUpdater,

    // Window capabilities
    /// Create new windows.
    #[serde(rename = "window.create")]
    WindowCreate,
    /// Access window management.
    #[serde(rename = "window.manage")]
    WindowManage,

    // Storage capabilities
    /// Access local storage.
    #[serde(rename = "storage.local")]
    StorageLocal,
    /// Access session storage.
    #[serde(rename = "storage.session")]
    StorageSession,

    // Device capabilities
    /// Access device information.
    #[serde(rename = "device.info")]
    DeviceInfo,
    /// Access camera.
    #[serde(rename = "device.camera")]
    DeviceCamera,
    /// Access microphone.
    #[serde(rename = "device.microphone")]
    DeviceMicrophone,
}

impl Capability {
    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            Capability::FilesystemRead => "filesystem.read",
            Capability::FilesystemWrite => "filesystem.write",
            Capability::KeychainAccess => "keychain.access",
            Capability::NetworkHttp => "network.http",
            Capability::NetworkWebsocket => "network.websocket",
            Capability::NotificationsSend => "notifications.send",
            Capability::ProcessSpawn => "process.spawn",
            Capability::ClipboardRead => "clipboard.read",
            Capability::ClipboardWrite => "clipboard.write",
            Capability::SystemTray => "system.tray",
            Capability::AutoUpdater => "system.auto_updater",
            Capability::WindowCreate => "window.create",
            Capability::WindowManage => "window.manage",
            Capability::StorageLocal => "storage.local",
            Capability::StorageSession => "storage.session",
            Capability::DeviceInfo => "device.info",
            Capability::DeviceCamera => "device.camera",
            Capability::DeviceMicrophone => "device.microphone",
        }
    }

    /// Get the category of this capability.
    pub fn category(&self) -> &'static str {
        match self {
            Capability::FilesystemRead | Capability::FilesystemWrite => "filesystem",
            Capability::KeychainAccess => "keychain",
            Capability::NetworkHttp | Capability::NetworkWebsocket => "network",
            Capability::NotificationsSend => "notifications",
            Capability::ProcessSpawn => "process",
            Capability::ClipboardRead | Capability::ClipboardWrite => "clipboard",
            Capability::SystemTray | Capability::AutoUpdater => "system",
            Capability::WindowCreate | Capability::WindowManage => "window",
            Capability::StorageLocal | Capability::StorageSession => "storage",
            Capability::DeviceInfo | Capability::DeviceCamera | Capability::DeviceMicrophone => "device",
        }
    }

    /// Get the risk level of this capability.
    pub fn risk_level(&self) -> RiskLevel {
        match self {
            // High risk capabilities
            Capability::ProcessSpawn => RiskLevel::High,
            Capability::KeychainAccess => RiskLevel::High,
            Capability::FilesystemWrite => RiskLevel::High,

            // Medium risk capabilities
            Capability::FilesystemRead => RiskLevel::Medium,
            Capability::NetworkHttp => RiskLevel::Medium,
            Capability::NetworkWebsocket => RiskLevel::Medium,
            Capability::ClipboardRead => RiskLevel::Medium,
            Capability::DeviceCamera => RiskLevel::Medium,
            Capability::DeviceMicrophone => RiskLevel::Medium,

            // Low risk capabilities
            Capability::NotificationsSend => RiskLevel::Low,
            Capability::ClipboardWrite => RiskLevel::Low,
            Capability::SystemTray => RiskLevel::Low,
            Capability::AutoUpdater => RiskLevel::Low,
            Capability::WindowCreate => RiskLevel::Low,
            Capability::WindowManage => RiskLevel::Low,
            Capability::StorageLocal => RiskLevel::Low,
            Capability::StorageSession => RiskLevel::Low,
            Capability::DeviceInfo => RiskLevel::Low,
        }
    }

    /// Get a description of this capability.
    pub fn description(&self) -> &'static str {
        match self {
            Capability::FilesystemRead => "Read files from the filesystem",
            Capability::FilesystemWrite => "Write files to the filesystem",
            Capability::KeychainAccess => "Access secure credentials in the system keychain",
            Capability::NetworkHttp => "Make HTTP/HTTPS requests to external servers",
            Capability::NetworkWebsocket => "Establish WebSocket connections",
            Capability::NotificationsSend => "Send system notifications",
            Capability::ProcessSpawn => "Spawn external processes",
            Capability::ClipboardRead => "Read from the clipboard",
            Capability::ClipboardWrite => "Write to the clipboard",
            Capability::SystemTray => "Add icons and menus to the system tray",
            Capability::AutoUpdater => "Automatically download and install updates",
            Capability::WindowCreate => "Create new application windows",
            Capability::WindowManage => "Manage existing windows (resize, move, close)",
            Capability::StorageLocal => "Store data in local storage",
            Capability::StorageSession => "Store data in session storage",
            Capability::DeviceInfo => "Access device information",
            Capability::DeviceCamera => "Access the device camera",
            Capability::DeviceMicrophone => "Access the device microphone",
        }
    }

    /// Get all capabilities.
    pub fn all() -> &'static [Capability] {
        &[
            Capability::FilesystemRead,
            Capability::FilesystemWrite,
            Capability::KeychainAccess,
            Capability::NetworkHttp,
            Capability::NetworkWebsocket,
            Capability::NotificationsSend,
            Capability::ProcessSpawn,
            Capability::ClipboardRead,
            Capability::ClipboardWrite,
            Capability::SystemTray,
            Capability::AutoUpdater,
            Capability::WindowCreate,
            Capability::WindowManage,
            Capability::StorageLocal,
            Capability::StorageSession,
            Capability::DeviceInfo,
            Capability::DeviceCamera,
            Capability::DeviceMicrophone,
        ]
    }
}

impl FromStr for Capability {
    type Err = PluginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "filesystem.read" => Ok(Capability::FilesystemRead),
            "filesystem.write" => Ok(Capability::FilesystemWrite),
            "keychain.access" => Ok(Capability::KeychainAccess),
            "network.http" => Ok(Capability::NetworkHttp),
            "network.websocket" => Ok(Capability::NetworkWebsocket),
            "notifications.send" => Ok(Capability::NotificationsSend),
            "process.spawn" => Ok(Capability::ProcessSpawn),
            "clipboard.read" => Ok(Capability::ClipboardRead),
            "clipboard.write" => Ok(Capability::ClipboardWrite),
            "system.tray" => Ok(Capability::SystemTray),
            "system.auto_updater" => Ok(Capability::AutoUpdater),
            "window.create" => Ok(Capability::WindowCreate),
            "window.manage" => Ok(Capability::WindowManage),
            "storage.local" => Ok(Capability::StorageLocal),
            "storage.session" => Ok(Capability::StorageSession),
            "device.info" => Ok(Capability::DeviceInfo),
            "device.camera" => Ok(Capability::DeviceCamera),
            "device.microphone" => Ok(Capability::DeviceMicrophone),
            _ => Err(PluginError::PermissionDenied(format!("unknown capability: {}", s))),
        }
    }
}

impl std::fmt::Display for Capability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Risk level of a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    /// Low risk - minimal security implications.
    Low,
    /// Medium risk - requires user awareness.
    Medium,
    /// High risk - requires explicit user approval.
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Medium => write!(f, "medium"),
            RiskLevel::High => write!(f, "high"),
        }
    }
}

/// A permission grant for a specific capability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    /// The capability being granted.
    pub capability: Capability,
    /// Optional scope restriction.
    pub scope: Option<PermissionScope>,
    /// Whether this permission was explicitly granted by the user.
    pub explicit: bool,
}

impl Permission {
    /// Create a new permission.
    pub fn new(capability: Capability) -> Self {
        Self {
            capability,
            scope: None,
            explicit: false,
        }
    }

    /// Create a permission with scope.
    pub fn with_scope(capability: Capability, scope: PermissionScope) -> Self {
        Self {
            capability,
            scope: Some(scope),
            explicit: false,
        }
    }

    /// Mark this permission as explicitly granted.
    pub fn explicit(mut self) -> Self {
        self.explicit = true;
        self
    }
}

/// Scope restriction for a permission.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PermissionScope {
    /// Restrict filesystem access to specific paths.
    Paths(Vec<String>),
    /// Restrict network access to specific domains.
    Domains(Vec<String>),
    /// Custom scope with key-value pairs.
    Custom(std::collections::HashMap<String, String>),
}

/// A set of permissions for a plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PermissionSet {
    /// Granted permissions.
    permissions: HashSet<Capability>,
    /// Permissions with scopes.
    scoped: Vec<Permission>,
}

impl PermissionSet {
    /// Create a new empty permission set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a capability to the permission set.
    pub fn allow(&mut self, capability: Capability) {
        self.permissions.insert(capability);
    }

    /// Add a scoped permission.
    pub fn allow_scoped(&mut self, permission: Permission) {
        self.permissions.insert(permission.capability);
        self.scoped.push(permission);
    }

    /// Check if a capability is allowed.
    pub fn is_allowed(&self, capability: Capability) -> bool {
        self.permissions.contains(&capability)
    }

    /// Check if a capability is allowed with a specific scope.
    pub fn is_allowed_for(&self, capability: Capability, context: &str) -> bool {
        // Check if basic permission exists
        if !self.permissions.contains(&capability) {
            return false;
        }

        // Check scoped permissions
        for perm in &self.scoped {
            if perm.capability == capability {
                if let Some(scope) = &perm.scope {
                    match scope {
                        PermissionScope::Paths(paths) => {
                            return paths.iter().any(|p| context.starts_with(p));
                        }
                        PermissionScope::Domains(domains) => {
                            return domains.iter().any(|d| context.contains(d));
                        }
                        PermissionScope::Custom(_) => {
                            // Custom scopes require application-specific handling
                            return true;
                        }
                    }
                }
            }
        }

        // No scope restriction, permission is granted
        true
    }

    /// Get all allowed capabilities.
    pub fn allowed_capabilities(&self) -> impl Iterator<Item = &Capability> {
        self.permissions.iter()
    }

    /// Check if the permission set is empty.
    pub fn is_empty(&self) -> bool {
        self.permissions.is_empty()
    }

    /// Get the number of permissions.
    pub fn len(&self) -> usize {
        self.permissions.len()
    }

    /// Merge another permission set into this one.
    pub fn merge(&mut self, other: &PermissionSet) {
        for cap in &other.permissions {
            self.permissions.insert(*cap);
        }
        for perm in &other.scoped {
            self.scoped.push(perm.clone());
        }
    }
}

/// Check if a set of requested capabilities is satisfied by a permission set.
pub fn check_permissions(
    required: &[Capability],
    granted: &PermissionSet,
) -> PluginResult<()> {
    for cap in required {
        if !granted.is_allowed(*cap) {
            return Err(PluginError::PermissionDenied(format!(
                "capability '{}' is required but not granted",
                cap
            )));
        }
    }
    Ok(())
}

/// Parse capability strings into a permission set.
pub fn parse_capabilities(capabilities: &[String]) -> PluginResult<Vec<Capability>> {
    capabilities
        .iter()
        .map(|s| s.parse::<Capability>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_parsing() {
        assert_eq!(
            "filesystem.read".parse::<Capability>().unwrap(),
            Capability::FilesystemRead
        );
        assert!("invalid.cap".parse::<Capability>().is_err());
    }

    #[test]
    fn test_permission_set() {
        let mut perms = PermissionSet::new();
        perms.allow(Capability::FilesystemRead);
        perms.allow(Capability::NetworkHttp);

        assert!(perms.is_allowed(Capability::FilesystemRead));
        assert!(perms.is_allowed(Capability::NetworkHttp));
        assert!(!perms.is_allowed(Capability::ProcessSpawn));
    }

    #[test]
    fn test_scoped_permissions() {
        let mut perms = PermissionSet::new();
        perms.allow_scoped(Permission::with_scope(
            Capability::FilesystemRead,
            PermissionScope::Paths(vec!["/home/user/docs".to_string()]),
        ));

        assert!(perms.is_allowed_for(Capability::FilesystemRead, "/home/user/docs/file.txt"));
        assert!(!perms.is_allowed_for(Capability::FilesystemRead, "/etc/passwd"));
    }

    #[test]
    fn test_risk_levels() {
        assert_eq!(Capability::ProcessSpawn.risk_level(), RiskLevel::High);
        assert_eq!(Capability::NetworkHttp.risk_level(), RiskLevel::Medium);
        assert_eq!(Capability::NotificationsSend.risk_level(), RiskLevel::Low);
    }
}
