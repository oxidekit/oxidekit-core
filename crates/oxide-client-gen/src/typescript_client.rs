//! TypeScript client generation from OpenAPI specs
//!
//! Generates type-safe TypeScript clients for web applications.

use crate::{ClientConfig, ClientGenerator, ClientLanguage, GeneratedClient, GeneratedFile, Result};
use openapiv3::{OpenAPI, Operation, PathItem, ReferenceOr, Schema, SchemaKind, Type};
use std::collections::{HashMap, HashSet};

/// TypeScript client generator
#[derive(Debug)]
pub struct TypeScriptClientGenerator {
    config: ClientConfig,
    /// Generate ESM modules (default: true)
    pub esm: bool,
    /// Use fetch API (default: true, alternative: axios)
    pub use_fetch: bool,
}

impl TypeScriptClientGenerator {
    /// Create a new TypeScript client generator
    pub fn new(package_name: impl Into<String>) -> Self {
        Self {
            config: ClientConfig::new(package_name),
            esm: true,
            use_fetch: true,
        }
    }

    /// Create with full configuration
    pub fn with_config(config: ClientConfig) -> Self {
        Self {
            config,
            esm: true,
            use_fetch: true,
        }
    }

    /// Use axios instead of fetch
    pub fn with_axios(mut self) -> Self {
        self.use_fetch = false;
        self
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
            description: schema.schema_data.description.clone(),
            fields: Vec::new(),
            is_enum: false,
            enum_values: Vec::new(),
        };

