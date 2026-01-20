//! `oxide export` command - export project artifacts

use anyhow::Result;
use oxide_components::ComponentRegistry;

/// Export AI schema to oxide.ai.json
pub fn run_ai_schema(output: Option<&str>) -> Result<()> {
    let output_path = output.unwrap_or("oxide.ai.json");

    tracing::info!("Exporting AI schema to {}", output_path);

    // Create registry with core components
    let registry = ComponentRegistry::with_core_components();

    // Export to JSON
    let json = registry.export_ai_json()?;

    // Write to file
    std::fs::write(output_path, &json)?;

    println!("Exported AI schema to {}", output_path);
    println!();
    println!("  Components: {}", registry.list_components().len());
    println!("  Packs: {}", registry.list_packs().join(", "));
    println!();
    println!("This file can be used by AI tools to understand your UI components.");

    Ok(())
}

/// Export typography roles to typography.toml
pub fn run_typography(output: Option<&str>) -> Result<()> {
    use oxide_components::FontRegistry;

    let output_path = output.unwrap_or("typography.toml");

    tracing::info!("Exporting typography to {}", output_path);

    let registry = FontRegistry::with_defaults();
    let toml = registry.export_typography_toml()?;

    std::fs::write(output_path, &toml)?;

    println!("Exported typography roles to {}", output_path);

    Ok(())
}

/// Export theme to theme.toml
pub fn run_theme(theme: &str, output: Option<&str>) -> Result<()> {
    use oxide_components::Theme;

    let output_path = output.unwrap_or("theme.toml");

    let theme = match theme {
        "dark" => Theme::dark(),
        "light" => Theme::light(),
        _ => anyhow::bail!("Unknown theme: {}. Available: dark, light", theme),
    };

    let toml = theme.to_toml()?;
    std::fs::write(output_path, &toml)?;

    println!("Exported {} theme to {}", theme.name, output_path);

    Ok(())
}
