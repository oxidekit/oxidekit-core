//! OxideKit Backend Builder
//!
//! Contract-first backend builder system that enforces:
//! - Consistent naming conventions (camelCase by default)
//! - Stable auth flows (access + refresh tokens)
//! - OpenAPI as the source of truth
//! - Zero hand-written API glue
//!
//! # Architecture
//!
//! The backend builder separates concerns:
//! - Backend services generate and commit OpenAPI specs
//! - Frontend apps consume generated, version-pinned clients
//! - Naming, error shapes, pagination, and auth flows are enforced centrally
//!
//! # Modules
//!
//! - [`scaffold`]: Project scaffolding for FastAPI/Rust backend stacks
//! - [`openapi`]: OpenAPI spec generation and validation
//! - [`auth`]: Standardized auth contracts (access/refresh tokens)
//! - [`naming`]: Naming policy enforcement (camelCase)
//! - [`deploy`]: Deployment kit generation

pub mod auth;
pub mod deploy;
pub mod naming;
pub mod openapi;
pub mod scaffold;

pub use auth::{AuthConfig, AuthEndpoint, AuthFlow, TokenConfig};
pub use deploy::{DeployConfig, DeployTarget, DeploymentKit};
pub use naming::{NamingConvention, NamingPolicy, NamingValidator};
pub use openapi::{OpenApiConfig, OpenApiGenerator, OpenApiValidator};
pub use scaffold::{BackendStack, ProjectConfig, ProjectScaffold};

/// Error types for the backend builder
#[derive(Debug, thiserror::Error)]
pub enum BackendError {
    /// IO error during file operations
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// Template rendering error
    #[error("Template error: {0}")]
    Template(String),

    /// OpenAPI validation error
    #[error("OpenAPI validation error: {0}")]
    OpenApiValidation(String),

    /// Naming convention violation
    #[error("Naming convention violation: {0}")]
    NamingViolation(String),

    /// Auth configuration error
    #[error("Auth configuration error: {0}")]
    AuthConfig(String),

    /// Deployment configuration error
    #[error("Deployment error: {0}")]
    Deployment(String),

    /// JSON serialization/deserialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// TOML parsing error
    #[error("TOML error: {0}")]
    Toml(#[from] toml::de::Error),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

/// Result type alias for backend operations
pub type Result<T> = std::result::Result<T, BackendError>;

/// Backend builder configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BackendBuilder {
    /// Project configuration
    pub project: ProjectConfig,
    /// Naming policy
    pub naming: NamingPolicy,
    /// Auth configuration
    pub auth: AuthConfig,
    /// Deployment configuration
    pub deploy: DeployConfig,
}

impl BackendBuilder {
    /// Create a new backend builder with the given project name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            project: ProjectConfig::new(name),
            naming: NamingPolicy::default(),
            auth: AuthConfig::default(),
            deploy: DeployConfig::default(),
        }
    }

    /// Set the backend stack
    pub fn with_stack(mut self, stack: BackendStack) -> Self {
        self.project.stack = stack;
        self
    }

    /// Set the naming convention
    pub fn with_naming(mut self, convention: NamingConvention) -> Self {
        self.naming.convention = convention;
        self
    }

    /// Enable authentication
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = auth;
        self
    }

    /// Set deployment target
    pub fn with_deploy_target(mut self, target: DeployTarget) -> Self {
        self.deploy.target = target;
        self
    }

    /// Build the backend project
    pub fn build(self, output_dir: impl AsRef<std::path::Path>) -> Result<()> {
        let scaffold = ProjectScaffold::new(self.project.clone());
        scaffold.generate(output_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_builder_creation() {
        let builder = BackendBuilder::new("test-api");
        assert_eq!(builder.project.name, "test-api");
    }

    #[test]
    fn test_backend_builder_with_stack() {
        let builder = BackendBuilder::new("test-api").with_stack(BackendStack::FastApi);
        assert!(matches!(builder.project.stack, BackendStack::FastApi));
    }

    #[test]
    fn test_backend_builder_with_naming() {
        let builder = BackendBuilder::new("test-api").with_naming(NamingConvention::CamelCase);
        assert!(matches!(
            builder.naming.convention,
            NamingConvention::CamelCase
        ));
    }
}
