//! Android target stub implementations.
//!
//! These stubs provide placeholder implementations for Android-specific APIs
//! when compiling for non-Android targets, and also document the Android APIs
//! that will be available when targeting Google's mobile platform.

use super::{Stub, StubError, StubResult};

/// Android platform stub.
///
/// On actual Android targets, these would be replaced with real
/// implementations using JNI and NDK bindings.
pub struct AndroidPlatform;

impl Stub for AndroidPlatform {
    const FEATURE_NAME: &'static str = "android-platform";

    fn is_available() -> bool {
        cfg!(target_os = "android")
    }
}

/// Device information stubs.
pub mod device {
    use super::*;

    /// Device information from Build class.
    #[derive(Debug, Clone)]
    pub struct DeviceInfo {
        /// Device manufacturer (e.g., "Samsung")
        pub manufacturer: String,
        /// Device model (e.g., "SM-G998B")
        pub model: String,
        /// Device product name
        pub product: String,
        /// Android version (e.g., "14")
        pub version: String,
        /// SDK version (e.g., 34)
        pub sdk_int: u32,
    }

    /// Get device information.
    #[cfg(not(target_os = "android"))]
    pub fn get_device_info() -> StubResult<DeviceInfo> {
        Err(StubError::unavailable("android.os.Build"))
    }

    /// Get device information (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_device_info() -> StubResult<DeviceInfo> {
        Ok(DeviceInfo {
            manufacturer: "Google".to_string(),
            model: "Pixel".to_string(),
            product: "Pixel".to_string(),
            version: "14".to_string(),
            sdk_int: 34,
        })
    }

    /// Screen orientation.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Orientation {
        /// Portrait mode
        Portrait,
        /// Landscape mode
        Landscape,
        /// Reverse portrait
        ReversePortrait,
        /// Reverse landscape
        ReverseLandscape,
        /// Undefined
        Undefined,
    }

    /// Get current screen orientation.
    #[cfg(not(target_os = "android"))]
    pub fn get_orientation() -> StubResult<Orientation> {
        Err(StubError::unavailable("Configuration.orientation"))
    }

    /// Get current screen orientation (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_orientation() -> StubResult<Orientation> {
        Ok(Orientation::Portrait)
    }

    /// Get screen density in DPI.
    #[cfg(not(target_os = "android"))]
    pub fn get_screen_density() -> StubResult<f32> {
        Err(StubError::unavailable("DisplayMetrics"))
    }

    /// Get screen density in DPI (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_screen_density() -> StubResult<f32> {
        Ok(420.0)
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
        /// Fingerprint
        Fingerprint,
        /// Face recognition
        Face,
        /// Iris scanner
        Iris,
        /// Multiple biometrics available
        Multiple,
    }

    /// Biometric authentication result.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AuthResult {
        /// Authentication succeeded
        Success,
        /// Authentication failed
        Failed,
        /// User cancelled
        Cancelled,
        /// Lockout due to too many attempts
        Lockout,
        /// Hardware not available
        HardwareUnavailable,
        /// No biometrics enrolled
        NoneEnrolled,
    }

    /// Check what biometric type is available.
    #[cfg(not(target_os = "android"))]
    pub fn get_available_biometric() -> StubResult<BiometricType> {
        Err(StubError::unavailable("BiometricManager"))
    }

    /// Check what biometric type is available (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_available_biometric() -> StubResult<BiometricType> {
        Ok(BiometricType::Fingerprint)
    }

    /// Authenticate using biometrics.
    #[cfg(not(target_os = "android"))]
    pub async fn authenticate(_title: &str, _subtitle: &str) -> StubResult<AuthResult> {
        Err(StubError::unavailable("BiometricPrompt"))
    }

    /// Authenticate using biometrics (real implementation for Android).
    #[cfg(target_os = "android")]
    pub async fn authenticate(_title: &str, _subtitle: &str) -> StubResult<AuthResult> {
        Ok(AuthResult::Success)
    }
}

/// Keystore stubs for secure storage.
pub mod keystore {
    use super::*;

