//! Number Field Types
//!
//! Provides NumberField for numeric input with step, min, and max support.

use crate::error::ValidationError;
use crate::field::FieldState;
use crate::mode::{ValidationConfig, ValidationMode};
use crate::validator::{ValidatorChain, Validator};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// A numeric input field
pub struct NumberField<T>
where
    T: Clone + PartialEq + Send + Sync + 'static + fmt::Display + FromStr,
{
    /// Field name
    pub name: String,
    /// Current state
    pub state: FieldState<Option<T>>,
    /// Validation configuration
    pub config: ValidationConfig,
    /// Validators
    validators: ValidatorChain<Option<T>>,
    /// Field label
    pub label: Option<String>,
    /// Placeholder text
    pub placeholder: Option<String>,
    /// Help text
    pub help_text: Option<String>,
    /// Minimum value
    pub min: Option<T>,
    /// Maximum value
    pub max: Option<T>,
    /// Step value
    pub step: Option<T>,
    /// Whether the field is disabled
    pub disabled: bool,
    /// Whether the field is read-only
    pub read_only: bool,
}

impl<T> NumberField<T>
where
    T: Clone + PartialEq + Send + Sync + 'static + fmt::Display + FromStr + Default,
{
    /// Create a new number field
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(None),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            placeholder: None,
            help_text: None,
            min: None,
            max: None,
            step: None,
            disabled: false,
            read_only: false,
        }
    }

    /// Create a new number field with an initial value
    pub fn with_value(name: impl Into<String>, initial_value: T) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(Some(initial_value)),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            placeholder: None,
            help_text: None,
            min: None,
            max: None,
            step: None,
            disabled: false,
            read_only: false,
        }
    }

    /// Set the validation mode
    pub fn validate_on(mut self, mode: ValidationMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Add a validator
    pub fn validate<V: Validator<Option<T>> + 'static>(mut self, validator: V) -> Self {
        self.validators = self.validators.add(validator);
        self
    }

    /// Set the label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Set the placeholder
    pub fn placeholder(mut self, placeholder: impl Into<String>) -> Self {
        self.placeholder = Some(placeholder.into());
        self
    }

    /// Set the help text
    pub fn help_text(mut self, text: impl Into<String>) -> Self {
        self.help_text = Some(text.into());
        self
    }

    /// Set the minimum value
    pub fn min(mut self, min: T) -> Self {
        self.min = Some(min);
        self
    }

    /// Set the maximum value
    pub fn max(mut self, max: T) -> Self {
        self.max = Some(max);
        self
    }

    /// Set the step value
    pub fn step(mut self, step: T) -> Self {
        self.step = Some(step);
        self
    }

    /// Set whether disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Get the current value
    pub fn value(&self) -> Option<&T> {
        self.state.value.as_ref()
    }

    /// Set the field value
    pub fn set_value(&mut self, value: Option<T>) {
        if self.disabled || self.read_only {
            return;
        }
        self.state.set_value(value);
    }

    /// Set the field value from a string
    pub fn set_value_from_str(&mut self, value: &str) {
        if self.disabled || self.read_only {
            return;
        }
        let parsed = if value.trim().is_empty() {
            None
        } else {
            value.trim().parse().ok()
        };
        self.state.set_value(parsed);
    }

    /// Handle change event
    pub fn on_change(&mut self, value: Option<T>) {
        self.set_value(value);
        if self.config.mode.validates_on_change(self.state.validated) {
            self.run_validation();
        }
    }

    /// Handle change event from string input
    pub fn on_change_str(&mut self, value: &str) {
        self.set_value_from_str(value);
        if self.config.mode.validates_on_change(self.state.validated) {
            self.run_validation();
        }
    }

    /// Handle focus event
    pub fn on_focus(&mut self) {
        self.state.mark_focused();
    }

    /// Handle blur event
    pub fn on_blur(&mut self) {
        self.state.mark_blurred();
        if self.config.mode.validates_on_blur(self.state.validated) {
            self.run_validation();
        }
    }

    /// Run validation
    pub fn run_validation(&mut self) -> bool {
        if self.disabled {
            self.state.clear_errors();
            return true;
        }

        let errors = if self.config.stop_on_first_error {
            self.validators
                .validate_first(&self.state.value, &self.name)
                .into_iter()
                .collect()
        } else {
            self.validators.validate(&self.state.value, &self.name)
        };

        self.state.set_errors(errors);
        self.state.is_valid()
    }

    /// Reset the field
    pub fn reset(&mut self) {
        self.state.reset();
    }

    /// Clear the field
    pub fn clear(&mut self) {
        self.state.set_value(None);
        self.state.dirty = self.state.value.is_some();
        self.state.clear_errors();
    }

    /// Check if valid
    pub fn is_valid(&self) -> bool {
        self.state.is_valid()
    }

    /// Check if dirty
    pub fn is_dirty(&self) -> bool {
        self.state.dirty
    }

    /// Check if touched
    pub fn is_touched(&self) -> bool {
        self.state.touched
    }

    /// Get errors
    pub fn errors(&self) -> &[ValidationError] {
        &self.state.errors
    }

    /// Get first error
    pub fn first_error(&self) -> Option<&str> {
        self.state.first_error()
    }

    /// Should show error
    pub fn should_show_error(&self) -> bool {
        self.state.should_show_error(self.config.mode)
    }

    /// Get the value as a string for display
    pub fn value_as_string(&self) -> String {
        self.state
            .value
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default()
    }
}

