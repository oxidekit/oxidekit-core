//! Video playback components.
//!
//! Provides video player and related components.

use serde::{Deserialize, Serialize};

/// Video playback state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PlaybackState {
    /// Not loaded
    #[default]
    Idle,
    /// Loading
    Loading,
    /// Ready to play
    Ready,
    /// Currently playing
    Playing,
    /// Paused
    Paused,
    /// Ended
    Ended,
    /// Error occurred
    Error,
}

/// Video player configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VideoConfig {
    /// Auto-play on load
    pub autoplay: bool,
    /// Loop playback
    pub loop_playback: bool,
    /// Muted
    pub muted: bool,
    /// Volume (0.0 to 1.0)
    pub volume: f32,
    /// Playback rate
    pub playback_rate: f32,
}

impl Default for VideoConfig {
    fn default() -> Self {
        Self {
            autoplay: false,
            loop_playback: false,
            muted: false,
            volume: 1.0,
            playback_rate: 1.0,
        }
    }
}

/// Video player component
#[derive(Debug, Clone)]
pub struct VideoPlayer {
    /// Source URL
    pub source: Option<String>,
    /// Configuration
    pub config: VideoConfig,
    /// Current playback state
    pub state: PlaybackState,
    /// Current position in seconds
    pub position: f64,
    /// Total duration in seconds
    pub duration: f64,
}

impl Default for VideoPlayer {
    fn default() -> Self {
        Self {
            source: None,
            config: VideoConfig::default(),
            state: PlaybackState::Idle,
            position: 0.0,
            duration: 0.0,
        }
    }
}

impl VideoPlayer {
    /// Create a new video player
    pub fn new() -> Self {
        Self::default()
    }

    /// Load a video source
    pub fn load(&mut self, source: impl Into<String>) {
        self.source = Some(source.into());
        self.state = PlaybackState::Loading;
    }

    /// Play video
    pub fn play(&mut self) {
        if matches!(self.state, PlaybackState::Ready | PlaybackState::Paused) {
            self.state = PlaybackState::Playing;
        }
    }

    /// Pause video
    pub fn pause(&mut self) {
        if self.state == PlaybackState::Playing {
            self.state = PlaybackState::Paused;
        }
    }

    /// Stop video
    pub fn stop(&mut self) {
        self.state = PlaybackState::Ready;
        self.position = 0.0;
    }

    /// Seek to position
    pub fn seek(&mut self, position: f64) {
        self.position = position.clamp(0.0, self.duration);
    }

    /// Set volume
    pub fn set_volume(&mut self, volume: f32) {
        self.config.volume = volume.clamp(0.0, 1.0);
    }

    /// Toggle mute
    pub fn toggle_mute(&mut self) {
        self.config.muted = !self.config.muted;
    }

    /// Get progress as percentage
    pub fn progress(&self) -> f64 {
        if self.duration > 0.0 {
            self.position / self.duration
        } else {
            0.0
        }
    }
}