        match &schema.schema_kind {
            SchemaKind::Type(Type::Object(obj)) => {
                let required: HashSet<_> = obj.required.iter().cloned().collect();

                for (field_name, field_schema) in &obj.properties {
                    let field_type = self.boxed_schema_to_ts_type(field_schema);
                    let is_required = required.contains(field_name);

                    info.fields.push(FieldInfo {
                        name: field_name.clone(),
                        ts_type: field_type,
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

    /// Convert an OpenAPI schema reference (boxed) to a TypeScript type
    fn boxed_schema_to_ts_type(&self, schema_ref: &ReferenceOr<Box<Schema>>) -> String {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                // Extract type name from reference like "#/components/schemas/User"
                reference
                    .split('/')
                    .last()
                    .unwrap_or("unknown")
                    .to_string()
            }
            ReferenceOr::Item(schema) => self.schema_kind_to_ts_type(&schema.schema_kind),
        }
    }

    /// Convert an OpenAPI schema reference to a TypeScript type
    fn schema_to_ts_type(&self, schema_ref: &ReferenceOr<Schema>) -> String {
        match schema_ref {
            ReferenceOr::Reference { reference } => {
                // Extract type name from reference like "#/components/schemas/User"
                reference
                    .split('/')
                    .last()
                    .unwrap_or("unknown")
                    .to_string()
            }
            ReferenceOr::Item(schema) => self.schema_kind_to_ts_type(&schema.schema_kind),
        }
    }

    /// Convert a schema kind to TypeScript type
    fn schema_kind_to_ts_type(&self, kind: &SchemaKind) -> String {
        match kind {
            SchemaKind::Type(Type::String(_)) => "string".to_string(),
            SchemaKind::Type(Type::Integer(_)) => "number".to_string(),
            SchemaKind::Type(Type::Number(_)) => "number".to_string(),
            SchemaKind::Type(Type::Boolean(_)) => "boolean".to_string(),
            SchemaKind::Type(Type::Array(arr)) => {
                let item_type = arr
                    .items
                    .as_ref()
                    .map(|i| self.boxed_schema_to_ts_type(i))
                    .unwrap_or_else(|| "unknown".to_string());
                format!("{}[]", item_type)
            }
            SchemaKind::Type(Type::Object(_)) => "Record<string, unknown>".to_string(),
            _ => "unknown".to_string(),
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
            ("GET", &item.get),
            ("POST", &item.post),
            ("PUT", &item.put),
            ("DELETE", &item.delete),
            ("PATCH", &item.patch),
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
            .cloned()
            .unwrap_or_else(|| format!("{}{}", method.to_lowercase(), path_to_function_name(path)));

        let mut params = Vec::new();

        // Path parameters
        for param_ref in &operation.parameters {
            if let ReferenceOr::Item(param) = param_ref {
                if let openapiv3::Parameter::Path { parameter_data, .. } = param {
                    params.push(ParamInfo {
                        name: parameter_data.name.clone(),
                        ts_type: "string".to_string(),
                        location: ParamLocation::Path,
                        required: true,
                    });
                } else if let openapiv3::Parameter::Query { parameter_data, .. } = param {
                    params.push(ParamInfo {
                        name: parameter_data.name.clone(),
                        ts_type: "string".to_string(),
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
                    media.schema.as_ref().map(|s| self.schema_to_ts_type(s))
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
                        media.schema.as_ref().map(|s| self.schema_to_ts_type(s))
                    })
                } else {
                    None
                }
            })
            .unwrap_or_else(|| "void".to_string());

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

    /// Generate package.json content
    fn generate_package_json(&self) -> String {
        let type_field = if self.esm { r#""type": "module","# } else { "" };
        let deps = if self.use_fetch {
            ""
        } else {
            r#"
  "dependencies": {
    "axios": "^1.6.0"
  },"#
        };

        format!(
            r#"{{
  "name": "{name}",
  "version": "{version}",
  "description": "{description}",
  {type_field}
  "main": "dist/index.js",
  "types": "dist/index.d.ts",{deps}
  "devDependencies": {{
    "typescript": "^5.3.0"
  }},
  "scripts": {{
    "build": "tsc",
    "prepare": "npm run build"
  }}
}}
"#,
            name = self.config.package_name,
            version = self.config.version,
            description = self.config.description,
            type_field = type_field,
            deps = deps
        )
    }

    /// Generate tsconfig.json content
    fn generate_tsconfig(&self) -> String {
        let module = if self.esm { "ESNext" } else { "CommonJS" };
        let module_resolution = if self.esm { "bundler" } else { "node" };

        format!(
            r#"{{
  "compilerOptions": {{
    "target": "ES2022",
    "module": "{module}",
    "moduleResolution": "{module_resolution}",
    "lib": ["ES2022", "DOM"],
    "declaration": true,
    "declarationMap": true,
    "sourceMap": true,
    "outDir": "./dist",
    "rootDir": "./src",
    "strict": true,
    "esModuleInterop": true,
    "skipLibCheck": true,
    "forceConsistentCasingInFileNames": true
  }},
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist"]
}}
"#,
            module = module,
            module_resolution = module_resolution
        )
    }

    /// Generate index.ts content
    fn generate_index_ts(&self, spec: &OpenAPI, schemas: &HashMap<String, SchemaInfo>, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        // Header
        content.push_str(&format!(
            r#"/**
 * {} API Client
 *
 * Generated by OxideKit Client Generator.
 * Do not edit manually.
 */

"#,
            spec.info.title
        ));

        // Generate types
        content.push_str("// ============ Types ============\n\n");
        for schema in schemas.values() {
            content.push_str(&self.generate_type(schema));
            content.push('\n');
        }

        // Error type
        content.push_str(
            r#"/** API Error */
export interface ApiError {
  status: number;
  message: string;
  details?: unknown;
}

/** API Response wrapper */
export type ApiResponse<T> = { data: T; error?: never } | { data?: never; error: ApiError };

"#,
        );

        // Generate client
        content.push_str("// ============ Client ============\n\n");
        content.push_str(&self.generate_client_class(endpoints));

        content
    }

    /// Generate a TypeScript type from schema info
    fn generate_type(&self, schema: &SchemaInfo) -> String {
        let mut content = String::new();

        if let Some(desc) = &schema.description {
            content.push_str(&format!("/** {} */\n", desc));
        }

        if schema.is_enum {
            content.push_str(&format!(
                "export type {} = {};\n",
                schema.name,
                schema
                    .enum_values
                    .iter()
                    .map(|v| format!("\"{}\"", v))
                    .collect::<Vec<_>>()
                    .join(" | ")
            ));
        } else {
            content.push_str(&format!("export interface {} {{\n", schema.name));
            for field in &schema.fields {
                if let Some(desc) = &field.description {
                    content.push_str(&format!("  /** {} */\n", desc));
                }
                let optional = if field.is_required { "" } else { "?" };
                content.push_str(&format!("  {}{}: {};\n", field.name, optional, field.ts_type));
            }
            content.push_str("}\n");
        }

        content
    }

    /// Generate the client class
    fn generate_client_class(&self, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        if self.use_fetch {
            content.push_str(&self.generate_fetch_client(endpoints));
        } else {
            content.push_str(&self.generate_axios_client(endpoints));
        }

        content
    }

    /// Generate fetch-based client
    fn generate_fetch_client(&self, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        content.push_str(
            r#"/** API Client Configuration */
export interface ClientConfig {
  baseUrl: string;
  headers?: Record<string, string>;
}

/** API Client */
export class ApiClient {
  private baseUrl: string;
  private headers: Record<string, string>;
  private authToken?: string;

  constructor(config: ClientConfig) {
    this.baseUrl = config.baseUrl.replace(/\/$/, '');
    this.headers = config.headers ?? {};
  }

  /** Set the authentication token */
  setAuthToken(token: string): void {
    this.authToken = token;
  }

  /** Clear the authentication token */
  clearAuthToken(): void {
    this.authToken = undefined;
  }

  private async request<T>(
    method: string,
    path: string,
    options?: { body?: unknown; query?: Record<string, string> }
  ): Promise<ApiResponse<T>> {
    let url = `${this.baseUrl}${path}`;

    if (options?.query) {
      const params = new URLSearchParams(options.query);
      url += `?${params.toString()}`;
    }

    const headers: Record<string, string> = {
      'Content-Type': 'application/json',
      ...this.headers,
    };

    if (this.authToken) {
      headers['Authorization'] = `Bearer ${this.authToken}`;
    }

    try {
      const response = await fetch(url, {
        method,
        headers,
        body: options?.body ? JSON.stringify(options.body) : undefined,
      });

      if (!response.ok) {
        const message = await response.text();
        return {
          error: {
            status: response.status,
            message: message || response.statusText,
          },
        };
      }

      const contentType = response.headers.get('content-type');
      if (contentType?.includes('application/json')) {
        const data = await response.json();
        return { data };
      }

      return { data: undefined as T };
    } catch (err) {
      return {
        error: {
          status: 0,
          message: err instanceof Error ? err.message : 'Unknown error',
        },
      };
    }
  }

"#,
        );

        // Generate methods for each endpoint
        for endpoint in endpoints {
            content.push_str(&self.generate_fetch_method(endpoint));
            content.push('\n');
        }

        content.push_str("}\n");

        content
    }

    /// Generate axios-based client
    fn generate_axios_client(&self, endpoints: &[EndpointInfo]) -> String {
        let mut content = String::new();

        content.push_str(
            r#"import axios, { AxiosInstance } from 'axios';

/** API Client Configuration */
export interface ClientConfig {
  baseUrl: string;
  headers?: Record<string, string>;
}

/** API Client */
export class ApiClient {
  private client: AxiosInstance;
  private authToken?: string;

  constructor(config: ClientConfig) {
    this.client = axios.create({
      baseURL: config.baseUrl.replace(/\/$/, ''),
      headers: config.headers,
    });
  }

  /** Set the authentication token */
  setAuthToken(token: string): void {
    this.authToken = token;
    this.client.defaults.headers.common['Authorization'] = `Bearer ${token}`;
  }

  /** Clear the authentication token */
  clearAuthToken(): void {
    this.authToken = undefined;
    delete this.client.defaults.headers.common['Authorization'];
  }

"#,
        );

        // Generate methods for each endpoint
        for endpoint in endpoints {
            content.push_str(&self.generate_axios_method(endpoint));
            content.push('\n');
        }

        content.push_str("}\n");

        content
    }

    /// Generate a fetch-based method for an endpoint
    fn generate_fetch_method(&self, endpoint: &EndpointInfo) -> String {
        let mut content = String::new();

        // Doc comment
        if let Some(desc) = &endpoint.description {
            content.push_str(&format!("  /** {} */\n", desc));
        }

        // Build parameter list
        let mut params = Vec::new();
        for param in &endpoint.params {
            let optional = if param.required { "" } else { "?" };
            params.push(format!("{}{}: string", param.name, optional));
        }
        if let Some(body_type) = &endpoint.request_body_type {
            params.push(format!("body: {}", body_type));
        }

        let params_str = params.join(", ");

        content.push_str(&format!(
            "  async {}({}): Promise<ApiResponse<{}>> {{\n",
            endpoint.function_name, params_str, endpoint.response_type
        ));

        // Build path with parameters
        let path = endpoint
            .params
            .iter()
            .filter(|p| p.location == ParamLocation::Path)
            .fold(endpoint.path.clone(), |path, param| {
                path.replace(&format!("{{{}}}", param.name), &format!("${{{}}}", param.name))
            });

        // Query parameters
        let query_params: Vec<_> = endpoint
            .params
            .iter()
            .filter(|p| p.location == ParamLocation::Query)
            .collect();

        if !query_params.is_empty() {
            content.push_str("    const query: Record<string, string> = {};\n");
            for param in &query_params {
                if param.required {
                    content.push_str(&format!(
                        "    query['{}'] = {};\n",
                        param.name, param.name
                    ));
                } else {
                    content.push_str(&format!(
                        "    if ({} !== undefined) query['{}'] = {};\n",
                        param.name, param.name, param.name
                    ));
                }
            }
        }

        // Call request
        let options_str = if endpoint.request_body_type.is_some() || !query_params.is_empty() {
            let mut parts = Vec::new();
            if endpoint.request_body_type.is_some() {
                parts.push("body");
            }
            if !query_params.is_empty() {
                parts.push("query");
            }
            format!(", {{ {} }}", parts.join(", "))
        } else {
            String::new()
        };

        content.push_str(&format!(
            "    return this.request<{}>('{}', `{}`{});\n",
            endpoint.response_type,
            endpoint.method,
            path,
            options_str
        ));

        content.push_str("  }\n");

        content
    }

    /// Generate an axios-based method for an endpoint
    fn generate_axios_method(&self, endpoint: &EndpointInfo) -> String {
        let mut content = String::new();

        // Doc comment
        if let Some(desc) = &endpoint.description {
            content.push_str(&format!("  /** {} */\n", desc));
        }

        // Build parameter list
        let mut params = Vec::new();
        for param in &endpoint.params {
            let optional = if param.required { "" } else { "?" };
            params.push(format!("{}{}: string", param.name, optional));
        }
        if let Some(body_type) = &endpoint.request_body_type {
            params.push(format!("body: {}", body_type));
        }

        let params_str = params.join(", ");

        content.push_str(&format!(
            "  async {}({}): Promise<ApiResponse<{}>> {{\n",
            endpoint.function_name, params_str, endpoint.response_type
        ));

        // Build path with parameters
        let path = endpoint
            .params
            .iter()
            .filter(|p| p.location == ParamLocation::Path)
            .fold(endpoint.path.clone(), |path, param| {
                path.replace(&format!("{{{}}}", param.name), &format!("${{{}}}", param.name))
            });

        content.push_str("    try {\n");
        content.push_str(&format!(
            "      const response = await this.client.{}(`{}`",
            endpoint.method.to_lowercase(),
            path
        ));

        if endpoint.request_body_type.is_some() {
            content.push_str(", body");
        }

        content.push_str(");\n");
        content.push_str("      return { data: response.data };\n");
        content.push_str("    } catch (err: any) {\n");
        content.push_str("      return {\n");
        content.push_str("        error: {\n");
        content.push_str("          status: err.response?.status ?? 0,\n");
        content.push_str("          message: err.response?.data ?? err.message,\n");
        content.push_str("        },\n");
        content.push_str("      };\n");
        content.push_str("    }\n");
        content.push_str("  }\n");

        content
    }
}

impl ClientGenerator for TypeScriptClientGenerator {
    fn generate(&self, spec: &OpenAPI) -> Result<GeneratedClient> {
        let schemas = self.extract_schemas(spec);
        let endpoints = self.extract_endpoints(spec);

        let mut files = vec![];

        // package.json
        files.push(GeneratedFile {
            path: "package.json".to_string(),
            content: self.generate_package_json(),
        });

        // tsconfig.json
        files.push(GeneratedFile {
            path: "tsconfig.json".to_string(),
            content: self.generate_tsconfig(),
        });

        // src/index.ts
        files.push(GeneratedFile {
            path: "src/index.ts".to_string(),
            content: self.generate_index_ts(spec, &schemas, &endpoints),
        });

        Ok(GeneratedClient {
            name: self.config.package_name.clone(),
            files,
            language: ClientLanguage::TypeScript,
        })
    }
}

// ============ Helper Types ============

#[derive(Debug)]
struct SchemaInfo {
    name: String,
    description: Option<String>,
    fields: Vec<FieldInfo>,
    is_enum: bool,
    enum_values: Vec<String>,
}

#[derive(Debug)]
struct FieldInfo {
    name: String,
    ts_type: String,
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
    ts_type: String,
    location: ParamLocation,
    required: bool,
}

#[derive(Debug, PartialEq)]
enum ParamLocation {
    Path,
    Query,
}

// ============ Helper Functions ============

fn path_to_function_name(path: &str) -> String {
    path.trim_start_matches('/')
        .split('/')
        .map(|segment| {
            if segment.starts_with('{') && segment.ends_with('}') {
                let name = &segment[1..segment.len() - 1];
                format!("By{}", capitalize_first(name))
            } else {
                capitalize_first(segment)
            }
        })
        .collect::<Vec<_>>()
        .join("")
}

fn capitalize_first(s: &str) -> String {
    let mut chars = s.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_openapi() -> OpenAPI {
        serde_json::from_str(
            r##"{
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
                    }
                }
            }
        }"##,
        )
        .unwrap()
    }

