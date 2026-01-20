//! Font Registry and Typography Roles
//!
//! Manages font families and defines semantic typography roles.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

/// Font registry for managing available fonts
pub struct FontRegistry {
    /// Registered font families
    families: RwLock<HashMap<String, FontFamily>>,

    /// Typography role definitions
    roles: RwLock<HashMap<String, TypographyRole>>,
}

impl FontRegistry {
    /// Create a new empty font registry
    pub fn new() -> Self {
        Self {
            families: RwLock::new(HashMap::new()),
            roles: RwLock::new(HashMap::new()),
        }
    }

    /// Create a registry with default fonts and roles
    pub fn with_defaults() -> Self {
        let registry = Self::new();
        registry.register_default_families();
        registry.register_default_roles();
        registry
    }

    /// Register a font family
    pub fn register_family(&self, family: FontFamily) -> Result<(), FontError> {
        let mut families = self.families.write().map_err(|_| FontError::LockError)?;
        let id = family.id.clone();
        families.insert(id, family);
        Ok(())
    }

    /// Get a font family by ID
    pub fn get_family(&self, id: &str) -> Option<FontFamily> {
        self.families.read().ok()?.get(id).cloned()
    }

    /// List all registered font family IDs
    pub fn list_families(&self) -> Vec<String> {
        self.families
            .read()
            .map(|f| f.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Register a typography role
    pub fn register_role(&self, role: TypographyRole) -> Result<(), FontError> {
        let mut roles = self.roles.write().map_err(|_| FontError::LockError)?;
        let name = role.name.clone();
        roles.insert(name, role);
        Ok(())
    }

    /// Get a typography role by name
    pub fn get_role(&self, name: &str) -> Option<TypographyRole> {
        self.roles.read().ok()?.get(name).cloned()
    }

    /// List all registered role names
    pub fn list_roles(&self) -> Vec<String> {
        self.roles
            .read()
            .map(|r| r.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Resolve a role to its font properties
    pub fn resolve_role(&self, role_name: &str) -> Option<ResolvedTypography> {
        let role = self.get_role(role_name)?;
        let family = self.get_family(&role.family)?;

        Some(ResolvedTypography {
            family_name: family.name.clone(),
            fallbacks: family.fallbacks.clone(),
            size: role.size,
            weight: role.weight,
            line_height: role.line_height,
            letter_spacing: role.letter_spacing,
        })
    }

    /// Export registry to fonts.toml format
    pub fn export_fonts_toml(&self) -> Result<String, FontError> {
        let families = self.families.read().map_err(|_| FontError::LockError)?;
        let export = FontsExport {
            families: families.values().cloned().collect(),
        };
        toml::to_string_pretty(&export).map_err(|e| FontError::SerializationError(e.to_string()))
    }

    /// Export roles to typography.toml format
    pub fn export_typography_toml(&self) -> Result<String, FontError> {
        let roles = self.roles.read().map_err(|_| FontError::LockError)?;
        let export = TypographyExport {
            roles: roles.values().cloned().collect(),
        };
        toml::to_string_pretty(&export).map_err(|e| FontError::SerializationError(e.to_string()))
    }

    fn register_default_families(&self) {
        // System sans-serif
        let _ = self.register_family(FontFamily {
            id: "sans".into(),
            name: "System Sans".into(),
            source: FontSource::System,
            fallbacks: vec![
                "Inter".into(),
                "system-ui".into(),
                "-apple-system".into(),
                "sans-serif".into(),
            ],
            weights: vec![400, 500, 600, 700],
            styles: vec![FontStyle::Normal, FontStyle::Italic],
        });

        // Monospace
        let _ = self.register_family(FontFamily {
            id: "mono".into(),
            name: "System Mono".into(),
            source: FontSource::System,
            fallbacks: vec![
                "JetBrains Mono".into(),
                "Fira Code".into(),
                "SF Mono".into(),
                "monospace".into(),
            ],
            weights: vec![400, 500, 700],
            styles: vec![FontStyle::Normal],
        });

        // Serif
        let _ = self.register_family(FontFamily {
            id: "serif".into(),
            name: "System Serif".into(),
            source: FontSource::System,
            fallbacks: vec!["Georgia".into(), "Times New Roman".into(), "serif".into()],
            weights: vec![400, 700],
            styles: vec![FontStyle::Normal, FontStyle::Italic],
        });
    }

    fn register_default_roles(&self) {
        // Body text
        let _ = self.register_role(TypographyRole {
            name: "body".into(),
            description: "Default body text".into(),
            family: "sans".into(),
            size: 16.0,
            weight: 400,
            line_height: 1.5,
            letter_spacing: 0.0,
        });

        // Small body text
        let _ = self.register_role(TypographyRole {
            name: "body_small".into(),
            description: "Small body text".into(),
            family: "sans".into(),
            size: 14.0,
            weight: 400,
            line_height: 1.5,
            letter_spacing: 0.0,
        });

        // Headings
        let _ = self.register_role(TypographyRole {
            name: "heading".into(),
            description: "Page headings".into(),
            family: "sans".into(),
            size: 24.0,
            weight: 600,
            line_height: 1.25,
            letter_spacing: -0.025,
        });

        let _ = self.register_role(TypographyRole {
            name: "title".into(),
            description: "Large titles".into(),
            family: "sans".into(),
            size: 32.0,
            weight: 700,
            line_height: 1.2,
            letter_spacing: -0.025,
        });

        let _ = self.register_role(TypographyRole {
            name: "subtitle".into(),
            description: "Subtitles".into(),
            family: "sans".into(),
            size: 18.0,
            weight: 500,
            line_height: 1.4,
            letter_spacing: 0.0,
        });

        // UI elements
        let _ = self.register_role(TypographyRole {
            name: "button".into(),
            description: "Button labels".into(),
            family: "sans".into(),
            size: 14.0,
            weight: 500,
            line_height: 1.0,
            letter_spacing: 0.025,
        });

        let _ = self.register_role(TypographyRole {
            name: "label".into(),
            description: "Form labels".into(),
            family: "sans".into(),
            size: 14.0,
            weight: 500,
            line_height: 1.0,
            letter_spacing: 0.0,
        });

        let _ = self.register_role(TypographyRole {
            name: "caption".into(),
            description: "Captions and helper text".into(),
            family: "sans".into(),
            size: 12.0,
            weight: 400,
            line_height: 1.4,
            letter_spacing: 0.0,
        });

        // Code
        let _ = self.register_role(TypographyRole {
            name: "code".into(),
            description: "Code and monospace text".into(),
            family: "mono".into(),
            size: 14.0,
            weight: 400,
            line_height: 1.6,
            letter_spacing: 0.0,
        });

        let _ = self.register_role(TypographyRole {
            name: "code_small".into(),
            description: "Small code snippets".into(),
            family: "mono".into(),
            size: 12.0,
            weight: 400,
            line_height: 1.4,
            letter_spacing: 0.0,
        });
    }
}

impl Default for FontRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// A font family definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontFamily {
    /// Logical ID (e.g., "sans", "mono")
    pub id: String,

    /// Display name
    pub name: String,

    /// Font source
    pub source: FontSource,

    /// Fallback fonts in order
    pub fallbacks: Vec<String>,

    /// Available weights
    pub weights: Vec<u16>,

    /// Available styles
    pub styles: Vec<FontStyle>,
}

/// Font source type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum FontSource {
    /// System font
    System,

    /// Local asset file
    Asset { path: String },

    /// OxideKit font package
    Package { name: String, version: String },
}

/// Font style
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum FontStyle {
    Normal,
    Italic,
    Oblique,
}

/// A typography role definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypographyRole {
    /// Role name (e.g., "body", "heading")
    pub name: String,

    /// Role description
    pub description: String,

    /// Font family ID
    pub family: String,

    /// Font size in pixels
    pub size: f32,

    /// Font weight (100-900)
    pub weight: u16,

    /// Line height multiplier
    pub line_height: f32,

    /// Letter spacing (em)
    pub letter_spacing: f32,
}

/// Resolved typography properties
#[derive(Debug, Clone)]
pub struct ResolvedTypography {
    /// Actual font family name
    pub family_name: String,

