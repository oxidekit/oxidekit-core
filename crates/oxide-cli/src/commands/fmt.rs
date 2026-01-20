//! `oxide fmt` command - format code

use anyhow::Result;
use std::process::Command;

pub fn run(check: bool) -> Result<()> {
    tracing::info!("Formatting code...");

    let mut cmd = Command::new("cargo");
    cmd.arg("fmt");

    if check {
        cmd.arg("--check");
    }

    let status = cmd.status()?;

    if !status.success() {
        if check {
            anyhow::bail!("Code is not formatted. Run `oxide fmt` to fix.");
        } else {
            anyhow::bail!("Formatting failed");
        }
    }

    if !check {
        println!("  Code formatted successfully");
    }

    Ok(())
}
