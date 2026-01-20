//! Prompt Templates for AI Assistants
//!
//! Pre-built prompts for common OxideKit tasks.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A prompt template for AI assistants
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub template: String,
    pub variables: Vec<PromptVariable>,
    pub category: PromptCategory,
}

impl PromptTemplate {
    /// Create a new prompt template
    pub fn new(id: &str, name: &str, template: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            description: String::new(),
            template: template.to_string(),
            variables: Vec::new(),
            category: PromptCategory::General,
        }
    }

    /// Set the description
    pub fn with_description(mut self, desc: &str) -> Self {
        self.description = desc.to_string();
        self
    }

    /// Add a variable
    pub fn with_variable(mut self, var: PromptVariable) -> Self {
        self.variables.push(var);
        self
    }

    /// Render the template with variables
    pub fn render(&self, vars: &HashMap<String, String>) -> String {
        let mut result = self.template.clone();
        for (key, value) in vars {
            result = result.replace(&format!("{{{{{}}}}}", key), value);
        }
        result
    }
}

/// A variable in a prompt template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptVariable {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default: Option<String>,
}

/// Prompt category
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum PromptCategory {
    General,
    ComponentCreation,
    Styling,
    Layout,
    DataBinding,
    Debugging,
    Migration,
}

/// Library of prompt templates
#[derive(Debug, Clone, Default)]
pub struct PromptLibrary {
    templates: HashMap<String, PromptTemplate>,
}

impl PromptLibrary {
    /// Create a new prompt library
    pub fn new() -> Self {
        Self::default()
    }

    /// Create with built-in templates
    pub fn with_builtins() -> Self {
        let mut lib = Self::new();

        lib.add(PromptTemplate::new(
            "create-component",
            "Create Component",
            "Create a new OxideKit component named {{name}} with the following properties:\n{{props}}\n\nUse the oxide.ai.json schema to validate.",
        ).with_description("Create a new UI component"));

        lib.add(PromptTemplate::new(
            "style-component",
            "Style Component",
            "Apply styling to the component {{component}} using design tokens:\n- Colors: {{colors}}\n- Spacing: {{spacing}}",
        ).with_description("Apply design tokens to a component"));

        lib.add(PromptTemplate::new(
            "debug-layout",
            "Debug Layout",
            "The following .oui file has layout issues:\n```oui\n{{code}}\n```\n\nAnalyze and suggest fixes.",
        ).with_description("Debug layout issues"));

        lib
    }

    /// Add a template
    pub fn add(&mut self, template: PromptTemplate) {
        self.templates.insert(template.id.clone(), template);
    }

    /// Get a template by ID
    pub fn get(&self, id: &str) -> Option<&PromptTemplate> {
        self.templates.get(id)
    }

    /// List all templates
    pub fn list(&self) -> impl Iterator<Item = &PromptTemplate> {
        self.templates.values()
    }

    /// List templates by category
    pub fn by_category(&self, category: PromptCategory) -> impl Iterator<Item = &PromptTemplate> {
        self.templates.values().filter(move |t| t.category == category)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_render() {
        let template = PromptTemplate::new("test", "Test", "Hello {{name}}!");
        let mut vars = HashMap::new();
        vars.insert("name".to_string(), "World".to_string());
        assert_eq!(template.render(&vars), "Hello World!");
    }
}
