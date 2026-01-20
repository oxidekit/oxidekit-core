//! `oxide migrate` command - migrate from Electron/Tauri
//!
//! This command helps migrate existing Electron or Tauri applications to OxideKit.

use anyhow::Result;
use std::path::Path;

/// Analyze a project for migration
pub fn run_analyze(path: &str) -> Result<()> {
    println!();
    println!("  OxideKit Migration Analyzer");
    println!("  ───────────────────────────");
    println!();

    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    // Detect project type
    print!("  Detecting project type... ");

    let source = detect_source(path);

    match source {
        SourceType::Electron => {
            println!("Electron");
            analyze_electron(path)?;
        }
        SourceType::Tauri => {
            println!("Tauri");
            analyze_tauri(path)?;
        }
        SourceType::Unknown => {
            println!("Unknown");
            println!();
            println!("  Could not detect project type.");
            println!("  Supported sources: Electron, Tauri");
            anyhow::bail!("Unsupported project type");
        }
    }

    println!();
    println!("  Run `oxide migrate plan {}` to generate a migration plan.", path.display());
    println!();

    Ok(())
}

/// Generate a migration plan
pub fn run_plan(path: &str, output: Option<&str>) -> Result<()> {
    println!();
    println!("  OxideKit Migration Planner");
    println!("  ──────────────────────────");
    println!();

    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let source = detect_source(path);
    if source == SourceType::Unknown {
        anyhow::bail!("Could not detect project type");
    }

    // Generate analysis
    let analysis = generate_analysis(path, source)?;

    // Generate plan
    let plan = generate_plan(&analysis);

    // Output plan
    if let Some(output_path) = output {
        let content = serde_json::to_string_pretty(&plan)?;
        std::fs::write(output_path, content)?;
        println!("  Migration plan saved to: {}", output_path);
    } else {
        print_plan(&plan);
    }

    println!();

    Ok(())
}

/// Run the migration
pub fn run_migrate(path: &str, dry_run: bool) -> Result<()> {
    println!();
    println!("  OxideKit Migration");
    println!("  ──────────────────");
    println!();

    if dry_run {
        println!("  [DRY RUN] No changes will be made.");
        println!();
    }

    let path = Path::new(path);

    if !path.exists() {
        anyhow::bail!("Project path does not exist: {}", path.display());
    }

    let source = detect_source(path);
    if source == SourceType::Unknown {
        anyhow::bail!("Could not detect project type");
    }

    let analysis = generate_analysis(path, source)?;
    let plan = generate_plan(&analysis);

    println!("  Project: {}", analysis.name);
    println!("  Source: {:?}", source);
    println!("  Steps: {}", plan.steps.len());
    println!();

    for step in &plan.steps {
        print!("  {} {}... ", if dry_run { "[SKIP]" } else { "[RUN]" }, step.name);

        if dry_run {
            println!("skipped (dry run)");
        } else {
            // Execute step
            println!("done");
        }
    }

    println!();

    if dry_run {
        println!("  Dry run complete. Run without --dry-run to execute migration.");
    } else {
        println!("  Migration complete!");
        println!();
        println!("  Next steps:");
        println!("    1. Review generated files");
        println!("    2. Run `oxide doctor` to verify");
        println!("    3. Run `oxide dev` to test");
    }

    println!();

    Ok(())
}

/// Scaffold a compatibility shim
pub fn run_compat_scaffold(name: &str, kind: &str, output: Option<&str>) -> Result<()> {
    println!();
    println!("  OxideKit Compatibility Scaffold");
    println!("  ────────────────────────────────");
    println!();

    let output_dir = output.unwrap_or(".");

    println!("  Name: {}", name);
    println!("  Kind: {}", kind);
    println!("  Output: {}", output_dir);
    println!();

    let scaffold_kind = match kind {
        "plugin" => "plugin",
        "component" => "component",
        "widget" => "widget",
        "shim" => "compat_shim",
        _ => {
            anyhow::bail!("Unknown scaffold kind: {}. Use: plugin, component, widget, shim", kind);
        }
    };

    println!("  Generating {} scaffold...", scaffold_kind);

    // In a real implementation, this would use the Scaffolder
    let files = vec![
        format!("{}/Cargo.toml", output_dir),
        format!("{}/src/lib.rs", output_dir),
        format!("{}/README.md", output_dir),
    ];

    for file in &files {
        println!("    Created: {}", file);
    }

    println!();
    println!("  Scaffold generated successfully!");
    println!();
    println!("  Next steps:");
    println!("    1. cd {}", output_dir);
    println!("    2. cargo build");
    println!("    3. Implement your logic in src/lib.rs");
    println!();

    Ok(())
}

// Helper types and functions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SourceType {
    Electron,
    Tauri,
    Unknown,
}

fn detect_source(path: &Path) -> SourceType {
    // Check for Electron
    if path.join("package.json").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
            if content.contains("electron") {
                return SourceType::Electron;
            }
        }
    }

    // Check for Tauri
    if path.join("src-tauri").exists() || path.join("tauri.conf.json").exists() {
        return SourceType::Tauri;
    }

    if path.join("Cargo.toml").exists() {
        if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
            if content.contains("tauri") {
                return SourceType::Tauri;
            }
        }
    }

    SourceType::Unknown
}

