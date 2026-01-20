//! `oxide legal` commands - license scanning, SBOM generation, compliance
//!
//! Commands:
//! - `oxide licenses` - Scan and display dependency licenses
//! - `oxide sbom` - Generate Software Bill of Materials
//! - `oxide compliance` - Check license compliance
//! - `oxide cla` - Check CLA status

use anyhow::Result;
use std::path::Path;

#[cfg(feature = "legal")]
use oxide_legal::{
    LicenseScanner, ScanResult, License, LicenseCategory,
    LicensePolicy, ComplianceReport, ReportFormat, ComplianceStatus,
};

#[cfg(all(feature = "legal", feature = "legal-sbom"))]
use oxide_legal::{SbomGenerator, SbomFormat};

#[cfg(all(feature = "legal", feature = "legal-cla"))]
use oxide_legal::{ClaChecker, ClaStatus, AgreementType, CommitInfo};

#[cfg(all(feature = "legal", feature = "legal-export"))]
use oxide_legal::ExportControl;

/// Scan and display dependency licenses
#[cfg(feature = "legal")]
pub fn run_licenses(
    format: Option<&str>,
    check_policy: Option<&str>,
    output: Option<&str>,
) -> Result<()> {
    let manifest_path = find_manifest()?;

    tracing::info!("Scanning licenses in {}", manifest_path.display());

    let mut scanner = LicenseScanner::new(&manifest_path);
    let scan = scanner.scan()?;

    // Apply policy check if requested
    if let Some(policy_name) = check_policy {
        let policy = match policy_name {
            "permissive" => LicensePolicy::permissive(),
            "copyleft" => LicensePolicy::copyleft_friendly(),
            "commercial" => LicensePolicy::commercial(),
            path if Path::new(path).exists() => LicensePolicy::from_file(path)?,
            _ => {
                anyhow::bail!("Unknown policy: {}. Use 'permissive', 'copyleft', 'commercial', or a file path.", policy_name);
            }
        };

        let validation = policy.validate(&scan);

        if !validation.passed {
            println!("License Policy Check: FAILED");
            println!();
            println!("Violations:");
            for v in &validation.violations {
                println!("  [!] {} v{}: {} ({})", v.package, v.version, v.license, v.message);
            }
            println!();

            if let Some(out) = output {
                let report = ComplianceReport::generate(scan, &policy);
                let format = match format.unwrap_or("text") {
                    "json" => ReportFormat::Json,
                    "html" => ReportFormat::Html,
                    "csv" => ReportFormat::Csv,
                    "markdown" | "md" => ReportFormat::Markdown,
                    _ => ReportFormat::Text,
                };
                report.export_to_file(out, format)?;
                println!("Report saved to: {}", out);
            }

            anyhow::bail!("License policy violations found");
        }

        println!("License Policy Check: PASSED");
        if !validation.warnings.is_empty() {
            println!("\nWarnings:");
            for w in &validation.warnings {
                println!("  [~] {} v{}: {}", w.package, w.version, w.message);
            }
        }
        println!();
    }

    // Output format
    let output_format = format.unwrap_or("table");

    match output_format {
        "json" => {
            let json = scan.to_json()?;
            if let Some(out) = output {
                std::fs::write(out, &json)?;
                println!("License data saved to: {}", out);
            } else {
                println!("{}", json);
            }
        }
        "csv" => {
            print_licenses_csv(&scan);
        }
        _ => {
            print_licenses_table(&scan);
        }
    }

    Ok(())
}

/// Fallback when legal feature is not enabled
#[cfg(not(feature = "legal"))]
pub fn run_licenses(
    _format: Option<&str>,
    _check_policy: Option<&str>,
    _output: Option<&str>,
) -> Result<()> {
    println!("License scanning requires the 'legal' feature.");
    println!("Recompile oxide-cli with: cargo build --features legal");
    Ok(())
}

