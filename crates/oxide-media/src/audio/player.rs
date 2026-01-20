//! Audio player implementation with full playback controls.
//!
//! Provides play/pause/stop, volume control, seeking, speed control,
//! and loop modes for audio playback.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use parking_lot::RwLock;
use thiserror::Error;
use tracing::{debug, error, info, warn};

/// Audio player error types.
#[derive(Debug, Error)]
pub enum AudioPlayerError {
    /// Failed to open audio file.
    #[error("Failed to open audio file: {0}")]
    FileOpen(String),

    /// Failed to decode audio.
    #[error("Failed to decode audio: {0}")]
    Decode(String),

    /// Failed to initialize audio output.
    #[error("Failed to initialize audio output: {0}")]
    OutputInit(String),

    /// Invalid playback speed.
    #[error("Invalid playback speed: {0}. Must be between 0.25 and 4.0")]
    InvalidSpeed(f32),

    /// Invalid volume.
    #[error("Invalid volume: {0}. Must be between 0.0 and 1.0")]
    InvalidVolume(f32),

    /// Invalid seek position.
    #[error("Invalid seek position: {0:?}. Duration is {1:?}")]
    InvalidSeek(Duration, Duration),

    /// No source loaded.
    #[error("No audio source loaded")]
    NoSource,

    /// Streaming error.
    #[error("Streaming error: {0}")]
    Streaming(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for audio player operations.
pub type Result<T> = std::result::Result<T, AudioPlayerError>;

/// Loop mode for audio playback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum LoopMode {
    /// No looping, stop after playback.
    #[default]
    None,
    /// Loop the current track.
    One,
    /// Loop the entire playlist.
    All,
}

impl From<u8> for LoopMode {
    fn from(value: u8) -> Self {
        match value {
            1 => LoopMode::One,
            2 => LoopMode::All,
            _ => LoopMode::None,
        }
    }
}

impl From<LoopMode> for u8 {
    fn from(mode: LoopMode) -> Self {
        match mode {
            LoopMode::None => 0,
            LoopMode::One => 1,
            LoopMode::All => 2,
        }
    }
}

/// Playback speed options.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlaybackSpeed {
    /// 0.5x speed (slow).
    Half,
    /// 0.75x speed.
    ThreeQuarters,
    /// 1.0x speed (normal).
    Normal,
    /// 1.25x speed.
    OneTwentyFive,
    /// 1.5x speed.
    OneHalf,
    /// 1.75x speed.
    OneSeventyFive,
    /// 2.0x speed (fast).
    Double,
    /// Custom speed value.
    Custom(f32),
}

impl PlaybackSpeed {
    /// Get the speed multiplier.
    pub fn as_f32(&self) -> f32 {
        match self {
            PlaybackSpeed::Half => 0.5,
            PlaybackSpeed::ThreeQuarters => 0.75,
            PlaybackSpeed::Normal => 1.0,
            PlaybackSpeed::OneTwentyFive => 1.25,
            PlaybackSpeed::OneHalf => 1.5,
            PlaybackSpeed::OneSeventyFive => 1.75,
            PlaybackSpeed::Double => 2.0,
            PlaybackSpeed::Custom(v) => *v,
        }
    }

    /// Create from a float value.
    pub fn from_f32(value: f32) -> Result<Self> {
        if !(0.25..=4.0).contains(&value) {
            return Err(AudioPlayerError::InvalidSpeed(value));
        }
        Ok(match value {
            v if (v - 0.5).abs() < 0.01 => PlaybackSpeed::Half,
            v if (v - 0.75).abs() < 0.01 => PlaybackSpeed::ThreeQuarters,
            v if (v - 1.0).abs() < 0.01 => PlaybackSpeed::Normal,
            v if (v - 1.25).abs() < 0.01 => PlaybackSpeed::OneTwentyFive,
            v if (v - 1.5).abs() < 0.01 => PlaybackSpeed::OneHalf,
            v if (v - 1.75).abs() < 0.01 => PlaybackSpeed::OneSeventyFive,
            v if (v - 2.0).abs() < 0.01 => PlaybackSpeed::Double,
            _ => PlaybackSpeed::Custom(value),
        })
    }
}

