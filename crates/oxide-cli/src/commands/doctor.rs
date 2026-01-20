//! `oxide doctor` command - diagnose project issues

use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run() -> Result<()> {
    println!();
    println!("  OxideKit Doctor");
    println!("  ───────────────");
    println!();

    let mut issues = 0;

    // Check Rust toolchain
    print!("  Rust toolchain: ");
    match Command::new("rustc").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{}", version.trim());
        }
        Err(_) => {
            println!("NOT FOUND");
            issues += 1;
        }
    }

    // Check cargo
    print!("  Cargo: ");
    match Command::new("cargo").arg("--version").output() {
        Ok(output) => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{}", version.trim());
        }
        Err(_) => {
            println!("NOT FOUND");
            issues += 1;
        }
    }

    // Check oxide.toml
    print!("  oxide.toml: ");
    if Path::new("oxide.toml").exists() {
        println!("found");

        // Try to parse it
        print!("  manifest parse: ");
        match std::fs::read_to_string("oxide.toml") {
            Ok(content) => match toml::from_str::<oxide_runtime::Manifest>(&content) {
                Ok(manifest) => {
                    println!("ok ({} v{})", manifest.app.name, manifest.app.version);
                }
                Err(e) => {
                    println!("PARSE ERROR: {}", e);
                    issues += 1;
                }
            },
            Err(e) => {
                println!("READ ERROR: {}", e);
                issues += 1;
            }
        }
    } else {
        println!("not found (not in project directory)");
    }

    // Check UI files
    print!("  UI files: ");
    if Path::new("ui").exists() {
        let ui_files = std::fs::read_dir("ui")
            .map(|entries| {
                entries
                    .filter_map(|e| e.ok())
                    .filter(|e| {
                        e.path()
                            .extension()
                            .map(|ext| ext == "oui")
                            .unwrap_or(false)
                    })
                    .count()
            })
            .unwrap_or(0);
        println!("{} .oui file(s) found", ui_files);
    } else {
        println!("ui/ directory not found");
    }

    println!();

    if issues > 0 {
        println!("  Found {} issue(s)", issues);
        anyhow::bail!("Project has issues");
    } else {
        println!("  All checks passed!");
    }

    println!();

    Ok(())
}
