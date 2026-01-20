//! `oxide starters` command - starter template management

use anyhow::Result;
use oxide_starters::{StarterRegistry, StarterGenerator, StarterCategory, StarterTarget};

/// List all available starters
pub fn run_list(category: Option<&str>, target: Option<&str>) -> Result<()> {
    let registry = StarterRegistry::with_builtin();

    let starters = if let Some(cat) = category {
        let category = match cat {
            "admin" => StarterCategory::Admin,
            "docs" => StarterCategory::Docs,
            "website" => StarterCategory::Website,
            "wallet" => StarterCategory::Wallet,
            "monitoring" => StarterCategory::Monitoring,
            "app" => StarterCategory::App,
            _ => {
                println!("Unknown category: {}", cat);
                println!("Available: admin, docs, website, wallet, monitoring, app");
                return Ok(());
            }
        };
        registry.list_by_category(category)
    } else if let Some(tgt) = target {
        let target = match tgt {
            "desktop" => StarterTarget::Desktop,
            "web" => StarterTarget::Web,
            "static" => StarterTarget::Static,
            _ => {
                println!("Unknown target: {}", tgt);
                println!("Available: desktop, web, static");
                return Ok(());
            }
        };
        registry.list_by_target(target)
    } else {
        registry.list()
    };

    println!("Available Starters");
    println!("==================");
    println!();

    for starter in starters {
        let official = if starter.metadata.official { "[official]" } else { "" };
        let featured = if starter.metadata.featured { "*" } else { "" };

        println!(
            "  {}{} {} - {}",
            featured,
            starter.id,
            official,
            starter.description
        );

        let targets: Vec<_> = starter.targets.iter().map(|t| t.as_str()).collect();
        println!("    Targets: {}", targets.join(", "));
        println!();
    }

    println!("Use 'oxide starters info <starter-id>' for more details.");
    println!("Use 'oxide new <name> --starter <starter-id>' to create a project.");

    Ok(())
}

/// Show detailed information about a starter
pub fn run_info(starter_id: &str) -> Result<()> {
    let registry = StarterRegistry::with_builtin();

    let starter = match registry.get(starter_id) {
        Some(s) => s,
        None => {
            println!("Starter not found: {}", starter_id);
            println!();
            println!("Available starters:");
            for s in registry.list() {
                println!("  - {}", s.id);
            }
            return Ok(());
        }
    };

    println!("{}", starter.name);
    println!("{}", "=".repeat(starter.name.len()));
    println!();

    if starter.metadata.official {
        println!("[Official OxideKit Starter]");
        println!();
    }

    println!("{}", starter.description);
    println!();

    if let Some(ref long_desc) = starter.long_description {
        println!("{}", long_desc);
        println!();
    }

    println!("Details:");
    println!("  ID: {}", starter.id);
    println!("  Version: {}", starter.version);
    println!("  Category: {}", starter.metadata.category.as_str());

    let targets: Vec<_> = starter.targets.iter().map(|t| t.as_str()).collect();
    println!("  Targets: {}", targets.join(", "));

    if !starter.metadata.tags.is_empty() {
        println!("  Tags: {}", starter.metadata.tags.join(", "));
    }

    println!();
    println!("Plugins:");
    for plugin in &starter.plugins {
        let optional = if plugin.optional { "(optional)" } else { "" };
        let version = plugin.version.as_deref().unwrap_or("latest");
        println!("  - {} {} {}", plugin.id, version, optional);
    }

    println!();
    println!("Usage:");
    println!("  oxide new my-app --starter {}", starter.id);

    Ok(())
}

/// Search starters by query
pub fn run_search(query: &str) -> Result<()> {
    let registry = StarterRegistry::with_builtin();
    let results = registry.search(query);

    if results.is_empty() {
        println!("No starters found matching: {}", query);
        return Ok(());
    }

    println!("Search Results for '{}'", query);
    println!();

    for starter in results {
        println!("  {} - {}", starter.id, starter.description);
    }

    Ok(())
}

/// Create a new project from a starter
pub fn run_new_with_starter(name: &str, starter_id: &str, output_dir: Option<&str>) -> Result<()> {
    let registry = StarterRegistry::with_builtin();

    let starter = match registry.get(starter_id) {
        Some(s) => s,
        None => {
            anyhow::bail!("Starter not found: {}\n\nUse 'oxide starters list' to see available starters.", starter_id);
        }
    };

    let output_path = match output_dir {
        Some(dir) => std::path::PathBuf::from(dir),
        None => std::env::current_dir()?,
    };

    println!("Creating new project '{}' from starter '{}'...", name, starter.name);
    println!();

    let generator = StarterGenerator::new(starter);
    let result = generator.generate(name, &output_path)?;

    println!("{}", result.summary());

    Ok(())
}

/// Initialize current directory with a starter
pub fn run_init_with_starter(starter_id: &str) -> Result<()> {
    let registry = StarterRegistry::with_builtin();

    let starter = match registry.get(starter_id) {
        Some(s) => s,
        None => {
            anyhow::bail!("Starter not found: {}", starter_id);
        }
    };

    let current_dir = std::env::current_dir()?;
    let project_name = current_dir
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("my-app");

    // Check if directory is empty or has only .git
    let entries: Vec<_> = std::fs::read_dir(&current_dir)?
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() != ".git")
        .collect();

    if !entries.is_empty() {
        anyhow::bail!(
            "Directory is not empty. Use 'oxide new <name> --starter {}' to create in a new directory.",
            starter_id
        );
    }

    println!("Initializing project with starter '{}'...", starter.name);
    println!();

    // Generate files directly in current directory
    let generator = StarterGenerator::new(starter);

    // Create a temporary parent to generate into, then move files
    let temp_dir = tempfile::tempdir()?;
    let result = generator.generate(project_name, temp_dir.path())?;

    // Move generated files to current directory
    let generated_dir = temp_dir.path().join(project_name);
    for entry in std::fs::read_dir(&generated_dir)? {
        let entry = entry?;
        let dest = current_dir.join(entry.file_name());
        std::fs::rename(entry.path(), dest)?;
    }

    println!("Project initialized successfully!");
    println!();
    println!("Next steps:");
    println!("  1. Run 'oxide dev' to start development server");

    if !result.plugins_to_install.is_empty() {
        println!("  2. Install plugins:");
        for plugin in &result.plugins_to_install {
            println!("     oxide add {}", plugin);
        }
    }

    Ok(())
}
