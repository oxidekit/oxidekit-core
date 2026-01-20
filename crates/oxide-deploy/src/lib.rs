//! OxideKit Deployment Configuration System
//!
//! A comprehensive deployment configuration library for OxideKit applications.
//! This crate provides schema-driven configuration for environment variables,
//! ports, secrets, and deployment templates across multiple platforms.
//!
//! # Features
//!
//! - **Environment Schema**: Define and validate environment variables with types
//! - **Port Management**: Configure port mappings with collision detection
//! - **Secret Handling**: Secure secret management across environments
//! - **Deployment Templates**: Generate templates for Docker, Railway, Fly.io, Render
//! - **CI Workflows**: Generate CI workflow templates for GitHub Actions, GitLab CI
//! - **Configuration Validation**: Comprehensive validation with detailed reporting
//!
//! # Example
//!
//! ```no_run
//! use oxide_deploy::{
//!     env_schema::{EnvSchemaBuilder, EnvVarType},
//!     ports::PortConfigBuilder,
//!     templates::{TemplateConfig, TemplateGenerator, DeployTarget},
//!     validate::{DeployConfig, Validator},
//! };
//!
//! // Define environment schema
//! let schema = EnvSchemaBuilder::new()
//!     .description("My application configuration")
//!     .url("DATABASE_URL")
//!     .port("PORT", 8080)
//!     .secret("API_KEY")
//!     .bool("DEBUG", false)
//!     .build();
//!
//! // Define port configuration
//! let ports = PortConfigBuilder::new()
//!     .http(8080)
//!     .metrics(9090)
//!     .build();
//!
//! // Generate deployment template
//! let template_config = TemplateConfig::new("my-app")
//!     .with_port(8080)
//!     .with_env_schema(schema.clone());
//!
//! let generator = TemplateGenerator::new(template_config);
//! let dockerfile = generator.generate(DeployTarget::Docker).unwrap();
//!
//! // Validate configuration
//! let config = DeployConfig::new("my-app")
//!     .with_env_schema(schema)
//!     .with_port_config(ports);
//!
//! let validator = Validator::new(config);
//! let report = validator.validate();
//!
//! if !report.passed() {
//!     for error in report.errors() {
//!         eprintln!("Error: {}", error);
//!     }
//! }
//! ```
//!
//! # CLI Integration
//!
//! This crate is designed to integrate with the OxideKit CLI:
//!
//! ```bash
//! # Initialize environment schema
//! oxide env init
//!
//! # Validate environment configuration
//! oxide env validate
//!
//! # Print current environment
//! oxide env print
//!
//! # Generate deployment template
//! oxide deploy kit --target docker
//! oxide deploy kit --target railway
//! oxide deploy kit --target fly
//! ```

#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

pub mod ci;
pub mod env_schema;
pub mod error;
pub mod ports;
pub mod secrets;
pub mod templates;
pub mod validate;

