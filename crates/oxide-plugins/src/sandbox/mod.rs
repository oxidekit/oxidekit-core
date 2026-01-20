//! WASM sandbox for community plugins.
//!
//! Community plugins run in a WASM sandbox to prevent malicious code from
//! accessing system resources. This module provides:
//!
//! - WASM runtime configuration
//! - Capability-based access control
//! - Resource limits
//!
//! # Feature Gate
//!
//! This module requires the `wasm-sandbox` feature:
//!
//! ```toml
//! oxide-plugins = { version = "0.1", features = ["wasm-sandbox"] }
//! ```

use serde::{Deserialize, Serialize};
use crate::permissions::Capability;

/// Configuration for the WASM sandbox.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxConfig {
    /// Maximum memory in bytes (default: 256MB).
    pub max_memory: u64,
    /// Maximum execution time in milliseconds (default: 5000).
    pub max_execution_time_ms: u64,
    /// Maximum stack size in bytes (default: 1MB).
    pub max_stack_size: u64,
    /// Allowed capabilities.
    pub allowed_capabilities: Vec<Capability>,
    /// Whether to allow WASI filesystem access.
    pub allow_wasi_fs: bool,
    /// Allowed filesystem paths (if WASI fs is enabled).
    pub allowed_paths: Vec<String>,
    /// Whether to allow WASI network access.
    pub allow_wasi_net: bool,
    /// Allowed network hosts (if WASI net is enabled).
    pub allowed_hosts: Vec<String>,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_memory: 256 * 1024 * 1024, // 256MB
            max_execution_time_ms: 5000,
            max_stack_size: 1024 * 1024, // 1MB
            allowed_capabilities: Vec::new(),
            allow_wasi_fs: false,
            allowed_paths: Vec::new(),
            allow_wasi_net: false,
            allowed_hosts: Vec::new(),
        }
    }
}

impl SandboxConfig {
    /// Create a minimal sandbox with no capabilities.
    pub fn minimal() -> Self {
        Self {
            max_memory: 64 * 1024 * 1024, // 64MB
            max_execution_time_ms: 1000,
            max_stack_size: 512 * 1024, // 512KB
            ..Default::default()
        }
    }

    /// Create a sandbox with standard capabilities.
    pub fn standard() -> Self {
        Self::default()
    }

    /// Create a sandbox with extended capabilities (for verified plugins).
    pub fn extended() -> Self {
        Self {
            max_memory: 512 * 1024 * 1024, // 512MB
            max_execution_time_ms: 30000,
            max_stack_size: 2 * 1024 * 1024, // 2MB
            allowed_capabilities: vec![
                Capability::StorageLocal,
                Capability::StorageSession,
            ],
            allow_wasi_fs: false,
            allowed_paths: Vec::new(),
            allow_wasi_net: false,
            allowed_hosts: Vec::new(),
        }
    }

    /// Add a capability to the sandbox.
    pub fn with_capability(mut self, capability: Capability) -> Self {
        self.allowed_capabilities.push(capability);
        self
    }

    /// Allow filesystem access to specific paths.
    pub fn with_fs_access(mut self, paths: Vec<String>) -> Self {
        self.allow_wasi_fs = true;
        self.allowed_paths = paths;
        self
    }

    /// Allow network access to specific hosts.
    pub fn with_net_access(mut self, hosts: Vec<String>) -> Self {
        self.allow_wasi_net = true;
        self.allowed_hosts = hosts;
        self
    }

    /// Check if a capability is allowed.
    pub fn is_capability_allowed(&self, capability: &Capability) -> bool {
        self.allowed_capabilities.contains(capability)
    }
}

/// WASM plugin sandbox runtime.
///
/// This is only available with the `wasm-sandbox` feature.
#[cfg(feature = "wasm-sandbox")]
pub mod runtime {
    use super::*;
    use crate::error::{PluginError, PluginResult};
    use std::path::Path;
    use wasmtime::*;

    /// A sandboxed WASM plugin instance.
    pub struct SandboxedPlugin {
        /// The WASM engine.
        engine: Engine,
        /// The compiled module.
        module: Module,
        /// Sandbox configuration.
        config: SandboxConfig,
    }

    impl SandboxedPlugin {
        /// Create a new sandboxed plugin from a WASM file.
        pub fn from_file<P: AsRef<Path>>(
            path: P,
            config: SandboxConfig,
        ) -> PluginResult<Self> {
            let mut engine_config = Config::new();

            // Configure memory limits
            engine_config.max_wasm_stack(config.max_stack_size as usize);

            let engine = Engine::new(&engine_config)
                .map_err(|e| PluginError::WasmCompilationError(e.to_string()))?;

            let module = Module::from_file(&engine, path)
                .map_err(|e| PluginError::WasmCompilationError(e.to_string()))?;

            Ok(Self {
                engine,
                module,
                config,
            })
        }

