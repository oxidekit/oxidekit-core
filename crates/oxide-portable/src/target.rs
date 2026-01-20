//! Target abstraction layer for cross-platform support.
//!
//! Provides a unified way to query and work with different target platforms,
//! abstracting away the differences between desktop, web, and mobile targets.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Represents a build target with its platform, architecture, and capabilities.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Target {
    /// The target triple (e.g., "x86_64-apple-darwin")
    triple: String,
    /// The platform family
    platform: Platform,
    /// The CPU architecture
    architecture: Architecture,
    /// The target family (desktop, web, mobile)
    family: TargetFamily,
    /// Target-specific capabilities
    capabilities: TargetCapabilities,
}

/// Platform types supported by OxideKit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Platform {
    /// macOS desktop
    MacOS,
    /// Windows desktop
    Windows,
    /// Linux desktop
    Linux,
    /// Web/WASM
    Web,
    /// iOS mobile
    IOS,
    /// Android mobile
    Android,
    /// Unknown or unsupported platform
    Unknown,
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::MacOS => write!(f, "macos"),
            Platform::Windows => write!(f, "windows"),
            Platform::Linux => write!(f, "linux"),
            Platform::Web => write!(f, "web"),
            Platform::IOS => write!(f, "ios"),
            Platform::Android => write!(f, "android"),
            Platform::Unknown => write!(f, "unknown"),
        }
    }
}

/// CPU architecture types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Architecture {
    /// x86 64-bit
    X86_64,
    /// ARM 64-bit (Apple Silicon, modern ARM servers)
    Aarch64,
    /// ARM 32-bit
    Arm,
    /// WebAssembly 32-bit
    Wasm32,
    /// WebAssembly 64-bit
    Wasm64,
    /// Unknown architecture
    Unknown,
}

impl fmt::Display for Architecture {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Architecture::X86_64 => write!(f, "x86_64"),
            Architecture::Aarch64 => write!(f, "aarch64"),
            Architecture::Arm => write!(f, "arm"),
            Architecture::Wasm32 => write!(f, "wasm32"),
            Architecture::Wasm64 => write!(f, "wasm64"),
            Architecture::Unknown => write!(f, "unknown"),
        }
    }
}

/// Target family classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TargetFamily {
    /// Desktop platforms (macOS, Windows, Linux)
    Desktop,
    /// Web platform (WASM in browser)
    Web,
    /// Mobile platforms (iOS, Android)
    Mobile,
    /// Unknown target family
    Unknown,
}

impl fmt::Display for TargetFamily {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TargetFamily::Desktop => write!(f, "desktop"),
            TargetFamily::Web => write!(f, "web"),
            TargetFamily::Mobile => write!(f, "mobile"),
            TargetFamily::Unknown => write!(f, "unknown"),
        }
    }
}

/// Capabilities available on a specific target.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TargetCapabilities {
    /// Has native filesystem access
    pub filesystem: bool,
    /// Has native window management
    pub native_windows: bool,
    /// Has GPU acceleration
    pub gpu: bool,
    /// Has threading support
    pub threads: bool,
    /// Has network access
    pub network: bool,
    /// Has clipboard access
    pub clipboard: bool,
    /// Has system notifications
    pub notifications: bool,
    /// Has native menu support
    pub native_menus: bool,
    /// Has system tray support
    pub system_tray: bool,
    /// Has file dialogs
    pub file_dialogs: bool,
    /// Has keyboard/mouse input
    pub input: bool,
    /// Has touch input
    pub touch: bool,
    /// Has camera access
    pub camera: bool,
    /// Has microphone access
    pub microphone: bool,
    /// Has geolocation
    pub geolocation: bool,
    /// Has biometric authentication
    pub biometrics: bool,
    /// Has push notifications
    pub push_notifications: bool,
    /// Has persistent storage
    pub persistent_storage: bool,
    /// Has secure storage (keychain, etc.)
    pub secure_storage: bool,
}

