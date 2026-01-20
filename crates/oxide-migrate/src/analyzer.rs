//! Framework Analyzer
//!
//! Detects CSS frameworks (Bootstrap, Tailwind, custom) from HTML/CSS files,
//! inventories components, and calculates migration confidence scores.

use crate::error::{IssueCategory, MigrateError, MigrateResult, MigrationIssue, Severity};
use regex::Regex;
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// CSS framework type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Framework {
    /// Bootstrap CSS framework
    Bootstrap,
    /// Tailwind CSS framework
    Tailwind,
    /// Bulma CSS framework
    Bulma,
    /// Foundation CSS framework
    Foundation,
    /// Custom/unknown CSS framework
    Custom,
    /// Mixed frameworks detected
    Mixed,
}

impl Default for Framework {
    fn default() -> Self {
        Framework::Custom
    }
}

impl std::fmt::Display for Framework {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Framework::Bootstrap => write!(f, "Bootstrap"),
            Framework::Tailwind => write!(f, "Tailwind CSS"),
            Framework::Bulma => write!(f, "Bulma"),
            Framework::Foundation => write!(f, "Foundation"),
            Framework::Custom => write!(f, "Custom"),
            Framework::Mixed => write!(f, "Mixed"),
        }
    }
}

/// Framework version information
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FrameworkVersion {
    /// Major version
    pub major: Option<u32>,
    /// Minor version
    pub minor: Option<u32>,
    /// Patch version
    pub patch: Option<u32>,
    /// Version string as found
    pub raw: Option<String>,
}

impl FrameworkVersion {
    /// Parse version from string like "5.3.2" or "v5.3"
    pub fn parse(s: &str) -> Self {
        let version_re = Regex::new(r"v?(\d+)(?:\.(\d+))?(?:\.(\d+))?").unwrap();

        if let Some(caps) = version_re.captures(s) {
            Self {
                major: caps.get(1).and_then(|m| m.as_str().parse().ok()),
                minor: caps.get(2).and_then(|m| m.as_str().parse().ok()),
                patch: caps.get(3).and_then(|m| m.as_str().parse().ok()),
                raw: Some(s.to_string()),
            }
        } else {
            Self {
                raw: Some(s.to_string()),
                ..Default::default()
            }
        }
    }
}

impl std::fmt::Display for FrameworkVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref raw) = self.raw {
            write!(f, "{}", raw)
        } else if let Some(major) = self.major {
            write!(f, "{}", major)?;
            if let Some(minor) = self.minor {
                write!(f, ".{}", minor)?;
                if let Some(patch) = self.patch {
                    write!(f, ".{}", patch)?;
                }
            }
            Ok(())
        } else {
            write!(f, "unknown")
        }
    }
}

/// Component type detected in the source
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComponentType {
    // Navigation
    Navbar,
    Sidebar,
    Breadcrumb,
    Pagination,
    Tabs,
    Menu,
    Dropdown,

    // Content
    Card,
    Table,
    DataTable,
    List,
    Alert,
    Badge,
    Tag,
    Tooltip,
    Popover,
    Modal,
    Dialog,
    Accordion,
    Carousel,

    // Forms
    Form,
    Input,
    Select,
    Checkbox,
    Radio,
    Switch,
    Slider,
    DatePicker,
    TimePicker,
    FileUpload,
    SearchInput,

    // Buttons
    Button,
    ButtonGroup,
    IconButton,

    // Layout
    Container,
    Grid,
    Row,
    Column,
    Flex,
    Stack,
    Divider,

    // Feedback
    Progress,
    Spinner,
    Skeleton,
    Toast,

    // Media
    Image,
    Avatar,
    Icon,

    // Other
    Chart,
    Calendar,
    Statistics,
    Custom(String),
}

impl std::fmt::Display for ComponentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentType::Custom(name) => write!(f, "Custom({})", name),
            other => write!(f, "{:?}", other),
        }
    }
}