impl Default for PlaybackSpeed {
    fn default() -> Self {
        PlaybackSpeed::Normal
    }
}

/// Playback state of the audio player.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PlaybackState {
    /// Player is stopped (no audio loaded or at beginning).
    #[default]
    Stopped,
    /// Player is playing audio.
    Playing,
    /// Player is paused.
    Paused,
    /// Player is buffering (for streaming).
    Buffering,
    /// Player encountered an error.
    Error,
}

/// Audio source information.
#[derive(Debug, Clone)]
pub struct AudioSource {
    /// Source path or URL.
    pub path: String,
    /// Duration of the audio.
    pub duration: Option<Duration>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of channels.
    pub channels: Option<u16>,
    /// Bit depth.
    pub bit_depth: Option<u16>,
    /// Format name.
    pub format: Option<String>,
    /// Is this a streaming source.
    pub is_streaming: bool,
}

impl AudioSource {
    /// Create a new audio source from a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        Self {
            path: path.as_ref().to_string_lossy().to_string(),
            duration: None,
            sample_rate: None,
            channels: None,
            bit_depth: None,
            format: None,
            is_streaming: false,
        }
    }

    /// Create a new audio source from a URL (streaming).
    pub fn from_url(url: &str) -> Self {
        Self {
            path: url.to_string(),
            duration: None,
            sample_rate: None,
            channels: None,
            bit_depth: None,
            format: None,
            is_streaming: true,
        }
    }
}

/// Event callback type for audio player events.
pub type EventCallback = Box<dyn Fn(AudioPlayerEvent) + Send + Sync>;

/// Audio player events.
#[derive(Debug, Clone)]
pub enum AudioPlayerEvent {
    /// Playback started.
    Started,
    /// Playback paused.
    Paused,
    /// Playback stopped.
    Stopped,
    /// Playback completed (reached end).
    Completed,
    /// Position changed (during seek or playback).
    PositionChanged(Duration),
    /// Volume changed.
    VolumeChanged(f32),
    /// Speed changed.
    SpeedChanged(f32),
    /// Loop mode changed.
    LoopModeChanged(LoopMode),
    /// Mute state changed.
    MuteChanged(bool),
    /// Error occurred.
    Error(String),
    /// Buffering progress (for streaming).
    Buffering(f32),
    /// Source loaded.
    SourceLoaded(AudioSource),
}

/// Internal state for the audio player.
struct PlayerState {
    /// Current audio source.
    source: Option<AudioSource>,
    /// Current playback position.
    position: Duration,
    /// Current volume (0.0 to 1.0).
    volume: f32,
    /// Volume before mute.
    pre_mute_volume: f32,
    /// Current playback speed.
    speed: PlaybackSpeed,
    /// Current loop mode.
    loop_mode: LoopMode,
    /// Event callbacks.
    callbacks: Vec<EventCallback>,
}

impl Default for PlayerState {
    fn default() -> Self {
        Self {
            source: None,
            position: Duration::ZERO,
            volume: 1.0,
            pre_mute_volume: 1.0,
            speed: PlaybackSpeed::default(),
            loop_mode: LoopMode::default(),
            callbacks: Vec::new(),
        }
    }
}

/// Audio player for playing audio files and streams.
///
/// # Example
///
/// ```ignore
/// use oxide_media::audio::AudioPlayer;
///
/// let player = AudioPlayer::new("song.mp3")
///     .autoplay(true)
///     .volume(0.8)
///     .build()?;
///
/// player.play()?;
/// player.set_speed(PlaybackSpeed::OneHalf)?;
/// player.seek(Duration::from_secs(30))?;
/// ```
pub struct AudioPlayer {
    /// Player state (protected by RwLock).
    state: Arc<RwLock<PlayerState>>,
    /// Playback state (atomic for lock-free reads).
    playback_state: Arc<AtomicU8>,
    /// Is muted (atomic for lock-free reads).
    is_muted: Arc<AtomicBool>,
    /// Autoplay flag.
    autoplay: bool,
    /// Show controls.
    controls: bool,
}

