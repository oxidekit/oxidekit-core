//! Version migration helpers
//!
//! Provides tools for migrating between versions with guided instructions.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use crate::semver::Version;

/// A migration guide between two versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationGuide {
    /// Source version
    pub from_version: Version,
    /// Target version
    pub to_version: Version,
    /// Title of the migration guide
    pub title: String,
    /// Overview/summary
    pub summary: String,
    /// Estimated time to complete (in minutes)
    pub estimated_time: Option<u32>,
    /// Prerequisites before starting
    #[serde(default)]
    pub prerequisites: Vec<String>,
    /// Migration steps
    pub steps: Vec<MigrationStep>,
    /// Post-migration verification steps
    #[serde(default)]
    pub verification: Vec<String>,
    /// Common issues and solutions
    #[serde(default)]
    pub troubleshooting: Vec<TroubleshootingEntry>,
    /// Additional resources
    #[serde(default)]
    pub resources: Vec<Resource>,
}

impl MigrationGuide {
    /// Create a new migration guide
    pub fn new(
        from: Version,
        to: Version,
        title: impl Into<String>,
        summary: impl Into<String>,
    ) -> Self {
        Self {
            from_version: from,
            to_version: to,
            title: title.into(),
            summary: summary.into(),
            estimated_time: None,
            prerequisites: Vec::new(),
            steps: Vec::new(),
            verification: Vec::new(),
            troubleshooting: Vec::new(),
            resources: Vec::new(),
        }
    }

    /// Set estimated time
    pub fn with_time(mut self, minutes: u32) -> Self {
        self.estimated_time = Some(minutes);
        self
    }

    /// Add a prerequisite
    pub fn add_prerequisite(&mut self, prereq: impl Into<String>) {
        self.prerequisites.push(prereq.into());
    }

    /// Add a migration step
    pub fn add_step(&mut self, step: MigrationStep) {
        self.steps.push(step);
    }

    /// Add a verification step
    pub fn add_verification(&mut self, check: impl Into<String>) {
        self.verification.push(check.into());
    }

    /// Add troubleshooting entry
    pub fn add_troubleshooting(&mut self, entry: TroubleshootingEntry) {
        self.troubleshooting.push(entry);
    }

    /// Add a resource
    pub fn add_resource(&mut self, resource: Resource) {
        self.resources.push(resource);
    }

    /// Get total step count
    pub fn step_count(&self) -> usize {
        self.steps.len()
    }

    /// Calculate completion percentage
    pub fn completion_percentage(&self, completed_steps: usize) -> f32 {
        if self.steps.is_empty() {
            return 100.0;
        }
        (completed_steps as f32 / self.steps.len() as f32) * 100.0
    }

