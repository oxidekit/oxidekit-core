//! API markers for declaring portability of functions and types.
//!
//! These markers can be used to document and enforce portability requirements
//! at compile time (when using the proc-macro feature) or runtime.

use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

use crate::level::{ApiCategory, PortabilityConstraint, PortabilityLevel};
use crate::target::Target;

/// Trait for types that can describe their portability.
pub trait ApiMarker {
    /// Get the portability level of this API.
    fn portability_level() -> PortabilityLevel;

    /// Get the API category.
    fn category() -> ApiCategory;

    /// Get constraints for this API.
    fn constraints() -> PortabilityConstraint {
        PortabilityConstraint::default()
    }

    /// Check if this API works on the given target.
    fn works_on(target: &Target) -> bool {
        Self::constraints().is_satisfied_for(target)
    }

    /// Get the documentation for this API's portability.
    fn portability_doc() -> String {
        format!(
            "Portability: {} ({})\n{}",
            Self::portability_level(),
            Self::category(),
            Self::portability_level().description()
        )
    }
}

/// Marker for fully portable APIs.
///
/// Use this to mark APIs that work on all platforms:
/// - Desktop (macOS, Windows, Linux)
/// - Web (WASM)
/// - Mobile (iOS, Android)
///
/// # Example
///
/// ```rust,ignore
/// use oxide_portable::PortableApi;
///
/// struct MyPortableComponent;
///
/// impl PortableApi for MyPortableComponent {
///     fn category() -> ApiCategory {
///         ApiCategory::Ui
///     }
/// }
/// ```
pub trait PortableApi: ApiMarker {
    /// Get the category of this portable API.
    fn category() -> ApiCategory;
}

impl<T: PortableApi> ApiMarker for T {
    fn portability_level() -> PortabilityLevel {
        PortabilityLevel::Portable
    }

    fn category() -> ApiCategory {
        <T as PortableApi>::category()
    }
}

/// Marker for desktop-only APIs.
///
/// Use this to mark APIs that only work on desktop platforms:
/// - macOS
/// - Windows
/// - Linux
///
/// These APIs typically require:
/// - Native window management
/// - System tray access
/// - Native file dialogs
/// - Native menus
pub trait DesktopOnlyApi {
    /// Get the category of this desktop-only API.
    fn category() -> ApiCategory;

    /// Get the portability level (always DesktopOnly).
    fn portability_level() -> PortabilityLevel {
        PortabilityLevel::DesktopOnly
    }

    /// Get constraints for desktop-only APIs.
    fn constraints() -> PortabilityConstraint {
        crate::level::constraints::desktop_only()
    }

    /// Check if this API works on the given target.
    fn works_on(target: &Target) -> bool {
        target.is_desktop()
    }
}

/// Marker for web-only APIs.
///
/// Use this to mark APIs that only work in web/WASM environments:
/// - Browser APIs
/// - DOM manipulation
/// - Web Workers
pub trait WebOnlyApi {
    /// Get the category of this web-only API.
    fn category() -> ApiCategory;

    /// Get the portability level (always WebOnly).
    fn portability_level() -> PortabilityLevel {
        PortabilityLevel::WebOnly
    }

    /// Get constraints for web-only APIs.
    fn constraints() -> PortabilityConstraint {
        PortabilityConstraint::new()
            .allow_target("wasm32-unknown-unknown")
    }

    /// Check if this API works on the given target.
    fn works_on(target: &Target) -> bool {
        target.is_web()
    }
}

/// Marker for mobile-only APIs.
///
/// Use this to mark APIs that only work on mobile platforms:
/// - iOS
/// - Android
pub trait MobileOnlyApi {
    /// Get the category of this mobile-only API.
    fn category() -> ApiCategory;

    /// Get the portability level (always MobileOnly).
    fn portability_level() -> PortabilityLevel {
        PortabilityLevel::MobileOnly
    }

    /// Check if this API works on the given target.
    fn works_on(target: &Target) -> bool {
        target.is_mobile()
    }
}

/// Information about an API's portability, stored at runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortabilityInfo {
    /// The name of the API
    pub name: String,
    /// The portability level
    pub level: PortabilityLevel,
    /// The API category
    pub category: ApiCategory,
    /// Portability constraints
    pub constraints: PortabilityConstraint,
    /// Human-readable description
    pub description: Option<String>,
    /// Alternative APIs to use on unsupported platforms
    pub alternatives: Vec<AlternativeApi>,
}

impl PortabilityInfo {
    /// Create new portability info.
    pub fn new(name: impl Into<String>, level: PortabilityLevel, category: ApiCategory) -> Self {
        Self {
            name: name.into(),
            level,
            category,
            constraints: PortabilityConstraint::default(),
            description: None,
            alternatives: Vec::new(),
        }
    }

    /// Set the description.
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Set the constraints.
    pub fn with_constraints(mut self, constraints: PortabilityConstraint) -> Self {
        self.constraints = constraints;
        self
    }

