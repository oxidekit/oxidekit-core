//! Plugin backend traits for mobile platforms.
//!
//! These traits define the interface that platform-specific implementations
//! must provide. Plugins use these abstractions to interact with mobile
//! platform features without direct platform dependencies.

use serde::{Deserialize, Serialize};
use std::path::Path;
use std::time::Duration;

use crate::error::PluginResult;

// =============================================================================
// Camera Backend
// =============================================================================

/// Configuration for capturing a photo.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhotoConfig {
    /// Image quality (0.0 to 1.0).
    pub quality: f32,
    /// Which camera to use.
    pub camera: CameraPosition,
    /// Flash mode setting.
    pub flash: FlashMode,
    /// Maximum width in pixels (optional, for resizing).
    pub max_width: Option<u32>,
    /// Maximum height in pixels (optional, for resizing).
    pub max_height: Option<u32>,
}

impl Default for PhotoConfig {
    fn default() -> Self {
        Self {
            quality: 0.85,
            camera: CameraPosition::Back,
            flash: FlashMode::Auto,
            max_width: None,
            max_height: None,
        }
    }
}

/// Configuration for capturing video.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Which camera to use.
    pub camera: CameraPosition,
    /// Flash/torch mode setting.
    pub flash: FlashMode,
    /// Maximum duration in seconds.
    pub max_duration: Option<Duration>,
    /// Video quality preset.
    pub quality: VideoQuality,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            camera: CameraPosition::Back,
            flash: FlashMode::Off,
            max_duration: None,
            quality: VideoQuality::High,
        }
    }
}

/// Result of capturing a photo.
#[derive(Debug, Clone)]
pub struct PhotoResult {
    /// Raw image data (JPEG or PNG).
    pub data: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
    /// Image metadata.
    pub metadata: PhotoMetadata,
}

/// Metadata associated with a captured photo.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PhotoMetadata {
    /// MIME type of the image.
    pub mime_type: String,
    /// GPS latitude (if available).
    pub latitude: Option<f64>,
    /// GPS longitude (if available).
    pub longitude: Option<f64>,
    /// Capture timestamp (Unix timestamp).
    pub timestamp: Option<i64>,
    /// Device orientation when captured.
    pub orientation: Option<u32>,
}

/// Result of capturing video.
#[derive(Debug, Clone)]
pub struct VideoResult {
    /// Path to the captured video file.
    pub path: std::path::PathBuf,
    /// Video duration in seconds.
    pub duration: Duration,
    /// Video width in pixels.
    pub width: u32,
    /// Video height in pixels.
    pub height: u32,
    /// File size in bytes.
    pub size: u64,
}

/// Camera position selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CameraPosition {
    /// Front-facing camera.
    Front,
    /// Back/rear camera.
    Back,
}

/// Flash mode setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FlashMode {
    /// Automatic flash based on lighting conditions.
    Auto,
    /// Flash always on.
    On,
    /// Flash always off.
    Off,
    /// Torch mode (continuous light for video).
    Torch,
}

/// Video quality preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VideoQuality {
    /// Low quality (smaller file size).
    Low,
    /// Medium quality.
    Medium,
    /// High quality.
    High,
    /// Maximum quality available.
    Max,
}

/// Information about an available camera.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CameraInfo {
    /// Camera identifier.
    pub id: String,
    /// Camera position (front/back).
    pub position: CameraPosition,
    /// Whether flash is available.
    pub has_flash: bool,
    /// Whether video recording is supported.
    pub supports_video: bool,
    /// Maximum photo resolution.
    pub max_resolution: Option<(u32, u32)>,
}

/// Backend trait for camera plugins.
///
/// Platform implementations (iOS/Android) must implement this trait
/// to provide camera functionality.
pub trait CameraBackend: Send + Sync {
    /// Capture a photo with the given configuration.
    fn capture_photo(&self, config: PhotoConfig) -> PluginResult<PhotoResult>;

    /// Capture video with the given configuration.
    fn capture_video(&self, config: VideoConfig) -> PluginResult<VideoResult>;

    /// Check if flash is available on the current camera.
    fn has_flash(&self) -> bool;

    /// Get information about available cameras.
    fn available_cameras(&self) -> Vec<CameraInfo>;
}

