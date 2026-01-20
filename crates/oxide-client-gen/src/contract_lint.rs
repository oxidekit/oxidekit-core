//! Contract linting and breaking change detection
//!
//! Enforces API contract rules and detects breaking changes:
//! - Naming consistency
//! - Required auth flows
//! - Standard error envelope
//! - Pagination rules
//! - Breaking change detection between versions

use crate::{ClientGenError, Result};
use openapiv3::{OpenAPI, Operation, PathItem, ReferenceOr, Schema, SchemaKind, Type};
use oxide_backend::naming::{NamingConvention, NamingValidator};
use serde::{Deserialize, Serialize};
use similar::{ChangeTag, TextDiff};
use std::collections::HashSet;

/// Contract linting rules
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LintRule {
    /// Enforce consistent naming convention
    NamingConsistency,
    /// Require operation IDs for all endpoints
    RequireOperationIds,
    /// Require descriptions for all endpoints
    RequireDescriptions,
    /// Require standard error response schema
    StandardErrorEnvelope,
    /// Require standard pagination for list endpoints
    StandardPagination,
    /// Require standard auth endpoints
    RequireAuthEndpoints,
    /// No undocumented endpoints (all must have summary/description)
    NoUndocumentedEndpoints,
}

impl LintRule {
    /// Get all available lint rules
    pub fn all() -> Vec<LintRule> {
        vec![
            LintRule::NamingConsistency,
            LintRule::RequireOperationIds,
            LintRule::RequireDescriptions,
            LintRule::StandardErrorEnvelope,
            LintRule::StandardPagination,
            LintRule::RequireAuthEndpoints,
            LintRule::NoUndocumentedEndpoints,
        ]
    }

    /// Get the display name for this rule
    pub fn name(&self) -> &'static str {
        match self {
            LintRule::NamingConsistency => "naming-consistency",
            LintRule::RequireOperationIds => "require-operation-ids",
            LintRule::RequireDescriptions => "require-descriptions",
            LintRule::StandardErrorEnvelope => "standard-error-envelope",
            LintRule::StandardPagination => "standard-pagination",
            LintRule::RequireAuthEndpoints => "require-auth-endpoints",
            LintRule::NoUndocumentedEndpoints => "no-undocumented-endpoints",
        }
    }

    /// Get the description of this rule
    pub fn description(&self) -> &'static str {
        match self {
            LintRule::NamingConsistency => "All property names must follow the configured naming convention",
            LintRule::RequireOperationIds => "All operations must have an operationId",
            LintRule::RequireDescriptions => "All operations must have a description or summary",
            LintRule::StandardErrorEnvelope => "Error responses must use the standard error schema",
            LintRule::StandardPagination => "List endpoints must use cursor-based pagination",
            LintRule::RequireAuthEndpoints => "API must include standard auth endpoints",
            LintRule::NoUndocumentedEndpoints => "All endpoints must have documentation",
        }
    }
}

/// Result of a lint check
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintResult {
    /// Whether all checks passed
    pub passed: bool,
    /// List of lint errors
    pub errors: Vec<LintError>,
    /// List of lint warnings
    pub warnings: Vec<LintWarning>,
}

impl LintResult {
    /// Create an empty passing result
    pub fn new() -> Self {
        Self {
            passed: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Add an error
    pub fn add_error(&mut self, error: LintError) {
        self.passed = false;
        self.errors.push(error);
    }

    /// Add a warning
    pub fn add_warning(&mut self, warning: LintWarning) {
        self.warnings.push(warning);
    }

    /// Merge another result into this one
    pub fn merge(&mut self, other: LintResult) {
        if !other.passed {
            self.passed = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
    }
}

impl Default for LintResult {
    fn default() -> Self {
        Self::new()
    }
}

/// A lint error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintError {
    /// The rule that was violated
    pub rule: LintRule,
    /// Path in the spec where the error occurred
    pub path: String,
    /// Error message
    pub message: String,
    /// Suggested fix (if available)
    pub suggestion: Option<String>,
}

/// A lint warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LintWarning {
    /// The rule that triggered the warning
    pub rule: LintRule,
    /// Path in the spec where the warning occurred
    pub path: String,
    /// Warning message
    pub message: String,
}

/// Contract linter configuration
#[derive(Debug, Clone)]
pub struct LinterConfig {
    /// Rules to enforce
    pub rules: Vec<LintRule>,
    /// Naming convention to enforce
    pub naming_convention: NamingConvention,
    /// Treat warnings as errors
    pub strict: bool,
}

impl Default for LinterConfig {
    fn default() -> Self {
        Self {
            rules: LintRule::all(),
            naming_convention: NamingConvention::CamelCase,
            strict: false,
        }
    }
}

/// Contract linter for OpenAPI specs
#[derive(Debug)]
pub struct ContractLinter {
    config: LinterConfig,
    naming_validator: NamingValidator,
}

impl ContractLinter {
    /// Create a new contract linter with default configuration
    pub fn new() -> Self {
        Self::with_config(LinterConfig::default())
    }

