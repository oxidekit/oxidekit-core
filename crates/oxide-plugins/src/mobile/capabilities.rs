//! Mobile-specific capabilities that plugins can request.
//!
//! These capabilities map to platform-specific permissions on iOS and Android.
//! Plugins must declare required capabilities in their manifest, and the app
//! must grant them at runtime.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::str::FromStr;

use crate::error::{PluginError, PluginResult};

/// Mobile-specific capabilities that plugins can request.
///
/// Each capability maps to platform-specific permissions on iOS and Android.
/// Some capabilities require runtime permission prompts, while others only
/// need to be declared in the app manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MobileCapability {
    // Camera & Media
    /// Capture photos and videos using the device camera.
    #[serde(rename = "camera.capture")]
    CameraCapture,
    /// Control the camera flash.
    #[serde(rename = "camera.flash")]
    CameraFlash,
    /// Read photos from the device photo library.
    #[serde(rename = "photos.read")]
    PhotoLibraryRead,
    /// Save photos to the device photo library.
    #[serde(rename = "photos.write")]
    PhotoLibraryWrite,
    /// Record audio using the device microphone.
    #[serde(rename = "microphone.record")]
    MicrophoneRecord,

    // Storage
    /// Access secure storage (Keychain on iOS, Keystore on Android).
    #[serde(rename = "storage.secure")]
    SecureStorage,
    /// Access the app's sandboxed file system.
    #[serde(rename = "storage.app")]
    FileSystemApp,
    /// Access shared storage areas (Downloads, Documents).
    #[serde(rename = "storage.shared")]
    FileSystemShared,

    // Notifications
    /// Register for and receive push notifications.
    #[serde(rename = "notifications.push")]
    PushNotifications,
    /// Schedule and display local notifications.
    #[serde(rename = "notifications.local")]
    LocalNotifications,

    // Biometrics
    /// Authenticate using biometrics (FaceID/TouchID on iOS, fingerprint on Android).
    #[serde(rename = "biometric.auth")]
    BiometricAuth,

    // System
    /// Execute tasks in the background.
    #[serde(rename = "system.background")]
    BackgroundExecution,
    /// Register and handle deep links / universal links.
    #[serde(rename = "system.deeplinks")]
    DeepLinks,
    /// Show the system share sheet.
    #[serde(rename = "system.share")]
    ShareSheet,
    /// Trigger haptic feedback.
    #[serde(rename = "system.haptics")]
    Haptics,

    // Sensors
    /// Access location while the app is in use.
    #[serde(rename = "location.when_in_use")]
    LocationWhenInUse,
    /// Access location even when the app is in the background.
    #[serde(rename = "location.always")]
    LocationAlways,
    /// Access motion sensors (accelerometer, gyroscope).
    #[serde(rename = "sensors.motion")]
    MotionSensors,
}

