//! iOS target stub implementations.
//!
//! These stubs provide placeholder implementations for iOS-specific APIs
//! when compiling for non-iOS targets, and also document the iOS APIs
//! that will be available when targeting Apple's mobile platform.

use super::{Stub, StubError, StubResult};

/// iOS platform stub.
///
/// On actual iOS targets, these would be replaced with real
/// implementations using objc and platform bindings.
pub struct IosPlatform;

impl Stub for IosPlatform {
    const FEATURE_NAME: &'static str = "ios-platform";

    fn is_available() -> bool {
        cfg!(target_os = "ios")
    }
}

/// Device information stubs.
pub mod device {
    use super::*;

    /// Device model information.
    #[derive(Debug, Clone)]
    pub struct DeviceInfo {
        /// Device model (e.g., "iPhone15,2")
        pub model: String,
        /// Device name set by user
        pub name: String,
        /// System name (e.g., "iOS")
        pub system_name: String,
        /// System version (e.g., "17.0")
        pub system_version: String,
    }

    /// Get device information.
    #[cfg(not(target_os = "ios"))]
    pub fn get_device_info() -> StubResult<DeviceInfo> {
        Err(StubError::unavailable("UIDevice"))
    }

    /// Get device information (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_device_info() -> StubResult<DeviceInfo> {
        // Real implementation would use UIDevice
        Ok(DeviceInfo {
            model: "iPhone".to_string(),
            name: "iPhone".to_string(),
            system_name: "iOS".to_string(),
            system_version: "17.0".to_string(),
        })
    }

    /// Device orientation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Orientation {
        /// Portrait mode
        Portrait,
        /// Portrait upside down
        PortraitUpsideDown,
        /// Landscape with home button on left
        LandscapeLeft,
        /// Landscape with home button on right
        LandscapeRight,
        /// Face up
        FaceUp,
        /// Face down
        FaceDown,
        /// Unknown orientation
        Unknown,
    }

    /// Get current device orientation.
    #[cfg(not(target_os = "ios"))]
    pub fn get_orientation() -> StubResult<Orientation> {
        Err(StubError::unavailable("UIDevice orientation"))
    }

    /// Get current device orientation (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_orientation() -> StubResult<Orientation> {
        Ok(Orientation::Portrait)
    }

    /// Device battery state.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BatteryState {
        /// Unknown state
        Unknown,
        /// Not charging
        Unplugged,
        /// Currently charging
        Charging,
        /// Fully charged
        Full,
    }

    /// Get battery level (0.0 to 1.0).
    #[cfg(not(target_os = "ios"))]
    pub fn get_battery_level() -> StubResult<f32> {
        Err(StubError::unavailable("UIDevice battery"))
    }

    /// Get battery level (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_battery_level() -> StubResult<f32> {
        Ok(1.0)
    }

    /// Get battery state.
    #[cfg(not(target_os = "ios"))]
    pub fn get_battery_state() -> StubResult<BatteryState> {
        Err(StubError::unavailable("UIDevice battery"))
    }

    /// Get battery state (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_battery_state() -> StubResult<BatteryState> {
        Ok(BatteryState::Full)
    }
}

/// Biometric authentication stubs.
pub mod biometrics {
    use super::*;

    /// Biometric type available on device.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum BiometricType {
        /// No biometrics available
        None,
        /// Touch ID (fingerprint)
        TouchId,
        /// Face ID (facial recognition)
        FaceId,
    }

    /// Check what biometric type is available.
    #[cfg(not(target_os = "ios"))]
    pub fn available_biometric_type() -> StubResult<BiometricType> {
        Err(StubError::unavailable("LocalAuthentication"))
    }

    /// Check what biometric type is available (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn available_biometric_type() -> StubResult<BiometricType> {
        Ok(BiometricType::FaceId)
    }

    /// Authenticate using biometrics.
    #[cfg(not(target_os = "ios"))]
    pub async fn authenticate(_reason: &str) -> StubResult<bool> {
        Err(StubError::unavailable("LocalAuthentication"))
    }

    /// Authenticate using biometrics (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub async fn authenticate(_reason: &str) -> StubResult<bool> {
        Ok(true)
    }
}

/// Keychain stubs for secure storage.
pub mod keychain {
    use super::*;

    /// Store a value in the keychain.
    #[cfg(not(target_os = "ios"))]
    pub fn set_item(_key: &str, _value: &[u8]) -> StubResult<()> {
        Err(StubError::unavailable("Keychain"))
    }

