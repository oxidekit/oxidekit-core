//! Validation Mode Configuration
//!
//! Defines when validation should be triggered for form fields.

use serde::{Deserialize, Serialize};

/// Defines when validation should be triggered
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationMode {
    /// Validate as the user types (on every change)
    OnChange,

    /// Validate when the field loses focus
    #[default]
    OnBlur,

    /// Validate only when the form is submitted
    OnSubmit,

    /// Validate on blur, then on change after first validation
    OnBlurThenChange,

    /// Validate on submit, then on change after first validation
    OnSubmitThenChange,

    /// Never validate automatically (manual validation only)
    Manual,
}

impl ValidationMode {
    /// Check if validation should run on value change
    pub fn validates_on_change(&self, has_been_validated: bool) -> bool {
        match self {
            Self::OnChange => true,
            Self::OnBlurThenChange | Self::OnSubmitThenChange => has_been_validated,
            _ => false,
        }
    }

    /// Check if validation should run on blur
    pub fn validates_on_blur(&self, has_been_validated: bool) -> bool {
        match self {
            Self::OnBlur | Self::OnBlurThenChange => true,
            Self::OnChange => true, // Also validate on blur for OnChange mode
            Self::OnSubmitThenChange => has_been_validated,
            _ => false,
        }
    }

    /// Check if validation should run on submit
    pub fn validates_on_submit(&self) -> bool {
        // All modes except Manual validate on submit
        !matches!(self, Self::Manual)
    }
}

/// Configuration for validation behavior
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// When to trigger validation
    pub mode: ValidationMode,

    /// Debounce delay in milliseconds for OnChange validation
    #[serde(default = "default_debounce_ms")]
    pub debounce_ms: u64,

    /// Whether to validate empty fields (for required validation)
    #[serde(default = "default_true")]
    pub validate_empty: bool,

    /// Whether to stop on first error
    #[serde(default)]
    pub stop_on_first_error: bool,

    /// Whether to show success state
    #[serde(default = "default_true")]
    pub show_success: bool,
}

fn default_debounce_ms() -> u64 {
    300
}

fn default_true() -> bool {
    true
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self {
            mode: ValidationMode::default(),
            debounce_ms: default_debounce_ms(),
            validate_empty: true,
            stop_on_first_error: false,
            show_success: true,
        }
    }
}

impl ValidationConfig {
    /// Create a new validation config with the given mode
    pub fn new(mode: ValidationMode) -> Self {
        Self {
            mode,
            ..Default::default()
        }
    }

    /// Set the debounce delay
    pub fn with_debounce(mut self, ms: u64) -> Self {
        self.debounce_ms = ms;
        self
    }

    /// Set whether to validate empty fields
    pub fn validate_empty(mut self, validate: bool) -> Self {
        self.validate_empty = validate;
        self
    }

    /// Set whether to stop on first error
    pub fn stop_on_first_error(mut self, stop: bool) -> Self {
        self.stop_on_first_error = stop;
        self
    }

    /// Set whether to show success state
    pub fn show_success(mut self, show: bool) -> Self {
        self.show_success = show;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation_mode_default() {
        assert_eq!(ValidationMode::default(), ValidationMode::OnBlur);
    }

    #[test]
    fn test_validates_on_change() {
        assert!(ValidationMode::OnChange.validates_on_change(false));
        assert!(!ValidationMode::OnBlur.validates_on_change(false));
        assert!(!ValidationMode::OnSubmit.validates_on_change(false));
        assert!(!ValidationMode::OnBlurThenChange.validates_on_change(false));
        assert!(ValidationMode::OnBlurThenChange.validates_on_change(true));
        assert!(!ValidationMode::OnSubmitThenChange.validates_on_change(false));
        assert!(ValidationMode::OnSubmitThenChange.validates_on_change(true));
    }

    #[test]
    fn test_validates_on_blur() {
        assert!(ValidationMode::OnChange.validates_on_blur(false));
        assert!(ValidationMode::OnBlur.validates_on_blur(false));
        assert!(!ValidationMode::OnSubmit.validates_on_blur(false));
        assert!(ValidationMode::OnBlurThenChange.validates_on_blur(false));
        assert!(!ValidationMode::OnSubmitThenChange.validates_on_blur(false));
        assert!(ValidationMode::OnSubmitThenChange.validates_on_blur(true));
    }

    #[test]
    fn test_validates_on_submit() {
        assert!(ValidationMode::OnChange.validates_on_submit());
        assert!(ValidationMode::OnBlur.validates_on_submit());
        assert!(ValidationMode::OnSubmit.validates_on_submit());
        assert!(!ValidationMode::Manual.validates_on_submit());
    }

    #[test]
    fn test_validation_config_builder() {
        let config = ValidationConfig::new(ValidationMode::OnChange)
            .with_debounce(500)
            .validate_empty(false)
            .stop_on_first_error(true)
            .show_success(false);

        assert_eq!(config.mode, ValidationMode::OnChange);
        assert_eq!(config.debounce_ms, 500);
        assert!(!config.validate_empty);
        assert!(config.stop_on_first_error);
        assert!(!config.show_success);
    }
}
