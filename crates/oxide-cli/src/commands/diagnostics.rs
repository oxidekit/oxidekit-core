//! `oxide diagnostics` command - diagnostics management

use anyhow::Result;
use oxide_diagnostics::{
    AppInfo, DiagnosticsBundle, DiagnosticsCollector, DiagnosticsConfig,
    DiagnosticEvent, ErrorCode, ErrorDomain, Severity,
    RedactionRules, redact_string,
};
use std::path::Path;

/// Export diagnostics bundle
pub fn run_export(output: Option<&str>) -> Result<()> {
    let output_path = output.unwrap_or("diagnostics.json");

    tracing::info!("Exporting diagnostics bundle to {}", output_path);

    // Create app info from current project
    let app_info = get_app_info()?;

    // Create collector and gather some sample events
    let collector = DiagnosticsCollector::new(app_info.clone(), DiagnosticsConfig::default());

    // Export bundle
    let bundle = DiagnosticsBundle::new(
        app_info,
        collector.get_events(),
        collector.get_logs(),
    );

    // Save to file
    bundle.save_to_file(Path::new(output_path))?;

    println!("Exported diagnostics bundle to {}", output_path);
    println!();
    println!("  Bundle ID: {}", bundle.id);
    println!("  Events: {}", bundle.events.len());
    println!("  Logs: {}", bundle.logs.len());
    println!();
    println!("This file can be shared with developers for debugging.");

    Ok(())
}

/// Preview diagnostics without exporting
pub fn run_preview() -> Result<()> {
    let app_info = get_app_info()?;

    println!("Diagnostics Preview");
    println!("===================");
    println!();
    println!("App Information:");
    println!("  Name: {}", app_info.name);
    println!("  Version: {}", app_info.version);
    println!("  Build ID: {}", app_info.build_id);
    println!("  OxideKit Version: {}", app_info.oxidekit_version);
    println!("  OS: {} ({})", app_info.os, app_info.arch);
    println!("  Build Profile: {:?}", app_info.build_profile);
    println!();

    println!("Redaction Rules:");
    let rules = RedactionRules::default();
    println!("  Redact paths: {}", rules.redact_paths);
    println!("  Redact IPs: {}", rules.redact_ips);
    println!("  Redact emails: {}", rules.redact_emails);
    println!("  Protected fields: {:?}", rules.redact_fields.iter().take(5).collect::<Vec<_>>());
    println!();

    println!("Sample Redaction:");
    let test_input = "User email: user@example.com, IP: 192.168.1.100";
    let redacted = redact_string(test_input, &rules);
    println!("  Input:  {}", test_input);
    println!("  Output: {}", redacted);
    println!();

    println!("No data has been exported. Use 'oxide diagnostics export' to create a bundle.");

    Ok(())
}

/// Test diagnostics system
pub fn run_test() -> Result<()> {
    println!("Running diagnostics system tests...");
    println!();

    // Test 1: Error codes
    println!("1. Testing error codes...");
    let code = ErrorCode::new(ErrorDomain::Ui, 1);
    println!("   UI error code: {}", code);
    assert!(code.to_string().starts_with("OXD-UI-"));
    println!("   PASSED");

    // Test 2: Event creation
    println!("2. Testing event creation...");
    let event = DiagnosticEvent::new(
        ErrorCode::UI_INVALID_PROP,
        Severity::Warning,
        "Test warning event",
    );
    assert!(!event.id.is_nil());
    println!("   Event ID: {}", event.id);
    println!("   PASSED");

    // Test 3: Collector
    println!("3. Testing diagnostics collector...");
    let app_info = get_app_info()?;
    let collector = DiagnosticsCollector::new(app_info, DiagnosticsConfig::default());
    collector.record_event(event);
    let events = collector.get_events();
    assert_eq!(events.len(), 1);
    println!("   Recorded {} event(s)", events.len());
    println!("   PASSED");

    // Test 4: Bundle creation
    println!("4. Testing bundle creation...");
    let bundle = DiagnosticsBundle::new(
        collector.app_info().clone(),
        collector.get_events(),
        collector.get_logs(),
    );
    let json = bundle.to_json()?;
    assert!(json.contains("version"));
    println!("   Bundle size: {} bytes", json.len());
    println!("   PASSED");

    // Test 5: Redaction
    println!("5. Testing redaction...");
    let rules = RedactionRules::default();
    let test = "password=secret123";
    let redacted = redact_string(test, &rules);
    assert!(redacted.contains("[PASSWORD]"));
    println!("   Redacted sensitive data: OK");
    println!("   PASSED");

    println!();
    println!("All diagnostics tests passed!");
    println!();
    println!("Diagnostics system is ready for use.");

    Ok(())
}

/// Verify endpoint connectivity (without sending data)
pub fn run_verify_endpoint(endpoint: &str) -> Result<()> {
    println!("Verifying endpoint connectivity...");
    println!("  Endpoint: {}", endpoint);
    println!();

    // Just check URL validity for now
    if !endpoint.starts_with("https://") {
        println!("  WARNING: Endpoint should use HTTPS for security");
    }

    if endpoint.parse::<url::Url>().is_ok() || endpoint.starts_with("http") {
        println!("  Endpoint URL is valid");
        println!();
        println!("Note: No data was sent. Use auto-reporting in diagnostics config to enable.");
    } else {
        anyhow::bail!("Invalid endpoint URL: {}", endpoint);
    }

    Ok(())
}

/// Get app info from current project
fn get_app_info() -> Result<AppInfo> {
    let manifest_path = Path::new("oxide.toml");

    if manifest_path.exists() {
        let content = std::fs::read_to_string(manifest_path)?;
        let manifest: toml::Value = toml::from_str(&content)?;

        let app = manifest.get("app").ok_or_else(|| anyhow::anyhow!("Missing [app] in oxide.toml"))?;

        let name = app.get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("Unknown");

        let version = app.get("version")
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");

        Ok(AppInfo::from_env(name, version, "dev"))
    } else {
        Ok(AppInfo::from_env("OxideKit App", "0.0.0", "dev"))
    }
}
