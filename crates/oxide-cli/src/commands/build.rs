//! `oxide build` command - build the application

use anyhow::Result;
use std::path::Path;
use std::process::Command;

pub fn run(release: bool, target: Option<String>) -> Result<()> {
    let manifest_path = Path::new("oxide.toml");

    if !manifest_path.exists() {
        anyhow::bail!(
            "No oxide.toml found in current directory. Run `oxide new <name>` to create a project."
        );
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
