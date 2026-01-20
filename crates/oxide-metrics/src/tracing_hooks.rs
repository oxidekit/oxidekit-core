//! Tracing integration and instrumentation hooks

use crate::MetricsRegistry;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{span, Level, Span};

/// Tracing hooks for instrumenting the OxideKit runtime
pub struct TracingHooks {
    /// Active spans for timing
    active_spans: RwLock<HashMap<u64, SpanInfo>>,
    /// Next span ID
    next_id: RwLock<u64>,
    /// Reference to metrics registry
    registry: Option<&'static MetricsRegistry>,
}

/// Information about an active span
#[derive(Debug, Clone)]
struct SpanInfo {
    name: String,
    category: SpanCategory,
    start: Instant,
    #[allow(dead_code)]
    parent_id: Option<u64>,
}

/// Categories of spans for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpanCategory {
    /// Frame timing
    Frame,
    /// Layout computation
    Layout,
    /// Rendering
    Render,
    /// Event handling
    Event,
    /// IO operations
    Io,
    /// Custom/other
    Custom,
}

impl TracingHooks {
    /// Create new tracing hooks
    pub fn new() -> Self {
        Self {
            active_spans: RwLock::new(HashMap::new()),
            next_id: RwLock::new(0),
            registry: None,
        }
    }

    /// Create tracing hooks connected to a metrics registry
    pub fn with_registry(registry: &'static MetricsRegistry) -> Self {
        Self {
            active_spans: RwLock::new(HashMap::new()),
            next_id: RwLock::new(0),
            registry: Some(registry),
        }
    }

    /// Connect to the global registry
    pub fn global() -> Self {
        Self::with_registry(MetricsRegistry::global())
    }

    /// Begin a new span
    pub fn begin_span(&self, name: &str, category: SpanCategory) -> u64 {
        let id = {
            let mut next = self.next_id.write();
            let id = *next;
            *next += 1;
            id
        };

        let info = SpanInfo {
            name: name.to_string(),
            category,
            start: Instant::now(),
            parent_id: None,
        };

        self.active_spans.write().insert(id, info);

        tracing::trace!(span_id = id, name = name, "Span started");
        id
    }

    /// End a span and record its duration
    pub fn end_span(&self, id: u64) -> Option<Duration> {
        let info = self.active_spans.write().remove(&id)?;
        let duration = info.start.elapsed();

        tracing::trace!(
            span_id = id,
            name = %info.name,
            duration_ms = duration.as_secs_f64() * 1000.0,
            "Span ended"
        );

        // Record to metrics if registry is available
        if let Some(registry) = self.registry {
            match info.category {
                SpanCategory::Frame => {
                    registry.frame_metrics().record_frame_time(duration.as_secs_f64() * 1000.0);
                }
                SpanCategory::Layout => {
                    registry.frame_metrics().record_layout_time(duration);
                }
                SpanCategory::Render => {
                    registry.frame_metrics().record_render_time(duration);
                    registry.render_metrics().record_gpu_time(duration);
                }
                SpanCategory::Event => {
                    registry.frame_metrics().record_event_time(duration);
                }
                SpanCategory::Io => {
                    // IO spans need bytes info, which we don't have here
                    // Use the specific IO recording methods instead
                }
                SpanCategory::Custom => {
                    // Custom spans don't map to specific metrics
                }
            }
        }

        Some(duration)
    }

    /// Create a tracing span for frame timing
    pub fn frame_span(&self) -> Span {
        span!(Level::DEBUG, "frame")
    }

    /// Create a tracing span for layout
    pub fn layout_span(&self) -> Span {
        span!(Level::DEBUG, "layout")
    }

    /// Create a tracing span for rendering
    pub fn render_span(&self) -> Span {
        span!(Level::DEBUG, "render")
    }

    /// Create a tracing span for event handling
    pub fn event_span(&self, event_type: &str) -> Span {
        span!(Level::DEBUG, "event", event_type = %event_type)
    }

    /// Create a tracing span for IO
    pub fn io_span(&self, operation: &str) -> Span {
        span!(Level::DEBUG, "io", operation = %operation)
    }

    /// Get count of active spans
    pub fn active_span_count(&self) -> usize {
        self.active_spans.read().len()
    }

    /// Clear all active spans (useful for testing or recovery)
    pub fn clear(&self) {
        self.active_spans.write().clear();
    }
}

impl Default for TracingHooks {
    fn default() -> Self {
        Self::new()
    }
}

