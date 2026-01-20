//! Accessibility (A11y) validation
//!
//! Validates UI components against WCAG guidelines for accessibility compliance.
//!
//! # Checks Performed
//!
//! - Semantic roles (button, checkbox, textbox, etc.)
//! - Keyboard navigation (tab order, arrow keys)
//! - Screen reader compatibility (ARIA labels)
//! - Color contrast (WCAG AA/AAA)
//! - Focus visibility and management

use crate::{A11yConfig, ErrorCode, WcagLevel};
use oxide_compiler::{compile, ComponentIR, PropertyValue};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Accessibility report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yReport {
    /// Files analyzed
    pub files_analyzed: usize,
    /// WCAG level checked
    pub wcag_level: String,
    /// Total errors
    pub errors: usize,
    /// Total warnings
    pub warnings: usize,
    /// All violations
    pub violations: Vec<A11yViolation>,
    /// Whether a11y check passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl A11yReport {
    pub fn new(level: WcagLevel) -> Self {
        Self {
            files_analyzed: 0,
            wcag_level: level.to_string(),
            errors: 0,
            warnings: 0,
            violations: Vec::new(),
            passed: true,
            duration_ms: 0,
        }
    }

    pub fn add_violation(&mut self, violation: A11yViolation) {
        if violation.is_error {
            self.errors += 1;
            self.passed = false;
        } else {
            self.warnings += 1;
        }
        self.violations.push(violation);
    }
}

/// An accessibility violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A11yViolation {
    /// Error code
    pub code: ErrorCode,
    /// Rule that was violated
    pub rule: String,
    /// File path
    pub file: String,
    /// Line number
    pub line: usize,
    /// Column number
    pub column: usize,
    /// Component involved
    pub component: String,
    /// Violation message
    pub message: String,
    /// WCAG criterion
    pub wcag_criterion: String,
    /// Whether this is an error (vs warning)
    pub is_error: bool,
    /// Suggested fix
    pub fix: Option<String>,
    /// Impact level
    pub impact: A11yImpact,
}

impl A11yViolation {
    pub fn error(
        code: ErrorCode,
        rule: &str,
        file: &str,
        line: usize,
        component: &str,
        message: &str,
        wcag: &str,
        impact: A11yImpact,
    ) -> Self {
        Self {
            code,
            rule: rule.to_string(),
            file: file.to_string(),
            line,
            column: 1,
            component: component.to_string(),
            message: message.to_string(),
            wcag_criterion: wcag.to_string(),
            is_error: true,
            fix: None,
            impact,
        }
    }

    pub fn warning(
        code: ErrorCode,
        rule: &str,
        file: &str,
        line: usize,
        component: &str,
        message: &str,
        wcag: &str,
        impact: A11yImpact,
    ) -> Self {
        Self {
            code,
            rule: rule.to_string(),
            file: file.to_string(),
            line,
            column: 1,
            component: component.to_string(),
            message: message.to_string(),
            wcag_criterion: wcag.to_string(),
            is_error: false,
            fix: None,
            impact,
        }
    }

    pub fn with_fix(mut self, fix: &str) -> Self {
        self.fix = Some(fix.to_string());
        self
    }
}

/// Impact level for accessibility issues
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum A11yImpact {
    /// Prevents access entirely
    Critical,
    /// Significantly degrades experience
    Serious,
    /// Moderate impact on usability
    Moderate,
    /// Minor inconvenience
    Minor,
}

/// Run accessibility checks on a project
pub fn check(project_path: &Path, config: &A11yConfig) -> A11yReport {
    let start = std::time::Instant::now();
    let mut report = A11yReport::new(config.wcag_level);

    if !config.enabled {
        tracing::debug!("A11y checks disabled");
        return report;
    }

    // Find all .oui files
    let files = find_oui_files(project_path);
    report.files_analyzed = files.len();

    tracing::info!("Running a11y checks on {} files", files.len());

    for file in &files {
        if let Err(e) = analyze_file(file, config, &mut report) {
            tracing::warn!("Failed to analyze {:?}: {}", file, e);
        }
    }

    report.duration_ms = start.elapsed().as_millis() as u64;
    report
}