    /// Create a contract linter with custom configuration
    pub fn with_config(config: LinterConfig) -> Self {
        let naming_validator = NamingValidator::new(config.naming_convention);
        Self {
            config,
            naming_validator,
        }
    }

    /// Lint an OpenAPI specification
    pub fn lint(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        for rule in &self.config.rules {
            let rule_result = match rule {
                LintRule::NamingConsistency => self.check_naming_consistency(spec),
                LintRule::RequireOperationIds => self.check_operation_ids(spec),
                LintRule::RequireDescriptions => self.check_descriptions(spec),
                LintRule::StandardErrorEnvelope => self.check_error_envelope(spec),
                LintRule::StandardPagination => self.check_pagination(spec),
                LintRule::RequireAuthEndpoints => self.check_auth_endpoints(spec),
                LintRule::NoUndocumentedEndpoints => self.check_documentation(spec),
            };
            result.merge(rule_result);
        }

        if self.config.strict && !result.warnings.is_empty() {
            result.passed = false;
        }

        result
    }

    /// Lint from a file path
    pub fn lint_file(&self, path: impl AsRef<std::path::Path>) -> Result<LintResult> {
        let content = std::fs::read_to_string(path.as_ref())?;
        let spec: OpenAPI =
            serde_json::from_str(&content).map_err(|e| ClientGenError::OpenApiParse(e.to_string()))?;
        Ok(self.lint(&spec))
    }

    /// Check naming consistency
    fn check_naming_consistency(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        // Check schema property names
        if let Some(components) = &spec.components {
            for (schema_name, schema_ref) in &components.schemas {
                if let ReferenceOr::Item(schema) = schema_ref {
                    self.check_schema_naming(schema_name, schema, &mut result);
                }
            }
        }

        result
    }

    /// Check schema property naming
    fn check_schema_naming(&self, schema_name: &str, schema: &Schema, result: &mut LintResult) {
        if let SchemaKind::Type(Type::Object(obj)) = &schema.schema_kind {
            for prop_name in obj.properties.keys() {
                if !self.naming_validator.is_valid(prop_name) {
                    result.add_error(LintError {
                        rule: LintRule::NamingConsistency,
                        path: format!("components.schemas.{}.properties.{}", schema_name, prop_name),
                        message: format!(
                            "Property '{}' does not follow {} convention",
                            prop_name,
                            self.config.naming_convention.name()
                        ),
                        suggestion: Some(self.config.naming_convention.convert(prop_name)),
                    });
                }
            }
        }
    }

    /// Check that all operations have operation IDs
    fn check_operation_ids(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                self.check_operation_ids_for_path(path, item, &mut result);
            }
        }

