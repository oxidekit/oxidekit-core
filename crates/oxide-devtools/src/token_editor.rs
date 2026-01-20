//! Design Token Editor
//!
//! Provides editing capabilities for design tokens with theme integration.
//! Supports color pickers, spacing editors, and token reference resolution.

use oxide_components::{
    ColorToken, ColorTokens, DesignTokens, RadiusTokens, SpacingTokens, Theme,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Token editor state
#[derive(Debug)]
pub struct TokenEditor {
    /// Current theme being edited
    theme: Option<Theme>,
    /// Token overrides (applied on top of theme)
    overrides: TokenOverrides,
    /// Token change history
    history: Vec<TokenChange>,
    /// Redo stack
    redo_stack: Vec<TokenChange>,
    /// Currently selected token path
    selected: Option<String>,
}

impl Default for TokenEditor {
    fn default() -> Self {
        Self::new()
    }
}

impl TokenEditor {
    /// Create a new token editor
    pub fn new() -> Self {
        Self {
            theme: None,
            overrides: TokenOverrides::new(),
            history: Vec::new(),
            redo_stack: Vec::new(),
            selected: None,
        }
    }

    /// Create with a theme
    pub fn with_theme(theme: Theme) -> Self {
        Self {
            theme: Some(theme),
            overrides: TokenOverrides::new(),
            history: Vec::new(),
            redo_stack: Vec::new(),
            selected: None,
        }
    }

    /// Set the theme
    pub fn set_theme(&mut self, theme: Theme) {
        self.theme = Some(theme);
        self.overrides.clear();
    }

    /// Get the current theme
    pub fn theme(&self) -> Option<&Theme> {
        self.theme.as_ref()
    }

    /// Select a token by path
    pub fn select(&mut self, path: impl Into<String>) {
        self.selected = Some(path.into());
    }

    /// Clear selection
    pub fn clear_selection(&mut self) {
        self.selected = None;
    }

    /// Get selected token path
    pub fn selected(&self) -> Option<&str> {
        self.selected.as_deref()
    }

    /// Resolve a token value by path
    pub fn resolve(&self, path: &str) -> Option<ResolvedToken> {
        // Check overrides first
        if let Some(value) = self.overrides.get(path) {
            return Some(ResolvedToken {
                path: path.to_string(),
                value: value.clone(),
                source: TokenSource::Override,
                category: token_category(path),
            });
        }

        // Then check theme
        if let Some(theme) = &self.theme {
            let value = resolve_token_from_theme(&theme.tokens, path)?;
            return Some(ResolvedToken {
                path: path.to_string(),
                value,
                source: TokenSource::Theme,
                category: token_category(path),
            });
        }

        None
    }

    /// Set a token override
    pub fn set_override(&mut self, path: &str, value: TokenValue) {
        let old_value = self.resolve(path).map(|t| t.value);

        self.overrides.set(path.to_string(), value.clone());

        // Record change
        self.history.push(TokenChange {
            path: path.to_string(),
            old_value,
            new_value: Some(value),
        });
        self.redo_stack.clear();
    }

    /// Remove a token override
    pub fn remove_override(&mut self, path: &str) {
        if let Some(old_value) = self.overrides.remove(path) {
            self.history.push(TokenChange {
                path: path.to_string(),
                old_value: Some(old_value),
                new_value: None,
            });
            self.redo_stack.clear();
        }
    }

    /// Clear all overrides
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
    }

    /// Check if there are any overrides
    pub fn has_overrides(&self) -> bool {
        !self.overrides.is_empty()
    }

    /// Get all overrides
    pub fn overrides(&self) -> &HashMap<String, TokenValue> {
        &self.overrides.values
    }

    /// Undo last change
    pub fn undo(&mut self) -> Option<TokenChange> {
        let change = self.history.pop()?;

        // Revert the change
        if let Some(old_value) = &change.old_value {
            self.overrides.set(change.path.clone(), old_value.clone());
        } else {
            self.overrides.remove(&change.path);
        }

        self.redo_stack.push(change.clone());
        Some(change)
    }

    /// Redo last undone change
    pub fn redo(&mut self) -> Option<TokenChange> {
        let change = self.redo_stack.pop()?;

        // Apply the change again
        if let Some(new_value) = &change.new_value {
            self.overrides.set(change.path.clone(), new_value.clone());
        } else {
            self.overrides.remove(&change.path);
        }

        self.history.push(change.clone());
        Some(change)
    }

    /// Can undo
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }

    /// Can redo
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// Get all tokens in a category
    pub fn tokens_in_category(&self, category: TokenCategory) -> Vec<TokenInfo> {
        let mut tokens = Vec::new();

        if let Some(theme) = &self.theme {
            match category {
                TokenCategory::Color => {
                    add_color_tokens(&mut tokens, &theme.tokens.color, "color");
                }
                TokenCategory::Spacing => {
                    add_spacing_tokens(&mut tokens, &theme.tokens.spacing, "spacing");
                }
                TokenCategory::Radius => {
                    add_radius_tokens(&mut tokens, &theme.tokens.radius, "radius");
                }
                TokenCategory::Typography => {
                    // Add font family tokens
                    tokens.push(TokenInfo {
                        path: "typography.font_family.sans".to_string(),
                        name: "Sans".to_string(),
                        category,
                        value: TokenValue::String(theme.tokens.typography.font_family.sans.clone()),
                        has_override: self.overrides.has("typography.font_family.sans"),
                    });
                    tokens.push(TokenInfo {
                        path: "typography.font_family.serif".to_string(),
                        name: "Serif".to_string(),
                        category,
                        value: TokenValue::String(theme.tokens.typography.font_family.serif.clone()),
                        has_override: self.overrides.has("typography.font_family.serif"),
                    });
                    tokens.push(TokenInfo {
                        path: "typography.font_family.mono".to_string(),
                        name: "Mono".to_string(),
                        category,
                        value: TokenValue::String(theme.tokens.typography.font_family.mono.clone()),
                        has_override: self.overrides.has("typography.font_family.mono"),
                    });
                }
                TokenCategory::Shadow => {
                    // Add shadow tokens
                    for size in ["none", "sm", "md", "lg", "xl"] {
                        let path = format!("shadow.{}", size);
                        tokens.push(TokenInfo {
                            path: path.clone(),
                            name: size.to_string(),
                            category,
                            value: TokenValue::String(format!("shadow-{}", size)),
                            has_override: self.overrides.has(&path),
                        });
                    }
                }
                TokenCategory::Motion => {
                    // Add duration tokens
                    for name in ["instant", "fast", "normal", "slow"] {
                        let path = format!("motion.duration.{}", name);
                        tokens.push(TokenInfo {
                            path: path.clone(),
                            name: format!("Duration {}", name),
                            category,
                            value: TokenValue::Number(match name {
                                "instant" => 50.0,
                                "fast" => 150.0,
                                "normal" => 300.0,
                                "slow" => 500.0,
                                _ => 300.0,
                            }),
                            has_override: self.overrides.has(&path),
                        });
                    }
                }
            }
        }

        tokens
    }

    /// Get token info by path
    pub fn get_token_info(&self, path: &str) -> Option<TokenInfo> {
        let resolved = self.resolve(path)?;
        Some(TokenInfo {
            path: path.to_string(),
            name: path.split('.').last().unwrap_or(path).to_string(),
            category: resolved.category,
            value: resolved.value,
            has_override: self.overrides.has(path),
        })
    }

    /// Generate OUI code for token overrides
    pub fn generate_override_code(&self) -> String {
        if self.overrides.is_empty() {
            return String::new();
        }

        let mut lines = Vec::new();
        lines.push("// Token overrides".to_string());
        lines.push("[theme.overrides]".to_string());

        // Group by category
        let mut by_category: HashMap<String, Vec<(&String, &TokenValue)>> = HashMap::new();
        for (path, value) in &self.overrides.values {
            let category = path.split('.').next().unwrap_or("other").to_string();
            by_category.entry(category).or_default().push((path, value));
        }

        for (category, items) in by_category {
            lines.push(format!("\n[theme.overrides.{}]", category));
            for (path, value) in items {
                let key = path.split('.').skip(1).collect::<Vec<_>>().join(".");
                let formatted = format_token_value(value);
                lines.push(format!("{} = {}", key, formatted));
            }
        }

        lines.join("\n")
    }
}