        /// Create a new sandboxed plugin from WASM bytes.
        pub fn from_bytes(
            bytes: &[u8],
            config: SandboxConfig,
        ) -> PluginResult<Self> {
            let mut engine_config = Config::new();
            engine_config.max_wasm_stack(config.max_stack_size as usize);

            let engine = Engine::new(&engine_config)
                .map_err(|e| PluginError::WasmCompilationError(e.to_string()))?;

            let module = Module::new(&engine, bytes)
                .map_err(|e| PluginError::WasmCompilationError(e.to_string()))?;

            Ok(Self {
                engine,
                module,
                config,
            })
        }

        /// Get the sandbox configuration.
        pub fn config(&self) -> &SandboxConfig {
            &self.config
        }

        /// Call a function in the sandboxed plugin.
        pub fn call<T>(&self, _func_name: &str, _args: &[Val]) -> PluginResult<T>
        where
            T: Default,
        {
            // In a full implementation, this would:
            // 1. Create a store with resource limits
            // 2. Set up WASI if needed
            // 3. Create an instance
            // 4. Get the function
            // 5. Call the function with timeout
            // 6. Return the result

            // For now, return a placeholder
            Err(PluginError::SandboxError(
                "WASM function calls not yet implemented".to_string()
            ))
        }
    }
}

/// Host functions that can be provided to sandboxed plugins.
#[derive(Debug, Clone, Default)]
pub struct HostFunctions {
    /// Registered functions.
    functions: Vec<HostFunction>,
}

/// A host function that can be called from WASM.
#[derive(Debug, Clone)]
pub struct HostFunction {
    /// Function name.
    pub name: String,
    /// Module name (namespace).
    pub module: String,
    /// Required capability to call this function.
    pub required_capability: Option<Capability>,
    /// Function signature description.
    pub signature: String,
}

impl HostFunctions {
    /// Create a new empty set of host functions.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add the standard OxideKit host functions.
    pub fn with_standard_functions(mut self) -> Self {
        // Console functions
        self.functions.push(HostFunction {
            name: "log".to_string(),
            module: "console".to_string(),
            required_capability: None,
            signature: "(message: string) -> void".to_string(),
        });

        self.functions.push(HostFunction {
            name: "warn".to_string(),
            module: "console".to_string(),
            required_capability: None,
            signature: "(message: string) -> void".to_string(),
        });

        self.functions.push(HostFunction {
            name: "error".to_string(),
            module: "console".to_string(),
            required_capability: None,
            signature: "(message: string) -> void".to_string(),
        });

        // Storage functions (requires capability)
        self.functions.push(HostFunction {
            name: "get_item".to_string(),
            module: "storage".to_string(),
            required_capability: Some(Capability::StorageLocal),
            signature: "(key: string) -> string?".to_string(),
        });

        self.functions.push(HostFunction {
            name: "set_item".to_string(),
            module: "storage".to_string(),
            required_capability: Some(Capability::StorageLocal),
            signature: "(key: string, value: string) -> void".to_string(),
        });

        self
    }

    /// Register a custom host function.
    pub fn register(&mut self, function: HostFunction) {
        self.functions.push(function);
    }

    /// Get all registered functions.
    pub fn functions(&self) -> &[HostFunction] {
        &self.functions
    }

    /// Get functions available for a given set of capabilities.
    pub fn functions_for_capabilities(&self, capabilities: &[Capability]) -> Vec<&HostFunction> {
        self.functions
            .iter()
            .filter(|f| {
                match &f.required_capability {
                    None => true,
                    Some(cap) => capabilities.contains(cap),
                }
            })
            .collect()
    }
}

/// Resource usage statistics from sandbox execution.
#[derive(Debug, Clone, Default)]
pub struct SandboxStats {
    /// Memory used in bytes.
    pub memory_used: u64,
    /// Execution time in milliseconds.
    pub execution_time_ms: u64,
    /// Number of function calls.
    pub function_calls: u64,
    /// Number of host function calls.
    pub host_calls: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.max_memory, 256 * 1024 * 1024);
        assert_eq!(config.max_execution_time_ms, 5000);
        assert!(!config.allow_wasi_fs);
        assert!(!config.allow_wasi_net);
    }

    #[test]
    fn test_sandbox_config_minimal() {
        let config = SandboxConfig::minimal();
        assert_eq!(config.max_memory, 64 * 1024 * 1024);
        assert_eq!(config.max_execution_time_ms, 1000);
    }

    #[test]
    fn test_sandbox_config_with_capabilities() {
        let config = SandboxConfig::default()
            .with_capability(Capability::StorageLocal);

        assert!(config.is_capability_allowed(&Capability::StorageLocal));
        assert!(!config.is_capability_allowed(&Capability::FilesystemRead));
    }

    #[test]
    fn test_host_functions() {
        let functions = HostFunctions::new().with_standard_functions();
        assert!(!functions.functions().is_empty());

        // Console functions should be available without capabilities
        let no_caps = functions.functions_for_capabilities(&[]);
        assert!(no_caps.iter().any(|f| f.name == "log"));

        // Storage functions need capability
        let with_storage = functions.functions_for_capabilities(&[Capability::StorageLocal]);
        assert!(with_storage.iter().any(|f| f.name == "get_item"));
    }
}
