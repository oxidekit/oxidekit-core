//! Form Field and Field State
//!
//! Provides the core field abstraction with value, error, touched, and dirty states.

use crate::async_validator::AsyncValidationState;
use crate::error::{ValidationError, ValidationErrors};
use crate::mode::{ValidationConfig, ValidationMode};
use crate::validator::{BoxedValidator, ValidatorChain};
use serde::{Deserialize, Serialize};
use std::fmt;

/// State of a form field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldState<T>
where
    T: Clone,
{
    /// Current value of the field
    pub value: T,

    /// Initial value (for reset)
    #[serde(skip)]
    initial_value: T,

    /// Current validation errors
    #[serde(default)]
    pub errors: Vec<ValidationError>,

    /// Whether the field has been touched (focused and blurred)
    #[serde(default)]
    pub touched: bool,

    /// Whether the field value has been modified from initial
    #[serde(default)]
    pub dirty: bool,

    /// Whether the field is currently focused
    #[serde(default)]
    pub focused: bool,

    /// Whether validation has been run at least once
    #[serde(default)]
    pub validated: bool,

    /// Async validation state
    #[serde(default)]
    pub async_state: AsyncValidationState,
}

impl<T> FieldState<T>
where
    T: Clone + PartialEq,
{
    /// Create a new field state with an initial value
    pub fn new(initial_value: T) -> Self {
        Self {
            value: initial_value.clone(),
            initial_value,
            errors: Vec::new(),
            touched: false,
            dirty: false,
            focused: false,
            validated: false,
            async_state: AsyncValidationState::Idle,
        }
    }

    /// Check if the field is valid (no errors)
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty() && !self.async_state.is_validating()
    }

    /// Check if the field has errors
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }

    /// Get the first error message
    pub fn first_error(&self) -> Option<&str> {
        self.errors.first().map(|e| e.message.as_str())
    }

    /// Check if the field is pristine (not dirty)
    pub fn is_pristine(&self) -> bool {
        !self.dirty
    }

    /// Set the field value
    pub fn set_value(&mut self, value: T) {
        self.dirty = value != self.initial_value;
        self.value = value;
    }

    /// Reset the field to its initial value
    pub fn reset(&mut self) {
        self.value = self.initial_value.clone();
        self.errors.clear();
        self.touched = false;
        self.dirty = false;
        self.focused = false;
        self.validated = false;
        self.async_state = AsyncValidationState::Idle;
    }

    /// Clear the field value (to default)
    pub fn clear(&mut self)
    where
        T: Default,
    {
        self.value = T::default();
        self.dirty = T::default() != self.initial_value;
        self.errors.clear();
        self.validated = false;
        self.async_state = AsyncValidationState::Idle;
    }

    /// Mark the field as touched
    pub fn mark_touched(&mut self) {
        self.touched = true;
    }

    /// Mark the field as focused
    pub fn mark_focused(&mut self) {
        self.focused = true;
    }

    /// Mark the field as blurred (unfocused)
    pub fn mark_blurred(&mut self) {
        self.focused = false;
        self.touched = true;
    }

    /// Set validation errors
    pub fn set_errors(&mut self, errors: Vec<ValidationError>) {
        self.errors = errors;
        self.validated = true;
        self.async_state = if self.errors.is_empty() {
            AsyncValidationState::Valid
        } else {
            AsyncValidationState::Invalid
        };
    }

    /// Add a validation error
    pub fn add_error(&mut self, error: ValidationError) {
        self.errors.push(error);
        self.validated = true;
        self.async_state = AsyncValidationState::Invalid;
    }

    /// Clear validation errors
    pub fn clear_errors(&mut self) {
        self.errors.clear();
        self.async_state = AsyncValidationState::Idle;
    }

    /// Update the initial value (useful after successful submission)
    pub fn set_initial_value(&mut self, value: T) {
        self.initial_value = value;
        self.dirty = self.value != self.initial_value;
    }

    /// Should show error based on touched state and validation mode
    pub fn should_show_error(&self, mode: ValidationMode) -> bool {
        if self.errors.is_empty() {
            return false;
        }

        match mode {
            ValidationMode::OnChange | ValidationMode::OnBlur | ValidationMode::OnBlurThenChange => {
                self.touched || self.validated
            }
            ValidationMode::OnSubmit | ValidationMode::OnSubmitThenChange => self.validated,
            ValidationMode::Manual => self.validated,
        }
    }

    /// Should show success indicator
    pub fn should_show_success(&self, show_success: bool) -> bool {
        show_success && self.validated && self.errors.is_empty() && self.touched
    }
}