impl MobileCapability {
    /// Get the iOS Info.plist permission key for this capability.
    ///
    /// Returns `None` if the capability doesn't require an Info.plist entry.
    pub fn ios_permission_key(&self) -> Option<&'static str> {
        match self {
            MobileCapability::CameraCapture => Some("NSCameraUsageDescription"),
            MobileCapability::CameraFlash => None, // No separate permission
            MobileCapability::PhotoLibraryRead => Some("NSPhotoLibraryUsageDescription"),
            MobileCapability::PhotoLibraryWrite => Some("NSPhotoLibraryAddUsageDescription"),
            MobileCapability::MicrophoneRecord => Some("NSMicrophoneUsageDescription"),
            MobileCapability::SecureStorage => None, // Keychain doesn't need permission
            MobileCapability::FileSystemApp => None, // App sandbox is automatic
            MobileCapability::FileSystemShared => Some("NSDocumentsFolderUsageDescription"),
            MobileCapability::PushNotifications => None, // Handled by UNUserNotificationCenter
            MobileCapability::LocalNotifications => None, // Handled by UNUserNotificationCenter
            MobileCapability::BiometricAuth => Some("NSFaceIDUsageDescription"),
            MobileCapability::BackgroundExecution => Some("UIBackgroundModes"),
            MobileCapability::DeepLinks => None, // Configured in entitlements
            MobileCapability::ShareSheet => None, // No permission needed
            MobileCapability::Haptics => None,   // No permission needed
            MobileCapability::LocationWhenInUse => Some("NSLocationWhenInUseUsageDescription"),
            MobileCapability::LocationAlways => Some("NSLocationAlwaysAndWhenInUseUsageDescription"),
            MobileCapability::MotionSensors => Some("NSMotionUsageDescription"),
        }
    }

    /// Get the Android permission string for this capability.
    ///
    /// Returns `None` if the capability doesn't require an Android permission.
    pub fn android_permission(&self) -> Option<&'static str> {
        match self {
            MobileCapability::CameraCapture => Some("android.permission.CAMERA"),
            MobileCapability::CameraFlash => Some("android.permission.FLASHLIGHT"),
            MobileCapability::PhotoLibraryRead => Some("android.permission.READ_MEDIA_IMAGES"),
            MobileCapability::PhotoLibraryWrite => Some("android.permission.WRITE_EXTERNAL_STORAGE"),
            MobileCapability::MicrophoneRecord => Some("android.permission.RECORD_AUDIO"),
            MobileCapability::SecureStorage => None, // Keystore doesn't need permission
            MobileCapability::FileSystemApp => None, // App storage is automatic
            MobileCapability::FileSystemShared => {
                Some("android.permission.READ_EXTERNAL_STORAGE")
            }
            MobileCapability::PushNotifications => Some("android.permission.POST_NOTIFICATIONS"),
            MobileCapability::LocalNotifications => Some("android.permission.POST_NOTIFICATIONS"),
            MobileCapability::BiometricAuth => Some("android.permission.USE_BIOMETRIC"),
            MobileCapability::BackgroundExecution => {
                Some("android.permission.FOREGROUND_SERVICE")
            }
            MobileCapability::DeepLinks => None, // Configured in manifest intent-filters
            MobileCapability::ShareSheet => None, // No permission needed
            MobileCapability::Haptics => Some("android.permission.VIBRATE"),
            MobileCapability::LocationWhenInUse => {
                Some("android.permission.ACCESS_FINE_LOCATION")
            }
            MobileCapability::LocationAlways => {
                Some("android.permission.ACCESS_BACKGROUND_LOCATION")
            }
            MobileCapability::MotionSensors => Some("android.permission.BODY_SENSORS"),
        }
    }

    /// Check if this capability is considered dangerous (requires special review).
    ///
    /// Dangerous capabilities can access sensitive user data or affect device
    /// behavior in significant ways.
    pub fn is_dangerous(&self) -> bool {
        matches!(
            self,
            MobileCapability::CameraCapture
                | MobileCapability::MicrophoneRecord
                | MobileCapability::PhotoLibraryRead
                | MobileCapability::LocationWhenInUse
                | MobileCapability::LocationAlways
                | MobileCapability::BackgroundExecution
                | MobileCapability::FileSystemShared
        )
    }

    /// Check if this capability requires a runtime permission prompt.
    ///
    /// On both iOS and Android, certain permissions must be requested at runtime
    /// with user consent, rather than just being declared in the manifest.
    pub fn requires_runtime_prompt(&self) -> bool {
        matches!(
            self,
            MobileCapability::CameraCapture
                | MobileCapability::MicrophoneRecord
                | MobileCapability::PhotoLibraryRead
                | MobileCapability::PhotoLibraryWrite
                | MobileCapability::LocationWhenInUse
                | MobileCapability::LocationAlways
                | MobileCapability::BiometricAuth
                | MobileCapability::PushNotifications
                | MobileCapability::LocalNotifications
                | MobileCapability::MotionSensors
        )
    }

    /// Get the string representation.
    pub fn as_str(&self) -> &'static str {
        match self {
            MobileCapability::CameraCapture => "camera.capture",
            MobileCapability::CameraFlash => "camera.flash",
            MobileCapability::PhotoLibraryRead => "photos.read",
            MobileCapability::PhotoLibraryWrite => "photos.write",
            MobileCapability::MicrophoneRecord => "microphone.record",
            MobileCapability::SecureStorage => "storage.secure",
            MobileCapability::FileSystemApp => "storage.app",
            MobileCapability::FileSystemShared => "storage.shared",
            MobileCapability::PushNotifications => "notifications.push",
            MobileCapability::LocalNotifications => "notifications.local",
            MobileCapability::BiometricAuth => "biometric.auth",
            MobileCapability::BackgroundExecution => "system.background",
            MobileCapability::DeepLinks => "system.deeplinks",
            MobileCapability::ShareSheet => "system.share",
            MobileCapability::Haptics => "system.haptics",
            MobileCapability::LocationWhenInUse => "location.when_in_use",
            MobileCapability::LocationAlways => "location.always",
            MobileCapability::MotionSensors => "sensors.motion",
        }
    }

    /// Get the category of this capability.
    pub fn category(&self) -> &'static str {
        match self {
            MobileCapability::CameraCapture | MobileCapability::CameraFlash => "camera",
            MobileCapability::PhotoLibraryRead | MobileCapability::PhotoLibraryWrite => "photos",
            MobileCapability::MicrophoneRecord => "microphone",
            MobileCapability::SecureStorage
            | MobileCapability::FileSystemApp
            | MobileCapability::FileSystemShared => "storage",
            MobileCapability::PushNotifications | MobileCapability::LocalNotifications => {
                "notifications"
            }
            MobileCapability::BiometricAuth => "biometric",
            MobileCapability::BackgroundExecution
            | MobileCapability::DeepLinks
            | MobileCapability::ShareSheet
            | MobileCapability::Haptics => "system",
            MobileCapability::LocationWhenInUse | MobileCapability::LocationAlways => "location",
            MobileCapability::MotionSensors => "sensors",
        }
    }

    /// Get a human-readable description of this capability.
    pub fn description(&self) -> &'static str {
        match self {
            MobileCapability::CameraCapture => "Capture photos and videos using the device camera",
            MobileCapability::CameraFlash => "Control the camera flash",
            MobileCapability::PhotoLibraryRead => "Read photos from the device photo library",
            MobileCapability::PhotoLibraryWrite => "Save photos to the device photo library",
            MobileCapability::MicrophoneRecord => "Record audio using the device microphone",
            MobileCapability::SecureStorage => {
                "Store sensitive data securely (Keychain/Keystore)"
            }
            MobileCapability::FileSystemApp => "Access the app's sandboxed file system",
            MobileCapability::FileSystemShared => "Access shared storage areas",
            MobileCapability::PushNotifications => "Receive push notifications",
            MobileCapability::LocalNotifications => "Schedule and display local notifications",
            MobileCapability::BiometricAuth => "Authenticate using biometrics",
            MobileCapability::BackgroundExecution => "Execute tasks in the background",
            MobileCapability::DeepLinks => "Handle deep links and universal links",
            MobileCapability::ShareSheet => "Show the system share sheet",
            MobileCapability::Haptics => "Trigger haptic feedback",
            MobileCapability::LocationWhenInUse => "Access location while app is in use",
            MobileCapability::LocationAlways => "Access location in the background",
            MobileCapability::MotionSensors => "Access motion sensors",
        }
    }

    /// Get all mobile capabilities.
    pub fn all() -> &'static [MobileCapability] {
        &[
            MobileCapability::CameraCapture,
            MobileCapability::CameraFlash,
            MobileCapability::PhotoLibraryRead,
            MobileCapability::PhotoLibraryWrite,
            MobileCapability::MicrophoneRecord,
            MobileCapability::SecureStorage,
            MobileCapability::FileSystemApp,
            MobileCapability::FileSystemShared,
            MobileCapability::PushNotifications,
            MobileCapability::LocalNotifications,
            MobileCapability::BiometricAuth,
            MobileCapability::BackgroundExecution,
            MobileCapability::DeepLinks,
            MobileCapability::ShareSheet,
            MobileCapability::Haptics,
            MobileCapability::LocationWhenInUse,
            MobileCapability::LocationAlways,
            MobileCapability::MotionSensors,
        ]
    }
}

