//! Mobile target definitions.
//!
//! Defines target platforms for mobile builds, including iOS simulator/device
//! and Android emulator/device variants.

use serde::{Deserialize, Serialize};

/// Mobile target platforms for building.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobileTarget {
    /// iOS Simulator (x86_64 or arm64 on Apple Silicon).
    IosSimulator,
    /// Physical iOS device (arm64).
    IosDevice,
    /// Android Emulator (x86_64 or arm64).
    AndroidEmulator,
    /// Physical Android device (arm64, arm, x86_64, or x86).
    AndroidDevice,
}

impl MobileTarget {
    /// Get the parent platform for this target.
    pub fn platform(&self) -> MobilePlatform {
        match self {
            MobileTarget::IosSimulator | MobileTarget::IosDevice => MobilePlatform::Ios,
            MobileTarget::AndroidEmulator | MobileTarget::AndroidDevice => MobilePlatform::Android,
        }
    }

    /// Returns true if this target is a simulator/emulator.
    pub fn is_simulator(&self) -> bool {
        matches!(
            self,
            MobileTarget::IosSimulator | MobileTarget::AndroidEmulator
        )
    }

    /// Returns true if this target is a physical device.
    pub fn is_device(&self) -> bool {
        matches!(self, MobileTarget::IosDevice | MobileTarget::AndroidDevice)
    }

    /// Returns true if this is an iOS target.
    pub fn is_ios(&self) -> bool {
        matches!(
            self,
            MobileTarget::IosSimulator | MobileTarget::IosDevice
        )
    }

    /// Returns true if this is an Android target.
    pub fn is_android(&self) -> bool {
        matches!(
            self,
            MobileTarget::AndroidEmulator | MobileTarget::AndroidDevice
        )
    }

    /// Get the Rust target triple for this mobile target.
    pub fn rust_target_triple(&self) -> &'static str {
        match self {
            MobileTarget::IosSimulator => {
                #[cfg(target_arch = "aarch64")]
                {
                    "aarch64-apple-ios-sim"
                }
                #[cfg(not(target_arch = "aarch64"))]
                {
                    "x86_64-apple-ios"
                }
            }
            MobileTarget::IosDevice => "aarch64-apple-ios",
            MobileTarget::AndroidEmulator => {
                #[cfg(target_arch = "aarch64")]
                {
                    "aarch64-linux-android"
                }
                #[cfg(not(target_arch = "aarch64"))]
                {
                    "x86_64-linux-android"
                }
            }
            MobileTarget::AndroidDevice => "aarch64-linux-android",
        }
    }

    /// Get a human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            MobileTarget::IosSimulator => "iOS Simulator",
            MobileTarget::IosDevice => "iOS Device",
            MobileTarget::AndroidEmulator => "Android Emulator",
            MobileTarget::AndroidDevice => "Android Device",
        }
    }

    /// Get all available mobile targets.
    pub fn all() -> &'static [MobileTarget] {
        &[
            MobileTarget::IosSimulator,
            MobileTarget::IosDevice,
            MobileTarget::AndroidEmulator,
            MobileTarget::AndroidDevice,
        ]
    }
}

impl std::fmt::Display for MobileTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Top-level mobile platform.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MobilePlatform {
    /// Apple iOS platform.
    Ios,
    /// Google Android platform.
    Android,
}

impl MobilePlatform {
    /// Get the available targets for this platform.
    pub fn targets(&self) -> &'static [MobileTarget] {
        match self {
            MobilePlatform::Ios => &[MobileTarget::IosSimulator, MobileTarget::IosDevice],
            MobilePlatform::Android => {
                &[MobileTarget::AndroidEmulator, MobileTarget::AndroidDevice]
            }
        }
    }

    /// Get all available mobile platforms.
    pub fn all() -> &'static [MobilePlatform] {
        &[MobilePlatform::Ios, MobilePlatform::Android]
    }

    /// Get a human-readable display name.
    pub fn display_name(&self) -> &'static str {
        match self {
            MobilePlatform::Ios => "iOS",
            MobilePlatform::Android => "Android",
        }
    }
}

impl std::fmt::Display for MobilePlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

