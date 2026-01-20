//! Audio decoding for various formats.
//!
//! Supports MP3, WAV, OGG, FLAC, AAC and other common audio formats.

use std::io::{Read, Seek};
use std::path::Path;
use std::time::Duration;

use thiserror::Error;
use tracing::{debug, error, info};

/// Audio decoder error types.
#[derive(Debug, Error)]
pub enum DecoderError {
    /// Unsupported audio format.
    #[error("Unsupported audio format: {0}")]
    UnsupportedFormat(String),

    /// Failed to read audio data.
    #[error("Failed to read audio data: {0}")]
    ReadError(String),

    /// Failed to decode audio.
    #[error("Failed to decode audio: {0}")]
    DecodeError(String),

    /// Invalid audio file.
    #[error("Invalid audio file: {0}")]
    InvalidFile(String),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for decoder operations.
pub type Result<T> = std::result::Result<T, DecoderError>;

/// Supported audio formats.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AudioFormat {
    /// MP3 format.
    Mp3,
    /// WAV format.
    Wav,
    /// OGG Vorbis format.
    Ogg,
    /// FLAC format.
    Flac,
    /// AAC format.
    Aac,
    /// ALAC (Apple Lossless) format.
    Alac,
    /// AIFF format.
    Aiff,
    /// Unknown format.
    Unknown,
}

impl AudioFormat {
    /// Detect format from file extension.
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => AudioFormat::Mp3,
            "wav" | "wave" => AudioFormat::Wav,
            "ogg" | "oga" => AudioFormat::Ogg,
            "flac" => AudioFormat::Flac,
            "aac" | "m4a" => AudioFormat::Aac,
            "alac" => AudioFormat::Alac,
            "aiff" | "aif" => AudioFormat::Aiff,
            _ => AudioFormat::Unknown,
        }
    }

    /// Detect format from file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Self {
        path.as_ref()
            .extension()
            .and_then(|ext| ext.to_str())
            .map(Self::from_extension)
            .unwrap_or(AudioFormat::Unknown)
    }

    /// Detect format from MIME type.
    pub fn from_mime_type(mime: &str) -> Self {
        match mime.to_lowercase().as_str() {
            "audio/mpeg" | "audio/mp3" => AudioFormat::Mp3,
            "audio/wav" | "audio/wave" | "audio/x-wav" => AudioFormat::Wav,
            "audio/ogg" | "audio/vorbis" => AudioFormat::Ogg,
            "audio/flac" | "audio/x-flac" => AudioFormat::Flac,
            "audio/aac" | "audio/mp4" | "audio/x-m4a" => AudioFormat::Aac,
            "audio/x-alac" => AudioFormat::Alac,
            "audio/aiff" | "audio/x-aiff" => AudioFormat::Aiff,
            _ => AudioFormat::Unknown,
        }
    }

    /// Get the MIME type for this format.
    pub fn mime_type(&self) -> &'static str {
        match self {
            AudioFormat::Mp3 => "audio/mpeg",
            AudioFormat::Wav => "audio/wav",
            AudioFormat::Ogg => "audio/ogg",
            AudioFormat::Flac => "audio/flac",
            AudioFormat::Aac => "audio/aac",
            AudioFormat::Alac => "audio/x-alac",
            AudioFormat::Aiff => "audio/aiff",
            AudioFormat::Unknown => "application/octet-stream",
        }
    }

    /// Get common file extensions for this format.
    pub fn extensions(&self) -> &'static [&'static str] {
        match self {
            AudioFormat::Mp3 => &["mp3"],
            AudioFormat::Wav => &["wav", "wave"],
            AudioFormat::Ogg => &["ogg", "oga"],
            AudioFormat::Flac => &["flac"],
            AudioFormat::Aac => &["aac", "m4a"],
            AudioFormat::Alac => &["alac", "m4a"],
            AudioFormat::Aiff => &["aiff", "aif"],
            AudioFormat::Unknown => &[],
        }
    }

    /// Check if this format supports seeking.
    pub fn supports_seeking(&self) -> bool {
        matches!(
            self,
            AudioFormat::Mp3
                | AudioFormat::Wav
                | AudioFormat::Ogg
                | AudioFormat::Flac
                | AudioFormat::Aac
                | AudioFormat::Alac
                | AudioFormat::Aiff
        )
    }

    /// Check if this is a lossless format.
    pub fn is_lossless(&self) -> bool {
        matches!(
            self,
            AudioFormat::Wav | AudioFormat::Flac | AudioFormat::Alac | AudioFormat::Aiff
        )
    }
}

