//! Canonical ID Generator and Naming Service
//!
//! Enforces namespace rules and generates non-colliding identifiers for
//! OxideKit plugins, components, and packages.
//!
//! # Namespaces
//!
//! - `ui.*` - UI components
//! - `native.*` - Native platform integrations
//! - `auth.*` - Authentication plugins
//! - `db.*` - Database plugins
//! - `compat.*` - Compatibility layers (reserved)
//! - `core.*` - Core framework (reserved)

use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use thiserror::Error;

/// Errors in naming operations
#[derive(Error, Debug)]
pub enum NamingError {
    /// Invalid namespace
    #[error("Invalid namespace: {0}. Valid namespaces are: ui, native, auth, db, io, crypto, media, compat")]
    InvalidNamespace(String),

    /// Reserved namespace
    #[error("Namespace '{0}' is reserved for official plugins")]
    ReservedNamespace(String),

    /// Invalid identifier format
    #[error("Invalid identifier format: {0}")]
    InvalidFormat(String),

    /// ID collision
    #[error("ID '{0}' already exists")]
    IdCollision(String),

    /// Name too long
    #[error("Name '{0}' exceeds maximum length of {1} characters")]
    NameTooLong(String, usize),

    /// Name too short
    #[error("Name '{0}' must be at least {1} characters")]
    NameTooShort(String, usize),
}

/// Standard namespaces for OxideKit
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Namespace {
    /// UI components
    Ui,
    /// Native platform integrations
    Native,
    /// Authentication and authorization
    Auth,
    /// Database and storage
    Db,
    /// Input/output and file system
    Io,
    /// Cryptography
    Crypto,
    /// Media (audio, video, images)
    Media,
    /// Networking
    Net,
    /// Compatibility layers (reserved)
    Compat,
    /// Core framework (reserved)
    Core,
    /// Community namespace
    Community,
    /// Custom/user namespace
    Custom,
}

impl Namespace {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Namespace::Ui => "ui",
            Namespace::Native => "native",
            Namespace::Auth => "auth",
            Namespace::Db => "db",
            Namespace::Io => "io",
            Namespace::Crypto => "crypto",
            Namespace::Media => "media",
            Namespace::Net => "net",
            Namespace::Compat => "compat",
            Namespace::Core => "core",
            Namespace::Community => "community",
            Namespace::Custom => "custom",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "ui" => Some(Namespace::Ui),
            "native" => Some(Namespace::Native),
            "auth" => Some(Namespace::Auth),
            "db" => Some(Namespace::Db),
            "io" => Some(Namespace::Io),
            "crypto" => Some(Namespace::Crypto),
            "media" => Some(Namespace::Media),
            "net" => Some(Namespace::Net),
            "compat" => Some(Namespace::Compat),
            "core" => Some(Namespace::Core),
            "community" => Some(Namespace::Community),
            "custom" => Some(Namespace::Custom),
            _ => None,
        }
    }

    /// Check if this namespace is reserved
    pub fn is_reserved(&self) -> bool {
        matches!(self, Namespace::Core | Namespace::Compat)
    }

    /// Get all namespaces
    pub fn all() -> &'static [Namespace] {
        &[
            Namespace::Ui,
            Namespace::Native,
            Namespace::Auth,
            Namespace::Db,
            Namespace::Io,
            Namespace::Crypto,
            Namespace::Media,
            Namespace::Net,
            Namespace::Compat,
            Namespace::Core,
            Namespace::Community,
            Namespace::Custom,
        ]
    }
}

impl std::fmt::Display for Namespace {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A canonical identifier for OxideKit resources
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CanonicalId {
    /// Primary namespace
    pub namespace: Namespace,
    /// Sub-category within namespace
    pub category: Option<String>,
    /// Resource name
    pub name: String,
}

impl CanonicalId {
    /// Create a new canonical ID
    pub fn new(namespace: Namespace, name: &str) -> Self {
        Self {
            namespace,
            category: None,
            name: name.to_string(),
        }
    }

    /// Create with category
    pub fn with_category(namespace: Namespace, category: &str, name: &str) -> Self {
        Self {
            namespace,
            category: Some(category.to_string()),
            name: name.to_string(),
        }
    }

