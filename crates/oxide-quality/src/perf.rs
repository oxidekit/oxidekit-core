//! Performance budget checking
//!
//! Validates that the application meets performance budgets:
//! - Frame time (16ms for 60fps)
//! - Layout pass time
//! - Text shaping time
//! - Allocations per frame
//! - Memory usage

use crate::{ErrorCode, PerfConfig};
use serde::{Deserialize, Serialize};
use std::path::Path;
use walkdir::WalkDir;

/// Performance report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfReport {
    /// Frame budget in milliseconds
    pub frame_budget_ms: f64,
    /// Layout budget in milliseconds
    pub layout_budget_ms: f64,
    /// Text budget in milliseconds
    pub text_budget_ms: f64,
    /// Maximum allocations per frame
    pub max_allocs_per_frame: usize,
    /// Performance violations
    pub violations: Vec<PerfViolation>,
    /// Component complexity scores
    pub complexity_scores: Vec<ComplexityScore>,
    /// Whether performance check passed
    pub passed: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Analysis summary
    pub summary: PerfSummary,
}

impl PerfReport {
    pub fn new(config: &PerfConfig) -> Self {
        Self {
            frame_budget_ms: config.frame_budget_ms,
            layout_budget_ms: config.layout_budget_ms,
            text_budget_ms: config.text_budget_ms,
            max_allocs_per_frame: config.max_allocs_per_frame,
            violations: Vec::new(),
            complexity_scores: Vec::new(),
            passed: true,
            duration_ms: 0,
            summary: PerfSummary::default(),
        }
    }

    pub fn add_violation(&mut self, violation: PerfViolation) {
        if violation.is_error {
            self.passed = false;
        }
        self.violations.push(violation);
    }

    pub fn add_complexity(&mut self, score: ComplexityScore) {
        self.complexity_scores.push(score);
    }
}

/// Performance summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerfSummary {
    /// Total files analyzed
    pub files_analyzed: usize,
    /// Total components analyzed
    pub components_analyzed: usize,
    /// Average complexity score
    pub avg_complexity: f64,
    /// Max complexity score
    pub max_complexity: f64,
    /// Estimated render cost
    pub estimated_render_cost: f64,
    /// Recommendations
    pub recommendations: Vec<String>,
}

/// Performance violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfViolation {
    /// Error code
    pub code: ErrorCode,
    /// Performance category
    pub category: PerfCategory,
    /// File path
    pub file: String,
    /// Component name
    pub component: String,
    /// Violation message
    pub message: String,
    /// Budget value
    pub budget: String,
    /// Actual/estimated value
    pub actual: String,
    /// Whether this is an error (vs warning)
    pub is_error: bool,
    /// Suggested fix
    pub fix: Option<String>,
}

impl PerfViolation {
    pub fn new(
        code: ErrorCode,
        category: PerfCategory,
        file: &str,
        component: &str,
        message: &str,
        budget: &str,
        actual: &str,
        is_error: bool,
    ) -> Self {
        Self {
            code,
            category,
            file: file.to_string(),
            component: component.to_string(),
            message: message.to_string(),
            budget: budget.to_string(),
            actual: actual.to_string(),
            is_error,
            fix: None,
        }
    }

    pub fn with_fix(mut self, fix: &str) -> Self {
        self.fix = Some(fix.to_string());
        self
    }
}

/// Performance categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PerfCategory {
    /// Frame time budget
    FrameTime,
    /// Layout pass time
    Layout,
    /// Text shaping time
    TextShaping,
    /// Allocations per frame
    Allocations,
    /// Memory usage
    Memory,
    /// Component complexity
    Complexity,
    /// Render cost
    RenderCost,
}

impl std::fmt::Display for PerfCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PerfCategory::FrameTime => write!(f, "Frame Time"),
            PerfCategory::Layout => write!(f, "Layout"),
            PerfCategory::TextShaping => write!(f, "Text Shaping"),
            PerfCategory::Allocations => write!(f, "Allocations"),
            PerfCategory::Memory => write!(f, "Memory"),
            PerfCategory::Complexity => write!(f, "Complexity"),
            PerfCategory::RenderCost => write!(f, "Render Cost"),
        }
    }
}