impl std::fmt::Display for AudioFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AudioFormat::Mp3 => write!(f, "MP3"),
            AudioFormat::Wav => write!(f, "WAV"),
            AudioFormat::Ogg => write!(f, "OGG"),
            AudioFormat::Flac => write!(f, "FLAC"),
            AudioFormat::Aac => write!(f, "AAC"),
            AudioFormat::Alac => write!(f, "ALAC"),
            AudioFormat::Aiff => write!(f, "AIFF"),
            AudioFormat::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Audio metadata extracted from a file.
#[derive(Debug, Clone, Default)]
pub struct AudioMetadata {
    /// Duration of the audio.
    pub duration: Option<Duration>,
    /// Sample rate in Hz.
    pub sample_rate: Option<u32>,
    /// Number of channels.
    pub channels: Option<u16>,
    /// Bits per sample.
    pub bits_per_sample: Option<u16>,
    /// Bitrate in kbps.
    pub bitrate: Option<u32>,
    /// Audio format.
    pub format: AudioFormat,
    /// Total number of samples.
    pub total_samples: Option<u64>,
    /// Title tag.
    pub title: Option<String>,
    /// Artist tag.
    pub artist: Option<String>,
    /// Album tag.
    pub album: Option<String>,
    /// Track number.
    pub track_number: Option<u32>,
    /// Year.
    pub year: Option<u32>,
    /// Genre.
    pub genre: Option<String>,
    /// Album art data (if embedded).
    pub album_art: Option<Vec<u8>>,
    /// Album art MIME type.
    pub album_art_mime: Option<String>,
}

impl AudioMetadata {
    /// Create new empty metadata.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create metadata with format.
    pub fn with_format(format: AudioFormat) -> Self {
        Self {
            format,
            ..Default::default()
        }
    }

    /// Calculate the file size in bytes (approximate).
    pub fn approximate_file_size(&self) -> Option<u64> {
        if let (Some(duration), Some(bitrate)) = (self.duration, self.bitrate) {
            // bitrate is in kbps, duration in seconds
            Some((duration.as_secs() * bitrate as u64 * 1000) / 8)
        } else {
            None
        }
    }

    /// Check if metadata has audio tags.
    pub fn has_tags(&self) -> bool {
        self.title.is_some()
            || self.artist.is_some()
            || self.album.is_some()
            || self.genre.is_some()
    }
}

/// Sample format for decoded audio.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SampleFormat {
    /// Signed 16-bit integer.
    I16,
    /// Signed 32-bit integer.
    I32,
    /// 32-bit floating point.
    F32,
    /// 64-bit floating point.
    F64,
}

impl SampleFormat {
    /// Get the number of bytes per sample.
    pub fn bytes_per_sample(&self) -> usize {
        match self {
            SampleFormat::I16 => 2,
            SampleFormat::I32 => 4,
            SampleFormat::F32 => 4,
            SampleFormat::F64 => 8,
        }
    }
}

/// Decoded audio buffer.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Sample data (interleaved channels).
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
    /// Duration of this buffer.
    pub duration: Duration,
}

impl AudioBuffer {
    /// Create a new audio buffer.
    pub fn new(samples: Vec<f32>, sample_rate: u32, channels: u16) -> Self {
        let num_frames = samples.len() / channels as usize;
        let duration = Duration::from_secs_f64(num_frames as f64 / sample_rate as f64);

        Self {
            samples,
            sample_rate,
            channels,
            duration,
        }
    }

    /// Get the number of frames (samples per channel).
    pub fn num_frames(&self) -> usize {
        self.samples.len() / self.channels as usize
    }

    /// Get a sample at a specific frame and channel.
    pub fn get_sample(&self, frame: usize, channel: usize) -> Option<f32> {
        if channel >= self.channels as usize {
            return None;
        }
        let index = frame * self.channels as usize + channel;
        self.samples.get(index).copied()
    }

    /// Get the RMS (root mean square) amplitude.
    pub fn rms(&self) -> f32 {
        if self.samples.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.samples.iter().map(|s| s * s).sum();
        (sum / self.samples.len() as f32).sqrt()
    }

    /// Get the peak amplitude.
    pub fn peak(&self) -> f32 {
        self.samples
            .iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Convert to mono by averaging channels.
    pub fn to_mono(&self) -> AudioBuffer {
        if self.channels == 1 {
            return self.clone();
        }

        let mut mono_samples = Vec::with_capacity(self.num_frames());
        for frame in 0..self.num_frames() {
            let mut sum = 0.0;
            for channel in 0..self.channels as usize {
                sum += self.get_sample(frame, channel).unwrap_or(0.0);
            }
            mono_samples.push(sum / self.channels as f32);
        }

        AudioBuffer::new(mono_samples, self.sample_rate, 1)
    }

    /// Resample to a different sample rate.
    pub fn resample(&self, target_rate: u32) -> AudioBuffer {
        if self.sample_rate == target_rate {
            return self.clone();
        }

        let ratio = target_rate as f64 / self.sample_rate as f64;
        let new_num_frames = (self.num_frames() as f64 * ratio) as usize;
        let mut new_samples = Vec::with_capacity(new_num_frames * self.channels as usize);

        for frame in 0..new_num_frames {
            let src_frame = frame as f64 / ratio;
            let src_frame_int = src_frame as usize;
            let frac = src_frame - src_frame_int as f64;

            for channel in 0..self.channels as usize {
                let s1 = self.get_sample(src_frame_int, channel).unwrap_or(0.0);
                let s2 = self.get_sample(src_frame_int + 1, channel).unwrap_or(s1);
                let interpolated = s1 + (s2 - s1) * frac as f32;
                new_samples.push(interpolated);
            }
        }

        AudioBuffer::new(new_samples, target_rate, self.channels)
    }
}

/// Audio decoder trait for format-specific decoders.
pub trait AudioDecoder: Send + Sync {
    /// Get the audio format.
    fn format(&self) -> AudioFormat;