/// Generate SBOM (Software Bill of Materials)
#[cfg(all(feature = "legal", feature = "legal-sbom"))]
pub fn run_sbom(format: Option<&str>, output: Option<&str>) -> Result<()> {
    let manifest_path = find_manifest()?;

    let format_type = match format.unwrap_or("spdx-json") {
        "spdx" | "spdx-json" => SbomFormat::SpdxJson,
        "spdx-tv" | "spdx-tag-value" => SbomFormat::SpdxTagValue,
        "cdx" | "cyclonedx" | "cyclonedx-json" => SbomFormat::CycloneDxJson,
        "cdx-xml" | "cyclonedx-xml" => SbomFormat::CycloneDxXml,
        "oxidekit" | "native" => SbomFormat::OxideKit,
        other => {
            anyhow::bail!(
                "Unknown SBOM format: {}. Use 'spdx-json', 'spdx-tv', 'cyclonedx', 'cyclonedx-xml', or 'oxidekit'.",
                other
            );
        }
    };

    tracing::info!("Generating SBOM from {}", manifest_path.display());

    let mut generator = SbomGenerator::new(&manifest_path)
        .with_tool("oxide-cli", env!("CARGO_PKG_VERSION"))
        .with_namespace("https://oxidekit.com/sbom");

    let sbom_content = generator.generate(format_type)?;

    if let Some(out) = output {
        std::fs::write(out, &sbom_content)?;
        println!("SBOM saved to: {}", out);
    } else {
        let default_name = format!("sbom.{}", format_type.extension());
        std::fs::write(&default_name, &sbom_content)?;
        println!("SBOM saved to: {}", default_name);
    }

    println!();
    println!("SBOM Generation Complete");
    println!("  Format: {:?}", format_type);
    println!();

    Ok(())
}

/// Fallback when SBOM feature is not enabled
#[cfg(not(all(feature = "legal", feature = "legal-sbom")))]
pub fn run_sbom(_format: Option<&str>, _output: Option<&str>) -> Result<()> {
    println!("SBOM generation requires the 'legal' and 'legal-sbom' features.");
    println!("Recompile oxide-cli with: cargo build --features \"legal legal-sbom\"");
    Ok(())
}

/// Generate compliance report
#[cfg(feature = "legal")]
pub fn run_compliance(
    policy: Option<&str>,
    format: Option<&str>,
    output: Option<&str>,
) -> Result<()> {
    let manifest_path = find_manifest()?;

    let policy = match policy.unwrap_or("permissive") {
        "permissive" => LicensePolicy::permissive(),
        "copyleft" => LicensePolicy::copyleft_friendly(),
        "commercial" => LicensePolicy::commercial(),
        path if Path::new(path).exists() => LicensePolicy::from_file(path)?,
        name => {
            anyhow::bail!("Unknown policy: {}. Use 'permissive', 'copyleft', 'commercial', or a file path.", name);
        }
    };

    let mut scanner = LicenseScanner::new(&manifest_path);
    let scan = scanner.scan()?;
    let report = ComplianceReport::generate(scan, &policy);

    let format_type = match format.unwrap_or("text") {
        "json" => ReportFormat::Json,
        "html" => ReportFormat::Html,
        "csv" => ReportFormat::Csv,
        "markdown" | "md" => ReportFormat::Markdown,
        _ => ReportFormat::Text,
    };

    if let Some(out) = output {
        report.export_to_file(out, format_type)?;
        println!("Compliance report saved to: {}", out);
    } else {
        let content = report.export(format_type)?;
        println!("{}", content);
    }

    // Return error if non-compliant
    match report.status {
        ComplianceStatus::NonCompliant => {
            anyhow::bail!("Project is not compliant with license policy");
        }
        ComplianceStatus::RequiresReview => {
            println!("\nNote: Some dependencies require manual review.");
        }
        _ => {}
    }

    Ok(())
}

/// Fallback when legal feature is not enabled
#[cfg(not(feature = "legal"))]
pub fn run_compliance(
    _policy: Option<&str>,
    _format: Option<&str>,
    _output: Option<&str>,
) -> Result<()> {
    println!("Compliance checking requires the 'legal' feature.");
    println!("Recompile oxide-cli with: cargo build --features legal");
    Ok(())
}

