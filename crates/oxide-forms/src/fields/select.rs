//! Select/dropdown field for forms.

use serde::{Deserialize, Serialize};

/// Select option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    /// Option value
    pub value: String,
    /// Display label
    pub label: String,
    /// Whether option is disabled
    pub disabled: bool,
    /// Option group (for grouped selects)
    pub group: Option<String>,
}

impl SelectOption {
    /// Create a new select option
    pub fn new(value: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
            group: None,
        }
    }

    /// Set option group
    pub fn group(mut self, group: impl Into<String>) -> Self {
        self.group = Some(group.into());
        self
    }
}

/// Select field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectField {
    /// Field name
    pub name: String,
    /// Available options
    pub options: Vec<SelectOption>,
    /// Selected value(s)
    pub value: SelectValue,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Whether field is required
    pub required: bool,
    /// Whether field is disabled
    pub disabled: bool,
    /// Whether to allow multiple selections
    pub multiple: bool,
    /// Whether to allow search/filtering
    pub searchable: bool,
}

/// Select value (single or multiple)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SelectValue {
    /// No selection
    None,
    /// Single value
    Single(String),
    /// Multiple values
    Multiple(Vec<String>),
}

impl Default for SelectValue {
    fn default() -> Self {
        SelectValue::None
    }
}

impl Default for SelectField {
    fn default() -> Self {
        Self {
            name: String::new(),
            options: Vec::new(),
            value: SelectValue::None,
            placeholder: None,
            required: false,
            disabled: false,
            multiple: false,
            searchable: false,
        }
    }
}

impl SelectField {
    /// Create a new select field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Add an option
    pub fn option(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.options.push(SelectOption::new(value, label));
        self
    }

    /// Set placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set single value
    pub fn value(mut self, value: impl Into<String>) -> Self {
        self.value = SelectValue::Single(value.into());
        self
    }

    /// Set multiple values
    pub fn values(mut self, values: Vec<String>) -> Self {
        self.value = SelectValue::Multiple(values);
        self
    }

    /// Set required
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Enable multiple selection
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    /// Enable search
    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    /// Validate the field
    pub fn validate(&self) -> Result<(), String> {
        match &self.value {
            SelectValue::None if self.required => {
                Err("Selection is required".to_string())
            }
            SelectValue::Single(v) => {
                if !self.options.iter().any(|o| &o.value == v) {
                    Err("Invalid selection".to_string())
                } else {
                    Ok(())
                }
            }
            SelectValue::Multiple(vs) => {
                if self.required && vs.is_empty() {
                    return Err("Selection is required".to_string());
                }
                for v in vs {
                    if !self.options.iter().any(|o| &o.value == v) {
                        return Err(format!("Invalid selection: {}", v));
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