/// Device class based on screen size and capabilities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceClass {
    /// Small phone (< 360dp width).
    SmallPhone,
    /// Standard phone (360-400dp width).
    Phone,
    /// Large phone / phablet (400-600dp width).
    LargePhone,
    /// Small tablet (600-720dp width).
    SmallTablet,
    /// Standard tablet (720-840dp width).
    Tablet,
    /// Large tablet (> 840dp width).
    LargeTablet,
}

impl DeviceClass {
    /// Classify a device based on screen width in density-independent pixels.
    pub fn from_width_dp(width_dp: f32) -> Self {
        match width_dp {
            w if w < 360.0 => DeviceClass::SmallPhone,
            w if w < 400.0 => DeviceClass::Phone,
            w if w < 600.0 => DeviceClass::LargePhone,
            w if w < 720.0 => DeviceClass::SmallTablet,
            w if w < 840.0 => DeviceClass::Tablet,
            _ => DeviceClass::LargeTablet,
        }
    }

    /// Returns true if this is a phone-class device.
    pub fn is_phone(&self) -> bool {
        matches!(
            self,
            DeviceClass::SmallPhone | DeviceClass::Phone | DeviceClass::LargePhone
        )
    }

    /// Returns true if this is a tablet-class device.
    pub fn is_tablet(&self) -> bool {
        matches!(
            self,
            DeviceClass::SmallTablet | DeviceClass::Tablet | DeviceClass::LargeTablet
        )
    }

    /// Get the typical column count for this device class.
    pub fn typical_columns(&self) -> u32 {
        match self {
            DeviceClass::SmallPhone => 4,
            DeviceClass::Phone => 4,
            DeviceClass::LargePhone => 4,
            DeviceClass::SmallTablet => 8,
            DeviceClass::Tablet => 12,
            DeviceClass::LargeTablet => 12,
        }
    }

    /// Get the typical margin in dp for this device class.
    pub fn typical_margin_dp(&self) -> f32 {
        match self {
            DeviceClass::SmallPhone => 16.0,
            DeviceClass::Phone => 16.0,
            DeviceClass::LargePhone => 16.0,
            DeviceClass::SmallTablet => 24.0,
            DeviceClass::Tablet => 24.0,
            DeviceClass::LargeTablet => 32.0,
        }
    }
}

impl std::fmt::Display for DeviceClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            DeviceClass::SmallPhone => "Small Phone",
            DeviceClass::Phone => "Phone",
            DeviceClass::LargePhone => "Large Phone",
            DeviceClass::SmallTablet => "Small Tablet",
            DeviceClass::Tablet => "Tablet",
            DeviceClass::LargeTablet => "Large Tablet",
        };
        write!(f, "{}", name)
    }
}

/// Android ABI (Application Binary Interface).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AndroidAbi {
    /// ARM 64-bit (most modern devices).
    Arm64V8a,
    /// ARM 32-bit (legacy devices).
    ArmeabiV7a,
    /// x86 64-bit (emulators, some Intel devices).
    X86_64,
    /// x86 32-bit (legacy emulators).
    X86,
}

impl AndroidAbi {
    /// Get the NDK ABI name.
    pub fn ndk_name(&self) -> &'static str {
        match self {
            AndroidAbi::Arm64V8a => "arm64-v8a",
            AndroidAbi::ArmeabiV7a => "armeabi-v7a",
            AndroidAbi::X86_64 => "x86_64",
            AndroidAbi::X86 => "x86",
        }
    }

    /// Get the Rust target triple.
    pub fn rust_target(&self) -> &'static str {
        match self {
            AndroidAbi::Arm64V8a => "aarch64-linux-android",
            AndroidAbi::ArmeabiV7a => "armv7-linux-androideabi",
            AndroidAbi::X86_64 => "x86_64-linux-android",
            AndroidAbi::X86 => "i686-linux-android",
        }
    }

    /// Get all Android ABIs.
    pub fn all() -> &'static [AndroidAbi] {
        &[
            AndroidAbi::Arm64V8a,
            AndroidAbi::ArmeabiV7a,
            AndroidAbi::X86_64,
            AndroidAbi::X86,
        ]
    }

    /// Get the recommended ABIs for modern devices.
    pub fn modern() -> &'static [AndroidAbi] {
        &[AndroidAbi::Arm64V8a, AndroidAbi::X86_64]
    }
}