impl<T> fmt::Debug for NumberField<T>
where
    T: Clone + PartialEq + Send + Sync + 'static + fmt::Display + FromStr + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("NumberField")
            .field("name", &self.name)
            .field("value", &self.state.value)
            .field("min", &self.min)
            .field("max", &self.max)
            .field("step", &self.step)
            .field("disabled", &self.disabled)
            .finish()
    }
}

/// Type alias for integer number fields
pub type IntegerField = NumberField<i64>;

/// Type alias for floating-point number fields
pub type FloatField = NumberField<f64>;

/// Validator for required number fields
pub struct RequiredNumber;

impl<T> Validator<Option<T>> for RequiredNumber
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    fn validate(&self, value: &Option<T>, field_name: &str) -> Option<ValidationError> {
        if value.is_none() {
            Some(ValidationError::required(field_name))
        } else {
            None
        }
    }

    fn description(&self) -> String {
        "RequiredNumber".to_string()
    }
}

/// Validator for minimum number value
pub struct MinNumber<T>(pub T);

impl<T> Validator<Option<T>> for MinNumber<T>
where
    T: Clone + PartialEq + PartialOrd + Send + Sync + 'static + fmt::Display,
{
    fn validate(&self, value: &Option<T>, field_name: &str) -> Option<ValidationError> {
        match value {
            Some(v) if *v < self.0 => {
                Some(ValidationError::min_value(field_name, &self.0, v))
            }
            _ => None,
        }
    }

    fn description(&self) -> String {
        format!("MinNumber({})", self.0)
    }
}

/// Validator for maximum number value
pub struct MaxNumber<T>(pub T);

impl<T> Validator<Option<T>> for MaxNumber<T>
where
    T: Clone + PartialEq + PartialOrd + Send + Sync + 'static + fmt::Display,
{
    fn validate(&self, value: &Option<T>, field_name: &str) -> Option<ValidationError> {
        match value {
            Some(v) if *v > self.0 => {
                Some(ValidationError::max_value(field_name, &self.0, v))
            }
            _ => None,
        }
    }

    fn description(&self) -> String {
        format!("MaxNumber({})", self.0)
    }
}

/// Validator for number range
pub struct RangeNumber<T> {
    pub min: T,
    pub max: T,
}

impl<T> RangeNumber<T> {
    pub fn new(min: T, max: T) -> Self {
        Self { min, max }
    }
}

impl<T> Validator<Option<T>> for RangeNumber<T>
where
    T: Clone + PartialEq + PartialOrd + Send + Sync + 'static + fmt::Display,
{
    fn validate(&self, value: &Option<T>, field_name: &str) -> Option<ValidationError> {
        match value {
            Some(v) if *v < self.min || *v > self.max => {
                Some(ValidationError::out_of_range(field_name, &self.min, &self.max, v))
            }
            _ => None,
        }
    }

    fn description(&self) -> String {
        format!("RangeNumber({}, {})", self.min, self.max)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_field_new() {
        let field: NumberField<i64> = NumberField::new("age");
        assert_eq!(field.name, "age");
        assert!(field.value().is_none());
    }

    #[test]
    fn test_number_field_with_value() {
        let field = NumberField::with_value("quantity", 10i64);
        assert_eq!(field.value(), Some(&10));
    }

    #[test]
    fn test_number_field_min_max() {
        let field: NumberField<i64> = NumberField::new("score")
            .min(0)
            .max(100);
        assert_eq!(field.min, Some(0));
        assert_eq!(field.max, Some(100));
    }

    #[test]
    fn test_number_field_set_value_from_str() {
        let mut field: NumberField<i64> = NumberField::new("count");
        field.set_value_from_str("42");
        assert_eq!(field.value(), Some(&42));

        field.set_value_from_str("");
        assert!(field.value().is_none());

        field.set_value_from_str("invalid");
        assert!(field.value().is_none());
    }

    #[test]
    fn test_number_field_validation() {
        let mut field: NumberField<i64> = NumberField::new("age")
            .validate(RequiredNumber)
            .validate(MinNumber(0i64))
            .validate(MaxNumber(150i64));

        // No value - required fails
        field.run_validation();
        assert!(!field.is_valid());

        // Negative value - min fails
        field.set_value(Some(-5));
        field.run_validation();
        assert!(!field.is_valid());

        // Too high - max fails
        field.set_value(Some(200));
        field.run_validation();
        assert!(!field.is_valid());

        // Valid value
        field.set_value(Some(25));
        field.run_validation();
        assert!(field.is_valid());
    }

    #[test]
    fn test_range_number_validator() {
        let validator = RangeNumber::new(1i64, 10i64);

        // Below range
        assert!(validator.validate(&Some(0i64), "field").is_some());

        // Above range
        assert!(validator.validate(&Some(15i64), "field").is_some());

        // In range
        assert!(validator.validate(&Some(5i64), "field").is_none());

        // None value (not validated by range)
        assert!(validator.validate(&None, "field").is_none());
    }

    #[test]
    fn test_float_field() {
        let mut field: NumberField<f64> = NumberField::new("price")
            .min(0.0)
            .step(0.01);

        field.set_value_from_str("19.99");
        assert_eq!(field.value(), Some(&19.99));
    }
}