// =============================================================================
// Secure Storage Backend
// =============================================================================

/// Configuration for secure storage operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecureStoreConfig {
    /// Require biometric authentication to access.
    pub require_biometric: bool,
    /// Make item accessible only when device is unlocked.
    pub accessible_when_unlocked_only: bool,
    /// Service name / account group for the item.
    pub service_name: Option<String>,
}

/// Backend trait for secure storage (Keychain/Keystore).
///
/// Provides access to the platform's secure credential storage.
pub trait SecureStorageBackend: Send + Sync {
    /// Store data securely with the given key.
    fn store(&self, key: &str, data: &[u8], config: SecureStoreConfig) -> PluginResult<()>;

    /// Retrieve data for the given key.
    fn retrieve(&self, key: &str) -> PluginResult<Option<Vec<u8>>>;

    /// Delete data for the given key.
    fn delete(&self, key: &str) -> PluginResult<()>;

    /// Check if data exists for the given key.
    fn exists(&self, key: &str) -> PluginResult<bool>;

    /// List all keys stored by this app.
    fn list_keys(&self) -> PluginResult<Vec<String>>;
}

// =============================================================================
// Notification Backend
// =============================================================================

/// Permission status for notifications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NotificationPermission {
    /// User has granted permission.
    Authorized,
    /// User has denied permission.
    Denied,
    /// Permission has not been requested yet.
    NotDetermined,
    /// Permission is provisional (iOS silent notifications).
    Provisional,
}

/// A local notification to be scheduled.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalNotification {
    /// Unique identifier for this notification.
    pub id: String,
    /// Notification title.
    pub title: String,
    /// Notification body text.
    pub body: String,
    /// Optional subtitle (iOS).
    pub subtitle: Option<String>,
    /// Badge number to set (0 to clear).
    pub badge: Option<u32>,
    /// Sound to play (None for default, Some("") for silent).
    pub sound: Option<String>,
    /// Custom data payload.
    pub data: Option<serde_json::Value>,
    /// When to trigger the notification.
    pub trigger: NotificationTrigger,
}

/// Trigger condition for a local notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NotificationTrigger {
    /// Show immediately.
    Immediate,
    /// Show at a specific time (Unix timestamp in seconds).
    Scheduled(i64),
    /// Repeat at interval (in seconds).
    Interval(u64),
    /// Location-based trigger.
    Location {
        /// Latitude of the trigger location.
        latitude: f64,
        /// Longitude of the trigger location.
        longitude: f64,
        /// Radius in meters around the location.
        radius: f64,
        /// Trigger when entering the region.
        on_enter: bool,
        /// Trigger when exiting the region.
        on_exit: bool,
    },
}

/// Backend trait for push notifications.
pub trait NotificationBackend: Send + Sync {
    /// Request permission to send notifications.
    fn request_permission(&self) -> PluginResult<NotificationPermission>;

    /// Get the current permission status.
    fn get_permission_status(&self) -> PluginResult<NotificationPermission>;

    /// Get the device token for push notifications.
    fn get_device_token(&self) -> PluginResult<Option<String>>;

    /// Schedule a local notification.
    fn schedule_local(&self, notification: LocalNotification) -> PluginResult<String>;

    /// Cancel a scheduled local notification.
    fn cancel_local(&self, id: &str) -> PluginResult<()>;

    /// Cancel all pending notifications.
    fn cancel_all_local(&self) -> PluginResult<()>;

    /// Get all pending notification IDs.
    fn get_pending_notifications(&self) -> PluginResult<Vec<String>>;
}

// =============================================================================
// Biometric Backend
// =============================================================================

/// Type of biometric authentication available.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BiometricType {
    /// No biometric authentication available.
    #[default]
    None,
    /// TouchID (iOS fingerprint).
    TouchId,
    /// FaceID (iOS face recognition).
    FaceId,
    /// Generic fingerprint sensor (Android).
    Fingerprint,
    /// Iris scanner (some Android devices).
    Iris,
    /// Face recognition (Android).
    Face,
}

/// Result of a biometric authentication attempt.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiometricResult {
    /// Whether authentication succeeded.
    pub success: bool,
    /// Error if authentication failed.
    pub error: Option<BiometricError>,
}

