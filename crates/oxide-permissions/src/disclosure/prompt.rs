//! Permission prompt generation.
//!
//! Generates user-friendly permission prompts similar to mobile app
//! permission dialogs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::capabilities::{
    Capability, CapabilityCategory, CapabilityRegistry, PermissionManifest, RiskLevel,
};

/// A permission prompt to display to the user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PermissionPrompt {
    /// The capability being requested.
    pub capability: String,
    /// Human-readable title.
    pub title: String,
    /// Detailed description.
    pub description: String,
    /// Why the app needs this permission.
    pub reason: Option<String>,
    /// Risk level.
    pub risk_level: RiskLevel,
    /// Privacy implications.
    pub privacy_implications: Vec<String>,
    /// Whether the permission is required or optional.
    pub required: bool,
    /// Icon identifier for UI rendering.
    pub icon: String,
    /// Category for grouping.
    pub category: CapabilityCategory,
}

impl PermissionPrompt {
    /// Create a prompt for a capability.
    pub fn for_capability(capability: &str) -> Self {
        let cap = Capability::new(capability);
        let category = CapabilityCategory::from_capability(&cap);
        let registry = CapabilityRegistry::global();

        let (title, description, implications) = if let Some(registered) = registry.get(capability)
        {
            (
                registered.name.clone(),
                registered.description.clone(),
                registered.privacy_implications.clone(),
            )
        } else {
            (
                format_capability_name(capability),
                format!("Access to {}", capability),
                Vec::new(),
            )
        };

        Self {
            capability: capability.to_string(),
            title,
            description,
            reason: None,
            risk_level: category.risk_level(),
            privacy_implications: implications,
            required: false,
            icon: icon_for_category(category),
            category,
        }
    }

    /// Set the reason from the manifest.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Mark as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Generate a short summary for display.
    pub fn summary(&self) -> String {
        if let Some(reason) = &self.reason {
            format!("{}: {}", self.title, reason)
        } else {
            self.title.clone()
        }
    }

    /// Generate action text for the prompt button.
    pub fn action_text(&self) -> &'static str {
        if self.required {
            "Required"
        } else {
            "Allow"
        }
    }

    /// Check if this is a high-risk permission.
    pub fn is_high_risk(&self) -> bool {
        self.risk_level >= RiskLevel::High
    }
}