impl FromStr for MobileCapability {
    type Err = PluginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "camera.capture" => Ok(MobileCapability::CameraCapture),
            "camera.flash" => Ok(MobileCapability::CameraFlash),
            "photos.read" => Ok(MobileCapability::PhotoLibraryRead),
            "photos.write" => Ok(MobileCapability::PhotoLibraryWrite),
            "microphone.record" => Ok(MobileCapability::MicrophoneRecord),
            "storage.secure" => Ok(MobileCapability::SecureStorage),
            "storage.app" => Ok(MobileCapability::FileSystemApp),
            "storage.shared" => Ok(MobileCapability::FileSystemShared),
            "notifications.push" => Ok(MobileCapability::PushNotifications),
            "notifications.local" => Ok(MobileCapability::LocalNotifications),
            "biometric.auth" => Ok(MobileCapability::BiometricAuth),
            "system.background" => Ok(MobileCapability::BackgroundExecution),
            "system.deeplinks" => Ok(MobileCapability::DeepLinks),
            "system.share" => Ok(MobileCapability::ShareSheet),
            "system.haptics" => Ok(MobileCapability::Haptics),
            "location.when_in_use" => Ok(MobileCapability::LocationWhenInUse),
            "location.always" => Ok(MobileCapability::LocationAlways),
            "sensors.motion" => Ok(MobileCapability::MotionSensors),
            _ => Err(PluginError::PermissionDenied(format!(
                "unknown mobile capability: {}",
                s
            ))),
        }
    }
}

impl std::fmt::Display for MobileCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A set of mobile capabilities for a plugin.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MobileCapabilitySet {
    /// Granted capabilities.
    capabilities: HashSet<MobileCapability>,
}

impl MobileCapabilitySet {
    /// Create a new empty capability set.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a capability to the set.
    pub fn add(&mut self, capability: MobileCapability) {
        self.capabilities.insert(capability);
    }

    /// Check if a capability is in the set.
    pub fn has(&self, capability: MobileCapability) -> bool {
        self.capabilities.contains(&capability)
    }

