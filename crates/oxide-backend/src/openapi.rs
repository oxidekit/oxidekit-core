//! OpenAPI specification generation and validation
//!
//! This module provides:
//! - OpenAPI spec parsing and validation
//! - Schema extraction from specs
//! - Validation of naming conventions
//! - Version compatibility checking

use crate::naming::{NamingConvention, NamingValidator};
use crate::{BackendError, Result};
use indexmap::IndexMap;
use openapiv3::{OpenAPI, ReferenceOr, Schema, SchemaKind, Type};
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for OpenAPI generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenApiConfig {
    /// API title
    pub title: String,
    /// API description
    pub description: String,
    /// API version
    pub version: String,
    /// Server URLs
    pub servers: Vec<String>,
    /// Naming convention to enforce
    pub naming_convention: NamingConvention,
}

impl Default for OpenApiConfig {
    fn default() -> Self {
        Self {
            title: "API".to_string(),
            description: String::new(),
            version: "0.1.0".to_string(),
            servers: vec!["http://localhost:8000".to_string()],
            naming_convention: NamingConvention::CamelCase,
        }
    }
}

/// OpenAPI specification generator
#[derive(Debug)]
pub struct OpenApiGenerator {
    config: OpenApiConfig,
}

impl OpenApiGenerator {
    /// Create a new OpenAPI generator with the given configuration
    pub fn new(config: OpenApiConfig) -> Self {
        Self { config }
    }

    /// Generate a base OpenAPI specification
    pub fn generate_base(&self) -> OpenAPI {
        OpenAPI {
            openapi: "3.0.3".to_string(),
            info: openapiv3::Info {
                title: self.config.title.clone(),
                description: Some(self.config.description.clone()),
                version: self.config.version.clone(),
                ..Default::default()
            },
            servers: self
                .config
                .servers
                .iter()
                .map(|url| openapiv3::Server {
                    url: url.clone(),
                    ..Default::default()
                })
                .collect(),
            ..Default::default()
        }
    }

    /// Generate OpenAPI spec JSON
    pub fn to_json(&self) -> Result<String> {
        let spec = self.generate_base();
        serde_json::to_string_pretty(&spec).map_err(BackendError::Json)
    }
}

/// Validation results for an OpenAPI spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Whether the spec is valid
    pub valid: bool,
    /// List of errors found
    pub errors: Vec<ValidationError>,
    /// List of warnings
    pub warnings: Vec<ValidationWarning>,
}

impl ValidationResult {
    /// Create a successful validation result
    pub fn success() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error to the result
    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    /// Add a warning to the result
    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }
}

/// A validation error in the OpenAPI spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Path in the spec where error occurred
    pub path: String,
}

/// A validation warning in the OpenAPI spec
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Path in the spec where warning occurred
    pub path: String,
}

/// OpenAPI specification validator
#[derive(Debug)]
pub struct OpenApiValidator {
    naming_validator: NamingValidator,
    require_descriptions: bool,
    require_examples: bool,
}

impl OpenApiValidator {
    /// Create a new validator with the given naming convention
    pub fn new(convention: NamingConvention) -> Self {
        Self {
            naming_validator: NamingValidator::new(convention),
            require_descriptions: false,
            require_examples: false,
        }
    }

    /// Require descriptions for all schemas and operations
    pub fn with_required_descriptions(mut self, require: bool) -> Self {
        self.require_descriptions = require;
        self
    }

    /// Require examples for all schemas
    pub fn with_required_examples(mut self, require: bool) -> Self {
        self.require_examples = require;
        self
    }

    /// Load and validate an OpenAPI spec from a file
    pub fn validate_file(&self, path: impl AsRef<Path>) -> Result<ValidationResult> {
        let content = std::fs::read_to_string(path.as_ref())?;
        self.validate_string(&content)
    }

    /// Validate an OpenAPI spec from a JSON string
    pub fn validate_string(&self, content: &str) -> Result<ValidationResult> {
        let spec: OpenAPI =
            serde_json::from_str(content).map_err(|e| BackendError::OpenApiValidation(e.to_string()))?;
        self.validate(&spec)
    }

    /// Validate an OpenAPI specification
    pub fn validate(&self, spec: &OpenAPI) -> Result<ValidationResult> {
        let mut result = ValidationResult::success();

        // Validate info section
        self.validate_info(&spec.info, &mut result);

        // Validate paths
        self.validate_paths(spec, &mut result);

        // Validate schemas
        if let Some(components) = &spec.components {
            self.validate_schemas(&components.schemas, &mut result);
        }

        Ok(result)
    }

