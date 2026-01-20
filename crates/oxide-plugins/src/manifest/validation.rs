//! Manifest validation logic.
//!
//! Ensures plugin manifests are well-formed and follow OxideKit conventions.

use crate::error::{PluginError, PluginResult};
use crate::permissions::Capability;
use super::schema::PluginManifest;
use super::category::PluginCategory;

/// Validates plugin manifests for correctness and security.
pub struct ManifestValidator {
    /// Whether to perform strict validation.
    strict: bool,
}

impl ManifestValidator {
    /// Create a new validator with default settings.
    pub fn new() -> Self {
        Self { strict: false }
    }

    /// Create a strict validator that fails on warnings.
    pub fn strict() -> Self {
        Self { strict: true }
    }

    /// Validate a manifest.
    pub fn validate(&self, manifest: &PluginManifest) -> PluginResult<ValidationResult> {
        let mut result = ValidationResult::new();

        // Check required fields
        self.validate_required_fields(manifest, &mut result)?;

        // Check kind-specific configuration
        self.validate_kind_config(manifest, &mut result)?;

        // Check capabilities match kind
        self.validate_capabilities(manifest, &mut result)?;

        // Check dependencies
        self.validate_dependencies(manifest, &mut result)?;

        // Check naming conventions
        self.validate_naming(manifest, &mut result)?;

        // If strict mode, treat warnings as errors
        if self.strict && !result.warnings.is_empty() {
            return Err(PluginError::InvalidManifest(
                format!("Strict validation failed: {}", result.warnings.join(", "))
            ));
        }

        // If there are errors, return an error
        if !result.errors.is_empty() {
            return Err(PluginError::InvalidManifest(result.errors.join("; ")));
        }

        Ok(result)
    }

    /// Validate required fields are present.
    fn validate_required_fields(
        &self,
        manifest: &PluginManifest,
        result: &mut ValidationResult,
    ) -> PluginResult<()> {
        // ID is already validated by PluginId parsing

        // Publisher
        if manifest.plugin.publisher.is_empty() {
            result.add_error("publisher is required");
        }

        // Description
        if manifest.plugin.description.is_empty() {
            result.add_error("description is required");
        }

        // License
        if manifest.plugin.license.is_empty() {
            result.add_warning("license should be specified (SPDX format recommended)");
        }

        // Version is validated by semver parsing

        Ok(())
    }

    /// Validate kind-specific configuration.
    fn validate_kind_config(
        &self,
        manifest: &PluginManifest,
        result: &mut ValidationResult,
    ) -> PluginResult<()> {
        if !manifest.has_valid_kind_config() {
            result.add_error(&format!(
                "missing [{}] configuration section for plugin kind '{}'",
                manifest.plugin.kind,
                manifest.plugin.kind
            ));
            return Ok(());
        }

        // Kind-specific validations
        match manifest.plugin.kind {
            PluginCategory::Ui => {
                if let Some(ui) = &manifest.ui {
                    if ui.components.is_empty() && !ui.is_pack {
                        result.add_warning("UI plugin has no components defined");
                    }
                }
            }
            PluginCategory::Native => {
                if let Some(native) = &manifest.native {
                    if native.capabilities.is_empty() {
                        result.add_warning("Native plugin has no capabilities defined");
                    }
                }
            }
            PluginCategory::Service => {
                if let Some(service) = &manifest.service {
                    if service.entrypoints.is_empty() {
                        result.add_warning("Service plugin has no entrypoints defined");
                    }
                }
            }
            PluginCategory::Tooling => {
                if let Some(tooling) = &manifest.tooling {
                    if tooling.commands.is_empty() && tooling.hooks.is_empty() {
                        result.add_warning("Tooling plugin has no commands or hooks defined");
                    }
                }
            }
            PluginCategory::Theme => {
                if let Some(theme) = &manifest.theme {
                    if theme.tokens.is_empty() && theme.typography.is_empty() {
                        result.add_warning("Theme plugin has no tokens or typography defined");
                    }
                }
            }
            PluginCategory::Design => {
                if let Some(design) = &manifest.design {
                    if design.parts.is_empty() {
                        result.add_warning("Design plugin has no parts defined");
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate capabilities are appropriate for the plugin kind.
    fn validate_capabilities(
        &self,
        manifest: &PluginManifest,
        result: &mut ValidationResult,
    ) -> PluginResult<()> {
        let capabilities = manifest.required_capabilities();
        let category = manifest.plugin.kind;
        let allowed = category.allowed_capabilities();

        for cap_str in &capabilities {
            // Try to parse the capability
            if let Ok(cap) = cap_str.parse::<Capability>() {
                if !allowed.contains(&cap) {
                    result.add_error(&format!(
                        "capability '{}' is not allowed for {} plugins",
                        cap_str, category
                    ));
                }
            } else {
                result.add_warning(&format!("unknown capability: {}", cap_str));
            }
        }

        // Check for suspicious capability combinations
        if capabilities.iter().any(|c| c == "process.spawn") {
            if capabilities.iter().any(|c| c == "network.http") {
                result.add_warning(
                    "plugin requests both process.spawn and network.http - this combination requires extra scrutiny"
                );
            }
        }

        Ok(())
    }

    /// Validate dependencies.
    fn validate_dependencies(
        &self,
        manifest: &PluginManifest,
        result: &mut ValidationResult,
    ) -> PluginResult<()> {
        // Check for self-dependency
        for dep in &manifest.dependencies.plugins {
            if dep.id == manifest.plugin.id.full_name() {
                result.add_error("plugin cannot depend on itself");
            }
        }

        // Check for circular dependency indicators
        // (Full circular dependency detection requires the resolver)

        Ok(())
    }

    /// Validate naming conventions.
    fn validate_naming(
        &self,
        manifest: &PluginManifest,
        result: &mut ValidationResult,
    ) -> PluginResult<()> {
        // Check namespace matches kind
        let namespace = manifest.plugin.id.namespace();
        let kind = manifest.plugin.kind;

        let expected_namespace = match kind {
            PluginCategory::Ui => "ui",
            PluginCategory::Native => "native",
            PluginCategory::Service => {
                // Services can be in auth, db, data, or other namespaces
                return Ok(());
            }
            PluginCategory::Tooling => "tool",
            PluginCategory::Theme => "theme",
            PluginCategory::Design => "design",
        };

        if namespace.as_str() != expected_namespace {
            result.add_warning(&format!(
                "plugin kind '{}' typically uses '{}' namespace, but uses '{}' instead",
                kind, expected_namespace, namespace
            ));
        }

        Ok(())
    }
}

impl Default for ManifestValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of manifest validation.
#[derive(Debug, Clone, Default)]
pub struct ValidationResult {
    /// Validation errors.
    pub errors: Vec<String>,
    /// Validation warnings.
    pub warnings: Vec<String>,
}

impl ValidationResult {
    /// Create a new empty result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an error.
    pub fn add_error(&mut self, msg: &str) {
        self.errors.push(msg.to_string());
    }

    /// Add a warning.
    pub fn add_warning(&mut self, msg: &str) {
        self.warnings.push(msg.to_string());
    }

    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Check if validation passed without warnings.
    pub fn is_clean(&self) -> bool {
        self.errors.is_empty() && self.warnings.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = ManifestValidator::new();
        assert!(!validator.strict);

        let strict = ManifestValidator::strict();
        assert!(strict.strict);
    }
}