    /// Add an alternative API.
    pub fn with_alternative(mut self, alt: AlternativeApi) -> Self {
        self.alternatives.push(alt);
        self
    }

    /// Check if this API works on the given target.
    pub fn works_on(&self, target: &Target) -> bool {
        // First check the portability level
        let level_ok = match self.level {
            PortabilityLevel::Portable => true,
            PortabilityLevel::DesktopOnly => target.is_desktop(),
            PortabilityLevel::WebOnly => target.is_web(),
            PortabilityLevel::MobileOnly => target.is_mobile(),
            PortabilityLevel::NativeOnly => target.is_desktop() || target.is_mobile(),
            PortabilityLevel::IosOnly => target.triple().contains("ios") || target.triple().contains("aarch64-apple"),
            PortabilityLevel::AndroidOnly => target.triple().contains("android"),
            PortabilityLevel::MacosOnly => target.triple().contains("darwin") || target.triple().contains("apple-macos"),
            PortabilityLevel::WindowsOnly => target.triple().contains("windows") || target.triple().contains("-pc-windows"),
            PortabilityLevel::LinuxOnly => target.triple().contains("linux"),
            PortabilityLevel::Experimental => true, // Allow experimental everywhere
        };

        // Then check any additional constraints
        level_ok && self.constraints.is_satisfied_for(target)
    }

    /// Get the first suitable alternative for a target.
    pub fn alternative_for(&self, target: &Target) -> Option<&AlternativeApi> {
        self.alternatives.iter().find(|alt| alt.works_on(target))
    }
}

/// An alternative API to use on unsupported platforms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlternativeApi {
    /// Name of the alternative API
    pub name: String,
    /// Portability level of the alternative
    pub level: PortabilityLevel,
    /// When to use this alternative
    pub when: String,
    /// Import path or usage instructions
    pub usage: String,
}

impl AlternativeApi {
    /// Create a new alternative.
    pub fn new(name: impl Into<String>, level: PortabilityLevel) -> Self {
        Self {
            name: name.into(),
            level,
            when: String::new(),
            usage: String::new(),
        }
    }

    /// Set when to use this alternative.
    pub fn when(mut self, condition: impl Into<String>) -> Self {
        self.when = condition.into();
        self
    }

    /// Set usage instructions.
    pub fn usage(mut self, usage: impl Into<String>) -> Self {
        self.usage = usage.into();
        self
    }

    /// Check if this alternative works on the target.
    pub fn works_on(&self, target: &Target) -> bool {
        match self.level {
            PortabilityLevel::Portable => true,
            PortabilityLevel::DesktopOnly => target.is_desktop(),
            PortabilityLevel::WebOnly => target.is_web(),
            PortabilityLevel::MobileOnly => target.is_mobile(),
            _ => true, // Conservative default
        }
    }
}

/// Zero-sized type that carries portability information at compile time.
///
/// Use this to create type-safe API markers with level and category IDs:
///
/// ```rust,ignore
/// use oxide_portable::PortabilityMarker;
///
/// // LEVEL: 0=Portable, 1=DesktopOnly, 2=WebOnly, 3=MobileOnly
/// // CATEGORY: 0=Core, 1=Ui, 2=Layout, etc.
/// type MyApiMarker = PortabilityMarker<0, 1>; // Portable, UI
/// ```
#[derive(Debug, Clone, Copy, Default)]
pub struct PortabilityMarker<const LEVEL: u8, const CATEGORY: u8> {
    _phantom: PhantomData<()>,
}

impl<const LEVEL: u8, const CATEGORY: u8> PortabilityMarker<LEVEL, CATEGORY> {
    /// Get the portability level for this marker.
    pub fn level() -> PortabilityLevel {
        match LEVEL {
            0 => PortabilityLevel::Portable,
            1 => PortabilityLevel::DesktopOnly,
            2 => PortabilityLevel::WebOnly,
            3 => PortabilityLevel::MobileOnly,
            4 => PortabilityLevel::NativeOnly,
            _ => PortabilityLevel::Experimental,
        }
    }

    /// Get the API category for this marker.
    pub fn category() -> ApiCategory {
        match CATEGORY {
            0 => ApiCategory::Core,
            1 => ApiCategory::Ui,
            2 => ApiCategory::Layout,
            3 => ApiCategory::Render,
            4 => ApiCategory::Input,
            5 => ApiCategory::FileSystem,
            6 => ApiCategory::Network,
            7 => ApiCategory::Window,
            8 => ApiCategory::System,
            9 => ApiCategory::Clipboard,
            10 => ApiCategory::Notifications,
            11 => ApiCategory::Storage,
            12 => ApiCategory::Auth,
            13 => ApiCategory::Sensors,
            14 => ApiCategory::Native,
            _ => ApiCategory::Native,
        }
    }
}

