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

    // Copy assets to dist/
    let assets_copied = copy_assets(out_dir)?;

    println!();
    println!("  Static build complete");
    println!("  Output: {}", out_path.display());
    if assets_copied > 0 {
        println!("  Assets: {} files copied to dist/assets/", assets_copied);
    }
    println!();
    println!("  To serve locally:");
    println!("    cd dist && python3 -m http.server 8000");
    println!();

    Ok(())
}

/// Copy project assets to the dist directory
fn copy_assets(out_dir: &Path) -> Result<usize> {
    let mut count = 0;

    // Check common asset directories
    let asset_dirs = ["assets", "public", "static"];

    for dir_name in &asset_dirs {
        let source_dir = Path::new(dir_name);
        if source_dir.exists() && source_dir.is_dir() {
            let dest_dir = if *dir_name == "assets" {
                out_dir.join("assets")
            } else {
                // For public/static, copy contents directly to dist
                out_dir.to_path_buf()
            };
            count += copy_dir_recursive(source_dir, &dest_dir)?;
        }
    }

    Ok(count)
}

/// Recursively copy a directory
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize> {
    let mut count = 0;

    if !dst.exists() {
        fs::create_dir_all(dst)?;
    }

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if file_type.is_dir() {
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
            count += 1;
            tracing::debug!("Copied: {} -> {}", src_path.display(), dst_path.display());
        }
    }

    Ok(count)
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