/// RAII guard for span timing
pub struct SpanGuard<'a> {
    hooks: &'a TracingHooks,
    id: u64,
}

impl<'a> SpanGuard<'a> {
    /// Begin a new span
    pub fn new(hooks: &'a TracingHooks, name: &str, category: SpanCategory) -> Self {
        let id = hooks.begin_span(name, category);
        Self { hooks, id }
    }

    /// Begin a frame span
    pub fn frame(hooks: &'a TracingHooks) -> Self {
        Self::new(hooks, "frame", SpanCategory::Frame)
    }

    /// Begin a layout span
    pub fn layout(hooks: &'a TracingHooks) -> Self {
        Self::new(hooks, "layout", SpanCategory::Layout)
    }

    /// Begin a render span
    pub fn render(hooks: &'a TracingHooks) -> Self {
        Self::new(hooks, "render", SpanCategory::Render)
    }

    /// Begin an event span
    pub fn event(hooks: &'a TracingHooks, event_type: &str) -> Self {
        Self::new(hooks, event_type, SpanCategory::Event)
    }

    /// Begin an IO span
    pub fn io(hooks: &'a TracingHooks, operation: &str) -> Self {
        Self::new(hooks, operation, SpanCategory::Io)
    }

    /// Get the span ID
    pub fn id(&self) -> u64 {
        self.id
    }
}

impl<'a> Drop for SpanGuard<'a> {
    fn drop(&mut self) {
        self.hooks.end_span(self.id);
    }
}

/// Macro for easy span creation
#[macro_export]
macro_rules! instrument_span {
    ($hooks:expr, frame) => {
        $crate::SpanGuard::frame($hooks)
    };
    ($hooks:expr, layout) => {
        $crate::SpanGuard::layout($hooks)
    };
    ($hooks:expr, render) => {
        $crate::SpanGuard::render($hooks)
    };
    ($hooks:expr, event, $event_type:expr) => {
        $crate::SpanGuard::event($hooks, $event_type)
    };
    ($hooks:expr, io, $operation:expr) => {
        $crate::SpanGuard::io($hooks, $operation)
    };
    ($hooks:expr, $name:expr, $category:expr) => {
        $crate::SpanGuard::new($hooks, $name, $category)
    };
}

/// Convenience functions for instrumenting common operations
pub mod instrument {
    use super::*;

    /// Instrument a frame (call at the start of each frame)
    pub fn frame<F, R>(hooks: &TracingHooks, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = SpanGuard::frame(hooks);
        f()
    }

    /// Instrument layout computation
    pub fn layout<F, R>(hooks: &TracingHooks, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = SpanGuard::layout(hooks);
        f()
    }

    /// Instrument rendering
    pub fn render<F, R>(hooks: &TracingHooks, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = SpanGuard::render(hooks);
        f()
    }

    /// Instrument event handling
    pub fn event<F, R>(hooks: &TracingHooks, event_type: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = SpanGuard::event(hooks, event_type);
        f()
    }

    /// Instrument IO operation
    pub fn io<F, R>(hooks: &TracingHooks, operation: &str, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let _guard = SpanGuard::io(hooks, operation);
        f()
    }
}

// Note: A tracing-subscriber integration layer can be added in the future
// by enabling a `tracing-integration` feature and depending on `tracing-subscriber`.

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread::sleep;

    #[test]
    fn test_span_timing() {
        let hooks = TracingHooks::new();

        let id = hooks.begin_span("test", SpanCategory::Custom);
        sleep(Duration::from_millis(10));
        let duration = hooks.end_span(id);

        assert!(duration.is_some());
        assert!(duration.unwrap() >= Duration::from_millis(10));
    }

    #[test]
    fn test_span_guard() {
        let hooks = TracingHooks::new();

        {
            let _guard = SpanGuard::frame(&hooks);
            sleep(Duration::from_millis(5));
            assert_eq!(hooks.active_span_count(), 1);
        }

        assert_eq!(hooks.active_span_count(), 0);
    }

    #[test]
    fn test_instrument_functions() {
        let hooks = TracingHooks::new();

        let result = instrument::frame(&hooks, || {
            sleep(Duration::from_millis(1));
            42
        });

        assert_eq!(result, 42);
    }

    #[test]
    fn test_instrument_macro() {
        let hooks = TracingHooks::new();

        {
            let _guard = instrument_span!(&hooks, frame);
            sleep(Duration::from_millis(1));
        }

        assert_eq!(hooks.active_span_count(), 0);
    }
}
