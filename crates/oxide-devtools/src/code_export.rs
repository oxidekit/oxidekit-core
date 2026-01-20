//! Code Export
//!
//! Provides "copy as code" functionality for exporting tweaked styles
//! and component configurations in various formats.

use crate::editor::StyleValueChange;
use crate::inspector::ComponentInfo;
use crate::tree::{ComponentNode, ComponentTree};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Code export format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExportFormat {
    /// OxideUI markup (.oui)
    Oui,
    /// Rust code
    Rust,
    /// JSON
    Json,
    /// TOML (for theme overrides)
    Toml,
}

/// Options for code export
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    /// Format to export as
    pub format: ExportFormat,
    /// Include children
    pub include_children: bool,
    /// Include computed styles
    pub include_computed: bool,
    /// Only include explicit overrides
    pub overrides_only: bool,
    /// Indent string (for formatting)
    pub indent: String,
    /// Include comments
    pub include_comments: bool,
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Oui,
            include_children: true,
            include_computed: false,
            overrides_only: false,
            indent: "  ".to_string(),
            include_comments: true,
        }
    }
}

/// Code exporter
#[derive(Debug)]
pub struct CodeExporter {
    options: ExportOptions,
}

impl Default for CodeExporter {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeExporter {
    /// Create a new code exporter
    pub fn new() -> Self {
        Self {
            options: ExportOptions::default(),
        }
    }

    /// Create with options
    pub fn with_options(options: ExportOptions) -> Self {
        Self { options }
    }

    /// Export a component node
    pub fn export_node(&self, node: &ComponentNode, tree: &ComponentTree) -> String {
        match self.options.format {
            ExportFormat::Oui => self.export_oui(node, tree, 0),
            ExportFormat::Rust => self.export_rust(node, tree, 0),
            ExportFormat::Json => self.export_json(node, tree),
            ExportFormat::Toml => self.export_toml(node),
        }
    }

    /// Export component info (from inspector)
    pub fn export_info(&self, info: &ComponentInfo) -> String {
        match self.options.format {
            ExportFormat::Oui => self.info_to_oui(info, 0),
            ExportFormat::Rust => self.info_to_rust(info),
            ExportFormat::Json => serde_json::to_string_pretty(info).unwrap_or_default(),
            ExportFormat::Toml => self.info_to_toml(info),
        }
    }

    /// Export style overrides
    pub fn export_overrides(&self, overrides: &HashMap<String, StyleValueChange>) -> String {
        match self.options.format {
            ExportFormat::Oui => self.overrides_to_oui(overrides),
            ExportFormat::Rust => self.overrides_to_rust(overrides),
            ExportFormat::Json => self.overrides_to_json(overrides),
            ExportFormat::Toml => self.overrides_to_toml(overrides),
        }
    }

    // OUI format export

    fn export_oui(&self, node: &ComponentNode, tree: &ComponentTree, depth: usize) -> String {
        let indent = self.options.indent.repeat(depth);
        let mut lines = Vec::new();

        // Component opening tag
        let component_name = node.component_id.split('.').last().unwrap_or(&node.component_id);

        // Build attributes
        let mut attrs = Vec::new();

        // Add props
        for (name, value) in &node.props {
            if let Some(attr) = format_prop_oui(name, value) {
                attrs.push(attr);
            }
        }

        // Add styles
        if !node.styles.is_empty() {
            let style_str = self.format_styles_oui(&node.styles);
            if !style_str.is_empty() {
                attrs.push(format!("style={{ {} }}", style_str));
            }
        }

        // Format opening tag
        let attrs_str = if attrs.is_empty() {
            String::new()
        } else {
            format!(" {}", attrs.join(" "))
        };

        // Check if we have children
        let has_children = self.options.include_children && !node.children.is_empty();

        if has_children {
            lines.push(format!("{}<{}{}>", indent, component_name, attrs_str));

            // Export children
            for child_handle in &node.children {
                if let Some(child) = tree.get(*child_handle) {
                    lines.push(self.export_oui(child, tree, depth + 1));
                }
            }

            lines.push(format!("{}</{}>", indent, component_name));
        } else {
            lines.push(format!("{}<{}{} />", indent, component_name, attrs_str));
        }

        lines.join("\n")
    }

