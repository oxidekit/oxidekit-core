//! Tutorial definition and loading

use crate::types::Difficulty;
use crate::DocsResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

use super::TutorialStep;

/// An interactive tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tutorial {
    /// Unique identifier
    pub id: String,
    /// Display title
    pub title: String,
    /// Short description
    pub description: String,
    /// Long description (markdown)
    pub long_description: Option<String>,
    /// Difficulty level
    pub difficulty: Difficulty,
    /// Estimated time to complete (in minutes)
    pub estimated_minutes: u32,
    /// Prerequisites (other tutorial IDs)
    pub prerequisites: Vec<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Tutorial steps
    pub steps: Vec<TutorialStep>,
    /// Whether this tutorial can run completely offline
    pub offline_capable: bool,
    /// Example project to create (optional)
    pub example_project: Option<ExampleProject>,
    /// Author
    pub author: Option<String>,
    /// Last updated date
    pub updated: Option<String>,
}

/// Example project definition for a tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExampleProject {
    /// Project name
    pub name: String,
    /// Project template to use
    pub template: String,
    /// Files to create/modify
    pub files: Vec<ProjectFile>,
}

/// A file in an example project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectFile {
    /// File path relative to project root
    pub path: String,
    /// File content
    pub content: String,
    /// Optional description
    pub description: Option<String>,
}

impl Tutorial {
    /// Create a new tutorial
    pub fn new(id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: String::new(),
            long_description: None,
            difficulty: Difficulty::Beginner,
            estimated_minutes: 10,
            prerequisites: Vec::new(),
            tags: Vec::new(),
            steps: Vec::new(),
            offline_capable: true,
            example_project: None,
            author: None,
            updated: None,
        }
    }

    /// Set the description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Set the difficulty
    pub fn difficulty(mut self, difficulty: Difficulty) -> Self {
        self.difficulty = difficulty;
        self
    }

    /// Add a step
    pub fn add_step(mut self, step: TutorialStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Load a tutorial from a TOML or JSON file
    pub fn load(path: &Path) -> DocsResult<Self> {
        let content = fs::read_to_string(path)?;

        let tutorial: Self = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::from_str(&content)?
        } else {
            toml::from_str(&content)?
        };

        Ok(tutorial)
    }

    /// Save the tutorial to a file
    pub fn save(&self, path: &Path) -> DocsResult<()> {
        let content = if path.extension().map_or(false, |ext| ext == "json") {
            serde_json::to_string_pretty(self)?
        } else {
            toml::to_string_pretty(self)?
        };

        fs::write(path, content)?;
        Ok(())
    }

    /// List all tutorials in a directory
    pub fn list_in_directory(dir: &Path) -> DocsResult<Vec<Self>> {
        let mut tutorials = Vec::new();

        if !dir.exists() {
            return Ok(tutorials);
        }

        for entry in WalkDir::new(dir)
            .max_depth(2)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            let is_tutorial = path
                .file_name()
                .map_or(false, |name| {
                    let name = name.to_string_lossy();
                    name == "tutorial.toml" || name == "tutorial.json"
                });

            if is_tutorial {
                if let Ok(tutorial) = Self::load(path) {
                    tutorials.push(tutorial);
                }
            }
        }

        // Sort by difficulty then title
        tutorials.sort_by(|a, b| {
            a.difficulty.cmp(&b.difficulty)
                .then_with(|| a.title.cmp(&b.title))
        });

        Ok(tutorials)
    }

    /// Get the total number of steps
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Check if all prerequisites are met
    pub fn check_prerequisites(&self, completed: &[String]) -> Vec<String> {
        self.prerequisites
            .iter()
            .filter(|p| !completed.contains(p))
            .cloned()
            .collect()
    }

    /// Get a summary of the tutorial
    pub fn summary(&self) -> TutorialSummary {
        TutorialSummary {
            id: self.id.clone(),
            title: self.title.clone(),
            description: self.description.clone(),
            difficulty: self.difficulty,
            estimated_minutes: self.estimated_minutes,
            step_count: self.steps.len(),
            tags: self.tags.clone(),
        }
    }
}

/// Summary information about a tutorial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TutorialSummary {
    pub id: String,
    pub title: String,
    pub description: String,
    pub difficulty: Difficulty,
    pub estimated_minutes: u32,
    pub step_count: usize,
    pub tags: Vec<String>,
}

