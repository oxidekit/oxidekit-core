//! Version and compatibility commands
//!
//! `oxide version` - Version checking and compatibility enforcement

use anyhow::{Result, Context};
use oxide_version::prelude::*;
use std::path::Path;

/// Run version check command
pub fn run_check() -> Result<()> {
    println!("OxideKit Version Check\n");
    println!("{}", "=".repeat(40));

    // Check core version
    let core_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    println!("\nCore Version: {}", core_version);

    // Load lockfile if exists
    let lockfile_path = Path::new("oxide.lock");
    if lockfile_path.exists() {
        let lockfile = Lockfile::load(lockfile_path)
            .context("Failed to load lockfile")?;

        println!("\nInstalled Components:");
        println!("  Core: {}", lockfile.oxidekit.version);

        if !lockfile.plugins.is_empty() {
            println!("\n  Plugins:");
            for (name, entry) in &lockfile.plugins {
                println!("    - {} @ {}", name, entry.version);
            }
        }

        if !lockfile.themes.is_empty() {
            println!("\n  Themes:");
            for (name, entry) in &lockfile.themes {
                println!("    - {} @ {}", name, entry.version);
            }
        }

        // Check for version mismatches
        if lockfile.oxidekit.version != core_version {
            println!("\n[WARNING] Lockfile core version ({}) differs from installed ({})",
                lockfile.oxidekit.version, core_version);
        }
    } else {
        println!("\nNo lockfile found (oxide.lock)");
    }

    // Check manifest if exists
    let manifest_path = Path::new("oxide.toml");
    if manifest_path.exists() {
        println!("\nManifest found (oxide.toml)");
    }

    Ok(())
}

/// Run compatibility check between components
pub fn run_compat(component: &str, version: &str) -> Result<()> {
    let core_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let target_version = Version::parse(version)?;

    println!("Compatibility Check\n");
    println!("{}", "=".repeat(40));
    println!("\nChecking {} @ {} against OxideKit {}\n",
        component, target_version, core_version);

    // Create compatibility checker
    let compat = Compatibility::new(core_version.clone());

    // Create mock component for checking
    let component_version = ComponentVersion::new(
        component,
        ComponentType::Plugin, // Default to plugin for now
        target_version.clone(),
        VersionReq::any(), // Would come from actual manifest
    );

    let result = compat.check_component(&component_version);

    if result.compatible {
        println!("[OK] {} {} is compatible with OxideKit {}",
            component, target_version, core_version);
    } else {
        println!("[FAIL] {} {} is NOT compatible with OxideKit {}",
            component, target_version, core_version);
        println!("\nReason: {}", result.explanation);
        if let Some(suggestion) = result.suggestion {
            println!("\nSuggestion: {}", suggestion);
        }
    }

    Ok(())
}

/// Explain compatibility between versions
pub fn run_explain(from: &str, to: &str) -> Result<()> {
    let from_version = Version::parse(from)?;
    let to_version = Version::parse(to)?;

    println!("Version Compatibility Explanation\n");
    println!("{}", "=".repeat(40));
    println!("\nComparing {} to {}\n", from_version, to_version);

    // Determine bump type
    match from_version.bump_type_to(&to_version) {
        Some(VersionBump::Major) => {
            println!("This is a MAJOR version bump.");
            println!("\nMajor versions may contain:");
            println!("  - Breaking API changes");
            println!("  - Removed deprecated features");
            println!("  - Significant behavior changes");
            println!("\nRecommendation: Review migration guide before upgrading");
        }
        Some(VersionBump::Minor) => {
            println!("This is a MINOR version bump.");
            println!("\nMinor versions typically contain:");
            println!("  - New features (backwards compatible)");
            println!("  - Performance improvements");
            println!("  - Bug fixes");
            println!("\nRecommendation: Review changelog, upgrade should be safe");
        }
        Some(VersionBump::Patch) => {
            println!("This is a PATCH version bump.");
            println!("\nPatch versions contain:");
            println!("  - Bug fixes only");
            println!("  - Security patches");
            println!("  - No API changes");
            println!("\nRecommendation: Safe to upgrade");
        }
        None => {
            if from_version > to_version {
                println!("This is a DOWNGRADE.");
                println!("\nWarning: Downgrading may cause issues if you rely on newer features.");
            } else {
                println!("Versions are identical.");
            }
        }
    }

    // Check semver compatibility
    if from_version.is_compatible_with(&to_version) {
        println!("\n[OK] Versions are semver-compatible");
    } else {
        println!("\n[WARNING] Versions may have breaking changes");
    }

    Ok(())
}