    /// Format as markdown
    pub fn to_markdown(&self) -> String {
        let mut md = String::new();

        // Header
        md.push_str(&format!("# {}\n\n", self.title));
        md.push_str(&format!(
            "**Migration:** {} -> {}\n\n",
            self.from_version, self.to_version
        ));

        if let Some(time) = self.estimated_time {
            md.push_str(&format!("**Estimated time:** {} minutes\n\n", time));
        }

        // Summary
        md.push_str("## Overview\n\n");
        md.push_str(&self.summary);
        md.push_str("\n\n");

        // Prerequisites
        if !self.prerequisites.is_empty() {
            md.push_str("## Prerequisites\n\n");
            for prereq in &self.prerequisites {
                md.push_str(&format!("- [ ] {}\n", prereq));
            }
            md.push_str("\n");
        }

        // Steps
        md.push_str("## Migration Steps\n\n");
        for (i, step) in self.steps.iter().enumerate() {
            md.push_str(&format!("### Step {}: {}\n\n", i + 1, step.title));

            if let Some(ref desc) = step.description {
                md.push_str(desc);
                md.push_str("\n\n");
            }

            if !step.actions.is_empty() {
                for action in &step.actions {
                    md.push_str(&format!("- [ ] {}\n", action));
                }
                md.push_str("\n");
            }

            if let Some(ref code) = step.code_example {
                md.push_str("**Example:**\n\n");
                md.push_str(&format!("```{}\n", code.language));
                md.push_str(&code.code);
                md.push_str("\n```\n\n");
            }

            if !step.warnings.is_empty() {
                md.push_str("> **Warnings:**\n");
                for warning in &step.warnings {
                    md.push_str(&format!("> - {}\n", warning));
                }
                md.push_str("\n");
            }
        }

        // Verification
        if !self.verification.is_empty() {
            md.push_str("## Verification\n\n");
            md.push_str("After completing the migration, verify:\n\n");
            for check in &self.verification {
                md.push_str(&format!("- [ ] {}\n", check));
            }
            md.push_str("\n");
        }

        // Troubleshooting
        if !self.troubleshooting.is_empty() {
            md.push_str("## Troubleshooting\n\n");
            for entry in &self.troubleshooting {
                md.push_str(&format!("### {}\n\n", entry.problem));
                md.push_str(&format!("**Solution:** {}\n\n", entry.solution));
            }
        }

        // Resources
        if !self.resources.is_empty() {
            md.push_str("## Resources\n\n");
            for resource in &self.resources {
                md.push_str(&format!("- [{}]({})", resource.title, resource.url));
                if let Some(ref desc) = resource.description {
                    md.push_str(&format!(" - {}", desc));
                }
                md.push_str("\n");
            }
        }

        md
    }
}

/// A single migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Step title
    pub title: String,
    /// Detailed description
    pub description: Option<String>,
    /// Action items
    #[serde(default)]
    pub actions: Vec<String>,
    /// Code example
    pub code_example: Option<CodeExample>,
    /// Warnings
    #[serde(default)]
    pub warnings: Vec<String>,
    /// Whether this step is optional
    #[serde(default)]
    pub optional: bool,
    /// Category (e.g., "api", "config", "dependencies")
    pub category: Option<String>,
}

impl MigrationStep {
    /// Create a new step
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            description: None,
            actions: Vec::new(),
            code_example: None,
            warnings: Vec::new(),
            optional: false,
            category: None,
        }
    }

    /// Set description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }

    /// Add action
    pub fn add_action(&mut self, action: impl Into<String>) {
        self.actions.push(action.into());
    }

    /// Set code example
    pub fn with_example(mut self, example: CodeExample) -> Self {
        self.code_example = Some(example);
        self
    }

    /// Add warning
    pub fn add_warning(&mut self, warning: impl Into<String>) {
        self.warnings.push(warning.into());
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Set category
    pub fn with_category(mut self, category: impl Into<String>) -> Self {
        self.category = Some(category.into());
        self
    }
}

/// A code example
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeExample {
    /// Language (rust, toml, json, etc.)
    pub language: String,
    /// The code
    pub code: String,
    /// Optional "before" code for comparison
    pub before: Option<String>,
}

impl CodeExample {
    /// Create a Rust code example
    pub fn rust(code: impl Into<String>) -> Self {
        Self {
            language: "rust".to_string(),
            code: code.into(),
            before: None,
        }
    }

    /// Create a TOML example
    pub fn toml(code: impl Into<String>) -> Self {
        Self {
            language: "toml".to_string(),
            code: code.into(),
            before: None,
        }
    }

    /// Add "before" code for comparison
    pub fn with_before(mut self, before: impl Into<String>) -> Self {
        self.before = Some(before.into());
        self
    }
}

/// A troubleshooting entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroubleshootingEntry {
    /// Problem description
    pub problem: String,
    /// Solution
    pub solution: String,
    /// Related error messages
    #[serde(default)]
    pub error_messages: Vec<String>,
}

impl TroubleshootingEntry {
    /// Create a new entry
    pub fn new(problem: impl Into<String>, solution: impl Into<String>) -> Self {
        Self {
            problem: problem.into(),
            solution: solution.into(),
            error_messages: Vec::new(),
        }
    }

    /// Add related error message
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.error_messages.push(error.into());
        self
    }
}

