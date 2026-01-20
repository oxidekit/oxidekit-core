//! OxideKit Development Tools
//!
//! Provides dev-only in-app editing system that goes beyond "inspect element":
//! - Component selection and inspection
//! - Live style tweaking with token-based editing
//! - Context action menus for rapid iteration
//! - Patch-to-source with undo/redo
//! - Performance profiling
//! - Layout debugging overlays
//!
//! # Feature Flags
//!
//! - `dev-editor`: Enables the full dev editor UI (MUST be disabled in production)
//! - `diagnostics-export`: Read-only diagnostics export (safe for release builds)
//!
//! # Production Safety
//!
//! The dev editor is gated behind the `dev-editor` feature flag. When this flag
//! is not enabled, all dev editor code is completely stripped from the binary.
//!
//! ```rust,ignore
//! // Only available with dev-editor feature
//! #[cfg(feature = "dev-editor")]
//! fn enable_dev_tools() {
//!     // Dev-only code
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::time::Duration;

// Core modules (always available)
mod profiler;
pub use profiler::*;

// Inspector module (always available for basic inspection)
pub mod inspector;
pub use inspector::{InspectorState, ComponentInfo, SourceLocation};

// Component tree module
pub mod tree;
pub use tree::{ComponentTree, ComponentNode, NodeHandle};

// Dev editor modules (only when feature is enabled)
#[cfg(feature = "dev-editor")]
pub mod editor;
#[cfg(feature = "dev-editor")]
pub use editor::*;

#[cfg(feature = "dev-editor")]
pub mod actions;
#[cfg(feature = "dev-editor")]
pub use actions::*;

#[cfg(feature = "dev-editor")]
pub mod overlay;
#[cfg(feature = "dev-editor")]
pub use overlay::*;

#[cfg(feature = "dev-editor")]
pub mod shortcuts;
#[cfg(feature = "dev-editor")]
pub use shortcuts::*;

#[cfg(feature = "dev-editor")]
pub mod patch;
#[cfg(feature = "dev-editor")]
pub use patch::{EditPatch, PatchOperation, OperationType, PatchHistory, SourcePatcher, PatchBundle, PatchError};

#[cfg(feature = "dev-editor")]
pub mod token_editor;
#[cfg(feature = "dev-editor")]
pub use token_editor::{TokenEditor, TokenValue, TokenOverrides, ResolvedToken, TokenSource, TokenCategory, TokenInfo};

#[cfg(feature = "dev-editor")]
pub mod code_export;
#[cfg(feature = "dev-editor")]
pub use code_export::*;

// Diagnostics export (safe for release+diagnostics builds)
#[cfg(feature = "diagnostics-export")]
pub mod diagnostics;
#[cfg(feature = "diagnostics-export")]
pub use diagnostics::*;

// Hot reload system (always available in dev builds)
pub mod hot_reload;
pub use hot_reload::{
    HotReloadRuntime, HotReloadConfig, HotReloadHandle,
    HotReloadEvent, EventBus,
    FileWatcher, WatchEvent, WatchEventKind,
    StateManager, StateSnapshot,
    IncrementalCompiler, CompileResult,
    DevServer, DevServerConfig,
    ErrorOverlay, OverlayConfig, DiagnosticDisplay,
};

// Mobile-specific development tools
pub mod mobile;
pub use mobile::{
    // Inspector
    MobileInspector, MobileInspectorConfig, InspectorActivation,
    ComponentId, TouchPoint, TouchPhase,
    ComponentInspectInfo, Bounds, PropInfo, StyleInfo, AccessibilityInfo,
    // Diagnostics
    MobileDiagnostics, MobileDiagnosticsCollector, DiagnosticsBundle,
    DeviceInfo, SafeAreaInfo, AppInfo, PerformanceSnapshot,
    MemoryInfo, MemoryPressure, BatteryInfo, NetworkInfo, ConnectionType,
    // Performance HUD
    PerformanceHUD, HUDPosition, PerformanceStats,
};

/// Performance metrics for a frame
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameMetrics {
    /// Total frame time
    pub frame_time: Duration,
    /// Layout computation time
    pub layout_time: Duration,
    /// Render time
    pub render_time: Duration,
    /// Number of components rendered
    pub component_count: usize,
    /// Memory usage (if available)
    pub memory_bytes: Option<usize>,
}

impl Default for FrameMetrics {
    fn default() -> Self {
        Self {
            frame_time: Duration::ZERO,
            layout_time: Duration::ZERO,
            render_time: Duration::ZERO,
            component_count: 0,
            memory_bytes: None,
        }
    }
}

/// Dev mode configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevConfig {
    /// Whether dev tools are enabled
    pub enabled: bool,
    /// Show inspector overlay by default
    pub show_inspector: bool,
    /// Show performance metrics
    pub show_profiler: bool,
    /// Show layout bounds
    pub show_layout_bounds: bool,
    /// Enable hot reload
    pub hot_reload: bool,
    /// Keyboard shortcut modifier (Ctrl, Alt, Meta)
    pub modifier_key: ModifierKey,
}

impl Default for DevConfig {
    fn default() -> Self {
        Self {
            enabled: cfg!(feature = "dev-editor"),
            show_inspector: false,
            show_profiler: false,
            show_layout_bounds: false,
            hot_reload: true,
            modifier_key: ModifierKey::Ctrl,
        }
    }
}

/// Modifier key for shortcuts
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModifierKey {
    Ctrl,
    Alt,
    Shift,
    Meta,
}

impl Default for ModifierKey {
    fn default() -> Self {
        Self::Ctrl
    }
}

/// Check if dev editor is available at compile time
pub const fn is_dev_editor_available() -> bool {
    cfg!(feature = "dev-editor")
}

/// Check if diagnostics export is available at compile time
pub const fn is_diagnostics_available() -> bool {
    cfg!(feature = "diagnostics-export")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_metrics_default() {
        let metrics = FrameMetrics::default();
        assert_eq!(metrics.component_count, 0);
        assert!(metrics.frame_time.is_zero());
    }

    #[test]
    fn test_dev_config_default() {
        let config = DevConfig::default();
        assert!(!config.show_inspector);
        assert!(config.hot_reload);
    }
}