/// Registry of API portability information.
///
/// Used by the portability checker to track APIs and their portability.
#[derive(Debug, Default)]
pub struct PortabilityRegistry {
    apis: Vec<PortabilityInfo>,
}

impl PortabilityRegistry {
    /// Create a new empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Register an API.
    pub fn register(&mut self, info: PortabilityInfo) {
        self.apis.push(info);
    }

    /// Get all registered APIs.
    pub fn apis(&self) -> &[PortabilityInfo] {
        &self.apis
    }

    /// Get APIs by category.
    pub fn by_category(&self, category: ApiCategory) -> Vec<&PortabilityInfo> {
        self.apis.iter().filter(|api| api.category == category).collect()
    }

    /// Get APIs by portability level.
    pub fn by_level(&self, level: PortabilityLevel) -> Vec<&PortabilityInfo> {
        self.apis.iter().filter(|api| api.level == level).collect()
    }

    /// Get portable APIs.
    pub fn portable(&self) -> Vec<&PortabilityInfo> {
        self.by_level(PortabilityLevel::Portable)
    }

    /// Get desktop-only APIs.
    pub fn desktop_only(&self) -> Vec<&PortabilityInfo> {
        self.by_level(PortabilityLevel::DesktopOnly)
    }

    /// Find APIs that work on a specific target.
    pub fn for_target(&self, target: &Target) -> Vec<&PortabilityInfo> {
        self.apis.iter().filter(|api| api.works_on(target)).collect()
    }

    /// Find APIs that DON'T work on a specific target.
    pub fn not_for_target(&self, target: &Target) -> Vec<&PortabilityInfo> {
        self.apis.iter().filter(|api| !api.works_on(target)).collect()
    }
}

/// Macro to create a portable API implementation.
///
/// This is available without the proc-macro feature.
#[macro_export]
macro_rules! impl_portable_api {
    ($type:ty, $category:expr) => {
        impl $crate::api::PortableApi for $type {
            fn category() -> $crate::level::ApiCategory {
                $category
            }
        }
    };
}

/// Macro to create portability info inline.
#[macro_export]
macro_rules! portability_info {
    ($name:expr, portable, $category:expr) => {
        $crate::api::PortabilityInfo::new(
            $name,
            $crate::level::PortabilityLevel::Portable,
            $category,
        )
    };
    ($name:expr, desktop_only, $category:expr) => {
        $crate::api::PortabilityInfo::new(
            $name,
            $crate::level::PortabilityLevel::DesktopOnly,
            $category,
        )
    };
    ($name:expr, web_only, $category:expr) => {
        $crate::api::PortabilityInfo::new(
            $name,
            $crate::level::PortabilityLevel::WebOnly,
            $category,
        )
    };
    ($name:expr, mobile_only, $category:expr) => {
        $crate::api::PortabilityInfo::new(
            $name,
            $crate::level::PortabilityLevel::MobileOnly,
            $category,
        )
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::target::targets;

    #[test]
    fn test_portability_info() {
        let info = PortabilityInfo::new("test_api", PortabilityLevel::Portable, ApiCategory::Ui)
            .with_description("A test API");

        assert!(info.works_on(&targets::macos_arm64()));
        assert!(info.works_on(&targets::web_wasm32()));
        assert!(info.works_on(&targets::ios_arm64()));
    }

    #[test]
    fn test_desktop_only_info() {
        let info = PortabilityInfo::new("window_api", PortabilityLevel::DesktopOnly, ApiCategory::Window)
            .with_constraints(crate::level::constraints::desktop_only());

        assert!(info.works_on(&targets::macos_arm64()));
        assert!(!info.works_on(&targets::web_wasm32()));
    }

    #[test]
    fn test_alternatives() {
        let info = PortabilityInfo::new("native_dialog", PortabilityLevel::DesktopOnly, ApiCategory::Window)
            .with_constraints(crate::level::constraints::desktop_only())
            .with_alternative(
                AlternativeApi::new("web_dialog", PortabilityLevel::WebOnly)
                    .when("target is wasm32")
                    .usage("use oxide_web::dialog::Dialog")
            );

        let web = targets::web_wasm32();
        assert!(!info.works_on(&web));
        assert!(info.alternative_for(&web).is_some());
    }

    #[test]
    fn test_registry() {
        let mut registry = PortabilityRegistry::new();

        registry.register(PortabilityInfo::new("portable_api", PortabilityLevel::Portable, ApiCategory::Ui));
        registry.register(PortabilityInfo::new("desktop_api", PortabilityLevel::DesktopOnly, ApiCategory::Window));

        assert_eq!(registry.portable().len(), 1);
        assert_eq!(registry.desktop_only().len(), 1);
        assert_eq!(registry.for_target(&targets::macos_arm64()).len(), 2);
        assert_eq!(registry.for_target(&targets::web_wasm32()).len(), 1);
    }
}