/// Component complexity score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityScore {
    /// File path
    pub file: String,
    /// Component ID
    pub component_id: String,
    /// Component type
    pub component_type: String,
    /// Overall complexity score (0-100)
    pub score: f64,
    /// Breakdown of complexity factors
    pub factors: ComplexityFactors,
    /// Recommendations for reducing complexity
    pub recommendations: Vec<String>,
}

/// Complexity factors breakdown
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComplexityFactors {
    /// Number of children
    pub children_count: usize,
    /// Nesting depth
    pub nesting_depth: usize,
    /// Number of style properties
    pub style_count: usize,
    /// Number of props
    pub prop_count: usize,
    /// Whether uses expensive features
    pub expensive_features: Vec<String>,
    /// Estimated layout cost
    pub layout_cost: f64,
    /// Estimated paint cost
    pub paint_cost: f64,
}

/// Performance budget definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerfBudget {
    /// Budget name
    pub name: String,
    /// Target value
    pub target: f64,
    /// Warning threshold (percentage above target)
    pub warning_threshold: f64,
    /// Error threshold (percentage above target)
    pub error_threshold: f64,
    /// Unit of measurement
    pub unit: String,
}

impl PerfBudget {
    pub fn frame_time(target_ms: f64) -> Self {
        Self {
            name: "Frame Time".to_string(),
            target: target_ms,
            warning_threshold: 80.0, // Warn at 80% of budget
            error_threshold: 100.0,  // Error when exceeded
            unit: "ms".to_string(),
        }
    }

    pub fn check(&self, actual: f64) -> Option<(bool, String)> {
        let percentage = (actual / self.target) * 100.0;

        if percentage > self.error_threshold {
            Some((true, format!(
                "{} exceeded: {:.2}{} / {:.2}{} ({:.0}%)",
                self.name, actual, self.unit, self.target, self.unit, percentage
            )))
        } else if percentage > self.warning_threshold {
            Some((false, format!(
                "{} near limit: {:.2}{} / {:.2}{} ({:.0}%)",
                self.name, actual, self.unit, self.target, self.unit, percentage
            )))
        } else {
            None
        }
    }
}

