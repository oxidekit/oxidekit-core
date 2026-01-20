//! Environment schema types and validation
//!
//! This module provides a schema-driven system for defining and validating
//! environment variables used in OxideKit applications.

use crate::error::{DeployError, DeployResult, ValidationCategory, ValidationError, ValidationErrors};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Type of an environment variable
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EnvVarType {
    /// String value
    String,
    /// Integer value
    Int,
    /// Boolean value (true/false, 1/0, yes/no)
    Bool,
    /// URL value
    Url,
    /// Enumeration with allowed values
    Enum,
    /// Port number (1-65535)
    Port,
    /// File path
    Path,
}

impl Default for EnvVarType {
    fn default() -> Self {
        Self::String
    }
}

impl std::fmt::Display for EnvVarType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::String => write!(f, "string"),
            Self::Int => write!(f, "int"),
            Self::Bool => write!(f, "bool"),
            Self::Url => write!(f, "url"),
            Self::Enum => write!(f, "enum"),
            Self::Port => write!(f, "port"),
            Self::Path => write!(f, "path"),
        }
    }
}

/// Definition of a single environment variable
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVarDefinition {
    /// Variable name
    pub name: String,
    /// Variable type
    #[serde(rename = "type", default)]
    pub var_type: EnvVarType,
    /// Whether the variable is required
    #[serde(default)]
    pub required: bool,
    /// Default value if not provided
    #[serde(default)]
    pub default: Option<String>,
    /// Whether this is a secret (should never be committed)
    #[serde(default)]
    pub secret: bool,
    /// Human-readable description
    #[serde(default)]
    pub description: Option<String>,
    /// Allowed values for enum type
    #[serde(default)]
    pub allowed_values: Option<Vec<String>>,
    /// Example value for documentation
    #[serde(default)]
    pub example: Option<String>,
    /// Validation regex pattern
    #[serde(default)]
    pub pattern: Option<String>,
}