// Re-exports for convenience
pub use error::{DeployError, DeployResult, ValidationErrors};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::ci::{BuildPlatform, CiConfig, CiGenerator, CiPlatform};
    pub use crate::env_schema::{EnvSchema, EnvSchemaBuilder, EnvVarDefinition, EnvVarType};
    pub use crate::error::{DeployError, DeployResult, ValidationCategory, ValidationError, ValidationErrors};
    pub use crate::ports::{PortConfig, PortConfigBuilder, PortMapping, Protocol};
    pub use crate::secrets::{SecretBackend, SecretConfig, SecretDefinition, SecretScanner};
    pub use crate::templates::{AppType, DeployTarget, TemplateConfig, TemplateGenerator};
    pub use crate::validate::{DeployConfig, Severity, ValidationFinding, ValidationReport, Validator};
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn test_full_workflow() {
        // Create environment schema
        let schema = EnvSchemaBuilder::new()
            .description("Test application")
            .string("APP_NAME")
            .port("PORT", 8080)
            .bool("DEBUG", false)
            .secret("API_KEY")
            .build();

        assert_eq!(schema.variables.len(), 4);
        assert_eq!(schema.required_variables().len(), 2); // APP_NAME and API_KEY
        assert_eq!(schema.secret_variables().len(), 1);

        // Create port configuration
        let ports = PortConfigBuilder::new()
            .http(8080)
            .metrics(9090)
            .database("postgres", 5432)
            .build();

        assert_eq!(ports.ports.len(), 3);
        assert!(ports.is_port_available(3000));
        assert!(!ports.is_port_available(8080));

        // Validate port collisions
        let collisions = ports.detect_collisions();
        assert!(collisions.is_empty());

        // Generate .env.example
        let env_example = schema.generate_env_example();
        assert!(env_example.contains("APP_NAME="));
        assert!(env_example.contains("PORT=8080"));
        assert!(env_example.contains("SECRET"));

        // Generate CI secret checklist
        let checklist = schema.generate_ci_secret_checklist();
        assert!(checklist.contains("API_KEY"));

        // Generate deployment template
        let template_config = TemplateConfig::new("test-app")
            .with_port(8080)
            .with_env_schema(schema.clone());

        let generator = TemplateGenerator::new(template_config);

        let dockerfile = generator.generate(DeployTarget::Docker).unwrap();
        assert!(dockerfile.contains("FROM rust:"));
        assert!(dockerfile.contains("EXPOSE 8080"));

        let compose = generator.generate(DeployTarget::DockerCompose).unwrap();
        assert!(compose.contains("services:"));
        assert!(compose.contains("test-app"));

        let railway = generator.generate(DeployTarget::Railway).unwrap();
        assert!(railway.contains("[build]"));

        let fly = generator.generate(DeployTarget::FlyIo).unwrap();
        assert!(fly.contains("app = \"test-app\""));

        let render = generator.generate(DeployTarget::Render).unwrap();
        assert!(render.contains("type: web"));

        // Generate CI workflow
        let ci_config = CiConfig::new("test-app")
            .with_platforms(vec![BuildPlatform::LinuxX64, BuildPlatform::MacOSArm64]);

        let ci_generator = CiGenerator::new(ci_config);
        let github_actions = ci_generator.generate(CiPlatform::GitHubActions).unwrap();
        assert!(github_actions.contains("name: CI"));
        assert!(github_actions.contains("cargo test"));
    }

    #[test]
    fn test_validation_integration() {
        let schema = EnvSchemaBuilder::new()
            .string("REQUIRED_VAR")
            .secret("API_KEY")
            .build();

        let ports = PortConfigBuilder::new()
            .http(8080)
            .build();

        let config = DeployConfig::new("test-app")
            .with_env_schema(schema)
            .with_port_config(ports);

        let validator = Validator::with_environment(config, std::collections::HashMap::new());
        let report = validator.validate();

        // Should have errors for missing required variables
        assert!(report.has_errors());
        assert!(report.error_count() >= 2); // REQUIRED_VAR and API_KEY
    }

    #[test]
    fn test_secret_scanner() {
        let scanner = SecretScanner::new();

        // Scanner should be able to detect common sensitive patterns
        let temp_dir = tempfile::TempDir::new().unwrap();
        let test_file = temp_dir.path().join("config.json");
        std::fs::write(&test_file, r#"{"api_key": "sk_live_abcdefghijklmnopqrstuvwxyz"}"#).unwrap();

        let findings = scanner.scan_file(&test_file).unwrap();
        assert!(!findings.is_empty());
    }

    #[test]
    fn test_template_targets() {
        // Verify all deploy targets can be parsed
        assert_eq!("docker".parse::<DeployTarget>().unwrap(), DeployTarget::Docker);
        assert_eq!("docker-compose".parse::<DeployTarget>().unwrap(), DeployTarget::DockerCompose);
        assert_eq!("railway".parse::<DeployTarget>().unwrap(), DeployTarget::Railway);
        assert_eq!("fly.io".parse::<DeployTarget>().unwrap(), DeployTarget::FlyIo);
        assert_eq!("render".parse::<DeployTarget>().unwrap(), DeployTarget::Render);
        assert_eq!("nginx".parse::<DeployTarget>().unwrap(), DeployTarget::Nginx);
        assert_eq!("caddy".parse::<DeployTarget>().unwrap(), DeployTarget::Caddy);
    }

    #[test]
    fn test_build_platforms() {
        // Verify build platform properties
        assert_eq!(BuildPlatform::LinuxX64.github_runner(), "ubuntu-latest");
        assert_eq!(BuildPlatform::MacOSArm64.github_runner(), "macos-14");
        assert_eq!(BuildPlatform::WindowsX64.artifact_extension(), ".exe");
        assert_eq!(BuildPlatform::LinuxX64.artifact_extension(), "");
    }
}