/// Token value types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TokenValue {
    /// Color value (hex)
    Color(String),
    /// Numeric value
    Number(f64),
    /// String value
    String(String),
    /// Spacing value
    Spacing {
        value: f64,
        x: Option<f64>,
        y: Option<f64>,
    },
    /// Shadow value
    Shadow {
        x: f64,
        y: f64,
        blur: f64,
        spread: f64,
        color: String,
    },
}

impl TokenValue {
    pub fn color(hex: impl Into<String>) -> Self {
        Self::Color(hex.into())
    }

    pub fn number(value: f64) -> Self {
        Self::Number(value)
    }

    pub fn string(s: impl Into<String>) -> Self {
        Self::String(s.into())
    }

    pub fn spacing(value: f64) -> Self {
        Self::Spacing {
            value,
            x: None,
            y: None,
        }
    }

    pub fn spacing_xy(x: f64, y: f64) -> Self {
        Self::Spacing {
            value: 0.0,
            x: Some(x),
            y: Some(y),
        }
    }
}

/// Token overrides storage
#[derive(Debug, Default)]
pub struct TokenOverrides {
    values: HashMap<String, TokenValue>,
}

impl TokenOverrides {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, path: String, value: TokenValue) {
        self.values.insert(path, value);
    }

    pub fn get(&self, path: &str) -> Option<&TokenValue> {
        self.values.get(path)
    }

    pub fn remove(&mut self, path: &str) -> Option<TokenValue> {
        self.values.remove(path)
    }

    pub fn has(&self, path: &str) -> bool {
        self.values.contains_key(path)
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }
}