impl EnvVarDefinition {
    /// Create a new required string environment variable
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            var_type: EnvVarType::String,
            required: true,
            default: None,
            secret: false,
            description: None,
            allowed_values: None,
            example: None,
            pattern: None,
        }
    }

    /// Set the variable type
    pub fn with_type(mut self, var_type: EnvVarType) -> Self {
        self.var_type = var_type;
        self
    }

    /// Mark as optional with a default value
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.required = false;
        self.default = Some(default.into());
        self
    }

    /// Mark as secret
    pub fn secret(mut self) -> Self {
        self.secret = true;
        self
    }

    /// Add a description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set allowed values for enum type
    pub fn with_allowed_values(mut self, values: Vec<String>) -> Self {
        self.var_type = EnvVarType::Enum;
        self.allowed_values = Some(values);
        self
    }

    /// Add an example value
    pub fn with_example(mut self, example: impl Into<String>) -> Self {
        self.example = Some(example.into());
        self
    }

    /// Add a validation pattern
    pub fn with_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.pattern = Some(pattern.into());
        self
    }

    /// Validate a value against this definition
    pub fn validate(&self, value: Option<&str>) -> DeployResult<()> {
        match value {
            None if self.required && self.default.is_none() => {
                Err(DeployError::MissingEnvVar(self.name.clone()))
            }
            None => Ok(()),
            Some(val) => self.validate_value(val),
        }
    }

    /// Validate a specific value
    fn validate_value(&self, value: &str) -> DeployResult<()> {
        match self.var_type {
            EnvVarType::String => Ok(()),
            EnvVarType::Int => {
                value.parse::<i64>().map_err(|_| DeployError::InvalidEnvVarType {
                    name: self.name.clone(),
                    expected: "int".to_string(),
                    value: value.to_string(),
                })?;
                Ok(())
            }
            EnvVarType::Bool => {
                let lower = value.to_lowercase();
                if matches!(
                    lower.as_str(),
                    "true" | "false" | "1" | "0" | "yes" | "no" | "on" | "off"
                ) {
                    Ok(())
                } else {
                    Err(DeployError::InvalidEnvVarType {
                        name: self.name.clone(),
                        expected: "bool (true/false, 1/0, yes/no)".to_string(),
                        value: value.to_string(),
                    })
                }
            }
            EnvVarType::Url => {
                if value.starts_with("http://")
                    || value.starts_with("https://")
                    || value.starts_with("postgres://")
                    || value.starts_with("mysql://")
                    || value.starts_with("redis://")
                    || value.starts_with("mongodb://")
                {
                    Ok(())
                } else {
                    Err(DeployError::InvalidEnvVarType {
                        name: self.name.clone(),
                        expected: "url".to_string(),
                        value: value.to_string(),
                    })
                }
            }
            EnvVarType::Enum => {
                if let Some(ref allowed) = self.allowed_values {
                    if allowed.contains(&value.to_string()) {
                        Ok(())
                    } else {
                        Err(DeployError::InvalidEnvVarType {
                            name: self.name.clone(),
                            expected: format!("one of: {}", allowed.join(", ")),
                            value: value.to_string(),
                        })
                    }
                } else {
                    Ok(())
                }
            }
            EnvVarType::Port => {
                let port: u16 = value.parse().map_err(|_| DeployError::InvalidEnvVarType {
                    name: self.name.clone(),
                    expected: "port (1-65535)".to_string(),
                    value: value.to_string(),
                })?;
                if port == 0 {
                    Err(DeployError::InvalidEnvVarType {
                        name: self.name.clone(),
                        expected: "port (1-65535)".to_string(),
                        value: value.to_string(),
                    })
                } else {
                    Ok(())
                }
            }
            EnvVarType::Path => {
                // Basic path validation - just check it's not empty
                if value.is_empty() {
                    Err(DeployError::InvalidEnvVarType {
                        name: self.name.clone(),
                        expected: "path".to_string(),
                        value: value.to_string(),
                    })
                } else {
                    Ok(())
                }
            }
        }?;

        // Check pattern if specified
        if let Some(ref pattern) = self.pattern {
            let re = regex::Regex::new(pattern)?;
            if !re.is_match(value) {
                return Err(DeployError::InvalidEnvVarType {
                    name: self.name.clone(),
                    expected: format!("pattern: {}", pattern),
                    value: value.to_string(),
                });
            }
        }

        Ok(())
    }

    /// Get the effective value (provided or default)
    pub fn get_value(&self, value: Option<&str>) -> Option<String> {
        value
            .map(|s| s.to_string())
            .or_else(|| self.default.clone())
    }
}

/// Environment schema containing all variable definitions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnvSchema {
    /// Schema version
    #[serde(default = "default_schema_version")]
    pub version: String,
    /// Schema description
    #[serde(default)]
    pub description: Option<String>,
    /// Variable definitions
    #[serde(default)]
    pub variables: Vec<EnvVarDefinition>,
}

fn default_schema_version() -> String {
    "1.0".to_string()
}

impl EnvSchema {
    /// Create a new empty schema
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a variable definition
    pub fn add_variable(&mut self, var: EnvVarDefinition) {
        self.variables.push(var);
    }

    /// Get a variable definition by name
    pub fn get_variable(&self, name: &str) -> Option<&EnvVarDefinition> {
        self.variables.iter().find(|v| v.name == name)
    }

    /// Get all required variables
    pub fn required_variables(&self) -> Vec<&EnvVarDefinition> {
        self.variables.iter().filter(|v| v.required).collect()
    }

    /// Get all secret variables
    pub fn secret_variables(&self) -> Vec<&EnvVarDefinition> {
        self.variables.iter().filter(|v| v.secret).collect()
    }

    /// Load schema from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> DeployResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(DeployError::ConfigNotFound(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Parse schema from TOML string
    pub fn from_toml(content: &str) -> DeployResult<Self> {
        Ok(toml::from_str(content)?)
    }

