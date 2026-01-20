//! OUI file analyzers
//!
//! Implements lint rule checking for both compiled IR and source code.

use super::{LintSeverity, LintViolation};
use crate::{ErrorCode, LintConfig};
use oxide_compiler::{ComponentIR, PropertyValue};
use regex_lite::Regex;

/// Maximum allowed nesting depth
const MAX_NESTING_DEPTH: usize = 10;

/// Maximum recommended children count
const MAX_CHILDREN_COUNT: usize = 50;

/// Large inline data threshold (bytes)
const LARGE_DATA_THRESHOLD: usize = 1024;

/// Analyzer for compiled component IR
pub struct ComponentAnalyzer<'a> {
    source: &'a str,
    file: &'a str,
    source_lines: Vec<&'a str>,
}

impl<'a> ComponentAnalyzer<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Self {
            source,
            file,
            source_lines: source.lines().collect(),
        }
    }

    /// Check a specific rule against a component
    pub fn check_rule(
        &self,
        rule_id: &str,
        ir: &ComponentIR,
        config: &LintConfig,
    ) -> Vec<LintViolation> {
        match rule_id {
            "no-hardcoded-colors" => self.check_hardcoded_colors(ir),
            "no-hardcoded-spacing" => self.check_hardcoded_spacing(ir),
            "no-empty-component" => self.check_empty_component(ir),
            "max-nesting-depth" => self.check_nesting_depth(ir, 0),
            "limit-children" => self.check_children_count(ir),
            "require-alt-text" => self.check_alt_text(ir),
            "require-label" => self.check_label(ir),
            "no-large-inline-data" => self.check_inline_data(ir),
            "heading-order" => vec![], // Handled at file level
            "no-positive-tabindex" => self.check_tabindex(ir),
            "clickable-has-role" => self.check_clickable_role(ir),
            "no-autofocus" => self.check_autofocus(ir),
            _ => vec![],
        }
    }

    /// Check for hardcoded color values
    fn check_hardcoded_colors(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let color_regex = Regex::new(r"^#[0-9A-Fa-f]{3,8}$|^rgb\(|^rgba\(|^hsl\(|^hsla\(").unwrap();
        let color_props = ["color", "background", "border_color", "fill", "stroke"];

        for prop in &ir.props {
            if color_props.contains(&prop.name.as_str()) {
                if let PropertyValue::String(s) = &prop.value {
                    if color_regex.is_match(s) && !s.starts_with("@") && !s.starts_with("$") {
                        let (line, col) = self.find_prop_location(&prop.name);
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(100),
                                "no-hardcoded-colors",
                                LintSeverity::Warning,
                                self.file,
                                line,
                                col,
                                &format!(
                                    "Hardcoded color '{}' in property '{}'. Use a design token instead.",
                                    s, prop.name
                                ),
                            )
                            .with_fix(&format!("Replace with a token: color.primary, color.secondary, etc."))
                        );
                    }
                }
            }
        }

        // Also check style properties
        for prop in &ir.style {
            if color_props.contains(&prop.name.as_str()) {
                if let PropertyValue::String(s) = &prop.value {
                    if color_regex.is_match(s) && !s.starts_with("@") && !s.starts_with("$") {
                        let (line, col) = self.find_prop_location(&prop.name);
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(100),
                                "no-hardcoded-colors",
                                LintSeverity::Warning,
                                self.file,
                                line,
                                col,
                                &format!(
                                    "Hardcoded color '{}' in style property '{}'. Use a design token instead.",
                                    s, prop.name
                                ),
                            )
                            .with_fix("Replace with a token: color.primary, color.secondary, etc.")
                        );
                    }
                }
            }
        }

        violations
    }

    /// Check for hardcoded spacing values
    fn check_hardcoded_spacing(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let spacing_props = [
            "padding", "margin", "gap", "padding_top", "padding_right",
            "padding_bottom", "padding_left", "margin_top", "margin_right",
            "margin_bottom", "margin_left",
        ];
        let allowed_values = [0.0, 4.0, 8.0, 12.0, 16.0, 20.0, 24.0, 32.0, 48.0, 64.0];

        for prop in ir.props.iter().chain(ir.style.iter()) {
            if spacing_props.contains(&prop.name.as_str()) {
                if let PropertyValue::Number(n) = &prop.value {
                    if !allowed_values.contains(n) && *n > 0.0 {
                        let (line, col) = self.find_prop_location(&prop.name);
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(101),
                                "no-hardcoded-spacing",
                                LintSeverity::Warning,
                                self.file,
                                line,
                                col,
                                &format!(
                                    "Non-standard spacing value {} in property '{}'. Use spacing tokens (4, 8, 12, 16, 24, 32, 48, 64).",
                                    n, prop.name
                                ),
                            )
                            .with_fix("Use a spacing token: spacing.sm, spacing.md, spacing.lg, etc.")
                        );
                    }
                }
            }
        }

        violations
    }

    /// Check for empty components
    fn check_empty_component(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        // Components that don't need children
        let self_contained = ["Text", "Image", "Icon", "Spacer", "Divider", "Input", "TextInput"];

        if self_contained.contains(&ir.kind.as_str()) {
            return violations;
        }

        // Check if component has no children and no meaningful props
        let has_content_prop = ir.props.iter().any(|p|
            matches!(p.name.as_str(), "content" | "text" | "src" | "source" | "value")
        );

        if ir.children.is_empty() && !has_content_prop {
            let (line, col) = self.find_component_location(&ir.kind);
            violations.push(
                LintViolation::new(
                    ErrorCode::lint(200),
                    "no-empty-component",
                    LintSeverity::Warning,
                    self.file,
                    line,
                    col,
                    &format!("Component '{}' has no content or children", ir.kind),
                )
                .with_fix("Add children or a content property")
            );
        }

        violations
    }

    /// Check nesting depth
    fn check_nesting_depth(&self, ir: &ComponentIR, depth: usize) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        if depth > MAX_NESTING_DEPTH {
            let (line, col) = self.find_component_location(&ir.kind);
            violations.push(
                LintViolation::new(
                    ErrorCode::lint(104),
                    "max-nesting-depth",
                    LintSeverity::Warning,
                    self.file,
                    line,
                    col,
                    &format!(
                        "Component nesting depth ({}) exceeds maximum ({})",
                        depth, MAX_NESTING_DEPTH
                    ),
                )
                .with_fix("Refactor into smaller, reusable components")
            );
        }

        for child in &ir.children {
            violations.extend(self.check_nesting_depth(child, depth + 1));
        }

        violations
    }

    /// Check children count
    fn check_children_count(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        if ir.children.len() > MAX_CHILDREN_COUNT {
            let (line, col) = self.find_component_location(&ir.kind);
            violations.push(
                LintViolation::new(
                    ErrorCode::lint(401),
                    "limit-children",
                    LintSeverity::Warning,
                    self.file,
                    line,
                    col,
                    &format!(
                        "Component '{}' has {} children, which may impact performance",
                        ir.kind, ir.children.len()
                    ),
                )
                .with_fix("Consider using virtualized lists or pagination")
            );
        }

        violations
    }

    /// Check for alt text on images
    fn check_alt_text(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        if ir.kind == "Image" {
            let has_alt = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "alt" | "alt_text" | "description")
            );

            if !has_alt {
                let (line, col) = self.find_component_location(&ir.kind);
                violations.push(
                    LintViolation::new(
                        ErrorCode::lint(300),
                        "require-alt-text",
                        LintSeverity::Error,
                        self.file,
                        line,
                        col,
                        "Image component must have alt text for accessibility",
                    )
                    .with_fix("Add an 'alt' property with descriptive text")
                );
            }
        }

        violations
    }

    /// Check for labels on interactive elements
    fn check_label(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let interactive = ["Button", "Input", "TextInput", "Checkbox", "Radio", "Select", "Slider"];

        if interactive.contains(&ir.kind.as_str()) {
            let has_label = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "label" | "aria_label" | "aria_labelledby" | "title")
            );

            // For buttons, content or text also works
            let is_button = ir.kind == "Button";
            let has_text = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "content" | "text")
            );

            if !has_label && !(is_button && has_text) {
                let (line, col) = self.find_component_location(&ir.kind);
                violations.push(
                    LintViolation::new(
                        ErrorCode::lint(301),
                        "require-label",
                        LintSeverity::Error,
                        self.file,
                        line,
                        col,
                        &format!(
                            "Interactive component '{}' must have a label for accessibility",
                            ir.kind
                        ),
                    )
                    .with_fix("Add a 'label' or 'aria_label' property")
                );
            }
        }

        violations
    }

    /// Check for large inline data
    fn check_inline_data(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        for prop in &ir.props {
            if let PropertyValue::String(s) = &prop.value {
                if s.len() > LARGE_DATA_THRESHOLD {
                    // Check if it looks like inline data (base64, data URIs, etc.)
                    if s.starts_with("data:") || s.chars().all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=') {
                        let (line, col) = self.find_prop_location(&prop.name);
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(400),
                                "no-large-inline-data",
                                LintSeverity::Warning,
                                self.file,
                                line,
                                col,
                                &format!(
                                    "Large inline data ({} bytes) in property '{}'. Use external resources instead.",
                                    s.len(), prop.name
                                ),
                            )
                            .with_fix("Move data to an external file and reference it")
                        );
                    }
                }
            }
        }

        violations
    }

    /// Check for positive tabindex values
    fn check_tabindex(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        for prop in &ir.props {
            if prop.name == "tabindex" || prop.name == "tab_index" {
                if let PropertyValue::Number(n) = &prop.value {
                    if *n > 0.0 {
                        let (line, col) = self.find_prop_location(&prop.name);
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(305),
                                "no-positive-tabindex",
                                LintSeverity::Warning,
                                self.file,
                                line,
                                col,
                                &format!(
                                    "Positive tabindex value ({}). This disrupts natural tab order.",
                                    n
                                ),
                            )
                            .with_fix("Use tabindex=\"0\" to make focusable or tabindex=\"-1\" to remove from tab order")
                        );
                    }
                }
            }
        }

        violations
    }

    /// Check for clickable elements without proper roles
    fn check_clickable_role(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        // Check if component has click handler but is not naturally interactive
        let has_click = ir.props.iter().any(|p|
            matches!(p.name.as_str(), "on_click" | "onclick" | "on_press")
        );

        let naturally_interactive = ["Button", "Link", "Input", "TextInput", "Checkbox", "Radio", "Select"];

        if has_click && !naturally_interactive.contains(&ir.kind.as_str()) {
            let has_role = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "role" | "aria_role")
            );

            if !has_role {
                let (line, col) = self.find_component_location(&ir.kind);
                violations.push(
                    LintViolation::new(
                        ErrorCode::lint(306),
                        "clickable-has-role",
                        LintSeverity::Warning,
                        self.file,
                        line,
                        col,
                        &format!(
                            "Clickable '{}' component should have an explicit role for accessibility",
                            ir.kind
                        ),
                    )
                    .with_fix("Add role=\"button\" or another appropriate ARIA role")
                );
            }
        }

        violations
    }

    /// Check for autofocus usage
    fn check_autofocus(&self, ir: &ComponentIR) -> Vec<LintViolation> {
        let mut violations = Vec::new();

        let has_autofocus = ir.props.iter().any(|p|
            matches!(p.name.as_str(), "autofocus" | "auto_focus")
        );

        if has_autofocus {
            let (line, col) = self.find_component_location(&ir.kind);
            violations.push(
                LintViolation::new(
                    ErrorCode::lint(307),
                    "no-autofocus",
                    LintSeverity::Warning,
                    self.file,
                    line,
                    col,
                    "Autofocus can disorient users, especially those using screen readers",
                )
                .with_fix("Consider managing focus programmatically based on user actions")
            );
        }

        violations
    }

    /// Find the location of a property in source
    fn find_prop_location(&self, prop_name: &str) -> (usize, usize) {
        for (line_num, line) in self.source_lines.iter().enumerate() {
            if let Some(col) = line.find(&format!("{}:", prop_name)) {
                return (line_num + 1, col + 1);
            }
        }
        (1, 1)
    }

    /// Find the location of a component in source
    fn find_component_location(&self, component_name: &str) -> (usize, usize) {
        for (line_num, line) in self.source_lines.iter().enumerate() {
            if let Some(col) = line.find(&format!("{} {{", component_name)) {
                return (line_num + 1, col + 1);
            }
            if let Some(col) = line.find(&format!("{}{{", component_name)) {
                return (line_num + 1, col + 1);
            }
        }
        (1, 1)
    }
}