    fn validate_info(&self, info: &openapiv3::Info, result: &mut ValidationResult) {
        if info.title.is_empty() {
            result.add_error(ValidationError {
                code: "MISSING_TITLE".to_string(),
                message: "API title is required".to_string(),
                path: "info.title".to_string(),
            });
        }

        if info.version.is_empty() {
            result.add_error(ValidationError {
                code: "MISSING_VERSION".to_string(),
                message: "API version is required".to_string(),
                path: "info.version".to_string(),
            });
        }

        if self.require_descriptions && info.description.is_none() {
            result.add_warning(ValidationWarning {
                code: "MISSING_DESCRIPTION".to_string(),
                message: "API description is recommended".to_string(),
                path: "info.description".to_string(),
            });
        }
    }

    fn validate_paths(&self, spec: &OpenAPI, result: &mut ValidationResult) {
        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                // Validate each operation
                let operations = [
                    ("get", &item.get),
                    ("post", &item.post),
                    ("put", &item.put),
                    ("delete", &item.delete),
                    ("patch", &item.patch),
                ];

                for (method, op) in operations {
                    if let Some(operation) = op {
                        self.validate_operation(path, method, operation, result);
                    }
                }
            }
        }
    }

    fn validate_operation(
        &self,
        path: &str,
        method: &str,
        operation: &openapiv3::Operation,
        result: &mut ValidationResult,
    ) {
        let op_path = format!("paths.{}.{}", path, method);

        // Check for operation ID
        if operation.operation_id.is_none() {
            result.add_warning(ValidationWarning {
                code: "MISSING_OPERATION_ID".to_string(),
                message: "Operation ID is recommended for client generation".to_string(),
                path: op_path.clone(),
            });
        } else if let Some(op_id) = &operation.operation_id {
            // Validate operation ID naming
            if !self.naming_validator.is_valid(op_id) {
                result.add_error(ValidationError {
                    code: "NAMING_VIOLATION".to_string(),
                    message: format!(
                        "Operation ID '{}' does not follow {} convention",
                        op_id,
                        self.naming_validator.convention().name()
                    ),
                    path: format!("{}.operationId", op_path),
                });
            }
        }

        // Check for description
        if self.require_descriptions && operation.description.is_none() && operation.summary.is_none() {
            result.add_warning(ValidationWarning {
                code: "MISSING_DESCRIPTION".to_string(),
                message: "Operation description or summary is recommended".to_string(),
                path: op_path,
            });
        }
    }

    fn validate_schemas(
        &self,
        schemas: &IndexMap<String, ReferenceOr<Schema>>,
        result: &mut ValidationResult,
    ) {
        for (name, schema_ref) in schemas {
            let schema_path = format!("components.schemas.{}", name);

            // Validate schema name
            if !self.naming_validator.is_valid_type_name(name) {
                result.add_error(ValidationError {
                    code: "NAMING_VIOLATION".to_string(),
                    message: format!(
                        "Schema name '{}' should be PascalCase",
                        name
                    ),
                    path: schema_path.clone(),
                });
            }

            if let ReferenceOr::Item(schema) = schema_ref {
                self.validate_schema_properties(schema, &schema_path, result);
            }
        }
    }

    fn validate_schema_properties(
        &self,
        schema: &Schema,
        base_path: &str,
        result: &mut ValidationResult,
    ) {
        if let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind {
            for (prop_name, _prop_schema) in &obj.properties {
                // Validate property names follow naming convention
                if !self.naming_validator.is_valid(prop_name) {
                    result.add_error(ValidationError {
                        code: "NAMING_VIOLATION".to_string(),
                        message: format!(
                            "Property '{}' does not follow {} convention",
                            prop_name,
                            self.naming_validator.convention().name()
                        ),
                        path: format!("{}.properties.{}", base_path, prop_name),
                    });
                }
            }
        }
    }
}

/// Extract schema information from an OpenAPI spec
#[derive(Debug)]
pub struct SchemaExtractor;