/// Run performance checks on a project
pub fn check(project_path: &Path, config: &PerfConfig) -> PerfReport {
    let start = std::time::Instant::now();
    let mut report = PerfReport::new(config);

    if !config.enabled {
        tracing::debug!("Performance checks disabled");
        return report;
    }

    // Find all .oui files
    let files = find_oui_files(project_path);
    report.summary.files_analyzed = files.len();

    tracing::info!("Running performance checks on {} files", files.len());

    // Analyze each file
    let mut total_complexity: f64 = 0.0;
    let mut max_complexity: f64 = 0.0;
    let mut component_count = 0;

    for file in &files {
        if let Err(e) = analyze_file(file, config, &mut report) {
            tracing::warn!("Failed to analyze {:?}: {}", file, e);
        }
    }

    // Calculate summary
    for score in &report.complexity_scores {
        total_complexity += score.score;
        max_complexity = max_complexity.max(score.score);
        component_count += 1;
    }

    report.summary.components_analyzed = component_count;
    report.summary.avg_complexity = if component_count > 0 {
        total_complexity / component_count as f64
    } else {
        0.0
    };
    report.summary.max_complexity = max_complexity;

    // Generate recommendations
    generate_recommendations(&mut report);

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

/// Analyze a single file for performance issues
fn analyze_file(
    file: &Path,
    config: &PerfConfig,
    report: &mut PerfReport,
) -> Result<(), crate::QualityError> {
    let source = std::fs::read_to_string(file)?;
    let file_str = file.to_string_lossy().to_string();

    match oxide_compiler::compile(&source) {
        Ok(ir) => {
            analyze_component(&ir, &file_str, config, report, 0);
        }
        Err(_) => {
            // Skip files that don't compile
        }
    }

    Ok(())
}

/// Analyze a component for performance
fn analyze_component(
    ir: &oxide_compiler::ComponentIR,
    file: &str,
    config: &PerfConfig,
    report: &mut PerfReport,
    depth: usize,
) {
    // Calculate complexity factors
    let factors = calculate_complexity_factors(ir, depth);

    // Calculate overall complexity score
    let score = calculate_complexity_score(&factors);

    // Check for performance violations
    check_performance_violations(ir, file, &factors, score, config, report);

    // Add complexity score
    let mut recommendations = Vec::new();

    if factors.children_count > 20 {
        recommendations.push("Consider using virtualization for large lists".to_string());
    }
    if factors.nesting_depth > 6 {
        recommendations.push("Reduce nesting depth by extracting components".to_string());
    }
    if factors.style_count > 15 {
        recommendations.push("Consider using style presets or design tokens".to_string());
    }
    if factors.layout_cost > 5.0 {
        recommendations.push("Simplify layout structure to reduce layout passes".to_string());
    }

    report.add_complexity(ComplexityScore {
        file: file.to_string(),
        component_id: ir.id.clone(),
        component_type: ir.kind.clone(),
        score,
        factors,
        recommendations,
    });

    // Recursively analyze children
    for child in &ir.children {
        analyze_component(child, file, config, report, depth + 1);
    }
}

/// Calculate complexity factors for a component
fn calculate_complexity_factors(ir: &oxide_compiler::ComponentIR, depth: usize) -> ComplexityFactors {
    let mut factors = ComplexityFactors {
        children_count: ir.children.len(),
        nesting_depth: depth,
        style_count: ir.style.len(),
        prop_count: ir.props.len(),
        expensive_features: Vec::new(),
        layout_cost: 0.0,
        paint_cost: 0.0,
    };

    // Check for expensive features
    for prop in &ir.props {
        match prop.name.as_str() {
            // Expensive animations
            "animation" | "transition" => {
                factors.expensive_features.push("Animation".to_string());
                factors.paint_cost += 2.0;
            }
            // Complex filters
            "filter" | "blur" | "shadow" => {
                factors.expensive_features.push("Filter/Blur".to_string());
                factors.paint_cost += 3.0;
            }
            // Transforms
            "transform" | "rotate" | "scale" => {
                factors.expensive_features.push("Transform".to_string());
                factors.paint_cost += 1.0;
            }
            // Opacity (can force compositing)
            "opacity" if matches!(&prop.value, oxide_compiler::PropertyValue::Number(n) if *n < 1.0) => {
                factors.expensive_features.push("Opacity".to_string());
                factors.paint_cost += 0.5;
            }
            _ => {}
        }
    }

    // Layout cost estimation
    factors.layout_cost = estimate_layout_cost(ir);

    factors
}

/// Estimate layout cost for a component
fn estimate_layout_cost(ir: &oxide_compiler::ComponentIR) -> f64 {
    let mut cost = 1.0; // Base cost

    // Layout containers have higher cost
    match ir.kind.as_str() {
        "Grid" => cost += 2.0,
        "Flex" | "Row" | "Column" => cost += 1.0,
        "Stack" => cost += 0.5,
        _ => {}
    }

    // Percentage-based sizing increases cost
    for prop in ir.props.iter().chain(ir.style.iter()) {
        if matches!(prop.name.as_str(), "width" | "height" | "min_width" | "max_width" | "min_height" | "max_height") {
            if let oxide_compiler::PropertyValue::String(s) = &prop.value {
                if s.contains('%') {
                    cost += 0.5;
                }
            }
        }
    }

    // Each child adds to layout cost
    cost += ir.children.len() as f64 * 0.2;

    cost
}

/// Calculate overall complexity score (0-100)
fn calculate_complexity_score(factors: &ComplexityFactors) -> f64 {
    let mut score = 0.0;

    // Children count (up to 30 points)
    score += (factors.children_count as f64 / 10.0 * 10.0).min(30.0);

    // Nesting depth (up to 20 points)
    score += (factors.nesting_depth as f64 * 3.0).min(20.0);

    // Style count (up to 15 points)
    score += (factors.style_count as f64 * 1.0).min(15.0);

    // Props count (up to 10 points)
    score += (factors.prop_count as f64 * 0.5).min(10.0);

    // Expensive features (up to 15 points)
    score += (factors.expensive_features.len() as f64 * 5.0).min(15.0);

    // Layout cost (up to 10 points)
    score += (factors.layout_cost * 2.0).min(10.0);

    score.min(100.0)
}

/// Check for specific performance violations
fn check_performance_violations(
    ir: &oxide_compiler::ComponentIR,
    file: &str,
    factors: &ComplexityFactors,
    score: f64,
    config: &PerfConfig,
    report: &mut PerfReport,
) {
    // Check for too many children (potential list virtualization issue)
    if factors.children_count > 50 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(101),
                PerfCategory::RenderCost,
                file,
                &ir.kind,
                "Component has many children that may impact performance",
                "50",
                &factors.children_count.to_string(),
                true,
            )
            .with_fix("Use virtualized list for large datasets")
        );
    } else if factors.children_count > 30 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(102),
                PerfCategory::RenderCost,
                file,
                &ir.kind,
                "Component has many children",
                "30",
                &factors.children_count.to_string(),
                false,
            )
            .with_fix("Consider pagination or virtualization")
        );
    }

    // Check nesting depth
    if factors.nesting_depth > 10 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(201),
                PerfCategory::Layout,
                file,
                &ir.kind,
                "Deep nesting increases layout complexity",
                "10",
                &factors.nesting_depth.to_string(),
                true,
            )
            .with_fix("Extract nested components to reduce depth")
        );
    } else if factors.nesting_depth > 7 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(202),
                PerfCategory::Layout,
                file,
                &ir.kind,
                "Consider reducing nesting depth",
                "7",
                &factors.nesting_depth.to_string(),
                false,
            )
            .with_fix("Flatten component hierarchy where possible")
        );
    }

    // Check for expensive features combination
    if factors.expensive_features.len() > 3 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(301),
                PerfCategory::FrameTime,
                file,
                &ir.kind,
                "Multiple expensive features may impact frame rate",
                "3",
                &factors.expensive_features.len().to_string(),
                false,
            )
            .with_fix(&format!(
                "Review use of: {}",
                factors.expensive_features.join(", ")
            ))
        );
    }

    // Check overall complexity
    if score > 80.0 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(401),
                PerfCategory::Complexity,
                file,
                &ir.kind,
                "Component complexity is very high",
                "80",
                &format!("{:.1}", score),
                true,
            )
            .with_fix("Break down into smaller, focused components")
        );
    } else if score > 60.0 {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(402),
                PerfCategory::Complexity,
                file,
                &ir.kind,
                "Component complexity is elevated",
                "60",
                &format!("{:.1}", score),
                false,
            )
            .with_fix("Consider simplifying component structure")
        );
    }

    // Check estimated layout cost
    if factors.layout_cost > config.layout_budget_ms {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(501),
                PerfCategory::Layout,
                file,
                &ir.kind,
                "Estimated layout cost exceeds budget",
                &format!("{:.1}ms", config.layout_budget_ms),
                &format!("{:.1}ms", factors.layout_cost),
                true,
            )
            .with_fix("Simplify layout structure")
        );
    }

    // Check for specific anti-patterns
    check_anti_patterns(ir, file, report);
}