/// Analyzer for source code (before compilation)
pub struct SourceAnalyzer<'a> {
    source: &'a str,
    file: &'a str,
    source_lines: Vec<&'a str>,
}

impl<'a> SourceAnalyzer<'a> {
    pub fn new(source: &'a str, file: &'a str) -> Self {
        Self {
            source,
            file,
            source_lines: source.lines().collect(),
        }
    }

    /// Check a specific rule against source code
    pub fn check_rule(&self, rule_id: &str, config: &LintConfig) -> Vec<LintViolation> {
        match rule_id {
            "duplicate-prop" => self.check_duplicate_props(),
            "heading-order" => self.check_heading_order(),
            "no-inline-scripts" => self.check_inline_scripts(),
            "consistent-naming" => self.check_naming(),
            _ => vec![],
        }
    }

    /// Check for duplicate property definitions
    fn check_duplicate_props(&self) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let prop_regex = Regex::new(r"(\w+)\s*:").unwrap();

        let mut in_component = false;
        let mut current_props: Vec<(String, usize)> = Vec::new();
        let mut brace_depth = 0;

        for (line_num, line) in self.source_lines.iter().enumerate() {
            let trimmed = line.trim();

            // Track brace depth
            brace_depth += trimmed.matches('{').count();
            brace_depth = brace_depth.saturating_sub(trimmed.matches('}').count());

            // Entering a component
            if trimmed.contains('{') && !trimmed.starts_with("style") && !trimmed.starts_with("//") {
                if brace_depth == 1 {
                    current_props.clear();
                }
            }

            // Check for props
            for cap in prop_regex.captures_iter(line) {
                if let Some(prop_match) = cap.get(1) {
                    let prop_name = prop_match.as_str();

                    // Skip style block identifiers
                    if prop_name == "style" {
                        continue;
                    }

                    // Check for duplicates
                    if let Some((_, first_line)) = current_props.iter().find(|(name, _)| name == prop_name) {
                        let col = line.find(prop_name).unwrap_or(0) + 1;
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(6),
                                "duplicate-prop",
                                LintSeverity::Error,
                                self.file,
                                line_num + 1,
                                col,
                                &format!(
                                    "Property '{}' is defined multiple times (first defined at line {})",
                                    prop_name, first_line
                                ),
                            )
                            .with_fix("Remove the duplicate property definition")
                        );
                    } else {
                        current_props.push((prop_name.to_string(), line_num + 1));
                    }
                }
            }

            // Reset on component exit
            if brace_depth == 0 {
                current_props.clear();
            }
        }

        violations
    }

    /// Check heading order
    fn check_heading_order(&self) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let heading_regex = Regex::new(r"Heading(\d)|H(\d)\s*\{").unwrap();
        let mut last_level = 0;

        for (line_num, line) in self.source_lines.iter().enumerate() {
            if let Some(cap) = heading_regex.captures(line) {
                let level = cap.get(1)
                    .or_else(|| cap.get(2))
                    .and_then(|m| m.as_str().parse::<u32>().ok())
                    .unwrap_or(0);

                if level > 0 && last_level > 0 && level > last_level + 1 {
                    let col = line.find("Heading").or_else(|| line.find("H")).unwrap_or(0) + 1;
                    violations.push(
                        LintViolation::new(
                            ErrorCode::lint(304),
                            "heading-order",
                            LintSeverity::Warning,
                            self.file,
                            line_num + 1,
                            col,
                            &format!(
                                "Heading level skipped from H{} to H{}. Don't skip heading levels.",
                                last_level, level
                            ),
                        )
                        .with_fix(&format!("Use H{} instead or add intermediate heading levels", last_level + 1))
                    );
                }

                if level > 0 {
                    last_level = level;
                }
            }
        }

        violations
    }

    /// Check for inline scripts
    fn check_inline_scripts(&self) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let script_patterns = [
            "javascript:",
            "eval(",
            "Function(",
            "<script",
            "onclick=\"",
        ];

        for (line_num, line) in self.source_lines.iter().enumerate() {
            for pattern in &script_patterns {
                if let Some(col) = line.to_lowercase().find(&pattern.to_lowercase()) {
                    violations.push(
                        LintViolation::new(
                            ErrorCode::lint(500),
                            "no-inline-scripts",
                            LintSeverity::Error,
                            self.file,
                            line_num + 1,
                            col + 1,
                            &format!("Inline script detected: '{}'", pattern),
                        )
                        .with_fix("Use event handlers and separate script files instead")
                    );
                }
            }
        }

        violations
    }

    /// Check naming conventions
    fn check_naming(&self) -> Vec<LintViolation> {
        let mut violations = Vec::new();
        let component_regex = Regex::new(r"^(\s*)([a-z_][a-zA-Z0-9_]*)\s*\{").unwrap();

        for (line_num, line) in self.source_lines.iter().enumerate() {
            // Skip lines that look like property definitions
            if line.contains(':') && !line.contains('{') {
                continue;
            }

            if let Some(cap) = component_regex.captures(line) {
                if let Some(name_match) = cap.get(2) {
                    let name = name_match.as_str();

                    // Component names should be PascalCase
                    if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                        // Skip known keywords
                        if matches!(name, "app" | "style" | "if" | "for" | "else") {
                            continue;
                        }

                        let col = line.find(name).unwrap_or(0) + 1;
                        violations.push(
                            LintViolation::new(
                                ErrorCode::lint(103),
                                "consistent-naming",
                                LintSeverity::Warning,
                                self.file,
                                line_num + 1,
                                col,
                                &format!(
                                    "Component name '{}' should be PascalCase",
                                    name
                                ),
                            )
                            .with_fix(&format!("Rename to '{}'", to_pascal_case(name)))
                        );
                    }
                }
            }
        }

        violations
    }
}

