//! Rust client generation from OpenAPI specs
//!
//! Generates type-safe Rust clients for OxideKit applications.

use crate::{ClientConfig, ClientGenerator, ClientLanguage, GeneratedClient, GeneratedFile, Result};
use openapiv3::{OpenAPI, Operation, PathItem, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::{HashMap, HashSet};

/// Rust client generator
#[derive(Debug)]
pub struct RustClientGenerator {
    config: ClientConfig,
}

impl RustClientGenerator {
    /// Create a new Rust client generator
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            config: ClientConfig::new(package_name),
        }
    }

    /// Create with full configuration
    pub fn with_config(config: ClientConfig) -> Self {
        Self { config }
    }

    /// Extract all schemas from the OpenAPI spec
    fn extract_schemas(&self, spec: &OpenAPI) -> HashMap<String, SchemaInfo> {
        let mut schemas = HashMap::new();

        if let Some(components) = &spec.components {
            for (name, schema_ref) in &components.schemas {
                if let ReferenceOr::Item(schema) = schema_ref {
                    schemas.insert(name.clone(), self.parse_schema(name, schema));
                }
            }
        }

        schemas
    }

    /// Parse a schema into our internal representation
    fn parse_schema(&self, name: &str, schema: &Schema) -> SchemaInfo {
        let mut info = SchemaInfo {
            name: name.to_string(),
            rust_name: to_pascal_case(name),
            description: schema.schema_data.description.clone(),
            fields: Vec::new(),
            is_enum: false,
            enum_values: Vec::new(),
        };

        match &schema.schema_kind {
            SchemaKind::Type(Type::Object(obj)) => {
                let required: HashSet<_> = obj.required.iter().cloned().collect();

                for (field_name, field_schema) in &obj.properties {
                    let field_type = self.boxed_schema_to_rust_type(field_schema);
                    let is_required = required.contains(field_name);

                    info.fields.push(FieldInfo {
                        name: field_name.clone(),
                        rust_name: to_snake_case(field_name),
                        rust_type: if is_required {
                            field_type
                        } else {
                            format!("Option<{}>", field_type)
                        },
                        is_required,
                        description: None,
                    });
                }
            }
            SchemaKind::Type(Type::String(string_type)) => {
                if !string_type.enumeration.is_empty() {
                    info.is_enum = true;
                    info.enum_values = string_type
                        .enumeration
                        .iter()
                        .filter_map(|v| v.clone())
                        .collect();
                }
            }
            _ => {}
        }

        info
    }

    /// Convert an OpenAPI schema reference (boxed) to a Rust type
    fn boxed_schema_to_rust_type(&self, schema_ref: &ReferenceOr<Box<Schema>>) -> String {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                // Extract type name from reference like "#/components/schemas/User"
                reference
                    .split('/')
                    .last()
                    .map(to_pascal_case)
                    .unwrap_or_else(|| "serde_json::Value".to_string())
            }
            ReferenceOr::Item(schema) => self.schema_kind_to_rust_type(&schema.schema_kind),
        }
    }

    /// Convert an OpenAPI schema reference to a Rust type
    fn schema_to_rust_type(&self, schema_ref: &ReferenceOr<Schema>) -> String {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                // Extract type name from reference like "#/components/schemas/User"
                reference
                    .split('/')
                    .last()
                    .map(to_pascal_case)
                    .unwrap_or_else(|| "serde_json::Value".to_string())
            }
            ReferenceOr::Item(schema) => self.schema_kind_to_rust_type(&schema.schema_kind),
        }
    }

    /// Convert a schema kind to Rust type
    fn schema_kind_to_rust_type(&self, kind: &SchemaKind) -> String {
        match kind {
            SchemaKind::Type(Type::String(_)) => "String".to_string(),
            SchemaKind::Type(Type::Integer(_)) => "i64".to_string(),
            SchemaKind::Type(Type::Number(_)) => "f64".to_string(),
            SchemaKind::Type(Type::Boolean(_)) => "bool".to_string(),
            SchemaKind::Type(Type::Array(arr)) => {
                let item_type = arr
                    .items
                    .as_ref()
                    .map(|i| self.boxed_schema_to_rust_type(i))
                    .unwrap_or_else(|| "serde_json::Value".to_string());
                format!("Vec<{}>", item_type)
            }
            SchemaKind::Type(Type::Object(_)) => "serde_json::Value".to_string(),
            _ => "serde_json::Value".to_string(),
        }
    }

    /// Extract all endpoints from the OpenAPI spec
    fn extract_endpoints(&self, spec: &OpenAPI) -> Vec<EndpointInfo> {
        let mut endpoints = Vec::new();

        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                self.extract_path_endpoints(path, item, &mut endpoints);
            }
        }

        endpoints
    }

    /// Extract endpoints from a path item
    fn extract_path_endpoints(&self, path: &str, item: &PathItem, endpoints: &mut Vec<EndpointInfo>) {
        let operations = [
            ("get", &item.get),
            ("post", &item.post),
            ("put", &item.put),
            ("delete", &item.delete),
            ("patch", &item.patch),
        ];

        for (method, op) in operations {
            if let Some(operation) = op {
                endpoints.push(self.parse_endpoint(path, method, operation));
            }
        }
    }

    /// Parse an operation into an endpoint info
    fn parse_endpoint(&self, path: &str, method: &str, operation: &Operation) -> EndpointInfo {
        let function_name = operation
            .operation_id
            .as_ref()
            .map(|id| to_snake_case(id))
            .unwrap_or_else(|| format!("{}_{}", method, path_to_function_name(path)));

        let mut params = Vec::new();

        // Path parameters
        for param_ref in &operation.parameters {
            if let ReferenceOr::Item(param) = param_ref {
                if let openapiv3::Parameter::Path { parameter_data, .. } = param {
                    params.push(ParamInfo {
                        name: parameter_data.name.clone(),
                        rust_name: to_snake_case(&parameter_data.name),
                        rust_type: "String".to_string(),
                        location: ParamLocation::Path,
                        required: true,
                    });
                } else if let openapiv3::Parameter::Query { parameter_data, .. } = param {
                    params.push(ParamInfo {
                        name: parameter_data.name.clone(),
                        rust_name: to_snake_case(&parameter_data.name),
                        rust_type: if parameter_data.required {
                            "String".to_string()
                        } else {
                            "Option<String>".to_string()
                        },
                        location: ParamLocation::Query,
                        required: parameter_data.required,
                    });
                }
            }
        }

        // Request body
        let request_body_type = operation.request_body.as_ref().and_then(|body| {
            if let ReferenceOr::Item(body) = body {
                body.content.get("application/json").and_then(|media| {
                    media.schema.as_ref().map(|s| self.schema_to_rust_type(s))
                })
            } else {
                None
            }
        });

        // Response type
        let response_type = operation
            .responses
            .responses
            .get(&openapiv3::StatusCode::Code(200))
            .or_else(|| operation.responses.responses.get(&openapiv3::StatusCode::Code(201)))
            .and_then(|resp| {
                if let ReferenceOr::Item(response) = resp {
                    response.content.get("application/json").and_then(|media| {
                        media.schema.as_ref().map(|s| self.schema_to_rust_type(s))
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "()".to_string());

        EndpointInfo {
            path: path.to_string(),
            method: method.to_string(),
            function_name,
            description: operation.description.clone().or(operation.summary.clone()),
            params,
            request_body_type,
            response_type,
        }
    }

    /// Generate Cargo.toml content
    fn generate_cargo_toml(&self) -> String {
        format!(
            r#"[package]
name = "{name}"
version = "{version}"
edition = "2021"
description = "{description}"

[dependencies]
reqwest = {{ version = "0.12", features = ["json", "rustls-tls"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
thiserror = "2.0"
tokio = {{ version = "1.0", features = ["full"] }}
"#,
            name = self.config.package_name,
            version = self.config.version,
            description = self.config.description
        )
    }

    /// Generate lib.rs content
    fn generate_lib_rs(&self, spec: &OpenAPI, schemas: &HashMap<String, SchemaInfo>, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        // Header
        content.push_str(&format!(
            "//! {} API Client\n//!\n//! Generated by OxideKit Client Generator.\n//! Do not edit manually.\n\n",
            spec.info.title
        ));

        content.push_str("use reqwest::Client;\n");
        content.push_str("use serde::{Deserialize, Serialize};\n\n");

        // Error type
        content.push_str(
            r#"/// API client error type
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Request error: {0}")]
    Request(#[from] reqwest::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("API error: {status} - {message}")]
    Api { status: u16, message: String },
}

pub type Result<T> = std::result::Result<T, ApiError>;

"#,
        );

        // Generate types
        content.push_str("// ============ Types ============\n\n");
        for schema in schemas.values() {
            content.push_str(&self.generate_type(schema));
            content.push('\n');
        }

        // Generate client
        content.push_str("// ============ Client ============\n\n");
        content.push_str(&self.generate_client_struct(endpoints));

        content
    }

    /// Generate a Rust type from schema info
    fn generate_type(&self, schema: &SchemaInfo) -> String {
        let mut content = String::new();

        if let Some(desc) = &schema.description {
            content.push_str(&format!("/// {}\n", desc));
        }

        if schema.is_enum {
            content.push_str("#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\n");
            content.push_str(&format!("pub enum {} {{\n", schema.rust_name));
            for value in &schema.enum_values {
                let variant_name = to_pascal_case(value);
                content.push_str(&format!("    #[serde(rename = \"{}\")]\n", value));
                content.push_str(&format!("    {},\n", variant_name));
            }
            content.push_str("}\n");
        } else {
            content.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
            content.push_str(&format!("pub struct {} {{\n", schema.rust_name));
            for field in &schema.fields {
                if let Some(desc) = &field.description {
                    content.push_str(&format!("    /// {}\n", desc));
                }
                // Use serde rename if the field name differs
                if field.name != field.rust_name {
                    content.push_str(&format!("    #[serde(rename = \"{}\")]\n", field.name));
                }
                if !field.is_required {
                    content.push_str("    #[serde(skip_serializing_if = \"Option::is_none\")]\n");
                }
                content.push_str(&format!("    pub {}: {},\n", field.rust_name, field.rust_type));
            }
            content.push_str("}\n");
        }

        content
    }

    /// Generate the client struct and impl
    fn generate_client_struct(&self, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        // Client struct
        content.push_str(
            r#"/// API Client
pub struct ApiClient {
    client: Client,
    base_url: String,
    auth_token: Option<String>,
}

impl ApiClient {
    /// Create a new API client
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.into(),
            auth_token: None,
        }
    }

    /// Set the authentication token
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Set the authentication token (mutable)
    pub fn set_auth(&mut self, token: impl Into<String>) {
        self.auth_token = Some(token.into());
    }

    /// Clear the authentication token
    pub fn clear_auth(&mut self) {
        self.auth_token = None;
    }

    fn build_request(&self, method: reqwest::Method, path: &str) -> reqwest::RequestBuilder {
        let url = format!("{}{}", self.base_url, path);
        let mut request = self.client.request(method, &url);

        if let Some(token) = &self.auth_token {
            request = request.bearer_auth(token);
        }

        request
    }

"#,
        );

        // Generate methods for each endpoint
        for endpoint in endpoints {
            content.push_str(&self.generate_endpoint_method(endpoint));
            content.push('\n');
        }

        content.push_str("}\n");

        content
    }

    /// Generate a method for an endpoint
    fn generate_endpoint_method(&self, endpoint: &EndpointInfo) -> String {
        let mut content = String::new();

        // Doc comment
        if let Some(desc) = &endpoint.description {
            content.push_str(&format!("    /// {}\n", desc));
        }

        // Function signature
        let mut params_str = String::new();
        for param in &endpoint.params {
            if !params_str.is_empty() {
                params_str.push_str(", ");
            }
            params_str.push_str(&format!("{}: {}", param.rust_name, param.rust_type));
        }

        if let Some(body_type) = &endpoint.request_body_type {
            if !params_str.is_empty() {
                params_str.push_str(", ");
            }
            params_str.push_str(&format!("body: &{}", body_type));
        }

        content.push_str(&format!(
            "    pub async fn {}(&self{}) -> Result<{}> {{\n",
            endpoint.function_name,
            if params_str.is_empty() {
                String::new()
            } else {
                format!(", {}", params_str)
            },
            endpoint.response_type
        ));

        // Build path with parameters
        let mut path = endpoint.path.clone();
        for param in endpoint.params.iter().filter(|p| p.location == ParamLocation::Path) {
            path = path.replace(&format!("{{{}}}", param.name), &format!("{{{}}}", param.rust_name));
        }

        // Method body
        let method = match endpoint.method.as_str() {
            "get" => "reqwest::Method::GET",
            "post" => "reqwest::Method::POST",
            "put" => "reqwest::Method::PUT",
            "delete" => "reqwest::Method::DELETE",
            "patch" => "reqwest::Method::PATCH",
            _ => "reqwest::Method::GET",
        };

        // Build the path string
        if endpoint.params.iter().any(|p| p.location == ParamLocation::Path) {
            content.push_str(&format!(
                "        let path = format!(\"{}\");\n",
                path.replace('{', "{").replace('}', "}")
            ));
            content.push_str(&format!("        let mut request = self.build_request({}, &path);\n", method));
        } else {
            content.push_str(&format!(
                "        let mut request = self.build_request({}, \"{}\");\n",
                method, path
            ));
        }

        // Add query parameters
        let query_params: Vec<_> = endpoint
            .params
            .iter()
            .filter(|p| p.location == ParamLocation::Query)
            .collect();
        if !query_params.is_empty() {
            content.push_str("        let mut query_params = vec![];\n");
            for param in query_params {
                if param.required {
                    content.push_str(&format!(
                        "        query_params.push((\"{}\", {}.clone()));\n",
                        param.name, param.rust_name
                    ));
                } else {
                    content.push_str(&format!(
                        "        if let Some(v) = &{} {{ query_params.push((\"{}\", v.clone())); }}\n",
                        param.rust_name, param.name
                    ));
                }
            }
            content.push_str("        request = request.query(&query_params);\n");
        }

        // Add request body
        if endpoint.request_body_type.is_some() {
            content.push_str("        request = request.json(body);\n");
        }

        // Send request
        content.push_str(
            r#"
        let response = request.send().await?;
        let status = response.status();

        if !status.is_success() {
            let message = response.text().await.unwrap_or_default();
            return Err(ApiError::Api {
                status: status.as_u16(),
                message,
            });
        }

"#,
        );

        // Parse response
        if endpoint.response_type == "()" {
            content.push_str("        Ok(())\n");
        } else {
            content.push_str("        let data = response.json().await?;\n");
            content.push_str("        Ok(data)\n");
        }

        content.push_str("    }\n");

        content
    }
}

impl ClientGenerator for RustClientGenerator {
    fn generate(&self, spec: &OpenAPI) -> Result<GeneratedClient> {
        let schemas = self.extract_schemas(spec);
        let endpoints = self.extract_endpoints(spec);

        let mut files = vec![];

        // Cargo.toml
        files.push(GeneratedFile {
            path: "Cargo.toml".to_string(),
            content: self.generate_cargo_toml(),
        });

        // src/lib.rs
        files.push(GeneratedFile {
            path: "src/lib.rs".to_string(),
            content: self.generate_lib_rs(spec, &schemas, &endpoints),
        });

        Ok(GeneratedClient {
            name: self.config.package_name.clone(),
            files,
            language: ClientLanguage::Rust,
        })
    }
}

// ============ Helper Types ============

#[derive(Debug)]
struct SchemaInfo {
    name: String,
    rust_name: String,
    description: Option<String>,
    fields: Vec<FieldInfo>,
    is_enum: bool,
    enum_values: Vec<String>,
}

#[derive(Debug)]
struct FieldInfo {
    name: String,
    rust_name: String,
    rust_type: String,
    is_required: bool,
    description: Option<String>,
}

#[derive(Debug)]
struct EndpointInfo {
    path: String,
    method: String,
    function_name: String,
    description: Option<String>,
    params: Vec<ParamInfo>,
    request_body_type: Option<String>,
    response_type: String,
}

#[derive(Debug)]
struct ParamInfo {
    name: String,
    rust_name: String,
    rust_type: String,
    location: ParamLocation,
    required: bool,
}

#[derive(Debug, PartialEq)]
enum ParamLocation {
    Path,
    Query,
}

// ============ Helper Functions ============

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' || c == ' ' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();

    for (i, c) in s.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_ascii_lowercase());
        } else if c == '-' || c == ' ' {
            result.push('_');
        } else {
            result.push(c);
        }
    }

    result
}

fn path_to_function_name(path: &str) -> String {
    path.trim_start_matches('/')
        .replace('/', "_")
        .replace('{', "")
        .replace('}', "")
        .replace('-', "_")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_openapi() -> OpenAPI {
        let json = r##"{
            "openapi": "3.0.3",
            "info": {
                "title": "Test API",
                "version": "1.0.0"
            },
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "getUsers",
                        "summary": "Get all users",
                        "responses": {
                            "200": {
                                "description": "Success",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "type": "array",
                                            "items": {
                                                "$ref": "#/components/schemas/User"
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "post": {
                        "operationId": "createUser",
                        "summary": "Create a user",
                        "requestBody": {
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "$ref": "#/components/schemas/CreateUserRequest"
                                    }
                                }
                            }
                        },
                        "responses": {
                            "201": {
                                "description": "Created",
                                "content": {
                                    "application/json": {
                                        "schema": {
                                            "$ref": "#/components/schemas/User"
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "required": ["id", "email"],
                        "properties": {
                            "id": { "type": "string" },
                            "email": { "type": "string" },
                            "userName": { "type": "string" }
                        }
                    },
                    "CreateUserRequest": {
                        "type": "object",
                        "required": ["email"],
                        "properties": {
                            "email": { "type": "string" },
                            "userName": { "type": "string" }
                        }
                    }
                }
            }
        }"##;
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_rust_client_generator() {
        let generator = RustClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();

        assert_eq!(client.name, "test-api-client");
        assert!(matches!(client.language, ClientLanguage::Rust));
        assert!(client.files.iter().any(|f| f.path == "Cargo.toml"));
        assert!(client.files.iter().any(|f| f.path == "src/lib.rs"));
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("user_name"), "UserName");
        assert_eq!(to_pascal_case("userId"), "UserId");
        assert_eq!(to_pascal_case("user-profile"), "UserProfile");
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("userName"), "user_name");
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("user-id"), "user_id");
    }

    #[test]
    fn test_path_to_function_name() {
        assert_eq!(path_to_function_name("/users"), "users");
        assert_eq!(path_to_function_name("/users/{id}"), "users_id");
        assert_eq!(path_to_function_name("/api/v1/users"), "api_v1_users");
    }

    #[test]
    fn test_generated_lib_contains_types() {
        let generator = RustClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();
        let lib_rs = client.files.iter().find(|f| f.path == "src/lib.rs").unwrap();

        assert!(lib_rs.content.contains("pub struct User"));
        assert!(lib_rs.content.contains("pub struct CreateUserRequest"));
        assert!(lib_rs.content.contains("pub struct ApiClient"));
    }

    #[test]
    fn test_generated_lib_contains_methods() {
        let generator = RustClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();
        let lib_rs = client.files.iter().find(|f| f.path == "src/lib.rs").unwrap();

        assert!(lib_rs.content.contains("async fn get_users"));
        assert!(lib_rs.content.contains("async fn create_user"));
    }
}
