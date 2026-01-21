//! Static HTML Generator
//!
//! Converts ComponentIR to static HTML + CSS for web deployment.

use std::cell::RefCell;
use std::collections::HashMap;

use crate::{ComponentIR, Property, PropertyValue};

/// Theme tokens loaded from project's theme.toml
pub type ThemeTokens = HashMap<String, String>;

// Thread-local storage for theme tokens during HTML generation
thread_local! {
    static CURRENT_THEME: RefCell<Option<ThemeTokens>> = RefCell::new(None);
}

/// Set the current theme tokens for the generation context
fn set_theme_context(tokens: Option<&ThemeTokens>) {
    CURRENT_THEME.with(|t| {
        *t.borrow_mut() = tokens.cloned();
    });
}

/// Get a theme token value from the current context
fn get_theme_token(path: &str) -> Option<String> {
    CURRENT_THEME.with(|t| {
        t.borrow().as_ref().and_then(|tokens| {
            // Try the path as-is
            if let Some(value) = tokens.get(path) {
                return Some(value.clone());
            }
            // Try with hyphenated variant: color.text.secondary -> colors.text-secondary
            let hyphenated = path_to_theme_key(path);
            tokens.get(&hyphenated).cloned()
        })
    })
}

/// Generate static HTML from ComponentIR
///
/// # Arguments
/// * `ir` - The compiled component IR
/// * `title` - Page title
/// * `theme_tokens` - Optional theme tokens from theme.toml
pub fn generate_html(ir: &ComponentIR, title: &str, theme_tokens: Option<&ThemeTokens>) -> String {
    // Set the theme context for this generation
    set_theme_context(theme_tokens);

    let body_html = ir_to_html(ir);
    let css = generate_css();

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title}</title>
    <style>
{css}
    </style>
</head>
<body>
{body_html}
</body>
</html>"#
    )
}

/// Convert ComponentIR to HTML string
fn ir_to_html(ir: &ComponentIR) -> String {
    let mut html = String::new();
    ir_to_html_recursive(ir, &mut html, 0);
    html
}

/// Normalize property name to handle both camelCase and snake_case
/// Converts camelCase to snake_case for consistent internal handling
fn normalize_prop_name(name: &str) -> String {
    let mut result = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            result.push('_');
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c.to_lowercase().next().unwrap());
        }
    }
    result
}

/// Resolve design tokens in string values
/// Handles both `token("color.primary")` syntax and `{colors.primary}` syntax
fn resolve_tokens(value: &str) -> String {
    let mut result = value.to_string();

    // First, handle token() syntax: token("color.primary")
    while let Some(start) = result.find("token(") {
        if let Some(end) = result[start..].find(')') {
            let token_call = &result[start..start + end + 1];
            // Extract the token path from token("path") or token('path')
            let inner = &token_call[6..token_call.len() - 1].trim();
            let token_path = inner.trim_matches(|c| c == '"' || c == '\'');

            if let Some(resolved) = lookup_token(token_path) {
                // Replace the token() call with the resolved value
                result = format!("{}{}{}", &result[..start], resolved, &result[start + end + 1..]);
            } else {
                break; // Unknown token, stop processing
            }
        } else {
            break;
        }
    }

    // Then, handle {path} syntax: {colors.primary} or {spacing.4}
    while let Some(start) = result.find('{') {
        if let Some(end) = result[start..].find('}') {
            let token_path = &result[start + 1..start + end];
            // Convert {colors.primary} to color.primary format
            let normalized_path = normalize_token_path(token_path);

            if let Some(resolved) = lookup_token(&normalized_path) {
                result = format!("{}{}{}", &result[..start], resolved, &result[start + end + 1..]);
            } else {
                break; // Unknown token, stop processing
            }
        } else {
            break;
        }
    }

    result
}

/// Normalize token path from theme.toml format to internal format
/// {colors.primary} -> color.primary
/// {colors.text_primary} -> color.text.primary
/// {colors.text-secondary} -> color.text.secondary
/// {spacing.md} -> spacing.md
fn normalize_token_path(path: &str) -> String {
    let path = path.trim();

    // Normalize hyphens to dots for nested paths: text-secondary -> text.secondary
    let path = path.replace('-', ".");

    // Map common theme.toml paths to internal token paths
    if path.starts_with("colors.") {
        // colors.primary -> color.primary
        // colors.text_primary -> color.text.primary
        // colors.text.secondary -> color.text.secondary
        let suffix = &path[7..];
        // Convert snake_case to dot notation for nested paths
        let normalized_suffix = suffix.replace('_', ".");
        return format!("color.{}", normalized_suffix);
    }
    if path.starts_with("color.") {
        // Also convert snake_case in color.* paths
        let suffix = &path[6..];
        let normalized_suffix = suffix.replace('_', ".");
        return format!("color.{}", normalized_suffix);
    }
    if path.starts_with("spacing.") {
        return path.to_string();
    }
    if path.starts_with("radius.") {
        return path.to_string();
    }
    if path.starts_with("shadow.") {
        return path.to_string();
    }
    if path.starts_with("font.") {
        let suffix = &path[5..];
        let normalized_suffix = suffix.replace('_', ".");
        return format!("font.{}", normalized_suffix);
    }
    if path.starts_with("line.") {
        let suffix = &path[5..];
        let normalized_suffix = suffix.replace('_', ".");
        return format!("line.{}", normalized_suffix);
    }

    // Default: return as-is
    path.to_string()
}