    fn format_styles_oui(&self, styles: &HashMap<String, serde_json::Value>) -> String {
        styles
            .iter()
            .filter_map(|(name, value)| {
                format_value_oui(value).map(|v| format!("{}: {}", name, v))
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn info_to_oui(&self, info: &ComponentInfo, depth: usize) -> String {
        let indent = self.options.indent.repeat(depth);
        let component_name = info
            .component_id
            .split('.')
            .last()
            .unwrap_or(&info.component_id);

        let mut lines = Vec::new();

        // Add comment with source location
        if self.options.include_comments {
            if let Some(source) = &info.source {
                lines.push(format!("{}// Source: {}", indent, source.to_string()));
            }
        }

        // Build props
        let props_str: Vec<String> = info
            .props
            .iter()
            .filter(|p| p.is_set || self.options.include_computed)
            .filter_map(|p| {
                format_value_oui(&p.value).map(|v| format!("{}={}", p.name, v))
            })
            .collect();

        // Build styles from computed
        let styles_str: Vec<String> = info
            .computed_styles
            .iter()
            .filter_map(|(name, sv)| Some(format!("{}: {}", name, sv.raw)))
            .collect();

        let attrs = if props_str.is_empty() && styles_str.is_empty() {
            String::new()
        } else {
            let mut all_attrs = props_str;
            if !styles_str.is_empty() {
                all_attrs.push(format!("style={{ {} }}", styles_str.join(", ")));
            }
            format!(" {}", all_attrs.join(" "))
        };

        lines.push(format!("{}<{}{} />", indent, component_name, attrs));

        lines.join("\n")
    }

    fn overrides_to_oui(&self, overrides: &HashMap<String, StyleValueChange>) -> String {
        let formatted: Vec<String> = overrides
            .iter()
            .map(|(name, value)| format!("{}: {}", name, format_style_value_oui(value)))
            .collect();

        format!("style={{ {} }}", formatted.join(", "))
    }

    // Rust format export

    fn export_rust(&self, node: &ComponentNode, tree: &ComponentTree, depth: usize) -> String {
        let indent = self.options.indent.repeat(depth);
        let mut lines = Vec::new();

        let component_name = node.component_id.split('.').last().unwrap_or(&node.component_id);

        // Builder pattern
        lines.push(format!("{}{}::new()", indent, component_name));

        // Add props
        for (name, value) in &node.props {
            if let Some(formatted) = format_value_rust(value) {
                lines.push(format!("{}    .{}({})", indent, to_snake_case(name), formatted));
            }
        }

        // Add styles
        for (name, value) in &node.styles {
            if let Some(formatted) = format_value_rust(value) {
                lines.push(format!("{}    .style_{}: {}", indent, to_snake_case(name), formatted));
            }
        }

        // Add children
        if self.options.include_children && !node.children.is_empty() {
            lines.push(format!("{}    .children(vec![", indent));
            for child_handle in &node.children {
                if let Some(child) = tree.get(*child_handle) {
                    lines.push(format!("{},", self.export_rust(child, tree, depth + 2)));
                }
            }
            lines.push(format!("{}    ])", indent));
        }

        lines.push(format!("{}    .build()", indent));

        lines.join("\n")
    }

    fn info_to_rust(&self, info: &ComponentInfo) -> String {
        let component_name = info
            .component_id
            .split('.')
            .last()
            .unwrap_or(&info.component_id);

        let mut lines = Vec::new();
        lines.push(format!("{}::new()", component_name));

        for prop in &info.props {
            if prop.is_set || !self.options.overrides_only {
                if let Some(formatted) = format_value_rust(&prop.value) {
                    lines.push(format!("    .{}({})", to_snake_case(&prop.name), formatted));
                }
            }
        }

        lines.push("    .build()".to_string());
        lines.join("\n")
    }

    fn overrides_to_rust(&self, overrides: &HashMap<String, StyleValueChange>) -> String {
        let mut lines = Vec::new();
        lines.push("StyleOverrides::new()".to_string());

        for (name, value) in overrides {
            let formatted = format_style_value_rust(value);
            lines.push(format!("    .set(\"{}\", {})", name, formatted));
        }

        lines.join("\n")
    }

    // JSON format export

    fn export_json(&self, node: &ComponentNode, tree: &ComponentTree) -> String {
        let json_value = self.node_to_json(node, tree);
        serde_json::to_string_pretty(&json_value).unwrap_or_default()
    }

    fn node_to_json(&self, node: &ComponentNode, tree: &ComponentTree) -> serde_json::Value {
        let mut obj = serde_json::Map::new();

        obj.insert(
            "component".to_string(),
            serde_json::json!(node.component_id),
        );

        if !node.name.is_empty() {
            obj.insert("name".to_string(), serde_json::json!(node.name));
        }

        if !node.props.is_empty() {
            obj.insert("props".to_string(), serde_json::json!(node.props));
        }

        if !node.styles.is_empty() {
            obj.insert("styles".to_string(), serde_json::json!(node.styles));
        }

        if self.options.include_children && !node.children.is_empty() {
            let children: Vec<serde_json::Value> = node
                .children
                .iter()
                .filter_map(|h| tree.get(*h))
                .map(|c| self.node_to_json(c, tree))
                .collect();
            obj.insert("children".to_string(), serde_json::json!(children));
        }

        serde_json::Value::Object(obj)
    }

    fn overrides_to_json(&self, overrides: &HashMap<String, StyleValueChange>) -> String {
        let json_map: HashMap<String, serde_json::Value> = overrides
            .iter()
            .map(|(k, v)| (k.clone(), v.to_json()))
            .collect();

        serde_json::to_string_pretty(&json_map).unwrap_or_default()
    }

    // TOML format export

    fn export_toml(&self, node: &ComponentNode) -> String {
        let mut lines = Vec::new();

        lines.push(format!("[component]"));
        lines.push(format!("type = \"{}\"", node.component_id));

        if !node.name.is_empty() {
            lines.push(format!("name = \"{}\"", node.name));
        }

        if !node.props.is_empty() {
            lines.push(String::new());
            lines.push("[component.props]".to_string());
            for (name, value) in &node.props {
                if let Some(formatted) = format_value_toml(value) {
                    lines.push(format!("{} = {}", name, formatted));
                }
            }
        }

        if !node.styles.is_empty() {
            lines.push(String::new());
            lines.push("[component.styles]".to_string());
            for (name, value) in &node.styles {
                if let Some(formatted) = format_value_toml(value) {
                    lines.push(format!("{} = {}", name, formatted));
                }
            }
        }

        lines.join("\n")
    }

    fn info_to_toml(&self, info: &ComponentInfo) -> String {
        let mut lines = Vec::new();

        lines.push("[component]".to_string());
        lines.push(format!("type = \"{}\"", info.component_id));
        lines.push(format!("id = \"{}\"", info.id));

        if !info.props.is_empty() {
            lines.push(String::new());
            lines.push("[component.props]".to_string());
            for prop in &info.props {
                if prop.is_set || !self.options.overrides_only {
                    if let Some(formatted) = format_value_toml(&prop.value) {
                        lines.push(format!("{} = {}", prop.name, formatted));
                    }
                }
            }
        }

        lines.join("\n")
    }

    fn overrides_to_toml(&self, overrides: &HashMap<String, StyleValueChange>) -> String {
        let mut lines = Vec::new();
        lines.push("[style_overrides]".to_string());

        for (name, value) in overrides {
            let formatted = format_style_value_toml(value);
            lines.push(format!("{} = {}", name, formatted));
        }

        lines.join("\n")
    }
}

// Helper functions

fn format_prop_oui(name: &str, value: &serde_json::Value) -> Option<String> {
    let formatted = format_value_oui(value)?;
    Some(format!("{}={}", name, formatted))
}

fn format_value_oui(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(format!("\"{}\"", s)),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null => None,
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().filter_map(format_value_oui).collect();
            Some(format!("[{}]", items.join(", ")))
        }
        serde_json::Value::Object(obj) => {
            let items: Vec<String> = obj
                .iter()
                .filter_map(|(k, v)| format_value_oui(v).map(|fv| format!("{}: {}", k, fv)))
                .collect();
            Some(format!("{{ {} }}", items.join(", ")))
        }
    }
}

fn format_style_value_oui(value: &StyleValueChange) -> String {
    match value {
        StyleValueChange::Color(c) => format!("\"{}\"", c),
        StyleValueChange::Number { value, unit } => {
            if let Some(u) = unit {
                format!("{}{}", value, u)
            } else {
                format!("{}", value)
            }
        }
        StyleValueChange::String(s) => format!("\"{}\"", s),
        StyleValueChange::Bool(b) => b.to_string(),
        StyleValueChange::Token(t) => format!("${}", t),
        StyleValueChange::Enum(e) => e.clone(),
        StyleValueChange::Unset => "unset".to_string(),
    }
}

fn format_value_rust(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(format!("\"{}\"", s)),
        serde_json::Value::Number(n) => {
            if n.is_f64() {
                Some(format!("{}_f64", n.as_f64().unwrap()))
            } else {
                Some(n.to_string())
            }
        }
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null => Some("None".to_string()),
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().filter_map(format_value_rust).collect();
            Some(format!("vec![{}]", items.join(", ")))
        }
        serde_json::Value::Object(_) => None, // Complex objects need special handling
    }
}

