//! Migration Helpers for Electron and Tauri Projects
//!
//! This module provides tools to analyze and migrate applications from
//! Electron and Tauri to OxideKit.
//!
//! # Migration Process
//!
//! 1. **Analysis**: Scan the source project to understand its structure
//! 2. **Planning**: Generate a migration plan with steps and estimates
//! 3. **Execution**: Apply migration transformations
//! 4. **Verification**: Validate the migrated project
//!
//! # Supported Sources
//!
//! - Electron (JavaScript/TypeScript)
//! - Tauri (Rust + Web frontend)

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use walkdir::WalkDir;

/// Errors in migration operations
#[derive(Error, Debug)]
pub enum MigrationError {
    /// Source project not found
    #[error("Source project not found: {0}")]
    SourceNotFound(PathBuf),

    /// Unsupported source type
    #[error("Unsupported source type: {0}")]
    UnsupportedSource(String),

    /// Analysis failed
    #[error("Analysis failed: {0}")]
    AnalysisFailed(String),

    /// Migration step failed
    /// Migration step failure
    #[error("Migration step failed: {step}: {reason}")]
    StepFailed {
        /// The step that failed
        step: String,
        /// The reason for failure
        reason: String,
    },

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),

    /// TOML error
    #[error("TOML error: {0}")]
    TomlError(#[from] toml::de::Error),
}

/// Source framework for migration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MigrationSource {
    /// Electron application
    Electron,
    /// Tauri application
    Tauri,
    /// Unknown/other
    Unknown,
}

impl MigrationSource {
    /// Detect source type from project directory
    pub fn detect<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        // Check for Electron indicators
        if path.join("package.json").exists() {
            if let Ok(content) = std::fs::read_to_string(path.join("package.json")) {
                if content.contains("electron") {
                    return MigrationSource::Electron;
                }
            }
        }

        // Check for Tauri indicators
        if path.join("src-tauri").exists() || path.join("tauri.conf.json").exists() {
            return MigrationSource::Tauri;
        }

        if path.join("Cargo.toml").exists() {
            if let Ok(content) = std::fs::read_to_string(path.join("Cargo.toml")) {
                if content.contains("tauri") {
                    return MigrationSource::Tauri;
                }
            }
        }

        MigrationSource::Unknown
    }
}

/// Analysis result for a project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectAnalysis {
    /// Source framework
    pub source: MigrationSource,
    /// Project name
    pub name: String,
    /// Project version
    pub version: Option<String>,
    /// Total file count
    pub file_count: usize,
    /// Lines of code by language
    pub lines_of_code: HashMap<String, usize>,
    /// Dependencies
    pub dependencies: Vec<Dependency>,
    /// UI framework detected
    pub ui_framework: Option<String>,
    /// IPC calls found
    pub ipc_calls: Vec<IpcCall>,
    /// Native APIs used
    pub native_apis: Vec<NativeApi>,
    /// Complexity score (0-100)
    pub complexity_score: u32,
    /// Estimated migration effort (hours)
    pub estimated_hours: u32,
    /// Warnings
    pub warnings: Vec<String>,
    /// Blocking issues
    pub blockers: Vec<String>,
}

/// A project dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dependency {
    /// Package name
    pub name: String,
    /// Version
    pub version: String,
    /// Whether it's a dev dependency
    pub dev: bool,
    /// OxideKit equivalent (if known)
    pub oxide_equivalent: Option<String>,
    /// Migration notes
    pub migration_note: Option<String>,
}

/// An IPC call found in the project
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IpcCall {
    /// Call name/type
    pub name: String,
    /// File where it was found
    pub file: PathBuf,
    /// Line number
    pub line: usize,
    /// Call direction (main-to-renderer, renderer-to-main)
    pub direction: IpcDirection,
    /// Migration complexity
    pub complexity: Complexity,
}

/// IPC call direction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IpcDirection {
    /// Main process to renderer
    MainToRenderer,
    /// Renderer to main process
    RendererToMain,
    /// Bidirectional
    Bidirectional,
}