/// Convert internal token path to theme.toml key format
/// color.text.secondary -> colors.text-secondary
/// color.primary -> colors.primary
fn path_to_theme_key(path: &str) -> String {
    if path.starts_with("color.") {
        let suffix = &path[6..];
        // Convert nested dots back to hyphens: text.secondary -> text-secondary
        let key = suffix.replace('.', "-");
        format!("colors.{}", key)
    } else {
        path.to_string()
    }
}

/// Look up a design token by its path
/// First checks theme_tokens from theme.toml (via thread-local context), then falls back to defaults
fn lookup_token(path: &str) -> Option<String> {
    // First, check theme tokens from the current context
    if let Some(value) = get_theme_token(path) {
        return Some(value);
    }

    // Fall back to default dark theme token values
    let default: Option<&'static str> = match path {
        // Colors - primary semantic
        "color.primary" => Some("#3B82F6"),
        "color.primary.light" => Some("#60A5FA"),
        "color.primary.dark" => Some("#2563EB"),
        "color.secondary" => Some("#6B7280"),
        "color.success" => Some("#22C55E"),
        "color.warning" => Some("#F59E0B"),
        "color.danger" => Some("#EF4444"),
        "color.info" => Some("#06B6D4"),

        // Colors - surfaces
        "color.background" => Some("#0B0F14"),
        "color.surface" => Some("#1F2937"),
        "color.surface.variant" | "color.surface_variant" => Some("#374151"),

        // Colors - text
        "color.text" => Some("#E5E7EB"),
        "color.text.primary" => Some("#E5E7EB"),
        "color.text.secondary" => Some("#9CA3AF"),
        "color.text.disabled" => Some("#6B7280"),
        "color.text.inverse" => Some("#111827"),

        // Colors - borders
        "color.border" => Some("#374151"),
        "color.border.strong" => Some("#4B5563"),
        "color.divider" => Some("#374151"),

        // Colors - states
        "color.hover" => Some("rgba(255,255,255,0.05)"),
        "color.focus" => Some("rgba(59,130,246,0.5)"),
        "color.active" => Some("rgba(59,130,246,0.2)"),
        "color.disabled" => Some("rgba(255,255,255,0.1)"),

        // Spacing
        "spacing.0" => Some("0"),
        "spacing.1" => Some("4px"),
        "spacing.2" => Some("8px"),
        "spacing.3" => Some("12px"),
        "spacing.4" => Some("16px"),
        "spacing.5" => Some("20px"),
        "spacing.6" => Some("24px"),
        "spacing.8" => Some("32px"),
        "spacing.10" => Some("40px"),
        "spacing.12" => Some("48px"),
        "spacing.16" => Some("64px"),
        "spacing.20" => Some("80px"),
        "spacing.24" => Some("96px"),

        // Radius
        "radius.none" => Some("0"),
        "radius.sm" => Some("4px"),
        "radius.md" => Some("8px"),
        "radius.lg" => Some("12px"),
        "radius.xl" => Some("16px"),
        "radius.full" => Some("9999px"),

        // Shadows
        "shadow.none" => Some("none"),
        "shadow.sm" => Some("0 1px 2px rgba(0,0,0,0.05)"),
        "shadow.md" => Some("0 4px 6px rgba(0,0,0,0.1)"),
        "shadow.lg" => Some("0 10px 15px rgba(0,0,0,0.1)"),
        "shadow.xl" => Some("0 20px 25px rgba(0,0,0,0.15)"),

        // Font sizes
        "font.size.xs" => Some("12px"),
        "font.size.sm" => Some("14px"),
        "font.size.md" | "font.size.base" => Some("16px"),
        "font.size.lg" => Some("18px"),
        "font.size.xl" => Some("20px"),
        "font.size.2xl" => Some("24px"),
        "font.size.3xl" => Some("30px"),
        "font.size.4xl" => Some("36px"),

        // Font weights
        "font.weight.normal" => Some("400"),
        "font.weight.medium" => Some("500"),
        "font.weight.semibold" => Some("600"),
        "font.weight.bold" => Some("700"),

        // Line heights
        "line.height.tight" => Some("1.25"),
        "line.height.normal" => Some("1.5"),
        "line.height.relaxed" => Some("1.75"),

        _ => None,
    };

    default.map(|s| s.to_string())
}