impl AudioPlayer {
    /// Create a new audio player builder.
    pub fn new<S: Into<String>>(source: S) -> AudioPlayerBuilder {
        AudioPlayerBuilder::new(source)
    }

    /// Get the current playback state.
    pub fn playback_state(&self) -> PlaybackState {
        match self.playback_state.load(Ordering::Acquire) {
            0 => PlaybackState::Stopped,
            1 => PlaybackState::Playing,
            2 => PlaybackState::Paused,
            3 => PlaybackState::Buffering,
            _ => PlaybackState::Error,
        }
    }

    /// Check if the player is currently playing.
    pub fn is_playing(&self) -> bool {
        self.playback_state() == PlaybackState::Playing
    }

    /// Check if the player is paused.
    pub fn is_paused(&self) -> bool {
        self.playback_state() == PlaybackState::Paused
    }

    /// Check if the player is stopped.
    pub fn is_stopped(&self) -> bool {
        self.playback_state() == PlaybackState::Stopped
    }

    /// Start playback.
    pub fn play(&self) -> Result<()> {
        let state = self.state.read();
        if state.source.is_none() {
            return Err(AudioPlayerError::NoSource);
        }
        drop(state);

        self.set_playback_state(PlaybackState::Playing);
        self.emit_event(AudioPlayerEvent::Started);
        info!("Audio playback started");
        Ok(())
    }

    /// Pause playback.
    pub fn pause(&self) -> Result<()> {
        if !self.is_playing() {
            return Ok(());
        }

        self.set_playback_state(PlaybackState::Paused);
        self.emit_event(AudioPlayerEvent::Paused);
        info!("Audio playback paused");
        Ok(())
    }

    /// Toggle play/pause state.
    pub fn toggle(&self) -> Result<()> {
        if self.is_playing() {
            self.pause()
        } else {
            self.play()
        }
    }

    /// Stop playback and reset position to beginning.
    pub fn stop(&self) -> Result<()> {
        self.set_playback_state(PlaybackState::Stopped);
        {
            let mut state = self.state.write();
            state.position = Duration::ZERO;
        }
        self.emit_event(AudioPlayerEvent::Stopped);
        self.emit_event(AudioPlayerEvent::PositionChanged(Duration::ZERO));
        info!("Audio playback stopped");
        Ok(())
    }

    /// Seek to a specific position.
    pub fn seek(&self, position: Duration) -> Result<()> {
        let state = self.state.read();
        if let Some(ref source) = state.source {
            if let Some(duration) = source.duration {
                if position > duration {
                    return Err(AudioPlayerError::InvalidSeek(position, duration));
                }
            }
        }
        drop(state);

        {
            let mut state = self.state.write();
            state.position = position;
        }

        self.emit_event(AudioPlayerEvent::PositionChanged(position));
        debug!("Seeked to position: {:?}", position);
        Ok(())
    }

    /// Seek forward by a duration.
    pub fn seek_forward(&self, duration: Duration) -> Result<()> {
        let current = self.position();
        self.seek(current + duration)
    }

    /// Seek backward by a duration.
    pub fn seek_backward(&self, duration: Duration) -> Result<()> {
        let current = self.position();
        let new_pos = current.saturating_sub(duration);
        self.seek(new_pos)
    }

    /// Get the current playback position.
    pub fn position(&self) -> Duration {
        self.state.read().position
    }

    /// Get the total duration (if known).
    pub fn duration(&self) -> Option<Duration> {
        self.state.read().source.as_ref().and_then(|s| s.duration)
    }

