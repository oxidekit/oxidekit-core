//! Bundle size analysis
//!
//! Analyzes bundle size and tracks changes over time.

use crate::{BundleConfig, ErrorCode};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Bundle analysis report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleReport {
    /// Total bundle size in bytes
    pub total_size_bytes: u64,
    /// Total size formatted
    pub total_size_formatted: String,
    /// Size limit in bytes
    pub size_limit_bytes: u64,
    /// Size limit formatted
    pub size_limit_formatted: String,
    /// Whether size limit is exceeded
    pub exceeded_size: bool,
    /// File breakdown
    pub files: Vec<BundleFile>,
    /// Largest files
    pub largest_files: Vec<BundleFile>,
    /// Size change from baseline (if available)
    pub size_change: Option<SizeChange>,
    /// Warnings
    pub warnings: Vec<String>,
    /// Whether bundle check passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl BundleReport {
    pub fn new(config: &BundleConfig) -> Self {
        Self {
            total_size_bytes: 0,
            total_size_formatted: "0 B".to_string(),
            size_limit_bytes: config.max_size_bytes,
            size_limit_formatted: format_size(config.max_size_bytes),
            exceeded_size: false,
            files: Vec::new(),
            largest_files: Vec::new(),
            size_change: None,
            warnings: Vec::new(),
            passed: true,
            duration_ms: 0,
        }
    }
}

/// Bundle file entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleFile {
    /// File path relative to project
    pub path: String,
    /// File size in bytes
    pub size_bytes: u64,
    /// Formatted size
    pub size_formatted: String,
    /// Percentage of total
    pub percentage: f64,
    /// File type
    pub file_type: String,
    /// Whether file exceeds individual size limit
    pub exceeds_limit: bool,
}

/// Size change from baseline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeChange {
    /// Previous size in bytes
    pub previous_bytes: u64,
    /// Current size in bytes
    pub current_bytes: u64,
    /// Absolute change in bytes
    pub change_bytes: i64,
    /// Percentage change
    pub change_percent: f64,
    /// Whether change exceeds warning threshold
    pub exceeds_warning: bool,
    /// Whether change exceeds failure threshold
    pub exceeds_failure: bool,
}

/// Bundle violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleViolation {
    /// Error code
    pub code: ErrorCode,
    /// File path
    pub file: String,
    /// Message
    pub message: String,
    /// Expected value
    pub expected: String,
    /// Actual value
    pub actual: String,
}

/// Run bundle analysis
pub fn check(project_path: &Path, config: &BundleConfig) -> BundleReport {
    let start = std::time::Instant::now();
    let mut report = BundleReport::new(config);

    if !config.enabled {
        tracing::debug!("Bundle checks disabled");
        return report;
    }

    tracing::info!("Analyzing bundle size");

    // Find build output directory
    let build_paths = [
        project_path.join("target/release"),
        project_path.join("dist"),
        project_path.join("build"),
        project_path.join("out"),
    ];

    let mut build_path = None;
    for path in &build_paths {
        if path.exists() {
            build_path = Some(path.clone());
            break;
        }
    }

    // If no build directory found, analyze source
    let default_path = project_path.to_path_buf();
    let analysis_path = build_path.as_ref().unwrap_or(&default_path);

    // Analyze files
    let files = analyze_directory(analysis_path, config);
    let total_size: u64 = files.iter().map(|f| f.size_bytes).sum();

    report.total_size_bytes = total_size;
    report.total_size_formatted = format_size(total_size);

    // Check size limit
    if total_size > config.max_size_bytes {
        report.exceeded_size = true;
        report.passed = false;
        report.warnings.push(format!(
            "Bundle size ({}) exceeds limit ({})",
            report.total_size_formatted,
            report.size_limit_formatted
        ));
    }

    // Calculate percentages and find largest files
    let mut files_with_percentage: Vec<BundleFile> = files
        .into_iter()
        .map(|mut f| {
            f.percentage = if total_size > 0 {
                (f.size_bytes as f64 / total_size as f64) * 100.0
            } else {
                0.0
            };

            // Check individual file size limit
            if f.size_bytes > config.max_file_size_bytes {
                f.exceeds_limit = true;
                report.warnings.push(format!(
                    "File '{}' ({}) exceeds individual limit ({})",
                    f.path, f.size_formatted, format_size(config.max_file_size_bytes)
                ));
            }

            f
        })
        .collect();

    // Sort by size for largest files
    files_with_percentage.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    report.largest_files = files_with_percentage.iter().take(10).cloned().collect();

    report.files = files_with_percentage;

    // Try to load baseline for comparison
    if config.track_changes {
        if let Some(baseline_path) = find_baseline(project_path) {
            if let Ok(baseline) = load_baseline(&baseline_path) {
                let change = calculate_size_change(baseline, total_size, config);

                if change.exceeds_failure {
                    report.passed = false;
                    report.warnings.push(format!(
                        "Bundle size increased by {:.1}% (threshold: {:.1}%)",
                        change.change_percent,
                        config.fail_threshold_percent
                    ));
                } else if change.exceeds_warning {
                    report.warnings.push(format!(
                        "Bundle size increased by {:.1}% (threshold: {:.1}%)",
                        change.change_percent,
                        config.warning_threshold_percent
                    ));
                }

                report.size_change = Some(change);
            }
        }
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    report
}

/// Analyze a directory for bundled files
fn analyze_directory(path: &Path, config: &BundleConfig) -> Vec<BundleFile> {
    let mut files = Vec::new();

    for entry in WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let entry_path = entry.path();

        if !entry_path.is_file() {
            continue;
        }

        let relative_path = entry_path
            .strip_prefix(path)
            .unwrap_or(entry_path)
            .to_string_lossy()
            .to_string();

        // Check exclusions
        let is_excluded = config.exclude.iter().any(|pattern| {
            glob::Pattern::new(pattern)
                .map(|p| p.matches(&relative_path))
                .unwrap_or(false)
        });

        if is_excluded {
            continue;
        }

        // Check inclusions if specified
        if !config.include.is_empty() {
            let is_included = config.include.iter().any(|pattern| {
                glob::Pattern::new(pattern)
                    .map(|p| p.matches(&relative_path))
                    .unwrap_or(false)
            });

            if !is_included {
                continue;
            }
        }

        if let Ok(metadata) = entry_path.metadata() {
            let size = metadata.len();
            let file_type = entry_path
                .extension()
                .map(|e| e.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());

            files.push(BundleFile {
                path: relative_path,
                size_bytes: size,
                size_formatted: format_size(size),
                percentage: 0.0, // Calculated later
                file_type,
                exceeds_limit: false,
            });
        }
    }

    files
}