/// A resource link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Resource {
    /// Resource title
    pub title: String,
    /// URL
    pub url: String,
    /// Optional description
    pub description: Option<String>,
}

impl Resource {
    /// Create a new resource
    pub fn new(title: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            url: url.into(),
            description: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = Some(desc.into());
        self
    }
}

/// A collection of migration guides
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Guides for sequential migration
    guides: Vec<MigrationGuide>,
}

impl MigrationPlan {
    /// Create an empty plan
    pub fn new() -> Self {
        Self { guides: Vec::new() }
    }

    /// Add a guide
    pub fn add_guide(&mut self, guide: MigrationGuide) {
        self.guides.push(guide);
    }

    /// Find a migration path between two versions
    pub fn find_path(&self, from: &Version, to: &Version) -> Vec<&MigrationGuide> {
        // Simple case: direct path
        let direct = self.guides.iter().find(|g| {
            &g.from_version == from && &g.to_version == to
        });

        if let Some(guide) = direct {
            return vec![guide];
        }

        // Find multi-step path
        let mut path = Vec::new();
        let mut current = from.clone();

        while &current < to {
            // Find next step
            let next_guide = self.guides.iter()
                .filter(|g| &g.from_version == &current && &g.to_version <= to)
                .max_by_key(|g| &g.to_version);

            match next_guide {
                Some(guide) => {
                    current = guide.to_version.clone();
                    path.push(guide);
                }
                None => break, // No path found
            }
        }

        if &current == to {
            path
        } else {
            Vec::new() // No complete path
        }
    }

    /// Get total estimated time for a migration path
    pub fn total_time(&self, path: &[&MigrationGuide]) -> Option<u32> {
        let times: Vec<u32> = path.iter()
            .filter_map(|g| g.estimated_time)
            .collect();

        if times.len() == path.len() {
            Some(times.iter().sum())
        } else {
            None
        }
    }

    /// Get total steps for a migration path
    pub fn total_steps(&self, path: &[&MigrationGuide]) -> usize {
        path.iter().map(|g| g.step_count()).sum()
    }

    /// Generate a combined markdown guide for a path
    pub fn path_to_markdown(&self, path: &[&MigrationGuide]) -> String {
        if path.is_empty() {
            return "No migration path found.".to_string();
        }

        let mut md = String::new();

        // Overview
        let from = &path[0].from_version;
        let to = &path[path.len() - 1].to_version;

        md.push_str(&format!("# Migration Path: {} -> {}\n\n", from, to));
        md.push_str(&format!("This migration consists of {} step(s):\n\n", path.len()));

        for (i, guide) in path.iter().enumerate() {
            md.push_str(&format!(
                "{}. {} -> {}: {}\n",
                i + 1, guide.from_version, guide.to_version, guide.title
            ));
        }
        md.push_str("\n---\n\n");

        // Individual guides
        for (i, guide) in path.iter().enumerate() {
            md.push_str(&format!("# Part {}: {}\n\n", i + 1, guide.title));
            md.push_str(&guide.to_markdown());
            md.push_str("\n---\n\n");
        }

        md
    }
}

/// Migration status tracker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStatus {
    /// Guide being followed
    pub guide_title: String,
    /// From version
    pub from_version: Version,
    /// To version
    pub to_version: Version,
    /// Completed steps (by index)
    pub completed_steps: Vec<usize>,
    /// Started timestamp
    pub started_at: chrono::DateTime<chrono::Utc>,
    /// Last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,
    /// Notes
    #[serde(default)]
    pub notes: HashMap<usize, String>,
}

impl MigrationStatus {
    /// Create a new status tracker
    pub fn new(guide: &MigrationGuide) -> Self {
        let now = chrono::Utc::now();
        Self {
            guide_title: guide.title.clone(),
            from_version: guide.from_version.clone(),
            to_version: guide.to_version.clone(),
            completed_steps: Vec::new(),
            started_at: now,
            updated_at: now,
            notes: HashMap::new(),
        }
    }