fn format_style_value_rust(value: &StyleValueChange) -> String {
    match value {
        StyleValueChange::Color(c) => format!("Color::from_hex(\"{}\")", c),
        StyleValueChange::Number { value, unit } => {
            if let Some(u) = unit {
                format!("Length::new({}, \"{}\")", value, u)
            } else {
                format!("{}_f64", value)
            }
        }
        StyleValueChange::String(s) => format!("\"{}\".to_string()", s),
        StyleValueChange::Bool(b) => b.to_string(),
        StyleValueChange::Token(t) => format!("Token::ref_(\"{}\")", t),
        StyleValueChange::Enum(e) => e.clone(),
        StyleValueChange::Unset => "StyleValue::Unset".to_string(),
    }
}

fn format_value_toml(value: &serde_json::Value) -> Option<String> {
    match value {
        serde_json::Value::String(s) => Some(format!("\"{}\"", s)),
        serde_json::Value::Number(n) => Some(n.to_string()),
        serde_json::Value::Bool(b) => Some(b.to_string()),
        serde_json::Value::Null => None,
        serde_json::Value::Array(arr) => {
            let items: Vec<String> = arr.iter().filter_map(format_value_toml).collect();
            Some(format!("[{}]", items.join(", ")))
        }
        serde_json::Value::Object(_) => None, // Tables need special handling in TOML
    }
}