    /// Get the audio metadata.
    fn metadata(&self) -> Result<AudioMetadata>;

    /// Decode the next buffer of samples.
    fn decode_next(&mut self) -> Result<Option<AudioBuffer>>;

    /// Seek to a specific position.
    fn seek(&mut self, position: Duration) -> Result<()>;

    /// Reset the decoder to the beginning.
    fn reset(&mut self) -> Result<()>;

    /// Check if the decoder supports seeking.
    fn supports_seeking(&self) -> bool {
        self.format().supports_seeking()
    }
}

/// Generic audio decoder that auto-detects format.
pub struct GenericDecoder {
    format: AudioFormat,
    metadata: AudioMetadata,
    position: Duration,
    total_duration: Option<Duration>,
}

impl GenericDecoder {
    /// Create a new decoder for a file path.
    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let format = AudioFormat::from_path(path);

        if format == AudioFormat::Unknown {
            return Err(DecoderError::UnsupportedFormat(
                path.to_string_lossy().to_string(),
            ));
        }

        info!("Creating decoder for {:?} file: {:?}", format, path);

        let metadata = AudioMetadata::with_format(format);

        Ok(Self {
            format,
            metadata,
            position: Duration::ZERO,
            total_duration: None,
        })
    }

    /// Create a new decoder from a reader with specified format.
    pub fn from_reader<R: Read + Seek + Send + 'static>(
        _reader: R,
        format: AudioFormat,
    ) -> Result<Self> {
        if format == AudioFormat::Unknown {
            return Err(DecoderError::UnsupportedFormat("unknown".to_string()));
        }

        let metadata = AudioMetadata::with_format(format);

        Ok(Self {
            format,
            metadata,
            position: Duration::ZERO,
            total_duration: None,
        })
    }

    /// Get the current position.
    pub fn position(&self) -> Duration {
        self.position
    }

    /// Get the total duration if known.
    pub fn duration(&self) -> Option<Duration> {
        self.total_duration
    }
}

impl AudioDecoder for GenericDecoder {
    fn format(&self) -> AudioFormat {
        self.format
    }

    fn metadata(&self) -> Result<AudioMetadata> {
        Ok(self.metadata.clone())
    }

    fn decode_next(&mut self) -> Result<Option<AudioBuffer>> {
        // Placeholder implementation
        // Real implementation would decode actual audio data
        debug!("Decoding next buffer at position {:?}", self.position);
        Ok(None)
    }

    fn seek(&mut self, position: Duration) -> Result<()> {
        if let Some(duration) = self.total_duration {
            if position > duration {
                return Err(DecoderError::DecodeError(format!(
                    "Seek position {:?} exceeds duration {:?}",
                    position, duration
                )));
            }
        }
        self.position = position;
        debug!("Seeked to position {:?}", position);
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        self.position = Duration::ZERO;
        debug!("Decoder reset to beginning");
        Ok(())
    }
}