    /// Serialize schema to TOML
    pub fn to_toml(&self) -> DeployResult<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Save schema to a file
    pub fn save(&self, path: impl AsRef<Path>) -> DeployResult<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Validate environment against this schema
    pub fn validate(&self, env: &HashMap<String, String>) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        for var in &self.variables {
            let value = env.get(&var.name);
            if let Err(e) = var.validate(value.map(|s| s.as_str())) {
                errors.push(ValidationError::new(
                    ValidationCategory::EnvVar,
                    e.to_string(),
                ));
            }
        }

        errors
    }

    /// Validate current process environment against this schema
    pub fn validate_current_env(&self) -> ValidationErrors {
        let mut env = HashMap::new();
        for var in &self.variables {
            if let Ok(value) = std::env::var(&var.name) {
                env.insert(var.name.clone(), value);
            }
        }
        self.validate(&env)
    }

    /// Generate .env.example content
    pub fn generate_env_example(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# Environment Variables".to_string());
        lines.push(format!("# Generated from oxide.env.schema.toml"));
        lines.push(String::new());

        let mut _current_section: Option<String> = None;

        for var in &self.variables {
            // Add description as comment
            if let Some(ref desc) = var.description {
                lines.push(format!("# {}", desc));
            }

            // Add type and required info
            let mut info_parts = vec![format!("Type: {}", var.var_type)];
            if var.required {
                info_parts.push("Required".to_string());
            } else {
                info_parts.push("Optional".to_string());
            }
            if var.secret {
                info_parts.push("SECRET".to_string());
            }
            lines.push(format!("# {}", info_parts.join(" | ")));

            // Add allowed values for enum
            if let Some(ref allowed) = var.allowed_values {
                lines.push(format!("# Allowed: {}", allowed.join(", ")));
            }

            // Generate the example line
            let value = var
                .example
                .clone()
                .or_else(|| var.default.clone())
                .unwrap_or_else(|| {
                    if var.secret {
                        "your-secret-here".to_string()
                    } else {
                        match var.var_type {
                            EnvVarType::String => "value".to_string(),
                            EnvVarType::Int => "0".to_string(),
                            EnvVarType::Bool => "false".to_string(),
                            EnvVarType::Url => "https://example.com".to_string(),
                            EnvVarType::Enum => var
                                .allowed_values
                                .as_ref()
                                .and_then(|v| v.first().cloned())
                                .unwrap_or_else(|| "value".to_string()),
                            EnvVarType::Port => "8080".to_string(),
                            EnvVarType::Path => "/path/to/file".to_string(),
                        }
                    }
                });

            lines.push(format!("{}={}", var.name, value));
            lines.push(String::new());
        }

        lines.join("\n")
    }

    /// Generate CI secret checklist (markdown)
    pub fn generate_ci_secret_checklist(&self) -> String {
        let mut lines = Vec::new();
        lines.push("# CI Secrets Checklist".to_string());
        lines.push(String::new());
        lines.push("The following secrets must be configured in your CI environment:".to_string());
        lines.push(String::new());

        let secrets: Vec<_> = self.variables.iter().filter(|v| v.secret).collect();

        if secrets.is_empty() {
            lines.push("No secrets are defined in the schema.".to_string());
        } else {
            for var in secrets {
                let required_str = if var.required { " (required)" } else { "" };
                lines.push(format!("- [ ] `{}`{}", var.name, required_str));
                if let Some(ref desc) = var.description {
                    lines.push(format!("  - {}", desc));
                }
            }
        }

        lines.push(String::new());
        lines.push("## GitHub Actions".to_string());
        lines.push(String::new());
        lines.push("Add these in: Settings > Secrets and variables > Actions > New repository secret".to_string());
        lines.push(String::new());
        lines.push("## Railway".to_string());
        lines.push(String::new());
        lines.push("Add these in: Project > Variables".to_string());
        lines.push(String::new());
        lines.push("## Fly.io".to_string());
        lines.push(String::new());
        lines.push("```bash".to_string());
        for var in self.secret_variables() {
            lines.push(format!("fly secrets set {}=\"...\"", var.name));
        }
        lines.push("```".to_string());

        lines.join("\n")
    }
}