/// Find all .oui files
fn find_oui_files(project_path: &Path) -> Vec<std::path::PathBuf> {
    let mut files = Vec::new();

    for entry in WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().map(|e| e == "oui").unwrap_or(false) {
            files.push(path.to_path_buf());
        }
    }

    files
}

/// Analyze a single file for accessibility issues
fn analyze_file(
    file: &Path,
    config: &A11yConfig,
    report: &mut A11yReport,
) -> Result<(), crate::QualityError> {
    let source = std::fs::read_to_string(file)?;
    let file_str = file.to_string_lossy().to_string();

    match compile(&source) {
        Ok(ir) => {
            let mut context = A11yContext::new(&source, &file_str, config);
            check_component(&ir, &mut context, report, 0);
        }
        Err(_) => {
            // Skip files that don't compile (handled by lint)
        }
    }

    Ok(())
}

/// Context for accessibility analysis
struct A11yContext<'a> {
    source: &'a str,
    file: &'a str,
    config: &'a A11yConfig,
    source_lines: Vec<&'a str>,
    heading_levels: Vec<u32>,
    focus_elements: Vec<String>,
}

impl<'a> A11yContext<'a> {
    fn new(source: &'a str, file: &'a str, config: &'a A11yConfig) -> Self {
        Self {
            source,
            file,
            config,
            source_lines: source.lines().collect(),
            heading_levels: Vec::new(),
            focus_elements: Vec::new(),
        }
    }

    fn find_component_line(&self, component_name: &str) -> usize {
        for (i, line) in self.source_lines.iter().enumerate() {
            if line.contains(&format!("{} {{", component_name)) || line.contains(&format!("{}{{", component_name)) {
                return i + 1;
            }
        }
        1
    }
}

/// Check a component and its children for accessibility
fn check_component(
    ir: &ComponentIR,
    context: &mut A11yContext,
    report: &mut A11yReport,
    depth: usize,
) {
    let line = context.find_component_line(&ir.kind);

    // Skip exempt components
    if context.config.exempt.contains(&ir.kind) {
        return;
    }

    // Check for semantic roles
    if context.config.check_aria {
        check_semantic_role(ir, context, report, line);
    }

    // Check keyboard navigation
    if context.config.check_keyboard {
        check_keyboard_access(ir, context, report, line);
    }

    // Check focus visibility
    if context.config.check_focus {
        check_focus_management(ir, context, report, line);
    }

    // Check color contrast
    if context.config.check_contrast {
        check_color_contrast(ir, context, report, line);
    }

    // Check ARIA attributes
    if context.config.check_aria {
        check_aria_attributes(ir, context, report, line);
    }

    // Component-specific checks
    check_component_specific(ir, context, report, line);

    // Check children
    for child in &ir.children {
        check_component(child, context, report, depth + 1);
    }
}

