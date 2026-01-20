//! Theme Inheritance with Brand Overrides
//!
//! Provides a sophisticated theme inheritance system that respects brand governance
//! while allowing appropriate customization at different levels.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::brand_pack::BrandPack;
use crate::app_pack::AppPack;
use crate::governance::TokenGovernance;
use crate::error::{BrandingError, BrandingResult, LockLevel};

/// Theme inheritance manager
#[derive(Debug)]
pub struct ThemeInheritance {
    /// Inheritance chain
    chain: InheritanceChain,

    /// Token governance
    governance: TokenGovernance,

    /// Resolved tokens cache
    resolved: HashMap<String, ResolvedToken>,
}

impl ThemeInheritance {
    /// Create a new inheritance manager from a brand pack
    pub fn from_brand(brand: &BrandPack) -> Self {
        let chain = InheritanceChain::new()
            .with_brand(brand.clone());

        let governance = TokenGovernance::from_brand(brand);

        Self {
            chain,
            governance,
            resolved: HashMap::new(),
        }
    }

    /// Add an app pack layer
    pub fn with_app(mut self, app: &AppPack) -> Self {
        self.chain = self.chain.with_app(app.clone());
        self
    }

    /// Add a theme layer
    pub fn with_theme(mut self, name: impl Into<String>, tokens: HashMap<String, serde_json::Value>) -> Self {
        self.chain = self.chain.with_theme(name, tokens);
        self
    }

    /// Resolve a token value
    pub fn resolve(&mut self, token_path: &str) -> BrandingResult<ResolvedToken> {
        // Check cache
        if let Some(resolved) = self.resolved.get(token_path) {
            return Ok(resolved.clone());
        }

        // Resolve through inheritance chain
        let resolved = self.chain.resolve(token_path, &self.governance)?;

        // Cache the result
        self.resolved.insert(token_path.to_string(), resolved.clone());

        Ok(resolved)
    }

    /// Resolve all tokens
    pub fn resolve_all(&mut self) -> BrandingResult<HashMap<String, ResolvedToken>> {
        let paths = self.chain.all_token_paths();
        let mut resolved = HashMap::new();

        for path in paths {
            resolved.insert(path.clone(), self.resolve(&path)?);
        }

        self.resolved = resolved.clone();
        Ok(resolved)
    }

    /// Get the effective value for a token
    pub fn get_value(&mut self, token_path: &str) -> BrandingResult<serde_json::Value> {
        let resolved = self.resolve(token_path)?;
        Ok(resolved.value)
    }

    /// Check if a token can be overridden at a specific level
    pub fn can_override_at(&self, token_path: &str, level: InheritanceLevel) -> bool {
        let lock = self.governance.get_lock(token_path);

        match lock {
            None => true,
            Some(lock) => {
                match (&lock.level, &level) {
                    (LockLevel::Brand, _) => false,
                    (LockLevel::Organization, InheritanceLevel::App) => false,
                    (LockLevel::Organization, InheritanceLevel::Theme) => false,
                    (LockLevel::Organization, InheritanceLevel::Component) => false,
                    (LockLevel::App, InheritanceLevel::Theme) => false,
                    (LockLevel::App, InheritanceLevel::Component) => false,
                    _ => true,
                }
            }
        }
    }

    /// Apply overrides from a context
    pub fn apply_context(&mut self, context: OverrideContext) -> BrandingResult<()> {
        for (path, value) in context.overrides {
            if self.can_override_at(&path, context.level) {
                self.chain.apply_override(&path, value, context.level)?;
                // Clear cache for this token
                self.resolved.remove(&path);
            } else {
                return Err(BrandingError::TokenLocked {
                    token: path,
                    level: LockLevel::Brand,
                });
            }
        }
        Ok(())
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.resolved.clear();
    }

    /// Get the inheritance chain for debugging
    pub fn chain(&self) -> &InheritanceChain {
        &self.chain
    }
}