/// Errors that can occur during biometric authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiometricError {
    /// User cancelled the authentication.
    Cancelled,
    /// Too many failed attempts.
    Lockout,
    /// Biometric not enrolled.
    NotEnrolled,
    /// Biometric not available on device.
    NotAvailable,
    /// User chose to use passcode/password instead.
    Fallback,
    /// System cancelled authentication.
    SystemCancel,
    /// Other error with message.
    Other(String),
}

impl std::fmt::Display for BiometricError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiometricError::Cancelled => write!(f, "User cancelled authentication"),
            BiometricError::Lockout => write!(f, "Too many failed attempts"),
            BiometricError::NotEnrolled => write!(f, "Biometric not enrolled"),
            BiometricError::NotAvailable => write!(f, "Biometric not available"),
            BiometricError::Fallback => write!(f, "User chose fallback authentication"),
            BiometricError::SystemCancel => write!(f, "System cancelled authentication"),
            BiometricError::Other(msg) => write!(f, "{}", msg),
        }
    }
}

impl std::error::Error for BiometricError {}

/// Backend trait for biometric authentication.
pub trait BiometricBackend: Send + Sync {
    /// Check if biometric authentication is available.
    fn is_available(&self) -> bool;

    /// Get the type of biometric authentication available.
    fn biometric_type(&self) -> BiometricType;

    /// Authenticate the user with biometrics.
    ///
    /// # Arguments
    /// * `reason` - The reason to display to the user (e.g., "Access your account")
    fn authenticate(&self, reason: &str) -> PluginResult<BiometricResult>;

    /// Check if biometrics can be used (enrolled and not locked out).
    fn can_authenticate(&self) -> bool;
}

// =============================================================================
// Share Backend
// =============================================================================

/// Result of a share operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShareResult {
    /// Whether the share was completed (not cancelled).
    pub completed: bool,
    /// The activity type selected (e.g., "com.apple.UIKit.activity.CopyToPasteboard").
    pub activity_type: Option<String>,
}

/// Backend trait for share sheet functionality.
pub trait ShareBackend: Send + Sync {
    /// Share text content.
    fn share_text(&self, text: &str) -> PluginResult<ShareResult>;

    /// Share a URL.
    fn share_url(&self, url: &str) -> PluginResult<ShareResult>;

    /// Share a file.
    fn share_file(&self, path: &Path, mime_type: &str) -> PluginResult<ShareResult>;

    /// Share image data.
    fn share_image(&self, data: &[u8]) -> PluginResult<ShareResult>;

    /// Share multiple items.
    fn share_multiple(&self, items: Vec<ShareItem>) -> PluginResult<ShareResult>;
}

/// An item to share via the share sheet.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShareItem {
    /// Plain text.
    Text(String),
    /// URL string.
    Url(String),
    /// File path with MIME type.
    File {
        /// Path to the file.
        path: String,
        /// MIME type of the file.
        mime_type: String,
    },
    /// Image data.
    Image(Vec<u8>),
}

// =============================================================================
// Deep Link Backend
// =============================================================================

/// Event emitted when a deep link is received.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkEvent {
    /// The full URL that was opened.
    pub url: String,
    /// The source app (if known).
    pub source: Option<String>,
    /// Parsed URL scheme.
    pub scheme: String,
    /// Parsed URL host.
    pub host: Option<String>,
    /// Parsed URL path.
    pub path: String,
    /// Parsed query parameters.
    pub query: std::collections::HashMap<String, String>,
}

/// An action to take based on a deep link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepLinkAction {
    /// Route/screen to navigate to.
    pub route: String,
    /// Parameters to pass.
    pub params: std::collections::HashMap<String, String>,
}

/// Backend trait for deep links / universal links.
pub trait DeepLinkBackend: Send + Sync {
    /// Register a URL scheme for the app.
    fn register_scheme(&self, scheme: &str) -> PluginResult<()>;

    /// Parse and handle a URL, returning an action if matched.
    fn handle_url(&self, url: &str) -> Option<DeepLinkAction>;