impl<T> Default for FieldState<T>
where
    T: Clone + PartialEq + Default,
{
    fn default() -> Self {
        Self::new(T::default())
    }
}

/// A form field with validation
pub struct FormField<T>
where
    T: Clone + Send + Sync + 'static,
{
    /// Field name/identifier
    pub name: String,

    /// Current field state
    pub state: FieldState<T>,

    /// Validation configuration
    pub config: ValidationConfig,

    /// Synchronous validators
    validators: ValidatorChain<T>,

    /// Field label for display
    pub label: Option<String>,

    /// Placeholder text
    pub placeholder: Option<String>,

    /// Help text
    pub help_text: Option<String>,

    /// Whether the field is disabled
    pub disabled: bool,

    /// Whether the field is read-only
    pub read_only: bool,
}

impl<T> FormField<T>
where
    T: Clone + PartialEq + Send + Sync + 'static,
{
    /// Create a new form field
    pub fn new(name: impl Into<String>, initial_value: T) -> Self {
        Self {
            name: name.into(),
            state: FieldState::new(initial_value),
            config: ValidationConfig::default(),
            validators: ValidatorChain::new(),
            label: None,
            placeholder: None,
            help_text: None,
            disabled: false,
            read_only: false,
        }
    }

    /// Set the validation mode
    pub fn validate_on(mut self, mode: ValidationMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Set the validation config
    pub fn with_config(mut self, config: ValidationConfig) -> Self {
        self.config = config;
        self
    }

    /// Add a validator
    pub fn validate<V>(mut self, validator: V) -> Self
    where
        V: crate::validator::Validator<T> + 'static,
    {
        self.validators = self.validators.add(validator);
        self
    }

    /// Set the field label
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

    /// Set whether the field is disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set whether the field is read-only
    pub fn read_only(mut self, read_only: bool) -> Self {
        self.read_only = read_only;
        self
    }

    /// Get the current value
    pub fn value(&self) -> &T {
        &self.state.value
    }

    /// Set the field value
    pub fn set_value(&mut self, value: T) {
        if self.disabled || self.read_only {
            return;
        }
        self.state.set_value(value);
    }

    /// Handle value change event
    pub fn on_change(&mut self, value: T) {
        self.set_value(value);

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
        // Skip if disabled
        if self.disabled {
            self.state.clear_errors();
            return true;
        }

        // Run validators
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
    pub fn clear(&mut self)
    where
        T: Default,
    {
        self.state.clear();
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

    /// Get first error message
    pub fn first_error(&self) -> Option<&str> {
        self.state.first_error()
    }

    /// Should show error
    pub fn should_show_error(&self) -> bool {
        self.state.should_show_error(self.config.mode)
    }

    /// Should show success
    pub fn should_show_success(&self) -> bool {
        self.state.should_show_success(self.config.show_success)
    }

    /// Get display label
    pub fn display_label(&self) -> &str {
        self.label.as_deref().unwrap_or(&self.name)
    }
}

impl<T> fmt::Debug for FormField<T>
where
    T: Clone + fmt::Debug + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FormField")
            .field("name", &self.name)
            .field("state", &self.state)
            .field("config", &self.config)
            .field("label", &self.label)
            .field("disabled", &self.disabled)
            .field("read_only", &self.read_only)
            .finish()
    }
}

/// Trait for types that can be used as form field values
pub trait FieldValue: Clone + PartialEq + Send + Sync + 'static {
    /// Get the default/empty value
    fn empty() -> Self;

    /// Check if the value is empty
    fn is_empty(&self) -> bool;
}

impl FieldValue for String {
    fn empty() -> Self {
        String::new()
    }