    /// Parse from dot-notation string (e.g., "ui.tables.advanced")
    pub fn parse(s: &str) -> Result<Self, NamingError> {
        let parts: Vec<&str> = s.split('.').collect();

        if parts.len() < 2 {
            return Err(NamingError::InvalidFormat(
                "ID must have at least namespace.name format".to_string(),
            ));
        }

        let namespace = Namespace::from_str(parts[0])
            .ok_or_else(|| NamingError::InvalidNamespace(parts[0].to_string()))?;

        if parts.len() == 2 {
            Ok(Self::new(namespace, parts[1]))
        } else {
            // namespace.category.name or namespace.cat1.cat2...name
            let name = parts.last().unwrap();
            let category = parts[1..parts.len() - 1].join(".");
            Ok(Self::with_category(namespace, &category, name))
        }
    }

    /// Convert to dot-notation string
    pub fn to_string(&self) -> String {
        match &self.category {
            Some(cat) => format!("{}.{}.{}", self.namespace, cat, self.name),
            None => format!("{}.{}", self.namespace, self.name),
        }
    }

    /// Convert to crate name (snake_case with oxide_ prefix)
    pub fn to_crate_name(&self) -> String {
        let base = match &self.category {
            Some(cat) => format!("oxide_{}_{}", self.namespace, cat.replace('.', "_")),
            None => format!("oxide_{}", self.namespace),
        };
        format!("{}_{}", base, self.name.replace('-', "_"))
    }

    /// Convert to package name (kebab-case)
    pub fn to_package_name(&self) -> String {
        match &self.category {
            Some(cat) => format!(
                "oxide-{}-{}-{}",
                self.namespace,
                cat.replace('.', "-"),
                self.name
            ),
            None => format!("oxide-{}-{}", self.namespace, self.name),
        }
    }

    /// Convert to repository name
    pub fn to_repo_name(&self) -> String {
        match &self.category {
            Some(cat) => format!(
                "oxidekit-extensions/{}-{}-{}",
                self.namespace,
                cat.replace('.', "-"),
                self.name
            ),
            None => format!("oxidekit-extensions/{}-{}", self.namespace, self.name),
        }
    }
}

impl std::fmt::Display for CanonicalId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

/// ID generation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdSuggestion {
    /// Suggested canonical ID
    pub id: CanonicalId,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Reasoning for this suggestion
    pub reasoning: String,
    /// Alternative suggestions
    pub alternatives: Vec<CanonicalId>,
    /// Derived names
    pub derived_names: DerivedNames,
}

/// Names derived from a canonical ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DerivedNames {
    /// Crate name (e.g., oxide_ui_tables_advanced)
    pub crate_name: String,
    /// Package name (e.g., oxide-ui-tables-advanced)
    pub package_name: String,
    /// Repository name (e.g., oxidekit-extensions/ui-tables-advanced)
    pub repo_name: String,
    /// Module name (e.g., tables_advanced)
    pub module_name: String,
    /// Struct name (e.g., TablesAdvanced)
    pub struct_name: String,
}

impl DerivedNames {
    /// Create from a canonical ID
    pub fn from_id(id: &CanonicalId) -> Self {
        let module_name = id.name.replace('-', "_");
        let struct_name = to_pascal_case(&id.name);

        Self {
            crate_name: id.to_crate_name(),
            package_name: id.to_package_name(),
            repo_name: id.to_repo_name(),
            module_name,
            struct_name,
        }
    }
}

/// ID generator with collision detection
pub struct IdGenerator {
    /// Existing IDs (for collision detection)
    existing_ids: HashSet<String>,
    /// Reserved prefixes
    reserved_prefixes: HashSet<String>,
    /// Name validation regex
    name_regex: Regex,
}

impl IdGenerator {
    /// Create a new ID generator
    pub fn new() -> Self {
        let mut reserved = HashSet::new();
        reserved.insert("core".to_string());
        reserved.insert("compat".to_string());
        reserved.insert("oxide".to_string());
        reserved.insert("oxidekit".to_string());

        Self {
            existing_ids: HashSet::new(),
            reserved_prefixes: reserved,
            // Allow lowercase letters, numbers, and hyphens, starting with a letter
            name_regex: Regex::new(r"^[a-z][a-z0-9-]*$").unwrap(),
        }
    }

    /// Load existing IDs from registry
    pub fn load_existing(&mut self, ids: impl IntoIterator<Item = String>) {
        self.existing_ids.extend(ids);
    }

    /// Check if an ID exists
    pub fn exists(&self, id: &str) -> bool {
        self.existing_ids.contains(id)
    }