/// A detected component instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedComponent {
    /// Component type
    pub component_type: ComponentType,
    /// CSS classes used
    pub classes: Vec<String>,
    /// Source file where found
    pub source_file: Option<String>,
    /// Number of occurrences
    pub occurrences: usize,
    /// Framework-specific variant (e.g., "primary", "outline")
    pub variant: Option<String>,
    /// Size variant (e.g., "sm", "lg")
    pub size: Option<String>,
    /// Can this component be auto-mapped to OxideKit?
    pub mappable: bool,
    /// Confidence of mapping (0.0 to 1.0)
    pub mapping_confidence: f32,
}

impl DetectedComponent {
    /// Create a new detected component
    pub fn new(component_type: ComponentType) -> Self {
        Self {
            component_type,
            classes: Vec::new(),
            source_file: None,
            occurrences: 1,
            variant: None,
            size: None,
            mappable: true,
            mapping_confidence: 0.8,
        }
    }

    /// Add classes
    pub fn with_classes(mut self, classes: Vec<String>) -> Self {
        self.classes = classes;
        self
    }

    /// Set source file
    pub fn with_file(mut self, file: impl Into<String>) -> Self {
        self.source_file = Some(file.into());
        self
    }
}

/// Complete analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResult {
    /// Detected framework
    pub framework: Framework,
    /// Framework version (if detected)
    pub version: FrameworkVersion,
    /// Confidence in framework detection (0.0 to 1.0)
    pub framework_confidence: f32,
    /// Detected components
    pub components: Vec<DetectedComponent>,
    /// Component inventory summary
    pub inventory: ComponentInventory,
    /// Files analyzed
    pub files_analyzed: FileAnalysisSummary,
    /// Issues found during analysis
    pub issues: Vec<MigrationIssue>,
    /// Overall migration confidence score (0.0 to 1.0)
    pub migration_confidence: f32,
    /// CSS custom properties (variables) found
    pub css_variables: HashMap<String, String>,
    /// Color palette detected
    pub detected_colors: Vec<String>,
    /// Font families detected
    pub detected_fonts: Vec<String>,
}

impl Default for AnalysisResult {
    fn default() -> Self {
        Self {
            framework: Framework::Custom,
            version: FrameworkVersion::default(),
            framework_confidence: 0.0,
            components: Vec::new(),
            inventory: ComponentInventory::default(),
            files_analyzed: FileAnalysisSummary::default(),
            issues: Vec::new(),
            migration_confidence: 0.0,
            css_variables: HashMap::new(),
            detected_colors: Vec::new(),
            detected_fonts: Vec::new(),
        }
    }
}

/// Component inventory summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ComponentInventory {
    /// Total unique component types
    pub total_types: usize,
    /// Total component instances
    pub total_instances: usize,
    /// Components that can be mapped to OxideKit
    pub mappable_count: usize,
    /// Components that need manual conversion
    pub manual_count: usize,
    /// Components with no OxideKit equivalent
    pub unmappable_count: usize,
    /// Breakdown by component type
    pub by_type: HashMap<String, usize>,
}

/// File analysis summary
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FileAnalysisSummary {
    /// HTML files analyzed
    pub html_files: usize,
    /// CSS files analyzed
    pub css_files: usize,
    /// JS/TS files found
    pub js_files: usize,
    /// Total bytes analyzed
    pub total_bytes: u64,
    /// Files that couldn't be parsed
    pub parse_errors: usize,
}

/// Theme analyzer for CSS framework detection
pub struct Analyzer {
    /// Bootstrap class patterns
    bootstrap_patterns: Vec<Regex>,
    /// Tailwind class patterns
    tailwind_patterns: Vec<Regex>,
    /// Component detection patterns
    component_patterns: HashMap<ComponentType, Vec<Regex>>,
    /// CSS variable pattern
    css_var_pattern: Regex,
    /// Font family pattern
    font_pattern: Regex,
}