/// Check for proper semantic roles
fn check_semantic_role(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    let interactive_components = [
        ("Button", "button"),
        ("Checkbox", "checkbox"),
        ("Radio", "radio"),
        ("Slider", "slider"),
        ("Switch", "switch"),
        ("Select", "combobox"),
        ("TextInput", "textbox"),
        ("Input", "textbox"),
        ("Link", "link"),
        ("Tab", "tab"),
        ("TabList", "tablist"),
        ("Menu", "menu"),
        ("MenuItem", "menuitem"),
        ("Dialog", "dialog"),
        ("Alert", "alert"),
        ("Tooltip", "tooltip"),
        ("Table", "table"),
        ("Grid", "grid"),
        ("Tree", "tree"),
        ("TreeItem", "treeitem"),
    ];

    // Check if interactive component has proper role
    if let Some((_, expected_role)) = interactive_components.iter().find(|(c, _)| *c == ir.kind) {
        let has_role = ir.props.iter().any(|p| p.name == "role" || p.name == "aria_role");

        // It's okay if the component has the expected implicit role
        // But if they override it, warn if it's incorrect
        if has_role {
            if let Some(role_prop) = ir.props.iter().find(|p| p.name == "role" || p.name == "aria_role") {
                if let PropertyValue::String(role) = &role_prop.value {
                    if role != *expected_role && !is_valid_role_override(expected_role, role) {
                        report.add_violation(
                            A11yViolation::warning(
                                ErrorCode::a11y(102),
                                "semantic-role",
                                context.file,
                                line,
                                &ir.kind,
                                &format!(
                                    "Component '{}' has role '{}' but '{}' may be more appropriate",
                                    ir.kind, role, expected_role
                                ),
                                "4.1.2",
                                A11yImpact::Moderate,
                            )
                            .with_fix(&format!("Consider using role=\"{}\"", expected_role))
                        );
                    }
                }
            }
        }
    }

    // Generic containers with click handlers should have a role
    let has_click = ir.props.iter().any(|p|
        matches!(p.name.as_str(), "on_click" | "onclick" | "on_press" | "onpress")
    );

    let is_naturally_interactive = interactive_components.iter().any(|(c, _)| *c == ir.kind);
    let has_explicit_role = ir.props.iter().any(|p| p.name == "role" || p.name == "aria_role");

    if has_click && !is_naturally_interactive && !has_explicit_role {
        report.add_violation(
            A11yViolation::warning(
                ErrorCode::a11y(103),
                "clickable-role",
                context.file,
                line,
                &ir.kind,
                &format!(
                    "Clickable '{}' should have an explicit ARIA role for screen readers",
                    ir.kind
                ),
                "4.1.2",
                A11yImpact::Serious,
            )
            .with_fix("Add role=\"button\" or another appropriate ARIA role")
        );
    }
}

/// Check if role override is valid
fn is_valid_role_override(expected: &str, actual: &str) -> bool {
    // Some roles can be validly overridden
    match (expected, actual) {
        ("button", "switch") => true,
        ("button", "menuitem") => true,
        ("button", "tab") => true,
        ("textbox", "searchbox") => true,
        ("combobox", "listbox") => true,
        _ => false,
    }
}

/// Check keyboard accessibility
fn check_keyboard_access(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    let interactive_components = ["Button", "Checkbox", "Radio", "Slider", "Switch", "Select", "TextInput", "Input", "Link", "Tab"];

    if interactive_components.contains(&ir.kind.as_str()) {
        // Check for explicit keyboard handling
        let has_keyboard_handler = ir.props.iter().any(|p|
            matches!(p.name.as_str(), "on_key_down" | "onkeydown" | "on_key_press" | "onkeypress" | "on_key_up" | "onkeyup")
        );

        // Check for tabindex
        let has_tabindex = ir.props.iter().any(|p| p.name == "tabindex" || p.name == "tab_index");

        // Check for disabled state
        let is_disabled = ir.props.iter().any(|p| {
            p.name == "disabled" && matches!(&p.value, PropertyValue::Bool(true))
        });

        // Positive tabindex is discouraged
        if has_tabindex {
            if let Some(tabindex_prop) = ir.props.iter().find(|p| p.name == "tabindex" || p.name == "tab_index") {
                if let PropertyValue::Number(n) = &tabindex_prop.value {
                    if *n > 0.0 {
                        report.add_violation(
                            A11yViolation::warning(
                                ErrorCode::a11y(201),
                                "positive-tabindex",
                                context.file,
                                line,
                                &ir.kind,
                                "Positive tabindex disrupts natural tab order",
                                "2.4.3",
                                A11yImpact::Moderate,
                            )
                            .with_fix("Use tabindex=\"0\" for focusable or remove to use natural order")
                        );
                    }
                }
            }
        }

        // Check if disabled elements are still in tab order
        if is_disabled && has_tabindex {
            if let Some(tabindex_prop) = ir.props.iter().find(|p| p.name == "tabindex" || p.name == "tab_index") {
                if let PropertyValue::Number(n) = &tabindex_prop.value {
                    if *n >= 0.0 {
                        report.add_violation(
                            A11yViolation::warning(
                                ErrorCode::a11y(202),
                                "disabled-in-tab-order",
                                context.file,
                                line,
                                &ir.kind,
                                "Disabled element is still in tab order",
                                "2.1.1",
                                A11yImpact::Minor,
                            )
                            .with_fix("Set tabindex=\"-1\" for disabled elements")
                        );
                    }
                }
            }
        }
    }

    // Dialogs should trap focus
    if ir.kind == "Dialog" || ir.kind == "Modal" {
        let has_focus_trap = ir.props.iter().any(|p|
            matches!(p.name.as_str(), "focus_trap" | "focusTrap" | "trap_focus")
        );

        if !has_focus_trap {
            report.add_violation(
                A11yViolation::warning(
                    ErrorCode::a11y(203),
                    "dialog-focus-trap",
                    context.file,
                    line,
                    &ir.kind,
                    "Dialogs should trap focus to prevent users from navigating outside",
                    "2.4.3",
                    A11yImpact::Serious,
                )
                .with_fix("Add focus_trap property or implement focus management")
            );
        }
    }
}

