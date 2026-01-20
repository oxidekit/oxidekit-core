//! Plugin categories and kind-specific configuration.
//!
//! OxideKit supports six plugin categories, each with different capabilities
//! and trust requirements:
//!
//! - **UI**: Components with no permissions by default
//! - **Native**: OS capabilities requiring explicit permissions
//! - **Service**: App-level building blocks
//! - **Tooling**: Dev/build-time tools (higher risk)
//! - **Theme**: Data-first styling packages
//! - **Design**: Templates and layout kits

use serde::{Deserialize, Serialize};
use crate::permissions::Capability;

/// The category of a plugin, determining its capabilities and trust requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PluginCategory {
    /// UI components and component packs (tables, forms, charts).
    /// Should request no permissions by default.
    Ui,

    /// OS capabilities: filesystem, keychain, notifications, tray, updates.
    /// Must declare permissions explicitly.
    Native,

    /// App-level building blocks: auth, db, query cache, analytics.
    /// May depend on native capabilities.
    Service,

    /// Dev/build-time tools: generators, linters, migrations, formatters.
    /// Runs during dev/build; treated as higher-risk.
    Tooling,

    /// Token packs, typography packs, icon sets.
    /// Data-first, no code execution.
    Theme,

    /// Admin shells, templates, layout kits.
    /// Data-first, extractable parts.
    Design,
}

impl PluginCategory {
    /// Get a human-readable description of this category.
    pub fn description(&self) -> &'static str {
        match self {
            PluginCategory::Ui => "UI components and component packs",
            PluginCategory::Native => "OS capabilities and native integrations",
            PluginCategory::Service => "App-level building blocks and services",
            PluginCategory::Tooling => "Development and build-time tools",
            PluginCategory::Theme => "Styling tokens, typography, and icon sets",
            PluginCategory::Design => "Templates, admin shells, and layout kits",
        }
    }

    /// Get the allowed capabilities for this plugin category.
    pub fn allowed_capabilities(&self) -> Vec<Capability> {
        match self {
            PluginCategory::Ui => vec![
                // UI plugins should not request dangerous capabilities
                // They may only use safe UI-related APIs
            ],
            PluginCategory::Native => vec![
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
            ],
            PluginCategory::Service => vec![
                Capability::NetworkHttp,
                Capability::NetworkWebsocket,
                Capability::FilesystemRead,
                Capability::FilesystemWrite,
            ],
            PluginCategory::Tooling => vec![
                Capability::FilesystemRead,
                Capability::FilesystemWrite,
                Capability::ProcessSpawn,
                Capability::NetworkHttp,
            ],
            PluginCategory::Theme => vec![
                // Theme plugins are data-only, no capabilities
            ],
            PluginCategory::Design => vec![
                // Design plugins are data-only, no capabilities
            ],
        }
    }

    /// Check if a capability is allowed for this category.
    pub fn is_capability_allowed(&self, capability: &Capability) -> bool {
        self.allowed_capabilities().contains(capability)
    }

    /// Check if this category typically runs code (vs data-only).
    pub fn executes_code(&self) -> bool {
        match self {
            PluginCategory::Ui => true,
            PluginCategory::Native => true,
            PluginCategory::Service => true,
            PluginCategory::Tooling => true,
            PluginCategory::Theme => false,
            PluginCategory::Design => false,
        }
    }

    /// Get the default trust requirement for this category.
    pub fn default_trust_requirement(&self) -> crate::trust::TrustLevel {
        match self {
            PluginCategory::Ui => crate::trust::TrustLevel::Community,
            PluginCategory::Native => crate::trust::TrustLevel::Verified,
            PluginCategory::Service => crate::trust::TrustLevel::Verified,
            PluginCategory::Tooling => crate::trust::TrustLevel::Verified,
            PluginCategory::Theme => crate::trust::TrustLevel::Community,
            PluginCategory::Design => crate::trust::TrustLevel::Community,
        }
    }

    /// Check if this category requires WASM sandbox for community plugins.
    pub fn requires_sandbox_for_community(&self) -> bool {
        match self {
            PluginCategory::Ui => true,
            PluginCategory::Native => true, // Native plugins from community should be sandboxed
            PluginCategory::Service => true,
            PluginCategory::Tooling => true,
            PluginCategory::Theme => false, // Data-only
            PluginCategory::Design => false, // Data-only
        }
    }
}

impl std::fmt::Display for PluginCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            PluginCategory::Ui => "ui",
            PluginCategory::Native => "native",
            PluginCategory::Service => "service",
            PluginCategory::Tooling => "tooling",
            PluginCategory::Theme => "theme",
            PluginCategory::Design => "design",
        };
        write!(f, "{}", s)
    }
}

/// Kind-specific configuration sections in the manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PluginKindConfig {
    /// Configuration for UI plugins.
    Ui(UiConfig),
    /// Configuration for native plugins.
    Native(NativeConfig),
    /// Configuration for service plugins.
    Service(ServiceConfig),
    /// Configuration for tooling plugins.
    Tooling(ToolingConfig),
    /// Configuration for theme plugins.
    Theme(ThemeConfig),
    /// Configuration for design plugins.
    Design(DesignConfig),
}

/// Configuration specific to UI plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UiConfig {
    /// List of exported components.
    #[serde(default)]
    pub components: Vec<ComponentDefinition>,

    /// Token hooks for theming.
    #[serde(default)]
    pub token_hooks: Vec<String>,

    /// Whether this is a component pack (multiple components).
    #[serde(default)]
    pub is_pack: bool,
}

/// Definition of a UI component.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    /// Component name.
    pub name: String,

    /// Component description.
    #[serde(default)]
    pub description: String,

    /// Component props/inputs.
    #[serde(default)]
    pub props: Vec<PropDefinition>,

    /// Component events/outputs.
    #[serde(default)]
    pub events: Vec<String>,
}

