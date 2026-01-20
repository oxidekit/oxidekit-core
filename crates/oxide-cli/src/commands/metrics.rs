//! `oxide metrics` command - metrics and observability management

use anyhow::Result;
use oxide_metrics::{
    MetricsRegistry, MetricsConfig, MetricsSnapshot, MetricsCollector,
    HealthReport, HealthStatus,
};
use std::path::Path;

/// Show current metrics status
pub fn run_status() -> Result<()> {
    println!("OxideKit Metrics Status");
    println!("=======================");
    println!();

    let registry = MetricsRegistry::global();
    let config = registry.config();

    println!("Configuration:");
    println!("  Enabled: {}", config.enabled);
    println!("  Metric Prefix: {}", config.metric_prefix);
    println!("  Frame Window Size: {}", config.frame_window_size);
    println!("  Memory Sample Interval: {}ms", config.memory_sample_interval_ms);
    println!();

    let snapshot = MetricsSnapshot::from_registry(registry);

    println!("Frame Metrics:");
    println!("  FPS: {:.1}", snapshot.frame.fps);
    println!("  Frame Time: {:.2}ms (avg)", snapshot.frame.avg_frame_time_ms);
    println!("  Frame Count: {}", snapshot.frame.frame_count);
    println!("  Jank Count: {} ({:.1}%)", snapshot.frame.jank_count, snapshot.frame.jank_percent);
    println!("  Smooth: {}", if snapshot.frame.is_smooth { "Yes" } else { "No" });
    println!();

    println!("Memory Metrics:");
    println!("  Heap: {}", snapshot.memory.heap_formatted);
    println!("  Peak: {} bytes", snapshot.memory.peak_bytes);
    println!("  Allocations: {}", snapshot.memory.allocations);
    println!("  GPU Memory: {} bytes", snapshot.memory.gpu_bytes);
    println!("  Potential Leak: {}", if snapshot.memory.potential_leak { "WARNING" } else { "No" });
    println!();

    println!("Render Metrics:");
    println!("  Draw Calls: {}", snapshot.render.draw_calls);
    println!("  Triangles: {}", snapshot.render.triangles);
    println!("  GPU Time: {:.2}ms (avg)", snapshot.render.avg_gpu_time_ms);
    println!("  Shader Compiles: {}", snapshot.render.shader_compiles);
    println!("  Total Frames: {}", snapshot.render.total_frames);
    println!();

    println!("IO Metrics:");
    println!("  Bytes Read: {}", snapshot.io.bytes_read);
    println!("  Bytes Written: {}", snapshot.io.bytes_written);
    println!("  Read Ops: {}", snapshot.io.read_ops);
    println!("  Write Ops: {}", snapshot.io.write_ops);
    println!("  Network Requests: {}", snapshot.io.network_requests);
    println!("  Success Rate: {:.1}%", snapshot.io.success_rate * 100.0);
    println!();

    Ok(())
}

/// Export metrics snapshot
pub fn run_export(output: Option<&str>, format: &str) -> Result<()> {
    let output_path = output.unwrap_or("metrics.json");
    tracing::info!("Exporting metrics to {}", output_path);

    let snapshot = MetricsSnapshot::from_global();

    let content = match format {
        "json" => snapshot.to_json(),
        "json-compact" => snapshot.to_json_compact(),
        _ => {
            anyhow::bail!("Unknown format: {}. Supported: json, json-compact", format);
        }
    };

    std::fs::write(output_path, &content)?;

    println!("Exported metrics snapshot to {}", output_path);
    println!();
    println!("  Format: {}", format);
    println!("  Size: {} bytes", content.len());
    println!("  FPS: {:.1}", snapshot.frame.fps);
    println!("  Memory: {}", snapshot.memory.heap_formatted);
    println!();

    Ok(())
}

/// Run health check
pub fn run_health() -> Result<()> {
    let collector = MetricsCollector::new();
    collector.collect();

    let report = collector.health_report();

    println!("OxideKit Health Report");
    println!("======================");
    println!();

    let status_str = |status: &HealthStatus| match status {
        HealthStatus::Good => "GOOD",
        HealthStatus::Warning => "WARNING",
        HealthStatus::Critical => "CRITICAL",
        HealthStatus::Unknown => "UNKNOWN",
    };

    let status_color = |status: &HealthStatus| match status {
        HealthStatus::Good => "\x1b[32m", // green
        HealthStatus::Warning => "\x1b[33m", // yellow
        HealthStatus::Critical => "\x1b[31m", // red
        HealthStatus::Unknown => "\x1b[90m", // gray
    };

    let reset = "\x1b[0m";

    println!(
        "Overall: {}{}{}",
        status_color(&report.overall),
        status_str(&report.overall),
        reset
    );
    println!();

    println!("Checks:");
    println!(
        "  FPS: {}{}{} - {}",
        status_color(&report.fps.status),
        status_str(&report.fps.status),
        reset,
        report.fps.message
    );
    println!(
        "  Memory: {}{}{} - {}",
        status_color(&report.memory.status),
        status_str(&report.memory.status),
        reset,
        report.memory.message
    );
    println!(
        "  IO: {}{}{} - {}",
        status_color(&report.io.status),
        status_str(&report.io.status),
        reset,
        report.io.message
    );
    println!();

    if report.is_healthy() {
        println!("Application is healthy.");
    } else {
        println!("Issues detected. Check the warnings above.");
    }

    Ok(())
}