    /// Validate a name segment
    pub fn validate_name(&self, name: &str) -> Result<(), NamingError> {
        if name.len() < 2 {
            return Err(NamingError::NameTooShort(name.to_string(), 2));
        }

        if name.len() > 50 {
            return Err(NamingError::NameTooLong(name.to_string(), 50));
        }

        if !self.name_regex.is_match(name) {
            return Err(NamingError::InvalidFormat(format!(
                "'{}' must start with a letter and contain only lowercase letters, numbers, and hyphens",
                name
            )));
        }

        Ok(())
    }

    /// Validate a canonical ID
    pub fn validate(&self, id: &CanonicalId) -> Result<(), NamingError> {
        // Check reserved namespaces
        if id.namespace.is_reserved() {
            return Err(NamingError::ReservedNamespace(id.namespace.to_string()));
        }

        // Validate name
        self.validate_name(&id.name)?;

        // Validate category if present
        if let Some(cat) = &id.category {
            for segment in cat.split('.') {
                self.validate_name(segment)?;
            }
        }

        // Check for collision
        let id_string = id.to_string();
        if self.existing_ids.contains(&id_string) {
            return Err(NamingError::IdCollision(id_string));
        }

        Ok(())
    }

    /// Suggest an ID for a given description
    pub fn suggest(&self, description: &str, hint_namespace: Option<Namespace>) -> IdSuggestion {
        // Analyze description to determine namespace
        let (namespace, confidence) = hint_namespace
            .map(|ns| (ns, 0.9))
            .unwrap_or_else(|| self.infer_namespace(description));

        // Generate name from description
        let name = self.generate_name(description);

        // Check for category hints
        let category = self.infer_category(description, namespace);

        let id = match category {
            Some(cat) => CanonicalId::with_category(namespace, &cat, &name),
            None => CanonicalId::new(namespace, &name),
        };

        // Generate alternatives
        let alternatives = self.generate_alternatives(&id, description);

        // Ensure no collision
        let final_id = self.ensure_unique(id);

        IdSuggestion {
            id: final_id.clone(),
            confidence,
            reasoning: self.generate_reasoning(&final_id, description),
            alternatives,
            derived_names: DerivedNames::from_id(&final_id),
        }
    }

    /// Infer namespace from description
    fn infer_namespace(&self, description: &str) -> (Namespace, f32) {
        let desc_lower = description.to_lowercase();

        let keywords = [
            (Namespace::Ui, vec!["table", "chart", "form", "button", "input", "list", "modal", "dialog", "menu", "widget", "component", "layout", "grid"]),
            (Namespace::Auth, vec!["auth", "login", "oauth", "jwt", "session", "permission", "role", "user", "identity"]),
            (Namespace::Db, vec!["database", "sql", "storage", "cache", "redis", "postgres", "sqlite", "query"]),
            (Namespace::Native, vec!["native", "platform", "os", "system", "window", "tray", "notification", "clipboard"]),
            (Namespace::Io, vec!["file", "directory", "path", "read", "write", "stream", "io"]),
            (Namespace::Crypto, vec!["crypto", "encrypt", "decrypt", "hash", "sign", "verify", "key"]),
            (Namespace::Media, vec!["audio", "video", "image", "media", "camera", "microphone", "player"]),
            (Namespace::Net, vec!["http", "websocket", "api", "request", "fetch", "network", "socket"]),
        ];

        let mut best_match = (Namespace::Community, 0.3f32);

        for (namespace, words) in keywords {
            let score = words
                .iter()
                .filter(|w| desc_lower.contains(*w))
                .count() as f32
                / words.len() as f32;

            if score > best_match.1 {
                best_match = (namespace, score.min(0.9));
            }
        }

        best_match
    }