impl std::fmt::Display for AndroidAbi {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.ndk_name())
    }
}

/// iOS architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum IosArch {
    /// ARM 64-bit (all modern iOS devices).
    Arm64,
    /// ARM 64-bit simulator (Apple Silicon Macs).
    Arm64Sim,
    /// x86 64-bit simulator (Intel Macs).
    X86_64,
}

impl IosArch {
    /// Get the Rust target triple.
    pub fn rust_target(&self) -> &'static str {
        match self {
            IosArch::Arm64 => "aarch64-apple-ios",
            IosArch::Arm64Sim => "aarch64-apple-ios-sim",
            IosArch::X86_64 => "x86_64-apple-ios",
        }
    }

    /// Get all iOS architectures.
    pub fn all() -> &'static [IosArch] {
        &[IosArch::Arm64, IosArch::Arm64Sim, IosArch::X86_64]
    }

    /// Returns true if this is a simulator architecture.
    pub fn is_simulator(&self) -> bool {
        matches!(self, IosArch::Arm64Sim | IosArch::X86_64)
    }
}

impl std::fmt::Display for IosArch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.rust_target())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mobile_target_platform() {
        assert_eq!(MobileTarget::IosDevice.platform(), MobilePlatform::Ios);
        assert_eq!(
            MobileTarget::AndroidDevice.platform(),
            MobilePlatform::Android
        );
    }

    #[test]
    fn test_mobile_target_is_simulator() {
        assert!(MobileTarget::IosSimulator.is_simulator());
        assert!(MobileTarget::AndroidEmulator.is_simulator());
        assert!(!MobileTarget::IosDevice.is_simulator());
        assert!(!MobileTarget::AndroidDevice.is_simulator());
    }

    #[test]
    fn test_mobile_target_is_device() {
        assert!(MobileTarget::IosDevice.is_device());
        assert!(MobileTarget::AndroidDevice.is_device());
        assert!(!MobileTarget::IosSimulator.is_device());
        assert!(!MobileTarget::AndroidEmulator.is_device());
    }

    #[test]
    fn test_device_class_from_width() {
        assert_eq!(DeviceClass::from_width_dp(320.0), DeviceClass::SmallPhone);
        assert_eq!(DeviceClass::from_width_dp(375.0), DeviceClass::Phone);
        assert_eq!(DeviceClass::from_width_dp(414.0), DeviceClass::LargePhone);
        assert_eq!(DeviceClass::from_width_dp(600.0), DeviceClass::SmallTablet);
        assert_eq!(DeviceClass::from_width_dp(768.0), DeviceClass::Tablet);
        assert_eq!(DeviceClass::from_width_dp(1024.0), DeviceClass::LargeTablet);
    }

    #[test]
    fn test_device_class_is_phone_tablet() {
        assert!(DeviceClass::Phone.is_phone());
        assert!(!DeviceClass::Phone.is_tablet());
        assert!(DeviceClass::Tablet.is_tablet());
        assert!(!DeviceClass::Tablet.is_phone());
    }

    #[test]
    fn test_android_abi_names() {
        assert_eq!(AndroidAbi::Arm64V8a.ndk_name(), "arm64-v8a");
        assert_eq!(AndroidAbi::Arm64V8a.rust_target(), "aarch64-linux-android");
    }

    #[test]
    fn test_ios_arch_rust_target() {
        assert_eq!(IosArch::Arm64.rust_target(), "aarch64-apple-ios");
        assert!(IosArch::Arm64Sim.is_simulator());
        assert!(!IosArch::Arm64.is_simulator());
    }

    #[test]
    fn test_mobile_platform_targets() {
        let ios_targets = MobilePlatform::Ios.targets();
        assert!(ios_targets.contains(&MobileTarget::IosSimulator));
        assert!(ios_targets.contains(&MobileTarget::IosDevice));

        let android_targets = MobilePlatform::Android.targets();
        assert!(android_targets.contains(&MobileTarget::AndroidEmulator));
        assert!(android_targets.contains(&MobileTarget::AndroidDevice));
    }
}