/// Check CLA status
#[cfg(all(feature = "legal", feature = "legal-cla"))]
pub fn run_cla_check(email: &str, database: Option<&str>) -> Result<()> {
    let mut checker = ClaChecker::new(AgreementType::IndividualCla)
        .with_dco_fallback(true);

    if let Some(db_path) = database {
        if Path::new(db_path).exists() {
            checker.load_from_file(db_path)?;
        }
    }

    let result = checker.check(email, None);

    println!("CLA Status Check");
    println!("================");
    println!();
    println!("  Email: {}", result.email);
    println!("  Status: {:?}", result.status);

    if let Some(agreement_type) = result.agreement_type {
        println!("  Agreement Type: {}", agreement_type.display_name());
    }

    if let Some(signed_at) = result.signed_at {
        println!("  Signed At: {}", signed_at.format("%Y-%m-%d"));
    }

    println!();
    println!("  Message: {}", result.message);

    if !result.status.allows_contribution() {
        println!();
        println!("To contribute, please sign the CLA:");
        println!("{}", checker.signing_instructions());
    }

    Ok(())
}

/// Fallback when CLA feature is not enabled
#[cfg(not(all(feature = "legal", feature = "legal-cla")))]
pub fn run_cla_check(_email: &str, _database: Option<&str>) -> Result<()> {
    println!("CLA checking requires the 'legal' and 'legal-cla' features.");
    println!("Recompile oxide-cli with: cargo build --features \"legal legal-cla\"");
    Ok(())
}

/// Generate NOTICE file (third-party attributions)
#[cfg(feature = "legal")]
pub fn run_notice(output: Option<&str>) -> Result<()> {
    let manifest_path = find_manifest()?;

    let mut scanner = LicenseScanner::new(&manifest_path);
    let scan = scanner.scan()?;

    let policy = LicensePolicy::permissive();
    let report = ComplianceReport::generate(scan, &policy);
    let notice = report.generate_notice();

    let output_path = output.unwrap_or("NOTICE");
    std::fs::write(output_path, &notice)?;

    println!("NOTICE file generated: {}", output_path);
    println!();
    println!("  Attributions: {}", report.attributions.len());
    println!();
    println!("Include this file with your distribution to comply with license requirements.");

    Ok(())
}

/// Fallback when legal feature is not enabled
#[cfg(not(feature = "legal"))]
pub fn run_notice(_output: Option<&str>) -> Result<()> {
    println!("NOTICE generation requires the 'legal' feature.");
    println!("Recompile oxide-cli with: cargo build --features legal");
    Ok(())
}

/// Check export control status
#[cfg(all(feature = "legal", feature = "legal-export"))]
pub fn run_export_control(country: Option<&str>) -> Result<()> {
    let manifest_path = find_manifest()?;

    let mut scanner = LicenseScanner::new(&manifest_path);
    let scan = scanner.scan()?;

    let control = ExportControl::new().set_public(true);

    if let Some(country_code) = country {
        let check = control.check_country(country_code);
        println!("Export Control Check: {}", country_code);
        println!("======================");
        println!();
        println!("  Allowed: {}", if check.is_allowed { "Yes" } else { "No" });

        if !check.restrictions.is_empty() {
            println!("\nRestrictions:");
            for r in &check.restrictions {
                println!("  - {}", r);
            }
        }

        if !check.required_actions.is_empty() {
            println!("\nRequired Actions:");
            for a in &check.required_actions {
                println!("  - {}", a);
            }
        }
    } else {
        let analysis = control.analyze(&scan);

        println!("Export Control Analysis");
        println!("=======================");
        println!();
        println!("{}", analysis.summary());

        if !analysis.encryption_dependencies.is_empty() {
            println!("\nEncryption Dependencies:");
            for dep in &analysis.encryption_dependencies {
                println!("  - {} v{}", dep.name, dep.version);
            }
        }

        if !analysis.recommendations.is_empty() {
            println!("\nRecommendations:");
            for r in &analysis.recommendations {
                println!("  - {}", r);
            }
        }

        if !analysis.can_distribute_freely() {
            println!("\nWARNING: Distribution may require export license. Consult legal counsel.");
        }
    }

    Ok(())
}

/// Fallback when export control feature is not enabled
#[cfg(not(all(feature = "legal", feature = "legal-export")))]
pub fn run_export_control(_country: Option<&str>) -> Result<()> {
    println!("Export control checking requires the 'legal' and 'legal-export' features.");
    println!("Recompile oxide-cli with: cargo build --features \"legal legal-export\"");
    Ok(())
}