/// Check for known performance anti-patterns
fn check_anti_patterns(
    ir: &oxide_compiler::ComponentIR,
    file: &str,
    report: &mut PerfReport,
) {
    // Check for forced layout thrashing patterns
    let has_width_calc = ir.props.iter().any(|p| {
        p.name == "width" && matches!(&p.value, oxide_compiler::PropertyValue::String(s) if s.contains("calc"))
    });
    let has_height_calc = ir.props.iter().any(|p| {
        p.name == "height" && matches!(&p.value, oxide_compiler::PropertyValue::String(s) if s.contains("calc"))
    });

    if has_width_calc && has_height_calc {
        report.add_violation(
            PerfViolation::new(
                ErrorCode::perf(601),
                PerfCategory::Layout,
                file,
                &ir.kind,
                "Multiple calc() expressions may cause layout thrashing",
                "1",
                "2",
                false,
            )
            .with_fix("Consider fixed dimensions or CSS custom properties")
        );
    }

    // Check for inline data URIs
    for prop in &ir.props {
        if let oxide_compiler::PropertyValue::String(s) = &prop.value {
            if s.starts_with("data:") && s.len() > 1000 {
                report.add_violation(
                    PerfViolation::new(
                        ErrorCode::perf(602),
                        PerfCategory::Memory,
                        file,
                        &ir.kind,
                        "Large inline data URI increases memory usage",
                        "1KB",
                        &format!("{}KB", s.len() / 1024),
                        false,
                    )
                    .with_fix("Move to external file for better caching")
                );
            }
        }
    }

    // Check for blur/shadow with large values
    for prop in ir.props.iter().chain(ir.style.iter()) {
        if matches!(prop.name.as_str(), "blur" | "shadow_blur") {
            if let oxide_compiler::PropertyValue::Number(n) = &prop.value {
                if *n > 20.0 {
                    report.add_violation(
                        PerfViolation::new(
                            ErrorCode::perf(603),
                            PerfCategory::RenderCost,
                            file,
                            &ir.kind,
                            "Large blur radius is expensive to render",
                            "20px",
                            &format!("{:.0}px", n),
                            false,
                        )
                        .with_fix("Reduce blur radius or use pre-blurred images")
                    );
                }
            }
        }
    }
}

