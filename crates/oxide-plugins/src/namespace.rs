//! Plugin namespacing and naming conventions.
//!
//! OxideKit uses a hierarchical namespace system for plugins:
//!
//! - `ui.*` - UI components and component packs
//! - `native.*` - OS capabilities
//! - `auth.*` - Authentication services
//! - `db.*` - Database plugins
//! - `data.*` - Data management
//! - `tool.*` - Tooling plugins
//! - `theme.*` - Theme packs
//! - `design.*` - Design templates
//! - `icons.*` - Icon sets
//! - `fonts.*` - Font packs

use std::fmt;
use std::str::FromStr;
use serde::{Deserialize, Serialize};
use regex::Regex;

use crate::error::{PluginError, PluginResult};

/// Valid top-level namespaces for plugins.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Namespace {
    /// UI components and component packs
    Ui,
    /// OS capabilities (filesystem, keychain, etc.)
    Native,
    /// Authentication services
    Auth,
    /// Database plugins
    Db,
    /// Data management (query cache, etc.)
    Data,
    /// Tooling plugins (generators, linters, etc.)
    Tool,
    /// Theme packs (tokens, typography, etc.)
    Theme,
    /// Design templates (admin shells, layouts)
    Design,
    /// Icon sets
    Icons,
    /// Font packs
    Fonts,
}

impl Namespace {
    /// All valid namespace values.
    pub const ALL: &'static [Namespace] = &[
        Namespace::Ui,
        Namespace::Native,
        Namespace::Auth,
        Namespace::Db,
        Namespace::Data,
        Namespace::Tool,
        Namespace::Theme,
        Namespace::Design,
        Namespace::Icons,
        Namespace::Fonts,
    ];

    /// Get the string representation of the namespace.
    pub fn as_str(&self) -> &'static str {
        match self {
            Namespace::Ui => "ui",
            Namespace::Native => "native",
            Namespace::Auth => "auth",
            Namespace::Db => "db",
            Namespace::Data => "data",
            Namespace::Tool => "tool",
            Namespace::Theme => "theme",
            Namespace::Design => "design",
            Namespace::Icons => "icons",
            Namespace::Fonts => "fonts",
        }
    }

    /// Get a description of what this namespace is for.
    pub fn description(&self) -> &'static str {
        match self {
            Namespace::Ui => "UI components and component packs (tables, forms, charts)",
            Namespace::Native => "OS capabilities (filesystem, keychain, notifications)",
            Namespace::Auth => "Authentication services (sessions, OAuth)",
            Namespace::Db => "Database plugins (SQLite, migrations)",
            Namespace::Data => "Data management (query cache, invalidation)",
            Namespace::Tool => "Tooling plugins (generators, linters, formatters)",
            Namespace::Theme => "Theme packs (tokens, typography, color schemes)",
            Namespace::Design => "Design templates (admin shells, layouts)",
            Namespace::Icons => "Icon sets and icon packs",
            Namespace::Fonts => "Font packs and typography resources",
        }
    }

    /// Check if this namespace typically requires permissions.
    pub fn requires_permissions(&self) -> bool {
        matches!(self, Namespace::Native | Namespace::Tool)
    }
}

impl FromStr for Namespace {
    type Err = PluginError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "ui" => Ok(Namespace::Ui),
            "native" => Ok(Namespace::Native),
            "auth" => Ok(Namespace::Auth),
            "db" => Ok(Namespace::Db),
            "data" => Ok(Namespace::Data),
            "tool" => Ok(Namespace::Tool),
            "theme" => Ok(Namespace::Theme),
            "design" => Ok(Namespace::Design),
            "icons" => Ok(Namespace::Icons),
            "fonts" => Ok(Namespace::Fonts),
            _ => Err(PluginError::InvalidNamespace(s.to_string())),
        }
    }
}

impl fmt::Display for Namespace {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// A fully-qualified plugin identifier.
///
/// Plugin IDs follow the pattern `namespace.name` where:
/// - `namespace` is one of the predefined namespaces
/// - `name` can be hierarchical (e.g., `admin.modern.dark`)
///
/// # Examples
///
/// ```
/// use oxide_plugins::PluginId;
///
/// let id = PluginId::parse("ui.tables").unwrap();
/// assert_eq!(id.namespace().as_str(), "ui");
/// assert_eq!(id.name(), "tables");
///
/// let nested = PluginId::parse("theme.admin.modern.dark").unwrap();
/// assert_eq!(nested.full_name(), "theme.admin.modern.dark");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PluginId {
    namespace: Namespace,
    name: String,
    full_name: String,
}

