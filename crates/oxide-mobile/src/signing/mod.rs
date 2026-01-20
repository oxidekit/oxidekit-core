//! Code signing abstractions for mobile platforms.
//!
//! Provides unified interfaces for code signing on iOS and Android platforms.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::target::MobilePlatform;

/// Code signing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SigningConfig {
    /// Platform for signing.
    pub platform: MobilePlatform,
    /// Signing identity.
    pub identity: SigningIdentity,
    /// Additional signing options.
    pub options: SigningOptions,
}

impl SigningConfig {
    /// Create a new signing configuration for iOS.
    pub fn ios(identity: SigningIdentity) -> Self {
        Self {
            platform: MobilePlatform::Ios,
            identity,
            options: SigningOptions::default(),
        }
    }

    /// Create a new signing configuration for Android.
    pub fn android(identity: SigningIdentity) -> Self {
        Self {
            platform: MobilePlatform::Android,
            identity,
            options: SigningOptions::default(),
        }
    }

    /// Set signing options.
    pub fn with_options(mut self, options: SigningOptions) -> Self {
        self.options = options;
        self
    }

    /// Validate the signing configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        match self.platform {
            MobilePlatform::Ios => {
                if let SigningIdentity::Ios(ref ios) = self.identity {
                    if ios.certificate_name.is_empty() {
                        errors.push("iOS signing requires a certificate name".into());
                    }
                } else {
                    errors.push("iOS platform requires iOS signing identity".into());
                }
            }
            MobilePlatform::Android => {
                if let SigningIdentity::Android(ref android) = self.identity {
                    if android.keystore_path.as_os_str().is_empty() {
                        errors.push("Android signing requires a keystore path".into());
                    }
                    if android.key_alias.is_empty() {
                        errors.push("Android signing requires a key alias".into());
                    }
                } else {
                    errors.push("Android platform requires Android signing identity".into());
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Signing identity for a platform.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SigningIdentity {
    /// iOS signing identity.
    Ios(IosSigningIdentity),
    /// Android signing identity.
    Android(AndroidSigningIdentity),
}

impl SigningIdentity {
    /// Get the platform for this identity.
    pub fn platform(&self) -> MobilePlatform {
        match self {
            SigningIdentity::Ios(_) => MobilePlatform::Ios,
            SigningIdentity::Android(_) => MobilePlatform::Android,
        }
    }

    /// Check if the identity is valid (not expired, etc.).
    pub fn is_valid(&self) -> bool {
        match self {
            SigningIdentity::Ios(ios) => ios.is_valid(),
            SigningIdentity::Android(android) => android.is_valid(),
        }
    }
}

/// iOS signing identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosSigningIdentity {
    /// Certificate common name (e.g., "Apple Distribution: Company Name").
    pub certificate_name: String,
    /// Team ID.
    pub team_id: String,
    /// Certificate type.
    pub certificate_type: IosCertificateType,
    /// Certificate expiration date.
    pub expires_at: Option<DateTime<Utc>>,
    /// Provisioning profile.
    pub provisioning_profile: Option<IosProvisioningProfile>,
}

impl IosSigningIdentity {
    /// Create a new iOS signing identity.
    pub fn new(
        certificate_name: impl Into<String>,
        team_id: impl Into<String>,
        certificate_type: IosCertificateType,
    ) -> Self {
        Self {
            certificate_name: certificate_name.into(),
            team_id: team_id.into(),
            certificate_type,
            expires_at: None,
            provisioning_profile: None,
        }
    }

    /// Create a development identity.
    pub fn development(certificate_name: impl Into<String>, team_id: impl Into<String>) -> Self {
        Self::new(certificate_name, team_id, IosCertificateType::Development)
    }

    /// Create a distribution identity.
    pub fn distribution(certificate_name: impl Into<String>, team_id: impl Into<String>) -> Self {
        Self::new(certificate_name, team_id, IosCertificateType::Distribution)
    }

    /// Set the provisioning profile.
    pub fn with_profile(mut self, profile: IosProvisioningProfile) -> Self {
        self.provisioning_profile = Some(profile);
        self
    }

    /// Set the expiration date.
    pub fn with_expiration(mut self, expires_at: DateTime<Utc>) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    /// Check if the certificate is valid.
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => expiry > Utc::now(),
            None => true, // Assume valid if no expiry date
        }
    }

    /// Check if the certificate will expire soon (within 30 days).
    pub fn expires_soon(&self) -> bool {
        match self.expires_at {
            Some(expiry) => {
                let thirty_days = chrono::Duration::days(30);
                expiry - Utc::now() < thirty_days
            }
            None => false,
        }
    }

