//! `oxide check` command - run various checks on the project
//!
//! Subcommands:
//! - `oxide check --portability` - Check code for cross-platform compatibility

use anyhow::Result;
use std::path::Path;

/// Run portability check.
pub fn run_portability(target: Option<String>, strict: bool, all_targets: bool) -> Result<()> {
    use oxide_portable::{
        checker::{CheckerConfig, PortabilityChecker, ReportStatus},
        target::targets,
        Target,
    };

    let project_path = Path::new(".");

    // Determine which targets to check
    let check_targets = if all_targets {
        targets::all()
    } else if let Some(target_str) = target {
        vec![Target::from_triple(&target_str)]
    } else {
        // Default: check current target + web + mobile
        vec![
            Target::current(),
            targets::web_wasm32(),
            targets::ios_arm64(),
            targets::android_arm64(),
        ]
    };

    println!();
    println!("  Portability Check");
    println!("  =================");
    println!();

    let config = CheckerConfig {
        strict,
        ..Default::default()
    };

    let mut has_errors = false;
    let mut has_warnings = false;

    for target in &check_targets {
        println!("  Checking target: {} ({})", target.triple(), target.family());

        let checker = PortabilityChecker::for_target(target.clone())
            .with_config(config.clone());

        match checker.check_project(project_path) {
            Ok(report) => {
                match report.status {
                    ReportStatus::Passed => {
                        println!("    Status: PASSED");
                    }
                    ReportStatus::PassedWithWarnings => {
                        println!("    Status: PASSED (with {} warnings)", report.summary.warnings);
                        has_warnings = true;
                    }
                    ReportStatus::Failed => {
                        println!("    Status: FAILED ({} errors)", report.summary.errors);
                        has_errors = true;
                    }
                }

                // Show issues
                for issue in &report.issues {
                    let icon = match issue.severity {
                        oxide_portable::checker::IssueSeverity::Error => "ERROR",
                        oxide_portable::checker::IssueSeverity::Warning => "WARN ",
                        oxide_portable::checker::IssueSeverity::Info => "INFO ",
                    };
                    println!("      [{}] {}", icon, issue.message);

                    if let Some(suggestion) = &issue.suggestion {
                        println!("             Suggestion: {}", suggestion);
                    }
                }
            }
            Err(e) => {
                println!("    Error checking target: {}", e);
                has_errors = true;
            }
        }

        println!();
    }

    // Summary
    println!("  Summary");
    println!("  -------");
    println!("  Targets checked: {}", check_targets.len());

    if has_errors {
        println!("  Result: FAILED - portability issues found");
        if strict {
            anyhow::bail!("Portability check failed");
        }
    } else if has_warnings {
        println!("  Result: PASSED with warnings");
        if strict {
            println!("  (strict mode: treating warnings as errors)");
            anyhow::bail!("Portability check failed (strict mode)");
        }
    } else {
        println!("  Result: PASSED - code is portable to all checked targets");
    }

    println!();

    Ok(())
}

/// Show portability summary for a plugin manifest.
pub fn run_plugin_portability(manifest_path: Option<String>) -> Result<()> {
    use oxide_portable::{
        plugin::PortabilityManifest,
        target::targets,
    };

    let manifest_path = manifest_path
        .map(|p| std::path::PathBuf::from(p))
        .unwrap_or_else(|| std::path::PathBuf::from("oxide-plugin.toml"));

    if !manifest_path.exists() {
        anyhow::bail!("Plugin manifest not found at: {:?}", manifest_path);
    }

    let manifest = PortabilityManifest::load(&manifest_path)?;

    println!();
    println!("  Plugin Portability Summary");
    println!("  ==========================");
    println!();
    println!("  Plugin: {} v{}", manifest.id, manifest.version);
    println!("  Level: {}", manifest.portability.level);
    println!();

    // Show target support
    println!("  Target Support:");
    for target in targets::all() {
        let supported = manifest.supports(&target);
        let icon = if supported { "+" } else { "-" };
        let reason = if !supported {
            manifest.portability.unsupported_reason(&target)
                .map(|r| format!(" ({})", r))
                .unwrap_or_default()
        } else {
            String::new()
        };
        println!("    [{}] {}{}", icon, target.triple(), reason);
    }

    println!();

    // Show API portability if any overrides
    if !manifest.apis.is_empty() {
        println!("  API-specific Portability:");
        for (api_name, api_portability) in &manifest.apis {
            println!("    {}: {}", api_name, api_portability.level);
        }
        println!();
    }

    // Validate manifest
    if let Err(e) = manifest.validate() {
        println!("  WARNING: Manifest has validation issues: {}", e);
        println!();
    }

    Ok(())
}

/// List all known targets and their capabilities.
pub fn run_list_targets() -> Result<()> {
    use oxide_portable::target::targets;

    println!();
    println!("  Available Targets");
    println!("  =================");
    println!();

    for target in targets::all() {
        println!("  {} ({}/{})",
            target.triple(),
            target.platform(),
            target.family()
        );
        println!("    Capabilities:");

        let caps = target.capabilities();
        let cap_list = [
            ("filesystem", caps.filesystem),
            ("native_windows", caps.native_windows),
            ("gpu", caps.gpu),
            ("threads", caps.threads),
            ("network", caps.network),
            ("clipboard", caps.clipboard),
            ("notifications", caps.notifications),
            ("native_menus", caps.native_menus),
            ("system_tray", caps.system_tray),
            ("touch", caps.touch),
            ("biometrics", caps.biometrics),
            ("geolocation", caps.geolocation),
        ];

        let enabled: Vec<&str> = cap_list.iter()
            .filter(|(_, enabled)| *enabled)
            .map(|(name, _)| *name)
            .collect();

        println!("      {}", enabled.join(", "));
        println!();
    }

    Ok(())
}