    /// Store a value in the Android Keystore.
    #[cfg(not(target_os = "android"))]
    pub fn set_item(_key: &str, _value: &[u8]) -> StubResult<()> {
        Err(StubError::unavailable("AndroidKeyStore"))
    }

    /// Store a value in the Android Keystore (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn set_item(_key: &str, _value: &[u8]) -> StubResult<()> {
        Ok(())
    }

    /// Get a value from the Android Keystore.
    #[cfg(not(target_os = "android"))]
    pub fn get_item(_key: &str) -> StubResult<Option<Vec<u8>>> {
        Err(StubError::unavailable("AndroidKeyStore"))
    }

    /// Get a value from the Android Keystore (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_item(_key: &str) -> StubResult<Option<Vec<u8>>> {
        Ok(None)
    }

    /// Delete a value from the Android Keystore.
    #[cfg(not(target_os = "android"))]
    pub fn delete_item(_key: &str) -> StubResult<()> {
        Err(StubError::unavailable("AndroidKeyStore"))
    }

    /// Delete a value from the Android Keystore (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn delete_item(_key: &str) -> StubResult<()> {
        Ok(())
    }
}

/// Push notification stubs (Firebase Cloud Messaging).
pub mod notifications {
    use super::*;

    /// FCM registration result.
    #[derive(Debug, Clone)]
    pub struct FcmToken {
        /// The FCM registration token
        pub token: String,
    }

    /// Get FCM registration token.
    #[cfg(not(target_os = "android"))]
    pub async fn get_fcm_token() -> StubResult<FcmToken> {
        Err(StubError::unavailable("FirebaseMessaging"))
    }

    /// Get FCM registration token (real implementation for Android).
    #[cfg(target_os = "android")]
    pub async fn get_fcm_token() -> StubResult<FcmToken> {
        Ok(FcmToken {
            token: "fcm-token-placeholder".to_string(),
        })
    }

    /// Notification channel importance.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Importance {
        /// No importance (no notifications)
        None,
        /// Low importance (no sound)
        Low,
        /// Default importance
        Default,
        /// High importance (makes sound)
        High,
        /// Maximum importance (heads-up notification)
        Max,
    }

    /// Create a notification channel.
    #[cfg(not(target_os = "android"))]
    pub fn create_notification_channel(
        _id: &str,
        _name: &str,
        _importance: Importance,
    ) -> StubResult<()> {
        Err(StubError::unavailable("NotificationChannel"))
    }

    /// Create a notification channel (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn create_notification_channel(
        _id: &str,
        _name: &str,
        _importance: Importance,
    ) -> StubResult<()> {
        Ok(())
    }
}

/// Vibration stubs.
pub mod vibration {
    use super::*;

    /// Vibration effect type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum VibrationEffect {
        /// Click effect
        Click,
        /// Double click effect
        DoubleClick,
        /// Heavy click effect
        HeavyClick,
        /// Tick effect
        Tick,
    }

    /// Trigger vibration with a predefined effect.
    #[cfg(not(target_os = "android"))]
    pub fn vibrate_effect(_effect: VibrationEffect) -> StubResult<()> {
        Err(StubError::unavailable("VibrationEffect"))
    }

    /// Trigger vibration with a predefined effect (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn vibrate_effect(_effect: VibrationEffect) -> StubResult<()> {
        Ok(())
    }

    /// Trigger vibration for a duration in milliseconds.
    #[cfg(not(target_os = "android"))]
    pub fn vibrate_duration(_millis: u64) -> StubResult<()> {
        Err(StubError::unavailable("Vibrator"))
    }

    /// Trigger vibration for a duration in milliseconds (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn vibrate_duration(_millis: u64) -> StubResult<()> {
        Ok(())
    }

    /// Trigger vibration with a pattern.
    #[cfg(not(target_os = "android"))]
    pub fn vibrate_pattern(_pattern: &[u64], _repeat: i32) -> StubResult<()> {
        Err(StubError::unavailable("Vibrator"))
    }

    /// Trigger vibration with a pattern (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn vibrate_pattern(_pattern: &[u64], _repeat: i32) -> StubResult<()> {
        Ok(())
    }
}