    /// Get the CODE_SIGN_IDENTITY value for Xcode.
    pub fn xcode_identity(&self) -> String {
        self.certificate_name.clone()
    }

    /// Get the DEVELOPMENT_TEAM value for Xcode.
    pub fn xcode_team(&self) -> &str {
        &self.team_id
    }
}

/// iOS certificate type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IosCertificateType {
    /// Development certificate (for debugging).
    Development,
    /// Distribution certificate (for App Store or Ad Hoc).
    Distribution,
    /// Enterprise certificate (for in-house distribution).
    Enterprise,
}

impl IosCertificateType {
    /// Get the certificate type prefix used by Apple.
    pub fn prefix(&self) -> &'static str {
        match self {
            IosCertificateType::Development => "Apple Development",
            IosCertificateType::Distribution => "Apple Distribution",
            IosCertificateType::Enterprise => "iPhone Distribution",
        }
    }
}

impl std::fmt::Display for IosCertificateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            IosCertificateType::Development => "Development",
            IosCertificateType::Distribution => "Distribution",
            IosCertificateType::Enterprise => "Enterprise",
        };
        write!(f, "{}", name)
    }
}

/// iOS provisioning profile.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IosProvisioningProfile {
    /// Profile name.
    pub name: String,
    /// Profile UUID.
    pub uuid: String,
    /// Bundle ID the profile is for.
    pub bundle_id: String,
    /// Profile type.
    pub profile_type: IosProfileType,
    /// Expiration date.
    pub expires_at: Option<DateTime<Utc>>,
    /// Team ID.
    pub team_id: String,
    /// Path to the profile file.
    pub path: Option<PathBuf>,
}

impl IosProvisioningProfile {
    /// Create a new provisioning profile.
    pub fn new(
        name: impl Into<String>,
        uuid: impl Into<String>,
        bundle_id: impl Into<String>,
        team_id: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            uuid: uuid.into(),
            bundle_id: bundle_id.into(),
            profile_type: IosProfileType::Development,
            expires_at: None,
            team_id: team_id.into(),
            path: None,
        }
    }

    /// Set the profile type.
    pub fn with_type(mut self, profile_type: IosProfileType) -> Self {
        self.profile_type = profile_type;
        self
    }

    /// Set the path.
    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Check if the profile is valid.
    pub fn is_valid(&self) -> bool {
        match self.expires_at {
            Some(expiry) => expiry > Utc::now(),
            None => true,
        }
    }

    /// Get the PROVISIONING_PROFILE_SPECIFIER value for Xcode.
    pub fn xcode_specifier(&self) -> &str {
        &self.name
    }
}

/// iOS provisioning profile type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IosProfileType {
    /// Development profile.
    Development,
    /// Ad Hoc distribution profile.
    AdHoc,
    /// App Store distribution profile.
    AppStore,
    /// Enterprise distribution profile.
    Enterprise,
}

impl std::fmt::Display for IosProfileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            IosProfileType::Development => "Development",
            IosProfileType::AdHoc => "Ad Hoc",
            IosProfileType::AppStore => "App Store",
            IosProfileType::Enterprise => "Enterprise",
        };
        write!(f, "{}", name)
    }
}

/// Android signing identity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidSigningIdentity {
    /// Path to the keystore file.
    pub keystore_path: PathBuf,
    /// Key alias in the keystore.
    pub key_alias: String,
    /// Keystore password (should be loaded from secure storage).
    #[serde(skip_serializing)]
    pub keystore_password: String,
    /// Key password (should be loaded from secure storage).
    #[serde(skip_serializing)]
    pub key_password: String,
    /// Certificate validity end date.
    pub valid_until: Option<DateTime<Utc>>,
    /// Distinguished name from the certificate.
    pub distinguished_name: Option<String>,
}

impl AndroidSigningIdentity {
    /// Create a new Android signing identity.
    pub fn new(
        keystore_path: impl Into<PathBuf>,
        key_alias: impl Into<String>,
        keystore_password: impl Into<String>,
        key_password: impl Into<String>,
    ) -> Self {
        Self {
            keystore_path: keystore_path.into(),
            key_alias: key_alias.into(),
            keystore_password: keystore_password.into(),
            key_password: key_password.into(),
            valid_until: None,
            distinguished_name: None,
        }
    }

    /// Create an identity for the debug keystore.
    pub fn debug() -> Self {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
        Self {
            keystore_path: PathBuf::from(home).join(".android/debug.keystore"),
            key_alias: "androiddebugkey".into(),
            keystore_password: "android".into(),
            key_password: "android".into(),
            valid_until: None,
            distinguished_name: Some("CN=Android Debug".into()),
        }
    }