/// Create a license policy file
#[cfg(feature = "legal")]
pub fn run_create_policy(policy_type: &str, output: &str) -> Result<()> {
    let policy = match policy_type {
        "permissive" => LicensePolicy::permissive(),
        "copyleft" => LicensePolicy::copyleft_friendly(),
        "commercial" => LicensePolicy::commercial(),
        _ => {
            anyhow::bail!("Unknown policy type: {}. Use 'permissive', 'copyleft', or 'commercial'.", policy_type);
        }
    };

    policy.to_file(output)?;

    println!("License policy created: {}", output);
    println!();
    println!("  Policy Type: {}", policy.name);
    println!("  Description: {}", policy.description);
    println!();
    println!("You can customize this file and use it with:");
    println!("  oxide licenses --policy {}", output);
    println!("  oxide compliance --policy {}", output);

    Ok(())
}

/// Fallback when legal feature is not enabled
#[cfg(not(feature = "legal"))]
pub fn run_create_policy(_policy_type: &str, _output: &str) -> Result<()> {
    println!("Policy creation requires the 'legal' feature.");
    println!("Recompile oxide-cli with: cargo build --features legal");
    Ok(())
}

// Helper functions

/// Find the Cargo.toml manifest
fn find_manifest() -> Result<std::path::PathBuf> {
    let cwd = std::env::current_dir()?;

    // Check current directory
    let manifest = cwd.join("Cargo.toml");
    if manifest.exists() {
        return Ok(manifest);
    }

    // Check parent directories
    let mut current = cwd.as_path();
    while let Some(parent) = current.parent() {
        let manifest = parent.join("Cargo.toml");
        if manifest.exists() {
            return Ok(manifest);
        }
        current = parent;
    }

    anyhow::bail!("Could not find Cargo.toml in current directory or parents")
}

/// Print licenses in table format
#[cfg(feature = "legal")]
fn print_licenses_table(scan: &ScanResult) {
    println!("License Scan Results");
    println!("====================");
    println!();
    println!("Project: {} v{}", scan.project_name, scan.project_version);
    println!("License: {}", scan.project_license.as_deref().unwrap_or("Unknown"));
    println!();

    println!("Summary:");
    println!("  Total Dependencies: {}", scan.summary.total_dependencies);
    println!("  Permissive: {}", scan.summary.permissive_count);
    println!("  Weak Copyleft: {}", scan.summary.weak_copyleft_count);
    println!("  Strong Copyleft: {}", scan.summary.copyleft_count);
    println!("  Unknown: {}", scan.summary.unknown_count);
    println!();

    if scan.summary.is_permissive_only() {
        println!("Status: All dependencies use permissive licenses");
    } else if scan.summary.has_copyleft() {
        println!("Status: Contains copyleft dependencies - review requirements");
    } else if scan.summary.has_unknown() {
        println!("Status: Contains unknown licenses - manual review required");
    }
    println!();

    // Group by category
    println!("Dependencies by Category:");
    println!("--------------------------");

    let categories = [
        (LicenseCategory::PublicDomain, "Public Domain"),
        (LicenseCategory::Permissive, "Permissive"),
        (LicenseCategory::WeakCopyleft, "Weak Copyleft"),
        (LicenseCategory::StrongCopyleft, "Strong Copyleft"),
        (LicenseCategory::NetworkCopyleft, "Network Copyleft"),
        (LicenseCategory::Unknown, "Unknown"),
    ];

    for (category, label) in &categories {
        let deps: Vec<_> = scan.dependencies.iter()
            .filter(|d| &d.license.category == category)
            .collect();

        if !deps.is_empty() {
            println!("\n{}:", label);
            for dep in deps {
                println!("  {} v{} ({})", dep.name, dep.version, dep.license.spdx_id);
            }
        }
    }
}

/// Print licenses in CSV format
#[cfg(feature = "legal")]
fn print_licenses_csv(scan: &ScanResult) {
    println!("name,version,license,category,osi_approved");
    for dep in &scan.dependencies {
        println!(
            "{},{},{},{:?},{}",
            dep.name,
            dep.version,
            dep.license.spdx_id,
            dep.license.category,
            dep.license.osi_approved
        );
    }
}