fn ir_to_html_recursive(ir: &ComponentIR, html: &mut String, indent: usize) {
    let indent_str = "    ".repeat(indent);

    match ir.kind.as_str() {
        "Text" => {
            let content = get_prop_string(&ir.props, "content").unwrap_or_default();
            let style = build_text_style(&ir.props, &ir.style);
            html.push_str(&format!(
                "{}<span id=\"{}\" style=\"{}\">{}</span>\n",
                indent_str, ir.id, style, content
            ));
        }
        "Button" => {
            // Button renders as a button element with text content
            let text = get_prop_string(&ir.props, "text")
                .or_else(|| get_prop_string(&ir.props, "content"))
                .unwrap_or_default();
            let style = build_button_style(&ir.props, &ir.style);
            html.push_str(&format!(
                "{}<button id=\"{}\" style=\"{}\">{}</button>\n",
                indent_str, ir.id, style, text
            ));
        }
        "Badge" => {
            // Badge renders as a span with badge styling
            let text = get_prop_string(&ir.props, "text")
                .or_else(|| get_prop_string(&ir.props, "content"))
                .unwrap_or_default();
            let style = build_badge_style(&ir.props, &ir.style);
            html.push_str(&format!(
                "{}<span id=\"{}\" class=\"badge\" style=\"{}\">{}</span>\n",
                indent_str, ir.id, style, text
            ));
        }
        "Image" => {
            let src = get_prop_string(&ir.props, "src").unwrap_or_default();
            let alt = get_prop_string(&ir.props, "alt").unwrap_or_default();
            let style = build_image_style(&ir.props, &ir.style);
            html.push_str(&format!(
                "{}<img id=\"{}\" src=\"{}\" alt=\"{}\" style=\"{}\"/>\n",
                indent_str, ir.id, src, alt, style
            ));
        }
        "Link" => {
            let href = get_prop_string(&ir.props, "href").unwrap_or_else(|| "#".to_string());
            let style = build_container_style("Link", &ir.props, &ir.style);
            html.push_str(&format!(
                "{}<a id=\"{}\" href=\"{}\" style=\"{}\">\n",
                indent_str, ir.id, href, style
            ));
            for child in &ir.children {
                ir_to_html_recursive(child, html, indent + 1);
            }
            html.push_str(&format!("{}</a>\n", indent_str));
        }
        "Column" | "Row" | "Container" | "Box" => {
            let kind = if ir.kind == "Box" { "Container" } else { &ir.kind };
            let style = build_container_style(kind, &ir.props, &ir.style);
            html.push_str(&format!(
                "{}<div id=\"{}\" style=\"{}\">\n",
                indent_str, ir.id, style
            ));
            for child in &ir.children {
                ir_to_html_recursive(child, html, indent + 1);
            }
            html.push_str(&format!("{}</div>\n", indent_str));
        }
        _ => {
            // Unknown component - render as div with container styling
            let style = build_container_style("Container", &ir.props, &ir.style);
            html.push_str(&format!(
                "{}<div id=\"{}\" class=\"{}\" style=\"{}\">\n",
                indent_str, ir.id, ir.kind.to_lowercase(), style
            ));
            for child in &ir.children {
                ir_to_html_recursive(child, html, indent + 1);
            }
            html.push_str(&format!("{}</div>\n", indent_str));
        }
    }
}

