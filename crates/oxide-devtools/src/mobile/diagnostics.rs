//! Mobile diagnostics collection

use serde::{Deserialize, Serialize};

/// Mobile device diagnostics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MobileDiagnostics {
    pub device: DeviceInfo,
    pub app: AppInfo,
    pub performance: PerformanceSnapshot,
    pub memory: MemoryInfo,
    pub battery: Option<BatteryInfo>,
    pub network: NetworkInfo,
}

/// Device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub platform: String,        // "iOS", "Android"
    pub os_version: String,
    pub device_model: String,
    pub device_id: Option<String>, // Redacted in production
    pub screen_width: u32,
    pub screen_height: u32,
    pub scale_factor: f32,
    pub safe_area_insets: SafeAreaInfo,
    pub locale: String,
    pub timezone: String,
}

impl Default for DeviceInfo {
    fn default() -> Self {
        Self {
            platform: std::env::consts::OS.to_string(),
            os_version: "Unknown".to_string(),
            device_model: "Unknown".to_string(),
            device_id: None,
            screen_width: 0,
            screen_height: 0,
            scale_factor: 1.0,
            safe_area_insets: SafeAreaInfo::default(),
            locale: "en-US".to_string(),
            timezone: "UTC".to_string(),
        }
    }
}

/// Safe area insets for notched displays
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SafeAreaInfo {
    pub top: f32,
    pub bottom: f32,
    pub left: f32,
    pub right: f32,
}

impl SafeAreaInfo {
    /// Create new safe area insets
    pub fn new(top: f32, bottom: f32, left: f32, right: f32) -> Self {
        Self { top, bottom, left, right }
    }

    /// Check if any safe area insets are present
    pub fn has_insets(&self) -> bool {
        self.top > 0.0 || self.bottom > 0.0 || self.left > 0.0 || self.right > 0.0
    }
}

/// Application information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub bundle_id: String,
    pub version: String,
    pub build_number: String,
    pub oxide_version: String,
    pub debug_build: bool,
}

impl Default for AppInfo {
    fn default() -> Self {
        Self {
            bundle_id: "com.example.app".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_number: "1".to_string(),
            oxide_version: env!("CARGO_PKG_VERSION").to_string(),
            debug_build: cfg!(debug_assertions),
        }
    }
}

/// Performance snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub fps_current: f32,
    pub fps_average: f32,
    pub frame_time_ms: f32,
    pub render_time_ms: f32,
    pub layout_time_ms: f32,
    pub dropped_frames: u64,
}

impl Default for PerformanceSnapshot {
    fn default() -> Self {
        Self {
            fps_current: 60.0,
            fps_average: 60.0,
            frame_time_ms: 16.67,
            render_time_ms: 8.0,
            layout_time_ms: 2.0,
            dropped_frames: 0,
        }
    }
}

impl PerformanceSnapshot {
    /// Check if performance is below target (60 FPS)
    pub fn is_below_target(&self) -> bool {
        self.fps_current < 55.0 || self.dropped_frames > 0
    }
}

/// Memory information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub app_memory_mb: f32,
    pub system_memory_mb: f32,
    pub memory_pressure: MemoryPressure,
}

impl Default for MemoryInfo {
    fn default() -> Self {
        Self {
            app_memory_mb: 0.0,
            system_memory_mb: 0.0,
            memory_pressure: MemoryPressure::Normal,
        }
    }
}

/// Memory pressure level
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum MemoryPressure {
    Normal,
    Warning,
    Critical,
}

impl Default for MemoryPressure {
    fn default() -> Self {
        Self::Normal
    }
}

/// Battery information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatteryInfo {
    pub level: f32,          // 0.0 - 1.0
    pub charging: bool,
    pub low_power_mode: bool,
}

impl BatteryInfo {
    /// Check if battery level is low (below 20%)
    pub fn is_low(&self) -> bool {
        self.level < 0.2 && !self.charging
    }
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub connection_type: ConnectionType,
    pub reachable: bool,
}

impl Default for NetworkInfo {
    fn default() -> Self {
        Self {
            connection_type: ConnectionType::Unknown,
            reachable: true,
        }
    }
}

/// Network connection type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionType {
    None,
    Wifi,
    Cellular,
    Ethernet,
    Unknown,
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::Unknown
    }
}

/// Collect mobile diagnostics
pub struct MobileDiagnosticsCollector {
    snapshots: Vec<MobileDiagnostics>,
    max_snapshots: usize,
}

