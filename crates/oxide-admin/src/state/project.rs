//! Project management state

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{info, warn};
use walkdir::WalkDir;

/// Information about an OxideKit project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectInfo {
    /// Unique project ID (from oxide.toml)
    pub id: String,

    /// Project name
    pub name: String,

    /// Project version
    pub version: String,

    /// Project description
    pub description: Option<String>,

    /// Path to project root
    pub path: PathBuf,

    /// Project status
    pub status: ProjectStatus,

    /// Last modified time
    pub last_modified: DateTime<Utc>,

    /// Project target(s)
    pub targets: Vec<ProjectTarget>,

    /// Dependencies count
    pub dependency_count: usize,

    /// Project size in bytes (approximate)
    pub size_bytes: u64,

    /// Whether the project has errors
    pub has_errors: bool,

    /// Error messages if any
    pub errors: Vec<String>,

    /// Project metadata
    pub metadata: ProjectMetadata,
}

/// Project status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ProjectStatus {
    #[default]
    Ready,
    Building,
    Running,
    Error,
    Outdated,
}

/// Project build target
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProjectTarget {
    Desktop,
    Web,
    Static,
}

/// Additional project metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectMetadata {
    /// Author information
    pub authors: Vec<String>,

    /// Repository URL
    pub repository: Option<String>,

    /// License
    pub license: Option<String>,

    /// Keywords/tags
    pub keywords: Vec<String>,

    /// Category (admin, docs, website, etc.)
    pub category: Option<String>,
}

/// Registry of discovered projects
pub struct ProjectRegistry {
    projects: HashMap<String, ProjectInfo>,
    by_path: HashMap<PathBuf, String>,
}

impl ProjectRegistry {
    /// Create a new project registry
    pub fn new() -> Self {
        Self {
            projects: HashMap::new(),
            by_path: HashMap::new(),
        }
    }