/// Detect audio format from file header bytes.
pub fn detect_format_from_header(header: &[u8]) -> AudioFormat {
    if header.len() < 12 {
        return AudioFormat::Unknown;
    }

    // Check for various magic bytes
    if header.starts_with(b"ID3") || (header[0] == 0xFF && (header[1] & 0xE0) == 0xE0) {
        return AudioFormat::Mp3;
    }

    if header.starts_with(b"RIFF") && header.len() >= 12 && &header[8..12] == b"WAVE" {
        return AudioFormat::Wav;
    }

    if header.starts_with(b"OggS") {
        return AudioFormat::Ogg;
    }

    if header.starts_with(b"fLaC") {
        return AudioFormat::Flac;
    }

    if header.len() >= 8 && &header[4..8] == b"ftyp" {
        // Check for M4A/AAC
        if header.len() >= 12 {
            let ftyp = &header[8..12];
            if ftyp == b"M4A " || ftyp == b"mp42" || ftyp == b"isom" {
                return AudioFormat::Aac;
            }
        }
    }

    if header.starts_with(b"FORM") && header.len() >= 12 && &header[8..12] == b"AIFF" {
        return AudioFormat::Aiff;
    }

    AudioFormat::Unknown
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_from_extension() {
        assert_eq!(AudioFormat::from_extension("mp3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("MP3"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_extension("wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_extension("ogg"), AudioFormat::Ogg);
        assert_eq!(AudioFormat::from_extension("flac"), AudioFormat::Flac);
        assert_eq!(AudioFormat::from_extension("aac"), AudioFormat::Aac);
        assert_eq!(AudioFormat::from_extension("xyz"), AudioFormat::Unknown);
    }

    #[test]
    fn test_format_from_mime() {
        assert_eq!(AudioFormat::from_mime_type("audio/mpeg"), AudioFormat::Mp3);
        assert_eq!(AudioFormat::from_mime_type("audio/wav"), AudioFormat::Wav);
        assert_eq!(AudioFormat::from_mime_type("audio/ogg"), AudioFormat::Ogg);
        assert_eq!(AudioFormat::from_mime_type("audio/flac"), AudioFormat::Flac);
    }

    #[test]
    fn test_format_properties() {
        assert!(AudioFormat::Flac.is_lossless());
        assert!(AudioFormat::Wav.is_lossless());
        assert!(!AudioFormat::Mp3.is_lossless());
        assert!(!AudioFormat::Ogg.is_lossless());

        assert!(AudioFormat::Mp3.supports_seeking());
        assert!(AudioFormat::Wav.supports_seeking());
    }

    #[test]
    fn test_detect_format_from_header() {
        // MP3 with ID3 tag
        assert_eq!(
            detect_format_from_header(b"ID3\x04\x00\x00\x00\x00\x00\x00\x00\x00"),
            AudioFormat::Mp3
        );

        // WAV
        assert_eq!(
            detect_format_from_header(b"RIFF\x00\x00\x00\x00WAVEfmt "),
            AudioFormat::Wav
        );

        // OGG
        assert_eq!(
            detect_format_from_header(b"OggS\x00\x02\x00\x00\x00\x00\x00\x00"),
            AudioFormat::Ogg
        );

        // FLAC
        assert_eq!(
            detect_format_from_header(b"fLaC\x00\x00\x00\x22\x00\x00\x00\x00"),
            AudioFormat::Flac
        );
    }

    #[test]
    fn test_audio_buffer() {
        let samples = vec![0.0, 0.5, 1.0, -1.0, 0.5, 0.0];
        let buffer = AudioBuffer::new(samples, 44100, 2);

        assert_eq!(buffer.num_frames(), 3);
        assert_eq!(buffer.channels, 2);
        assert!((buffer.get_sample(0, 0).unwrap() - 0.0).abs() < 0.01);
        assert!((buffer.get_sample(0, 1).unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_audio_buffer_mono() {
        let samples = vec![0.0, 1.0, 0.5, 0.5, 1.0, 0.0];
        let buffer = AudioBuffer::new(samples, 44100, 2);
        let mono = buffer.to_mono();

        assert_eq!(mono.channels, 1);
        assert_eq!(mono.num_frames(), 3);
        // First frame: (0.0 + 1.0) / 2 = 0.5
        assert!((mono.get_sample(0, 0).unwrap() - 0.5).abs() < 0.01);
    }

    #[test]
    fn test_audio_buffer_rms() {
        let samples = vec![1.0, -1.0, 1.0, -1.0];
        let buffer = AudioBuffer::new(samples, 44100, 1);

        // RMS of [1, -1, 1, -1] = sqrt(4/4) = 1.0
        assert!((buffer.rms() - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_audio_buffer_peak() {
        let samples = vec![0.5, -0.8, 0.3, -0.2];
        let buffer = AudioBuffer::new(samples, 44100, 1);

        assert!((buffer.peak() - 0.8).abs() < 0.01);
    }

    #[test]
    fn test_audio_metadata() {
        let mut metadata = AudioMetadata::new();
        assert!(!metadata.has_tags());

        metadata.title = Some("Test Song".to_string());
        assert!(metadata.has_tags());

        metadata.duration = Some(Duration::from_secs(180));
        metadata.bitrate = Some(320);
        // 180 * 320 * 1000 / 8 = 7,200,000 bytes
        assert_eq!(metadata.approximate_file_size(), Some(7_200_000));
    }

    #[test]
    fn test_sample_format() {
        assert_eq!(SampleFormat::I16.bytes_per_sample(), 2);
        assert_eq!(SampleFormat::I32.bytes_per_sample(), 4);
        assert_eq!(SampleFormat::F32.bytes_per_sample(), 4);
        assert_eq!(SampleFormat::F64.bytes_per_sample(), 8);
    }
}
