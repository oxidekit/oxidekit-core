//! Capability registry for managing known capabilities.
//!
//! The registry provides metadata about all supported capabilities,
//! including descriptions, risk levels, and enforcement requirements.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::types::{Capability, CapabilityCategory, RiskLevel};

/// A registered capability with full metadata.
#[derive(Debug, Clone)]
pub struct RegisteredCapability {
    /// The capability identifier.
    pub capability: Capability,
    /// Human-readable name.
    pub name: String,
    /// Detailed description of what this capability allows.
    pub description: String,
    /// The capability category.
    pub category: CapabilityCategory,
    /// Risk level.
    pub risk_level: RiskLevel,
    /// Whether this capability can be enforced at runtime.
    pub enforceable: bool,
    /// Parent capability (if hierarchical).
    pub parent: Option<Capability>,
    /// Whether user consent prompt is recommended.
    pub prompt_recommended: bool,
    /// Potential privacy implications.
    pub privacy_implications: Vec<String>,
}

impl RegisteredCapability {
    /// Create a new registered capability builder.
    pub fn builder(capability: impl Into<String>) -> RegisteredCapabilityBuilder {
        RegisteredCapabilityBuilder::new(capability)
    }
}

/// Builder for creating registered capabilities.
pub struct RegisteredCapabilityBuilder {
    capability: Capability,
    name: String,
    description: String,
    category: CapabilityCategory,
    risk_level: RiskLevel,
    enforceable: bool,
    parent: Option<Capability>,
    prompt_recommended: bool,
    privacy_implications: Vec<String>,
}

impl RegisteredCapabilityBuilder {
    /// Create a new builder.
    pub fn new(capability: impl Into<String>) -> Self {
        let cap = Capability::new(capability);
        let category = CapabilityCategory::from_capability(&cap);
        Self {
            capability: cap,
            name: String::new(),
            description: String::new(),
            category,
            risk_level: category.risk_level(),
            enforceable: true,
            parent: None,
            prompt_recommended: false,
            privacy_implications: Vec::new(),
        }
    }

    /// Set the human-readable name.
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Set the description.
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the risk level.
    pub fn risk_level(mut self, level: RiskLevel) -> Self {
        self.risk_level = level;
        self
    }

    /// Set whether this capability is enforceable.
    pub fn enforceable(mut self, enforceable: bool) -> Self {
        self.enforceable = enforceable;
        self
    }

    /// Set the parent capability.
    pub fn parent(mut self, parent: impl Into<String>) -> Self {
        self.parent = Some(Capability::new(parent));
        self
    }

    /// Set whether user prompt is recommended.
    pub fn prompt_recommended(mut self, recommended: bool) -> Self {
        self.prompt_recommended = recommended;
        self
    }

    /// Add a privacy implication.
    pub fn privacy_implication(mut self, implication: impl Into<String>) -> Self {
        self.privacy_implications.push(implication.into());
        self
    }

    /// Build the registered capability.
    pub fn build(self) -> RegisteredCapability {
        RegisteredCapability {
            capability: self.capability,
            name: self.name,
            description: self.description,
            category: self.category,
            risk_level: self.risk_level,
            enforceable: self.enforceable,
            parent: self.parent,
            prompt_recommended: self.prompt_recommended,
            privacy_implications: self.privacy_implications,
        }
    }
}

/// The global capability registry.
pub struct CapabilityRegistry {
    capabilities: HashMap<String, RegisteredCapability>,
}

