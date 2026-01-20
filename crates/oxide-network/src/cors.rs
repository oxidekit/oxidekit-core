//! CORS configuration generation for backend frameworks.
//!
//! This module generates correct CORS configurations for popular backend frameworks
//! when cross-origin mode is unavoidable. The recommended approach is same-origin
//! deployment (see `reverse_proxy` module), but this provides fallback support.
//!
//! # Supported Frameworks
//!
//! - **FastAPI** (Python): CORSMiddleware configuration
//! - **Express** (Node.js): cors middleware configuration
//! - **Actix-web** (Rust): actix-cors configuration
//! - **Axum** (Rust): tower-http cors configuration
//!
//! # When to Use CORS
//!
//! CORS configuration is needed when:
//! 1. You cannot use same-origin deployment
//! 2. Development requires cross-origin requests before proxy is set up
//! 3. Public APIs that must accept requests from multiple origins
//!
//! # Example
//!
//! ```rust
//! use oxide_network::cors::{CorsConfig, BackendFramework, CorsPreset};
//!
//! // Generate CORS config for FastAPI development
//! let config = CorsConfig::for_development("http://localhost:3000")
//!     .with_preset(CorsPreset::Development);
//!
//! let fastapi_code = config.generate(BackendFramework::FastApi);
//! println!("{}", fastapi_code);
//! ```
//!
//! # CLI Integration
//!
//! ```bash
//! oxide backend cors --stack fastapi --dev-origin http://localhost:3000
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;

/// Supported backend frameworks for CORS configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BackendFramework {
    /// Python FastAPI with CORSMiddleware.
    FastApi,
    /// Node.js Express with cors middleware.
    Express,
    /// Rust Actix-web with actix-cors.
    ActixWeb,
    /// Rust Axum with tower-http cors.
    Axum,
    /// Generic HTTP headers (for custom implementations).
    Generic,
}

impl fmt::Display for BackendFramework {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BackendFramework::FastApi => write!(f, "fastapi"),
            BackendFramework::Express => write!(f, "express"),
            BackendFramework::ActixWeb => write!(f, "actix-web"),
            BackendFramework::Axum => write!(f, "axum"),
            BackendFramework::Generic => write!(f, "generic"),
        }
    }
}

impl std::str::FromStr for BackendFramework {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fastapi" | "fast-api" | "fast_api" => Ok(BackendFramework::FastApi),
            "express" | "expressjs" | "express.js" => Ok(BackendFramework::Express),
            "actix" | "actix-web" | "actix_web" => Ok(BackendFramework::ActixWeb),
            "axum" => Ok(BackendFramework::Axum),
            "generic" | "custom" => Ok(BackendFramework::Generic),
            _ => Err(format!(
                "Unknown backend framework '{}'. Supported: fastapi, express, actix-web, axum, generic",
                s
            )),
        }
    }
}

/// CORS configuration presets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CorsPreset {
    /// Development preset: permissive for ease of development.
    /// - Allows localhost origins
    /// - Allows all standard methods
    /// - Allows credentials
    #[default]
    Development,

    /// Production preset: restrictive for security.
    /// - Only specified origins allowed
    /// - Limited methods
    /// - Credentials optional
    Production,

    /// Public API preset: for APIs accessed by third parties.
    /// - Allow all origins (*)
    /// - No credentials (incompatible with *)
    /// - Limited to safe methods
    PublicApi,

    /// Custom preset: fully manual configuration.
    Custom,
}

impl CorsPreset {
    /// Get a description of this preset.
    pub fn description(&self) -> &'static str {
        match self {
            CorsPreset::Development => "Permissive CORS for development (localhost allowed)",
            CorsPreset::Production => "Restrictive CORS for production (explicit origins only)",
            CorsPreset::PublicApi => "Open CORS for public APIs (all origins, no credentials)",
            CorsPreset::Custom => "Custom CORS configuration",
        }
    }
}