fn build_text_style(props: &[Property], style: &[Property]) -> String {
    let mut css_parts = Vec::new();

    for prop in props {
        let normalized_name = normalize_prop_name(&prop.name);
        match normalized_name.as_str() {
            "size" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-size: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        // Handle string values like "60" or "60px" or "2em"
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("font-size: {}px", n));
                        } else {
                            css_parts.push(format!("font-size: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "weight" | "font_weight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-weight: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("font-weight: {}", s));
                    }
                    _ => {}
                }
            }
            "line_height" | "lineheight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("line-height: {}", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("line-height: {}", n));
                        } else {
                            css_parts.push(format!("line-height: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "color" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("color: {}", resolve_tokens(s)));
                }
            }
            "align" | "text_align" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("text-align: {}", s));
                }
            }
            "font" | "font_family" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("font-family: {}", s));
                }
            }
            "letter_spacing" | "letterspacing" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("letter-spacing: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("letter-spacing: {}px", n));
                        } else {
                            css_parts.push(format!("letter-spacing: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "text_transform" | "transform" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("text-transform: {}", s));
                }
            }
            "opacity" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("opacity: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("opacity: {}", s));
                    }
                    _ => {}
                }
            }
            "max_width" | "maxwidth" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("max-width: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("max-width: {}px", n));
                        } else if s == "fill" {
                            css_parts.push("max-width: 100%".to_string());
                        } else {
                            css_parts.push(format!("max-width: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Apply style block properties
    for prop in style {
        if let Some(css) = prop_to_css(prop) {
            css_parts.push(css);
        }
    }

    css_parts.join("; ")
}

fn build_container_style(kind: &str, props: &[Property], style: &[Property]) -> String {
    let mut css_parts = Vec::new();

    // Base layout for Column/Row
    match kind {
        "Column" => {
            css_parts.push("display: flex".to_string());
            css_parts.push("flex-direction: column".to_string());
        }
        "Row" => {
            css_parts.push("display: flex".to_string());
            css_parts.push("flex-direction: row".to_string());
        }
        _ => {}
    }

    // Process props
    for prop in props {
        let normalized_name = normalize_prop_name(&prop.name);
        match normalized_name.as_str() {
            "width" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("width: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("width: {}px", n));
                        } else {
                            css_parts.push(format!("width: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("width: {}px", n));
                    }
                    _ => {}
                }
            }
            "height" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("height: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("height: {}px", n));
                        } else {
                            css_parts.push(format!("height: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("height: {}px", n));
                    }
                    _ => {}
                }
            }
            "min_width" | "minwidth" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("min-width: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("min-width: {}px", n));
                        } else {
                            css_parts.push(format!("min-width: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "min_height" | "minheight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("min-height: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("min-height: {}px", n));
                        } else {
                            css_parts.push(format!("min-height: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "max_width" | "maxwidth" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("max-width: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("max-width: {}px", n));
                        } else if s == "fill" {
                            css_parts.push("max-width: 100%".to_string());
                        } else {
                            css_parts.push(format!("max-width: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "max_height" | "maxheight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("max-height: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("max-height: {}px", n));
                        } else if s == "fill" {
                            css_parts.push("max-height: 100%".to_string());
                        } else {
                            css_parts.push(format!("max-height: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "gap" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("gap: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        // Handle multi-value gap like "16 8" -> "16px 8px"
                        css_parts.push(format!("gap: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "padding" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("padding: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        // Handle multi-value padding like "120 64" -> "120px 64px"
                        css_parts.push(format!("padding: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "padding_top" | "paddingtop" => {
                css_parts.push(format!("padding-top: {}", parse_single_value(&prop.value)));
            }
            "padding_bottom" | "paddingbottom" => {
                css_parts.push(format!("padding-bottom: {}", parse_single_value(&prop.value)));
            }
            "padding_left" | "paddingleft" => {
                css_parts.push(format!("padding-left: {}", parse_single_value(&prop.value)));
            }
            "padding_right" | "paddingright" => {
                css_parts.push(format!("padding-right: {}", parse_single_value(&prop.value)));
            }
            "margin" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("margin: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("margin: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "margin_top" | "margintop" => {
                css_parts.push(format!("margin-top: {}", parse_single_value(&prop.value)));
            }
            "margin_bottom" | "marginbottom" => {
                css_parts.push(format!("margin-bottom: {}", parse_single_value(&prop.value)));
            }
            "margin_left" | "marginleft" => {
                css_parts.push(format!("margin-left: {}", parse_single_value(&prop.value)));
            }
            "margin_right" | "marginright" => {
                css_parts.push(format!("margin-right: {}", parse_single_value(&prop.value)));
            }
            "align" | "align_items" | "alignitems" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("align-items: {}", s));
                }
            }
            "justify" | "justify_content" | "justifycontent" => {
                if let PropertyValue::String(s) = &prop.value {
                    let val = match s.as_str() {
                        "space_between" | "spaceBetween" => "space-between",
                        "space_around" | "spaceAround" => "space-around",
                        "space_evenly" | "spaceEvenly" => "space-evenly",
                        "flex_start" | "flexStart" => "flex-start",
                        "flex_end" | "flexEnd" => "flex-end",
                        _ => s.as_str(),
                    };
                    css_parts.push(format!("justify-content: {}", val));
                }
            }
            "wrap" | "flex_wrap" | "flexwrap" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("flex-wrap: {}", s));
                } else if let PropertyValue::Bool(b) = &prop.value {
                    if *b {
                        css_parts.push("flex-wrap: wrap".to_string());
                    }
                }
            }
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    // Resolve any design tokens first
                    let resolved = resolve_tokens(s);
                    // Check if it's a gradient or other CSS function
                    if resolved.contains("gradient(") || resolved.contains("url(") || resolved.contains("linear-gradient") || resolved.contains("radial-gradient") {
                        css_parts.push(format!("background: {}", resolved));
                    } else {
                        css_parts.push(format!("background-color: {}", resolved));
                    }
                }
            }
            "radius" | "border_radius" | "borderradius" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border-radius: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border-radius: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "border" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border: {}px solid", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("border: {}px solid", n));
                        } else {
                            css_parts.push(format!("border: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "border_width" | "borderwidth" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border-width: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border-width: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "border_color" | "bordercolor" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("border-color: {}", resolve_tokens(s)));
                }
            }
            "border_style" | "borderstyle" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("border-style: {}", s));
                }
            }
            "flex" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("flex: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("flex: {}", s));
                    }
                    _ => {}
                }
            }
            "flex_grow" | "flexgrow" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("flex-grow: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("flex-grow: {}", s));
                    }
                    _ => {}
                }
            }
            "flex_shrink" | "flexshrink" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("flex-shrink: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("flex-shrink: {}", s));
                    }
                    _ => {}
                }
            }
            "overflow" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("overflow: {}", s));
                }
            }
            "overflow_x" | "overflowx" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("overflow-x: {}", s));
                }
            }
            "overflow_y" | "overflowy" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("overflow-y: {}", s));
                }
            }
            "position" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("position: {}", s));
                }
            }
            "top" => css_parts.push(format!("top: {}", parse_single_value(&prop.value))),
            "bottom" => css_parts.push(format!("bottom: {}", parse_single_value(&prop.value))),
            "left" => css_parts.push(format!("left: {}", parse_single_value(&prop.value))),
            "right" => css_parts.push(format!("right: {}", parse_single_value(&prop.value))),
            "z_index" | "zindex" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("z-index: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("z-index: {}", s));
                    }
                    _ => {}
                }
            }
            "opacity" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("opacity: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("opacity: {}", s));
                    }
                    _ => {}
                }
            }
            "cursor" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("cursor: {}", s));
                }
            }
            "box_shadow" | "boxshadow" | "shadow" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("box-shadow: {}", s));
                }
            }
            _ => {}
        }
    }

    // Apply style block properties
    for prop in style {
        if let Some(css) = prop_to_css(prop) {
            css_parts.push(css);
        }
    }

    css_parts.join("; ")
}