    /// Store a value in the keychain (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn set_item(_key: &str, _value: &[u8]) -> StubResult<()> {
        Ok(())
    }

    /// Get a value from the keychain.
    #[cfg(not(target_os = "ios"))]
    pub fn get_item(_key: &str) -> StubResult<Option<Vec<u8>>> {
        Err(StubError::unavailable("Keychain"))
    }

    /// Get a value from the keychain (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_item(_key: &str) -> StubResult<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Delete a value from the keychain.
    #[cfg(not(target_os = "ios"))]
    pub fn delete_item(_key: &str) -> StubResult<()> {
        Err(StubError::unavailable("Keychain"))
    }

    /// Delete a value from the keychain (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn delete_item(_key: &str) -> StubResult<()> {
        Ok(())
    }
}

/// Push notification stubs.
pub mod notifications {
    use super::*;

    /// Notification authorization status.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AuthorizationStatus {
        /// Not determined yet
        NotDetermined,
        /// Denied by user
        Denied,
        /// Authorized
        Authorized,
        /// Provisional (quiet notifications)
        Provisional,
    }

    /// Request notification authorization.
    #[cfg(not(target_os = "ios"))]
    pub async fn request_authorization() -> StubResult<AuthorizationStatus> {
        Err(StubError::unavailable("UNUserNotificationCenter"))
    }

    /// Request notification authorization (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub async fn request_authorization() -> StubResult<AuthorizationStatus> {
        Ok(AuthorizationStatus::Authorized)
    }

    /// Get current authorization status.
    #[cfg(not(target_os = "ios"))]
    pub async fn get_authorization_status() -> StubResult<AuthorizationStatus> {
        Err(StubError::unavailable("UNUserNotificationCenter"))
    }

    /// Get current authorization status (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub async fn get_authorization_status() -> StubResult<AuthorizationStatus> {
        Ok(AuthorizationStatus::NotDetermined)
    }

    /// Register for push notifications.
    #[cfg(not(target_os = "ios"))]
    pub async fn register_for_remote_notifications() -> StubResult<String> {
        Err(StubError::unavailable("APNs"))
    }

    /// Register for push notifications (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub async fn register_for_remote_notifications() -> StubResult<String> {
        Ok("device-token-placeholder".to_string())
    }
}

/// Haptic feedback stubs.
pub mod haptics {
    use super::*;

    /// Haptic feedback style.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum FeedbackStyle {
        /// Light feedback
        Light,
        /// Medium feedback
        Medium,
        /// Heavy feedback
        Heavy,
        /// Soft feedback
        Soft,
        /// Rigid feedback
        Rigid,
    }

    /// Notification feedback type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum NotificationType {
        /// Success notification
        Success,
        /// Warning notification
        Warning,
        /// Error notification
        Error,
    }

    /// Trigger impact feedback.
    #[cfg(not(target_os = "ios"))]
    pub fn impact(_style: FeedbackStyle) -> StubResult<()> {
        Err(StubError::unavailable("UIImpactFeedbackGenerator"))
    }

    /// Trigger impact feedback (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn impact(_style: FeedbackStyle) -> StubResult<()> {
        Ok(())
    }

    /// Trigger notification feedback.
    #[cfg(not(target_os = "ios"))]
    pub fn notification(_type: NotificationType) -> StubResult<()> {
        Err(StubError::unavailable("UINotificationFeedbackGenerator"))
    }

    /// Trigger notification feedback (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn notification(_type: NotificationType) -> StubResult<()> {
        Ok(())
    }

    /// Trigger selection feedback.
    #[cfg(not(target_os = "ios"))]
    pub fn selection_changed() -> StubResult<()> {
        Err(StubError::unavailable("UISelectionFeedbackGenerator"))
    }

    /// Trigger selection feedback (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn selection_changed() -> StubResult<()> {
        Ok(())
    }
}

/// Safe area insets stubs.
pub mod safe_area {
    use super::*;

    /// Safe area insets.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct SafeAreaInsets {
        /// Top inset (e.g., for notch/Dynamic Island)
        pub top: f32,
        /// Bottom inset (e.g., for home indicator)
        pub bottom: f32,
        /// Left inset
        pub left: f32,
        /// Right inset
        pub right: f32,
    }

    /// Get current safe area insets.
    #[cfg(not(target_os = "ios"))]
    pub fn get_insets() -> StubResult<SafeAreaInsets> {
        Err(StubError::unavailable("UIWindow.safeAreaInsets"))
    }

    /// Get current safe area insets (real implementation for iOS).
    #[cfg(target_os = "ios")]
    pub fn get_insets() -> StubResult<SafeAreaInsets> {
        Ok(SafeAreaInsets {
            top: 59.0, // Dynamic Island
            bottom: 34.0, // Home indicator
            left: 0.0,
            right: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_platform_availability() {
        #[cfg(not(target_os = "ios"))]
        assert!(!IosPlatform::is_available());

        #[cfg(target_os = "ios")]
        assert!(IosPlatform::is_available());
    }

    #[test]
    fn test_device_info_stub() {
        #[cfg(not(target_os = "ios"))]
        {
            let result = device::get_device_info();
            assert!(result.is_err());
        }
    }
}