impl Analyzer {
    /// Create a new analyzer with default patterns
    pub fn new() -> MigrateResult<Self> {
        Ok(Self {
            bootstrap_patterns: Self::build_bootstrap_patterns()?,
            tailwind_patterns: Self::build_tailwind_patterns()?,
            component_patterns: Self::build_component_patterns()?,
            css_var_pattern: Regex::new(r"--([a-zA-Z0-9_-]+)\s*:\s*([^;]+);")?,
            font_pattern: Regex::new(r#"font-family\s*:\s*([^;]+)"#)?,
        })
    }

    fn build_bootstrap_patterns() -> MigrateResult<Vec<Regex>> {
        let patterns = [
            r"\bcontainer(-fluid|-sm|-md|-lg|-xl|-xxl)?\b",
            r"\brow\b",
            r"\bcol(-\d+|-sm|-md|-lg|-xl|-xxl|-auto)?\b",
            r"\bbtn(-primary|-secondary|-success|-danger|-warning|-info|-light|-dark|-outline-\w+)?\b",
            r"\bcard(-header|-body|-footer|-title|-text|-img-top|-img-bottom)?\b",
            r"\bnavbar(-brand|-nav|-toggler|-collapse|-expand-\w+)?\b",
            r"\bnav(-link|-item|-tabs|-pills)?\b",
            r"\balert(-primary|-secondary|-success|-danger|-warning|-info|-light|-dark|-dismissible)?\b",
            r"\bmodal(-dialog|-content|-header|-body|-footer|-title)?\b",
            r"\bform(-control|-label|-text|-check|-select|-range|-floating)?\b",
            r"\btable(-striped|-bordered|-hover|-dark|-sm|-responsive)?\b",
            r"\bbadge(-primary|-secondary|-success|-danger|-warning|-info|-light|-dark)?\b",
            r"\bdropdown(-menu|-item|-toggle|-divider)?\b",
            r"\bprogress(-bar)?\b",
            r"\bspinner(-border|-grow)?\b",
            r"\btoast(-header|-body)?\b",
            r"\baccordion(-item|-header|-body|-button|-collapse)?\b",
            r"\bbreadcrumb(-item)?\b",
            r"\bpagination(-item|-link)?\b",
            r"\blist-group(-item|-flush)?\b",
        ];
        patterns
            .iter()
            .map(|p| Regex::new(p).map_err(MigrateError::from))
            .collect()
    }

    fn build_tailwind_patterns() -> MigrateResult<Vec<Regex>> {
        let patterns = [
            r"\b(sm|md|lg|xl|2xl):[\w-]+\b",                       // Responsive prefixes
            r"\b(hover|focus|active|disabled|group-hover):[\w-]+\b", // State prefixes
            r"\bp[xytblr]?-\d+\b",                                  // Padding
            r"\bm[xytblr]?-\d+\b",                                  // Margin
            r"\bw-(\d+|full|screen|auto|px|1\/[234])\b",           // Width
            r"\bh-(\d+|full|screen|auto|px)\b",                    // Height
            r"\bbg-([\w-]+)(-\d+)?\b",                              // Background
            r"\btext-([\w-]+)(-\d+)?\b",                            // Text
            r"\bflex(-row|-col|-wrap|-nowrap|-1|-auto|-initial|-none)?\b", // Flex
            r"\bgrid(-cols-\d+|-rows-\d+)?\b",                      // Grid
            r"\bgap-\d+\b",                                         // Gap
            r"\brounded(-none|-sm|-md|-lg|-xl|-2xl|-3xl|-full)?\b", // Border radius
            r"\bshadow(-sm|-md|-lg|-xl|-2xl|-inner|-none)?\b",     // Shadow
            r"\bfont-(sans|serif|mono|thin|extralight|light|normal|medium|semibold|bold|extrabold|black)\b",
            r"\bleading-(\d+|none|tight|snug|normal|relaxed|loose)\b",
            r"\btracking-(tighter|tight|normal|wide|wider|widest)\b",
            r"\bborder(-\d+|-t|-r|-b|-l)?\b",
        ];
        patterns
            .iter()
            .map(|p| Regex::new(p).map_err(MigrateError::from))
            .collect()
    }

    fn build_component_patterns() -> MigrateResult<HashMap<ComponentType, Vec<Regex>>> {
        let mut patterns = HashMap::new();

        // Bootstrap component patterns
        patterns.insert(
            ComponentType::Button,
            vec![
                Regex::new(r"\bbtn\b")?,
                Regex::new(r"\bbutton\b")?,
                Regex::new(r#"<button[^>]*>"#)?,
            ],
        );
        patterns.insert(
            ComponentType::Card,
            vec![Regex::new(r"\bcard\b")?, Regex::new(r"\bpanel\b")?],
        );
        patterns.insert(
            ComponentType::Navbar,
            vec![
                Regex::new(r"\bnavbar\b")?,
                Regex::new(r"\bheader\b")?,
                Regex::new(r"\btopbar\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Sidebar,
            vec![
                Regex::new(r"\bsidebar\b")?,
                Regex::new(r"\bside-nav\b")?,
                Regex::new(r"\baside\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Table,
            vec![Regex::new(r"\btable\b")?, Regex::new(r#"<table[^>]*>"#)?],
        );
        patterns.insert(
            ComponentType::DataTable,
            vec![
                Regex::new(r"\bdatatable\b")?,
                Regex::new(r"\bdata-table\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Form,
            vec![
                Regex::new(r"\bform\b")?,
                Regex::new(r#"<form[^>]*>"#)?,
                Regex::new(r"\bform-group\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Input,
            vec![
                Regex::new(r"\bform-control\b")?,
                Regex::new(r#"<input[^>]*>"#)?,
            ],
        );
        patterns.insert(
            ComponentType::Modal,
            vec![
                Regex::new(r"\bmodal\b")?,
                Regex::new(r"\bdialog\b")?,
                Regex::new(r"\bpopup\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Alert,
            vec![
                Regex::new(r"\balert\b")?,
                Regex::new(r"\bnotification\b")?,
                Regex::new(r"\bmessage\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Dropdown,
            vec![
                Regex::new(r"\bdropdown\b")?,
                Regex::new(r"\bselect2\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Tabs,
            vec![
                Regex::new(r"\bnav-tabs\b")?,
                Regex::new(r"\btab-content\b")?,
                Regex::new(r"\btabs\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Accordion,
            vec![
                Regex::new(r"\baccordion\b")?,
                Regex::new(r"\bcollapse\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Progress,
            vec![
                Regex::new(r"\bprogress\b")?,
                Regex::new(r"\bprogress-bar\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Badge,
            vec![
                Regex::new(r"\bbadge\b")?,
                Regex::new(r"\blabel\b")?,
                Regex::new(r"\btag\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Breadcrumb,
            vec![Regex::new(r"\bbreadcrumb\b")?],
        );
        patterns.insert(
            ComponentType::Pagination,
            vec![
                Regex::new(r"\bpagination\b")?,
                Regex::new(r"\bpager\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Spinner,
            vec![
                Regex::new(r"\bspinner\b")?,
                Regex::new(r"\bloading\b")?,
                Regex::new(r"\bloader\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Toast,
            vec![Regex::new(r"\btoast\b")?, Regex::new(r"\bsnackbar\b")?],
        );
        patterns.insert(
            ComponentType::Tooltip,
            vec![
                Regex::new(r"\btooltip\b")?,
                Regex::new(r#"data-toggle="tooltip""#)?,
            ],
        );
        patterns.insert(
            ComponentType::Popover,
            vec![
                Regex::new(r"\bpopover\b")?,
                Regex::new(r#"data-toggle="popover""#)?,
            ],
        );
        patterns.insert(
            ComponentType::Carousel,
            vec![
                Regex::new(r"\bcarousel\b")?,
                Regex::new(r"\bslider\b")?,
                Regex::new(r"\bslick\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Avatar,
            vec![Regex::new(r"\bavatar\b")?, Regex::new(r"\bprofile-img\b")?],
        );
        patterns.insert(
            ComponentType::Statistics,
            vec![
                Regex::new(r"\bstat-card\b")?,
                Regex::new(r"\bstatistics\b")?,
                Regex::new(r"\bwidget\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Chart,
            vec![
                Regex::new(r"\bchart\b")?,
                Regex::new(r"\bchartjs\b")?,
                Regex::new(r"\bapexcharts\b")?,
            ],
        );
        patterns.insert(
            ComponentType::Calendar,
            vec![
                Regex::new(r"\bcalendar\b")?,
                Regex::new(r"\bfullcalendar\b")?,
                Regex::new(r"\bdatepicker\b")?,
            ],
        );

        Ok(patterns)
    }

    /// Analyze a directory or zip file
    pub fn analyze(&self, path: &Path) -> MigrateResult<AnalysisResult> {
        if path.is_dir() {
            self.analyze_directory(path)
        } else if path.extension().map_or(false, |e| e == "zip") {
            self.analyze_zip(path)
        } else {
            Err(MigrateError::InvalidPath {
                path: path.to_path_buf(),
                reason: "Expected directory or .zip file".into(),
            })
        }
    }

    /// Analyze a directory
    fn analyze_directory(&self, path: &Path) -> MigrateResult<AnalysisResult> {
        let mut result = AnalysisResult::default();
        let mut all_classes = Vec::new();
        let mut html_content = String::new();
        let mut css_content = String::new();

        for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
            let file_path = entry.path();
            if !file_path.is_file() {
                continue;
            }

            let extension = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            let relative_path = file_path
                .strip_prefix(path)
                .unwrap_or(file_path)
                .to_string_lossy()
                .to_string();

            match extension {
                "html" | "htm" => {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            result.files_analyzed.html_files += 1;
                            result.files_analyzed.total_bytes += content.len() as u64;
                            html_content.push_str(&content);

                            let classes = self.extract_html_classes(&content);
                            all_classes.extend(classes);

                            self.detect_components(&content, &relative_path, &mut result);
                        }
                        Err(_) => {
                            result.files_analyzed.parse_errors += 1;
                            result.issues.push(
                                MigrationIssue::warning(
                                    IssueCategory::General,
                                    "Could not read file",
                                )
                                .with_file(&relative_path),
                            );
                        }
                    }
                }
                "css" => {
                    match fs::read_to_string(file_path) {
                        Ok(content) => {
                            result.files_analyzed.css_files += 1;
                            result.files_analyzed.total_bytes += content.len() as u64;
                            css_content.push_str(&content);

                            self.extract_css_variables(&content, &mut result);
                            self.extract_colors(&content, &mut result);
                            self.extract_fonts(&content, &mut result);
                        }
                        Err(_) => {
                            result.files_analyzed.parse_errors += 1;
                        }
                    }
                }
                "js" | "ts" | "jsx" | "tsx" => {
                    result.files_analyzed.js_files += 1;
                }
                _ => {}
            }
        }

        // Detect framework from collected classes
        self.detect_framework(&all_classes, &css_content, &mut result);

        // Build inventory summary
        self.build_inventory(&mut result);

        // Calculate migration confidence
        result.migration_confidence = self.calculate_migration_confidence(&result);

        Ok(result)
    }

    /// Analyze a zip file
    fn analyze_zip(&self, path: &Path) -> MigrateResult<AnalysisResult> {
        let file = fs::File::open(path)?;
        let mut archive = zip::ZipArchive::new(file)?;
        let mut result = AnalysisResult::default();
        let mut all_classes = Vec::new();
        let mut css_content = String::new();

        for i in 0..archive.len() {
            let mut file = archive.by_index(i)?;
            let name = file.name().to_string();

            if file.is_dir() {
                continue;
            }

            let extension = Path::new(&name)
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("");

            match extension {
                "html" | "htm" => {
                    let mut content = String::new();
                    if std::io::Read::read_to_string(&mut file, &mut content).is_ok() {
                        result.files_analyzed.html_files += 1;
                        result.files_analyzed.total_bytes += content.len() as u64;

                        let classes = self.extract_html_classes(&content);
                        all_classes.extend(classes);

                        self.detect_components(&content, &name, &mut result);
                    } else {
                        result.files_analyzed.parse_errors += 1;
                    }
                }
                "css" => {
                    let mut content = String::new();
                    if std::io::Read::read_to_string(&mut file, &mut content).is_ok() {
                        result.files_analyzed.css_files += 1;
                        result.files_analyzed.total_bytes += content.len() as u64;
                        css_content.push_str(&content);

                        self.extract_css_variables(&content, &mut result);
                        self.extract_colors(&content, &mut result);
                        self.extract_fonts(&content, &mut result);
                    } else {
                        result.files_analyzed.parse_errors += 1;
                    }
                }
                "js" | "ts" | "jsx" | "tsx" => {
                    result.files_analyzed.js_files += 1;
                }
                _ => {}
            }
        }

        self.detect_framework(&all_classes, &css_content, &mut result);
        self.build_inventory(&mut result);
        result.migration_confidence = self.calculate_migration_confidence(&result);

        Ok(result)
    }

    /// Extract CSS class names from HTML content
    fn extract_html_classes(&self, html: &str) -> Vec<String> {
        let document = Html::parse_document(html);
        let selector = Selector::parse("[class]").unwrap();
        let mut classes = Vec::new();

        for element in document.select(&selector) {
            if let Some(class_attr) = element.value().attr("class") {
                for class in class_attr.split_whitespace() {
                    classes.push(class.to_string());
                }
            }
        }

        classes
    }

    /// Detect framework from classes and CSS content
    fn detect_framework(
        &self,
        classes: &[String],
        css_content: &str,
        result: &mut AnalysisResult,
    ) {
        let mut bootstrap_score = 0;
        let mut tailwind_score = 0;

        // Check classes against patterns
        for class in classes {
            for pattern in &self.bootstrap_patterns {
                if pattern.is_match(class) {
                    bootstrap_score += 1;
                }
            }
            for pattern in &self.tailwind_patterns {
                if pattern.is_match(class) {
                    tailwind_score += 1;
                }
            }
        }

        // Check CSS content for framework-specific markers
        if css_content.contains("Bootstrap") || css_content.contains("getbootstrap.com") {
            bootstrap_score += 50;

            // Try to detect version
            let version_re = Regex::new(r"Bootstrap\s+v?(\d+(?:\.\d+)*)")
                .expect("Invalid regex");
            if let Some(caps) = version_re.captures(css_content) {
                result.version = FrameworkVersion::parse(caps.get(1).unwrap().as_str());
            }
        }

        if css_content.contains("tailwindcss") || css_content.contains("Tailwind") {
            tailwind_score += 50;

            let version_re = Regex::new(r"tailwindcss\s+v?(\d+(?:\.\d+)*)")
                .expect("Invalid regex");
            if let Some(caps) = version_re.captures(css_content) {
                result.version = FrameworkVersion::parse(caps.get(1).unwrap().as_str());
            }
        }

        // Determine framework based on scores
        let total_score = bootstrap_score + tailwind_score;
        if total_score == 0 {
            result.framework = Framework::Custom;
            result.framework_confidence = 0.5;
            result.issues.push(MigrationIssue::info(
                IssueCategory::FrameworkDetection,
                "No standard CSS framework detected, treating as custom",
            ));
        } else if bootstrap_score > tailwind_score * 2 {
            result.framework = Framework::Bootstrap;
            result.framework_confidence = (bootstrap_score as f32 / total_score as f32).min(0.95);
        } else if tailwind_score > bootstrap_score * 2 {
            result.framework = Framework::Tailwind;
            result.framework_confidence = (tailwind_score as f32 / total_score as f32).min(0.95);
        } else if bootstrap_score > 0 && tailwind_score > 0 {
            result.framework = Framework::Mixed;
            result.framework_confidence = 0.6;
            result.issues.push(MigrationIssue::warning(
                IssueCategory::FrameworkDetection,
                "Multiple CSS frameworks detected",
            ).with_suggestion("Consider consolidating to a single framework during migration"));
        } else {
            result.framework = Framework::Custom;
            result.framework_confidence = 0.5;
        }
    }

    /// Detect components in HTML content
    fn detect_components(
        &self,
        content: &str,
        file_name: &str,
        result: &mut AnalysisResult,
    ) {
        for (component_type, patterns) in &self.component_patterns {
            for pattern in patterns {
                let matches: Vec<_> = pattern.find_iter(content).collect();
                if !matches.is_empty() {
                    // Check if we already have this component type
                    let existing = result
                        .components
                        .iter_mut()
                        .find(|c| c.component_type == *component_type);

                    if let Some(existing) = existing {
                        existing.occurrences += matches.len();
                    } else {
                        let component = DetectedComponent::new(component_type.clone())
                            .with_file(file_name);
                        result.components.push(DetectedComponent {
                            occurrences: matches.len(),
                            ..component
                        });
                    }
                }
            }
        }
    }

    /// Extract CSS custom properties
    fn extract_css_variables(&self, css: &str, result: &mut AnalysisResult) {
        for caps in self.css_var_pattern.captures_iter(css) {
            let name = caps.get(1).map_or("", |m| m.as_str()).to_string();
            let value = caps.get(2).map_or("", |m| m.as_str()).trim().to_string();
            result.css_variables.insert(name, value);
        }
    }

    /// Extract color values from CSS
    fn extract_colors(&self, css: &str, result: &mut AnalysisResult) {
        // Look for color properties
        let color_props = [
            "color",
            "background-color",
            "border-color",
            "fill",
            "stroke",
        ];

        for prop in color_props {
            let pattern = Regex::new(&format!(r"{}:\s*([^;]+);", prop))
                .expect("Invalid regex");
            for caps in pattern.captures_iter(css) {
                let value = caps.get(1).map_or("", |m| m.as_str()).trim();
                // Extract color values
                if value.starts_with('#')
                    || value.starts_with("rgb")
                    || value.starts_with("hsl")
                {
                    if !result.detected_colors.contains(&value.to_string()) {
                        result.detected_colors.push(value.to_string());
                    }
                }
            }
        }
    }

    /// Extract font families from CSS
    fn extract_fonts(&self, css: &str, result: &mut AnalysisResult) {
        for caps in self.font_pattern.captures_iter(css) {
            let fonts = caps.get(1).map_or("", |m| m.as_str()).trim();
            // Split font stack and get individual fonts
            for font in fonts.split(',') {
                let font = font.trim().trim_matches('"').trim_matches('\'').to_string();
                if !font.is_empty() && !result.detected_fonts.contains(&font) {
                    result.detected_fonts.push(font);
                }
            }
        }
    }

    /// Build component inventory summary
    fn build_inventory(&self, result: &mut AnalysisResult) {
        let mut inventory = ComponentInventory::default();
        let mut type_counts: HashMap<String, usize> = HashMap::new();

        for component in &result.components {
            inventory.total_instances += component.occurrences;

            let type_name = format!("{:?}", component.component_type);
            *type_counts.entry(type_name.clone()).or_insert(0) += component.occurrences;

            if component.mappable {
                if component.mapping_confidence > 0.7 {
                    inventory.mappable_count += component.occurrences;
                } else {
                    inventory.manual_count += component.occurrences;
                }
            } else {
                inventory.unmappable_count += component.occurrences;
            }
        }

        inventory.total_types = type_counts.len();
        inventory.by_type = type_counts;
        result.inventory = inventory;
    }

    /// Calculate overall migration confidence
    fn calculate_migration_confidence(&self, result: &AnalysisResult) -> f32 {
        let mut confidence = result.framework_confidence;

        // Adjust based on component mappability
        let total = result.inventory.total_instances.max(1) as f32;
        let mappable_ratio = result.inventory.mappable_count as f32 / total;
        confidence = confidence * 0.5 + mappable_ratio * 0.5;

        // Penalize for errors
        let error_count = result
            .issues
            .iter()
            .filter(|i| i.severity == Severity::Error)
            .count();
        confidence -= (error_count as f32 * 0.05).min(0.3);

        // Bonus for having CSS variables (easier to extract tokens)
        if !result.css_variables.is_empty() {
            confidence += 0.05;
        }

        confidence.clamp(0.0, 1.0)
    }
}

impl Default for Analyzer {
    fn default() -> Self {
        Self::new().expect("Failed to create default analyzer")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_framework_detection_bootstrap() {
        let analyzer = Analyzer::new().unwrap();
        let mut result = AnalysisResult::default();

        let classes = vec![
            "container".to_string(),
            "row".to_string(),
            "col-md-6".to_string(),
            "btn".to_string(),
            "btn-primary".to_string(),
            "card".to_string(),
            "navbar".to_string(),
        ];

        analyzer.detect_framework(&classes, "", &mut result);
        assert_eq!(result.framework, Framework::Bootstrap);
        assert!(result.framework_confidence > 0.5);
    }

    #[test]
    fn test_framework_detection_tailwind() {
        let analyzer = Analyzer::new().unwrap();
        let mut result = AnalysisResult::default();

        let classes = vec![
            "flex".to_string(),
            "p-4".to_string(),
            "m-2".to_string(),
            "bg-blue-500".to_string(),
            "text-white".to_string(),
            "rounded-lg".to_string(),
            "shadow-md".to_string(),
            "hover:bg-blue-600".to_string(),
            "md:flex-row".to_string(),
        ];

        analyzer.detect_framework(&classes, "", &mut result);
        assert_eq!(result.framework, Framework::Tailwind);
        assert!(result.framework_confidence > 0.5);
    }

    #[test]
    fn test_extract_html_classes() {
        let analyzer = Analyzer::new().unwrap();
        let html = r#"
            <div class="container">
                <button class="btn btn-primary">Click me</button>
            </div>
        "#;

        let classes = analyzer.extract_html_classes(html);
        assert!(classes.contains(&"container".to_string()));
        assert!(classes.contains(&"btn".to_string()));
        assert!(classes.contains(&"btn-primary".to_string()));
    }

    #[test]
    fn test_extract_css_variables() {
        let analyzer = Analyzer::new().unwrap();
        let mut result = AnalysisResult::default();

        let css = r#"
            :root {
                --primary-color: #3B82F6;
                --text-color: #1F2937;
                --spacing-md: 16px;
            }
        "#;

        analyzer.extract_css_variables(css, &mut result);
        assert_eq!(
            result.css_variables.get("primary-color"),
            Some(&"#3B82F6".to_string())
        );
        assert_eq!(
            result.css_variables.get("text-color"),
            Some(&"#1F2937".to_string())
        );
    }

    #[test]
    fn test_framework_version_parse() {
        let v1 = FrameworkVersion::parse("5.3.2");
        assert_eq!(v1.major, Some(5));
        assert_eq!(v1.minor, Some(3));
        assert_eq!(v1.patch, Some(2));

        let v2 = FrameworkVersion::parse("v4.6");
        assert_eq!(v2.major, Some(4));
        assert_eq!(v2.minor, Some(6));
        assert_eq!(v2.patch, None);
    }

    #[test]
    fn test_component_detection() {
        let analyzer = Analyzer::new().unwrap();
        let mut result = AnalysisResult::default();

        let html = concat!(
            "<nav class=\"navbar navbar-expand-lg\">",
            "<div class=\"container\">",
            "<a class=\"navbar-brand\" href=\"#\">Brand</a>",
            "</div></nav>",
            "<div class=\"card\"><div class=\"card-body\">",
            "<button class=\"btn btn-primary\">Submit</button>",
            "</div></div>",
            "<table class=\"table table-striped\"><tr><td>Data</td></tr></table>"
        );

        analyzer.detect_components(html, "test.html", &mut result);

        let has_navbar = result.components.iter().any(|c| c.component_type == ComponentType::Navbar);
        let has_card = result.components.iter().any(|c| c.component_type == ComponentType::Card);
        let has_button = result.components.iter().any(|c| c.component_type == ComponentType::Button);
        let has_table = result.components.iter().any(|c| c.component_type == ComponentType::Table);

        assert!(has_navbar);
        assert!(has_card);
        assert!(has_button);
        assert!(has_table);
    }

    #[test]
    fn test_migration_confidence() {
        let analyzer = Analyzer::new().unwrap();
        let mut result = AnalysisResult::default();
        result.framework = Framework::Bootstrap;
        result.framework_confidence = 0.9;
        result.inventory.total_instances = 100;
        result.inventory.mappable_count = 80;
        result.inventory.manual_count = 15;
        result.inventory.unmappable_count = 5;

        let confidence = analyzer.calculate_migration_confidence(&result);
        assert!(confidence > 0.5);
        assert!(confidence <= 1.0);
    }
}