/// Convert string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' || c == '-' {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("hello_world"), "HelloWorld");
        assert_eq!(to_pascal_case("button"), "Button");
        assert_eq!(to_pascal_case("my-component"), "MyComponent");
    }

    #[test]
    fn test_component_analyzer() {
        let source = r##"
            Column {
                color: "#FF0000"
                padding: 15
            }
        "##;

        let analyzer = ComponentAnalyzer::new(source, "test.oui");

        // Create a mock IR
        let ir = ComponentIR {
            id: "col_1".to_string(),
            kind: "Column".to_string(),
            props: vec![
                oxide_compiler::Property {
                    name: "color".to_string(),
                    value: PropertyValue::String("#FF0000".to_string()),
                },
                oxide_compiler::Property {
                    name: "padding".to_string(),
                    value: PropertyValue::Number(15.0),
                },
            ],
            style: vec![],
            children: vec![],
        };

        let config = LintConfig::default();

        let violations = analyzer.check_rule("no-hardcoded-colors", &ir, &config);
        assert!(!violations.is_empty());
    }

    #[test]
    fn test_source_analyzer_duplicates() {
        let source = r#"
            Button {
                label: "Click"
                color: "red"
                label: "Click Again"
            }
        "#;

        let analyzer = SourceAnalyzer::new(source, "test.oui");
        let config = LintConfig::default();

        let violations = analyzer.check_rule("duplicate-prop", &config);
        assert!(!violations.is_empty());
    }
}
