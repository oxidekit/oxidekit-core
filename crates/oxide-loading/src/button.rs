//! Button loading states.

use serde::{Deserialize, Serialize};

/// Loading state for buttons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ButtonLoadingState {
    /// Normal state
    #[default]
    Idle,
    /// Loading
    Loading,
    /// Success
    Success,
    /// Error
    Error,
}

impl ButtonLoadingState {
    /// Check if loading
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Check if completed
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Success | Self::Error)
    }
}

/// Loading button component
#[derive(Debug, Clone)]
pub struct LoadingButton {
    /// Label text
    pub label: String,
    /// Loading text
    pub loading_label: Option<String>,
    /// Current state
    pub state: ButtonLoadingState,
    /// Disabled
    pub disabled: bool,
    /// Show spinner when loading
    pub show_spinner: bool,
    /// Minimum loading duration (ms)
    pub min_loading_ms: u64,
}

impl Default for LoadingButton {
    fn default() -> Self {
        Self::new("Submit")
    }
}

impl LoadingButton {
    /// Create new loading button
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            loading_label: None,
            state: ButtonLoadingState::Idle,
            disabled: false,
            show_spinner: true,
            min_loading_ms: 0,
        }
    }

    /// Set loading label
    pub fn loading_label(mut self, label: impl Into<String>) -> Self {
        self.loading_label = Some(label.into());
        self
    }

    /// Set state
    pub fn state(mut self, state: ButtonLoadingState) -> Self {
        self.state = state;
        self
    }

    /// Set disabled
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Set minimum loading duration
    pub fn min_loading_ms(mut self, ms: u64) -> Self {
        self.min_loading_ms = ms;
        self
    }

    /// Hide spinner
    pub fn no_spinner(mut self) -> Self {
        self.show_spinner = false;
        self
    }

    /// Start loading
    pub fn start_loading(&mut self) {
        self.state = ButtonLoadingState::Loading;
    }

    /// Set success
    pub fn set_success(&mut self) {
        self.state = ButtonLoadingState::Success;
    }

    /// Set error
    pub fn set_error(&mut self) {
        self.state = ButtonLoadingState::Error;
    }

    /// Reset to idle
    pub fn reset(&mut self) {
        self.state = ButtonLoadingState::Idle;
    }

    /// Get current label
    pub fn current_label(&self) -> &str {
        if self.state.is_loading() {
            self.loading_label.as_deref().unwrap_or(&self.label)
        } else {
            &self.label
        }
    }

    /// Check if button is clickable
    pub fn is_clickable(&self) -> bool {
        !self.disabled && !self.state.is_loading()
    }
}