impl SchemaExtractor {
    /// Extract all schema names from a spec
    pub fn extract_schema_names(spec: &OpenAPI) -> Vec<String> {
        spec.components
            .as_ref()
            .map(|c| c.schemas.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Extract all operation IDs from a spec
    pub fn extract_operation_ids(spec: &OpenAPI) -> Vec<String> {
        let mut ids = Vec::new();

        for (_path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                let operations = [
                    &item.get,
                    &item.post,
                    &item.put,
                    &item.delete,
                    &item.patch,
                ];

                for op in operations.into_iter().flatten() {
                    if let Some(id) = &op.operation_id {
                        ids.push(id.clone());
                    }
                }
            }
        }

        ids
    }

    /// Extract all endpoints from a spec
    pub fn extract_endpoints(spec: &OpenAPI) -> Vec<EndpointInfo> {
        let mut endpoints = Vec::new();

        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                let operations = [
                    ("GET", &item.get),
                    ("POST", &item.post),
                    ("PUT", &item.put),
                    ("DELETE", &item.delete),
                    ("PATCH", &item.patch),
                ];

                for (method, op) in operations {
                    if let Some(operation) = op {
                        endpoints.push(EndpointInfo {
                            path: path.clone(),
                            method: method.to_string(),
                            operation_id: operation.operation_id.clone(),
                            summary: operation.summary.clone(),
                            description: operation.description.clone(),
                        });
                    }
                }
            }
        }

        endpoints
    }
}

/// Information about an API endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointInfo {
    /// The path of the endpoint
    pub path: String,
    /// HTTP method
    pub method: String,
    /// Operation ID (if defined)
    pub operation_id: Option<String>,
    /// Summary
    pub summary: Option<String>,
    /// Description
    pub description: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_openapi_json() -> &'static str {
        r#"{
            "openapi": "3.0.3",
            "info": {
                "title": "Test API",
                "version": "1.0.0",
                "description": "A test API"
            },
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "getUsers",
                        "summary": "Get all users",
                        "responses": {
                            "200": {
                                "description": "Success"
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "properties": {
                            "userId": {
                                "type": "string"
                            },
                            "userName": {
                                "type": "string"
                            }
                        }
                    }
                }
            }
        }"#
    }

    #[test]
    fn test_openapi_generator() {
        let config = OpenApiConfig {
            title: "My API".to_string(),
            description: "Test API".to_string(),
            version: "1.0.0".to_string(),
            ..Default::default()
        };

        let generator = OpenApiGenerator::new(config);
        let json = generator.to_json().unwrap();

        assert!(json.contains("My API"));
        assert!(json.contains("1.0.0"));
    }

    #[test]
    fn test_openapi_validator_valid_spec() {
        let validator = OpenApiValidator::new(NamingConvention::CamelCase);
        let result = validator.validate_string(sample_openapi_json()).unwrap();

        assert!(result.valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_openapi_validator_snake_case_violation() {
        let json = r#"{
            "openapi": "3.0.3",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {},
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "properties": {
                            "user_name": {
                                "type": "string"
                            }
                        }
                    }
                }
            }
        }"#;

        let validator = OpenApiValidator::new(NamingConvention::CamelCase);
        let result = validator.validate_string(json).unwrap();

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "NAMING_VIOLATION"));
    }

    #[test]
    fn test_schema_extractor_names() {
        let spec: OpenAPI = serde_json::from_str(sample_openapi_json()).unwrap();
        let names = SchemaExtractor::extract_schema_names(&spec);

        assert!(names.contains(&"User".to_string()));
    }

    #[test]
    fn test_schema_extractor_operation_ids() {
        let spec: OpenAPI = serde_json::from_str(sample_openapi_json()).unwrap();
        let ids = SchemaExtractor::extract_operation_ids(&spec);

        assert!(ids.contains(&"getUsers".to_string()));
    }

    #[test]
    fn test_schema_extractor_endpoints() {
        let spec: OpenAPI = serde_json::from_str(sample_openapi_json()).unwrap();
        let endpoints = SchemaExtractor::extract_endpoints(&spec);

        assert_eq!(endpoints.len(), 1);
        assert_eq!(endpoints[0].path, "/users");
        assert_eq!(endpoints[0].method, "GET");
    }

    #[test]
    fn test_validation_missing_title() {
        let json = r#"{
            "openapi": "3.0.3",
            "info": {
                "title": "",
                "version": "1.0.0"
            },
            "paths": {}
        }"#;

        let validator = OpenApiValidator::new(NamingConvention::CamelCase);
        let result = validator.validate_string(json).unwrap();

        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.code == "MISSING_TITLE"));
    }
}