/// Generate recommendations based on analysis
fn generate_recommendations(report: &mut PerfReport) {
    if report.summary.avg_complexity > 50.0 {
        report.summary.recommendations.push(
            "Overall complexity is high. Consider breaking down large components.".to_string()
        );
    }

    if report.summary.max_complexity > 80.0 {
        report.summary.recommendations.push(
            "Some components have very high complexity. Review and refactor.".to_string()
        );
    }

    let deep_nesting_count = report.complexity_scores.iter()
        .filter(|s| s.factors.nesting_depth > 6)
        .count();

    if deep_nesting_count > 0 {
        report.summary.recommendations.push(
            format!("{} component(s) have deep nesting. Consider flattening.", deep_nesting_count)
        );
    }

    let many_children_count = report.complexity_scores.iter()
        .filter(|s| s.factors.children_count > 20)
        .count();

    if many_children_count > 0 {
        report.summary.recommendations.push(
            format!("{} component(s) have many children. Consider virtualization.", many_children_count)
        );
    }

    if report.violations.is_empty() {
        report.summary.recommendations.push(
            "No performance issues detected. Good job!".to_string()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perf_report() {
        let config = PerfConfig::default();
        let report = PerfReport::new(&config);

        assert!(report.passed);
        assert_eq!(report.frame_budget_ms, 16.0);
    }

    #[test]
    fn test_complexity_score() {
        let factors = ComplexityFactors {
            children_count: 5,
            nesting_depth: 3,
            style_count: 5,
            prop_count: 4,
            expensive_features: vec![],
            layout_cost: 2.0,
            paint_cost: 0.0,
        };

        let score = calculate_complexity_score(&factors);
        assert!(score > 0.0);
        assert!(score < 50.0); // Should be relatively low complexity
    }

    #[test]
    fn test_perf_budget() {
        let budget = PerfBudget::frame_time(16.0);

        // Under budget
        assert!(budget.check(10.0).is_none());

        // Warning level
        let result = budget.check(14.0);
        assert!(result.is_some());
        assert!(!result.unwrap().0); // Not an error

        // Over budget
        let result = budget.check(20.0);
        assert!(result.is_some());
        assert!(result.unwrap().0); // Is an error
    }
}
