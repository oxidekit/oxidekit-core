//! Documentation commands
//!
//! Commands for managing and viewing OxideKit documentation offline.

use anyhow::{Context, Result};
use oxide_docs::{
    bundler::{DocBundle, DocBundler},
    codegen::CodeDocGenerator,
    tutorials::{self, TutorialRunner},
    DocsConfig,
};
use std::path::PathBuf;
use tracing::info;

/// Open documentation in the browser
pub fn run_open(offline: bool, topic: Option<&str>) -> Result<()> {
    if offline {
        run_serve(3030, true, topic)
    } else {
        // Open online docs
        let url = match topic {
            Some(t) => format!("https://oxidekit.com/docs/{}", t),
            None => "https://oxidekit.com/docs".to_string(),
        };

        println!("Opening documentation: {}", url);
        open_url(&url)?;
        Ok(())
    }
}

/// Serve documentation locally
pub fn run_serve(port: u16, open: bool, _topic: Option<&str>) -> Result<()> {
    let bundle_path = oxide_docs::default_docs_dir();

    if !bundle_path.join("manifest.json").exists() {
        println!("No documentation bundle found.");
        println!("Building documentation bundle...\n");
        run_build(None)?;
    }

    let bundle = DocBundle::load(&bundle_path)
        .context("Failed to load documentation bundle")?;

    println!("\n  OxideKit Documentation");
    println!("  ======================");
    println!("  Version: {}", bundle.version());
    println!("  Pages:   {}", bundle.manifest.page_count());
    println!();

    if open {
        oxide_docs::viewer::open_in_browser(port)?;
    }

    bundle.serve(port)?;
    Ok(())
}

/// Build documentation bundle
pub fn run_build(output: Option<&str>) -> Result<()> {
    let output_dir = output
        .map(PathBuf::from)
        .unwrap_or_else(oxide_docs::default_docs_dir);

    // Check for docs directory
    let source_dir = PathBuf::from("docs");
    if !source_dir.exists() {
        println!("Creating default documentation structure...");
        create_default_docs(&source_dir)?;
    }

    let config = DocsConfig::default()
        .source_dir(source_dir)
        .output_dir(&output_dir);

    println!("Building documentation bundle...");
    println!("  Source: {:?}", config.source_dir);
    println!("  Output: {:?}", config.output_dir);
    println!();

    let bundle = DocBundle::build(&config)
        .context("Failed to build documentation bundle")?;

    println!("Documentation bundle built successfully!");
    println!("  Version: {}", bundle.version());
    println!("  Pages:   {}", bundle.manifest.page_count());
    println!("  Size:    {} KB", bundle.manifest.bundle_size / 1024);
    println!();
    println!("Run `oxide docs` to view the documentation.");

    Ok(())
}

/// Export documentation to an archive
pub fn run_export(output: Option<&str>) -> Result<()> {
    let bundle_path = oxide_docs::default_docs_dir();

    if !bundle_path.join("manifest.json").exists() {
        println!("No documentation bundle found. Building first...\n");
        run_build(None)?;
    }

    let bundle = DocBundle::load(&bundle_path)
        .context("Failed to load documentation bundle")?;

    let output_path = output
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(format!("oxidekit-docs-{}.tar.gz", bundle.version()))
        });

    println!("Exporting documentation to {:?}...", output_path);

    let exported = bundle.export(&output_path)?;
    let size = std::fs::metadata(&exported)?.len();

    println!("Documentation exported successfully!");
    println!("  File: {:?}", exported);
    println!("  Size: {} KB", size / 1024);

    Ok(())
}

/// Search the documentation
pub fn run_search(query: &str, limit: usize) -> Result<()> {
    let bundle_path = oxide_docs::default_docs_dir();

    if !bundle_path.join("manifest.json").exists() {
        println!("No documentation bundle found. Run `oxide docs build` first.");
        return Ok(());
    }

    let mut bundle = DocBundle::load(&bundle_path)
        .context("Failed to load documentation bundle")?;

    println!("Searching for: {}\n", query);

    let results = bundle.search(query, limit)?;

    if results.is_empty() {
        println!("No results found.");
    } else {
        println!("Found {} results:\n", results.len());

        for (i, result) in results.iter().enumerate() {
            println!("{}. {} (score: {:.2})", i + 1, result.title, result.score);
            if !result.snippet.is_empty() {
                println!("   {}", result.snippet);
            }
            println!();
        }
    }

    Ok(())
}