        result
    }

    fn check_operation_ids_for_path(&self, path: &str, item: &PathItem, result: &mut LintResult) {
        let operations = [
            ("get", &item.get),
            ("post", &item.post),
            ("put", &item.put),
            ("delete", &item.delete),
            ("patch", &item.patch),
        ];

        for (method, op) in operations {
            if let Some(operation) = op {
                if operation.operation_id.is_none() {
                    result.add_error(LintError {
                        rule: LintRule::RequireOperationIds,
                        path: format!("paths.{}.{}", path, method),
                        message: "Operation must have an operationId".to_string(),
                        suggestion: Some(format!(
                            "Add operationId: \"{}{}\"",
                            method,
                            path.replace('/', "_").replace("{", "By_").replace("}", "")
                        )),
                    });
                }
            }
        }
    }

    /// Check that all operations have descriptions
    fn check_descriptions(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                self.check_descriptions_for_path(path, item, &mut result);
            }
        }

        result
    }

    fn check_descriptions_for_path(&self, path: &str, item: &PathItem, result: &mut LintResult) {
        let operations = [
            ("get", &item.get),
            ("post", &item.post),
            ("put", &item.put),
            ("delete", &item.delete),
            ("patch", &item.patch),
        ];

        for (method, op) in operations {
            if let Some(operation) = op {
                if operation.description.is_none() && operation.summary.is_none() {
                    result.add_warning(LintWarning {
                        rule: LintRule::RequireDescriptions,
                        path: format!("paths.{}.{}", path, method),
                        message: "Operation should have a description or summary".to_string(),
                    });
                }
            }
        }
    }

    /// Check for standard error envelope
    fn check_error_envelope(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        if let Some(components) = &spec.components {
            let has_error_schema = components.schemas.contains_key("Error")
                || components.schemas.contains_key("ApiError")
                || components.schemas.contains_key("ErrorResponse");

            if !has_error_schema {
                result.add_warning(LintWarning {
                    rule: LintRule::StandardErrorEnvelope,
                    path: "components.schemas".to_string(),
                    message: "No standard error schema found. Consider adding 'Error' or 'ApiError' schema".to_string(),
                });
            }
        }

        result
    }

    /// Check for standard pagination on list endpoints
    fn check_pagination(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        for (path, path_item) in &spec.paths.paths {
            if let ReferenceOr::Item(item) = path_item {
                if let Some(get) = &item.get {
                    // Check if this looks like a list endpoint
                    if self.looks_like_list_endpoint(path, get) {
                        self.check_pagination_params(path, get, &mut result);
                    }
                }
            }
        }

        result
    }

    fn looks_like_list_endpoint(&self, path: &str, operation: &Operation) -> bool {
        // Heuristic: path ends without {id} and returns array
        !path.ends_with('}') && {
            operation
                .responses
                .responses
                .get(&openapiv3::StatusCode::Code(200))
                .map(|resp| {
                    if let ReferenceOr::Item(response) = resp {
                        response.content.get("application/json").map(|media| {
                            media.schema.as_ref().map(|s| match s {
                                ReferenceOr::Item(schema) => {
                                    matches!(schema.schema_kind, SchemaKind::Type(Type::Array(_)))
                                }
                                _ => false,
                            })
                        })
                    } else {
                        None
                    }
                })
                .flatten()
                .flatten()
                .unwrap_or(false)
        }
    }

    fn check_pagination_params(&self, path: &str, operation: &Operation, result: &mut LintResult) {
        let param_names: HashSet<_> = operation
            .parameters
            .iter()
            .filter_map(|p| {
                if let ReferenceOr::Item(param) = p {
                    match param {
                        openapiv3::Parameter::Query { parameter_data, .. } => {
                            Some(parameter_data.name.as_str())
                        }
                        _ => None,
                    }
                } else {
                    None
                }
            })
            .collect();

        let has_cursor = param_names.contains("cursor") || param_names.contains("after");
        let has_limit = param_names.contains("limit") || param_names.contains("pageSize");

        if !has_cursor || !has_limit {
            result.add_warning(LintWarning {
                rule: LintRule::StandardPagination,
                path: format!("paths.{}.get", path),
                message: format!(
                    "List endpoint should have pagination parameters. Missing: {}",
                    if !has_cursor && !has_limit {
                        "cursor/after and limit/pageSize"
                    } else if !has_cursor {
                        "cursor/after"
                    } else {
                        "limit/pageSize"
                    }
                ),
            });
        }
    }

    /// Check for required auth endpoints
    fn check_auth_endpoints(&self, spec: &OpenAPI) -> LintResult {
        let mut result = LintResult::new();

        let required_auth_paths = [
            "/auth/login",
            "/auth/refresh",
            "/auth/logout",
        ];

        let paths: HashSet<_> = spec.paths.paths.keys().map(|s| s.as_str()).collect();

        for required_path in required_auth_paths {
            if !paths.contains(required_path) {
                result.add_warning(LintWarning {
                    rule: LintRule::RequireAuthEndpoints,
                    path: "paths".to_string(),
                    message: format!("Missing standard auth endpoint: {}", required_path),
                });
            }
        }

        result
    }

    /// Check that all endpoints are documented
    fn check_documentation(&self, spec: &OpenAPI) -> LintResult {
        // This is essentially the same as RequireDescriptions for now
        self.check_descriptions(spec)
    }
}