/// Format a capability name for human display.
fn format_capability_name(capability: &str) -> String {
    capability
        .split('.')
        .map(|part| {
            let mut chars: Vec<char> = part.chars().collect();
            if let Some(first) = chars.first_mut() {
                *first = first.to_ascii_uppercase();
            }
            chars.into_iter().collect::<String>()
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Get icon identifier for a category.
fn icon_for_category(category: CapabilityCategory) -> String {
    match category {
        CapabilityCategory::Filesystem => "folder".to_string(),
        CapabilityCategory::Keychain => "key".to_string(),
        CapabilityCategory::Network => "globe".to_string(),
        CapabilityCategory::Camera => "camera".to_string(),
        CapabilityCategory::Microphone => "mic".to_string(),
        CapabilityCategory::Screenshot => "screenshot".to_string(),
        CapabilityCategory::Clipboard => "clipboard".to_string(),
        CapabilityCategory::Background => "clock".to_string(),
        CapabilityCategory::Notifications => "bell".to_string(),
        CapabilityCategory::System => "settings".to_string(),
        CapabilityCategory::Location => "location".to_string(),
        CapabilityCategory::Custom => "puzzle".to_string(),
    }
}

/// Builder for creating first-launch permission prompts.
pub struct PromptBuilder {
    prompts: Vec<PermissionPrompt>,
    app_name: String,
    app_icon: Option<String>,
}

impl PromptBuilder {
    /// Create a new prompt builder.
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            prompts: Vec::new(),
            app_name: app_name.into(),
            app_icon: None,
        }
    }

    /// Set the app icon.
    pub fn with_icon(mut self, icon: impl Into<String>) -> Self {
        self.app_icon = Some(icon.into());
        self
    }

    /// Add a permission prompt.
    pub fn add_prompt(mut self, prompt: PermissionPrompt) -> Self {
        self.prompts.push(prompt);
        self
    }

    /// Add prompts from a permission manifest.
    pub fn from_manifest(mut self, manifest: &PermissionManifest) -> Self {
        for (cap_name, declaration) in &manifest.capabilities {
            let mut prompt = PermissionPrompt::for_capability(cap_name);

            if let Some(reason) = &declaration.reason {
                prompt = prompt.with_reason(reason.clone());
            }

            if declaration.required {
                prompt = prompt.required();
            }

            self.prompts.push(prompt);
        }

        // Add capabilities from permissions map that don't have declarations
        for caps in manifest.permissions.values() {
            for cap_name in caps {
                if !manifest.capabilities.contains_key(cap_name) {
                    self.prompts
                        .push(PermissionPrompt::for_capability(cap_name));
                }
            }
        }

        self
    }

    /// Build the prompt set.
    pub fn build(self) -> PromptSet {
        let mut by_category: HashMap<CapabilityCategory, Vec<PermissionPrompt>> = HashMap::new();

        for prompt in &self.prompts {
            by_category
                .entry(prompt.category)
                .or_default()
                .push(prompt.clone());
        }

        let max_risk = self
            .prompts
            .iter()
            .map(|p| p.risk_level)
            .max()
            .unwrap_or(RiskLevel::Low);

        let required_count = self.prompts.iter().filter(|p| p.required).count();
        let high_risk_count = self.prompts.iter().filter(|p| p.is_high_risk()).count();

        PromptSet {
            app_name: self.app_name,
            app_icon: self.app_icon,
            prompts: self.prompts,
            by_category,
            max_risk_level: max_risk,
            required_count,
            high_risk_count,
        }
    }
}

/// A set of permission prompts for display.
#[derive(Debug, Clone)]
pub struct PromptSet {
    /// App name.
    pub app_name: String,
    /// App icon.
    pub app_icon: Option<String>,
    /// All prompts.
    pub prompts: Vec<PermissionPrompt>,
    /// Prompts grouped by category.
    pub by_category: HashMap<CapabilityCategory, Vec<PermissionPrompt>>,
    /// Maximum risk level among all prompts.
    pub max_risk_level: RiskLevel,
    /// Number of required permissions.
    pub required_count: usize,
    /// Number of high-risk permissions.
    pub high_risk_count: usize,
}

impl PromptSet {
    /// Check if there are any prompts to show.
    pub fn is_empty(&self) -> bool {
        self.prompts.is_empty()
    }

    /// Get the total number of prompts.
    pub fn len(&self) -> usize {
        self.prompts.len()
    }

    /// Get required prompts only.
    pub fn required_prompts(&self) -> Vec<&PermissionPrompt> {
        self.prompts.iter().filter(|p| p.required).collect()
    }

    /// Get optional prompts only.
    pub fn optional_prompts(&self) -> Vec<&PermissionPrompt> {
        self.prompts.iter().filter(|p| !p.required).collect()
    }

    /// Get high-risk prompts only.
    pub fn high_risk_prompts(&self) -> Vec<&PermissionPrompt> {
        self.prompts.iter().filter(|p| p.is_high_risk()).collect()
    }

    /// Generate a header message for the prompt dialog.
    pub fn header_message(&self) -> String {
        if self.high_risk_count > 0 {
            format!(
                "{} requests access to {} permissions, including {} sensitive permissions.",
                self.app_name,
                self.prompts.len(),
                self.high_risk_count
            )
        } else {
            format!(
                "{} requests access to {} permissions.",
                self.app_name,
                self.prompts.len()
            )
        }
    }

    /// Generate a warning message if applicable.
    pub fn warning_message(&self) -> Option<String> {
        if self.max_risk_level >= RiskLevel::Critical {
            Some("This app requests access to highly sensitive resources. Review carefully before granting.".to_string())
        } else if self.max_risk_level >= RiskLevel::High {
            Some("This app requests access to sensitive resources.".to_string())
        } else {
            None
        }
    }

    /// Render as plain text for console/terminal.
    pub fn render_text(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("=== {} Permissions ===", self.app_name));
        lines.push(String::new());

        if let Some(warning) = self.warning_message() {
            lines.push(format!("WARNING: {}", warning));
            lines.push(String::new());
        }

        for (category, prompts) in &self.by_category {
            lines.push(format!("[{}]", category.description()));
            for prompt in prompts {
                let marker = if prompt.required { "*" } else { " " };
                let risk = match prompt.risk_level {
                    RiskLevel::Critical => " [CRITICAL]",
                    RiskLevel::High => " [HIGH RISK]",
                    _ => "",
                };

                lines.push(format!(
                    " {} {}: {}{}",
                    marker,
                    prompt.title,
                    prompt.reason.as_deref().unwrap_or("No reason provided"),
                    risk
                ));
            }
            lines.push(String::new());
        }

        if self.required_count > 0 {
            lines.push(format!("* = Required ({} total)", self.required_count));
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_for_capability() {
        let prompt = PermissionPrompt::for_capability("filesystem.read");

        assert_eq!(prompt.capability, "filesystem.read");
        assert_eq!(prompt.category, CapabilityCategory::Filesystem);
        assert!(prompt.risk_level >= RiskLevel::High);
    }

    #[test]
    fn test_prompt_builder() {
        let manifest_str = r#"
[capabilities]
"filesystem.read" = { reason = "Save user preferences", required = true }
"network.http" = { reason = "Fetch API data" }
"#;

        let manifest = PermissionManifest::from_str(manifest_str).unwrap();

        let prompt_set = PromptBuilder::new("TestApp").from_manifest(&manifest).build();

        assert_eq!(prompt_set.len(), 2);
        assert_eq!(prompt_set.required_count, 1);
    }

    #[test]
    fn test_prompt_set_rendering() {
        let prompt_set = PromptBuilder::new("TestApp")
            .add_prompt(PermissionPrompt::for_capability("filesystem.read").with_reason("Save files"))
            .add_prompt(PermissionPrompt::for_capability("network.http").with_reason("API calls"))
            .build();

        let text = prompt_set.render_text();
        assert!(text.contains("TestApp Permissions"));
        assert!(text.contains("Save files"));
    }

    #[test]
    fn test_format_capability_name() {
        assert_eq!(
            format_capability_name("filesystem.read"),
            "Filesystem Read"
        );
        assert_eq!(
            format_capability_name("network.http"),
            "Network Http"
        );
    }
}