/// Definition of a component prop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropDefinition {
    /// Prop name.
    pub name: String,

    /// Prop type.
    #[serde(rename = "type")]
    pub prop_type: String,

    /// Whether the prop is required.
    #[serde(default)]
    pub required: bool,

    /// Default value (as string).
    #[serde(default)]
    pub default: Option<String>,

    /// Prop description.
    #[serde(default)]
    pub description: String,
}

/// Configuration specific to native plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NativeConfig {
    /// Required capabilities.
    #[serde(default)]
    pub capabilities: Vec<String>,

    /// Platform-specific configurations.
    #[serde(default)]
    pub platforms: PlatformConfig,

    /// Native library name (for dynamic loading).
    #[serde(default)]
    pub library_name: Option<String>,
}

/// Platform-specific configurations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformConfig {
    /// macOS-specific settings.
    #[serde(default)]
    pub macos: Option<PlatformSettings>,

    /// Windows-specific settings.
    #[serde(default)]
    pub windows: Option<PlatformSettings>,

    /// Linux-specific settings.
    #[serde(default)]
    pub linux: Option<PlatformSettings>,
}

/// Settings for a specific platform.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PlatformSettings {
    /// Required frameworks or libraries.
    #[serde(default)]
    pub frameworks: Vec<String>,

    /// Minimum OS version.
    #[serde(default)]
    pub min_os_version: Option<String>,

    /// Additional compiler flags.
    #[serde(default)]
    pub compile_flags: Vec<String>,
}

/// Configuration specific to service plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// Service entrypoints.
    #[serde(default)]
    pub entrypoints: Vec<ServiceEntrypoint>,

    /// Service dependencies.
    #[serde(default)]
    pub dependencies: Vec<String>,

    /// Configuration schema (for runtime config).
    #[serde(default)]
    pub config_schema: Option<String>,
}

/// Definition of a service entrypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceEntrypoint {
    /// Entrypoint name.
    pub name: String,

    /// Entrypoint type (init, handler, middleware, etc.).
    #[serde(rename = "type")]
    pub entry_type: String,

    /// Description.
    #[serde(default)]
    pub description: String,
}

/// Configuration specific to tooling plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ToolingConfig {
    /// Commands registered by this tool.
    #[serde(default)]
    pub commands: Vec<ToolCommand>,

    /// Hooks registered by this tool.
    #[serde(default)]
    pub hooks: Vec<ToolHook>,

    /// File patterns this tool operates on.
    #[serde(default)]
    pub file_patterns: Vec<String>,
}

/// Definition of a tool command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCommand {
    /// Command name (added to `oxide` CLI).
    pub name: String,

    /// Command description.
    #[serde(default)]
    pub description: String,

    /// Command arguments.
    #[serde(default)]
    pub args: Vec<CommandArg>,
}

/// Definition of a command argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandArg {
    /// Argument name.
    pub name: String,

    /// Argument type.
    #[serde(rename = "type")]
    pub arg_type: String,

    /// Whether the argument is required.
    #[serde(default)]
    pub required: bool,

    /// Description.
    #[serde(default)]
    pub description: String,
}

/// Definition of a tool hook.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolHook {
    /// Hook event (pre-build, post-build, pre-compile, etc.).
    pub event: String,

    /// Hook priority (lower = runs first).
    #[serde(default)]
    pub priority: i32,

    /// Description.
    #[serde(default)]
    pub description: String,
}

/// Configuration specific to theme plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThemeConfig {
    /// Token files included.
    #[serde(default)]
    pub tokens: Vec<String>,

    /// Typography definitions.
    #[serde(default)]
    pub typography: Vec<String>,

    /// Color schemes.
    #[serde(default)]
    pub color_schemes: Vec<String>,

    /// Icon sets included.
    #[serde(default)]
    pub icons: Vec<String>,

    /// Preview images.
    #[serde(default)]
    pub previews: Vec<String>,

    /// Base theme this extends (if any).
    #[serde(default)]
    pub extends: Option<String>,
}

/// Configuration specific to design plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DesignConfig {
    /// Design template type.
    #[serde(default)]
    pub template_type: Option<String>,

    /// Extractable parts with tags.
    #[serde(default)]
    pub parts: Vec<DesignPart>,

    /// Preview images/screenshots.
    #[serde(default)]
    pub previews: Vec<String>,

    /// Demo URL (if available).
    #[serde(default)]
    pub demo_url: Option<String>,

    /// Required plugins.
    #[serde(default)]
    pub requires_plugins: Vec<String>,
}

/// Definition of an extractable design part.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DesignPart {
    /// Part name.
    pub name: String,

    /// Part tags for categorization.
    #[serde(default)]
    pub tags: Vec<String>,

    /// Part description.
    #[serde(default)]
    pub description: String,

    /// Path to the part's files.
    #[serde(default)]
    pub path: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_category_capabilities() {
        // UI should have no dangerous capabilities
        assert!(PluginCategory::Ui.allowed_capabilities().is_empty());

        // Native should have all capabilities
        let native_caps = PluginCategory::Native.allowed_capabilities();
        assert!(native_caps.contains(&Capability::FilesystemRead));
        assert!(native_caps.contains(&Capability::ProcessSpawn));

        // Theme/Design are data-only
        assert!(!PluginCategory::Theme.executes_code());
        assert!(!PluginCategory::Design.executes_code());
    }

    #[test]
    fn test_sandbox_requirements() {
        assert!(PluginCategory::Ui.requires_sandbox_for_community());
        assert!(PluginCategory::Native.requires_sandbox_for_community());
        assert!(!PluginCategory::Theme.requires_sandbox_for_community());
    }
}