impl Target {
    /// Get the current compile target.
    pub fn current() -> Self {
        Self::from_current_cfg()
    }

    /// Parse a target from a target triple string.
    pub fn from_triple(triple: &str) -> Self {
        let parts: Vec<&str> = triple.split('-').collect();

        let architecture = if !parts.is_empty() {
            match parts[0] {
                "x86_64" => Architecture::X86_64,
                "aarch64" | "arm64" => Architecture::Aarch64,
                "arm" | "armv7" | "thumbv7" => Architecture::Arm,
                "wasm32" => Architecture::Wasm32,
                "wasm64" => Architecture::Wasm64,
                _ => Architecture::Unknown,
            }
        } else {
            Architecture::Unknown
        };

        let (platform, family) = Self::detect_platform_from_triple(triple);
        let capabilities = Self::capabilities_for_platform(platform);

        Self {
            triple: triple.to_string(),
            platform,
            architecture,
            family,
            capabilities,
        }
    }

    /// Create a target from the current compilation configuration.
    fn from_current_cfg() -> Self {
        let triple = Self::current_triple();
        Self::from_triple(&triple)
    }

    /// Get the current target triple.
    fn current_triple() -> String {
        // Build target triple from cfg attributes
        let arch = if cfg!(target_arch = "x86_64") {
            "x86_64"
        } else if cfg!(target_arch = "aarch64") {
            "aarch64"
        } else if cfg!(target_arch = "arm") {
            "arm"
        } else if cfg!(target_arch = "wasm32") {
            "wasm32"
        } else if cfg!(target_arch = "wasm64") {
            "wasm64"
        } else {
            "unknown"
        };

        let os = if cfg!(target_os = "macos") {
            "apple-darwin"
        } else if cfg!(target_os = "windows") {
            "pc-windows-msvc"
        } else if cfg!(target_os = "linux") {
            "unknown-linux-gnu"
        } else if cfg!(target_os = "ios") {
            "apple-ios"
        } else if cfg!(target_os = "android") {
            "linux-android"
        } else if cfg!(target_arch = "wasm32") {
            "unknown-unknown"
        } else {
            "unknown-unknown"
        };

        format!("{}-{}", arch, os)
    }

    /// Detect platform and family from target triple.
    fn detect_platform_from_triple(triple: &str) -> (Platform, TargetFamily) {
        let triple_lower = triple.to_lowercase();

        if triple_lower.contains("darwin") || triple_lower.contains("macos") {
            (Platform::MacOS, TargetFamily::Desktop)
        } else if triple_lower.contains("windows") {
            (Platform::Windows, TargetFamily::Desktop)
        } else if triple_lower.contains("linux") && !triple_lower.contains("android") {
            (Platform::Linux, TargetFamily::Desktop)
        } else if triple_lower.contains("wasm") {
            (Platform::Web, TargetFamily::Web)
        } else if triple_lower.contains("ios") {
            (Platform::IOS, TargetFamily::Mobile)
        } else if triple_lower.contains("android") {
            (Platform::Android, TargetFamily::Mobile)
        } else {
            (Platform::Unknown, TargetFamily::Unknown)
        }
    }

