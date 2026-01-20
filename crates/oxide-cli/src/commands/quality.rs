//! Quality gate CLI commands
//!
//! Implements `oxide check`, `oxide a11y`, `oxide audit`, `oxide bundle` commands.

use anyhow::Result;
use std::path::PathBuf;

/// Run all quality gates
pub fn run_check(
    path: Option<&str>,
    format: &str,
    output: Option<&str>,
    strict: bool,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Running quality gates on {:?}", project_path);

    // Load configuration
    let mut config = oxide_quality::QualityConfig::from_project(&project_path);
    if strict {
        config.strict = true;
    }

    // Run all checks
    let report = oxide_quality::check_all(&project_path, &config);

    // Output report
    let output_content = match format {
        "json" => report.to_json()?,
        "markdown" | "md" => report.to_markdown(),
        _ => report.to_text(),
    };

    if let Some(output_path) = output {
        std::fs::write(output_path, &output_content)?;
        println!("Report saved to: {}", output_path);
    } else {
        println!("{}", output_content);
    }

    // Exit with error if failed
    if !report.passed() {
        if config.strict && report.summary.total_warnings > 0 {
            anyhow::bail!(
                "Quality gates failed with {} errors and {} warnings",
                report.summary.total_errors,
                report.summary.total_warnings
            );
        } else if report.summary.total_errors > 0 {
            anyhow::bail!(
                "Quality gates failed with {} errors",
                report.summary.total_errors
            );
        }
    }

    println!("\nQuality gates passed!");
    Ok(())
}

/// Enhanced lint command
pub fn run_lint_enhanced(
    path: Option<&str>,
    rules: &[String],
    disable: &[String],
    fix: bool,
    format: &str,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Running enhanced lint on {:?}", project_path);

    let mut config = oxide_quality::LintConfig::default();

    if !rules.is_empty() {
        config.rules = rules.to_vec();
    }

    if !disable.is_empty() {
        config.disable = disable.to_vec();
    }

    let report = oxide_quality::lint::check(&project_path, &config);

    // Output report
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            println!("Lint Report");
            println!("===========");
            println!("Files analyzed: {}", report.files_analyzed);
            println!("Errors: {}, Warnings: {}", report.by_severity.errors, report.by_severity.warnings);
            println!();

            for violation in &report.violations {
                let severity = match violation.severity {
                    oxide_quality::LintSeverity::Error => "ERROR",
                    oxide_quality::LintSeverity::Warning => "WARN",
                    oxide_quality::LintSeverity::Info => "INFO",
                };

                println!(
                    "{}:{}: {} [{}] {}",
                    violation.file,
                    violation.line,
                    severity,
                    violation.rule,
                    violation.message
                );

                if let Some(fix) = &violation.fix {
                    println!("  Fix: {}", fix);
                }
            }
        }
    }

    if !report.passed {
        anyhow::bail!("Lint failed with {} errors", report.by_severity.errors);
    }

    Ok(())
}

/// Run accessibility checks
#[cfg(feature = "a11y")]
pub fn run_a11y_check(
    path: Option<&str>,
    level: &str,
    strict: bool,
    format: &str,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Running accessibility checks on {:?}", project_path);

    let mut config = oxide_quality::A11yConfig::default();

    config.wcag_level = match level.to_uppercase().as_str() {
        "A" => oxide_quality::WcagLevel::A,
        "AAA" => oxide_quality::WcagLevel::AAA,
        _ => oxide_quality::WcagLevel::AA,
    };

    let report = oxide_quality::a11y::check(&project_path, &config);

    // Output report
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            println!("Accessibility Report (WCAG {})", report.wcag_level);
            println!("================================");
            println!("Files analyzed: {}", report.files_analyzed);
            println!("Errors: {}, Warnings: {}", report.errors, report.warnings);
            println!();

            for violation in &report.violations {
                let severity = if violation.is_error { "ERROR" } else { "WARN" };

                println!(
                    "{}:{}: {} [{}] {} (WCAG {})",
                    violation.file,
                    violation.line,
                    severity,
                    violation.rule,
                    violation.message,
                    violation.wcag_criterion
                );

                if let Some(fix) = &violation.fix {
                    println!("  Fix: {}", fix);
                }
            }
        }
    }

    if !report.passed || (strict && report.warnings > 0) {
        anyhow::bail!(
            "Accessibility check failed with {} errors and {} warnings",
            report.errors,
            report.warnings
        );
    }

    println!("\nAccessibility checks passed!");
    Ok(())
}