impl MobileDiagnosticsCollector {
    /// Create a new diagnostics collector
    pub fn new(max_snapshots: usize) -> Self {
        Self {
            snapshots: Vec::with_capacity(max_snapshots),
            max_snapshots,
        }
    }

    /// Capture a diagnostics snapshot
    pub fn capture(&mut self) -> MobileDiagnostics {
        let diag = MobileDiagnostics {
            device: self.collect_device_info(),
            app: self.collect_app_info(),
            performance: self.collect_performance(),
            memory: self.collect_memory(),
            battery: self.collect_battery(),
            network: self.collect_network(),
        };

        if self.snapshots.len() >= self.max_snapshots {
            self.snapshots.remove(0);
        }
        self.snapshots.push(diag.clone());

        diag
    }

    /// Get all captured snapshots
    pub fn snapshots(&self) -> &[MobileDiagnostics] {
        &self.snapshots
    }

    /// Get the number of captured snapshots
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Clear all snapshots
    pub fn clear(&mut self) {
        self.snapshots.clear();
    }

    /// Export diagnostics bundle
    pub fn export_bundle(&self) -> DiagnosticsBundle {
        DiagnosticsBundle {
            collected_at: chrono::Utc::now().to_rfc3339(),
            snapshots: self.snapshots.clone(),
        }
    }

    fn collect_device_info(&self) -> DeviceInfo {
        DeviceInfo::default()
    }

    fn collect_app_info(&self) -> AppInfo {
        AppInfo::default()
    }

    fn collect_performance(&self) -> PerformanceSnapshot {
        PerformanceSnapshot::default()
    }

    fn collect_memory(&self) -> MemoryInfo {
        MemoryInfo::default()
    }

    fn collect_battery(&self) -> Option<BatteryInfo> {
        None // Platform-specific
    }

    fn collect_network(&self) -> NetworkInfo {
        NetworkInfo::default()
    }
}

impl Default for MobileDiagnosticsCollector {
    fn default() -> Self {
        Self::new(100)
    }
}

/// Bundle of diagnostics snapshots for export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsBundle {
    pub collected_at: String,
    pub snapshots: Vec<MobileDiagnostics>,
}

impl DiagnosticsBundle {
    /// Export to JSON string
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Get the number of snapshots in the bundle
    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    /// Check if the bundle is empty
    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostics_collector_capture() {
        let mut collector = MobileDiagnosticsCollector::new(10);
        assert_eq!(collector.snapshot_count(), 0);

        collector.capture();
        assert_eq!(collector.snapshot_count(), 1);

        collector.capture();
        assert_eq!(collector.snapshot_count(), 2);
    }

    #[test]
    fn test_diagnostics_collector_max_snapshots() {
        let mut collector = MobileDiagnosticsCollector::new(3);

        for _ in 0..5 {
            collector.capture();
        }

        assert_eq!(collector.snapshot_count(), 3);
    }

    #[test]
    fn test_diagnostics_bundle_export() {
        let mut collector = MobileDiagnosticsCollector::new(10);
        collector.capture();
        collector.capture();

        let bundle = collector.export_bundle();
        assert_eq!(bundle.len(), 2);
        assert!(!bundle.is_empty());

        let json = bundle.to_json().unwrap();
        assert!(json.contains("collected_at"));
        assert!(json.contains("snapshots"));
    }

    #[test]
    fn test_safe_area_insets() {
        let no_insets = SafeAreaInfo::default();
        assert!(!no_insets.has_insets());

        let with_insets = SafeAreaInfo::new(47.0, 34.0, 0.0, 0.0);
        assert!(with_insets.has_insets());
    }

    #[test]
    fn test_performance_below_target() {
        let good = PerformanceSnapshot::default();
        assert!(!good.is_below_target());

        let bad = PerformanceSnapshot {
            fps_current: 45.0,
            ..Default::default()
        };
        assert!(bad.is_below_target());
    }

    #[test]
    fn test_battery_is_low() {
        let low = BatteryInfo {
            level: 0.15,
            charging: false,
            low_power_mode: false,
        };
        assert!(low.is_low());

        let charging = BatteryInfo {
            level: 0.15,
            charging: true,
            low_power_mode: false,
        };
        assert!(!charging.is_low());

        let ok = BatteryInfo {
            level: 0.5,
            charging: false,
            low_power_mode: false,
        };
        assert!(!ok.is_low());
    }

    #[test]
    fn test_collector_clear() {
        let mut collector = MobileDiagnosticsCollector::new(10);
        collector.capture();
        collector.capture();
        assert_eq!(collector.snapshot_count(), 2);

        collector.clear();
        assert_eq!(collector.snapshot_count(), 0);
    }
}