    /// Set validity information.
    pub fn with_validity(mut self, valid_until: DateTime<Utc>, dn: impl Into<String>) -> Self {
        self.valid_until = Some(valid_until);
        self.distinguished_name = Some(dn.into());
        self
    }

    /// Check if the certificate is valid.
    pub fn is_valid(&self) -> bool {
        match self.valid_until {
            Some(expiry) => expiry > Utc::now(),
            None => true,
        }
    }

    /// Get environment variables for Gradle signing.
    pub fn gradle_env(&self) -> Vec<(String, String)> {
        vec![
            (
                "SIGNING_STORE_FILE".into(),
                self.keystore_path.display().to_string(),
            ),
            (
                "SIGNING_STORE_PASSWORD".into(),
                self.keystore_password.clone(),
            ),
            ("SIGNING_KEY_ALIAS".into(), self.key_alias.clone()),
            ("SIGNING_KEY_PASSWORD".into(), self.key_password.clone()),
        ]
    }
}

/// Signing options.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SigningOptions {
    /// Enable timestamp (for long-term validation).
    pub timestamp: bool,
    /// Timestamp server URL.
    pub timestamp_url: Option<String>,
    /// Enable hardened runtime (macOS/iOS).
    pub hardened_runtime: bool,
    /// Entitlements file path.
    pub entitlements: Option<PathBuf>,
    /// Signature algorithm.
    pub algorithm: Option<String>,
    /// V1 signing (JAR signature) for Android.
    pub android_v1_signing: bool,
    /// V2 signing (APK signature scheme) for Android.
    pub android_v2_signing: bool,
    /// V3 signing (APK signature scheme v3) for Android.
    pub android_v3_signing: bool,
}

impl SigningOptions {
    /// Create options for iOS signing.
    pub fn ios() -> Self {
        Self {
            timestamp: true,
            hardened_runtime: true,
            ..Default::default()
        }
    }

    /// Create options for Android signing.
    pub fn android() -> Self {
        Self {
            android_v1_signing: true,
            android_v2_signing: true,
            android_v3_signing: true,
            ..Default::default()
        }
    }

    /// Set entitlements file.
    pub fn with_entitlements(mut self, path: impl Into<PathBuf>) -> Self {
        self.entitlements = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ios_signing_identity() {
        let identity = IosSigningIdentity::distribution("Apple Distribution: My Company", "TEAM123");

        assert_eq!(identity.team_id, "TEAM123");
        assert!(matches!(
            identity.certificate_type,
            IosCertificateType::Distribution
        ));
        assert!(identity.is_valid()); // No expiry set, assumed valid
    }

    #[test]
    fn test_ios_signing_identity_expired() {
        let past = Utc::now() - chrono::Duration::days(1);
        let identity = IosSigningIdentity::development("Apple Development", "TEAM123")
            .with_expiration(past);

        assert!(!identity.is_valid());
    }

    #[test]
    fn test_android_signing_identity() {
        let identity = AndroidSigningIdentity::debug();

        assert_eq!(identity.key_alias, "androiddebugkey");
        assert_eq!(identity.keystore_password, "android");
        assert!(identity.is_valid());
    }

    #[test]
    fn test_android_gradle_env() {
        let identity = AndroidSigningIdentity::new(
            "/path/to/keystore.jks",
            "release",
            "storepass",
            "keypass",
        );

        let env = identity.gradle_env();
        assert_eq!(env.len(), 4);
        assert!(env.iter().any(|(k, _)| k == "SIGNING_STORE_FILE"));
        assert!(env.iter().any(|(k, _)| k == "SIGNING_KEY_ALIAS"));
    }

    #[test]
    fn test_signing_config_validation() {
        let ios_identity = SigningIdentity::Ios(IosSigningIdentity::distribution(
            "Apple Distribution",
            "TEAM123",
        ));

        let config = SigningConfig::ios(ios_identity);
        assert!(config.validate().is_ok());

        // Invalid: empty certificate name
        let invalid_identity = SigningIdentity::Ios(IosSigningIdentity::new(
            "",
            "TEAM123",
            IosCertificateType::Development,
        ));
        let invalid_config = SigningConfig::ios(invalid_identity);
        assert!(invalid_config.validate().is_err());
    }

    #[test]
    fn test_provisioning_profile() {
        let profile = IosProvisioningProfile::new(
            "MyApp Development",
            "ABC123-DEF456",
            "com.example.myapp",
            "TEAM123",
        )
        .with_type(IosProfileType::Development);

        assert_eq!(profile.xcode_specifier(), "MyApp Development");
        assert!(profile.is_valid());
    }
}