/// Builder for creating environment schemas
pub struct EnvSchemaBuilder {
    schema: EnvSchema,
}

impl EnvSchemaBuilder {
    /// Create a new schema builder
    pub fn new() -> Self {
        Self {
            schema: EnvSchema::new(),
        }
    }

    /// Set the schema description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.schema.description = Some(description.into());
        self
    }

    /// Add a string variable
    pub fn string(mut self, name: impl Into<String>) -> Self {
        self.schema.add_variable(EnvVarDefinition::new(name));
        self
    }

    /// Add an optional string variable with default
    pub fn string_optional(mut self, name: impl Into<String>, default: impl Into<String>) -> Self {
        self.schema
            .add_variable(EnvVarDefinition::new(name).with_default(default));
        self
    }

    /// Add an integer variable
    pub fn int(mut self, name: impl Into<String>) -> Self {
        self.schema
            .add_variable(EnvVarDefinition::new(name).with_type(EnvVarType::Int));
        self
    }

    /// Add a boolean variable
    pub fn bool(mut self, name: impl Into<String>, default: bool) -> Self {
        self.schema.add_variable(
            EnvVarDefinition::new(name)
                .with_type(EnvVarType::Bool)
                .with_default(default.to_string()),
        );
        self
    }

    /// Add a URL variable
    pub fn url(mut self, name: impl Into<String>) -> Self {
        self.schema
            .add_variable(EnvVarDefinition::new(name).with_type(EnvVarType::Url));
        self
    }

    /// Add a secret variable
    pub fn secret(mut self, name: impl Into<String>) -> Self {
        self.schema
            .add_variable(EnvVarDefinition::new(name).secret());
        self
    }

    /// Add a port variable
    pub fn port(mut self, name: impl Into<String>, default: u16) -> Self {
        self.schema.add_variable(
            EnvVarDefinition::new(name)
                .with_type(EnvVarType::Port)
                .with_default(default.to_string()),
        );
        self
    }

    /// Add a custom variable
    pub fn custom(mut self, var: EnvVarDefinition) -> Self {
        self.schema.add_variable(var);
        self
    }

    /// Build the schema
    pub fn build(self) -> EnvSchema {
        self.schema
    }
}