    /// Scan a directory for OxideKit projects
    pub fn scan_directory(&mut self, dir: &Path) -> Result<()> {
        if !dir.exists() {
            return Ok(());
        }

        info!("Scanning {:?} for projects", dir);

        for entry in WalkDir::new(dir)
            .max_depth(4)
            .follow_links(false)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.file_name().map(|f| f == "oxide.toml").unwrap_or(false) {
                if let Some(project_dir) = path.parent() {
                    if let Err(e) = self.add_project(project_dir) {
                        warn!("Failed to add project at {:?}: {}", project_dir, e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Add a project from its directory
    pub fn add_project(&mut self, project_dir: &Path) -> Result<()> {
        let manifest_path = project_dir.join("oxide.toml");
        if !manifest_path.exists() {
            anyhow::bail!("No oxide.toml found in {:?}", project_dir);
        }

        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: ProjectManifest = toml::from_str(&content)?;

        // Get file metadata for last modified time
        let metadata = std::fs::metadata(&manifest_path)?;
        let last_modified = metadata.modified()
            .map(|t| DateTime::<Utc>::from(t))
            .unwrap_or_else(|_| Utc::now());

        // Calculate approximate project size
        let size_bytes = calculate_dir_size(project_dir);

        // Detect targets
        let targets = detect_targets(project_dir, &manifest);

        // Count dependencies
        let dependency_count = count_dependencies(project_dir);

        let info = ProjectInfo {
            id: manifest.app.id.clone(),
            name: manifest.app.name.clone(),
            version: manifest.app.version.clone(),
            description: manifest.app.description.clone(),
            path: project_dir.to_path_buf(),
            status: ProjectStatus::Ready,
            last_modified,
            targets,
            dependency_count,
            size_bytes,
            has_errors: false,
            errors: Vec::new(),
            metadata: ProjectMetadata {
                authors: manifest.app.authors.unwrap_or_default(),
                repository: manifest.app.repository.clone(),
                license: manifest.app.license.clone(),
                keywords: manifest.app.keywords.unwrap_or_default(),
                category: manifest.app.category.clone(),
            },
        };

        let id = info.id.clone();
        let path = info.path.clone();

        self.projects.insert(id.clone(), info);
        self.by_path.insert(path, id);

        Ok(())
    }

    /// Get a project by ID
    pub fn get(&self, id: &str) -> Option<&ProjectInfo> {
        self.projects.get(id)
    }

    /// Get a project by path
    pub fn get_by_path(&self, path: &Path) -> Option<&ProjectInfo> {
        self.by_path.get(path).and_then(|id| self.projects.get(id))
    }

    /// Get all projects
    pub fn all(&self) -> Vec<&ProjectInfo> {
        self.projects.values().collect()
    }

    /// Search projects
    pub fn search(&self, query: &str) -> Vec<&ProjectInfo> {
        let query = query.to_lowercase();
        self.projects
            .values()
            .filter(|p| {
                p.name.to_lowercase().contains(&query)
                    || p.id.to_lowercase().contains(&query)
                    || p.description.as_ref().map(|d| d.to_lowercase().contains(&query)).unwrap_or(false)
            })
            .collect()
    }

    /// Get number of projects
    pub fn len(&self) -> usize {
        self.projects.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }

    /// Get count of active (running/building) projects
    pub fn active_count(&self) -> usize {
        self.projects
            .values()
            .filter(|p| matches!(p.status, ProjectStatus::Running | ProjectStatus::Building))
            .count()
    }

    /// Update project status
    pub fn update_status(&mut self, id: &str, status: ProjectStatus) {
        if let Some(project) = self.projects.get_mut(id) {
            project.status = status;
        }
    }

    /// Remove a project
    pub fn remove(&mut self, id: &str) -> Option<ProjectInfo> {
        if let Some(project) = self.projects.remove(id) {
            self.by_path.remove(&project.path);
            Some(project)
        } else {
            None
        }
    }

    /// Get projects sorted by last modified
    pub fn recent(&self, limit: usize) -> Vec<&ProjectInfo> {
        let mut projects: Vec<_> = self.projects.values().collect();
        projects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));
        projects.into_iter().take(limit).collect()
    }

    /// Get projects by status
    pub fn by_status(&self, status: ProjectStatus) -> Vec<&ProjectInfo> {
        self.projects
            .values()
            .filter(|p| p.status == status)
            .collect()
    }

    /// Get projects with errors
    pub fn with_errors(&self) -> Vec<&ProjectInfo> {
        self.projects
            .values()
            .filter(|p| p.has_errors)
            .collect()
    }
}

impl Default for ProjectRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Project manifest structure (from oxide.toml)
#[derive(Debug, Deserialize)]
struct ProjectManifest {
    app: AppSection,
    #[serde(default)]
    build: Option<BuildSection>,
}

#[derive(Debug, Deserialize)]
struct AppSection {
    id: String,
    name: String,
    version: String,
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    authors: Option<Vec<String>>,
    #[serde(default)]
    repository: Option<String>,
    #[serde(default)]
    license: Option<String>,
    #[serde(default)]
    keywords: Option<Vec<String>>,
    #[serde(default)]
    category: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
struct BuildSection {
    #[serde(default)]
    targets: Vec<String>,
}

/// Calculate approximate directory size
fn calculate_dir_size(dir: &Path) -> u64 {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

/// Detect project targets
fn detect_targets(dir: &Path, manifest: &ProjectManifest) -> Vec<ProjectTarget> {
    let mut targets = Vec::new();

    // Check manifest build targets
    if let Some(build) = &manifest.build {
        for target in &build.targets {
            match target.as_str() {
                "desktop" => targets.push(ProjectTarget::Desktop),
                "web" | "wasm" => targets.push(ProjectTarget::Web),
                "static" => targets.push(ProjectTarget::Static),
                _ => {}
            }
        }
    }

    // Default to desktop if no targets specified
    if targets.is_empty() {
        targets.push(ProjectTarget::Desktop);
    }

    // Also check for presence of target-specific files
    if dir.join("web").exists() || dir.join("public").exists() {
        if !targets.contains(&ProjectTarget::Web) {
            targets.push(ProjectTarget::Web);
        }
    }

    targets
}

/// Count dependencies from Cargo.toml
fn count_dependencies(dir: &Path) -> usize {
    let cargo_path = dir.join("Cargo.toml");
    if !cargo_path.exists() {
        return 0;
    }

    if let Ok(content) = std::fs::read_to_string(&cargo_path) {
        // Simple count of lines containing dependencies
        content.lines()
            .filter(|line| {
                let trimmed = line.trim();
                trimmed.contains("=") && !trimmed.starts_with("#") && !trimmed.starts_with("[")
            })
            .count()
    } else {
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_registry() {
        let registry = ProjectRegistry::new();
        assert!(registry.is_empty());
        assert_eq!(registry.len(), 0);
    }

    #[test]
    fn test_project_status_default() {
        assert_eq!(ProjectStatus::default(), ProjectStatus::Ready);
    }
}
