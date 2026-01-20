//! Incremental compiler for hot reload
//!
//! Efficiently recompiles only changed .oui files and tracks dependencies.

use oxide_compiler::{compile, ComponentIR, CompilerError};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};
use parking_lot::RwLock;
use thiserror::Error;

/// Errors that can occur during incremental compilation
#[derive(Debug, Error)]
pub enum CompileError {
    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("Failed to read file {path}: {source}")]
    ReadError {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },

    #[error("Compilation failed for {path}: {message}")]
    CompilationFailed {
        path: PathBuf,
        message: String,
        line: usize,
        column: usize,
    },

    #[error("Dependency cycle detected: {0:?}")]
    DependencyCycle(Vec<PathBuf>),

    #[error("Invalid component: {0}")]
    InvalidComponent(String),
}

impl From<CompilerError> for CompileError {
    fn from(err: CompilerError) -> Self {
        match err {
            CompilerError::LexerError { line, column, message } => CompileError::CompilationFailed {
                path: PathBuf::new(),
                message,
                line,
                column,
            },
            CompilerError::ParseError { line, column, message } => CompileError::CompilationFailed {
                path: PathBuf::new(),
                message,
                line,
                column,
            },
            CompilerError::InvalidComponent(msg) => CompileError::InvalidComponent(msg),
            CompilerError::IoError(e) => CompileError::ReadError {
                path: PathBuf::new(),
                source: e,
            },
        }
    }
}

/// Result of compiling a file
#[derive(Debug, Clone)]
pub struct CompileResult {
    /// The path that was compiled
    pub path: PathBuf,
    /// The compiled IR
    pub ir: ComponentIR,
    /// Compilation duration
    pub duration: Duration,
    /// Whether this was a cache hit
    pub cached: bool,
    /// Dependencies of this file
    pub dependencies: Vec<PathBuf>,
    /// Components defined in this file
    pub components: Vec<String>,
}

/// Cache entry for a compiled file
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Compiled IR
    ir: ComponentIR,
    /// Source file hash
    source_hash: u64,
    /// File modification time
    mtime: SystemTime,
    /// Dependencies
    dependencies: Vec<PathBuf>,
    /// When this entry was created
    compiled_at: Instant,
}

/// Configuration for the incremental compiler
#[derive(Debug, Clone)]
pub struct CompilerConfig {
    /// Maximum cache size (number of entries)
    pub max_cache_size: usize,
    /// Whether to track dependencies
    pub track_dependencies: bool,
    /// Base directory for resolving imports
    pub base_dir: PathBuf,
    /// Whether to enable parallel compilation
    pub parallel: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            max_cache_size: 100,
            track_dependencies: true,
            base_dir: PathBuf::from("."),
            parallel: true,
        }
    }
}

/// Incremental compiler with caching and dependency tracking
pub struct IncrementalCompiler {
    config: CompilerConfig,
    /// Compilation cache: path -> CacheEntry
    cache: Arc<RwLock<HashMap<PathBuf, CacheEntry>>>,
    /// Dependency graph: path -> set of files that depend on it
    dependents: Arc<RwLock<HashMap<PathBuf, HashSet<PathBuf>>>>,
    /// Reverse dependency graph: path -> set of files it depends on
    dependencies: Arc<RwLock<HashMap<PathBuf, HashSet<PathBuf>>>>,
    /// Component registry: component name -> defining file
    component_registry: Arc<RwLock<HashMap<String, PathBuf>>>,
}