/// Show upgrade path between versions
pub fn run_upgrade_path(from: &str, to: &str) -> Result<()> {
    let from_version = Version::parse(from)?;
    let to_version = Version::parse(to)?;

    println!("Upgrade Path: {} -> {}\n", from_version, to_version);
    println!("{}", "=".repeat(40));

    if from_version >= to_version {
        println!("\nNo upgrade needed (target version is not newer)");
        return Ok(());
    }

    // Show intermediate versions that might need migration guides
    let mut current = from_version.clone();
    let mut steps = Vec::new();

    // Check if major version upgrade
    while current.major < to_version.major {
        let next = current.next_major();
        steps.push((current.clone(), next.clone(), "Major upgrade - check migration guide"));
        current = next;
    }

    // Check if minor version upgrade
    while current.minor < to_version.minor {
        let next = current.next_minor();
        steps.push((current.clone(), next.clone(), "Minor upgrade"));
        current = next;
    }

    // Check patch
    if current.patch < to_version.patch {
        steps.push((current.clone(), to_version.clone(), "Patch upgrade"));
    }

    if steps.is_empty() {
        println!("\nDirect upgrade is possible.");
    } else {
        println!("\nRecommended upgrade path:\n");
        for (i, (from, to, note)) in steps.iter().enumerate() {
            println!("  Step {}: {} -> {} ({})", i + 1, from, to, note);
        }
    }

    Ok(())
}

/// Show deprecation warnings for a version
pub fn run_deprecations(version: Option<&str>) -> Result<()> {
    let target_version = match version {
        Some(v) => Version::parse(v)?,
        None => Version::parse(env!("CARGO_PKG_VERSION"))?,
    };

    println!("Deprecation Warnings for OxideKit {}\n", target_version);
    println!("{}", "=".repeat(40));

    // Create sample deprecation registry (in real impl, load from data)
    let registry = DeprecationRegistryBuilder::new()
        .deprecate(
            "Widget::render",
            "0.5.0",
            "Use Widget::draw instead for better performance"
        )
        .deprecate_for_removal(
            "old_config_format",
            "0.4.0",
            "1.0.0",
            "TOML config format is now required"
        )
        .build();

    let warnings = registry.warnings_for(&target_version);

    if warnings.is_empty() {
        println!("\nNo deprecation warnings for this version.");
    } else {
        println!("\nFound {} deprecation warning(s):\n", warnings.len());
        for warning in warnings {
            println!("{}", warning);
        }
    }

    Ok(())
}

/// Generate compatibility matrix report
pub fn run_matrix(core_version: Option<&str>) -> Result<()> {
    let target = match core_version {
        Some(v) => Version::parse(v)?,
        None => Version::parse(env!("CARGO_PKG_VERSION"))?,
    };

    println!("Compatibility Matrix for OxideKit {}\n", target);
    println!("{}", "=".repeat(40));

    // In real implementation, load matrix from network or cache
    let matrix = CompatibilityMatrix::new();
    let report = matrix.generate_report(&target);

    println!("\n{}", report.to_text());

    Ok(())
}

/// Validate lockfile
pub fn run_validate_lockfile() -> Result<()> {
    let lockfile_path = Path::new("oxide.lock");

    if !lockfile_path.exists() {
        println!("No lockfile found (oxide.lock)");
        return Ok(());
    }

    println!("Validating oxide.lock\n");
    println!("{}", "=".repeat(40));

    let lockfile = Lockfile::load(lockfile_path)
        .context("Failed to load lockfile")?;

    println!("\nLockfile version: {}", lockfile.version);
    println!("Total components: {}", lockfile.component_count());

    // Validate schema version
    let current_schema = LockfileVersion::current();
    if lockfile.version != current_schema.schema {
        println!("\n[WARNING] Lockfile schema version mismatch");
        println!("  Expected: {}", current_schema.schema);
        println!("  Found: {}", lockfile.version);
    } else {
        println!("\n[OK] Lockfile schema is valid");
    }

    // Check all components
    let core_version = Version::parse(env!("CARGO_PKG_VERSION"))?;
    let compat = Compatibility::new(core_version.clone());

    let mut issues = 0;

    for (comp_type, name, entry) in lockfile.all_entries() {
        if comp_type == ComponentType::Core {
            continue; // Skip core itself
        }

        let component = ComponentVersion::new(
            name,
            comp_type,
            entry.version.clone(),
            VersionReq::any(), // Would come from manifest
        );

        let result = compat.check_component(&component);
        if !result.compatible {
            println!("\n[FAIL] {} {} incompatible: {}", name, entry.version, result.explanation);
            issues += 1;
        }
    }

    if issues == 0 {
        println!("\n[OK] All components are compatible");
    } else {
        println!("\n[FAIL] Found {} compatibility issue(s)", issues);
    }

    Ok(())
}

/// Update lockfile
pub fn run_update_lockfile() -> Result<()> {
    let lockfile_path = Path::new("oxide.lock");

    println!("Updating oxide.lock\n");
    println!("{}", "=".repeat(40));

    let core_version = Version::parse(env!("CARGO_PKG_VERSION"))?;

    let lockfile = if lockfile_path.exists() {
        let mut existing = Lockfile::load(lockfile_path)?;

        // Check if update needed
        if existing.oxidekit.version == core_version {
            println!("\nLockfile is up to date.");
            return Ok(());
        }

        println!("\nUpdating core version: {} -> {}",
            existing.oxidekit.version, core_version);

        existing.oxidekit.version = core_version;
        existing.metadata.update();
        existing
    } else {
        println!("\nCreating new lockfile...");
        let mut lockfile = Lockfile::new(core_version);
        lockfile.metadata.update();
        lockfile.metadata.generator_version = Some(env!("CARGO_PKG_VERSION").to_string());
        lockfile
    };

    lockfile.save(lockfile_path)?;
    println!("\n[OK] Lockfile updated successfully");

    Ok(())
}