    /// Fallback fonts
    pub fallbacks: Vec<String>,

    /// Font size
    pub size: f32,

    /// Font weight
    pub weight: u16,

    /// Line height
    pub line_height: f32,

    /// Letter spacing
    pub letter_spacing: f32,
}

impl ResolvedTypography {
    /// Get the full font-family string with fallbacks
    pub fn font_family_string(&self) -> String {
        let mut families = vec![self.family_name.clone()];
        families.extend(self.fallbacks.clone());
        families.join(", ")
    }
}

/// Font error types
#[derive(Debug, thiserror::Error)]
pub enum FontError {
    #[error("Font family not found: {0}")]
    FamilyNotFound(String),

    #[error("Typography role not found: {0}")]
    RoleNotFound(String),

    #[error("Failed to load font: {0}")]
    LoadError(String),

    #[error("Failed to acquire lock")]
    LockError,

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

/// Export format for fonts.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct FontsExport {
    pub families: Vec<FontFamily>,
}

/// Export format for typography.toml
#[derive(Debug, Serialize, Deserialize)]
pub struct TypographyExport {
    pub roles: Vec<TypographyRole>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_registry_defaults() {
        let registry = FontRegistry::with_defaults();
        let families = registry.list_families();

        assert!(families.contains(&"sans".to_string()));
        assert!(families.contains(&"mono".to_string()));
        assert!(families.contains(&"serif".to_string()));
    }

    #[test]
    fn test_typography_roles() {
        let registry = FontRegistry::with_defaults();
        let roles = registry.list_roles();

        assert!(roles.contains(&"body".to_string()));
        assert!(roles.contains(&"heading".to_string()));
        assert!(roles.contains(&"code".to_string()));
    }

    #[test]
    fn test_resolve_role() {
        let registry = FontRegistry::with_defaults();
        let resolved = registry.resolve_role("body").unwrap();

        assert_eq!(resolved.size, 16.0);
        assert_eq!(resolved.weight, 400);
    }

    #[test]
    fn test_export_typography_toml() {
        let registry = FontRegistry::with_defaults();
        let toml = registry.export_typography_toml().unwrap();

        assert!(toml.contains("body"));
        assert!(toml.contains("heading"));
    }
}
