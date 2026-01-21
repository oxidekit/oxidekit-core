//! Loading state management.

use serde::{Deserialize, Serialize};

/// Loading state enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoadingState {
    /// Idle - not loading
    #[default]
    Idle,
    /// Loading
    Loading,
    /// Success
    Success,
    /// Error
    Error,
}

impl LoadingState {
    /// Check if loading
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }

    /// Check if complete (success or error)
    pub fn is_complete(&self) -> bool {
        matches!(self, Self::Success | Self::Error)
    }

    /// Check if success
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }

    /// Check if error
    pub fn is_error(&self) -> bool {
        matches!(self, Self::Error)
    }
}

/// Loader type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LoaderType {
    /// Spinner
    #[default]
    Spinner,
    /// Dots
    Dots,
    /// Bars
    Bars,
    /// Pulse
    Pulse,
    /// Skeleton
    Skeleton,
}

/// Loading boundary component
#[derive(Debug, Clone)]
pub struct LoadingBoundary {
    /// State
    pub state: LoadingState,
    /// Loader type
    pub loader_type: LoaderType,
    /// Message
    pub message: Option<String>,
    /// Error message
    pub error_message: Option<String>,
    /// Retry enabled
    pub retry_enabled: bool,
}

impl Default for LoadingBoundary {
    fn default() -> Self {
        Self::new(LoadingState::Idle)
    }
}

impl LoadingBoundary {
    /// Create new loading boundary
    pub fn new(state: LoadingState) -> Self {
        Self {
            state,
            loader_type: LoaderType::Spinner,
            message: None,
            error_message: None,
            retry_enabled: true,
        }
    }

    /// Set loader type
    pub fn loader(mut self, loader_type: LoaderType) -> Self {
        self.loader_type = loader_type;
        self
    }

    /// Set message
    pub fn message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// Set error message
    pub fn error_message(mut self, message: impl Into<String>) -> Self {
        self.error_message = Some(message.into());
        self
    }

    /// Enable/disable retry
    pub fn retry_enabled(mut self, enabled: bool) -> Self {
        self.retry_enabled = enabled;
        self
    }

    /// Set loading
    pub fn set_loading(&mut self) {
        self.state = LoadingState::Loading;
        self.error_message = None;
    }

    /// Set success
    pub fn set_success(&mut self) {
        self.state = LoadingState::Success;
    }

    /// Set error
    pub fn set_error(&mut self, message: impl Into<String>) {
        self.state = LoadingState::Error;
        self.error_message = Some(message.into());
    }

    /// Reset to idle
    pub fn reset(&mut self) {
        self.state = LoadingState::Idle;
        self.error_message = None;
    }

    /// Should show content
    pub fn should_show_content(&self) -> bool {
        matches!(self.state, LoadingState::Idle | LoadingState::Success)
    }

    /// Should show loader
    pub fn should_show_loader(&self) -> bool {
        self.state.is_loading()
    }

    /// Should show error
    pub fn should_show_error(&self) -> bool {
        self.state.is_error()
    }
}

/// Accessibility announcer for loading states
#[derive(Debug, Clone, Default)]
pub struct AccessibilityAnnouncer {
    /// Current announcement
    pub announcement: Option<String>,
    /// Politeness level
    pub politeness: Politeness,
}

/// ARIA politeness level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Politeness {
    /// Polite - wait for current speech
    #[default]
    Polite,
    /// Assertive - interrupt current speech
    Assertive,
}

impl AccessibilityAnnouncer {
    /// Create new announcer
    pub fn new() -> Self {
        Self::default()
    }

    /// Announce loading started
    pub fn announce_loading(&mut self, context: Option<&str>) {
        self.announcement = Some(match context {
            Some(ctx) => format!("Loading {}", ctx),
            None => "Loading".to_string(),
        });
        self.politeness = Politeness::Polite;
    }

    /// Announce loading complete
    pub fn announce_complete(&mut self, context: Option<&str>) {
        self.announcement = Some(match context {
            Some(ctx) => format!("{} loaded", ctx),
            None => "Content loaded".to_string(),
        });
        self.politeness = Politeness::Polite;
    }

    /// Announce error
    pub fn announce_error(&mut self, message: &str) {
        self.announcement = Some(format!("Error: {}", message));
        self.politeness = Politeness::Assertive;
    }

    /// Clear announcement
    pub fn clear(&mut self) {
        self.announcement = None;
    }
}