impl Default for ContractLinter {
    fn default() -> Self {
        Self::new()
    }
}

// ============ Breaking Change Detection ============

/// Types of breaking changes
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BreakingChange {
    /// An endpoint was removed
    EndpointRemoved { path: String, method: String },
    /// A required field was added
    RequiredFieldAdded { schema: String, field: String },
    /// A field was removed
    FieldRemoved { schema: String, field: String },
    /// A field type changed
    FieldTypeChanged { schema: String, field: String, old_type: String, new_type: String },
    /// A required parameter was added
    RequiredParameterAdded { path: String, method: String, param: String },
    /// Response type changed
    ResponseTypeChanged { path: String, method: String, old_type: String, new_type: String },
    /// Schema was removed
    SchemaRemoved { schema: String },
}

impl BreakingChange {
    /// Get a human-readable description
    pub fn description(&self) -> String {
        match self {
            BreakingChange::EndpointRemoved { path, method } => {
                format!("Endpoint {} {} was removed", method.to_uppercase(), path)
            }
            BreakingChange::RequiredFieldAdded { schema, field } => {
                format!("Required field '{}' was added to schema '{}'", field, schema)
            }
            BreakingChange::FieldRemoved { schema, field } => {
                format!("Field '{}' was removed from schema '{}'", field, schema)
            }
            BreakingChange::FieldTypeChanged { schema, field, old_type, new_type } => {
                format!(
                    "Field '{}' in schema '{}' changed type from '{}' to '{}'",
                    field, schema, old_type, new_type
                )
            }
            BreakingChange::RequiredParameterAdded { path, method, param } => {
                format!(
                    "Required parameter '{}' was added to {} {}",
                    param, method.to_uppercase(), path
                )
            }
            BreakingChange::ResponseTypeChanged { path, method, old_type, new_type } => {
                format!(
                    "Response type for {} {} changed from '{}' to '{}'",
                    method.to_uppercase(), path, old_type, new_type
                )
            }
            BreakingChange::SchemaRemoved { schema } => {
                format!("Schema '{}' was removed", schema)
            }
        }
    }
}

/// Result of a contract diff
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDiff {
    /// Breaking changes found
    pub breaking_changes: Vec<BreakingChange>,
    /// Non-breaking changes (additions, etc.)
    pub non_breaking_changes: Vec<String>,
    /// Whether there are any breaking changes
    pub has_breaking_changes: bool,
}

impl ContractDiff {
    /// Create an empty diff
    pub fn new() -> Self {
        Self {
            breaking_changes: Vec::new(),
            non_breaking_changes: Vec::new(),
            has_breaking_changes: false,
        }
    }

    /// Add a breaking change
    pub fn add_breaking(&mut self, change: BreakingChange) {
        self.has_breaking_changes = true;
        self.breaking_changes.push(change);
    }

    /// Add a non-breaking change
    pub fn add_non_breaking(&mut self, description: String) {
        self.non_breaking_changes.push(description);
    }
}

impl Default for ContractDiff {
    fn default() -> Self {
        Self::new()
    }
}

/// Compare two OpenAPI specifications and detect breaking changes
pub fn diff_contracts(old: &OpenAPI, new: &OpenAPI) -> ContractDiff {
    let mut diff = ContractDiff::new();

    // Check for removed endpoints
    check_endpoint_changes(old, new, &mut diff);

    // Check for schema changes
    check_schema_changes(old, new, &mut diff);

    diff
}

fn check_endpoint_changes(old: &OpenAPI, new: &OpenAPI, diff: &mut ContractDiff) {
    let new_paths: HashSet<_> = new.paths.paths.keys().collect();

    for (path, old_item) in &old.paths.paths {
        if !new_paths.contains(path) {
            // Entire path removed
            if let ReferenceOr::Item(item) = old_item {
                for (method, has_op) in [
                    ("get", item.get.is_some()),
                    ("post", item.post.is_some()),
                    ("put", item.put.is_some()),
                    ("delete", item.delete.is_some()),
                    ("patch", item.patch.is_some()),
                ] {
                    if has_op {
                        diff.add_breaking(BreakingChange::EndpointRemoved {
                            path: path.clone(),
                            method: method.to_string(),
                        });
                    }
                }
            }
        } else {
            // Path exists, check individual methods
            if let (ReferenceOr::Item(old_item), Some(ReferenceOr::Item(new_item))) =
                (old_item, new.paths.paths.get(path))
            {
                check_method_changes(path, old_item, new_item, diff);
            }
        }
    }

    // Check for new endpoints (non-breaking)
    for path in new.paths.paths.keys() {
        if !old.paths.paths.contains_key(path) {
            diff.add_non_breaking(format!("New endpoint added: {}", path));
        }
    }
}