impl PluginId {
    /// Parse a plugin ID from a string.
    pub fn parse(id: &str) -> PluginResult<Self> {
        // Validate format: must start with namespace, contain at least one dot
        let parts: Vec<&str> = id.splitn(2, '.').collect();
        if parts.len() != 2 {
            return Err(PluginError::InvalidPluginId(
                format!("{}: must be in format 'namespace.name'", id)
            ));
        }

        let namespace: Namespace = parts[0].parse()?;
        let name = parts[1].to_string();

        // Validate name
        Self::validate_name(&name)?;

        Ok(Self {
            namespace,
            name,
            full_name: id.to_string(),
        })
    }

    /// Create a new plugin ID from namespace and name.
    pub fn new(namespace: Namespace, name: &str) -> PluginResult<Self> {
        Self::validate_name(name)?;

        Ok(Self {
            namespace,
            name: name.to_string(),
            full_name: format!("{}.{}", namespace.as_str(), name),
        })
    }

    /// Validate a plugin name.
    fn validate_name(name: &str) -> PluginResult<()> {
        if name.is_empty() {
            return Err(PluginError::InvalidPluginId("name cannot be empty".to_string()));
        }

        // Name must be alphanumeric with dots and hyphens, lowercase
        let re = Regex::new(r"^[a-z][a-z0-9]*(\.[a-z][a-z0-9]*)*(-[a-z0-9]+)*$").unwrap();
        if !re.is_match(name) {
            return Err(PluginError::InvalidPluginId(
                format!("'{}': name must be lowercase alphanumeric with dots for hierarchy", name)
            ));
        }

        Ok(())
    }

    /// Get the namespace.
    pub fn namespace(&self) -> Namespace {
        self.namespace
    }

    /// Get the name (without namespace).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get the full name (including namespace).
    pub fn full_name(&self) -> &str {
        &self.full_name
    }

    /// Get the directory name for this plugin.
    ///
    /// This converts dots to directory separators.
    pub fn to_dir_path(&self) -> String {
        self.full_name.replace('.', "/")
    }

    /// Check if this ID is in the given namespace.
    pub fn is_in_namespace(&self, ns: Namespace) -> bool {
        self.namespace == ns
    }
}

impl fmt::Display for PluginId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.full_name)
    }
}

impl Serialize for PluginId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.full_name)
    }
}

impl<'de> Deserialize<'de> for PluginId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        PluginId::parse(&s).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_namespace_parsing() {
        assert_eq!(Namespace::from_str("ui").unwrap(), Namespace::Ui);
        assert_eq!(Namespace::from_str("native").unwrap(), Namespace::Native);
        assert_eq!(Namespace::from_str("UI").unwrap(), Namespace::Ui); // Case insensitive
        assert!(Namespace::from_str("invalid").is_err());
    }

    #[test]
    fn test_plugin_id_parsing() {
        let id = PluginId::parse("ui.tables").unwrap();
        assert_eq!(id.namespace(), Namespace::Ui);
        assert_eq!(id.name(), "tables");
        assert_eq!(id.full_name(), "ui.tables");

        let nested = PluginId::parse("theme.admin.modern.dark").unwrap();
        assert_eq!(nested.namespace(), Namespace::Theme);
        assert_eq!(nested.name(), "admin.modern.dark");

        // Invalid cases
        assert!(PluginId::parse("invalid").is_err()); // No namespace
        assert!(PluginId::parse("ui.").is_err()); // Empty name
        assert!(PluginId::parse(".tables").is_err()); // Empty namespace
        assert!(PluginId::parse("ui.UPPERCASE").is_err()); // Must be lowercase
    }

    #[test]
    fn test_plugin_id_dir_path() {
        let id = PluginId::parse("ui.tables").unwrap();
        assert_eq!(id.to_dir_path(), "ui/tables");

        let nested = PluginId::parse("theme.admin.modern").unwrap();
        assert_eq!(nested.to_dir_path(), "theme/admin/modern");
    }

    #[test]
    fn test_namespace_requires_permissions() {
        assert!(!Namespace::Ui.requires_permissions());
        assert!(Namespace::Native.requires_permissions());
        assert!(Namespace::Tool.requires_permissions());
        assert!(!Namespace::Theme.requires_permissions());
    }
}