/// Check focus visibility
fn check_focus_management(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    let interactive_components = ["Button", "Checkbox", "Radio", "Slider", "Switch", "Select", "TextInput", "Input", "Link"];

    if interactive_components.contains(&ir.kind.as_str()) {
        // Check if focus styles are explicitly removed
        let removes_focus_style = ir.style.iter().any(|p| {
            (p.name == "outline" && matches!(&p.value, PropertyValue::String(s) if s == "none" || s == "0"))
            || (p.name == "focus_ring" && matches!(&p.value, PropertyValue::Bool(false)))
        });

        // Check if custom focus style is provided
        let has_custom_focus = ir.style.iter().any(|p|
            matches!(p.name.as_str(), "focus_color" | "focus_ring" | "focus_outline" | "focus_border")
        ) || ir.props.iter().any(|p|
            matches!(p.name.as_str(), "focus_style" | "focusStyle")
        );

        if removes_focus_style && !has_custom_focus {
            report.add_violation(
                A11yViolation::error(
                    ErrorCode::a11y(301),
                    "focus-visible",
                    context.file,
                    line,
                    &ir.kind,
                    "Focus indicator is removed without providing an alternative",
                    "2.4.7",
                    A11yImpact::Serious,
                )
                .with_fix("Provide a visible focus indicator or use focus_ring property")
            );
        }
    }

    // Check for autofocus
    let has_autofocus = ir.props.iter().any(|p|
        matches!(p.name.as_str(), "autofocus" | "auto_focus")
    );

    if has_autofocus {
        report.add_violation(
            A11yViolation::warning(
                ErrorCode::a11y(302),
                "autofocus",
                context.file,
                line,
                &ir.kind,
                "Autofocus can disorient users, especially those using screen readers",
                "3.2.1",
                A11yImpact::Moderate,
            )
            .with_fix("Consider managing focus programmatically based on user actions")
        );
    }
}

/// Check color contrast
fn check_color_contrast(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    // Find text color and background color
    let text_color = find_color_prop(&ir.props, &ir.style, &["color", "text_color"]);
    let bg_color = find_color_prop(&ir.props, &ir.style, &["background", "background_color", "bg"]);

    if let (Some(text), Some(bg)) = (text_color, bg_color) {
        if let (Some(text_rgb), Some(bg_rgb)) = (parse_color(&text), parse_color(&bg)) {
            let contrast = calculate_contrast_ratio(text_rgb, bg_rgb);

            // Check font size to determine threshold
            let font_size = find_number_prop(&ir.props, &ir.style, &["size", "font_size"]);
            let is_large_text = font_size.map(|s| s >= 18.0 || (s >= 14.0 && is_bold(&ir.props))).unwrap_or(false);

            let min_contrast = if is_large_text {
                context.config.min_contrast_large
            } else {
                context.config.min_contrast_normal
            };

            if contrast < min_contrast {
                let level = if is_large_text { "large" } else { "normal" };
                report.add_violation(
                    A11yViolation::error(
                        ErrorCode::a11y(401),
                        "color-contrast",
                        context.file,
                        line,
                        &ir.kind,
                        &format!(
                            "Insufficient color contrast ({:.2}:1) for {} text. Minimum required: {:.1}:1",
                            contrast, level, min_contrast
                        ),
                        "1.4.3",
                        A11yImpact::Serious,
                    )
                    .with_fix("Increase the contrast between text and background colors")
                );
            }
        }
    }
}

