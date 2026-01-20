//! Changelog generation from git history
//!
//! Supports conventional commits and automatic categorization of changes.

use crate::config::ReleaseConfig;
use crate::error::{ReleaseError, ReleaseResult};
use git2::{Commit, Repository};
use regex::Regex;
use std::collections::HashMap;
use std::path::Path;

/// Changelog configuration
#[derive(Debug, Clone)]
pub struct ChangelogConfig {
    /// Include commit hashes
    pub include_hashes: bool,
    /// Include author names
    pub include_authors: bool,
    /// Maximum number of commits to include
    pub max_commits: Option<usize>,
    /// Categories to include (empty = all)
    pub categories: Vec<ChangeCategory>,
    /// Start from this tag/commit
    pub from: Option<String>,
    /// End at this tag/commit
    pub to: Option<String>,
}

impl Default for ChangelogConfig {
    fn default() -> Self {
        Self {
            include_hashes: true,
            include_authors: false,
            max_commits: None,
            categories: vec![],
            from: None,
            to: None,
        }
    }
}

/// Change category based on conventional commits
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ChangeCategory {
    /// New features (feat)
    Features,
    /// Bug fixes (fix)
    Fixes,
    /// Documentation (docs)
    Documentation,
    /// Performance (perf)
    Performance,
    /// Refactoring (refactor)
    Refactoring,
    /// Tests (test)
    Tests,
    /// Build/CI (build, ci)
    Build,
    /// Chores (chore)
    Chores,
    /// Breaking changes
    Breaking,
    /// Other/uncategorized
    Other,
}

impl ChangeCategory {
    /// Get the display name for the category
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Features => "Features",
            Self::Fixes => "Bug Fixes",
            Self::Documentation => "Documentation",
            Self::Performance => "Performance",
            Self::Refactoring => "Refactoring",
            Self::Tests => "Tests",
            Self::Build => "Build & CI",
            Self::Chores => "Chores",
            Self::Breaking => "Breaking Changes",
            Self::Other => "Other",
        }
    }

    /// Get the emoji for the category
    pub fn emoji(&self) -> &'static str {
        match self {
            Self::Features => "",
            Self::Fixes => "",
            Self::Documentation => "",
            Self::Performance => "",
            Self::Refactoring => "",
            Self::Tests => "",
            Self::Build => "",
            Self::Chores => "",
            Self::Breaking => "",
            Self::Other => "",
        }
    }

    /// Parse category from conventional commit prefix
    pub fn from_prefix(prefix: &str) -> Self {
        match prefix.to_lowercase().as_str() {
            "feat" | "feature" => Self::Features,
            "fix" | "bugfix" => Self::Fixes,
            "docs" | "doc" => Self::Documentation,
            "perf" | "performance" => Self::Performance,
            "refactor" => Self::Refactoring,
            "test" | "tests" => Self::Tests,
            "build" | "ci" => Self::Build,
            "chore" | "chores" => Self::Chores,
            _ => Self::Other,
        }
    }
}

/// A changelog entry
#[derive(Debug, Clone)]
pub struct ChangelogEntry {
    /// Commit message
    pub message: String,
    /// Short description (first line)
    pub summary: String,
    /// Category
    pub category: ChangeCategory,
    /// Scope (optional, from conventional commits)
    pub scope: Option<String>,
    /// Is this a breaking change?
    pub breaking: bool,
    /// Commit hash (short)
    pub hash: String,
    /// Author name
    pub author: String,
    /// Commit date
    pub date: chrono::DateTime<chrono::Utc>,
}

impl ChangelogEntry {
    /// Parse from a git commit
    pub fn from_commit(commit: &Commit) -> ReleaseResult<Self> {
        let message = commit.message().unwrap_or("").to_string();
        let summary = message.lines().next().unwrap_or("").to_string();
        let hash = commit.id().to_string()[..7].to_string();
        let author = commit.author().name().unwrap_or("Unknown").to_string();

        // Parse time
        let timestamp = commit.time().seconds();
        let date = chrono::DateTime::from_timestamp(timestamp, 0)
            .unwrap_or_else(chrono::Utc::now);

        // Parse conventional commit format: type(scope): description
        let (category, scope, breaking) = parse_conventional_commit(&summary);

        Ok(Self {
            message,
            summary,
            category,
            scope,
            breaking,
            hash,
            author,
            date,
        })
    }