impl IncrementalCompiler {
    /// Create a new incremental compiler with the given configuration
    pub fn new(config: CompilerConfig) -> Self {
        Self {
            config,
            cache: Arc::new(RwLock::new(HashMap::new())),
            dependents: Arc::new(RwLock::new(HashMap::new())),
            dependencies: Arc::new(RwLock::new(HashMap::new())),
            component_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new incremental compiler with default configuration
    pub fn with_defaults() -> Self {
        Self::new(CompilerConfig::default())
    }

    /// Compile a single .oui file
    pub fn compile_file(&self, path: impl AsRef<Path>) -> Result<CompileResult, CompileError> {
        let path = path.as_ref().to_path_buf();

        if !path.exists() {
            return Err(CompileError::FileNotFound(path));
        }

        // Check cache
        if let Some(cached) = self.check_cache(&path)? {
            return Ok(cached);
        }

        // Read and compile
        let start = Instant::now();
        let source = std::fs::read_to_string(&path).map_err(|e| CompileError::ReadError {
            path: path.clone(),
            source: e,
        })?;

        let ir = compile(&source).map_err(|e| {
            let (line, column, message) = match &e {
                CompilerError::LexerError { line, column, message } => (*line, *column, message.clone()),
                CompilerError::ParseError { line, column, message } => (*line, *column, message.clone()),
                _ => (0, 0, e.to_string()),
            };
            CompileError::CompilationFailed {
                path: path.clone(),
                message,
                line,
                column,
            }
        })?;

        let duration = start.elapsed();

        // Extract dependencies and components
        let dependencies = self.extract_dependencies(&ir);
        let components = self.extract_components(&ir);

        // Update cache
        self.update_cache(&path, &source, &ir, &dependencies)?;

        // Update dependency graph
        if self.config.track_dependencies {
            self.update_dependency_graph(&path, &dependencies);
        }

        // Register components
        for component in &components {
            self.component_registry
                .write()
                .insert(component.clone(), path.clone());
        }

        tracing::debug!(
            "Compiled {} in {:?} ({} components)",
            path.display(),
            duration,
            components.len()
        );

        Ok(CompileResult {
            path,
            ir,
            duration,
            cached: false,
            dependencies,
            components,
        })
    }

    /// Compile multiple files, handling dependencies
    pub fn compile_files(&self, paths: &[PathBuf]) -> Vec<Result<CompileResult, CompileError>> {
        // Sort by dependencies (topological sort would be ideal, but simple ordering works for now)
        let mut results = Vec::with_capacity(paths.len());

        for path in paths {
            results.push(self.compile_file(path));
        }

        results
    }

    /// Get all files that need recompilation when a file changes
    pub fn get_invalidated_files(&self, changed_path: &Path) -> Vec<PathBuf> {
        let mut invalidated = vec![changed_path.to_path_buf()];
        let mut visited = HashSet::new();
        visited.insert(changed_path.to_path_buf());

        // BFS to find all dependents
        let mut queue = vec![changed_path.to_path_buf()];
        while let Some(path) = queue.pop() {
            if let Some(deps) = self.dependents.read().get(&path) {
                for dep in deps {
                    if visited.insert(dep.clone()) {
                        invalidated.push(dep.clone());
                        queue.push(dep.clone());
                    }
                }
            }
        }

        invalidated
    }

    /// Invalidate cache for a file and its dependents
    pub fn invalidate(&self, path: &Path) {
        let invalidated = self.get_invalidated_files(path);

        let mut cache = self.cache.write();
        for p in &invalidated {
            cache.remove(p);
        }

        tracing::debug!(
            "Invalidated {} files due to change in {}",
            invalidated.len(),
            path.display()
        );
    }

    /// Clear the entire cache
    pub fn clear_cache(&self) {
        self.cache.write().clear();
        self.dependents.write().clear();
        self.dependencies.write().clear();
        self.component_registry.write().clear();
        tracing::debug!("Cleared compiler cache");
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        let cache = self.cache.read();
        CacheStats {
            entries: cache.len(),
            total_components: self.component_registry.read().len(),
        }
    }

    /// Check if a file is in cache and still valid
    fn check_cache(&self, path: &Path) -> Result<Option<CompileResult>, CompileError> {
        let cache = self.cache.read();

        if let Some(entry) = cache.get(path) {
            // Check if file has been modified
            let metadata = std::fs::metadata(path).map_err(|e| CompileError::ReadError {
                path: path.to_path_buf(),
                source: e,
            })?;

            if let Ok(mtime) = metadata.modified() {
                if mtime == entry.mtime {
                    // Cache hit!
                    return Ok(Some(CompileResult {
                        path: path.to_path_buf(),
                        ir: entry.ir.clone(),
                        duration: Duration::ZERO,
                        cached: true,
                        dependencies: entry.dependencies.clone(),
                        components: self.extract_components(&entry.ir),
                    }));
                }
            }
        }

        Ok(None)
    }

    /// Update the cache with a new compilation result
    fn update_cache(
        &self,
        path: &Path,
        source: &str,
        ir: &ComponentIR,
        dependencies: &[PathBuf],
    ) -> Result<(), CompileError> {
        let metadata = std::fs::metadata(path).map_err(|e| CompileError::ReadError {
            path: path.to_path_buf(),
            source: e,
        })?;

        let mtime = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
        let source_hash = Self::hash_source(source);

        let entry = CacheEntry {
            ir: ir.clone(),
            source_hash,
            mtime,
            dependencies: dependencies.to_vec(),
            compiled_at: Instant::now(),
        };

        let mut cache = self.cache.write();

        // Evict old entries if needed
        if cache.len() >= self.config.max_cache_size {
            // Remove oldest entry
            if let Some(oldest_key) = cache
                .iter()
                .min_by_key(|(_, v)| v.compiled_at)
                .map(|(k, _)| k.clone())
            {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(path.to_path_buf(), entry);
        Ok(())
    }

    /// Update the dependency graph
    fn update_dependency_graph(&self, path: &Path, deps: &[PathBuf]) {
        let path = path.to_path_buf();

        // Update dependencies (what this file depends on)
        self.dependencies
            .write()
            .insert(path.clone(), deps.iter().cloned().collect());

        // Update dependents (what depends on each dependency)
        let mut dependents = self.dependents.write();
        for dep in deps {
            dependents
                .entry(dep.clone())
                .or_default()
                .insert(path.clone());
        }
    }

    /// Extract dependencies from compiled IR
    fn extract_dependencies(&self, ir: &ComponentIR) -> Vec<PathBuf> {
        let mut deps = Vec::new();

        // Look for import statements in the IR (would need to be added to the compiler)
        // For now, we'll check for component references that might be from other files
        self.collect_component_refs(ir, &mut deps);

        deps
    }

    /// Recursively collect component references
    fn collect_component_refs(&self, ir: &ComponentIR, deps: &mut Vec<PathBuf>) {
        // Check if this component type is defined in another file
        if let Some(defining_file) = self.component_registry.read().get(&ir.kind) {
            if !deps.contains(defining_file) {
                deps.push(defining_file.clone());
            }
        }

        // Recurse into children
        for child in &ir.children {
            self.collect_component_refs(child, deps);
        }
    }

    /// Extract component names defined in the IR
    fn extract_components(&self, ir: &ComponentIR) -> Vec<String> {
        // The root component is the main component defined in this file
        vec![ir.kind.clone()]
    }

    /// Simple hash function for source code
    fn hash_source(source: &str) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        source.hash(&mut hasher);
        hasher.finish()
    }
}

impl Default for IncrementalCompiler {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Cache statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub entries: usize,
    pub total_components: usize,
}

/// Batch compilation result
#[derive(Debug)]
pub struct BatchCompileResult {
    /// Successfully compiled files
    pub successes: Vec<CompileResult>,
    /// Files that failed to compile
    pub failures: Vec<(PathBuf, CompileError)>,
    /// Total compilation time
    pub total_duration: Duration,
}

impl BatchCompileResult {
    /// Whether all compilations succeeded
    pub fn all_succeeded(&self) -> bool {
        self.failures.is_empty()
    }

    /// Get the number of files compiled
    pub fn compiled_count(&self) -> usize {
        self.successes.len()
    }

    /// Get the number of failures
    pub fn failure_count(&self) -> usize {
        self.failures.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(content.as_bytes()).unwrap();
        path
    }

    #[test]
    fn test_incremental_compiler_basic() {
        let temp_dir = TempDir::new().unwrap();
        let source = r#"
            app TestApp {
                Text {
                    content: "Hello"
                }
            }
        "#;
        let file_path = create_test_file(temp_dir.path(), "test.oui", source);

        let compiler = IncrementalCompiler::with_defaults();
        let result = compiler.compile_file(&file_path).unwrap();

        assert!(!result.cached);
        assert_eq!(result.ir.kind, "Text");
    }

    #[test]
    fn test_cache_hit() {
        let temp_dir = TempDir::new().unwrap();
        let source = r#"
            app TestApp {
                Column {
                    Text { content: "A" }
                }
            }
        "#;
        let file_path = create_test_file(temp_dir.path(), "test.oui", source);

        let compiler = IncrementalCompiler::with_defaults();

        // First compile
        let result1 = compiler.compile_file(&file_path).unwrap();
        assert!(!result1.cached);

        // Second compile should hit cache
        let result2 = compiler.compile_file(&file_path).unwrap();
        assert!(result2.cached);
    }

    #[test]
    fn test_invalidation() {
        let temp_dir = TempDir::new().unwrap();
        let source = r#"
            app TestApp {
                Text { content: "Test" }
            }
        "#;
        let file_path = create_test_file(temp_dir.path(), "test.oui", source);

        let compiler = IncrementalCompiler::with_defaults();

        // Compile
        let _ = compiler.compile_file(&file_path).unwrap();

        // Invalidate
        compiler.invalidate(&file_path);

        // Should not be cached anymore
        let stats = compiler.cache_stats();
        assert_eq!(stats.entries, 0);
    }
}