/// Find baseline file for comparison
fn find_baseline(project_path: &Path) -> Option<std::path::PathBuf> {
    let candidates = [
        project_path.join(".bundle-baseline.json"),
        project_path.join(".oxide/bundle-baseline.json"),
    ];

    for candidate in &candidates {
        if candidate.exists() {
            return Some(candidate.clone());
        }
    }

    None
}

/// Load baseline from file
fn load_baseline(path: &Path) -> Result<u64, std::io::Error> {
    let content = std::fs::read_to_string(path)?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    data.get("total_size_bytes")
        .and_then(|v| v.as_u64())
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "Missing total_size_bytes"))
}

/// Calculate size change
fn calculate_size_change(baseline: u64, current: u64, config: &BundleConfig) -> SizeChange {
    let change_bytes = current as i64 - baseline as i64;
    let change_percent = if baseline > 0 {
        (change_bytes as f64 / baseline as f64) * 100.0
    } else {
        0.0
    };

    SizeChange {
        previous_bytes: baseline,
        current_bytes: current,
        change_bytes,
        change_percent,
        exceeds_warning: change_percent > config.warning_threshold_percent,
        exceeds_failure: change_percent > config.fail_threshold_percent,
    }
}

/// Format bytes to human-readable string
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

/// Save current bundle size as baseline
pub fn save_baseline(project_path: &Path, report: &BundleReport) -> Result<(), std::io::Error> {
    let baseline_path = project_path.join(".bundle-baseline.json");

    let data = serde_json::json!({
        "total_size_bytes": report.total_size_bytes,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "files_count": report.files.len(),
    });

    let content = serde_json::to_string_pretty(&data)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    std::fs::write(baseline_path, content)
}

/// Generate SBOM (Software Bill of Materials)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sbom {
    /// SBOM format version
    pub sbom_version: String,
    /// Project name
    pub name: String,
    /// Project version
    pub version: String,
    /// Components/dependencies
    pub components: Vec<SbomComponent>,
    /// Generation timestamp
    pub timestamp: String,
}

/// SBOM component entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SbomComponent {
    /// Component name
    pub name: String,
    /// Component version
    pub version: String,
    /// Component type (library, application, etc.)
    pub component_type: String,
    /// License
    pub license: Option<String>,
    /// Package URL
    pub purl: Option<String>,
}

/// Generate SBOM from Cargo.lock
pub fn generate_sbom(project_path: &Path) -> Result<Sbom, std::io::Error> {
    let cargo_lock_path = project_path.join("Cargo.lock");

    if !cargo_lock_path.exists() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Cargo.lock not found",
        ));
    }

    let content = std::fs::read_to_string(&cargo_lock_path)?;
    let lock_file: toml::Value = toml::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let mut components = Vec::new();

    if let Some(packages) = lock_file.get("package").and_then(|p| p.as_array()) {
        for package in packages {
            let name = package.get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            let version = package.get("version")
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0")
                .to_string();

            components.push(SbomComponent {
                name: name.clone(),
                version: version.clone(),
                component_type: "library".to_string(),
                license: None, // Would need to look up from Cargo.toml or crates.io
                purl: Some(format!("pkg:cargo/{}@{}", name, version)),
            });
        }
    }

    // Read project info from Cargo.toml
    let cargo_toml_path = project_path.join("Cargo.toml");
    let (project_name, project_version) = if cargo_toml_path.exists() {
        let content = std::fs::read_to_string(&cargo_toml_path)?;
        let cargo_toml: toml::Value = toml::from_str(&content)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        let name = cargo_toml.get("package")
            .and_then(|p| p.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        let version = cargo_toml.get("package")
            .and_then(|p| p.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");

        (name.to_string(), version.to_string())
    } else {
        ("unknown".to_string(), "0.0.0".to_string())
    };

    Ok(Sbom {
        sbom_version: "1.0".to_string(),
        name: project_name,
        version: project_version,
        components,
        timestamp: chrono::Utc::now().to_rfc3339(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_size() {
        assert_eq!(format_size(0), "0 B");
        assert_eq!(format_size(512), "512 B");
        assert_eq!(format_size(1024), "1.00 KB");
        assert_eq!(format_size(1024 * 1024), "1.00 MB");
        assert_eq!(format_size(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_bundle_report() {
        let config = BundleConfig::default();
        let report = BundleReport::new(&config);

        assert!(report.passed);
        assert!(!report.exceeded_size);
    }

    #[test]
    fn test_size_change() {
        let config = BundleConfig::default();
        let change = calculate_size_change(1000, 1100, &config);

        assert_eq!(change.change_bytes, 100);
        assert_eq!(change.change_percent, 10.0);
        assert!(change.exceeds_warning);
        assert!(!change.exceeds_failure);
    }
}