/// Check ARIA attributes
fn check_aria_attributes(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    // Valid ARIA attributes
    let valid_aria = [
        "aria_label", "aria_labelledby", "aria_describedby", "aria_hidden",
        "aria_expanded", "aria_selected", "aria_checked", "aria_pressed",
        "aria_disabled", "aria_readonly", "aria_required", "aria_invalid",
        "aria_current", "aria_live", "aria_atomic", "aria_busy",
        "aria_controls", "aria_owns", "aria_haspopup", "aria_autocomplete",
        "aria_valuemin", "aria_valuemax", "aria_valuenow", "aria_valuetext",
    ];

    for prop in &ir.props {
        if prop.name.starts_with("aria_") {
            // Check if it's a valid ARIA attribute
            if !valid_aria.contains(&prop.name.as_str()) {
                report.add_violation(
                    A11yViolation::warning(
                        ErrorCode::a11y(501),
                        "invalid-aria",
                        context.file,
                        line,
                        &ir.kind,
                        &format!("Unknown ARIA attribute '{}'", prop.name),
                        "4.1.2",
                        A11yImpact::Minor,
                    )
                    .with_fix("Use a valid ARIA attribute or remove it")
                );
            }

            // Check for empty ARIA labels
            if prop.name == "aria_label" || prop.name == "aria_labelledby" {
                if let PropertyValue::String(s) = &prop.value {
                    if s.trim().is_empty() {
                        report.add_violation(
                            A11yViolation::error(
                                ErrorCode::a11y(502),
                                "empty-aria-label",
                                context.file,
                                line,
                                &ir.kind,
                                "ARIA label is empty",
                                "4.1.2",
                                A11yImpact::Serious,
                            )
                            .with_fix("Provide a meaningful label or remove the attribute")
                        );
                    }
                }
            }
        }
    }

    // Check aria-hidden on focusable elements
    let is_hidden = ir.props.iter().any(|p| {
        if p.name != "aria_hidden" {
            return false;
        }
        match &p.value {
            PropertyValue::Bool(true) => true,
            PropertyValue::String(s) if s == "true" => true,
            _ => false,
        }
    });

    let is_focusable = ir.props.iter().any(|p| {
        (p.name == "tabindex" || p.name == "tab_index") && matches!(&p.value, PropertyValue::Number(n) if *n >= 0.0)
    }) || ["Button", "Input", "TextInput", "Link", "Checkbox", "Radio", "Select"].contains(&ir.kind.as_str());

    if is_hidden && is_focusable {
        report.add_violation(
            A11yViolation::error(
                ErrorCode::a11y(503),
                "hidden-focusable",
                context.file,
                line,
                &ir.kind,
                "Focusable element is hidden from screen readers with aria-hidden",
                "4.1.2",
                A11yImpact::Critical,
            )
            .with_fix("Remove aria-hidden or make element unfocusable")
        );
    }
}