    #[test]
    fn test_typescript_client_generator() {
        let generator = TypeScriptClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();

        assert_eq!(client.name, "test-api-client");
        assert!(matches!(client.language, ClientLanguage::TypeScript));
        assert!(client.files.iter().any(|f| f.path == "package.json"));
        assert!(client.files.iter().any(|f| f.path == "tsconfig.json"));
        assert!(client.files.iter().any(|f| f.path == "src/index.ts"));
    }

    #[test]
    fn test_generated_index_contains_types() {
        let generator = TypeScriptClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();
        let index_ts = client.files.iter().find(|f| f.path == "src/index.ts").unwrap();

        assert!(index_ts.content.contains("export interface User"));
        assert!(index_ts.content.contains("export class ApiClient"));
    }

    #[test]
    fn test_generated_index_contains_methods() {
        let generator = TypeScriptClientGenerator::new("test-api-client");
        let spec = sample_openapi();

        let client = generator.generate(&spec).unwrap();
        let index_ts = client.files.iter().find(|f| f.path == "src/index.ts").unwrap();

        assert!(index_ts.content.contains("async getUsers"));
    }

    #[test]
    fn test_path_to_function_name() {
        assert_eq!(path_to_function_name("/users"), "Users");
        assert_eq!(path_to_function_name("/users/{id}"), "UsersById");
        assert_eq!(path_to_function_name("/api/v1/users"), "ApiV1Users");
    }
}