/// Generate API documentation from code
pub fn run_generate(path: Option<&str>, output: Option<&str>) -> Result<()> {
    let crate_path = path.map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));

    if !crate_path.join("Cargo.toml").exists() {
        println!("No Cargo.toml found at {:?}", crate_path);
        println!("Please run this command from a Rust crate directory.");
        return Ok(());
    }

    println!("Generating documentation from code...");
    println!("  Crate: {:?}", crate_path);

    let generator = CodeDocGenerator::new();
    let crate_docs = generator.generate_crate(&crate_path)
        .context("Failed to generate documentation")?;

    println!("  Name:    {}", crate_docs.name);
    println!("  Version: {}", crate_docs.version);
    println!("  Modules: {}", crate_docs.all_modules.len());
    println!("  Items:   {}", crate_docs.public_items.len());
    println!();

    // Generate markdown files
    let pages = generator.generate_markdown(&crate_docs)?;

    let output_dir = output.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from("docs").join("api")
    });

    std::fs::create_dir_all(&output_dir)?;

    for page in &pages {
        let file_path = output_dir.join(&page.path);
        if let Some(parent) = file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&file_path, &page.content)?;
        println!("  Generated: {:?}", file_path);
    }

    println!();
    println!("API documentation generated successfully!");
    println!("Run `oxide docs build` to include it in the documentation bundle.");

    Ok(())
}

/// List available tutorials
pub fn run_tutorials_list() -> Result<()> {
    let tutorials_dir = PathBuf::from("docs").join("tutorials");

    // Also check built-in tutorials
    let tutorials = if tutorials_dir.exists() {
        tutorials::list_tutorials(&tutorials_dir)?
    } else {
        Vec::new()
    };

    // Add built-in tutorials
    let mut all_tutorials = vec![
        tutorials::getting_started_tutorial().summary(),
        tutorials::component_tutorial().summary(),
    ];

    for tutorial in &tutorials {
        all_tutorials.push(tutorial.summary());
    }

    println!("Available Tutorials:\n");

    for summary in &all_tutorials {
        println!(
            "  {} - {} [{}] (~{} min)",
            summary.id,
            summary.title,
            summary.difficulty.display_name(),
            summary.estimated_minutes
        );
        println!("    {}", summary.description);
        println!();
    }

    println!("Run `oxide docs tutorial <id>` to start a tutorial.");

    Ok(())
}

/// Run a specific tutorial
pub fn run_tutorial(tutorial_id: &str) -> Result<()> {
    // Check built-in tutorials first
    let tutorial = match tutorial_id {
        "getting-started" => Some(tutorials::getting_started_tutorial()),
        "components-basics" => Some(tutorials::component_tutorial()),
        _ => {
            // Look in docs/tutorials directory
            let tutorials_dir = PathBuf::from("docs").join("tutorials");
            let tutorial_file = tutorials_dir.join(tutorial_id).join("tutorial.toml");

            if tutorial_file.exists() {
                Some(tutorials::load_tutorial(&tutorial_file)?)
            } else {
                None
            }
        }
    };

    match tutorial {
        Some(t) => {
            let mut runner = TutorialRunner::new(t)?;
            runner.run_interactive()?;
        }
        None => {
            println!("Tutorial not found: {}", tutorial_id);
            println!("Run `oxide docs tutorials` to see available tutorials.");
        }
    }

    Ok(())
}

// Helper functions

fn open_url(url: &str) -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .context("Failed to open URL")?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .context("Failed to open URL")?;
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .context("Failed to open URL")?;
    }

    Ok(())
}

fn create_default_docs(docs_dir: &PathBuf) -> Result<()> {
    std::fs::create_dir_all(docs_dir)?;

    // Create getting-started guide
    let getting_started = docs_dir.join("getting-started");
    std::fs::create_dir_all(&getting_started)?;

    std::fs::write(
        getting_started.join("index.md"),
        r#"---
title: Getting Started
category: getting-started
---

# Getting Started with OxideKit

Welcome to OxideKit! This guide will help you create your first application.

## Prerequisites

Before you begin, make sure you have:
- Rust installed (1.75 or later)
- OxideKit CLI installed (`cargo install oxide-cli`)

## Creating a New Project

Run the following command to create a new project:

```bash
oxide new my-app
```

## Running Your App

Navigate to your project and start the development server:

```bash
cd my-app
oxide dev
```

Your application is now running!

## Next Steps

- Explore the [Component Guide](/guides/components)
- Read about [State Management](/concepts/state)
- Browse [Example Projects](/examples)
"#,
    )?;

    // Create concepts directory
    let concepts = docs_dir.join("concepts");
    std::fs::create_dir_all(&concepts)?;

    std::fs::write(
        concepts.join("architecture.md"),
        r#"---
title: Architecture Overview
category: concepts
---

# OxideKit Architecture

OxideKit is built on a modern, modular architecture.

## Core Components

- **Renderer**: GPU-accelerated rendering using wgpu
- **Layout Engine**: Flexbox-based layout using Taffy
- **Component System**: Reactive component model
- **CLI Tools**: Project management and development

## Design Principles

1. **Native Performance**: Rust all the way down
2. **Small Footprint**: Minimal dependencies
3. **Developer Experience**: Fast iteration cycles
4. **Cross-Platform**: One codebase, multiple targets
"#,
    )?;

    println!("Created default documentation structure at {:?}", docs_dir);
    Ok(())
}