/// HTTP methods for CORS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    /// HTTP GET method.
    Get,
    /// HTTP POST method.
    Post,
    /// HTTP PUT method.
    Put,
    /// HTTP PATCH method.
    Patch,
    /// HTTP DELETE method.
    Delete,
    /// HTTP HEAD method.
    Head,
    /// HTTP OPTIONS method (used for preflight requests).
    Options,
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HttpMethod::Get => write!(f, "GET"),
            HttpMethod::Post => write!(f, "POST"),
            HttpMethod::Put => write!(f, "PUT"),
            HttpMethod::Patch => write!(f, "PATCH"),
            HttpMethod::Delete => write!(f, "DELETE"),
            HttpMethod::Head => write!(f, "HEAD"),
            HttpMethod::Options => write!(f, "OPTIONS"),
        }
    }
}

/// CORS configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    /// Configuration preset.
    pub preset: CorsPreset,
    /// Allowed origins.
    pub origins: HashSet<String>,
    /// Whether to allow all origins (*).
    pub allow_all_origins: bool,
    /// Allowed HTTP methods.
    pub methods: HashSet<HttpMethod>,
    /// Allowed request headers.
    pub allowed_headers: HashSet<String>,
    /// Headers to expose to the client.
    pub exposed_headers: HashSet<String>,
    /// Whether to allow credentials (cookies, auth headers).
    pub allow_credentials: bool,
    /// Max age for preflight cache (seconds).
    pub max_age_secs: u32,
    /// Whether to handle preflight requests automatically.
    pub preflight_continue: bool,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            preset: CorsPreset::Development,
            origins: HashSet::new(),
            allow_all_origins: false,
            methods: [
                HttpMethod::Get,
                HttpMethod::Post,
                HttpMethod::Put,
                HttpMethod::Patch,
                HttpMethod::Delete,
                HttpMethod::Options,
            ]
            .into_iter()
            .collect(),
            allowed_headers: [
                "Content-Type".to_string(),
                "Authorization".to_string(),
                "Accept".to_string(),
                "Origin".to_string(),
                "X-Requested-With".to_string(),
            ]
            .into_iter()
            .collect(),
            exposed_headers: HashSet::new(),
            allow_credentials: true,
            max_age_secs: 600, // 10 minutes
            preflight_continue: false,
        }
    }
}

impl CorsConfig {
    /// Create a new CORS configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a CORS configuration for development with a specific origin.
    pub fn for_development(origin: impl Into<String>) -> Self {
        let mut config = Self::default();
        config.origins.insert(origin.into());
        // Also allow common dev ports
        config.origins.insert("http://localhost:3000".to_string());
        config.origins.insert("http://localhost:5173".to_string()); // Vite
        config.origins.insert("http://localhost:8080".to_string());
        config.origins.insert("http://127.0.0.1:3000".to_string());
        config
    }

    /// Create a CORS configuration for production with explicit origins.
    pub fn for_production(origins: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut config = Self {
            preset: CorsPreset::Production,
            allow_credentials: false, // Safer default for production
            ..Default::default()
        };
        config.origins = origins.into_iter().map(|o| o.into()).collect();
        config
    }

    /// Create a CORS configuration for public APIs.
    pub fn for_public_api() -> Self {
        Self {
            preset: CorsPreset::PublicApi,
            allow_all_origins: true,
            allow_credentials: false, // Cannot use credentials with *
            methods: [HttpMethod::Get, HttpMethod::Post, HttpMethod::Options]
                .into_iter()
                .collect(),
            ..Default::default()
        }
    }

    /// Set the preset.
    pub fn with_preset(mut self, preset: CorsPreset) -> Self {
        self.preset = preset;
        match preset {
            CorsPreset::Development => {
                self.allow_credentials = true;
            }
            CorsPreset::Production => {
                self.allow_credentials = false;
            }
            CorsPreset::PublicApi => {
                self.allow_all_origins = true;
                self.allow_credentials = false;
            }
            CorsPreset::Custom => {}
        }
        self
    }