fn build_button_style(props: &[Property], style: &[Property]) -> String {
    let mut css_parts = Vec::new();

    // Default button styling
    css_parts.push("display: inline-flex".to_string());
    css_parts.push("align-items: center".to_string());
    css_parts.push("justify-content: center".to_string());
    css_parts.push("cursor: pointer".to_string());
    css_parts.push("border: none".to_string());

    for prop in props {
        let normalized_name = normalize_prop_name(&prop.name);
        match normalized_name.as_str() {
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s.contains("gradient(") {
                        css_parts.push(format!("background: {}", s));
                    } else {
                        css_parts.push(format!("background-color: {}", s));
                    }
                }
            }
            "color" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("color: {}", resolve_tokens(s)));
                }
            }
            "size" | "font_size" | "fontsize" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-size: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("font-size: {}px", n));
                        } else {
                            css_parts.push(format!("font-size: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "weight" | "font_weight" | "fontweight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-weight: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("font-weight: {}", s));
                    }
                    _ => {}
                }
            }
            "padding" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("padding: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("padding: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "radius" | "border_radius" | "borderradius" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border-radius: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border-radius: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "width" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("width: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("width: {}px", n));
                        } else {
                            css_parts.push(format!("width: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("width: {}px", n));
                    }
                    _ => {}
                }
            }
            "height" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("height: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("height: {}px", n));
                        } else {
                            css_parts.push(format!("height: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("height: {}px", n));
                    }
                    _ => {}
                }
            }
            "border" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border: {}px solid", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border: {}", s));
                    }
                    _ => {}
                }
            }
            "border_color" | "bordercolor" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("border-color: {}", resolve_tokens(s)));
                }
            }
            _ => {}
        }
    }

    // Apply style block properties
    for prop in style {
        if let Some(css) = prop_to_css(prop) {
            css_parts.push(css);
        }
    }

    css_parts.join("; ")
}