    /// Infer category from description
    fn infer_category(&self, description: &str, namespace: Namespace) -> Option<String> {
        let desc_lower = description.to_lowercase();

        match namespace {
            Namespace::Ui => {
                if desc_lower.contains("table") || desc_lower.contains("grid") || desc_lower.contains("data") {
                    Some("tables".to_string())
                } else if desc_lower.contains("chart") || desc_lower.contains("graph") {
                    Some("charts".to_string())
                } else if desc_lower.contains("form") || desc_lower.contains("input") {
                    Some("forms".to_string())
                } else if desc_lower.contains("layout") {
                    Some("layouts".to_string())
                } else {
                    None
                }
            }
            Namespace::Auth => {
                if desc_lower.contains("oauth") || desc_lower.contains("social") {
                    Some("oauth".to_string())
                } else if desc_lower.contains("jwt") || desc_lower.contains("token") {
                    Some("jwt".to_string())
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// Generate a name from description
    fn generate_name(&self, description: &str) -> String {
        // Extract key words and convert to kebab-case
        let words: Vec<&str> = description
            .split_whitespace()
            .filter(|w| w.len() > 2)
            .filter(|w| !["the", "and", "for", "with", "that", "this"].contains(w))
            .take(3)
            .collect();

        if words.is_empty() {
            return "plugin".to_string();
        }

        words
            .iter()
            .map(|w| w.to_lowercase().replace(|c: char| !c.is_alphanumeric(), ""))
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Generate alternative IDs
    fn generate_alternatives(&self, base: &CanonicalId, description: &str) -> Vec<CanonicalId> {
        let mut alternatives = Vec::new();

        // Try different namespaces
        for ns in [Namespace::Ui, Namespace::Native, Namespace::Community] {
            if ns != base.namespace {
                let alt = CanonicalId::new(ns, &base.name);
                if self.validate(&alt).is_ok() {
                    alternatives.push(alt);
                }
            }
        }

        // Try with/without category
        if base.category.is_some() {
            let without_cat = CanonicalId::new(base.namespace, &base.name);
            if self.validate(&without_cat).is_ok() {
                alternatives.push(without_cat);
            }
        } else if let Some(cat) = self.infer_category(description, base.namespace) {
            let with_cat = CanonicalId::with_category(base.namespace, &cat, &base.name);
            if self.validate(&with_cat).is_ok() {
                alternatives.push(with_cat);
            }
        }

        alternatives.truncate(3);
        alternatives
    }

    /// Ensure an ID is unique by appending numbers if necessary
    fn ensure_unique(&self, mut id: CanonicalId) -> CanonicalId {
        let original_name = id.name.clone();
        let mut counter = 1;

        while self.exists(&id.to_string()) {
            counter += 1;
            id.name = format!("{}-{}", original_name, counter);
        }

        id
    }

    /// Generate reasoning for a suggestion
    fn generate_reasoning(&self, id: &CanonicalId, description: &str) -> String {
        format!(
            "Based on '{}': namespace '{}' selected for {} functionality, name '{}' derived from key terms",
            description,
            id.namespace,
            id.namespace.as_str(),
            id.name
        )
    }

    /// Register a new ID
    pub fn register(&mut self, id: &CanonicalId) -> Result<(), NamingError> {
        self.validate(id)?;
        self.existing_ids.insert(id.to_string());
        Ok(())
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// Convert to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_id_parse() {
        let id = CanonicalId::parse("ui.tables.advanced").unwrap();
        assert_eq!(id.namespace, Namespace::Ui);
        assert_eq!(id.category, Some("tables".to_string()));
        assert_eq!(id.name, "advanced");
    }

    #[test]
    fn test_canonical_id_to_string() {
        let id = CanonicalId::with_category(Namespace::Ui, "tables", "advanced");
        assert_eq!(id.to_string(), "ui.tables.advanced");
    }

    #[test]
    fn test_derived_names() {
        let id = CanonicalId::with_category(Namespace::Ui, "tables", "advanced");
        let names = DerivedNames::from_id(&id);

        assert_eq!(names.crate_name, "oxide_ui_tables_advanced");
        assert_eq!(names.package_name, "oxide-ui-tables-advanced");
        assert_eq!(names.struct_name, "Advanced");
    }

    #[test]
    fn test_namespace_is_reserved() {
        assert!(Namespace::Core.is_reserved());
        assert!(Namespace::Compat.is_reserved());
        assert!(!Namespace::Ui.is_reserved());
    }

    #[test]
    fn test_id_generator_validation() {
        let generator = IdGenerator::new();

        let valid = CanonicalId::new(Namespace::Ui, "my-component");
        assert!(generator.validate(&valid).is_ok());

        let reserved = CanonicalId::new(Namespace::Core, "something");
        assert!(generator.validate(&reserved).is_err());
    }

    #[test]
    fn test_id_suggestion() {
        let generator = IdGenerator::new();
        let suggestion = generator.suggest("advanced data table component", None);

        // The suggestion should produce a valid ID with some confidence
        assert!(!suggestion.id.name.is_empty());
        assert!(suggestion.confidence >= 0.0 && suggestion.confidence <= 1.0);
    }

    #[test]
    fn test_pascal_case() {
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
        assert_eq!(to_pascal_case("advanced-table"), "AdvancedTable");
    }
}
