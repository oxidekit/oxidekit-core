//! Waveform visualization and analysis.
//!
//! Provides waveform data extraction and visualization generation
//! for audio visualization components.

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::debug;

use super::decoder::AudioBuffer;

/// Waveform error types.
#[derive(Debug, Error)]
pub enum WaveformError {
    /// Invalid parameters.
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),

    /// Insufficient data.
    #[error("Insufficient audio data")]
    InsufficientData,

    /// Processing error.
    #[error("Processing error: {0}")]
    Processing(String),
}

/// Result type for waveform operations.
pub type Result<T> = std::result::Result<T, WaveformError>;

/// Waveform data for visualization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaveformData {
    /// Peak values for each segment (0.0 to 1.0).
    pub peaks: Vec<f32>,
    /// RMS values for each segment (0.0 to 1.0).
    pub rms: Vec<f32>,
    /// Number of samples per segment.
    pub samples_per_segment: usize,
    /// Sample rate of the source audio.
    pub sample_rate: u32,
    /// Total duration of the audio.
    pub duration: Duration,
    /// Number of channels.
    pub channels: u16,
}

impl WaveformData {
    /// Get the duration of each segment.
    pub fn segment_duration(&self) -> Duration {
        Duration::from_secs_f64(self.samples_per_segment as f64 / self.sample_rate as f64)
    }

    /// Get the segment index for a given time position.
    pub fn segment_at(&self, time: Duration) -> Option<usize> {
        let segment_dur = self.segment_duration();
        if segment_dur.is_zero() {
            return None;
        }
        let index = (time.as_secs_f64() / segment_dur.as_secs_f64()) as usize;
        if index < self.peaks.len() {
            Some(index)
        } else {
            None
        }
    }

    /// Get the peak value at a given time.
    pub fn peak_at(&self, time: Duration) -> Option<f32> {
        self.segment_at(time).and_then(|i| self.peaks.get(i).copied())
    }

    /// Get the RMS value at a given time.
    pub fn rms_at(&self, time: Duration) -> Option<f32> {
        self.segment_at(time).and_then(|i| self.rms.get(i).copied())
    }

    /// Downsample the waveform to a specific number of segments.
    pub fn downsample(&self, target_segments: usize) -> WaveformData {
        if target_segments >= self.peaks.len() || target_segments == 0 {
            return self.clone();
        }

        let ratio = self.peaks.len() as f64 / target_segments as f64;
        let mut new_peaks = Vec::with_capacity(target_segments);
        let mut new_rms = Vec::with_capacity(target_segments);

        for i in 0..target_segments {
            let start = (i as f64 * ratio) as usize;
            let end = ((i + 1) as f64 * ratio) as usize;
            let end = end.min(self.peaks.len());

            if start < end {
                let peak = self.peaks[start..end]
                    .iter()
                    .max_by(|a, b| a.partial_cmp(b).unwrap())
                    .copied()
                    .unwrap_or(0.0);
                let rms = (self.rms[start..end].iter().map(|v| v * v).sum::<f32>()
                    / (end - start) as f32)
                    .sqrt();

                new_peaks.push(peak);
                new_rms.push(rms);
            }
        }

        WaveformData {
            peaks: new_peaks,
            rms: new_rms,
            samples_per_segment: (self.samples_per_segment as f64 * ratio) as usize,
            sample_rate: self.sample_rate,
            duration: self.duration,
            channels: self.channels,
        }
    }

    /// Normalize the waveform to fill the 0.0-1.0 range.
    pub fn normalize(&mut self) {
        let max_peak = self
            .peaks
            .iter()
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .copied()
            .unwrap_or(1.0);

        if max_peak > 0.0 {
            for peak in &mut self.peaks {
                *peak /= max_peak;
            }
            for rms in &mut self.rms {
                *rms /= max_peak;
            }
        }
    }
}

/// Waveform generator configuration.
#[derive(Debug, Clone)]
pub struct WaveformConfig {
    /// Number of segments to generate.
    pub segments: usize,
    /// Whether to normalize the output.
    pub normalize: bool,
    /// Whether to generate stereo waveform (separate channels).
    pub stereo: bool,
}

impl Default for WaveformConfig {
    fn default() -> Self {
        Self {
            segments: 200,
            normalize: true,
            stereo: false,
        }
    }
}