    /// Format as markdown
    pub fn to_markdown(&self, include_hash: bool, include_author: bool) -> String {
        let mut parts = vec![];

        // Scope prefix
        if let Some(ref scope) = self.scope {
            parts.push(format!("**{}:**", scope));
        }

        // Summary (remove conventional commit prefix)
        let clean_summary = remove_conventional_prefix(&self.summary);
        parts.push(clean_summary);

        // Hash
        if include_hash {
            parts.push(format!("({})", self.hash));
        }

        // Author
        if include_author {
            parts.push(format!("by {}", self.author));
        }

        format!("- {}", parts.join(" "))
    }
}

/// Parse conventional commit format
fn parse_conventional_commit(message: &str) -> (ChangeCategory, Option<String>, bool) {
    // Pattern: type(scope)!: description or type!: description or type: description
    let re = Regex::new(r"^(\w+)(?:\(([^)]+)\))?(!)?\s*:\s*(.*)$").unwrap();

    if let Some(caps) = re.captures(message) {
        let type_str = caps.get(1).map(|m| m.as_str()).unwrap_or("");
        let scope = caps.get(2).map(|m| m.as_str().to_string());
        let breaking = caps.get(3).is_some() || message.to_lowercase().contains("breaking");
        let category = if breaking {
            ChangeCategory::Breaking
        } else {
            ChangeCategory::from_prefix(type_str)
        };

        (category, scope, breaking)
    } else {
        (ChangeCategory::Other, None, false)
    }
}

/// Remove conventional commit prefix from message
fn remove_conventional_prefix(message: &str) -> String {
    let re = Regex::new(r"^(\w+)(?:\([^)]+\))?!?\s*:\s*(.*)$").unwrap();

    if let Some(caps) = re.captures(message) {
        caps.get(4).or(caps.get(2)).map(|m| m.as_str().to_string()).unwrap_or_else(|| message.to_string())
    } else {
        message.to_string()
    }
}

/// Generate changelog from git history
pub async fn generate_changelog(config: &ReleaseConfig) -> ReleaseResult<String> {
    let changelog_config = ChangelogConfig::default();
    generate_changelog_with_config(&config.project_root, &changelog_config).await
}

/// Generate changelog with custom configuration
pub async fn generate_changelog_with_config(
    project_root: &Path,
    config: &ChangelogConfig,
) -> ReleaseResult<String> {
    let repo = Repository::open(project_root)?;
    let entries = collect_changelog_entries(&repo, config)?;

    format_changelog(&entries, config)
}

/// Collect changelog entries from git
fn collect_changelog_entries(
    repo: &Repository,
    config: &ChangelogConfig,
) -> ReleaseResult<Vec<ChangelogEntry>> {
    let mut entries = Vec::new();

    // Get the commit range
    let mut revwalk = repo.revwalk()?;

    // Start from HEAD or specified commit
    if let Some(ref to) = config.to {
        let obj = repo.revparse_single(to)?;
        revwalk.push(obj.id())?;
    } else {
        revwalk.push_head()?;
    }

    // Stop at specified commit/tag
    if let Some(ref from) = config.from {
        let obj = repo.revparse_single(from)?;
        revwalk.hide(obj.id())?;
    }

    for oid in revwalk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;

        // Skip merge commits
        if commit.parent_count() > 1 {
            continue;
        }

        let entry = ChangelogEntry::from_commit(&commit)?;

        // Filter by category if specified
        if !config.categories.is_empty() && !config.categories.contains(&entry.category) {
            continue;
        }

        entries.push(entry);

        // Check max commits
        if let Some(max) = config.max_commits {
            if entries.len() >= max {
                break;
            }
        }
    }

    Ok(entries)
}

/// Format changelog entries as markdown
fn format_changelog(
    entries: &[ChangelogEntry],
    config: &ChangelogConfig,
) -> ReleaseResult<String> {
    if entries.is_empty() {
        return Ok("No changes found.".to_string());
    }

    // Group by category
    let mut by_category: HashMap<ChangeCategory, Vec<&ChangelogEntry>> = HashMap::new();
    for entry in entries {
        by_category.entry(entry.category).or_default().push(entry);
    }

    let mut output = String::new();

    // Order categories with breaking changes first
    let category_order = [
        ChangeCategory::Breaking,
        ChangeCategory::Features,
        ChangeCategory::Fixes,
        ChangeCategory::Performance,
        ChangeCategory::Documentation,
        ChangeCategory::Refactoring,
        ChangeCategory::Tests,
        ChangeCategory::Build,
        ChangeCategory::Chores,
        ChangeCategory::Other,
    ];

    for category in category_order {
        if let Some(cat_entries) = by_category.get(&category) {
            output.push_str(&format!("\n### {}\n\n", category.display_name()));

            for entry in cat_entries {
                let line = entry.to_markdown(config.include_hashes, config.include_authors);
                output.push_str(&line);
                output.push('\n');
            }
        }
    }

    Ok(output.trim().to_string())
}