    /// Get capabilities for a platform.
    fn capabilities_for_platform(platform: Platform) -> TargetCapabilities {
        match platform {
            Platform::MacOS | Platform::Windows | Platform::Linux => TargetCapabilities {
                filesystem: true,
                native_windows: true,
                gpu: true,
                threads: true,
                network: true,
                clipboard: true,
                notifications: true,
                native_menus: true,
                system_tray: true,
                file_dialogs: true,
                input: true,
                touch: false, // Some touchscreens, but not primary
                camera: true,
                microphone: true,
                geolocation: false, // Not typically available
                biometrics: platform == Platform::MacOS, // Touch ID
                push_notifications: false,
                persistent_storage: true,
                secure_storage: true,
            },
            Platform::Web => TargetCapabilities {
                filesystem: false, // Limited via File System Access API
                native_windows: false,
                gpu: true, // WebGPU
                threads: true, // Web Workers
                network: true, // Fetch API
                clipboard: true, // Clipboard API
                notifications: true, // Notification API
                native_menus: false,
                system_tray: false,
                file_dialogs: true, // File picker
                input: true,
                touch: true,
                camera: true, // MediaDevices
                microphone: true,
                geolocation: true,
                biometrics: true, // WebAuthn
                push_notifications: true, // Service Workers
                persistent_storage: true, // IndexedDB
                secure_storage: false, // No true secure storage
            },
            Platform::IOS => TargetCapabilities {
                filesystem: true, // App sandbox
                native_windows: false, // UIKit views
                gpu: true, // Metal
                threads: true,
                network: true,
                clipboard: true,
                notifications: true,
                native_menus: false, // Different paradigm
                system_tray: false,
                file_dialogs: true, // Document picker
                input: true,
                touch: true, // Primary input
                camera: true,
                microphone: true,
                geolocation: true,
                biometrics: true, // Face ID, Touch ID
                push_notifications: true, // APNs
                persistent_storage: true,
                secure_storage: true, // Keychain
            },
            Platform::Android => TargetCapabilities {
                filesystem: true, // App-specific storage
                native_windows: false, // Android views
                gpu: true, // Vulkan/OpenGL ES
                threads: true,
                network: true,
                clipboard: true,
                notifications: true,
                native_menus: false, // Different paradigm
                system_tray: false,
                file_dialogs: true, // Storage Access Framework
                input: true,
                touch: true, // Primary input
                camera: true,
                microphone: true,
                geolocation: true,
                biometrics: true, // Fingerprint, Face
                push_notifications: true, // FCM
                persistent_storage: true,
                secure_storage: true, // Keystore
            },
            Platform::Unknown => TargetCapabilities::default(),
        }
    }

    /// Get the target triple string.
    pub fn triple(&self) -> &str {
        &self.triple
    }

    /// Get the platform.
    pub fn platform(&self) -> Platform {
        self.platform
    }

    /// Get the architecture.
    pub fn architecture(&self) -> Architecture {
        self.architecture
    }

    /// Get the target family.
    pub fn family(&self) -> TargetFamily {
        self.family
    }

    /// Get the capabilities.
    pub fn capabilities(&self) -> &TargetCapabilities {
        &self.capabilities
    }

    /// Check if this is a desktop target.
    pub fn is_desktop(&self) -> bool {
        self.family == TargetFamily::Desktop
    }

    /// Check if this is a web target.
    pub fn is_web(&self) -> bool {
        self.family == TargetFamily::Web
    }

    /// Check if this is a mobile target.
    pub fn is_mobile(&self) -> bool {
        self.family == TargetFamily::Mobile
    }

    /// Check if a capability is available.
    pub fn has_capability(&self, capability: &str) -> bool {
        match capability {
            "filesystem" => self.capabilities.filesystem,
            "native_windows" => self.capabilities.native_windows,
            "gpu" => self.capabilities.gpu,
            "threads" => self.capabilities.threads,
            "network" => self.capabilities.network,
            "clipboard" => self.capabilities.clipboard,
            "notifications" => self.capabilities.notifications,
            "native_menus" => self.capabilities.native_menus,
            "system_tray" => self.capabilities.system_tray,
            "file_dialogs" => self.capabilities.file_dialogs,
            "input" => self.capabilities.input,
            "touch" => self.capabilities.touch,
            "camera" => self.capabilities.camera,
            "microphone" => self.capabilities.microphone,
            "geolocation" => self.capabilities.geolocation,
            "biometrics" => self.capabilities.biometrics,
            "push_notifications" => self.capabilities.push_notifications,
            "persistent_storage" => self.capabilities.persistent_storage,
            "secure_storage" => self.capabilities.secure_storage,
            _ => false,
        }
    }
}