    /// Add an allowed origin.
    pub fn allow_origin(mut self, origin: impl Into<String>) -> Self {
        self.origins.insert(origin.into());
        self
    }

    /// Allow all origins (*).
    pub fn allow_all_origins(mut self) -> Self {
        self.allow_all_origins = true;
        self.allow_credentials = false; // Cannot use credentials with *
        self
    }

    /// Allow a specific HTTP method.
    pub fn allow_method(mut self, method: HttpMethod) -> Self {
        self.methods.insert(method);
        self
    }

    /// Add an allowed header.
    pub fn allow_header(mut self, header: impl Into<String>) -> Self {
        self.allowed_headers.insert(header.into());
        self
    }

    /// Add an exposed header.
    pub fn expose_header(mut self, header: impl Into<String>) -> Self {
        self.exposed_headers.insert(header.into());
        self
    }

    /// Enable or disable credentials.
    pub fn with_credentials(mut self, allow: bool) -> Self {
        if allow && self.allow_all_origins {
            // Cannot use credentials with allow_all_origins
            self.allow_credentials = false;
        } else {
            self.allow_credentials = allow;
        }
        self
    }

    /// Set the max age for preflight caching.
    pub fn with_max_age(mut self, secs: u32) -> Self {
        self.max_age_secs = secs;
        self
    }

    /// Generate CORS configuration for the specified framework.
    pub fn generate(&self, framework: BackendFramework) -> String {
        match framework {
            BackendFramework::FastApi => self.generate_fastapi(),
            BackendFramework::Express => self.generate_express(),
            BackendFramework::ActixWeb => self.generate_actix(),
            BackendFramework::Axum => self.generate_axum(),
            BackendFramework::Generic => self.generate_generic(),
        }
    }

    /// Generate with documentation comments.
    pub fn generate_with_docs(&self, framework: BackendFramework) -> String {
        let warning = if self.preset != CorsPreset::Production && !self.allow_all_origins {
            format!(
                "\n# WARNING: Using {} preset. Review before deploying to production.\n",
                match self.preset {
                    CorsPreset::Development => "development",
                    CorsPreset::PublicApi => "public API",
                    _ => "custom",
                }
            )
        } else {
            String::new()
        };

        let explanation = format!(
            "# CORS Configuration - {}\n# {}\n#\n# Allowed Origins: {}\n# Allow Credentials: {}\n# Max Age: {} seconds\n{}\n",
            self.preset.description(),
            framework,
            if self.allow_all_origins {
                "*".to_string()
            } else {
                self.origins.iter().cloned().collect::<Vec<_>>().join(", ")
            },
            self.allow_credentials,
            self.max_age_secs,
            warning
        );

        format!("{}\n{}", explanation, self.generate(framework))
    }