fn check_method_changes(path: &str, old: &PathItem, new: &PathItem, diff: &mut ContractDiff) {
    let methods = [
        ("get", &old.get, &new.get),
        ("post", &old.post, &new.post),
        ("put", &old.put, &new.put),
        ("delete", &old.delete, &new.delete),
        ("patch", &old.patch, &new.patch),
    ];

    for (method, old_op, new_op) in methods {
        match (old_op, new_op) {
            (Some(_), None) => {
                diff.add_breaking(BreakingChange::EndpointRemoved {
                    path: path.to_string(),
                    method: method.to_string(),
                });
            }
            (None, Some(_)) => {
                diff.add_non_breaking(format!("New {} {} endpoint added", method.to_uppercase(), path));
            }
            _ => {}
        }
    }
}

fn check_schema_changes(old: &OpenAPI, new: &OpenAPI, diff: &mut ContractDiff) {
    let old_schemas = old.components.as_ref().map(|c| &c.schemas);
    let new_schemas = new.components.as_ref().map(|c| &c.schemas);

    if let (Some(old_schemas), Some(new_schemas)) = (old_schemas, new_schemas) {
        // Check for removed schemas
        for schema_name in old_schemas.keys() {
            if !new_schemas.contains_key(schema_name) {
                diff.add_breaking(BreakingChange::SchemaRemoved {
                    schema: schema_name.clone(),
                });
            }
        }

        // Check for schema property changes
        for (schema_name, old_schema_ref) in old_schemas {
            if let Some(new_schema_ref) = new_schemas.get(schema_name) {
                if let (ReferenceOr::Item(old_schema), ReferenceOr::Item(new_schema)) =
                    (old_schema_ref, new_schema_ref)
                {
                    check_schema_property_changes(schema_name, old_schema, new_schema, diff);
                }
            }
        }

        // Check for new schemas (non-breaking)
        for schema_name in new_schemas.keys() {
            if !old_schemas.contains_key(schema_name) {
                diff.add_non_breaking(format!("New schema added: {}", schema_name));
            }
        }
    }
}

fn check_schema_property_changes(
    schema_name: &str,
    old: &Schema,
    new: &Schema,
    diff: &mut ContractDiff,
) {
    if let (
        SchemaKind::Type(Type::Object(old_obj)),
        SchemaKind::Type(Type::Object(new_obj)),
    ) = (&old.schema_kind, &new.schema_kind)
    {
        let _old_required: HashSet<_> = old_obj.required.iter().collect();
        let new_required: HashSet<_> = new_obj.required.iter().collect();

        // Check for removed fields
        for field_name in old_obj.properties.keys() {
            if !new_obj.properties.contains_key(field_name) {
                diff.add_breaking(BreakingChange::FieldRemoved {
                    schema: schema_name.to_string(),
                    field: field_name.clone(),
                });
            }
        }

        // Check for new required fields (breaking for existing clients)
        for field_name in new_obj.properties.keys() {
            if !old_obj.properties.contains_key(field_name) && new_required.contains(field_name) {
                diff.add_breaking(BreakingChange::RequiredFieldAdded {
                    schema: schema_name.to_string(),
                    field: field_name.clone(),
                });
            } else if !old_obj.properties.contains_key(field_name) {
                diff.add_non_breaking(format!(
                    "Optional field '{}' added to schema '{}'",
                    field_name, schema_name
                ));
            }
        }
    }
}

