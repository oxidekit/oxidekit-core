//! `oxide lint` command - lint code

use anyhow::Result;
use std::process::Command;

pub fn run() -> Result<()> {
    tracing::info!("Linting code...");

    let status = Command::new("cargo")
        .arg("clippy")
        .arg("--")
        .arg("-D")
        .arg("warnings")
        .status()?;

    if !status.success() {
        anyhow::bail!("Linting failed - fix the warnings above");
    }

    println!("  No linting issues found");

    Ok(())
}