fn format_style_value_toml(value: &StyleValueChange) -> String {
    match value {
        StyleValueChange::Color(c) => format!("\"{}\"", c),
        StyleValueChange::Number { value, unit } => {
            if let Some(u) = unit {
                format!("\"{}{}\"", value, u)
            } else {
                format!("{}", value)
            }
        }
        StyleValueChange::String(s) => format!("\"{}\"", s),
        StyleValueChange::Bool(b) => b.to_string(),
        StyleValueChange::Token(t) => format!("\"${}\"", t),
        StyleValueChange::Enum(e) => format!("\"{}\"", e),
        StyleValueChange::Unset => "\"unset\"".to_string(),
    }
}

fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_export_oui() {
        let mut node = ComponentNode::new("ui.Button");
        node.props.insert("label".to_string(), serde_json::json!("Click me"));
        node.styles.insert("color".to_string(), serde_json::json!("#FF0000"));

        let tree = ComponentTree::new();
        let exporter = CodeExporter::new();

        let code = exporter.export_node(&node, &tree);
        assert!(code.contains("Button"));
        assert!(code.contains("label"));
        assert!(code.contains("Click me"));
    }

    #[test]
    fn test_export_json() {
        let mut node = ComponentNode::new("ui.Button");
        node.props.insert("label".to_string(), serde_json::json!("Test"));

        let tree = ComponentTree::new();
        let exporter = CodeExporter::with_options(ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        });

        let json = exporter.export_node(&node, &tree);
        assert!(json.contains("ui.Button"));
        assert!(json.contains("\"label\""));
    }

    #[test]
    fn test_export_overrides() {
        let mut overrides = HashMap::new();
        overrides.insert("color".to_string(), StyleValueChange::color("#FF0000"));
        overrides.insert("padding".to_string(), StyleValueChange::number(16.0));

        let exporter = CodeExporter::new();

        let oui = exporter.export_overrides(&overrides);
        assert!(oui.contains("color"));
        assert!(oui.contains("#FF0000"));

        let exporter_json = CodeExporter::with_options(ExportOptions {
            format: ExportFormat::Json,
            ..Default::default()
        });
        let json = exporter_json.export_overrides(&overrides);
        assert!(json.contains("\"color\""));
    }

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("backgroundColor"), "background_color");
        assert_eq!(to_snake_case("fontSize"), "font_size");
        assert_eq!(to_snake_case("color"), "color");
    }
}
