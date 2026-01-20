//! Date field for forms.

use serde::{Deserialize, Serialize};

/// Date field configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateField {
    /// Field name
    pub name: String,
    /// Current value
    pub value: Option<String>,
    /// Minimum date (ISO format)
    pub min: Option<String>,
    /// Maximum date (ISO format)
    pub max: Option<String>,
    /// Date format
    pub format: DateFormat,
    /// Whether field is required
    pub required: bool,
    /// Whether field is disabled
    pub disabled: bool,
}

/// Date format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum DateFormat {
    /// ISO 8601 format (YYYY-MM-DD)
    #[default]
    Iso,
    /// US format (MM/DD/YYYY)
    Us,
    /// European format (DD/MM/YYYY)
    European,
}

impl Default for DateField {
    fn default() -> Self {
        Self {
            name: String::new(),
            value: None,
            min: None,
            max: None,
            format: DateFormat::default(),
            required: false,
            disabled: false,
        }
    }
}

impl DateField {
    /// Create a new date field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    /// Set minimum date
    pub fn min(mut self, min: impl Into<String>) -> Self {
        self.min = Some(min.into());
        self
    }

    /// Set maximum date
    pub fn max(mut self, max: impl Into<String>) -> Self {
        self.max = Some(max.into());
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
            return Err("Date is required".to_string());
        }
        Ok(())
    }
}