/// Inheritance chain - ordered list of token sources
#[derive(Debug, Clone, Default)]
pub struct InheritanceChain {
    /// Layers in order of precedence (lowest to highest)
    layers: Vec<InheritanceLayer>,
}

impl InheritanceChain {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self { layers: Vec::new() }
    }

    /// Add brand layer (base)
    pub fn with_brand(mut self, brand: BrandPack) -> Self {
        self.layers.push(InheritanceLayer {
            name: "brand".into(),
            level: InheritanceLevel::Brand,
            tokens: brand_to_tokens(&brand),
        });
        self
    }

    /// Add app layer
    pub fn with_app(mut self, app: AppPack) -> Self {
        self.layers.push(InheritanceLayer {
            name: format!("app:{}", app.id),
            level: InheritanceLevel::App,
            tokens: app_to_tokens(&app),
        });
        self
    }

    /// Add theme layer
    pub fn with_theme(mut self, name: impl Into<String>, tokens: HashMap<String, serde_json::Value>) -> Self {
        self.layers.push(InheritanceLayer {
            name: format!("theme:{}", name.into()),
            level: InheritanceLevel::Theme,
            tokens,
        });
        self
    }

    /// Add component layer
    pub fn with_component(mut self, name: impl Into<String>, tokens: HashMap<String, serde_json::Value>) -> Self {
        self.layers.push(InheritanceLayer {
            name: format!("component:{}", name.into()),
            level: InheritanceLevel::Component,
            tokens,
        });
        self
    }

    /// Resolve a token through the chain
    pub fn resolve(&self, path: &str, governance: &TokenGovernance) -> BrandingResult<ResolvedToken> {
        let mut value = None;
        let mut source = None;
        let mut level = InheritanceLevel::Brand;

        // Walk through layers from lowest to highest precedence
        for layer in &self.layers {
            if let Some(token_value) = layer.tokens.get(path) {
                // Check if this layer can override
                if governance.can_override(path) || layer.level == InheritanceLevel::Brand {
                    value = Some(token_value.clone());
                    source = Some(layer.name.clone());
                    level = layer.level;
                }
            }
        }

        match value {
            Some(v) => Ok(ResolvedToken {
                path: path.to_string(),
                value: v,
                source: source.unwrap_or_default(),
                level,
                locked: !governance.can_override(path),
            }),
            None => Err(BrandingError::InheritanceError(
                format!("Token '{}' not found in inheritance chain", path)
            )),
        }
    }

    /// Get all token paths in the chain
    pub fn all_token_paths(&self) -> Vec<String> {
        let mut paths = std::collections::HashSet::new();
        for layer in &self.layers {
            paths.extend(layer.tokens.keys().cloned());
        }
        paths.into_iter().collect()
    }

    /// Apply an override at a specific level
    pub fn apply_override(
        &mut self,
        path: &str,
        value: serde_json::Value,
        level: InheritanceLevel,
    ) -> BrandingResult<()> {
        // Find or create the layer for this level
        let layer = self.layers.iter_mut().find(|l| l.level == level);

        if let Some(layer) = layer {
            layer.tokens.insert(path.to_string(), value);
        } else {
            // Create a new layer
            self.layers.push(InheritanceLayer {
                name: format!("{:?}", level).to_lowercase(),
                level,
                tokens: {
                    let mut tokens = HashMap::new();
                    tokens.insert(path.to_string(), value);
                    tokens
                },
            });
            // Re-sort layers by precedence
            self.layers.sort_by_key(|l| l.level);
        }

        Ok(())
    }
}

/// A single layer in the inheritance chain
#[derive(Debug, Clone)]
pub struct InheritanceLayer {
    /// Layer name (for debugging)
    pub name: String,

    /// Inheritance level
    pub level: InheritanceLevel,

    /// Tokens at this level
    pub tokens: HashMap<String, serde_json::Value>,
}