    /// Register a callback for deep link events.
    fn on_deep_link(&self, callback: Box<dyn Fn(DeepLinkEvent) + Send + Sync>);

    /// Get the URL that launched the app (if any).
    fn get_initial_url(&self) -> PluginResult<Option<String>>;
}

// =============================================================================
// Haptics Backend
// =============================================================================

/// Type of haptic feedback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HapticType {
    /// Light impact feedback.
    ImpactLight,
    /// Medium impact feedback.
    ImpactMedium,
    /// Heavy impact feedback.
    ImpactHeavy,
    /// Selection changed feedback.
    Selection,
    /// Success notification feedback.
    NotificationSuccess,
    /// Warning notification feedback.
    NotificationWarning,
    /// Error notification feedback.
    NotificationError,
}

/// Backend trait for haptic feedback.
pub trait HapticsBackend: Send + Sync {
    /// Check if haptics are available on this device.
    fn is_available(&self) -> bool;

    /// Trigger haptic feedback.
    fn trigger(&self, haptic_type: HapticType) -> PluginResult<()>;

    /// Trigger a custom vibration pattern (Android).
    fn vibrate(&self, pattern: &[u64]) -> PluginResult<()>;
}

// =============================================================================
// Mock Implementations for Testing
// =============================================================================

#[cfg(test)]
/// Mock implementations of mobile backends for testing.
pub mod mock {
    use super::*;
    use std::sync::{Arc, Mutex};

    /// Mock camera backend for testing.
    #[derive(Default)]
    pub struct MockCameraBackend {
        /// Counter for photos captured.
        pub photos_captured: Arc<Mutex<u32>>,
        /// Counter for videos captured.
        pub videos_captured: Arc<Mutex<u32>>,
    }

    impl CameraBackend for MockCameraBackend {
        fn capture_photo(&self, _config: PhotoConfig) -> PluginResult<PhotoResult> {
            *self.photos_captured.lock().unwrap() += 1;
            Ok(PhotoResult {
                data: vec![0xFF, 0xD8, 0xFF, 0xE0], // JPEG magic bytes
                width: 1920,
                height: 1080,
                metadata: PhotoMetadata {
                    mime_type: "image/jpeg".to_string(),
                    ..Default::default()
                },
            })
        }

        fn capture_video(&self, _config: VideoConfig) -> PluginResult<VideoResult> {
            *self.videos_captured.lock().unwrap() += 1;
            Ok(VideoResult {
                path: std::path::PathBuf::from("/tmp/mock_video.mp4"),
                duration: Duration::from_secs(10),
                width: 1920,
                height: 1080,
                size: 1024 * 1024,
            })
        }

        fn has_flash(&self) -> bool {
            true
        }

        fn available_cameras(&self) -> Vec<CameraInfo> {
            vec![
                CameraInfo {
                    id: "back".to_string(),
                    position: CameraPosition::Back,
                    has_flash: true,
                    supports_video: true,
                    max_resolution: Some((4032, 3024)),
                },
                CameraInfo {
                    id: "front".to_string(),
                    position: CameraPosition::Front,
                    has_flash: false,
                    supports_video: true,
                    max_resolution: Some((1920, 1080)),
                },
            ]
        }
    }

    /// Mock secure storage backend for testing.
    #[derive(Default)]
    pub struct MockSecureStorageBackend {
        storage: Arc<Mutex<std::collections::HashMap<String, Vec<u8>>>>,
    }

    impl SecureStorageBackend for MockSecureStorageBackend {
        fn store(
            &self,
            key: &str,
            data: &[u8],
            _config: SecureStoreConfig,
        ) -> PluginResult<()> {
            self.storage
                .lock()
                .unwrap()
                .insert(key.to_string(), data.to_vec());
            Ok(())
        }

        fn retrieve(&self, key: &str) -> PluginResult<Option<Vec<u8>>> {
            Ok(self.storage.lock().unwrap().get(key).cloned())
        }

        fn delete(&self, key: &str) -> PluginResult<()> {
            self.storage.lock().unwrap().remove(key);
            Ok(())
        }

        fn exists(&self, key: &str) -> PluginResult<bool> {
            Ok(self.storage.lock().unwrap().contains_key(key))
        }