    fn generate_fastapi(&self) -> String {
        let origins = if self.allow_all_origins {
            "    allow_origins=[\"*\"],".to_string()
        } else {
            format!(
                "    allow_origins=[\n{}\n    ],",
                self.origins
                    .iter()
                    .map(|o| format!("        \"{}\",", o))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let methods = format!(
            "    allow_methods=[\n{}\n    ],",
            self.methods
                .iter()
                .map(|m| format!("        \"{}\",", m))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let headers = format!(
            "    allow_headers=[\n{}\n    ],",
            self.allowed_headers
                .iter()
                .map(|h| format!("        \"{}\",", h))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let exposed = if self.exposed_headers.is_empty() {
            String::new()
        } else {
            format!(
                "\n    expose_headers=[\n{}\n    ],",
                self.exposed_headers
                    .iter()
                    .map(|h| format!("        \"{}\",", h))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        format!(
            r#"from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

app = FastAPI()

app.add_middleware(
    CORSMiddleware,
{}
{}
{}{}
    allow_credentials={},
    max_age={},
)"#,
            origins,
            methods,
            headers,
            exposed,
            if self.allow_credentials { "True" } else { "False" },
            self.max_age_secs
        )
    }

    fn generate_express(&self) -> String {
        let origins = if self.allow_all_origins {
            "  origin: '*',".to_string()
        } else if self.origins.len() == 1 {
            format!(
                "  origin: '{}',",
                self.origins.iter().next().unwrap()
            )
        } else {
            format!(
                "  origin: [\n{}\n  ],",
                self.origins
                    .iter()
                    .map(|o| format!("    '{}',", o))
                    .collect::<Vec<_>>()
                    .join("\n")
            )
        };

        let methods = format!(
            "  methods: [{}],",
            self.methods
                .iter()
                .map(|m| format!("'{}'", m))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let headers = format!(
            "  allowedHeaders: [{}],",
            self.allowed_headers
                .iter()
                .map(|h| format!("'{}'", h))
                .collect::<Vec<_>>()
                .join(", ")
        );

        let exposed = if self.exposed_headers.is_empty() {
            String::new()
        } else {
            format!(
                "\n  exposedHeaders: [{}],",
                self.exposed_headers
                    .iter()
                    .map(|h| format!("'{}'", h))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        format!(
            r#"const express = require('express');
const cors = require('cors');

const app = express();

const corsOptions = {{
{}
{}
{}{}
  credentials: {},
  maxAge: {},
}};

app.use(cors(corsOptions));"#,
            origins,
            methods,
            headers,
            exposed,
            self.allow_credentials,
            self.max_age_secs
        )
    }

    fn generate_actix(&self) -> String {
        let origins = if self.allow_all_origins {
            "        .allow_any_origin()".to_string()
        } else {
            self.origins
                .iter()
                .map(|o| format!("        .allowed_origin(\"{}\")", o))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let methods = self
            .methods
            .iter()
            .map(|m| match m {
                HttpMethod::Get => "Method::GET",
                HttpMethod::Post => "Method::POST",
                HttpMethod::Put => "Method::PUT",
                HttpMethod::Patch => "Method::PATCH",
                HttpMethod::Delete => "Method::DELETE",
                HttpMethod::Head => "Method::HEAD",
                HttpMethod::Options => "Method::OPTIONS",
            })
            .collect::<Vec<_>>()
            .join(", ");

        let headers = self
            .allowed_headers
            .iter()
            .map(|h| format!("header::HeaderName::from_static(\"{}\")", h.to_lowercase()))
            .collect::<Vec<_>>()
            .join(",\n            ");

        let exposed = if self.exposed_headers.is_empty() {
            String::new()
        } else {
            format!(
                "\n        .expose_headers(vec![{}])",
                self.exposed_headers
                    .iter()
                    .map(|h| format!("header::HeaderName::from_static(\"{}\")", h.to_lowercase()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        format!(
            r#"use actix_cors::Cors;
use actix_web::{{http::{{header, Method}}, App, HttpServer}};

#[actix_web::main]
async fn main() -> std::io::Result<()> {{
    HttpServer::new(|| {{
        let cors = Cors::default()
{}
        .allowed_methods(vec![{}])
        .allowed_headers(vec![
            {}
        ]){}
        .supports_credentials()
        .max_age({});

        App::new()
            .wrap(cors)
            // Add your routes here
    }})
    .bind("0.0.0.0:8000")?
    .run()
    .await
}}"#,
            origins,
            methods,
            headers,
            exposed,
            self.max_age_secs
        )
    }

    fn generate_axum(&self) -> String {
        let origins = if self.allow_all_origins {
            "        .allow_origin(Any)".to_string()
        } else {
            self.origins
                .iter()
                .map(|o| format!("        .allow_origin(\"{}\")", o))
                .collect::<Vec<_>>()
                .join("\n")
        };

        let methods = self
            .methods
            .iter()
            .map(|m| match m {
                HttpMethod::Get => "Method::GET",
                HttpMethod::Post => "Method::POST",
                HttpMethod::Put => "Method::PUT",
                HttpMethod::Patch => "Method::PATCH",
                HttpMethod::Delete => "Method::DELETE",
                HttpMethod::Head => "Method::HEAD",
                HttpMethod::Options => "Method::OPTIONS",
            })
            .collect::<Vec<_>>()
            .join(", ");

        let headers = self
            .allowed_headers
            .iter()
            .map(|h| format!("HeaderName::from_static(\"{}\")", h.to_lowercase()))
            .collect::<Vec<_>>()
            .join(",\n        ");

        let exposed = if self.exposed_headers.is_empty() {
            String::new()
        } else {
            format!(
                "\n        .expose_headers([{}])",
                self.exposed_headers
                    .iter()
                    .map(|h| format!("HeaderName::from_static(\"{}\")", h.to_lowercase()))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        };

        let credentials = if self.allow_credentials {
            "\n        .allow_credentials(true)"
        } else {
            ""
        };

        format!(
            r#"use axum::{{Router, routing::get}};
use tower_http::cors::{{CorsLayer, Any}};
use http::{{Method, HeaderName}};
use std::time::Duration;

#[tokio::main]
async fn main() {{
    let cors = CorsLayer::new()
{}
        .allow_methods([{}])
        .allow_headers([
        {}
        ]){}{}
        .max_age(Duration::from_secs({}));

    let app = Router::new()
        // Add your routes here
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}}"#,
            origins,
            methods,
            headers,
            exposed,
            credentials,
            self.max_age_secs
        )
    }

    fn generate_generic(&self) -> String {
        let origins_value = if self.allow_all_origins {
            "*".to_string()
        } else {
            self.origins.iter().cloned().collect::<Vec<_>>().join(", ")
        };

        let methods_value = self
            .methods
            .iter()
            .map(|m| m.to_string())
            .collect::<Vec<_>>()
            .join(", ");

        let headers_value = self
            .allowed_headers
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(", ");

        let exposed_value = if self.exposed_headers.is_empty() {
            String::new()
        } else {
            format!(
                "Access-Control-Expose-Headers: {}\n",
                self.exposed_headers.iter().cloned().collect::<Vec<_>>().join(", ")
            )
        };

        format!(
            r#"# CORS Headers to add to responses

# Preflight (OPTIONS) response headers:
Access-Control-Allow-Origin: {}
Access-Control-Allow-Methods: {}
Access-Control-Allow-Headers: {}
{}Access-Control-Allow-Credentials: {}
Access-Control-Max-Age: {}

# Actual response headers:
Access-Control-Allow-Origin: {}
{}Access-Control-Allow-Credentials: {}"#,
            origins_value,
            methods_value,
            headers_value,
            exposed_value,
            self.allow_credentials,
            self.max_age_secs,
            origins_value,
            exposed_value,
            self.allow_credentials
        )
    }

    /// Validate the CORS configuration.
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Check credentials with wildcard origin
        if self.allow_all_origins && self.allow_credentials {
            errors.push(
                "Cannot use credentials with wildcard (*) origin. \
                Set allow_credentials to false or specify explicit origins."
                    .to_string(),
            );
        }

        // Check for empty origins in non-public mode
        if !self.allow_all_origins && self.origins.is_empty() {
            errors.push("No origins specified. Add allowed origins or use allow_all_origins.".to_string());
        }

        // Validate origin formats
        for origin in &self.origins {
            if !origin.starts_with("http://") && !origin.starts_with("https://") {
                errors.push(format!(
                    "Invalid origin '{}'. Origins must start with http:// or https://",
                    origin
                ));
            }
        }

        // Check for OPTIONS method if credentials are used
        if self.allow_credentials && !self.methods.contains(&HttpMethod::Options) {
            errors.push(
                "OPTIONS method should be allowed for preflight requests when using credentials."
                    .to_string(),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// CORS diagnostic result.
#[derive(Debug, Clone)]
pub struct CorsDiagnostic {
    /// Whether the configuration is valid.
    pub valid: bool,
    /// Errors found.
    pub errors: Vec<String>,
    /// Warnings.
    pub warnings: Vec<String>,
    /// Recommendations.
    pub recommendations: Vec<String>,
}

impl CorsDiagnostic {
    /// Run diagnostics on a CORS configuration.
    pub fn analyze(config: &CorsConfig) -> Self {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();

        // Validation errors
        if let Err(validation_errors) = config.validate() {
            errors.extend(validation_errors);
        }

        // Warnings
        if config.allow_all_origins && config.preset == CorsPreset::Production {
            warnings.push(
                "Production preset with allow_all_origins is unusual. \
                Consider using explicit origins for better security."
                    .to_string(),
            );
        }

        if config.preset == CorsPreset::Development {
            warnings.push(
                "Development preset detected. Ensure you use stricter settings in production."
                    .to_string(),
            );
        }

        if config.max_age_secs > 86400 {
            warnings.push(format!(
                "Max age of {} seconds is very long. Consider a shorter cache time.",
                config.max_age_secs
            ));
        }

        // Recommendations
        if !config.allow_all_origins && config.origins.len() > 10 {
            recommendations.push(
                "Many origins specified. Consider using a reverse proxy for same-origin deployment."
                    .to_string(),
            );
        }

        if config.allowed_headers.contains("*") || config.allowed_headers.len() > 20 {
            recommendations.push(
                "Consider limiting allowed headers to only those actually needed."
                    .to_string(),
            );
        }

        if !config.methods.contains(&HttpMethod::Options) {
            recommendations.push(
                "Consider adding OPTIONS method to handle preflight requests properly."
                    .to_string(),
            );
        }

        recommendations.push(
            "For best security, consider same-origin deployment using a reverse proxy. \
            This eliminates CORS entirely. See `oxide network generate-proxy --help`."
                .to_string(),
        );

        Self {
            valid: errors.is_empty(),
            errors,
            warnings,
            recommendations,
        }
    }

    /// Format the diagnostic as a string.
    pub fn format(&self) -> String {
        let mut output = String::new();

        if self.valid {
            output.push_str("CORS Configuration: VALID\n\n");
        } else {
            output.push_str("CORS Configuration: INVALID\n\n");
        }

        if !self.errors.is_empty() {
            output.push_str("ERRORS:\n");
            for error in &self.errors {
                output.push_str(&format!("  - {}\n", error));
            }
            output.push('\n');
        }

        if !self.warnings.is_empty() {
            output.push_str("WARNINGS:\n");
            for warning in &self.warnings {
                output.push_str(&format!("  - {}\n", warning));
            }
            output.push('\n');
        }

        if !self.recommendations.is_empty() {
            output.push_str("RECOMMENDATIONS:\n");
            for rec in &self.recommendations {
                output.push_str(&format!("  - {}\n", rec));
            }
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cors_config_default() {
        let config = CorsConfig::new();
        assert_eq!(config.preset, CorsPreset::Development);
        assert!(!config.allow_all_origins);
        assert!(config.allow_credentials);
    }

    #[test]
    fn test_cors_for_development() {
        let config = CorsConfig::for_development("http://localhost:5173");
        assert!(config.origins.contains(&"http://localhost:5173".to_string()));
        assert!(config.origins.contains(&"http://localhost:3000".to_string()));
    }

    #[test]
    fn test_cors_for_production() {
        let config = CorsConfig::for_production(["https://app.example.com"]);
        assert_eq!(config.preset, CorsPreset::Production);
        assert!(!config.allow_credentials);
        assert!(config.origins.contains(&"https://app.example.com".to_string()));
    }

    #[test]
    fn test_cors_for_public_api() {
        let config = CorsConfig::for_public_api();
        assert!(config.allow_all_origins);
        assert!(!config.allow_credentials);
    }

    #[test]
    fn test_cors_validation() {
        // Invalid: credentials with wildcard
        let config = CorsConfig::new().allow_all_origins().with_credentials(true);
        // with_credentials should have prevented this
        assert!(!config.allow_credentials);

        // Valid config
        let config = CorsConfig::for_development("http://localhost:3000");
        assert!(config.validate().is_ok());

        // Invalid: empty origins
        let config = CorsConfig::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_fastapi_generation() {
        let config = CorsConfig::for_development("http://localhost:3000");
        let output = config.generate(BackendFramework::FastApi);

        assert!(output.contains("CORSMiddleware"));
        assert!(output.contains("http://localhost:3000"));
        assert!(output.contains("allow_credentials=True"));
    }

    #[test]
    fn test_express_generation() {
        let config = CorsConfig::for_development("http://localhost:3000");
        let output = config.generate(BackendFramework::Express);

        assert!(output.contains("cors"));
        assert!(output.contains("http://localhost:3000"));
        assert!(output.contains("credentials: true"));
    }

    #[test]
    fn test_actix_generation() {
        let config = CorsConfig::for_development("http://localhost:3000");
        let output = config.generate(BackendFramework::ActixWeb);

        assert!(output.contains("actix_cors"));
        assert!(output.contains("http://localhost:3000"));
        assert!(output.contains("supports_credentials"));
    }

    #[test]
    fn test_axum_generation() {
        let config = CorsConfig::for_development("http://localhost:3000");
        let output = config.generate(BackendFramework::Axum);

        assert!(output.contains("CorsLayer"));
        assert!(output.contains("http://localhost:3000"));
        assert!(output.contains("allow_credentials(true)"));
    }

    #[test]
    fn test_generic_generation() {
        let config = CorsConfig::for_public_api();
        let output = config.generate(BackendFramework::Generic);

        assert!(output.contains("Access-Control-Allow-Origin: *"));
        assert!(output.contains("Access-Control-Allow-Credentials: false"));
    }

    #[test]
    fn test_backend_framework_parsing() {
        assert_eq!(
            "fastapi".parse::<BackendFramework>().unwrap(),
            BackendFramework::FastApi
        );
        assert_eq!(
            "express".parse::<BackendFramework>().unwrap(),
            BackendFramework::Express
        );
        assert_eq!(
            "actix-web".parse::<BackendFramework>().unwrap(),
            BackendFramework::ActixWeb
        );
        assert_eq!(
            "axum".parse::<BackendFramework>().unwrap(),
            BackendFramework::Axum
        );
        assert!("unknown".parse::<BackendFramework>().is_err());
    }

    #[test]
    fn test_cors_diagnostic() {
        let config = CorsConfig::for_development("http://localhost:3000");
        let diagnostic = CorsDiagnostic::analyze(&config);

        assert!(diagnostic.valid);
        assert!(!diagnostic.warnings.is_empty()); // Should warn about dev preset
        assert!(!diagnostic.recommendations.is_empty());
    }

    #[test]
    fn test_cors_config_builder() {
        let config = CorsConfig::new()
            .allow_origin("https://app.example.com")
            .allow_method(HttpMethod::Get)
            .allow_header("X-Custom-Header")
            .expose_header("X-Response-Id")
            .with_credentials(true)
            .with_max_age(3600);

        assert!(config.origins.contains(&"https://app.example.com".to_string()));
        assert!(config.methods.contains(&HttpMethod::Get));
        assert!(config.allowed_headers.contains("X-Custom-Header"));
        assert!(config.exposed_headers.contains("X-Response-Id"));
        assert!(config.allow_credentials);
        assert_eq!(config.max_age_secs, 3600);
    }
}