/// Get the most recent tag
pub fn get_latest_tag(repo: &Repository) -> ReleaseResult<Option<String>> {
    let tags = repo.tag_names(None)?;

    // Find latest semver tag
    let mut versions: Vec<(&str, semver::Version)> = tags
        .iter()
        .filter_map(|t| t)
        .filter_map(|t| {
            let clean = t.trim_start_matches('v');
            semver::Version::parse(clean).ok().map(|v| (t, v))
        })
        .collect();

    versions.sort_by(|a, b| b.1.cmp(&a.1));
    Ok(versions.first().map(|(t, _)| (*t).to_string()))
}

/// Get commits since a tag
pub fn get_commits_since_tag(repo: &Repository, tag: &str) -> ReleaseResult<Vec<ChangelogEntry>> {
    let config = ChangelogConfig {
        from: Some(tag.to_string()),
        ..Default::default()
    };
    collect_changelog_entries(repo, &config)
}

/// Generate release notes for a version
pub fn generate_release_notes(
    entries: &[ChangelogEntry],
    version: &str,
    date: &str,
) -> String {
    let config = ChangelogConfig::default();
    let changelog = format_changelog(entries, &config).unwrap_or_default();

    format!(
        r#"## {} ({})

{}
"#,
        version, date, changelog
    )
}

/// Append to CHANGELOG.md file
pub fn append_to_changelog_file(
    changelog_path: &Path,
    version: &str,
    entries: &[ChangelogEntry],
) -> ReleaseResult<()> {
    let date = chrono::Utc::now().format("%Y-%m-%d").to_string();
    let new_content = generate_release_notes(entries, version, &date);

    let existing = if changelog_path.exists() {
        std::fs::read_to_string(changelog_path)?
    } else {
        "# Changelog\n\nAll notable changes to this project will be documented in this file.\n".to_string()
    };

    // Insert after header
    let mut lines: Vec<&str> = existing.lines().collect();
    let insert_pos = lines
        .iter()
        .position(|l| l.starts_with("## "))
        .unwrap_or(lines.len());

    let new_lines: Vec<&str> = new_content.lines().collect();
    for (i, line) in new_lines.iter().enumerate() {
        lines.insert(insert_pos + i, line);
    }

    std::fs::write(changelog_path, lines.join("\n"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_conventional_commit() {
        let (cat, scope, breaking) = parse_conventional_commit("feat(cli): add new command");
        assert_eq!(cat, ChangeCategory::Features);
        assert_eq!(scope, Some("cli".to_string()));
        assert!(!breaking);

        let (cat, scope, breaking) = parse_conventional_commit("fix!: critical bug");
        assert_eq!(cat, ChangeCategory::Breaking);
        assert_eq!(scope, None);
        assert!(breaking);

        let (cat, scope, breaking) = parse_conventional_commit("docs: update readme");
        assert_eq!(cat, ChangeCategory::Documentation);
        assert_eq!(scope, None);
        assert!(!breaking);
    }

    #[test]
    fn test_remove_conventional_prefix() {
        assert_eq!(
            remove_conventional_prefix("feat(cli): add new command"),
            "add new command"
        );
        assert_eq!(
            remove_conventional_prefix("fix!: critical bug"),
            "critical bug"
        );
        assert_eq!(
            remove_conventional_prefix("regular commit message"),
            "regular commit message"
        );
    }

    #[test]
    fn test_category_from_prefix() {
        assert_eq!(ChangeCategory::from_prefix("feat"), ChangeCategory::Features);
        assert_eq!(ChangeCategory::from_prefix("fix"), ChangeCategory::Fixes);
        assert_eq!(ChangeCategory::from_prefix("docs"), ChangeCategory::Documentation);
        assert_eq!(ChangeCategory::from_prefix("unknown"), ChangeCategory::Other);
    }
}