fn build_badge_style(props: &[Property], style: &[Property]) -> String {
    let mut css_parts = Vec::new();

    // Default badge styling
    css_parts.push("display: inline-flex".to_string());
    css_parts.push("align-items: center".to_string());
    css_parts.push("justify-content: center".to_string());
    css_parts.push("padding: 4px 8px".to_string());
    css_parts.push("font-size: 12px".to_string());
    css_parts.push("border-radius: 9999px".to_string());

    for prop in props {
        let normalized_name = normalize_prop_name(&prop.name);
        match normalized_name.as_str() {
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s.contains("gradient(") {
                        css_parts.push(format!("background: {}", s));
                    } else {
                        css_parts.push(format!("background-color: {}", s));
                    }
                }
            }
            "color" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("color: {}", resolve_tokens(s)));
                }
            }
            "size" | "font_size" | "fontsize" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-size: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("font-size: {}px", n));
                        } else {
                            css_parts.push(format!("font-size: {}", s));
                        }
                    }
                    _ => {}
                }
            }
            "weight" | "font_weight" | "fontweight" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("font-weight: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("font-weight: {}", s));
                    }
                    _ => {}
                }
            }
            "padding" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("padding: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("padding: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "radius" | "border_radius" | "borderradius" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border-radius: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border-radius: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "border" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border: {}px solid", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border: {}", s));
                    }
                    _ => {}
                }
            }
            "border_color" | "bordercolor" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("border-color: {}", resolve_tokens(s)));
                }
            }
            _ => {}
        }
    }

    // Apply style block properties
    for prop in style {
        if let Some(css) = prop_to_css(prop) {
            css_parts.push(css);
        }
    }

    css_parts.join("; ")
}