/// Inheritance level (determines precedence)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InheritanceLevel {
    /// Base brand tokens (lowest precedence)
    Brand = 0,
    /// Organization-level overrides
    Organization = 1,
    /// Application-level overrides
    App = 2,
    /// Theme-level overrides
    Theme = 3,
    /// Component-level overrides (highest precedence)
    Component = 4,
}

/// A resolved token with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResolvedToken {
    /// Token path
    pub path: String,

    /// Resolved value
    pub value: serde_json::Value,

    /// Source layer name
    pub source: String,

    /// Level where this value came from
    pub level: InheritanceLevel,

    /// Whether this token is locked
    pub locked: bool,
}

/// Override context for applying changes
#[derive(Debug, Clone)]
pub struct OverrideContext {
    /// Context name
    pub name: String,

    /// Override level
    pub level: InheritanceLevel,

    /// Overrides to apply
    pub overrides: HashMap<String, serde_json::Value>,
}

impl OverrideContext {
    /// Create a new theme context
    pub fn theme(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: InheritanceLevel::Theme,
            overrides: HashMap::new(),
        }
    }

    /// Create a new component context
    pub fn component(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            level: InheritanceLevel::Component,
            overrides: HashMap::new(),
        }
    }

    /// Add an override
    pub fn with_override(mut self, path: impl Into<String>, value: serde_json::Value) -> Self {
        self.overrides.insert(path.into(), value);
        self
    }

    /// Add color override
    pub fn color(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        let path = format!("colors.{}", name.into());
        self.with_override(path, serde_json::json!(value.into()))
    }

    /// Add spacing override
    pub fn spacing(self, name: impl Into<String>, value: f32) -> Self {
        let path = format!("spacing.{}", name.into());
        self.with_override(path, serde_json::json!(value))
    }
}

/// Convert a brand pack to token map
fn brand_to_tokens(brand: &BrandPack) -> HashMap<String, serde_json::Value> {
    let mut tokens = HashMap::new();

    // Colors
    tokens.insert("colors.primary".into(), serde_json::json!(brand.colors.primary.value));
    tokens.insert("colors.secondary".into(), serde_json::json!(brand.colors.secondary.value));
    tokens.insert("colors.accent".into(), serde_json::json!(brand.colors.accent.value));

    if let Some(ref bg) = brand.colors.background {
        tokens.insert("colors.background".into(), serde_json::json!(bg.value));
    }
    if let Some(ref fg) = brand.colors.foreground {
        tokens.insert("colors.foreground".into(), serde_json::json!(fg.value));
    }

    for (name, color) in &brand.colors.custom {
        tokens.insert(format!("colors.{}", name), serde_json::json!(color.value));
    }

    // Typography
    tokens.insert(
        "typography.primary_family".into(),
        serde_json::json!(brand.typography.primary_family.name),
    );
    if let Some(ref mono) = brand.typography.mono_family {
        tokens.insert("typography.mono_family".into(), serde_json::json!(mono.name));
    }
    tokens.insert(
        "typography.base_size".into(),
        serde_json::json!(brand.typography.scale.base_size),
    );

    // Custom tokens
    for (name, value) in &brand.tokens.spacing {
        tokens.insert(format!("spacing.{}", name), serde_json::json!(value));
    }
    for (name, value) in &brand.tokens.radius {
        tokens.insert(format!("radius.{}", name), serde_json::json!(value));
    }
    for (name, value) in &brand.tokens.shadows {
        tokens.insert(format!("shadows.{}", name), serde_json::json!(value));
    }

    tokens
}