/// A native API usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NativeApi {
    /// API name
    pub name: String,
    /// Files using this API
    pub files: Vec<PathBuf>,
    /// Usage count
    pub usage_count: usize,
    /// OxideKit equivalent
    pub oxide_equivalent: Option<String>,
    /// Migration notes
    pub migration_note: Option<String>,
    /// Complexity to migrate
    pub complexity: Complexity,
}

/// Complexity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Complexity {
    /// Simple, straightforward migration
    Low,
    /// Moderate complexity
    Medium,
    /// Complex, may require significant changes
    High,
    /// Very difficult or potentially impossible
    Critical,
}

/// Migration plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationPlan {
    /// Source project analysis
    pub analysis: ProjectAnalysis,
    /// Ordered list of migration steps
    pub steps: Vec<MigrationStep>,
    /// Total estimated hours
    pub total_hours: u32,
    /// Recommended approach
    pub approach: MigrationApproach,
    /// Risk assessment
    pub risk_level: RiskLevel,
    /// Prerequisites
    pub prerequisites: Vec<String>,
}

/// A migration step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MigrationStep {
    /// Step ID
    pub id: u32,
    /// Step name
    pub name: String,
    /// Description
    pub description: String,
    /// Estimated hours
    pub hours: u32,
    /// Status
    pub status: StepStatus,
    /// Files affected
    pub files: Vec<PathBuf>,
    /// Commands to run
    pub commands: Vec<String>,
    /// Transformations to apply
    pub transformations: Vec<Transformation>,
    /// Dependencies on other steps
    pub depends_on: Vec<u32>,
    /// Verification checks
    pub verification: Vec<String>,
}

/// Step status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StepStatus {
    /// Not started
    Pending,
    /// In progress
    InProgress,
    /// Completed successfully
    Completed,
    /// Skipped
    Skipped,
    /// Failed
    Failed,
}

/// Code transformation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transformation {
    /// Transformation type
    pub kind: TransformKind,
    /// Pattern to match
    pub pattern: String,
    /// Replacement
    pub replacement: String,
    /// Files to apply to
    pub files: Option<Vec<PathBuf>>,
}

/// Transformation type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TransformKind {
    /// Replace text
    Replace,
    /// Regular expression replacement
    Regex,
    /// Delete code
    Delete,
    /// Insert code
    Insert,
    /// Rename file
    Rename,
    /// Move file
    Move,
    /// Custom transformation
    Custom,
}

/// Migration approach
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MigrationApproach {
    /// Full rewrite recommended
    FullRewrite,
    /// Incremental migration possible
    Incremental,
    /// Use compatibility layer temporarily
    CompatLayer,
    /// Hybrid approach
    Hybrid,
}

/// Risk level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Migration analyzer
pub struct MigrationAnalyzer;

impl MigrationAnalyzer {
    /// Create a new analyzer
    pub fn new() -> Self {
        Self
    }

