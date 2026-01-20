//! AI Context Provider
//!
//! Provides project context for AI assistants.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// AI context for the current project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiContext {
    pub project: ProjectContext,
    pub components: Vec<String>,
    pub plugins: Vec<String>,
    pub themes: Vec<String>,
}

impl AiContext {
    /// Create a new AI context
    pub fn new(project: ProjectContext) -> Self {
        Self {
            project,
            components: Vec::new(),
            plugins: Vec::new(),
            themes: Vec::new(),
        }
    }

    /// Add a component to the context
    pub fn add_component(&mut self, name: String) {
        self.components.push(name);
    }

    /// Add a plugin to the context
    pub fn add_plugin(&mut self, name: String) {
        self.plugins.push(name);
    }

    /// Add a theme to the context
    pub fn add_theme(&mut self, name: String) {
        self.themes.push(name);
    }
}

/// Project context information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectContext {
    pub name: String,
    pub version: String,
    pub root: PathBuf,
    pub oxide_version: String,
}

impl ProjectContext {
    /// Create a new project context
    pub fn new(name: String, version: String, root: PathBuf) -> Self {
        Self {
            name,
            version,
            root,
            oxide_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Load project context from oxide.toml
    pub fn from_manifest(path: &std::path::Path) -> Result<Self, std::io::Error> {
        let content = std::fs::read_to_string(path)?;
        // Simple parsing - in production would use toml crate
        let name = content
            .lines()
            .find(|l| l.starts_with("name"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let version = content
            .lines()
            .find(|l| l.starts_with("version"))
            .and_then(|l| l.split('=').nth(1))
            .map(|s| s.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| "0.1.0".to_string());

        Ok(Self::new(name, version, path.parent().unwrap_or(path).to_path_buf()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let project = ProjectContext::new("test".to_string(), "1.0.0".to_string(), PathBuf::from("."));
        let context = AiContext::new(project);
        assert!(context.components.is_empty());
    }
}