    /// Mark a step as completed
    pub fn complete_step(&mut self, step_index: usize) {
        if !self.completed_steps.contains(&step_index) {
            self.completed_steps.push(step_index);
            self.completed_steps.sort();
        }
        self.updated_at = chrono::Utc::now();
    }

    /// Add a note for a step
    pub fn add_note(&mut self, step_index: usize, note: impl Into<String>) {
        self.notes.insert(step_index, note.into());
        self.updated_at = chrono::Utc::now();
    }

    /// Check if migration is complete
    pub fn is_complete(&self, total_steps: usize) -> bool {
        self.completed_steps.len() >= total_steps
    }

    /// Get next pending step index
    pub fn next_step(&self, total_steps: usize) -> Option<usize> {
        for i in 0..total_steps {
            if !self.completed_steps.contains(&i) {
                return Some(i);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_guide() -> MigrationGuide {
        let mut guide = MigrationGuide::new(
            Version::parse("0.5.0").unwrap(),
            Version::parse("0.6.0").unwrap(),
            "OxideKit 0.5 to 0.6 Migration",
            "This guide covers the migration from version 0.5 to 0.6",
        )
        .with_time(30);

        guide.add_prerequisite("Backup your project");
        guide.add_prerequisite("Update Rust to 1.75+");

        let mut step1 = MigrationStep::new("Update dependencies")
            .with_description("Update your Cargo.toml to use the new version")
            .with_category("dependencies");
        step1.add_action("Change oxide-kit = \"0.5\" to oxide-kit = \"0.6\"");
        step1.add_action("Run cargo update");
        guide.add_step(step1);

        let mut step2 = MigrationStep::new("Update API calls")
            .with_description("Some APIs have changed in this version")
            .with_example(CodeExample::rust(
                "// New API\nwidget.draw(ctx);"
            ).with_before("// Old API\nwidget.render(ctx);"));
        step2.add_warning("The render() method is completely removed");
        guide.add_step(step2);

        guide.add_verification("Run cargo build");
        guide.add_verification("Run cargo test");

        guide.add_troubleshooting(TroubleshootingEntry::new(
            "Compilation error about missing render method",
            "Replace all .render() calls with .draw()",
        ).with_error("method `render` not found"));

        guide.add_resource(Resource::new(
            "Full Changelog",
            "https://docs.oxidekit.com/changelog/0.6",
        ));

        guide
    }

    #[test]
    fn test_migration_guide_creation() {
        let guide = create_test_guide();
        assert_eq!(guide.step_count(), 2);
        assert_eq!(guide.estimated_time, Some(30));
    }

    #[test]
    fn test_completion_percentage() {
        let guide = create_test_guide();
        assert_eq!(guide.completion_percentage(0), 0.0);
        assert_eq!(guide.completion_percentage(1), 50.0);
        assert_eq!(guide.completion_percentage(2), 100.0);
    }

    #[test]
    fn test_markdown_generation() {
        let guide = create_test_guide();
        let md = guide.to_markdown();

        assert!(md.contains("# OxideKit 0.5 to 0.6 Migration"));
        assert!(md.contains("## Prerequisites"));
        assert!(md.contains("## Migration Steps"));
        assert!(md.contains("## Verification"));
        assert!(md.contains("## Troubleshooting"));
    }

    #[test]
    fn test_migration_plan() {
        let mut plan = MigrationPlan::new();
        plan.add_guide(create_test_guide());

        let from = Version::parse("0.5.0").unwrap();
        let to = Version::parse("0.6.0").unwrap();

        let path = plan.find_path(&from, &to);
        assert_eq!(path.len(), 1);
        assert_eq!(plan.total_steps(&path), 2);
    }

    #[test]
    fn test_migration_status() {
        let guide = create_test_guide();
        let mut status = MigrationStatus::new(&guide);

        assert!(!status.is_complete(2));
        assert_eq!(status.next_step(2), Some(0));

        status.complete_step(0);
        assert_eq!(status.next_step(2), Some(1));

        status.complete_step(1);
        assert!(status.is_complete(2));
        assert_eq!(status.next_step(2), None);
    }
}