        fn list_keys(&self) -> PluginResult<Vec<String>> {
            Ok(self.storage.lock().unwrap().keys().cloned().collect())
        }
    }

    /// Mock biometric backend for testing.
    #[derive(Default)]
    pub struct MockBiometricBackend {
        /// Whether biometric authentication is available.
        pub available: bool,
        /// The type of biometric authentication.
        pub biometric_type: BiometricType,
        /// Whether authentication should succeed.
        pub should_succeed: bool,
    }

    impl BiometricBackend for MockBiometricBackend {
        fn is_available(&self) -> bool {
            self.available
        }

        fn biometric_type(&self) -> BiometricType {
            self.biometric_type
        }

        fn authenticate(&self, _reason: &str) -> PluginResult<BiometricResult> {
            Ok(BiometricResult {
                success: self.should_succeed,
                error: if self.should_succeed {
                    None
                } else {
                    Some(BiometricError::Cancelled)
                },
            })
        }

        fn can_authenticate(&self) -> bool {
            self.available
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mock::*;

    #[test]
    fn test_photo_config_default() {
        let config = PhotoConfig::default();
        assert_eq!(config.quality, 0.85);
        assert_eq!(config.camera, CameraPosition::Back);
        assert_eq!(config.flash, FlashMode::Auto);
    }

    #[test]
    fn test_mock_camera_capture() {
        let backend = MockCameraBackend::default();
        let result = backend.capture_photo(PhotoConfig::default()).unwrap();

        assert_eq!(result.width, 1920);
        assert_eq!(result.height, 1080);
        assert!(!result.data.is_empty());
        assert_eq!(*backend.photos_captured.lock().unwrap(), 1);
    }

    #[test]
    fn test_mock_secure_storage() {
        let backend = MockSecureStorageBackend::default();

        // Store data
        backend
            .store("test_key", b"secret_data", SecureStoreConfig::default())
            .unwrap();

        // Verify it exists
        assert!(backend.exists("test_key").unwrap());

        // Retrieve data
        let data = backend.retrieve("test_key").unwrap();
        assert_eq!(data, Some(b"secret_data".to_vec()));

        // Delete and verify
        backend.delete("test_key").unwrap();
        assert!(!backend.exists("test_key").unwrap());
    }

    #[test]
    fn test_mock_biometric() {
        let backend = MockBiometricBackend {
            available: true,
            biometric_type: BiometricType::FaceId,
            should_succeed: true,
        };

        assert!(backend.is_available());
        assert_eq!(backend.biometric_type(), BiometricType::FaceId);

        let result = backend.authenticate("Test reason").unwrap();
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_notification_trigger_serialization() {
        let trigger = NotificationTrigger::Scheduled(1700000000);
        let json = serde_json::to_string(&trigger).unwrap();
        assert!(json.contains("scheduled"));

        let interval = NotificationTrigger::Interval(3600);
        let json = serde_json::to_string(&interval).unwrap();
        assert!(json.contains("interval"));
    }

    #[test]
    fn test_haptic_types() {
        let haptic = HapticType::ImpactMedium;
        let json = serde_json::to_string(&haptic).unwrap();
        assert_eq!(json, "\"impact_medium\"");
    }

    #[test]
    fn test_deep_link_event() {
        let mut query = std::collections::HashMap::new();
        query.insert("id".to_string(), "123".to_string());

        let event = DeepLinkEvent {
            url: "myapp://product?id=123".to_string(),
            source: Some("safari".to_string()),
            scheme: "myapp".to_string(),
            host: Some("product".to_string()),
            path: "".to_string(),
            query,
        };

        assert_eq!(event.scheme, "myapp");
        assert_eq!(event.query.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_share_item_variants() {
        let text_item = ShareItem::Text("Hello".to_string());
        let url_item = ShareItem::Url("https://example.com".to_string());
        let file_item = ShareItem::File {
            path: "/path/to/file.pdf".to_string(),
            mime_type: "application/pdf".to_string(),
        };

        // Just verify they serialize correctly
        assert!(serde_json::to_string(&text_item).is_ok());
        assert!(serde_json::to_string(&url_item).is_ok());
        assert!(serde_json::to_string(&file_item).is_ok());
    }
}