/// Run security audit
#[cfg(feature = "security")]
pub fn run_security_audit(
    path: Option<&str>,
    format: &str,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Running security audit on {:?}", project_path);

    let config = oxide_quality::SecurityConfig::default();
    let report = oxide_quality::security::check(&project_path, &config);

    // Output report
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            println!("Security Audit Report");
            println!("====================");
            println!(
                "Vulnerabilities: {} critical, {} high, {} medium, {} low",
                report.critical, report.high, report.medium, report.low
            );
            println!("Unsafe code usages: {}", report.unsafe_usage.len());
            println!("Forbidden API usages: {}", report.forbidden_apis.len());
            println!();

            if !report.vulnerabilities.is_empty() {
                println!("Vulnerabilities:");
                for vuln in &report.vulnerabilities {
                    println!(
                        "  {} [{}]: {} - {} ({})",
                        vuln.severity, vuln.id, vuln.title, vuln.package, vuln.version
                    );
                }
                println!();
            }

            if !report.unsafe_usage.is_empty() {
                println!("Unsafe code usage:");
                for usage in &report.unsafe_usage {
                    let status = if usage.allowed { "allowed" } else { "review" };
                    println!("  {}:{}: {} ({})", usage.file, usage.line, usage.snippet, status);
                }
                println!();
            }

            if !report.forbidden_apis.is_empty() {
                println!("Forbidden API usage:");
                for api in &report.forbidden_apis {
                    println!("  {}:{}: {}", api.file, api.line, api.api);
                }
            }
        }
    }

    if !report.passed {
        anyhow::bail!("Security audit failed");
    }

    println!("\nSecurity audit passed!");
    Ok(())
}

/// Run bundle analysis
#[cfg(feature = "bundle")]
pub fn run_bundle_check(
    path: Option<&str>,
    format: &str,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    tracing::info!("Analyzing bundle size for {:?}", project_path);

    let config = oxide_quality::BundleConfig::default();
    let report = oxide_quality::bundle::check(&project_path, &config);

    // Output report
    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&report)?;
            println!("{}", json);
        }
        _ => {
            println!("Bundle Size Report");
            println!("=================");
            println!(
                "Total size: {} / {}",
                report.total_size_formatted, report.size_limit_formatted
            );
            println!("Files analyzed: {}", report.files.len());

            if let Some(change) = &report.size_change {
                let direction = if change.change_bytes > 0 { "+" } else { "" };
                println!(
                    "Size change: {}{} bytes ({:+.1}%)",
                    direction, change.change_bytes, change.change_percent
                );
            }

            println!();

            if !report.largest_files.is_empty() {
                println!("Largest files:");
                for file in &report.largest_files {
                    println!(
                        "  {} - {} ({:.1}%)",
                        file.path, file.size_formatted, file.percentage
                    );
                }
            }

            if !report.warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &report.warnings {
                    println!("  - {}", warning);
                }
            }
        }
    }

    if !report.passed {
        anyhow::bail!("Bundle size check failed");
    }

    println!("\nBundle size check passed!");
    Ok(())
}

/// Generate CI configuration
#[cfg(feature = "ci")]
pub fn run_generate_ci(
    path: Option<&str>,
    github: bool,
    gitlab: bool,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let mut config = oxide_quality::CiConfig::default();
    config.github_actions = github;
    config.gitlab_ci = gitlab;

    let result = oxide_quality::ci::generate_ci_config(&project_path, &config);
    let created = result.save(&project_path)?;

    for file in created {
        println!("Created: {}", file);
    }

    println!("\nCI configuration generated!");
    Ok(())
}

/// Install pre-commit hooks
#[cfg(feature = "hooks")]
pub fn run_install_hooks(
    path: Option<&str>,
    lint: bool,
    format: bool,
    a11y: bool,
    test: bool,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let config = oxide_quality::HookConfig {
        lint,
        format,
        a11y,
        test,
        ..Default::default()
    };

    oxide_quality::hooks::install_hook(&project_path, &config)?;

    println!("Pre-commit hook installed!");
    println!("Enabled checks:");
    if lint { println!("  - Lint"); }
    if format { println!("  - Format"); }
    if a11y { println!("  - Accessibility"); }
    if test { println!("  - Tests"); }

    println!("\nTo bypass: SKIP_HOOKS=1 git commit -m \"message\"");
    Ok(())
}

/// Uninstall pre-commit hooks
#[cfg(feature = "hooks")]
pub fn run_uninstall_hooks(path: Option<&str>) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    oxide_quality::hooks::uninstall_hook(&project_path)?;

    println!("Pre-commit hook uninstalled!");
    Ok(())
}

/// Initialize quality configuration
pub fn run_init_quality(
    path: Option<&str>,
    preset: &str,
) -> Result<()> {
    let project_path = path
        .map(PathBuf::from)
        .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));

    let config = match preset {
        "strict" => {
            let mut c = oxide_quality::QualityConfig::default();
            c.strict = true;
            c.a11y.wcag_level = oxide_quality::WcagLevel::AAA;
            c.lint.max_warnings = Some(0);
            c
        }
        "minimal" => {
            let mut c = oxide_quality::QualityConfig::default();
            c.a11y.enabled = false;
            c.perf.enabled = false;
            c.security.audit_deps = false;
            c
        }
        _ => oxide_quality::QualityConfig::default(),
    };

    let config_path = project_path.join("oxide-quality.toml");
    config.save(&config_path)?;

    println!("Created: oxide-quality.toml");
    println!("Preset: {}", preset);
    Ok(())
}