/// Built-in tutorial templates
pub fn getting_started_tutorial() -> Tutorial {
    Tutorial::new("getting-started", "Getting Started with OxideKit")
        .description("Learn the basics of OxideKit by building your first application.")
        .difficulty(Difficulty::Beginner)
        .add_step(TutorialStep::new(
            "introduction",
            "Introduction to OxideKit",
            r#"# Welcome to OxideKit!

OxideKit is a Rust-native application platform that lets you build fast, lightweight desktop applications.

In this tutorial, you'll learn:
- How to create a new OxideKit project
- The basic project structure
- How to build and run your app

Let's get started!"#,
        ))
        .add_step(TutorialStep::new(
            "create-project",
            "Create Your First Project",
            r#"# Creating a Project

Run the following command to create a new OxideKit project:

```bash
oxide new my-first-app
```

This will create a new directory called `my-first-app` with the basic project structure."#,
        ).with_action(super::StepAction::RunCommand {
            command: "oxide new my-first-app".to_string(),
            working_dir: None,
        }))
        .add_step(TutorialStep::new(
            "explore-structure",
            "Explore the Project Structure",
            r#"# Project Structure

Your new project has the following structure:

```
my-first-app/
├── Cargo.toml        # Rust dependencies
├── Oxide.toml        # OxideKit configuration
├── src/
│   └── main.rs       # Application entry point
└── assets/           # Static assets (images, fonts, etc.)
```

The most important file is `src/main.rs` which contains your application code."#,
        ))
        .add_step(TutorialStep::new(
            "run-app",
            "Run Your Application",
            r#"# Running the App

Navigate to your project directory and run:

```bash
cd my-first-app
oxide dev
```

This will start the development server with hot reload enabled.

You should see a window appear with your application!"#,
        ).with_action(super::StepAction::RunCommand {
            command: "oxide dev".to_string(),
            working_dir: Some("my-first-app".to_string()),
        }))
        .add_step(TutorialStep::new(
            "conclusion",
            "Congratulations!",
            r#"# You Did It!

You've successfully created and run your first OxideKit application.

## Next Steps

- Explore the [Component Guide](/guides/components) to learn about UI components
- Check out the [Styling Guide](/guides/styling) for customizing appearance
- Browse [Example Projects](/examples) for inspiration

Happy coding!"#,
        ))
}

pub fn component_tutorial() -> Tutorial {
    Tutorial::new("components-basics", "Understanding OxideKit Components")
        .description("Learn how to use and create components in OxideKit.")
        .difficulty(Difficulty::Beginner)
        .add_step(TutorialStep::new(
            "what-are-components",
            "What Are Components?",
            r#"# Components in OxideKit

Components are the building blocks of OxideKit applications. They are reusable pieces of UI that can contain:
- Visual elements (text, images, buttons)
- Layout structure
- Interactive behavior
- Internal state

Think of components like LEGO blocks - you combine them to build your application."#,
        ))
        .add_step(TutorialStep::new(
            "built-in-components",
            "Built-in Components",
            r#"# Built-in Components

OxideKit provides several built-in components:

| Component | Description |
|-----------|-------------|
| `View` | Basic container for layout |
| `Text` | Display text content |
| `Button` | Clickable button |
| `Image` | Display images |
| `Input` | Text input field |
| `List` | Scrollable list |

Let's see how to use them!"#,
        ))
        .add_step(TutorialStep::new(
            "using-components",
            "Using Components",
            r#"# Using Components

Here's a simple example using basic components:

```rust
use oxide::prelude::*;

fn app() -> impl Component {
    View::new()
        .child(Text::new("Hello, OxideKit!"))
        .child(Button::new("Click Me")
            .on_click(|| println!("Button clicked!")))
}
```

Components are composed by nesting them inside each other."#,
        ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tutorial_creation() {
        let tutorial = Tutorial::new("test", "Test Tutorial")
            .description("A test tutorial")
            .difficulty(Difficulty::Beginner);

        assert_eq!(tutorial.id, "test");
        assert_eq!(tutorial.title, "Test Tutorial");
        assert_eq!(tutorial.difficulty, Difficulty::Beginner);
    }

    #[test]
    fn test_getting_started_tutorial() {
        let tutorial = getting_started_tutorial();
        assert!(!tutorial.steps.is_empty());
        assert_eq!(tutorial.difficulty, Difficulty::Beginner);
    }
}
