//! Pull to refresh component.

use serde::{Deserialize, Serialize};

/// Refresh state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RefreshState {
    /// Idle
    #[default]
    Idle,
    /// Pulling
    Pulling,
    /// Ready to refresh
    Ready,
    /// Refreshing
    Refreshing,
    /// Completing
    Completing,
}

impl RefreshState {
    /// Check if refreshing
    pub fn is_refreshing(&self) -> bool {
        matches!(self, Self::Refreshing)
    }

    /// Check if can refresh
    pub fn can_refresh(&self) -> bool {
        matches!(self, Self::Ready)
    }
}

/// Pull to refresh component
#[derive(Debug, Clone)]
pub struct PullToRefresh {
    /// State
    pub state: RefreshState,
    /// Threshold to trigger refresh
    pub threshold: f32,
    /// Max pull distance
    pub max_distance: f32,
    /// Current pull distance
    pub pull_distance: f32,
    /// Enabled
    pub enabled: bool,
}

impl Default for PullToRefresh {
    fn default() -> Self {
        Self::new()
    }
}

impl PullToRefresh {
    /// Create new pull to refresh
    pub fn new() -> Self {
        Self {
            state: RefreshState::Idle,
            threshold: 80.0,
            max_distance: 150.0,
            pull_distance: 0.0,
            enabled: true,
        }
    }

    /// Set threshold
    pub fn threshold(mut self, threshold: f32) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set max distance
    pub fn max_distance(mut self, max: f32) -> Self {
        self.max_distance = max;
        self
    }

    /// Set enabled
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Handle pull
    pub fn on_pull(&mut self, distance: f32) {
        if !self.enabled || self.state.is_refreshing() {
            return;
        }

        self.pull_distance = distance.min(self.max_distance);
        self.state = if distance >= self.threshold {
            RefreshState::Ready
        } else {
            RefreshState::Pulling
        };
    }

    /// Handle release
    pub fn on_release(&mut self) {
        if self.state == RefreshState::Ready {
            self.state = RefreshState::Refreshing;
        } else {
            self.reset();
        }
    }

    /// Complete refresh
    pub fn complete(&mut self) {
        self.state = RefreshState::Completing;
        self.pull_distance = 0.0;
        self.state = RefreshState::Idle;
    }

    /// Reset state
    pub fn reset(&mut self) {
        self.state = RefreshState::Idle;
        self.pull_distance = 0.0;
    }

    /// Get pull progress (0.0 - 1.0)
    pub fn progress(&self) -> f32 {
        (self.pull_distance / self.threshold).min(1.0)
    }
}

/// Pull indicator component
#[derive(Debug, Clone, Default)]
pub struct PullIndicator {
    /// Pull text
    pub pull_text: String,
    /// Release text
    pub release_text: String,
    /// Refresh text
    pub refresh_text: String,
}

impl PullIndicator {
    /// Create new pull indicator
    pub fn new() -> Self {
        Self {
            pull_text: "Pull to refresh".to_string(),
            release_text: "Release to refresh".to_string(),
            refresh_text: "Refreshing...".to_string(),
        }
    }

    /// Set pull text
    pub fn pull_text(mut self, text: impl Into<String>) -> Self {
        self.pull_text = text.into();
        self
    }

    /// Set release text
    pub fn release_text(mut self, text: impl Into<String>) -> Self {
        self.release_text = text.into();
        self
    }

    /// Set refresh text
    pub fn refresh_text(mut self, text: impl Into<String>) -> Self {
        self.refresh_text = text.into();
        self
    }

    /// Get text for state
    pub fn text_for_state(&self, state: RefreshState) -> &str {
        match state {
            RefreshState::Idle | RefreshState::Pulling => &self.pull_text,
            RefreshState::Ready | RefreshState::Completing => &self.release_text,
            RefreshState::Refreshing => &self.refresh_text,
        }
    }
}