/// Component-specific accessibility checks
fn check_component_specific(
    ir: &ComponentIR,
    context: &A11yContext,
    report: &mut A11yReport,
    line: usize,
) {
    match ir.kind.as_str() {
        "Image" => {
            // Images must have alt text
            let has_alt = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "alt" | "alt_text" | "aria_label")
            );

            // Decorative images should have empty alt or aria-hidden
            let is_decorative = ir.props.iter().any(|p| {
                (p.name == "decorative" && matches!(&p.value, PropertyValue::Bool(true)))
                || (p.name == "aria_hidden" && matches!(&p.value, PropertyValue::Bool(true)))
            });

            if !has_alt && !is_decorative {
                report.add_violation(
                    A11yViolation::error(
                        ErrorCode::a11y(601),
                        "image-alt",
                        context.file,
                        line,
                        &ir.kind,
                        "Image must have alt text for screen readers",
                        "1.1.1",
                        A11yImpact::Critical,
                    )
                    .with_fix("Add alt=\"description\" or mark as decorative with aria_hidden=\"true\"")
                );
            }
        }

        "Button" => {
            // Buttons must have accessible name
            let has_name = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "label" | "aria_label" | "title" | "content" | "text")
            ) || !ir.children.is_empty();

            if !has_name {
                report.add_violation(
                    A11yViolation::error(
                        ErrorCode::a11y(602),
                        "button-name",
                        context.file,
                        line,
                        &ir.kind,
                        "Button must have an accessible name",
                        "4.1.2",
                        A11yImpact::Critical,
                    )
                    .with_fix("Add label, aria_label, or text content")
                );
            }
        }

        "Input" | "TextInput" => {
            // Form inputs must have labels
            let has_label = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "label" | "aria_label" | "aria_labelledby")
            );

            if !has_label {
                report.add_violation(
                    A11yViolation::error(
                        ErrorCode::a11y(603),
                        "input-label",
                        context.file,
                        line,
                        &ir.kind,
                        "Form input must have a label for screen readers",
                        "1.3.1",
                        A11yImpact::Critical,
                    )
                    .with_fix("Add label or aria_label property")
                );
            }

            // Check for placeholder-only labeling
            let has_only_placeholder = ir.props.iter().any(|p| p.name == "placeholder") && !has_label;

            if has_only_placeholder {
                report.add_violation(
                    A11yViolation::warning(
                        ErrorCode::a11y(604),
                        "placeholder-label",
                        context.file,
                        line,
                        &ir.kind,
                        "Placeholder is not a substitute for a label",
                        "3.3.2",
                        A11yImpact::Moderate,
                    )
                    .with_fix("Add a visible label in addition to placeholder")
                );
            }
        }

        "Link" => {
            // Links must have accessible name
            let has_name = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "label" | "aria_label" | "title" | "content" | "text")
            ) || !ir.children.is_empty();

            if !has_name {
                report.add_violation(
                    A11yViolation::error(
                        ErrorCode::a11y(605),
                        "link-name",
                        context.file,
                        line,
                        &ir.kind,
                        "Link must have accessible text",
                        "2.4.4",
                        A11yImpact::Critical,
                    )
                    .with_fix("Add text content or aria_label")
                );
            }

            // Check for generic link text
            if let Some(text_prop) = ir.props.iter().find(|p| matches!(p.name.as_str(), "content" | "text")) {
                if let PropertyValue::String(text) = &text_prop.value {
                    let generic = ["click here", "here", "read more", "more", "link"];
                    if generic.contains(&text.to_lowercase().as_str()) {
                        report.add_violation(
                            A11yViolation::warning(
                                ErrorCode::a11y(606),
                                "link-purpose",
                                context.file,
                                line,
                                &ir.kind,
                                &format!("Link text '{}' is not descriptive of its purpose", text),
                                "2.4.4",
                                A11yImpact::Moderate,
                            )
                            .with_fix("Use descriptive link text that indicates the destination")
                        );
                    }
                }
            }
        }

        "Video" | "Audio" => {
            // Media must have captions/transcripts
            let has_captions = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "captions" | "subtitles" | "track" | "transcript")
            );

            if !has_captions {
                report.add_violation(
                    A11yViolation::warning(
                        ErrorCode::a11y(607),
                        "media-captions",
                        context.file,
                        line,
                        &ir.kind,
                        "Media should have captions or transcripts",
                        "1.2.2",
                        A11yImpact::Serious,
                    )
                    .with_fix("Add captions, subtitles, or a transcript")
                );
            }
        }

        "Table" => {
            // Tables should have captions or aria-label
            let has_caption = ir.props.iter().any(|p|
                matches!(p.name.as_str(), "caption" | "aria_label" | "aria_labelledby" | "title")
            );

            if !has_caption {
                report.add_violation(
                    A11yViolation::warning(
                        ErrorCode::a11y(608),
                        "table-caption",
                        context.file,
                        line,
                        &ir.kind,
                        "Table should have a caption or aria-label",
                        "1.3.1",
                        A11yImpact::Moderate,
                    )
                    .with_fix("Add caption property or aria_label")
                );
            }
        }

        _ => {}
    }
}

// Helper functions

fn find_color_prop(props: &[oxide_compiler::Property], style: &[oxide_compiler::Property], names: &[&str]) -> Option<String> {
    for prop in props.iter().chain(style.iter()) {
        if names.contains(&prop.name.as_str()) {
            if let PropertyValue::String(s) = &prop.value {
                return Some(s.clone());
            }
        }
    }
    None
}