    /// Get the playback progress as a percentage (0.0 to 1.0).
    pub fn progress(&self) -> f32 {
        let state = self.state.read();
        if let Some(ref source) = state.source {
            if let Some(duration) = source.duration {
                if duration.as_secs_f32() > 0.0 {
                    return state.position.as_secs_f32() / duration.as_secs_f32();
                }
            }
        }
        0.0
    }

    /// Set the volume (0.0 to 1.0).
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        if !(0.0..=1.0).contains(&volume) {
            return Err(AudioPlayerError::InvalidVolume(volume));
        }

        {
            let mut state = self.state.write();
            state.volume = volume;
            if !self.is_muted.load(Ordering::Acquire) {
                state.pre_mute_volume = volume;
            }
        }

        self.emit_event(AudioPlayerEvent::VolumeChanged(volume));
        debug!("Volume set to: {}", volume);
        Ok(())
    }

    /// Get the current volume.
    pub fn volume(&self) -> f32 {
        self.state.read().volume
    }

    /// Mute the audio.
    pub fn mute(&self) {
        if !self.is_muted.swap(true, Ordering::AcqRel) {
            let mut state = self.state.write();
            state.pre_mute_volume = state.volume;
            state.volume = 0.0;
            self.emit_event(AudioPlayerEvent::MuteChanged(true));
            self.emit_event(AudioPlayerEvent::VolumeChanged(0.0));
            debug!("Audio muted");
        }
    }

    /// Unmute the audio.
    pub fn unmute(&self) {
        if self.is_muted.swap(false, Ordering::AcqRel) {
            let mut state = self.state.write();
            state.volume = state.pre_mute_volume;
            let volume = state.volume;
            drop(state);
            self.emit_event(AudioPlayerEvent::MuteChanged(false));
            self.emit_event(AudioPlayerEvent::VolumeChanged(volume));
            debug!("Audio unmuted");
        }
    }

    /// Toggle mute state.
    pub fn toggle_mute(&self) {
        if self.is_muted() {
            self.unmute();
        } else {
            self.mute();
        }
    }

    /// Check if the audio is muted.
    pub fn is_muted(&self) -> bool {
        self.is_muted.load(Ordering::Acquire)
    }

    /// Set the playback speed.
    pub fn set_speed(&self, speed: PlaybackSpeed) -> Result<()> {
        let speed_value = speed.as_f32();
        if !(0.25..=4.0).contains(&speed_value) {
            return Err(AudioPlayerError::InvalidSpeed(speed_value));
        }

        {
            let mut state = self.state.write();
            state.speed = speed;
        }

        self.emit_event(AudioPlayerEvent::SpeedChanged(speed_value));
        debug!("Playback speed set to: {}x", speed_value);
        Ok(())
    }

    /// Get the current playback speed.
    pub fn speed(&self) -> PlaybackSpeed {
        self.state.read().speed
    }

    /// Set the loop mode.
    pub fn set_loop_mode(&self, mode: LoopMode) {
        {
            let mut state = self.state.write();
            state.loop_mode = mode;
        }

        self.emit_event(AudioPlayerEvent::LoopModeChanged(mode));
        debug!("Loop mode set to: {:?}", mode);
    }

    /// Get the current loop mode.
    pub fn loop_mode(&self) -> LoopMode {
        self.state.read().loop_mode
    }

    /// Cycle through loop modes (None -> One -> All -> None).
    pub fn cycle_loop_mode(&self) {
        let current = self.loop_mode();
        let next = match current {
            LoopMode::None => LoopMode::One,
            LoopMode::One => LoopMode::All,
            LoopMode::All => LoopMode::None,
        };
        self.set_loop_mode(next);
    }

    /// Get the current audio source.
    pub fn source(&self) -> Option<AudioSource> {
        self.state.read().source.clone()
    }

    /// Load a new audio source.
    pub fn load<S: Into<String>>(&self, source: S) -> Result<()> {
        let source_str = source.into();
        let audio_source = if source_str.starts_with("http://") || source_str.starts_with("https://") {
            AudioSource::from_url(&source_str)
        } else {
            let path = PathBuf::from(&source_str);
            if !path.exists() {
                return Err(AudioPlayerError::FileOpen(format!(
                    "File not found: {}",
                    source_str
                )));
            }
            AudioSource::from_path(path)
        };

        self.stop()?;

        {
            let mut state = self.state.write();
            state.source = Some(audio_source.clone());
        }

        self.emit_event(AudioPlayerEvent::SourceLoaded(audio_source));
        info!("Audio source loaded: {}", source_str);
        Ok(())
    }

    /// Register an event callback.
    pub fn on_event<F>(&self, callback: F)
    where
        F: Fn(AudioPlayerEvent) + Send + Sync + 'static,
    {
        let mut state = self.state.write();
        state.callbacks.push(Box::new(callback));
    }

    /// Check if autoplay is enabled.
    pub fn autoplay(&self) -> bool {
        self.autoplay
    }

    /// Check if controls are shown.
    pub fn controls(&self) -> bool {
        self.controls
    }

    /// Set the playback state (internal).
    fn set_playback_state(&self, state: PlaybackState) {
        let value = match state {
            PlaybackState::Stopped => 0,
            PlaybackState::Playing => 1,
            PlaybackState::Paused => 2,
            PlaybackState::Buffering => 3,
            PlaybackState::Error => 4,
        };
        self.playback_state.store(value, Ordering::Release);
    }

    /// Emit an event to all registered callbacks.
    fn emit_event(&self, event: AudioPlayerEvent) {
        let state = self.state.read();
        for callback in &state.callbacks {
            callback(event.clone());
        }
    }
}

