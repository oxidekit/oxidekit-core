//! Mobile performance monitoring HUD

use std::collections::VecDeque;

/// Performance HUD for mobile devices
pub struct PerformanceHUD {
    enabled: bool,
    position: HUDPosition,
    fps_history: VecDeque<f32>,
    memory_history: VecDeque<f32>,
    history_size: usize,
}

/// Position of the HUD overlay
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HUDPosition {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}

impl Default for HUDPosition {
    fn default() -> Self {
        Self::TopRight
    }
}

impl PerformanceHUD {
    /// Create a new performance HUD
    pub fn new() -> Self {
        Self {
            enabled: false,
            position: HUDPosition::TopRight,
            fps_history: VecDeque::with_capacity(60),
            memory_history: VecDeque::with_capacity(60),
            history_size: 60,
        }
    }

    /// Create a new performance HUD with custom history size
    pub fn with_history_size(history_size: usize) -> Self {
        Self {
            enabled: false,
            position: HUDPosition::TopRight,
            fps_history: VecDeque::with_capacity(history_size),
            memory_history: VecDeque::with_capacity(history_size),
            history_size,
        }
    }

    /// Enable the HUD
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the HUD
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the HUD is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Toggle the HUD on/off
    pub fn toggle(&mut self) {
        self.enabled = !self.enabled;
    }

    /// Set the HUD position
    pub fn set_position(&mut self, position: HUDPosition) {
        self.position = position;
    }

    /// Get the current HUD position
    pub fn position(&self) -> HUDPosition {
        self.position
    }

    /// Record a frame with FPS and memory data
    pub fn record_frame(&mut self, fps: f32, memory_mb: f32) {
        if self.fps_history.len() >= self.history_size {
            self.fps_history.pop_front();
            self.memory_history.pop_front();
        }
        self.fps_history.push_back(fps);
        self.memory_history.push_back(memory_mb);
    }

    /// Get the average FPS over the history window
    pub fn average_fps(&self) -> f32 {
        if self.fps_history.is_empty() {
            return 0.0;
        }
        self.fps_history.iter().sum::<f32>() / self.fps_history.len() as f32
    }

    /// Get the current (most recent) FPS
    pub fn current_fps(&self) -> f32 {
        self.fps_history.back().copied().unwrap_or(0.0)
    }

    /// Get the current (most recent) memory usage
    pub fn current_memory(&self) -> f32 {
        self.memory_history.back().copied().unwrap_or(0.0)
    }

    /// Get the average memory usage over the history window
    pub fn average_memory(&self) -> f32 {
        if self.memory_history.is_empty() {
            return 0.0;
        }
        self.memory_history.iter().sum::<f32>() / self.memory_history.len() as f32
    }