/// Well-known target definitions for common targets.
pub mod targets {
    use super::*;

    /// macOS on Apple Silicon
    pub fn macos_arm64() -> Target {
        Target::from_triple("aarch64-apple-darwin")
    }

    /// macOS on Intel
    pub fn macos_x86_64() -> Target {
        Target::from_triple("x86_64-apple-darwin")
    }

    /// Windows on x86_64
    pub fn windows_x86_64() -> Target {
        Target::from_triple("x86_64-pc-windows-msvc")
    }

    /// Linux on x86_64
    pub fn linux_x86_64() -> Target {
        Target::from_triple("x86_64-unknown-linux-gnu")
    }

    /// Web/WASM target
    pub fn web_wasm32() -> Target {
        Target::from_triple("wasm32-unknown-unknown")
    }

    /// iOS on ARM64
    pub fn ios_arm64() -> Target {
        Target::from_triple("aarch64-apple-ios")
    }

    /// iOS Simulator on ARM64
    pub fn ios_simulator_arm64() -> Target {
        Target::from_triple("aarch64-apple-ios-sim")
    }

    /// Android on ARM64
    pub fn android_arm64() -> Target {
        Target::from_triple("aarch64-linux-android")
    }

    /// Android on ARM32
    pub fn android_arm32() -> Target {
        Target::from_triple("armv7-linux-androideabi")
    }

    /// All desktop targets
    pub fn all_desktop() -> Vec<Target> {
        vec![
            macos_arm64(),
            macos_x86_64(),
            windows_x86_64(),
            linux_x86_64(),
        ]
    }

    /// All mobile targets
    pub fn all_mobile() -> Vec<Target> {
        vec![
            ios_arm64(),
            ios_simulator_arm64(),
            android_arm64(),
            android_arm32(),
        ]
    }

    /// All web targets
    pub fn all_web() -> Vec<Target> {
        vec![web_wasm32()]
    }

    /// All supported targets
    pub fn all() -> Vec<Target> {
        let mut targets = all_desktop();
        targets.extend(all_mobile());
        targets.extend(all_web());
        targets
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_target_from_triple() {
        let macos = Target::from_triple("aarch64-apple-darwin");
        assert_eq!(macos.platform(), Platform::MacOS);
        assert_eq!(macos.architecture(), Architecture::Aarch64);
        assert_eq!(macos.family(), TargetFamily::Desktop);

        let web = Target::from_triple("wasm32-unknown-unknown");
        assert_eq!(web.platform(), Platform::Web);
        assert_eq!(web.architecture(), Architecture::Wasm32);
        assert_eq!(web.family(), TargetFamily::Web);

        let ios = Target::from_triple("aarch64-apple-ios");
        assert_eq!(ios.platform(), Platform::IOS);
        assert_eq!(ios.family(), TargetFamily::Mobile);
    }

    #[test]
    fn test_capabilities() {
        let desktop = targets::macos_arm64();
        assert!(desktop.capabilities().filesystem);
        assert!(desktop.capabilities().native_windows);
        assert!(desktop.capabilities().system_tray);

        let web = targets::web_wasm32();
        assert!(!web.capabilities().filesystem);
        assert!(!web.capabilities().native_windows);
        assert!(!web.capabilities().system_tray);

        let mobile = targets::ios_arm64();
        assert!(mobile.capabilities().touch);
        assert!(mobile.capabilities().geolocation);
    }

    #[test]
    fn test_target_families() {
        assert!(targets::macos_arm64().is_desktop());
        assert!(targets::web_wasm32().is_web());
        assert!(targets::ios_arm64().is_mobile());
    }

    #[test]
    fn test_all_targets() {
        let all = targets::all();
        assert!(!all.is_empty());

        // Should have desktop, web, and mobile targets
        assert!(all.iter().any(|t| t.is_desktop()));
        assert!(all.iter().any(|t| t.is_web()));
        assert!(all.iter().any(|t| t.is_mobile()));
    }
}