    /// Analyze a project
    pub fn analyze<P: AsRef<Path>>(&self, path: P) -> Result<ProjectAnalysis, MigrationError> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(MigrationError::SourceNotFound(path.to_path_buf()));
        }

        let source = MigrationSource::detect(path);

        match source {
            MigrationSource::Electron => self.analyze_electron(path),
            MigrationSource::Tauri => self.analyze_tauri(path),
            MigrationSource::Unknown => Err(MigrationError::UnsupportedSource(
                "Could not detect project type".to_string(),
            )),
        }
    }

    /// Analyze an Electron project
    fn analyze_electron(&self, path: &Path) -> Result<ProjectAnalysis, MigrationError> {
        let mut analysis = ProjectAnalysis {
            source: MigrationSource::Electron,
            name: String::new(),
            version: None,
            file_count: 0,
            lines_of_code: HashMap::new(),
            dependencies: Vec::new(),
            ui_framework: None,
            ipc_calls: Vec::new(),
            native_apis: Vec::new(),
            complexity_score: 0,
            estimated_hours: 0,
            warnings: Vec::new(),
            blockers: Vec::new(),
        };

        // Read package.json
        let package_json_path = path.join("package.json");
        if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path)?;
            let package: serde_json::Value = serde_json::from_str(&content)?;

            analysis.name = package
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            analysis.version = package.get("version").and_then(|v| v.as_str()).map(String::from);

            // Extract dependencies
            if let Some(deps) = package.get("dependencies").and_then(|v| v.as_object()) {
                for (name, version) in deps {
                    let version_str = version.as_str().unwrap_or("*").to_string();
                    analysis.dependencies.push(Dependency {
                        name: name.clone(),
                        version: version_str,
                        dev: false,
                        oxide_equivalent: self.find_oxide_equivalent(name),
                        migration_note: self.get_migration_note(name),
                    });
                }
            }

            // Detect UI framework
            analysis.ui_framework = self.detect_ui_framework(&analysis.dependencies);
        }

        // Count files and lines
        self.count_files_and_lines(path, &mut analysis);

        // Scan for IPC calls
        self.scan_electron_ipc(path, &mut analysis)?;

        // Scan for native APIs
        self.scan_electron_native_apis(path, &mut analysis)?;

        // Calculate complexity and estimate
        analysis.complexity_score = self.calculate_complexity(&analysis);
        analysis.estimated_hours = self.estimate_hours(&analysis);

        Ok(analysis)
    }

    /// Analyze a Tauri project
    fn analyze_tauri(&self, path: &Path) -> Result<ProjectAnalysis, MigrationError> {
        let mut analysis = ProjectAnalysis {
            source: MigrationSource::Tauri,
            name: String::new(),
            version: None,
            file_count: 0,
            lines_of_code: HashMap::new(),
            dependencies: Vec::new(),
            ui_framework: None,
            ipc_calls: Vec::new(),
            native_apis: Vec::new(),
            complexity_score: 0,
            estimated_hours: 0,
            warnings: Vec::new(),
            blockers: Vec::new(),
        };

        // Read tauri.conf.json or Cargo.toml
        let tauri_conf_path = path.join("src-tauri/tauri.conf.json");
        let cargo_toml_path = path.join("src-tauri/Cargo.toml");

        if tauri_conf_path.exists() {
            let content = std::fs::read_to_string(&tauri_conf_path)?;
            let config: serde_json::Value = serde_json::from_str(&content)?;

            analysis.name = config
                .get("package")
                .and_then(|p| p.get("productName"))
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string();

            analysis.version = config
                .get("package")
                .and_then(|p| p.get("version"))
                .and_then(|v| v.as_str())
                .map(String::from);
        }

        if cargo_toml_path.exists() {
            let content = std::fs::read_to_string(&cargo_toml_path)?;
            // Parse dependencies from Cargo.toml
            if let Ok(cargo) = toml::from_str::<toml::Value>(&content) {
                if let Some(deps) = cargo.get("dependencies").and_then(|v| v.as_table()) {
                    for (name, value) in deps {
                        let version = match value {
                            toml::Value::String(s) => s.clone(),
                            toml::Value::Table(t) => t
                                .get("version")
                                .and_then(|v| v.as_str())
                                .unwrap_or("*")
                                .to_string(),
                            _ => "*".to_string(),
                        };
                        analysis.dependencies.push(Dependency {
                            name: name.clone(),
                            version,
                            dev: false,
                            oxide_equivalent: self.find_rust_oxide_equivalent(name),
                            migration_note: None,
                        });
                    }
                }
            }
        }

        // Also check for frontend dependencies
        let package_json_path = path.join("package.json");
        if package_json_path.exists() {
            let content = std::fs::read_to_string(&package_json_path)?;
            if let Ok(package) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(deps) = package.get("dependencies").and_then(|v| v.as_object()) {
                    for (name, version) in deps {
                        analysis.dependencies.push(Dependency {
                            name: name.clone(),
                            version: version.as_str().unwrap_or("*").to_string(),
                            dev: false,
                            oxide_equivalent: self.find_oxide_equivalent(name),
                            migration_note: None,
                        });
                    }
                }
            }

            // Detect UI framework
            analysis.ui_framework = self.detect_ui_framework(&analysis.dependencies);
        }

        // Count files and lines
        self.count_files_and_lines(path, &mut analysis);

        // Scan for Tauri commands
        self.scan_tauri_commands(path, &mut analysis)?;

        // Calculate complexity and estimate
        analysis.complexity_score = self.calculate_complexity(&analysis);
        analysis.estimated_hours = self.estimate_hours(&analysis);

        // Note that Tauri is already Rust-based, so migration is simpler
        analysis.warnings.push(
            "Tauri is already Rust-based. Focus on UI migration and Tauri-specific API replacement."
                .to_string(),
        );

        Ok(analysis)
    }

    /// Count files and lines of code
    fn count_files_and_lines(&self, path: &Path, analysis: &mut ProjectAnalysis) {
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // Skip node_modules and target directories
            if file_path
                .components()
                .any(|c| c.as_os_str() == "node_modules" || c.as_os_str() == "target")
            {
                continue;
            }

            analysis.file_count += 1;

            // Count lines by extension
            if let Some(ext) = file_path.extension().and_then(|e| e.to_str()) {
                let lang = match ext {
                    "js" | "jsx" | "mjs" => "javascript",
                    "ts" | "tsx" => "typescript",
                    "rs" => "rust",
                    "html" | "htm" => "html",
                    "css" | "scss" | "sass" => "css",
                    "vue" => "vue",
                    "svelte" => "svelte",
                    _ => continue,
                };

                if let Ok(content) = std::fs::read_to_string(file_path) {
                    let lines = content.lines().count();
                    *analysis.lines_of_code.entry(lang.to_string()).or_insert(0) += lines;
                }
            }
        }
    }

    /// Scan for Electron IPC calls
    fn scan_electron_ipc(&self, path: &Path, analysis: &mut ProjectAnalysis) -> Result<(), MigrationError> {
        let ipc_patterns = [
            ("ipcMain.on", IpcDirection::RendererToMain),
            ("ipcMain.handle", IpcDirection::RendererToMain),
            ("ipcRenderer.send", IpcDirection::RendererToMain),
            ("ipcRenderer.invoke", IpcDirection::RendererToMain),
            ("webContents.send", IpcDirection::MainToRenderer),
        ];

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            // Skip non-JS/TS files
            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !["js", "ts", "jsx", "tsx", "mjs"].contains(&ext) {
                continue;
            }

            // Skip node_modules
            if file_path.components().any(|c| c.as_os_str() == "node_modules") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (line_num, line) in content.lines().enumerate() {
                    for (pattern, direction) in &ipc_patterns {
                        if line.contains(pattern) {
                            analysis.ipc_calls.push(IpcCall {
                                name: pattern.to_string(),
                                file: file_path.to_path_buf(),
                                line: line_num + 1,
                                direction: *direction,
                                complexity: Complexity::Medium,
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Scan for Electron native APIs
    fn scan_electron_native_apis(&self, path: &Path, analysis: &mut ProjectAnalysis) -> Result<(), MigrationError> {
        let native_apis = [
            ("dialog", "ui.dialog", "Use oxide-ui dialog components"),
            ("Menu", "ui.menu", "Use oxide-ui menu components"),
            ("Tray", "native.tray", "Use oxide-native tray API"),
            ("Notification", "native.notification", "Use oxide-native notification API"),
            ("clipboard", "native.clipboard", "Use oxide-native clipboard API"),
            ("shell", "native.shell", "Use oxide-native shell API"),
            ("nativeTheme", "native.theme", "Use oxide-native theme API"),
            ("powerMonitor", "native.power", "Use oxide-native power API"),
            ("screen", "native.screen", "Use oxide-native screen API"),
        ];

        let mut api_usage: HashMap<String, Vec<PathBuf>> = HashMap::new();

        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
        {
            let file_path = entry.path();

            let ext = file_path.extension().and_then(|e| e.to_str()).unwrap_or("");
            if !["js", "ts", "jsx", "tsx", "mjs"].contains(&ext) {
                continue;
            }

            if file_path.components().any(|c| c.as_os_str() == "node_modules") {
                continue;
            }

            if let Ok(content) = std::fs::read_to_string(file_path) {
                for (api, _, _) in &native_apis {
                    if content.contains(api) {
                        api_usage
                            .entry(api.to_string())
                            .or_default()
                            .push(file_path.to_path_buf());
                    }
                }
            }
        }

        for (api, equivalent, note) in native_apis {
            if let Some(files) = api_usage.get(api) {
                analysis.native_apis.push(NativeApi {
                    name: api.to_string(),
                    files: files.clone(),
                    usage_count: files.len(),
                    oxide_equivalent: Some(equivalent.to_string()),
                    migration_note: Some(note.to_string()),
                    complexity: Complexity::Medium,
                });
            }
        }

        Ok(())
    }

    /// Scan for Tauri commands
    fn scan_tauri_commands(&self, path: &Path, analysis: &mut ProjectAnalysis) -> Result<(), MigrationError> {
        let src_tauri = path.join("src-tauri/src");

        if src_tauri.exists() {
            for entry in WalkDir::new(&src_tauri)
                .into_iter()
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_file())
            {
                let file_path = entry.path();

                if file_path.extension().and_then(|e| e.to_str()) != Some("rs") {
                    continue;
                }

                if let Ok(content) = std::fs::read_to_string(file_path) {
                    for (line_num, line) in content.lines().enumerate() {
                        if line.contains("#[tauri::command]") {
                            // Look for the function name on the next non-empty line
                            analysis.ipc_calls.push(IpcCall {
                                name: "tauri::command".to_string(),
                                file: file_path.to_path_buf(),
                                line: line_num + 1,
                                direction: IpcDirection::Bidirectional,
                                complexity: Complexity::Low, // Tauri commands are easier to migrate
                            });
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Find OxideKit equivalent for an npm package
    fn find_oxide_equivalent(&self, package: &str) -> Option<String> {
        match package {
            "react" | "vue" | "svelte" => Some("oxide-ui".to_string()),
            "electron" => Some("oxide-native".to_string()),
            "@tauri-apps/api" => Some("oxide-native".to_string()),
            "axios" | "fetch" => Some("oxide-net".to_string()),
            "lodash" | "underscore" => None, // Use native Rust
            "dayjs" | "moment" => Some("chrono (Rust)".to_string()),
            _ => None,
        }
    }

    /// Find OxideKit equivalent for a Rust crate
    fn find_rust_oxide_equivalent(&self, crate_name: &str) -> Option<String> {
        match crate_name {
            "tauri" => Some("oxide-runtime".to_string()),
            "tauri-build" => Some("oxide-cli (build)".to_string()),
            _ => None, // Most Rust crates are compatible
        }
    }

    /// Get migration note for a package
    fn get_migration_note(&self, package: &str) -> Option<String> {
        match package {
            "electron" => Some("Core dependency - will be replaced by oxide-runtime".to_string()),
            "react" | "vue" | "svelte" => Some(
                "UI framework - migrate to OxideKit native UI or use compat.webview temporarily"
                    .to_string(),
            ),
            "@tauri-apps/api" => Some("Replace with oxide-native APIs".to_string()),
            _ => None,
        }
    }

    /// Detect UI framework from dependencies
    fn detect_ui_framework(&self, dependencies: &[Dependency]) -> Option<String> {
        for dep in dependencies {
            match dep.name.as_str() {
                "react" | "react-dom" => return Some("React".to_string()),
                "vue" => return Some("Vue".to_string()),
                "svelte" => return Some("Svelte".to_string()),
                "@angular/core" => return Some("Angular".to_string()),
                "solid-js" => return Some("Solid".to_string()),
                _ => {}
            }
        }
        None
    }

    /// Calculate complexity score
    fn calculate_complexity(&self, analysis: &ProjectAnalysis) -> u32 {
        let mut score = 0u32;

        // Base score from file count
        score += (analysis.file_count / 10).min(20) as u32;

        // Add for total LOC
        let total_loc: usize = analysis.lines_of_code.values().sum();
        score += (total_loc / 1000).min(20) as u32;

        // Add for IPC complexity
        score += (analysis.ipc_calls.len() * 2).min(20) as u32;

        // Add for native API usage
        score += (analysis.native_apis.len() * 3).min(20) as u32;

        // Add for dependency count
        score += (analysis.dependencies.len() / 5).min(10) as u32;

        // Add extra for complex UI frameworks
        if let Some(ref framework) = analysis.ui_framework {
            match framework.as_str() {
                "React" | "Vue" | "Angular" => score += 10,
                "Svelte" => score += 5,
                _ => score += 3,
            }
        }

        score.min(100)
    }

    /// Estimate migration hours
    fn estimate_hours(&self, analysis: &ProjectAnalysis) -> u32 {
        let base_hours = match analysis.source {
            MigrationSource::Tauri => 8, // Tauri is easier since it's already Rust
            MigrationSource::Electron => 16,
            MigrationSource::Unknown => 40,
        };

        let complexity_factor = analysis.complexity_score as f32 / 50.0;

        (base_hours as f32 * (1.0 + complexity_factor)).ceil() as u32
    }
}

impl Default for MigrationAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Migration plan generator
pub struct MigrationPlanner;

impl MigrationPlanner {
    /// Create a new planner
    pub fn new() -> Self {
        Self
    }

    /// Generate a migration plan from analysis
    pub fn plan(&self, analysis: ProjectAnalysis) -> MigrationPlan {
        let mut steps = Vec::new();
        let mut step_id = 1;

        // Step 1: Project setup
        steps.push(MigrationStep {
            id: step_id,
            name: "Initialize OxideKit project".to_string(),
            description: "Create new OxideKit project structure".to_string(),
            hours: 1,
            status: StepStatus::Pending,
            files: Vec::new(),
            commands: vec![
                format!("oxide new {} --template migration", analysis.name),
            ],
            transformations: Vec::new(),
            depends_on: Vec::new(),
            verification: vec!["oxide doctor".to_string()],
        });
        step_id += 1;

        // Step 2: Migrate configuration
        steps.push(MigrationStep {
            id: step_id,
            name: "Migrate configuration".to_string(),
            description: "Convert project configuration to oxide.toml".to_string(),
            hours: 2,
            status: StepStatus::Pending,
            files: vec![PathBuf::from("oxide.toml")],
            commands: Vec::new(),
            transformations: Vec::new(),
            depends_on: vec![1],
            verification: vec!["oxide lint".to_string()],
        });
        step_id += 1;

        // Step 3: Migrate IPC (if any)
        if !analysis.ipc_calls.is_empty() {
            steps.push(MigrationStep {
                id: step_id,
                name: "Migrate IPC calls".to_string(),
                description: format!(
                    "Convert {} IPC calls to OxideKit native bindings",
                    analysis.ipc_calls.len()
                ),
                hours: (analysis.ipc_calls.len() as u32 / 2).max(2),
                status: StepStatus::Pending,
                files: analysis
                    .ipc_calls
                    .iter()
                    .map(|c| c.file.clone())
                    .collect(),
                commands: Vec::new(),
                transformations: Vec::new(),
                depends_on: vec![2],
                verification: vec!["cargo build".to_string()],
            });
            step_id += 1;
        }

        // Step 4: Migrate native APIs
        if !analysis.native_apis.is_empty() {
            steps.push(MigrationStep {
                id: step_id,
                name: "Migrate native APIs".to_string(),
                description: format!(
                    "Replace {} native API usages with OxideKit equivalents",
                    analysis.native_apis.len()
                ),
                hours: (analysis.native_apis.len() as u32 * 2).max(4),
                status: StepStatus::Pending,
                files: analysis
                    .native_apis
                    .iter()
                    .flat_map(|a| a.files.clone())
                    .collect(),
                commands: Vec::new(),
                transformations: Vec::new(),
                depends_on: vec![step_id - 1],
                verification: vec!["oxide build".to_string()],
            });
            step_id += 1;
        }

        // Step 5: Migrate UI
        if analysis.ui_framework.is_some() {
            steps.push(MigrationStep {
                id: step_id,
                name: "Migrate UI layer".to_string(),
                description: format!(
                    "Migrate {} UI to OxideKit native components or compat.webview",
                    analysis.ui_framework.as_deref().unwrap_or("unknown")
                ),
                hours: analysis.estimated_hours / 2,
                status: StepStatus::Pending,
                files: Vec::new(),
                commands: Vec::new(),
                transformations: Vec::new(),
                depends_on: vec![step_id - 1],
                verification: vec!["oxide dev".to_string()],
            });
            step_id += 1;
        }

        // Step 6: Testing
        steps.push(MigrationStep {
            id: step_id,
            name: "Testing and verification".to_string(),
            description: "Run tests and verify functionality".to_string(),
            hours: 4,
            status: StepStatus::Pending,
            files: Vec::new(),
            commands: vec![
                "oxide test".to_string(),
                "oxide build --release".to_string(),
            ],
            transformations: Vec::new(),
            depends_on: vec![step_id - 1],
            verification: vec![
                "All tests pass".to_string(),
                "Release build succeeds".to_string(),
            ],
        });

        let total_hours: u32 = steps.iter().map(|s| s.hours).sum();

        let approach = if analysis.complexity_score > 70 {
            MigrationApproach::FullRewrite
        } else if analysis.complexity_score > 40 {
            MigrationApproach::Hybrid
        } else {
            MigrationApproach::Incremental
        };

        let risk_level = match analysis.complexity_score {
            0..=30 => RiskLevel::Low,
            31..=50 => RiskLevel::Medium,
            51..=75 => RiskLevel::High,
            _ => RiskLevel::Critical,
        };

        let prerequisites = vec![
            "Rust toolchain installed".to_string(),
            "OxideKit CLI installed (cargo install oxide-cli)".to_string(),
            "Backup of original project".to_string(),
        ];

        MigrationPlan {
            analysis,
            steps,
            total_hours,
            approach,
            risk_level,
            prerequisites,
        }
    }
}

impl Default for MigrationPlanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_migration_source_detection() {
        // This would need actual test directories to work properly
        let source = MigrationSource::detect("nonexistent");
        assert_eq!(source, MigrationSource::Unknown);
    }

    #[test]
    fn test_complexity_calculation() {
        let analyzer = MigrationAnalyzer::new();
        let analysis = ProjectAnalysis {
            source: MigrationSource::Electron,
            name: "test".to_string(),
            version: None,
            file_count: 50,
            lines_of_code: [("javascript".to_string(), 5000)].into_iter().collect(),
            dependencies: Vec::new(),
            ui_framework: Some("React".to_string()),
            ipc_calls: Vec::new(),
            native_apis: Vec::new(),
            complexity_score: 0,
            estimated_hours: 0,
            warnings: Vec::new(),
            blockers: Vec::new(),
        };

        let score = analyzer.calculate_complexity(&analysis);
        assert!(score > 0);
        assert!(score <= 100);
    }

    #[test]
    fn test_migration_plan_generation() {
        let analysis = ProjectAnalysis {
            source: MigrationSource::Electron,
            name: "test-app".to_string(),
            version: Some("1.0.0".to_string()),
            file_count: 100,
            lines_of_code: HashMap::new(),
            dependencies: Vec::new(),
            ui_framework: Some("React".to_string()),
            ipc_calls: vec![IpcCall {
                name: "ipcMain.on".to_string(),
                file: PathBuf::from("main.js"),
                line: 10,
                direction: IpcDirection::RendererToMain,
                complexity: Complexity::Medium,
            }],
            native_apis: Vec::new(),
            complexity_score: 50,
            estimated_hours: 24,
            warnings: Vec::new(),
            blockers: Vec::new(),
        };

        let planner = MigrationPlanner::new();
        let plan = planner.plan(analysis);

        assert!(!plan.steps.is_empty());
        assert!(plan.total_hours > 0);
    }
}