/// Convert an app pack to token map
fn app_to_tokens(app: &AppPack) -> HashMap<String, serde_json::Value> {
    let mut tokens = HashMap::new();

    // Color overrides
    for (name, color) in &app.colors.overrides {
        tokens.insert(format!("colors.{}", name), serde_json::json!(color.value));
    }

    // Custom colors
    for (name, color) in &app.colors.custom {
        tokens.insert(format!("colors.{}", name), serde_json::json!(color.value));
    }

    // Typography overrides
    if let Some(ref font) = app.typography.primary_family {
        tokens.insert("typography.primary_family".into(), serde_json::json!(font));
    }

    // Token overrides
    for (name, value) in &app.tokens.spacing {
        tokens.insert(format!("spacing.{}", name), serde_json::json!(value));
    }
    for (name, value) in &app.tokens.radius {
        tokens.insert(format!("radius.{}", name), serde_json::json!(value));
    }

    tokens
}

/// Theme inheritance builder
#[derive(Debug, Default)]
pub struct ThemeInheritanceBuilder {
    brand: Option<BrandPack>,
    app: Option<AppPack>,
    themes: Vec<(String, HashMap<String, serde_json::Value>)>,
}

impl ThemeInheritanceBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn brand(mut self, brand: BrandPack) -> Self {
        self.brand = Some(brand);
        self
    }

    pub fn app(mut self, app: AppPack) -> Self {
        self.app = Some(app);
        self
    }

    pub fn theme(mut self, name: impl Into<String>, tokens: HashMap<String, serde_json::Value>) -> Self {
        self.themes.push((name.into(), tokens));
        self
    }

    pub fn build(self) -> BrandingResult<ThemeInheritance> {
        let brand = self.brand.ok_or_else(|| {
            BrandingError::InheritanceError("Brand pack is required".into())
        })?;

        let mut inheritance = ThemeInheritance::from_brand(&brand);

        if let Some(app) = self.app {
            inheritance = inheritance.with_app(&app);
        }

        for (name, tokens) in self.themes {
            inheritance = inheritance.with_theme(name, tokens);
        }

        Ok(inheritance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inheritance_chain() {
        let brand = BrandPack::new("Test Brand");
        let mut inheritance = ThemeInheritance::from_brand(&brand);

        let resolved = inheritance.resolve("colors.primary").unwrap();
        assert_eq!(resolved.source, "brand");
    }

    #[test]
    fn test_app_override() {
        let brand = BrandPack::new("Test Brand");
        let mut app = AppPack::new("test-app");
        app.colors.overrides.insert(
            "secondary".into(),
            crate::brand_pack::BrandColor::new("#00FF00"),
        );

        let mut inheritance = ThemeInheritance::from_brand(&brand).with_app(&app);

        let resolved = inheritance.resolve("colors.secondary").unwrap();
        assert_eq!(resolved.value, serde_json::json!("#00FF00"));
        assert_eq!(resolved.level, InheritanceLevel::App);
    }

    #[test]
    fn test_locked_token_override() {
        let mut brand = BrandPack::new("Test Brand");
        brand.colors.primary = crate::brand_pack::BrandColor::new("#FF0000").locked();

        let inheritance = ThemeInheritance::from_brand(&brand);

        assert!(!inheritance.can_override_at("colors.primary", InheritanceLevel::App));
        assert!(!inheritance.can_override_at("colors.primary", InheritanceLevel::Theme));
    }

    #[test]
    fn test_override_context() {
        let brand = BrandPack::new("Test Brand");
        let mut inheritance = ThemeInheritance::from_brand(&brand);

        let context = OverrideContext::theme("dark")
            .color("background", "#000000")
            .spacing("md", 24.0);

        inheritance.apply_context(context).unwrap();

        let resolved = inheritance.resolve("colors.background").unwrap();
        assert_eq!(resolved.value, serde_json::json!("#000000"));
    }

    #[test]
    fn test_builder() {
        let brand = BrandPack::new("Test Brand");
        let app = AppPack::new("test-app");

        let inheritance = ThemeInheritanceBuilder::new()
            .brand(brand)
            .app(app)
            .build()
            .unwrap();

        assert_eq!(inheritance.chain.layers.len(), 2);
    }
}
