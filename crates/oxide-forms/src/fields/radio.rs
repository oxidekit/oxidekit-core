//! Radio button field for forms.

use serde::{Deserialize, Serialize};

/// Radio button option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioOption {
    /// Option value
    pub value: String,
    /// Display label
    pub label: String,
    /// Whether option is disabled
    pub disabled: bool,
}

impl RadioOption {
    /// Create a new radio option
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }
}

/// Radio button group field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RadioField {
    /// Field name
    pub name: String,
    /// Available options
    pub options: Vec<RadioOption>,
    /// Selected value
    pub value: Option<String>,
    /// Whether field is required
    pub required: bool,
    /// Whether field is disabled
    pub disabled: bool,
    /// Layout direction
    pub direction: RadioDirection,
}

/// Radio button layout direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum RadioDirection {
    /// Vertical layout
    #[default]
    Vertical,
    /// Horizontal layout
    Horizontal,
}

impl Default for RadioField {
    fn default() -> Self {
        Self {
            name: String::new(),
            options: Vec::new(),
            value: None,
            required: false,
            disabled: false,
            direction: RadioDirection::default(),
        }
    }
}

impl RadioField {
    /// Create a new radio field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Add an option
    pub fn option(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(RadioOption::new(value, label));
        self
    }

    /// Set selected value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Validate the field
    pub fn validate(&self) -> Result<(), String> {
        if self.required && self.value.is_none() {
            return Err("Selection is required".to_string());
        }
        if let Some(ref v) = self.value {
            if !self.options.iter().any(|o| &o.value == v) {
                return Err("Invalid selection".to_string());
            }
        }
        Ok(())
    }
}