/// A token change record
#[derive(Debug, Clone)]
pub struct TokenChange {
    /// Token path
    pub path: String,
    /// Old value
    pub old_value: Option<TokenValue>,
    /// New value
    pub new_value: Option<TokenValue>,
}

/// Resolved token information
#[derive(Debug, Clone)]
pub struct ResolvedToken {
    /// Token path
    pub path: String,
    /// Resolved value
    pub value: TokenValue,
    /// Where the value came from
    pub source: TokenSource,
    /// Token category
    pub category: TokenCategory,
}

/// Source of a token value
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenSource {
    /// From theme defaults
    Theme,
    /// From an override
    Override,
    /// Computed/derived
    Computed,
}

/// Token category
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TokenCategory {
    Color,
    Spacing,
    Radius,
    Typography,
    Shadow,
    Motion,
}

/// Token information for display
#[derive(Debug, Clone)]
pub struct TokenInfo {
    /// Full path
    pub path: String,
    /// Display name
    pub name: String,
    /// Category
    pub category: TokenCategory,
    /// Current value
    pub value: TokenValue,
    /// Has an active override
    pub has_override: bool,
}

// Helper functions

fn token_category(path: &str) -> TokenCategory {
    if path.starts_with("color.") {
        TokenCategory::Color
    } else if path.starts_with("spacing.") {
        TokenCategory::Spacing
    } else if path.starts_with("radius.") {
        TokenCategory::Radius
    } else if path.starts_with("typography.") || path.starts_with("font.") {
        TokenCategory::Typography
    } else if path.starts_with("shadow.") {
        TokenCategory::Shadow
    } else if path.starts_with("motion.") || path.starts_with("duration.") {
        TokenCategory::Motion
    } else {
        TokenCategory::Color // Default
    }
}