    /// Get all capabilities in the set.
    pub fn iter(&self) -> impl Iterator<Item = &MobileCapability> {
        self.capabilities.iter()
    }

    /// Check if the set is empty.
    pub fn is_empty(&self) -> bool {
        self.capabilities.is_empty()
    }

    /// Get the number of capabilities.
    pub fn len(&self) -> usize {
        self.capabilities.len()
    }

    /// Get all iOS permission keys needed.
    pub fn ios_permission_keys(&self) -> Vec<&'static str> {
        self.capabilities
            .iter()
            .filter_map(|c| c.ios_permission_key())
            .collect()
    }

    /// Get all Android permissions needed.
    pub fn android_permissions(&self) -> Vec<&'static str> {
        self.capabilities
            .iter()
            .filter_map(|c| c.android_permission())
            .collect()
    }

    /// Get all dangerous capabilities in the set.
    pub fn dangerous_capabilities(&self) -> Vec<MobileCapability> {
        self.capabilities
            .iter()
            .filter(|c| c.is_dangerous())
            .copied()
            .collect()
    }

    /// Get all capabilities requiring runtime prompts.
    pub fn runtime_prompt_capabilities(&self) -> Vec<MobileCapability> {
        self.capabilities
            .iter()
            .filter(|c| c.requires_runtime_prompt())
            .copied()
            .collect()
    }
}

/// Parse mobile capability strings into a list.
pub fn parse_mobile_capabilities(
    capabilities: &[String],
) -> PluginResult<Vec<MobileCapability>> {
    capabilities
        .iter()
        .map(|s| s.parse::<MobileCapability>())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_parsing() {
        assert_eq!(
            "camera.capture".parse::<MobileCapability>().unwrap(),
            MobileCapability::CameraCapture
        );
        assert_eq!(
            "location.always".parse::<MobileCapability>().unwrap(),
            MobileCapability::LocationAlways
        );
        assert!("invalid.cap".parse::<MobileCapability>().is_err());
    }

    #[test]
    fn test_ios_permission_keys() {
        assert_eq!(
            MobileCapability::CameraCapture.ios_permission_key(),
            Some("NSCameraUsageDescription")
        );
        assert_eq!(
            MobileCapability::LocationAlways.ios_permission_key(),
            Some("NSLocationAlwaysAndWhenInUseUsageDescription")
        );
        assert_eq!(MobileCapability::SecureStorage.ios_permission_key(), None);
    }

    #[test]
    fn test_android_permissions() {
        assert_eq!(
            MobileCapability::CameraCapture.android_permission(),
            Some("android.permission.CAMERA")
        );
        assert_eq!(
            MobileCapability::LocationAlways.android_permission(),
            Some("android.permission.ACCESS_BACKGROUND_LOCATION")
        );
        assert_eq!(MobileCapability::SecureStorage.android_permission(), None);
    }

    #[test]
    fn test_dangerous_capabilities() {
        assert!(MobileCapability::CameraCapture.is_dangerous());
        assert!(MobileCapability::LocationAlways.is_dangerous());
        assert!(!MobileCapability::Haptics.is_dangerous());
        assert!(!MobileCapability::ShareSheet.is_dangerous());
    }

    #[test]
    fn test_runtime_prompt() {
        assert!(MobileCapability::CameraCapture.requires_runtime_prompt());
        assert!(MobileCapability::BiometricAuth.requires_runtime_prompt());
        assert!(!MobileCapability::Haptics.requires_runtime_prompt());
        assert!(!MobileCapability::SecureStorage.requires_runtime_prompt());
    }

    #[test]
    fn test_capability_set() {
        let mut set = MobileCapabilitySet::new();
        set.add(MobileCapability::CameraCapture);
        set.add(MobileCapability::BiometricAuth);
        set.add(MobileCapability::Haptics);

        assert!(set.has(MobileCapability::CameraCapture));
        assert!(!set.has(MobileCapability::LocationAlways));
        assert_eq!(set.len(), 3);

        // Check iOS permissions
        let ios_perms = set.ios_permission_keys();
        assert!(ios_perms.contains(&"NSCameraUsageDescription"));
        assert!(ios_perms.contains(&"NSFaceIDUsageDescription"));

        // Check dangerous capabilities
        let dangerous = set.dangerous_capabilities();
        assert_eq!(dangerous.len(), 1);
        assert!(dangerous.contains(&MobileCapability::CameraCapture));
    }

    #[test]
    fn test_parse_capabilities() {
        let caps = vec![
            "camera.capture".to_string(),
            "storage.secure".to_string(),
        ];
        let parsed = parse_mobile_capabilities(&caps).unwrap();
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0], MobileCapability::CameraCapture);
        assert_eq!(parsed[1], MobileCapability::SecureStorage);
    }
}