    /// Get the minimum FPS in the history window
    pub fn min_fps(&self) -> f32 {
        self.fps_history
            .iter()
            .copied()
            .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the maximum FPS in the history window
    pub fn max_fps(&self) -> f32 {
        self.fps_history
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the maximum memory usage in the history window
    pub fn peak_memory(&self) -> f32 {
        self.memory_history
            .iter()
            .copied()
            .max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(0.0)
    }

    /// Get the FPS history for graphing
    pub fn fps_history(&self) -> &VecDeque<f32> {
        &self.fps_history
    }

    /// Get the memory history for graphing
    pub fn memory_history(&self) -> &VecDeque<f32> {
        &self.memory_history
    }

    /// Clear all history
    pub fn clear_history(&mut self) {
        self.fps_history.clear();
        self.memory_history.clear();
    }

    /// Get the history size
    pub fn history_size(&self) -> usize {
        self.history_size
    }

    /// Get the current sample count
    pub fn sample_count(&self) -> usize {
        self.fps_history.len()
    }

    /// Check if performance is degraded (average FPS below 50)
    pub fn is_degraded(&self) -> bool {
        self.average_fps() < 50.0 && !self.fps_history.is_empty()
    }

    /// Check if there's a memory warning (above 200MB)
    pub fn has_memory_warning(&self) -> bool {
        self.current_memory() > 200.0
    }
}

impl Default for PerformanceHUD {
    fn default() -> Self {
        Self::new()
    }
}

/// Performance stats summary
#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub current_fps: f32,
    pub average_fps: f32,
    pub min_fps: f32,
    pub max_fps: f32,
    pub current_memory_mb: f32,
    pub average_memory_mb: f32,
    pub peak_memory_mb: f32,
    pub sample_count: usize,
}

impl PerformanceHUD {
    /// Get a summary of current performance stats
    pub fn stats(&self) -> PerformanceStats {
        PerformanceStats {
            current_fps: self.current_fps(),
            average_fps: self.average_fps(),
            min_fps: self.min_fps(),
            max_fps: self.max_fps(),
            current_memory_mb: self.current_memory(),
            average_memory_mb: self.average_memory(),
            peak_memory_mb: self.peak_memory(),
            sample_count: self.sample_count(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_hud_enable_disable() {
        let mut hud = PerformanceHUD::new();
        assert!(!hud.is_enabled());

        hud.enable();
        assert!(hud.is_enabled());

        hud.disable();
        assert!(!hud.is_enabled());
    }

    #[test]
    fn test_performance_hud_toggle() {
        let mut hud = PerformanceHUD::default();
        assert!(!hud.is_enabled());

        hud.toggle();
        assert!(hud.is_enabled());

        hud.toggle();
        assert!(!hud.is_enabled());
    }

    #[test]
    fn test_performance_hud_position() {
        let mut hud = PerformanceHUD::new();
        assert_eq!(hud.position(), HUDPosition::TopRight);

        hud.set_position(HUDPosition::BottomLeft);
        assert_eq!(hud.position(), HUDPosition::BottomLeft);
    }

    #[test]
    fn test_record_frame() {
        let mut hud = PerformanceHUD::new();
        assert_eq!(hud.sample_count(), 0);

        hud.record_frame(60.0, 100.0);
        assert_eq!(hud.sample_count(), 1);
        assert_eq!(hud.current_fps(), 60.0);
        assert_eq!(hud.current_memory(), 100.0);

        hud.record_frame(55.0, 120.0);
        assert_eq!(hud.sample_count(), 2);
        assert_eq!(hud.current_fps(), 55.0);
        assert_eq!(hud.current_memory(), 120.0);
    }

    #[test]
    fn test_average_fps() {
        let mut hud = PerformanceHUD::new();
        assert_eq!(hud.average_fps(), 0.0);

        hud.record_frame(60.0, 100.0);
        hud.record_frame(50.0, 100.0);
        hud.record_frame(40.0, 100.0);

        assert_eq!(hud.average_fps(), 50.0);
    }

    #[test]
    fn test_min_max_fps() {
        let mut hud = PerformanceHUD::new();

        hud.record_frame(60.0, 100.0);
        hud.record_frame(45.0, 100.0);
        hud.record_frame(55.0, 100.0);

        assert_eq!(hud.min_fps(), 45.0);
        assert_eq!(hud.max_fps(), 60.0);
    }

    #[test]
    fn test_history_limit() {
        let mut hud = PerformanceHUD::with_history_size(3);

        for i in 0..5 {
            hud.record_frame(60.0 - i as f32, 100.0);
        }

        assert_eq!(hud.sample_count(), 3);
        // Should only have the last 3 values: 58, 57, 56
        assert_eq!(hud.max_fps(), 58.0);
        assert_eq!(hud.min_fps(), 56.0);
    }

    #[test]
    fn test_clear_history() {
        let mut hud = PerformanceHUD::new();

        hud.record_frame(60.0, 100.0);
        hud.record_frame(55.0, 120.0);
        assert_eq!(hud.sample_count(), 2);

        hud.clear_history();
        assert_eq!(hud.sample_count(), 0);
        assert_eq!(hud.current_fps(), 0.0);
    }

    #[test]
    fn test_is_degraded() {
        let mut hud = PerformanceHUD::new();
        assert!(!hud.is_degraded()); // Empty history

        hud.record_frame(60.0, 100.0);
        assert!(!hud.is_degraded());

        hud.record_frame(30.0, 100.0);
        hud.record_frame(40.0, 100.0);
        // Average is now 43.33, which is below 50
        assert!(hud.is_degraded());
    }

    #[test]
    fn test_memory_warning() {
        let mut hud = PerformanceHUD::new();
        assert!(!hud.has_memory_warning());

        hud.record_frame(60.0, 150.0);
        assert!(!hud.has_memory_warning());

        hud.record_frame(60.0, 250.0);
        assert!(hud.has_memory_warning());
    }

    #[test]
    fn test_stats_summary() {
        let mut hud = PerformanceHUD::new();

        hud.record_frame(60.0, 100.0);
        hud.record_frame(50.0, 150.0);
        hud.record_frame(55.0, 120.0);

        let stats = hud.stats();
        assert_eq!(stats.current_fps, 55.0);
        assert_eq!(stats.min_fps, 50.0);
        assert_eq!(stats.max_fps, 60.0);
        assert_eq!(stats.current_memory_mb, 120.0);
        assert_eq!(stats.peak_memory_mb, 150.0);
        assert_eq!(stats.sample_count, 3);
    }
}