    fn is_empty(&self) -> bool {
        self.trim().is_empty()
    }
}

impl FieldValue for Option<String> {
    fn empty() -> Self {
        None
    }

    fn is_empty(&self) -> bool {
        self.as_ref().map(|s| s.trim().is_empty()).unwrap_or(true)
    }
}

impl FieldValue for bool {
    fn empty() -> Self {
        false
    }

    fn is_empty(&self) -> bool {
        !self
    }
}

impl FieldValue for i32 {
    fn empty() -> Self {
        0
    }

    fn is_empty(&self) -> bool {
        *self == 0
    }
}

impl FieldValue for i64 {
    fn empty() -> Self {
        0
    }

    fn is_empty(&self) -> bool {
        *self == 0
    }
}

impl FieldValue for f32 {
    fn empty() -> Self {
        0.0
    }

    fn is_empty(&self) -> bool {
        *self == 0.0
    }
}

impl FieldValue for f64 {
    fn empty() -> Self {
        0.0
    }

    fn is_empty(&self) -> bool {
        *self == 0.0
    }
}

impl<T: Clone + PartialEq + Send + Sync + 'static> FieldValue for Vec<T> {
    fn empty() -> Self {
        Vec::new()
    }

    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validator::{MinLength, Required};

    #[test]
    fn test_field_state_new() {
        let state: FieldState<String> = FieldState::new("initial".to_string());
        assert_eq!(state.value, "initial");
        assert!(!state.touched);
        assert!(!state.dirty);
        assert!(!state.focused);
        assert!(!state.validated);
    }

    #[test]
    fn test_field_state_set_value() {
        let mut state = FieldState::new("initial".to_string());
        state.set_value("changed".to_string());
        assert_eq!(state.value, "changed");
        assert!(state.dirty);
    }

    #[test]
    fn test_field_state_reset() {
        let mut state = FieldState::new("initial".to_string());
        state.set_value("changed".to_string());
        state.mark_touched();
        state.reset();

        assert_eq!(state.value, "initial");
        assert!(!state.dirty);
        assert!(!state.touched);
    }

    #[test]
    fn test_field_state_errors() {
        let mut state: FieldState<String> = FieldState::new(String::new());
        assert!(state.is_valid());

        state.add_error(ValidationError::required("field"));
        assert!(!state.is_valid());
        assert!(state.has_errors());
        assert_eq!(state.first_error(), Some("This field is required"));
    }

    #[test]
    fn test_form_field_validation() {
        let mut field = FormField::new("email", String::new())
            .validate(Required)
            .validate(MinLength(5));

        // Initially no errors (not validated)
        assert!(field.errors().is_empty());

        // Run validation
        field.run_validation();
        assert!(!field.is_valid());
        assert!(!field.errors().is_empty());
    }

    #[test]
    fn test_form_field_on_change() {
        let mut field = FormField::new("name", String::new())
            .validate_on(ValidationMode::OnChange)
            .validate(Required);

        // Change value triggers validation
        field.on_change("hello".to_string());
        assert!(field.is_valid());
        assert!(field.is_dirty());
    }

    #[test]
    fn test_form_field_on_blur() {
        let mut field = FormField::new("name", String::new())
            .validate_on(ValidationMode::OnBlur)
            .validate(Required);

        // Set value without triggering validation
        field.set_value("hello".to_string());
        assert!(field.errors().is_empty()); // Not validated yet

        // Blur triggers validation
        field.on_blur();
        assert!(field.is_touched());
        assert!(field.is_valid());
    }

    #[test]
    fn test_form_field_disabled() {
        let mut field = FormField::new("name", "initial".to_string()).disabled(true);

        // Disabled field ignores value changes
        field.set_value("changed".to_string());
        assert_eq!(field.value(), "initial");
    }

    #[test]
    fn test_form_field_should_show_error() {
        let mut field = FormField::new("email", String::new())
            .validate_on(ValidationMode::OnBlur)
            .validate(Required);

        // Before touch, don't show error
        field.run_validation();
        assert!(!field.should_show_error()); // Not touched yet

        // After touch, show error
        field.state.mark_touched();
        assert!(field.should_show_error());
    }
}