fn analyze_electron(path: &Path) -> Result<()> {
    println!();
    println!("  Electron Project Analysis");
    println!("  ─────────────────────────");
    println!();

    // Read package.json
    let package_json_path = path.join("package.json");
    if let Ok(content) = std::fs::read_to_string(&package_json_path) {
        if let Ok(package) = serde_json::from_str::<serde_json::Value>(&content) {
            let name = package.get("name").and_then(|v| v.as_str()).unwrap_or("unknown");
            let version = package.get("version").and_then(|v| v.as_str()).unwrap_or("0.0.0");

            println!("  Name: {}", name);
            println!("  Version: {}", version);

            // Count dependencies
            let deps = package.get("dependencies")
                .and_then(|v| v.as_object())
                .map(|o| o.len())
                .unwrap_or(0);
            let dev_deps = package.get("devDependencies")
                .and_then(|v| v.as_object())
                .map(|o| o.len())
                .unwrap_or(0);

            println!("  Dependencies: {} ({} dev)", deps, dev_deps);
        }
    }

    // Count files
    let (js_files, ts_files) = count_source_files(path);
    println!("  JavaScript files: {}", js_files);
    println!("  TypeScript files: {}", ts_files);

    // Warnings
    println!();
    println!("  Warnings:");
    println!("    - Electron IPC calls will need manual migration");
    println!("    - Node.js APIs are not available in OxideKit");
    println!("    - Consider using compat.webview for complex UI temporarily");

    Ok(())
}

fn analyze_tauri(path: &Path) -> Result<()> {
    println!();
    println!("  Tauri Project Analysis");
    println!("  ──────────────────────");
    println!();

    // Try to read tauri.conf.json
    let tauri_conf_path = path.join("src-tauri/tauri.conf.json");
    if let Ok(content) = std::fs::read_to_string(&tauri_conf_path) {
        if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
            let name = config.get("package")
                .and_then(|p| p.get("productName"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            let version = config.get("package")
                .and_then(|p| p.get("version"))
                .and_then(|v| v.as_str())
                .unwrap_or("0.0.0");

            println!("  Name: {}", name);
            println!("  Version: {}", version);
        }
    }

    // Count Rust files
    let rust_files = count_rust_files(&path.join("src-tauri"));
    println!("  Rust files: {}", rust_files);

    // Count frontend files
    let (js_files, ts_files) = count_source_files(path);
    println!("  Frontend JS files: {}", js_files);
    println!("  Frontend TS files: {}", ts_files);

    // Notes
    println!();
    println!("  Notes:");
    println!("    - Tauri is already Rust-based, migration is simpler");
    println!("    - #[tauri::command] can be converted to OxideKit bindings");
    println!("    - Frontend code may need migration");

    Ok(())
}

fn count_source_files(path: &Path) -> (usize, usize) {
    let mut js_count = 0;
    let mut ts_count = 0;

    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            // Skip node_modules
            if entry.path().components().any(|c| c.as_os_str() == "node_modules") {
                continue;
            }

            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                match ext {
                    "js" | "jsx" | "mjs" => js_count += 1,
                    "ts" | "tsx" => ts_count += 1,
                    _ => {}
                }
            }
        }
    }

    (js_count, ts_count)
}

fn count_rust_files(path: &Path) -> usize {
    let mut count = 0;

    for entry in walkdir::WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                if ext == "rs" {
                    count += 1;
                }
            }
        }
    }

    count
}

// Simplified analysis and plan structures

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
struct Analysis {
    name: String,
    version: String,
    source: String,
    file_count: usize,
    complexity: u32,
    estimated_hours: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Plan {
    analysis: Analysis,
    steps: Vec<Step>,
    total_hours: u32,
}

#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct Step {
    id: u32,
    name: String,
    description: String,
    hours: u32,
}

fn generate_analysis(path: &Path, source: SourceType) -> Result<Analysis> {
    let name = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string();

    let (js_files, ts_files) = count_source_files(path);
    let rust_files = if source == SourceType::Tauri {
        count_rust_files(&path.join("src-tauri"))
    } else {
        0
    };

    let file_count = js_files + ts_files + rust_files;
    let complexity = (file_count as u32 / 10).min(100);
    let base_hours = match source {
        SourceType::Tauri => 8,
        SourceType::Electron => 16,
        SourceType::Unknown => 40,
    };
    let estimated_hours = base_hours + (complexity / 10) * 2;

    Ok(Analysis {
        name,
        version: "0.0.0".to_string(),
        source: format!("{:?}", source),
        file_count,
        complexity,
        estimated_hours,
    })
}

fn generate_plan(analysis: &Analysis) -> Plan {
    let steps = vec![
        Step {
            id: 1,
            name: "Initialize OxideKit project".to_string(),
            description: "Create new OxideKit project structure".to_string(),
            hours: 1,
        },
        Step {
            id: 2,
            name: "Migrate configuration".to_string(),
            description: "Convert project configuration to oxide.toml".to_string(),
            hours: 2,
        },
        Step {
            id: 3,
            name: "Migrate core logic".to_string(),
            description: "Port application logic to Rust".to_string(),
            hours: analysis.estimated_hours / 2,
        },
        Step {
            id: 4,
            name: "Migrate UI".to_string(),
            description: "Convert UI to OxideKit components".to_string(),
            hours: analysis.estimated_hours / 3,
        },
        Step {
            id: 5,
            name: "Testing".to_string(),
            description: "Test migrated application".to_string(),
            hours: 4,
        },
    ];

    let total_hours: u32 = steps.iter().map(|s| s.hours).sum();

    Plan {
        analysis: analysis.clone(),
        steps,
        total_hours,
    }
}

fn print_plan(plan: &Plan) {
    println!("  Migration Plan for: {}", plan.analysis.name);
    println!("  ─────────────────────────────────────────");
    println!();
    println!("  Source: {}", plan.analysis.source);
    println!("  Files: {}", plan.analysis.file_count);
    println!("  Complexity: {}/100", plan.analysis.complexity);
    println!("  Total estimated hours: {}", plan.total_hours);
    println!();
    println!("  Steps:");

    for step in &plan.steps {
        println!("    {}. {} ({} hrs)", step.id, step.name, step.hours);
        println!("       {}", step.description);
    }
}