impl WaveformConfig {
    /// Create a new configuration with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the number of segments.
    pub fn segments(mut self, segments: usize) -> Self {
        self.segments = segments;
        self
    }

    /// Enable or disable normalization.
    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    /// Enable or disable stereo output.
    pub fn stereo(mut self, stereo: bool) -> Self {
        self.stereo = stereo;
        self
    }
}

/// Waveform generator for creating visualization data.
pub struct WaveformGenerator {
    config: WaveformConfig,
}

impl WaveformGenerator {
    /// Create a new waveform generator with default config.
    pub fn new() -> Self {
        Self {
            config: WaveformConfig::default(),
        }
    }

    /// Create a new waveform generator with custom config.
    pub fn with_config(config: WaveformConfig) -> Self {
        Self { config }
    }

    /// Generate waveform data from an audio buffer.
    pub fn generate(&self, buffer: &AudioBuffer) -> Result<WaveformData> {
        if buffer.samples.is_empty() {
            return Err(WaveformError::InsufficientData);
        }

        if self.config.segments == 0 {
            return Err(WaveformError::InvalidParameters(
                "segments must be greater than 0".to_string(),
            ));
        }

        let mono = if buffer.channels > 1 && !self.config.stereo {
            buffer.to_mono()
        } else {
            buffer.clone()
        };

        let samples_per_segment = (mono.num_frames() / self.config.segments).max(1);
        let num_segments = (mono.num_frames() / samples_per_segment).max(1);

        let mut peaks = Vec::with_capacity(num_segments);
        let mut rms_values = Vec::with_capacity(num_segments);

        for i in 0..num_segments {
            let start = i * samples_per_segment;
            let end = ((i + 1) * samples_per_segment).min(mono.num_frames());

            if start >= mono.samples.len() {
                break;
            }

            let segment_samples: Vec<f32> = (start..end)
                .filter_map(|frame| mono.get_sample(frame, 0))
                .collect();

            if segment_samples.is_empty() {
                peaks.push(0.0);
                rms_values.push(0.0);
                continue;
            }

            // Calculate peak
            let peak = segment_samples
                .iter()
                .map(|s| s.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);

            // Calculate RMS
            let rms = (segment_samples.iter().map(|s| s * s).sum::<f32>()
                / segment_samples.len() as f32)
                .sqrt();

            peaks.push(peak);
            rms_values.push(rms);
        }

        let mut waveform = WaveformData {
            peaks,
            rms: rms_values,
            samples_per_segment,
            sample_rate: mono.sample_rate,
            duration: mono.duration,
            channels: mono.channels,
        };

        if self.config.normalize {
            waveform.normalize();
        }

        debug!(
            "Generated waveform with {} segments from {} samples",
            waveform.peaks.len(),
            buffer.samples.len()
        );

        Ok(waveform)
    }

    /// Generate waveform from raw samples.
    pub fn generate_from_samples(
        &self,
        samples: &[f32],
        sample_rate: u32,
        channels: u16,
    ) -> Result<WaveformData> {
        let buffer = AudioBuffer::new(samples.to_vec(), sample_rate, channels);
        self.generate(&buffer)
    }
}

impl Default for WaveformGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Stereo waveform data with separate left and right channels.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StereoWaveformData {
    /// Left channel waveform.
    pub left: WaveformData,
    /// Right channel waveform.
    pub right: WaveformData,
}