fn build_image_style(props: &[Property], style: &[Property]) -> String {
    let mut css_parts = Vec::new();

    for prop in props {
        let normalized_name = normalize_prop_name(&prop.name);
        match normalized_name.as_str() {
            "width" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("width: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("width: {}px", n));
                        } else {
                            css_parts.push(format!("width: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("width: {}px", n));
                    }
                    _ => {}
                }
            }
            "height" => {
                match &prop.value {
                    PropertyValue::String(s) => {
                        if s == "fill" {
                            css_parts.push("height: 100%".to_string());
                        } else if let Ok(n) = s.parse::<f64>() {
                            css_parts.push(format!("height: {}px", n));
                        } else {
                            css_parts.push(format!("height: {}", s));
                        }
                    }
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("height: {}px", n));
                    }
                    _ => {}
                }
            }
            "radius" | "border_radius" | "borderradius" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("border-radius: {}px", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("border-radius: {}", parse_spacing_value(s)));
                    }
                    _ => {}
                }
            }
            "object_fit" | "objectfit" | "fit" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("object-fit: {}", s));
                }
            }
            "opacity" => {
                match &prop.value {
                    PropertyValue::Number(n) => {
                        css_parts.push(format!("opacity: {}", n));
                    }
                    PropertyValue::String(s) => {
                        css_parts.push(format!("opacity: {}", s));
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    // Apply style block properties
    for prop in style {
        if let Some(css) = prop_to_css(prop) {
            css_parts.push(css);
        }
    }

    css_parts.join("; ")
}

/// Parse spacing values that may be multi-value like "120 64" -> "120px 64px"
fn parse_spacing_value(s: &str) -> String {
    // If already contains px, em, rem, %, etc., pass through
    if s.contains("px") || s.contains("em") || s.contains("rem") || s.contains("%") || s.contains("vw") || s.contains("vh") || s.contains("auto") {
        return s.to_string();
    }

    // Split by whitespace and convert numbers to px
    let parts: Vec<String> = s
        .split_whitespace()
        .map(|part| {
            if let Ok(_n) = part.parse::<f64>() {
                format!("{}px", part)
            } else {
                part.to_string()
            }
        })
        .collect();

    parts.join(" ")
}

/// Parse a single value (number or string) and return CSS value
fn parse_single_value(value: &PropertyValue) -> String {
    match value {
        PropertyValue::Number(n) => format!("{}px", n),
        PropertyValue::String(s) => {
            if let Ok(n) = s.parse::<f64>() {
                format!("{}px", n)
            } else if s == "auto" {
                "auto".to_string()
            } else if s.contains("px") || s.contains("em") || s.contains("rem") || s.contains("%") || s.contains("vw") || s.contains("vh") {
                s.clone()
            } else {
                s.clone()
            }
        }
        _ => "0".to_string(),
    }
}

fn prop_to_css(prop: &Property) -> Option<String> {
    let normalized_name = normalize_prop_name(&prop.name);
    match normalized_name.as_str() {
        "background" => {
            if let PropertyValue::String(s) = &prop.value {
                if s.contains("gradient(") || s.contains("url(") {
                    Some(format!("background: {}", s))
                } else {
                    Some(format!("background-color: {}", s))
                }
            } else {
                None
            }
        }
        "padding" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("padding: {}px", n)),
                PropertyValue::String(s) => Some(format!("padding: {}", parse_spacing_value(s))),
                _ => None,
            }
        }
        "margin" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("margin: {}px", n)),
                PropertyValue::String(s) => Some(format!("margin: {}", parse_spacing_value(s))),
                _ => None,
            }
        }
        "radius" | "border_radius" | "borderradius" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("border-radius: {}px", n)),
                PropertyValue::String(s) => Some(format!("border-radius: {}", parse_spacing_value(s))),
                _ => None,
            }
        }
        "border" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("border: {}px solid", n)),
                PropertyValue::String(s) => {
                    if let Ok(n) = s.parse::<f64>() {
                        Some(format!("border: {}px solid", n))
                    } else {
                        Some(format!("border: {}", s))
                    }
                }
                _ => None,
            }
        }
        "border_color" | "bordercolor" => {
            if let PropertyValue::String(s) = &prop.value {
                Some(format!("border-color: {}", resolve_tokens(s)))
            } else {
                None
            }
        }
        "border_width" | "borderwidth" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("border-width: {}px", n)),
                PropertyValue::String(s) => Some(format!("border-width: {}", parse_spacing_value(s))),
                _ => None,
            }
        }
        "color" => {
            if let PropertyValue::String(s) = &prop.value {
                Some(format!("color: {}", resolve_tokens(s)))
            } else {
                None
            }
        }
        "opacity" => {
            match &prop.value {
                PropertyValue::Number(n) => Some(format!("opacity: {}", n)),
                PropertyValue::String(s) => Some(format!("opacity: {}", s)),
                _ => None,
            }
        }
        "box_shadow" | "boxshadow" | "shadow" => {
            if let PropertyValue::String(s) = &prop.value {
                Some(format!("box-shadow: {}", s))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_prop_string(props: &[Property], name: &str) -> Option<String> {
    let normalized_target = normalize_prop_name(name);
    props.iter().find(|p| normalize_prop_name(&p.name) == normalized_target).and_then(|p| {
        match &p.value {
            PropertyValue::String(s) => Some(s.clone()),
            PropertyValue::Number(n) => Some(n.to_string()),
            _ => None,
        }
    })
}

fn generate_css() -> String {
    r#"        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        html, body {
            height: 100%;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
        }
        body > div {
            min-height: 100%;
        }
        span {
            display: block;
        }
        a {
            color: inherit;
            text-decoration: none;
        }
        a:hover {
            opacity: 0.8;
        }
        button {
            font-family: inherit;
        }
        button:hover {
            opacity: 0.9;
        }
        .badge {
            display: inline-flex;
        }"#
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    #[test]
    fn test_generate_simple_html() {
        let source = r##"
            app Test {
                Column {
                    width: fill
                    height: fill
                    background: "#030712"

                    Text {
                        content: "Hello World"
                        size: 24
                        color: "#FFFFFF"
                    }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("Hello World"));
        assert!(html.contains("background-color: #030712"));
        assert!(html.contains("font-size: 24px"));
        assert!(html.contains("color: #FFFFFF"));
    }

    #[test]
    fn test_generate_nested_html() {
        let source = r#"
            app Test {
                Column {
                    Row {
                        gap: 16

                        Text { content: "A" }
                        Text { content: "B" }
                    }
                }
            }
        "#;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("flex-direction: column"));
        assert!(html.contains("flex-direction: row"));
        assert!(html.contains("gap: 16px"));
    }

    #[test]
    fn test_text_weight_and_line_height() {
        let source = r##"
            app Test {
                Column {
                    Text {
                        content: "Hello"
                        size: 60
                        weight: 700
                        color: "#FFFFFF"
                    }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("font-size: 60px"));
        assert!(html.contains("font-weight: 700"));
        assert!(html.contains("color: #FFFFFF"));
    }

    #[test]
    fn test_string_size_values() {
        let source = r##"
            app Test {
                Column {
                    Text {
                        content: "Hello"
                        size: "48"
                        weight: "bold"
                    }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("font-size: 48px"));
        assert!(html.contains("font-weight: bold"));
    }

    #[test]
    fn test_multi_value_padding() {
        let source = r##"
            app Test {
                Column {
                    padding: "120 64"
                    background: "#030712"

                    Text { content: "Test" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("padding: 120px 64px"));
    }

    #[test]
    fn test_gradient_background() {
        let source = r##"
            app Test {
                Column {
                    background: "linear-gradient(180deg, #1a1a2e 0%, #16213e 100%)"

                    Text { content: "Gradient" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("background: linear-gradient"));
    }

    #[test]
    fn test_margin_properties() {
        let source = r##"
            app Test {
                Column {
                    margin_top: 20
                    margin_bottom: 40

                    Text { content: "Test" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("margin-top: 20px"));
        assert!(html.contains("margin-bottom: 40px"));
    }

    #[test]
    fn test_border_color() {
        let source = r##"
            app Test {
                Container {
                    border: 1
                    border_color: "#FF0000"

                    Text { content: "Bordered" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("border: 1px solid"));
        assert!(html.contains("border-color: #FF0000"));
    }

    #[test]
    fn test_max_width() {
        let source = r##"
            app Test {
                Container {
                    max_width: 1200

                    Text { content: "Limited width" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("max-width: 1200px"));
    }

    #[test]
    fn test_flex_wrap() {
        let source = r##"
            app Test {
                Row {
                    wrap: "wrap"
                    gap: 16

                    Text { content: "A" }
                    Text { content: "B" }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        assert!(html.contains("flex-wrap: wrap"));
    }

    #[test]
    fn test_normalize_prop_name() {
        assert_eq!(normalize_prop_name("marginTop"), "margin_top");
        assert_eq!(normalize_prop_name("borderColor"), "border_color");
        assert_eq!(normalize_prop_name("maxWidth"), "max_width");
        assert_eq!(normalize_prop_name("lineHeight"), "line_height");
        assert_eq!(normalize_prop_name("flexWrap"), "flex_wrap");
        assert_eq!(normalize_prop_name("margin_top"), "margin_top");
    }

    #[test]
    fn test_parse_spacing_value() {
        assert_eq!(parse_spacing_value("120 64"), "120px 64px");
        assert_eq!(parse_spacing_value("10 20 30 40"), "10px 20px 30px 40px");
        assert_eq!(parse_spacing_value("10px 20px"), "10px 20px");
        assert_eq!(parse_spacing_value("auto"), "auto");
        assert_eq!(parse_spacing_value("50%"), "50%");
    }

    #[test]
    fn test_resolve_tokens() {
        // Basic token resolution
        assert_eq!(resolve_tokens(r#"token("color.primary")"#), "#3B82F6");
        assert_eq!(resolve_tokens(r#"token("color.surface")"#), "#1F2937");
        assert_eq!(resolve_tokens(r#"token("color.text.primary")"#), "#E5E7EB");

        // Token in the middle of a value
        assert_eq!(resolve_tokens(r#"1px solid token("color.border")"#), "1px solid #374151");

        // Unknown tokens pass through
        assert_eq!(resolve_tokens(r#"token("unknown.token")"#), r#"token("unknown.token")"#);

        // Non-token values pass through
        assert_eq!(resolve_tokens("#FF0000"), "#FF0000");
        assert_eq!(resolve_tokens("16px"), "16px");

        // Spacing tokens
        assert_eq!(resolve_tokens(r#"token("spacing.4")"#), "16px");
        assert_eq!(resolve_tokens(r#"token("radius.md")"#), "8px");

        // Curly brace syntax: {colors.primary} from theme.toml format
        assert_eq!(resolve_tokens("{colors.primary}"), "#3B82F6");
        assert_eq!(resolve_tokens("{color.surface}"), "#1F2937");
        assert_eq!(resolve_tokens("{spacing.4}"), "16px");
        assert_eq!(resolve_tokens("{radius.md}"), "8px");
    }

    #[test]
    fn test_curly_brace_token_in_html() {
        let source = r##"
            app Test {
                Column {
                    background: "{colors.primary}"

                    Text {
                        content: "Hello"
                        color: "{colors.text_primary}"
                    }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        // Curly brace tokens should be resolved to actual CSS values
        assert!(html.contains("background-color: #3B82F6"));
        assert!(html.contains("color: #E5E7EB"));
    }

    #[test]
    fn test_token_in_html_output() {
        let source = r##"
            app Test {
                Column {
                    background: "token(\"color.surface\")"
                    border_color: "token(\"color.border\")"

                    Text {
                        content: "Hello"
                        color: "token(\"color.text.primary\")"
                    }
                }
            }
        "##;

        let ir = compile(source).unwrap();
        let html = generate_html(&ir, "Test", None);

        // Tokens should be resolved to actual CSS values
        assert!(html.contains("background-color: #1F2937"));
        assert!(html.contains("border-color: #374151"));
        assert!(html.contains("color: #E5E7EB"));
    }
}