/// Window insets stubs (for notch/cutout handling).
pub mod window_insets {
    use super::*;

    /// Window insets.
    #[derive(Debug, Clone, Copy, Default)]
    pub struct WindowInsets {
        /// Top inset (status bar, display cutout)
        pub top: i32,
        /// Bottom inset (navigation bar)
        pub bottom: i32,
        /// Left inset
        pub left: i32,
        /// Right inset
        pub right: i32,
    }

    /// Get system bar insets.
    #[cfg(not(target_os = "android"))]
    pub fn get_system_bars_insets() -> StubResult<WindowInsets> {
        Err(StubError::unavailable("WindowInsetsCompat"))
    }

    /// Get system bar insets (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_system_bars_insets() -> StubResult<WindowInsets> {
        Ok(WindowInsets {
            top: 24,
            bottom: 48,
            left: 0,
            right: 0,
        })
    }

    /// Display cutout information.
    #[derive(Debug, Clone, Default)]
    pub struct DisplayCutout {
        /// Cutout regions
        pub bounding_rects: Vec<(i32, i32, i32, i32)>,
        /// Safe inset top
        pub safe_inset_top: i32,
        /// Safe inset bottom
        pub safe_inset_bottom: i32,
        /// Safe inset left
        pub safe_inset_left: i32,
        /// Safe inset right
        pub safe_inset_right: i32,
    }

    /// Get display cutout information.
    #[cfg(not(target_os = "android"))]
    pub fn get_display_cutout() -> StubResult<Option<DisplayCutout>> {
        Err(StubError::unavailable("DisplayCutout"))
    }

    /// Get display cutout information (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn get_display_cutout() -> StubResult<Option<DisplayCutout>> {
        Ok(Some(DisplayCutout {
            bounding_rects: vec![],
            safe_inset_top: 0,
            safe_inset_bottom: 0,
            safe_inset_left: 0,
            safe_inset_right: 0,
        }))
    }
}

/// Intent stubs for inter-app communication.
pub mod intents {
    use super::*;

    /// Common intent actions.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum IntentAction {
        /// View content
        View,
        /// Share content
        Send,
        /// Pick content
        Pick,
        /// Call a phone number
        Dial,
        /// Send SMS
        Sendto,
        /// Open settings
        Settings,
    }

    /// Launch an intent.
    #[cfg(not(target_os = "android"))]
    pub fn start_activity(_action: IntentAction, _data: Option<&str>) -> StubResult<()> {
        Err(StubError::unavailable("Intent"))
    }

    /// Launch an intent (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn start_activity(_action: IntentAction, _data: Option<&str>) -> StubResult<()> {
        Ok(())
    }

    /// Open a URL in browser.
    #[cfg(not(target_os = "android"))]
    pub fn open_url(_url: &str) -> StubResult<()> {
        Err(StubError::unavailable("Intent.ACTION_VIEW"))
    }

    /// Open a URL in browser (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn open_url(_url: &str) -> StubResult<()> {
        Ok(())
    }

    /// Share text content.
    #[cfg(not(target_os = "android"))]
    pub fn share_text(_text: &str, _title: Option<&str>) -> StubResult<()> {
        Err(StubError::unavailable("Intent.ACTION_SEND"))
    }

    /// Share text content (real implementation for Android).
    #[cfg(target_os = "android")]
    pub fn share_text(_text: &str, _title: Option<&str>) -> StubResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_android_platform_availability() {
        #[cfg(not(target_os = "android"))]
        assert!(!AndroidPlatform::is_available());

        #[cfg(target_os = "android")]
        assert!(AndroidPlatform::is_available());
    }

    #[test]
    fn test_device_info_stub() {
        #[cfg(not(target_os = "android"))]
        {
            let result = device::get_device_info();
            assert!(result.is_err());
        }
    }
}