impl StereoWaveformData {
    /// Create from a stereo audio buffer.
    pub fn from_buffer(buffer: &AudioBuffer, config: &WaveformConfig) -> Result<Self> {
        if buffer.channels < 2 {
            return Err(WaveformError::InvalidParameters(
                "Audio must be stereo".to_string(),
            ));
        }

        let samples_per_segment = (buffer.num_frames() / config.segments).max(1);
        let num_segments = (buffer.num_frames() / samples_per_segment).max(1);

        let mut left_peaks = Vec::with_capacity(num_segments);
        let mut left_rms = Vec::with_capacity(num_segments);
        let mut right_peaks = Vec::with_capacity(num_segments);
        let mut right_rms = Vec::with_capacity(num_segments);

        for i in 0..num_segments {
            let start = i * samples_per_segment;
            let end = ((i + 1) * samples_per_segment).min(buffer.num_frames());

            let left_samples: Vec<f32> = (start..end)
                .filter_map(|frame| buffer.get_sample(frame, 0))
                .collect();

            let right_samples: Vec<f32> = (start..end)
                .filter_map(|frame| buffer.get_sample(frame, 1))
                .collect();

            // Left channel
            let l_peak = left_samples
                .iter()
                .map(|s| s.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            let l_rms = if !left_samples.is_empty() {
                (left_samples.iter().map(|s| s * s).sum::<f32>() / left_samples.len() as f32)
                    .sqrt()
            } else {
                0.0
            };

            // Right channel
            let r_peak = right_samples
                .iter()
                .map(|s| s.abs())
                .max_by(|a, b| a.partial_cmp(b).unwrap())
                .unwrap_or(0.0);
            let r_rms = if !right_samples.is_empty() {
                (right_samples.iter().map(|s| s * s).sum::<f32>() / right_samples.len() as f32)
                    .sqrt()
            } else {
                0.0
            };

            left_peaks.push(l_peak);
            left_rms.push(l_rms);
            right_peaks.push(r_peak);
            right_rms.push(r_rms);
        }

        let left = WaveformData {
            peaks: left_peaks,
            rms: left_rms,
            samples_per_segment,
            sample_rate: buffer.sample_rate,
            duration: buffer.duration,
            channels: 1,
        };

        let right = WaveformData {
            peaks: right_peaks,
            rms: right_rms,
            samples_per_segment,
            sample_rate: buffer.sample_rate,
            duration: buffer.duration,
            channels: 1,
        };

        Ok(Self { left, right })
    }
}

/// Real-time waveform analyzer for live audio visualization.
pub struct RealtimeWaveformAnalyzer {
    /// Ring buffer for samples.
    buffer: Vec<f32>,
    /// Buffer size.
    buffer_size: usize,
    /// Current write position.
    write_pos: usize,
    /// Sample rate.
    sample_rate: u32,
}

impl RealtimeWaveformAnalyzer {
    /// Create a new realtime analyzer with given buffer duration.
    pub fn new(buffer_duration: Duration, sample_rate: u32) -> Self {
        let buffer_size = (buffer_duration.as_secs_f64() * sample_rate as f64) as usize;

        Self {
            buffer: vec![0.0; buffer_size],
            buffer_size,
            write_pos: 0,
            sample_rate,
        }
    }

    /// Push new samples into the analyzer.
    pub fn push_samples(&mut self, samples: &[f32]) {
        for &sample in samples {
            self.buffer[self.write_pos] = sample;
            self.write_pos = (self.write_pos + 1) % self.buffer_size;
        }
    }

    /// Get the current peak amplitude.
    pub fn current_peak(&self) -> f32 {
        self.buffer
            .iter()
            .map(|s| s.abs())
            .max_by(|a, b| a.partial_cmp(b).unwrap())
            .unwrap_or(0.0)
    }

    /// Get the current RMS amplitude.
    pub fn current_rms(&self) -> f32 {
        if self.buffer.is_empty() {
            return 0.0;
        }
        let sum: f32 = self.buffer.iter().map(|s| s * s).sum();
        (sum / self.buffer.len() as f32).sqrt()
    }

    /// Get the buffer as waveform data.
    pub fn as_waveform(&self, segments: usize) -> WaveformData {
        let generator = WaveformGenerator::with_config(WaveformConfig::new().segments(segments));

        let ordered_buffer = self.get_ordered_buffer();
        generator
            .generate_from_samples(&ordered_buffer, self.sample_rate, 1)
            .unwrap_or_else(|_| WaveformData {
                peaks: vec![0.0; segments],
                rms: vec![0.0; segments],
                samples_per_segment: self.buffer_size / segments,
                sample_rate: self.sample_rate,
                duration: Duration::from_secs_f64(self.buffer_size as f64 / self.sample_rate as f64),
                channels: 1,
            })
    }

    /// Get buffer in chronological order.
    fn get_ordered_buffer(&self) -> Vec<f32> {
        let mut ordered = Vec::with_capacity(self.buffer_size);
        ordered.extend_from_slice(&self.buffer[self.write_pos..]);
        ordered.extend_from_slice(&self.buffer[..self.write_pos]);
        ordered
    }