/// Generate a text diff between two OpenAPI specs
pub fn generate_text_diff(old_json: &str, new_json: &str) -> String {
    let diff = TextDiff::from_lines(old_json, new_json);
    let mut output = String::new();

    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            ChangeTag::Delete => "-",
            ChangeTag::Insert => "+",
            ChangeTag::Equal => " ",
        };
        output.push_str(&format!("{}{}", sign, change));
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_openapi() -> OpenAPI {
        serde_json::from_str(
            r#"{
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
                        "required": ["id"],
                        "properties": {
                            "id": { "type": "string" },
                            "userName": { "type": "string" }
                        }
                    }
                }
            }
        }"#,
        )
        .unwrap()
    }

    fn sample_openapi_with_snake_case() -> OpenAPI {
        serde_json::from_str(
            r#"{
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
                            "user_name": { "type": "string" }
                        }
                    }
                }
            }
        }"#,
        )
        .unwrap()
    }

    #[test]
    fn test_contract_linter_creation() {
        let linter = ContractLinter::new();
        assert_eq!(linter.config.naming_convention, NamingConvention::CamelCase);
    }

    #[test]
    fn test_lint_valid_spec() {
        let linter = ContractLinter::new();
        let spec = sample_openapi();
        let result = linter.lint(&spec);

        // Should pass basic checks
        assert!(result.errors.iter().all(|e| e.rule != LintRule::NamingConsistency));
    }

    #[test]
    fn test_lint_naming_violation() {
        let linter = ContractLinter::new();
        let spec = sample_openapi_with_snake_case();
        let result = linter.lint(&spec);

        assert!(result.errors.iter().any(|e| e.rule == LintRule::NamingConsistency));
    }

    #[test]
    fn test_lint_missing_operation_id() {
        let spec: OpenAPI = serde_json::from_str(
            r#"{
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {
                "/users": {
                    "get": {
                        "responses": { "200": { "description": "OK" } }
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let linter = ContractLinter::new();
        let result = linter.lint(&spec);

        assert!(result.errors.iter().any(|e| e.rule == LintRule::RequireOperationIds));
    }

    #[test]
    fn test_diff_no_changes() {
        let spec = sample_openapi();
        let diff = diff_contracts(&spec, &spec);

        assert!(!diff.has_breaking_changes);
        assert!(diff.breaking_changes.is_empty());
    }

    #[test]
    fn test_diff_endpoint_removed() {
        let old = sample_openapi();
        let new: OpenAPI = serde_json::from_str(
            r#"{
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {}
        }"#,
        )
        .unwrap();

        let diff = diff_contracts(&old, &new);

        assert!(diff.has_breaking_changes);
        assert!(diff.breaking_changes.iter().any(|c| matches!(c, BreakingChange::EndpointRemoved { .. })));
    }

    #[test]
    fn test_diff_schema_removed() {
        let old = sample_openapi();
        let new: OpenAPI = serde_json::from_str(
            r#"{
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "getUsers",
                        "responses": { "200": { "description": "OK" } }
                    }
                }
            },
            "components": {
                "schemas": {}
            }
        }"#,
        )
        .unwrap();

        let diff = diff_contracts(&old, &new);

        assert!(diff.has_breaking_changes);
        assert!(diff.breaking_changes.iter().any(|c| matches!(c, BreakingChange::SchemaRemoved { .. })));
    }

    #[test]
    fn test_diff_field_removed() {
        let old = sample_openapi();
        let new: OpenAPI = serde_json::from_str(
            r#"{
            "openapi": "3.0.3",
            "info": { "title": "Test", "version": "1.0.0" },
            "paths": {
                "/users": {
                    "get": {
                        "operationId": "getUsers",
                        "responses": { "200": { "description": "OK" } }
                    }
                }
            },
            "components": {
                "schemas": {
                    "User": {
                        "type": "object",
                        "required": ["id"],
                        "properties": {
                            "id": { "type": "string" }
                        }
                    }
                }
            }
        }"#,
        )
        .unwrap();

        let diff = diff_contracts(&old, &new);

        assert!(diff.has_breaking_changes);
        assert!(diff.breaking_changes.iter().any(|c| matches!(c, BreakingChange::FieldRemoved { field, .. } if field == "userName")));
    }

    #[test]
    fn test_lint_rule_names() {
        assert_eq!(LintRule::NamingConsistency.name(), "naming-consistency");
        assert_eq!(LintRule::RequireOperationIds.name(), "require-operation-ids");
    }

    #[test]
    fn test_breaking_change_description() {
        let change = BreakingChange::EndpointRemoved {
            path: "/users".to_string(),
            method: "get".to_string(),
        };

        assert!(change.description().contains("GET /users"));
    }
}