fn find_number_prop(props: &[oxide_compiler::Property], style: &[oxide_compiler::Property], names: &[&str]) -> Option<f64> {
    for prop in props.iter().chain(style.iter()) {
        if names.contains(&prop.name.as_str()) {
            if let PropertyValue::Number(n) = &prop.value {
                return Some(*n);
            }
        }
    }
    None
}

fn is_bold(props: &[oxide_compiler::Property]) -> bool {
    props.iter().any(|p| {
        (p.name == "font_weight" || p.name == "weight") &&
        matches!(&p.value, PropertyValue::String(s) if s == "bold" || s == "700" || s == "800" || s == "900")
    })
}

/// Parse a color string to RGB values
fn parse_color(color: &str) -> Option<(f64, f64, f64)> {
    let color = color.trim();

    // Hex color
    if color.starts_with('#') {
        let hex = &color[1..];
        if hex.len() == 3 {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()? as f64 / 255.0;
            return Some((r, g, b));
        } else if hex.len() >= 6 {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()? as f64 / 255.0;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()? as f64 / 255.0;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()? as f64 / 255.0;
            return Some((r, g, b));
        }
    }

    // Named colors (common ones)
    match color.to_lowercase().as_str() {
        "white" => Some((1.0, 1.0, 1.0)),
        "black" => Some((0.0, 0.0, 0.0)),
        "red" => Some((1.0, 0.0, 0.0)),
        "green" => Some((0.0, 0.5, 0.0)),
        "blue" => Some((0.0, 0.0, 1.0)),
        "yellow" => Some((1.0, 1.0, 0.0)),
        "gray" | "grey" => Some((0.5, 0.5, 0.5)),
        _ => None,
    }
}

/// Calculate contrast ratio between two colors using WCAG formula
fn calculate_contrast_ratio(fg: (f64, f64, f64), bg: (f64, f64, f64)) -> f64 {
    let fg_luminance = relative_luminance(fg);
    let bg_luminance = relative_luminance(bg);

    let lighter = fg_luminance.max(bg_luminance);
    let darker = fg_luminance.min(bg_luminance);

    (lighter + 0.05) / (darker + 0.05)
}

/// Calculate relative luminance
fn relative_luminance((r, g, b): (f64, f64, f64)) -> f64 {
    let r = if r <= 0.03928 { r / 12.92 } else { ((r + 0.055) / 1.055).powf(2.4) };
    let g = if g <= 0.03928 { g / 12.92 } else { ((g + 0.055) / 1.055).powf(2.4) };
    let b = if b <= 0.03928 { b / 12.92 } else { ((b + 0.055) / 1.055).powf(2.4) };

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_color() {
        assert_eq!(parse_color("#FFFFFF"), Some((1.0, 1.0, 1.0)));
        assert_eq!(parse_color("#000000"), Some((0.0, 0.0, 0.0)));
        assert_eq!(parse_color("#FFF"), Some((1.0, 1.0, 1.0)));
        assert_eq!(parse_color("white"), Some((1.0, 1.0, 1.0)));
    }

    #[test]
    fn test_contrast_ratio() {
        // White on black should be 21:1
        let ratio = calculate_contrast_ratio((1.0, 1.0, 1.0), (0.0, 0.0, 0.0));
        assert!((ratio - 21.0).abs() < 0.1);

        // Same color should be 1:1
        let ratio = calculate_contrast_ratio((0.5, 0.5, 0.5), (0.5, 0.5, 0.5));
        assert!((ratio - 1.0).abs() < 0.1);
    }

    #[test]
    fn test_a11y_report() {
        let mut report = A11yReport::new(WcagLevel::AA);
        assert!(report.passed);

        report.add_violation(A11yViolation::error(
            ErrorCode::a11y(1),
            "test",
            "test.oui",
            1,
            "Test",
            "Test error",
            "1.1.1",
            A11yImpact::Critical,
        ));

        assert!(!report.passed);
        assert_eq!(report.errors, 1);
    }
}