/// Builder for creating an AudioPlayer.
pub struct AudioPlayerBuilder {
    source: String,
    autoplay: bool,
    controls: bool,
    volume: f32,
    speed: PlaybackSpeed,
    loop_mode: LoopMode,
    muted: bool,
}

impl AudioPlayerBuilder {
    /// Create a new builder with the given source.
    pub fn new<S: Into<String>>(source: S) -> Self {
        Self {
            source: source.into(),
            autoplay: false,
            controls: true,
            volume: 1.0,
            speed: PlaybackSpeed::Normal,
            loop_mode: LoopMode::None,
            muted: false,
        }
    }

    /// Enable or disable autoplay.
    pub fn autoplay(mut self, autoplay: bool) -> Self {
        self.autoplay = autoplay;
        self
    }

    /// Enable or disable controls.
    pub fn controls(mut self, controls: bool) -> Self {
        self.controls = controls;
        self
    }

    /// Set the initial volume (0.0 to 1.0).
    pub fn volume(mut self, volume: f32) -> Self {
        self.volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the initial playback speed.
    pub fn speed(mut self, speed: PlaybackSpeed) -> Self {
        self.speed = speed;
        self
    }

    /// Set the initial loop mode.
    pub fn loop_mode(mut self, mode: LoopMode) -> Self {
        self.loop_mode = mode;
        self
    }

    /// Set the initial muted state.
    pub fn muted(mut self, muted: bool) -> Self {
        self.muted = muted;
        self
    }

    /// Build the AudioPlayer.
    pub fn build(self) -> Result<AudioPlayer> {
        let source_str = self.source.clone();
        let audio_source = if source_str.starts_with("http://") || source_str.starts_with("https://") {
            AudioSource::from_url(&source_str)
        } else {
            AudioSource::from_path(&source_str)
        };

        let state = PlayerState {
            source: Some(audio_source.clone()),
            position: Duration::ZERO,
            volume: if self.muted { 0.0 } else { self.volume },
            pre_mute_volume: self.volume,
            speed: self.speed,
            loop_mode: self.loop_mode,
            callbacks: Vec::new(),
        };

        let player = AudioPlayer {
            state: Arc::new(RwLock::new(state)),
            playback_state: Arc::new(AtomicU8::new(0)),
            is_muted: Arc::new(AtomicBool::new(self.muted)),
            autoplay: self.autoplay,
            controls: self.controls,
        };

        info!("Audio player created for source: {}", source_str);

        if self.autoplay {
            player.play()?;
        }

        Ok(player)
    }
}

impl Clone for AudioPlayer {
    fn clone(&self) -> Self {
        Self {
            state: Arc::clone(&self.state),
            playback_state: Arc::clone(&self.playback_state),
            is_muted: Arc::clone(&self.is_muted),
            autoplay: self.autoplay,
            controls: self.controls,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_player_builder() {
        let player = AudioPlayer::new("test.mp3")
            .volume(0.5)
            .speed(PlaybackSpeed::Double)
            .loop_mode(LoopMode::One)
            .controls(true)
            .build()
            .unwrap();

        assert_eq!(player.volume(), 0.5);
        assert_eq!(player.speed(), PlaybackSpeed::Double);
        assert_eq!(player.loop_mode(), LoopMode::One);
        assert!(player.controls());
    }

    #[test]
    fn test_playback_state() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        assert!(player.is_stopped());
        assert!(!player.is_playing());
        assert!(!player.is_paused());
    }

    #[test]
    fn test_volume_control() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        player.set_volume(0.5).unwrap();
        assert!((player.volume() - 0.5).abs() < 0.01);

        assert!(player.set_volume(1.5).is_err());
        assert!(player.set_volume(-0.1).is_err());
    }