    /// Clear the buffer.
    pub fn clear(&mut self) {
        self.buffer.fill(0.0);
        self.write_pos = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_buffer() -> AudioBuffer {
        // Create a simple sine wave
        let sample_rate = 44100;
        let duration_secs = 1.0;
        let num_samples = (sample_rate as f64 * duration_secs) as usize;

        let samples: Vec<f32> = (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                (t * 440.0 * 2.0 * std::f32::consts::PI).sin()
            })
            .collect();

        AudioBuffer::new(samples, sample_rate, 1)
    }

    #[test]
    fn test_waveform_generation() {
        let buffer = create_test_buffer();
        let generator = WaveformGenerator::with_config(WaveformConfig::new().segments(100));

        let waveform = generator.generate(&buffer).unwrap();

        assert_eq!(waveform.peaks.len(), 100);
        assert_eq!(waveform.rms.len(), 100);
        assert!(waveform.peaks.iter().all(|&p| p >= 0.0 && p <= 1.0));
    }

    #[test]
    fn test_waveform_normalization() {
        let samples = vec![0.25, 0.5, 0.25, 0.5];
        let buffer = AudioBuffer::new(samples, 44100, 1);

        let generator = WaveformGenerator::with_config(
            WaveformConfig::new().segments(2).normalize(true),
        );

        let waveform = generator.generate(&buffer).unwrap();

        // After normalization, max peak should be 1.0
        let max_peak = waveform.peaks.iter().cloned().fold(0.0, f32::max);
        assert!((max_peak - 1.0).abs() < 0.01);
    }

    #[test]
    fn test_waveform_downsample() {
        let buffer = create_test_buffer();
        let generator = WaveformGenerator::with_config(WaveformConfig::new().segments(200));

        let waveform = generator.generate(&buffer).unwrap();
        let downsampled = waveform.downsample(50);

        assert_eq!(downsampled.peaks.len(), 50);
        assert_eq!(downsampled.rms.len(), 50);
    }

    #[test]
    fn test_segment_at() {
        let waveform = WaveformData {
            peaks: vec![0.5; 100],
            rms: vec![0.3; 100],
            samples_per_segment: 441,
            sample_rate: 44100,
            duration: Duration::from_secs(1),
            channels: 1,
        };

        // Each segment is 441/44100 = 0.01 seconds
        assert_eq!(waveform.segment_at(Duration::from_millis(0)), Some(0));
        assert_eq!(waveform.segment_at(Duration::from_millis(50)), Some(5));
        assert_eq!(waveform.segment_at(Duration::from_millis(990)), Some(99));
    }

    #[test]
    fn test_realtime_analyzer() {
        let mut analyzer = RealtimeWaveformAnalyzer::new(Duration::from_millis(100), 44100);

        // Push some samples
        let samples: Vec<f32> = (0..4410)
            .map(|i| (i as f32 / 44100.0 * 440.0 * 2.0 * std::f32::consts::PI).sin())
            .collect();

        analyzer.push_samples(&samples);

        let peak = analyzer.current_peak();
        let rms = analyzer.current_rms();

        assert!(peak > 0.0);
        assert!(rms > 0.0);
        assert!(peak >= rms);
    }

    #[test]
    fn test_empty_buffer_error() {
        let buffer = AudioBuffer::new(vec![], 44100, 1);
        let generator = WaveformGenerator::new();

        let result = generator.generate(&buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_zero_segments_error() {
        let buffer = create_test_buffer();
        let generator = WaveformGenerator::with_config(WaveformConfig::new().segments(0));

        let result = generator.generate(&buffer);
        assert!(result.is_err());
    }

    #[test]
    fn test_stereo_waveform() {
        // Create stereo buffer
        let sample_rate = 44100;
        let num_frames = 4410;
        let mut samples = Vec::with_capacity(num_frames * 2);

        for i in 0..num_frames {
            let t = i as f32 / sample_rate as f32;
            // Left channel: 440 Hz
            samples.push((t * 440.0 * 2.0 * std::f32::consts::PI).sin());
            // Right channel: 880 Hz
            samples.push((t * 880.0 * 2.0 * std::f32::consts::PI).sin());
        }

        let buffer = AudioBuffer::new(samples, sample_rate, 2);
        let config = WaveformConfig::new().segments(50);

        let stereo = StereoWaveformData::from_buffer(&buffer, &config).unwrap();

        assert_eq!(stereo.left.peaks.len(), 50);
        assert_eq!(stereo.right.peaks.len(), 50);
    }
}