/// Test metrics system
pub fn run_test() -> Result<()> {
    println!("Running metrics system tests...");
    println!();

    // Test 1: Registry
    println!("1. Testing metrics registry...");
    let registry = MetricsRegistry::global();
    assert!(registry.is_enabled());
    println!("   Registry enabled: OK");
    println!("   PASSED");

    // Test 2: Custom metrics
    println!("2. Testing custom metrics...");
    registry.register_counter("test_counter", "Test counter");
    registry.increment_counter("test_counter");
    registry.add_counter("test_counter", 5);
    let value = registry.get_metric("test_counter");
    assert!(value.is_some());
    println!("   Custom counter: OK");
    println!("   PASSED");

    // Test 3: Frame metrics
    println!("3. Testing frame metrics...");
    let frame_metrics = registry.frame_metrics();
    frame_metrics.record_frame_time(16.67);
    let fps = frame_metrics.fps();
    assert!(fps > 0.0);
    println!("   Frame time recording: OK (FPS: {:.1})", fps);
    println!("   PASSED");

    // Test 4: Snapshot
    println!("4. Testing metrics snapshot...");
    let snapshot = MetricsSnapshot::from_registry(registry);
    let json = snapshot.to_json();
    assert!(json.contains("fps"));
    assert!(json.contains("memory"));
    println!("   Snapshot generation: OK ({} bytes)", json.len());
    println!("   PASSED");

    // Test 5: Collector
    println!("5. Testing metrics collector...");
    let collector = MetricsCollector::new();
    collector.collect();
    let latest = collector.latest();
    assert!(latest.is_some());
    println!("   Collection: OK");
    println!("   PASSED");

    // Test 6: Health report
    println!("6. Testing health report...");
    let report = collector.health_report();
    let _ = report.to_json();
    println!("   Health report generation: OK");
    println!("   PASSED");

    println!();
    println!("All metrics tests passed!");
    println!();
    println!("Metrics system is ready for use.");

    // Clean up test counter
    registry.reset();

    Ok(())
}

/// Show Prometheus endpoint info
pub fn run_prometheus_info(port: u16) -> Result<()> {
    println!("Prometheus Exporter Configuration");
    println!("==================================");
    println!();

    #[cfg(feature = "prometheus")]
    {
        use oxide_metrics::PrometheusExporter;

        let exporter = PrometheusExporter::new();
        println!("Status: Available");
        println!("  Default Port: {}", port);
        println!("  Metrics Path: /metrics");
        println!("  Health Path: /health");
        println!();
        println!("Sample Output:");
        println!("---");
        let output = exporter.render();
        // Show first 20 lines
        for line in output.lines().take(20) {
            println!("{}", line);
        }
        if output.lines().count() > 20 {
            println!("... ({} more lines)", output.lines().count() - 20);
        }
        println!("---");
        println!();
        println!("To start the server, use the Prometheus exporter in your application:");
        println!("  let exporter = PrometheusExporter::new();");
        println!("  exporter.start_server().await;");
    }

    #[cfg(not(feature = "prometheus"))]
    {
        println!("Status: Not Available");
        println!();
        println!("The Prometheus exporter is not enabled.");
        println!("Enable it by adding the 'prometheus' feature to oxide-metrics:");
        println!();
        println!("  oxide-metrics = {{ version = \"0.1\", features = [\"prometheus\"] }}");
    }

    Ok(())
}

/// Show configuration help
pub fn run_config() -> Result<()> {
    println!("OxideKit Metrics Configuration");
    println!("==============================");
    println!();

    println!("Configuration can be set in your oxide.toml:");
    println!();
    println!("[metrics]");
    println!("enabled = true");
    println!("frame_window_size = 120     # Frames to average for FPS");
    println!("memory_sample_interval_ms = 1000");
    println!("detailed_io = false");
    println!("render_instrumentation = false");
    println!("metric_prefix = \"oxide\"");
    println!();
    println!("[metrics.labels]");
    println!("app = \"my-app\"");
    println!("env = \"production\"");
    println!();

    println!("Or programmatically:");
    println!();
    println!("  use oxide_metrics::{{MetricsRegistry, MetricsConfig}};");
    println!();
    println!("  let config = MetricsConfig {{");
    println!("      enabled: true,");
    println!("      frame_window_size: 120,");
    println!("      ..Default::default()");
    println!("  }};");
    println!();
    println!("  MetricsRegistry::init_global(config).unwrap();");
    println!();

    println!("Available Features:");
    println!("  - prometheus: Prometheus metrics exporter");
    println!("  - structured-logs: JSON/OpenTelemetry log exporter");
    println!("  - devtools-panel: Devtools metrics UI components");
    println!("  - perf-counters: Detailed performance counters");
    println!("  - full: Enable all features");
    println!();

    Ok(())
}

/// Reset all metrics
pub fn run_reset() -> Result<()> {
    let registry = MetricsRegistry::global();
    registry.reset();

    println!("All metrics have been reset.");

    Ok(())
}