    #[test]
    fn test_mute_toggle() {
        let player = AudioPlayer::new("test.mp3").volume(0.8).build().unwrap();

        assert!(!player.is_muted());
        assert!((player.volume() - 0.8).abs() < 0.01);

        player.mute();
        assert!(player.is_muted());
        assert!((player.volume() - 0.0).abs() < 0.01);

        player.unmute();
        assert!(!player.is_muted());
        assert!((player.volume() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_playback_speed() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        player.set_speed(PlaybackSpeed::Half).unwrap();
        assert_eq!(player.speed(), PlaybackSpeed::Half);
        assert!((player.speed().as_f32() - 0.5).abs() < 0.01);

        player.set_speed(PlaybackSpeed::Double).unwrap();
        assert!((player.speed().as_f32() - 2.0).abs() < 0.01);

        let custom = PlaybackSpeed::from_f32(1.75).unwrap();
        player.set_speed(custom).unwrap();
    }

    #[test]
    fn test_loop_mode_cycle() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        assert_eq!(player.loop_mode(), LoopMode::None);

        player.cycle_loop_mode();
        assert_eq!(player.loop_mode(), LoopMode::One);

        player.cycle_loop_mode();
        assert_eq!(player.loop_mode(), LoopMode::All);

        player.cycle_loop_mode();
        assert_eq!(player.loop_mode(), LoopMode::None);
    }

    #[test]
    fn test_seek() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        player.seek(Duration::from_secs(30)).unwrap();
        assert_eq!(player.position(), Duration::from_secs(30));

        player.seek_forward(Duration::from_secs(10)).unwrap();
        assert_eq!(player.position(), Duration::from_secs(40));

        player.seek_backward(Duration::from_secs(15)).unwrap();
        assert_eq!(player.position(), Duration::from_secs(25));
    }

    #[test]
    fn test_play_pause_stop() {
        let player = AudioPlayer::new("test.mp3").build().unwrap();

        player.play().unwrap();
        assert!(player.is_playing());

        player.pause().unwrap();
        assert!(player.is_paused());

        player.toggle().unwrap();
        assert!(player.is_playing());

        player.stop().unwrap();
        assert!(player.is_stopped());
        assert_eq!(player.position(), Duration::ZERO);
    }

    #[test]
    fn test_autoplay() {
        let player = AudioPlayer::new("test.mp3")
            .autoplay(true)
            .build()
            .unwrap();

        assert!(player.is_playing());
    }

    #[test]
    fn test_audio_source_from_url() {
        let source = AudioSource::from_url("https://example.com/audio.mp3");
        assert!(source.is_streaming);
        assert_eq!(source.path, "https://example.com/audio.mp3");
    }
}