fn resolve_token_from_theme(tokens: &DesignTokens, path: &str) -> Option<TokenValue> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    match parts[0] {
        "color" if parts.len() >= 2 => {
            let color = get_color_token(&tokens.color, parts[1])?;
            Some(TokenValue::Color(color.value.clone()))
        }
        "spacing" if parts.len() >= 2 => {
            let spacing = get_spacing_value(&tokens.spacing, parts[1])?;
            Some(TokenValue::Number(spacing as f64))
        }
        "radius" if parts.len() >= 2 => {
            let radius = get_radius_value(&tokens.radius, parts[1])?;
            Some(TokenValue::Number(radius as f64))
        }
        "typography" if parts.len() >= 3 => {
            if parts[1] == "font_family" {
                let family = match parts[2] {
                    "sans" => &tokens.typography.font_family.sans,
                    "serif" => &tokens.typography.font_family.serif,
                    "mono" => &tokens.typography.font_family.mono,
                    _ => return None,
                };
                Some(TokenValue::String(family.clone()))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_color_token<'a>(colors: &'a ColorTokens, name: &str) -> Option<&'a ColorToken> {
    match name {
        "primary" => Some(&colors.primary),
        "secondary" => Some(&colors.secondary),
        "success" => Some(&colors.success),
        "warning" => Some(&colors.warning),
        "danger" => Some(&colors.danger),
        "info" => Some(&colors.info),
        "background" => Some(&colors.background),
        "surface" => Some(&colors.surface),
        "surface_variant" => Some(&colors.surface_variant),
        "text" => Some(&colors.text),
        "text_secondary" => Some(&colors.text_secondary),
        "text_disabled" => Some(&colors.text_disabled),
        "text_inverse" => Some(&colors.text_inverse),
        "border" => Some(&colors.border),
        "border_strong" => Some(&colors.border_strong),
        "divider" => Some(&colors.divider),
        "hover" => Some(&colors.hover),
        "focus" => Some(&colors.focus),
        "active" => Some(&colors.active),
        "disabled" => Some(&colors.disabled),
        other => colors.custom.get(other),
    }
}

fn get_spacing_value(spacing: &SpacingTokens, name: &str) -> Option<f32> {
    match name {
        "xs" => Some(spacing.xs.value),
        "sm" => Some(spacing.sm.value),
        "md" => Some(spacing.md.value),
        "lg" => Some(spacing.lg.value),
        "xl" => Some(spacing.xl.value),
        "xxl" => Some(spacing.xxl.value),
        "base" => Some(spacing.base),
        _ => spacing.custom.get(name).map(|s| s.value),
    }
}

fn get_radius_value(radius: &RadiusTokens, name: &str) -> Option<f32> {
    match name {
        "none" => Some(radius.none),
        "sm" => Some(radius.sm),
        "md" => Some(radius.md),
        "lg" => Some(radius.lg),
        "xl" => Some(radius.xl),
        "full" => Some(radius.full),
        "button" => Some(radius.button),
        "input" => Some(radius.input),
        "card" => Some(radius.card),
        "dialog" => Some(radius.dialog),
        _ => radius.custom.get(name).copied(),
    }
}

fn add_color_tokens(tokens: &mut Vec<TokenInfo>, colors: &ColorTokens, prefix: &str) {
    let color_names = [
        ("primary", &colors.primary),
        ("secondary", &colors.secondary),
        ("success", &colors.success),
        ("warning", &colors.warning),
        ("danger", &colors.danger),
        ("info", &colors.info),
        ("background", &colors.background),
        ("surface", &colors.surface),
        ("text", &colors.text),
        ("border", &colors.border),
    ];

    for (name, color) in color_names {
        tokens.push(TokenInfo {
            path: format!("{}.{}", prefix, name),
            name: name.to_string(),
            category: TokenCategory::Color,
            value: TokenValue::Color(color.value.clone()),
            has_override: false,
        });
    }
}

fn add_spacing_tokens(tokens: &mut Vec<TokenInfo>, spacing: &SpacingTokens, prefix: &str) {
    let sizes = [
        ("xs", spacing.xs.value),
        ("sm", spacing.sm.value),
        ("md", spacing.md.value),
        ("lg", spacing.lg.value),
        ("xl", spacing.xl.value),
        ("xxl", spacing.xxl.value),
    ];

    for (name, value) in sizes {
        tokens.push(TokenInfo {
            path: format!("{}.{}", prefix, name),
            name: name.to_string(),
            category: TokenCategory::Spacing,
            value: TokenValue::Number(value as f64),
            has_override: false,
        });
    }
}

fn add_radius_tokens(tokens: &mut Vec<TokenInfo>, radius: &RadiusTokens, prefix: &str) {
    let sizes = [
        ("none", radius.none),
        ("sm", radius.sm),
        ("md", radius.md),
        ("lg", radius.lg),
        ("xl", radius.xl),
        ("full", radius.full),
    ];

    for (name, value) in sizes {
        tokens.push(TokenInfo {
            path: format!("{}.{}", prefix, name),
            name: name.to_string(),
            category: TokenCategory::Radius,
            value: TokenValue::Number(value as f64),
            has_override: false,
        });
    }
}

fn format_token_value(value: &TokenValue) -> String {
    match value {
        TokenValue::Color(c) => format!("\"{}\"", c),
        TokenValue::Number(n) => format!("{}", n),
        TokenValue::String(s) => format!("\"{}\"", s),
        TokenValue::Spacing { value, x, y } => {
            if let (Some(x), Some(y)) = (x, y) {
                format!("{{ x = {}, y = {} }}", x, y)
            } else {
                format!("{}", value)
            }
        }
        TokenValue::Shadow {
            x,
            y,
            blur,
            spread,
            color,
        } => {
            format!(
                "{{ x = {}, y = {}, blur = {}, spread = {}, color = \"{}\" }}",
                x, y, blur, spread, color
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_editor() {
        let theme = Theme::dark();
        let mut editor = TokenEditor::with_theme(theme);

        // Check resolution
        let resolved = editor.resolve("color.primary").unwrap();
        assert!(matches!(resolved.source, TokenSource::Theme));

        // Add override
        editor.set_override("color.primary", TokenValue::color("#FF0000"));
        let resolved = editor.resolve("color.primary").unwrap();
        assert!(matches!(resolved.source, TokenSource::Override));
        assert!(matches!(&resolved.value, TokenValue::Color(c) if c == "#FF0000"));

        // Undo
        assert!(editor.can_undo());
        editor.undo();
        let resolved = editor.resolve("color.primary").unwrap();
        assert!(matches!(resolved.source, TokenSource::Theme));

        // Redo
        assert!(editor.can_redo());
        editor.redo();
        let resolved = editor.resolve("color.primary").unwrap();
        assert!(matches!(resolved.source, TokenSource::Override));
    }

    #[test]
    fn test_token_categories() {
        assert_eq!(token_category("color.primary"), TokenCategory::Color);
        assert_eq!(token_category("spacing.md"), TokenCategory::Spacing);
        assert_eq!(token_category("radius.lg"), TokenCategory::Radius);
        assert_eq!(token_category("shadow.sm"), TokenCategory::Shadow);
    }

    #[test]
    fn test_override_code_generation() {
        let mut editor = TokenEditor::new();
        editor.set_override("color.primary", TokenValue::color("#FF0000"));
        editor.set_override("spacing.md", TokenValue::number(20.0));

        let code = editor.generate_override_code();
        assert!(code.contains("color.primary"));
        assert!(code.contains("#FF0000"));
    }

    #[test]
    fn test_tokens_in_category() {
        let theme = Theme::dark();
        let editor = TokenEditor::with_theme(theme);

        let colors = editor.tokens_in_category(TokenCategory::Color);
        assert!(!colors.is_empty());
        assert!(colors.iter().any(|t| t.name == "primary"));
    }
}
