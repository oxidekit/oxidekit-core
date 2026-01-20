//! `oxide build` command - build the application

use anyhow::Result;
use std::fs;
use std::path::Path;
use std::process::Command;

pub fn run(release: bool, target: Option<String>) -> Result<()> {
    let manifest_path = Path::new("oxide.toml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No oxide.toml found in current directory. Run `oxide new <name>` to create a project."
        );
    }

    // Check for static target
    if let Some(ref t) = target {
        if t == "static" {
            return build_static();
        }
    }

    tracing::info!("Building application...");

    let mut cmd = Command::new("cargo");
    cmd.arg("build");

    if release {
        cmd.arg("--release");
    }

    if let Some(t) = target {
        cmd.arg("--target").arg(&t);
    }

    let status = cmd.status()?;

    if !status.success() {
        anyhow::bail!("Build failed");
    }

    println!();
    if release {
        println!("  Build complete (release)");
        println!("  Binary: target/release/<app-name>");
    } else {
        println!("  Build complete (debug)");
        println!("  Binary: target/debug/<app-name>");
    }
    println!();

    Ok(())
}

/// Build static HTML output from .oui files
fn build_static() -> Result<()> {
    use oxide_compiler::{compile_file, generate_html};

    tracing::info!("Building static HTML...");

    // Read oxide.toml to get app name
    let manifest = fs::read_to_string("oxide.toml")?;
    let title = extract_title(&manifest);

    // Find the main .oui file
    let ui_path = Path::new("ui/app.oui");
    if !ui_path.exists() {
        anyhow::bail!("ui/app.oui not found");
    }

    // Create output directory
    let out_dir = Path::new("dist");
    fs::create_dir_all(out_dir)?;

    // Compile and generate HTML
    let ir = compile_file(ui_path)?;
    let html = generate_html(&ir, &title);

    // Write output
    let out_path = out_dir.join("index.html");
    fs::write(&out_path, html)?;

    println!();
    println!("  Static build complete");
    println!("  Output: {}", out_path.display());
    println!();
    println!("  To serve locally:");
    println!("    cd dist && python3 -m http.server 8000");
    println!();

    Ok(())
}

fn extract_title(manifest: &str) -> String {
    // Simple TOML parsing for title
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with("title") {
            if let Some(val) = line.split('=').nth(1) {
                return val.trim().trim_matches('"').to_string();
            }
        }
        if line.starts_with("name") {
            if let Some(val) = line.split('=').nth(1) {
                return val.trim().trim_matches('"').to_string();
            }
        }
    }
    "OxideKit App".to_string()
}