impl CapabilityRegistry {
    /// Get the global capability registry instance.
    pub fn global() -> &'static Self {
        static REGISTRY: OnceLock<CapabilityRegistry> = OnceLock::new();
        REGISTRY.get_or_init(Self::new_with_defaults)
    }

    /// Create a new empty registry.
    pub fn new() -> Self {
        Self {
            capabilities: HashMap::new(),
        }
    }

    /// Create a new registry with default OxideKit capabilities.
    pub fn new_with_defaults() -> Self {
        let mut registry = Self::new();
        registry.register_defaults();
        registry
    }

    /// Register a capability.
    pub fn register(&mut self, capability: RegisteredCapability) {
        self.capabilities
            .insert(capability.capability.as_str().to_string(), capability);
    }

    /// Get a registered capability by name.
    pub fn get(&self, capability: &str) -> Option<&RegisteredCapability> {
        self.capabilities.get(capability)
    }

    /// Get all registered capabilities.
    pub fn all(&self) -> impl Iterator<Item = &RegisteredCapability> {
        self.capabilities.values()
    }

    /// Get capabilities by category.
    pub fn by_category(&self, category: CapabilityCategory) -> Vec<&RegisteredCapability> {
        self.capabilities
            .values()
            .filter(|c| c.category == category)
            .collect()
    }

    /// Get capabilities by risk level.
    pub fn by_risk_level(&self, min_level: RiskLevel) -> Vec<&RegisteredCapability> {
        self.capabilities
            .values()
            .filter(|c| c.risk_level >= min_level)
            .collect()
    }

    /// Check if a capability is registered.
    pub fn is_registered(&self, capability: &str) -> bool {
        self.capabilities.contains_key(capability)
    }

    /// Register the default OxideKit capabilities.
    fn register_defaults(&mut self) {
        // Filesystem capabilities
        self.register(
            RegisteredCapability::builder("filesystem")
                .name("Full Filesystem Access")
                .description("Read and write any file on the system")
                .risk_level(RiskLevel::Critical)
                .prompt_recommended(true)
                .privacy_implication("Can access any file including documents and downloads")
                .privacy_implication("Can modify or delete files")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("filesystem.read")
                .name("Filesystem Read")
                .description("Read files from the filesystem")
                .parent("filesystem")
                .prompt_recommended(true)
                .privacy_implication("Can read documents and user data")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("filesystem.write")
                .name("Filesystem Write")
                .description("Write files to the filesystem")
                .parent("filesystem")
                .prompt_recommended(true)
                .privacy_implication("Can create, modify, or delete files")
                .build(),
        );

        // Keychain capabilities
        self.register(
            RegisteredCapability::builder("keychain")
                .name("Keychain & Secrets")
                .description("Access to system keychain and secure storage")
                .risk_level(RiskLevel::Critical)
                .prompt_recommended(true)
                .privacy_implication("Can access stored passwords and secrets")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("keychain.access")
                .name("Keychain Access")
                .description("Read and write keychain entries")
                .parent("keychain")
                .prompt_recommended(true)
                .privacy_implication("Can store and retrieve secure credentials")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("keychain.read")
                .name("Keychain Read")
                .description("Read keychain entries")
                .parent("keychain")
                .risk_level(RiskLevel::High)
                .prompt_recommended(true)
                .privacy_implication("Can read stored passwords")
                .build(),
        );

        // Network capabilities
        self.register(
            RegisteredCapability::builder("network")
                .name("Full Network Access")
                .description("Unrestricted network access")
                .risk_level(RiskLevel::Critical)
                .prompt_recommended(true)
                .privacy_implication("Can connect to any server")
                .privacy_implication("Can transmit data without restriction")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("network.http")
                .name("HTTP Network Access")
                .description("Make HTTP/HTTPS requests")
                .parent("network")
                .prompt_recommended(true)
                .privacy_implication("Can send and receive data over HTTP")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("network.websocket")
                .name("WebSocket Access")
                .description("Establish WebSocket connections")
                .parent("network")
                .prompt_recommended(true)
                .privacy_implication("Can maintain persistent connections")
                .build(),
        );

        // Camera capabilities
        self.register(
            RegisteredCapability::builder("camera")
                .name("Camera Access")
                .description("Full camera access")
                .prompt_recommended(true)
                .privacy_implication("Can capture photos and video")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("camera.capture")
                .name("Camera Capture")
                .description("Capture still images from camera")
                .parent("camera")
                .prompt_recommended(true)
                .privacy_implication("Can take photos")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("camera.stream")
                .name("Camera Stream")
                .description("Stream video from camera")
                .parent("camera")
                .prompt_recommended(true)
                .privacy_implication("Can record video")
                .build(),
        );

        // Microphone capabilities
        self.register(
            RegisteredCapability::builder("microphone")
                .name("Microphone Access")
                .description("Full microphone access")
                .prompt_recommended(true)
                .privacy_implication("Can record audio")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("microphone.record")
                .name("Microphone Record")
                .description("Record audio from microphone")
                .parent("microphone")
                .prompt_recommended(true)
                .privacy_implication("Can capture audio recordings")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("microphone.stream")
                .name("Microphone Stream")
                .description("Stream audio from microphone")
                .parent("microphone")
                .prompt_recommended(true)
                .privacy_implication("Can stream audio in real-time")
                .build(),
        );

        // Screenshot capability
        self.register(
            RegisteredCapability::builder("screenshot.capture")
                .name("Screenshot Capture")
                .description("Capture screenshots of the screen")
                .prompt_recommended(true)
                .privacy_implication("Can capture screen contents")
                .privacy_implication("May capture sensitive information on screen")
                .build(),
        );

        // Clipboard capabilities
        self.register(
            RegisteredCapability::builder("clipboard")
                .name("Clipboard Access")
                .description("Read and write clipboard")
                .risk_level(RiskLevel::Medium)
                .privacy_implication("Can access copied data")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("clipboard.read")
                .name("Clipboard Read")
                .description("Read from clipboard")
                .parent("clipboard")
                .risk_level(RiskLevel::Medium)
                .privacy_implication("Can read copied text and data")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("clipboard.write")
                .name("Clipboard Write")
                .description("Write to clipboard")
                .parent("clipboard")
                .risk_level(RiskLevel::Low)
                .build(),
        );

        // Background capabilities
        self.register(
            RegisteredCapability::builder("background.task")
                .name("Background Tasks")
                .description("Run tasks in the background")
                .risk_level(RiskLevel::Medium)
                .privacy_implication("Can perform operations when app is not focused")
                .build(),
        );

        self.register(
            RegisteredCapability::builder("background.service")
                .name("Background Service")
                .description("Run persistent background services")
                .risk_level(RiskLevel::Medium)
                .prompt_recommended(true)
                .privacy_implication("Can run continuously in background")
                .build(),
        );

        // Notifications capability
        self.register(
            RegisteredCapability::builder("notifications")
                .name("System Notifications")
                .description("Display system notifications")
                .risk_level(RiskLevel::Low)
                .build(),
        );

        // System info capability
        self.register(
            RegisteredCapability::builder("system.info")
                .name("System Information")
                .description("Read basic system information")
                .risk_level(RiskLevel::Low)
                .privacy_implication("Can read OS version and hardware info")
                .build(),
        );

        // Location capability
        self.register(
            RegisteredCapability::builder("location")
                .name("Location Access")
                .description("Access device location")
                .prompt_recommended(true)
                .privacy_implication("Can track physical location")
                .build(),
        );
    }
}

impl Default for CapabilityRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_defaults() {
        let registry = CapabilityRegistry::new_with_defaults();

        assert!(registry.is_registered("filesystem.read"));
        assert!(registry.is_registered("network.http"));
        assert!(registry.is_registered("keychain.access"));
    }

    #[test]
    fn test_get_capability() {
        let registry = CapabilityRegistry::new_with_defaults();

        let cap = registry.get("filesystem.read").unwrap();
        assert_eq!(cap.category, CapabilityCategory::Filesystem);
        assert_eq!(cap.risk_level, RiskLevel::High);
    }

    #[test]
    fn test_by_category() {
        let registry = CapabilityRegistry::new_with_defaults();

        let network_caps = registry.by_category(CapabilityCategory::Network);
        assert!(!network_caps.is_empty());
        assert!(network_caps
            .iter()
            .any(|c| c.capability.as_str() == "network.http"));
    }

    #[test]
    fn test_by_risk_level() {
        let registry = CapabilityRegistry::new_with_defaults();

        let critical_caps = registry.by_risk_level(RiskLevel::Critical);
        assert!(!critical_caps.is_empty());
    }
}
