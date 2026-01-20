//! Static HTML Generator
//!
//! Converts ComponentIR to static HTML + CSS for web deployment.

use crate::{ComponentIR, Property, PropertyValue};

/// Generate static HTML from ComponentIR
pub fn generate_html(ir: &ComponentIR, title: &str) -> String {
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
        "Column" | "Row" | "Container" => {
            let style = build_container_style(&ir.kind, &ir.props, &ir.style);
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
            // Unknown component - render as div
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
        match prop.name.as_str() {
            "size" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("font-size: {}px", n));
                }
            }
            "color" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("color: {}", s));
                }
            }
            "align" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("text-align: {}", s));
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
        match prop.name.as_str() {
            "width" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "fill" {
                        css_parts.push("width: 100%".to_string());
                    } else {
                        css_parts.push(format!("width: {}", s));
                    }
                } else if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("width: {}px", n));
                }
            }
            "height" => {
                if let PropertyValue::String(s) = &prop.value {
                    if s == "fill" {
                        css_parts.push("height: 100%".to_string());
                    } else {
                        css_parts.push(format!("height: {}", s));
                    }
                } else if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("height: {}px", n));
                }
            }
            "gap" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("gap: {}px", n));
                }
            }
            "padding" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("padding: {}px", n));
                }
            }
            "margin_top" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("margin-top: {}px", n));
                }
            }
            "align" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("align-items: {}", s));
                }
            }
            "justify" => {
                if let PropertyValue::String(s) = &prop.value {
                    let val = match s.as_str() {
                        "space_between" => "space-between",
                        "space_around" => "space-around",
                        "space_evenly" => "space-evenly",
                        _ => s.as_str(),
                    };
                    css_parts.push(format!("justify-content: {}", val));
                }
            }
            "background" => {
                if let PropertyValue::String(s) = &prop.value {
                    css_parts.push(format!("background-color: {}", s));
                }
            }
            "radius" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("border-radius: {}px", n));
                }
            }
            "max_width" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("max-width: {}px", n));
                }
            }
            "flex" => {
                if let PropertyValue::Number(n) = &prop.value {
                    css_parts.push(format!("flex: {}", n));
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

fn prop_to_css(prop: &Property) -> Option<String> {
    match prop.name.as_str() {
        "background" => {
            if let PropertyValue::String(s) = &prop.value {
                Some(format!("background-color: {}", s))
            } else {
                None
            }
        }
        "padding" => {
            if let PropertyValue::Number(n) = &prop.value {
                Some(format!("padding: {}px", n))
            } else {
                None
            }
        }
        "radius" => {
            if let PropertyValue::Number(n) = &prop.value {
                Some(format!("border-radius: {}px", n))
            } else {
                None
            }
        }
        "border" => {
            if let PropertyValue::Number(n) = &prop.value {
                Some(format!("border: {}px solid", n))
            } else {
                None
            }
        }
        "border_color" => {
            if let PropertyValue::String(s) = &prop.value {
                Some(format!("border-color: {}", s))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn get_prop_string(props: &[Property], name: &str) -> Option<String> {
    props.iter().find(|p| p.name == name).and_then(|p| {
        if let PropertyValue::String(s) = &p.value {
            Some(s.clone())
        } else {
            None
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
        let html = generate_html(&ir, "Test");

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
        let html = generate_html(&ir, "Test");

        assert!(html.contains("flex-direction: column"));
        assert!(html.contains("flex-direction: row"));
        assert!(html.contains("gap: 16px"));
    }
}