impl Default for EnvSchemaBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_var_validation_string() {
        let var = EnvVarDefinition::new("TEST_VAR");
        assert!(var.validate(Some("any value")).is_ok());
        assert!(var.validate(None).is_err());
    }

    #[test]
    fn test_env_var_validation_int() {
        let var = EnvVarDefinition::new("PORT").with_type(EnvVarType::Int);
        assert!(var.validate(Some("8080")).is_ok());
        assert!(var.validate(Some("-1")).is_ok());
        assert!(var.validate(Some("not a number")).is_err());
    }

    #[test]
    fn test_env_var_validation_bool() {
        let var = EnvVarDefinition::new("DEBUG").with_type(EnvVarType::Bool);
        assert!(var.validate(Some("true")).is_ok());
        assert!(var.validate(Some("false")).is_ok());
        assert!(var.validate(Some("1")).is_ok());
        assert!(var.validate(Some("0")).is_ok());
        assert!(var.validate(Some("yes")).is_ok());
        assert!(var.validate(Some("no")).is_ok());
        assert!(var.validate(Some("maybe")).is_err());
    }

    #[test]
    fn test_env_var_validation_url() {
        let var = EnvVarDefinition::new("API_URL").with_type(EnvVarType::Url);
        assert!(var.validate(Some("https://api.example.com")).is_ok());
        assert!(var.validate(Some("http://localhost:8080")).is_ok());
        assert!(var.validate(Some("postgres://user:pass@host/db")).is_ok());
        assert!(var.validate(Some("not a url")).is_err());
    }

    #[test]
    fn test_env_var_validation_enum() {
        let var = EnvVarDefinition::new("LOG_LEVEL")
            .with_allowed_values(vec!["debug".to_string(), "info".to_string(), "error".to_string()]);
        assert!(var.validate(Some("debug")).is_ok());
        assert!(var.validate(Some("info")).is_ok());
        assert!(var.validate(Some("warning")).is_err());
    }

    #[test]
    fn test_env_var_validation_port() {
        let var = EnvVarDefinition::new("PORT").with_type(EnvVarType::Port);
        assert!(var.validate(Some("8080")).is_ok());
        assert!(var.validate(Some("443")).is_ok());
        assert!(var.validate(Some("0")).is_err());
        assert!(var.validate(Some("99999")).is_err());
    }

    #[test]
    fn test_env_var_with_default() {
        let var = EnvVarDefinition::new("OPTIONAL").with_default("default_value");
        assert!(!var.required);
        assert!(var.validate(None).is_ok());
        assert_eq!(
            var.get_value(None),
            Some("default_value".to_string())
        );
        assert_eq!(
            var.get_value(Some("custom")),
            Some("custom".to_string())
        );
    }

    #[test]
    fn test_env_schema_builder() {
        let schema = EnvSchemaBuilder::new()
            .description("Test application")
            .string("APP_NAME")
            .port("PORT", 8080)
            .bool("DEBUG", false)
            .secret("API_KEY")
            .url("DATABASE_URL")
            .build();

        assert_eq!(schema.variables.len(), 5);
        assert!(schema.get_variable("APP_NAME").is_some());
        assert!(schema.get_variable("API_KEY").unwrap().secret);
        assert_eq!(schema.secret_variables().len(), 1);
    }

    #[test]
    fn test_env_schema_validation() {
        let schema = EnvSchemaBuilder::new()
            .string("REQUIRED_VAR")
            .string_optional("OPTIONAL_VAR", "default")
            .build();

        let mut env = HashMap::new();
        let errors = schema.validate(&env);
        assert!(errors.has_errors());
        assert_eq!(errors.len(), 1);

        env.insert("REQUIRED_VAR".to_string(), "value".to_string());
        let errors = schema.validate(&env);
        assert!(!errors.has_errors());
    }

    #[test]
    fn test_env_schema_toml_roundtrip() {
        let schema = EnvSchemaBuilder::new()
            .description("Test schema")
            .string("APP_NAME")
            .port("PORT", 8080)
            .secret("API_KEY")
            .build();

        let toml = schema.to_toml().unwrap();
        let parsed = EnvSchema::from_toml(&toml).unwrap();

        assert_eq!(parsed.variables.len(), schema.variables.len());
        assert_eq!(parsed.description, schema.description);
    }

    #[test]
    fn test_generate_env_example() {
        let schema = EnvSchemaBuilder::new()
            .custom(
                EnvVarDefinition::new("DATABASE_URL")
                    .with_type(EnvVarType::Url)
                    .with_description("Database connection string")
                    .secret(),
            )
            .port("PORT", 8080)
            .build();

        let example = schema.generate_env_example();
        assert!(example.contains("DATABASE_URL="));
        assert!(example.contains("PORT=8080"));
        assert!(example.contains("SECRET"));
    }

    #[test]
    fn test_generate_ci_secret_checklist() {
        let schema = EnvSchemaBuilder::new()
            .secret("API_KEY")
            .secret("DATABASE_URL")
            .string("APP_NAME")
            .build();

        let checklist = schema.generate_ci_secret_checklist();
        assert!(checklist.contains("API_KEY"));
        assert!(checklist.contains("DATABASE_URL"));
        assert!(!checklist.contains("APP_NAME")); // Not a secret
        assert!(checklist.contains("GitHub Actions"));
        assert!(checklist.contains("fly secrets set"));
    }
}
