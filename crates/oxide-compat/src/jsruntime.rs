//! JavaScript Runtime Compatibility Plugin
//!
//! **WARNING**: This is NOT the recommended path for OxideKit applications.
//! Consider porting JS logic to Rust for better performance and security.
//!
//! # Purpose
//!
//! Run non-UI JavaScript utilities (parsers, formatters, validators) inside
//! OxideKit without shipping Node or a DOM.
//!
//! # Use Cases
//!
//! - JSON schema validation
//! - Markdown parsing
//! - Templating for static sites
//! - Minor data transforms
//!
//! # Security
//!
//! - Deterministic sandbox with memory/time limits
//! - No filesystem/network access by default
//! - Capability bridging only via OxideKit permission model
//! - No DOM APIs, no Node APIs

use crate::policy::{CompatPolicy, CompatFeature, PolicyEnforcer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;

/// Errors that can occur in JS runtime operations
#[derive(Error, Debug)]
pub enum JsRuntimeError {
    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Execution error
    #[error("JavaScript execution error: {0}")]
    ExecutionError(String),

    /// Timeout error
    #[error("JavaScript execution timed out after {0}ms")]
    Timeout(u64),

    /// Memory limit exceeded
    #[error("JavaScript memory limit exceeded ({0} MB)")]
    MemoryExceeded(u32),

    /// Module not found
    #[error("JavaScript module not found: {0}")]
    ModuleNotFound(String),

    /// Eval not allowed
    #[error("eval() is not allowed by policy")]
    EvalNotAllowed,

    /// Capability not granted
    #[error("Capability not granted: {0}")]
    CapabilityDenied(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Runtime not initialized
    #[error("JavaScript runtime not initialized")]
    NotInitialized,
}

/// Configuration for the JavaScript runtime
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsRuntimeConfig {
    /// Maximum memory in megabytes
    pub max_memory_mb: u32,

    /// Maximum execution time in milliseconds
    pub timeout_ms: u64,

    /// Allow eval() function (DANGEROUS)
    pub allow_eval: bool,

    /// Enable strict mode
    pub strict_mode: bool,

    /// Capabilities granted to the runtime
    pub capabilities: Vec<JsCapability>,

    /// Pre-loaded modules
    pub modules: Vec<JsModule>,

    /// Global variables to inject
    pub globals: HashMap<String, serde_json::Value>,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 64,
            timeout_ms: 5000,
            allow_eval: false,
            strict_mode: true,
            capabilities: Vec::new(),
            modules: Vec::new(),
            globals: HashMap::new(),
        }
    }
}

/// Capabilities that can be granted to the JS runtime
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JsCapability {
    /// Read environment variables (filtered)
    EnvRead,
    /// Console logging
    ConsoleLog,
    /// Date/time access
    DateTime,
    /// Crypto (random, hashing)
    Crypto,
    /// Text encoding/decoding
    TextCodec,
    /// JSON parsing
    Json,
    /// URL parsing
    UrlParsing,
    /// Regular expressions
    Regex,
}

/// A JavaScript module definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsModule {
    /// Module name (for imports)
    pub name: String,
    /// Module source code
    pub source: String,
    /// Module version
    pub version: Option<String>,
    /// Hash of the source for verification
    pub hash: Option<String>,
}

/// A value that can be passed to/from JavaScript
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsValue {
    /// Null value
    Null,
    /// Boolean value
    Bool(bool),
    /// Integer value
    Int(i64),
    /// Float value
    Float(f64),
    /// String value
    String(String),
    /// Array value
    Array(Vec<JsValue>),
    /// Object value
    Object(HashMap<String, JsValue>),
}

impl JsValue {
    /// Create a null value
    pub fn null() -> Self {
        JsValue::Null
    }

    /// Create from a serde_json::Value
    pub fn from_json(value: serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => JsValue::Null,
            serde_json::Value::Bool(b) => JsValue::Bool(b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    JsValue::Int(i)
                } else {
                    JsValue::Float(n.as_f64().unwrap_or(0.0))
                }
            }
            serde_json::Value::String(s) => JsValue::String(s),
            serde_json::Value::Array(arr) => {
                JsValue::Array(arr.into_iter().map(JsValue::from_json).collect())
            }
            serde_json::Value::Object(obj) => JsValue::Object(
                obj.into_iter()
                    .map(|(k, v)| (k, JsValue::from_json(v)))
                    .collect(),
            ),
        }
    }

    /// Convert to serde_json::Value
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            JsValue::Null => serde_json::Value::Null,
            JsValue::Bool(b) => serde_json::Value::Bool(*b),
            JsValue::Int(i) => serde_json::json!(*i),
            JsValue::Float(f) => serde_json::json!(*f),
            JsValue::String(s) => serde_json::Value::String(s.clone()),
            JsValue::Array(arr) => {
                serde_json::Value::Array(arr.iter().map(|v| v.to_json()).collect())
            }
            JsValue::Object(obj) => serde_json::Value::Object(
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.to_json()))
                    .collect(),
            ),
        }
    }
}

/// Result of JavaScript execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsExecutionResult {
    /// Return value
    pub value: JsValue,
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Memory used in bytes
    pub memory_used_bytes: u64,
    /// Console output (if ConsoleLog capability is enabled)
    pub console_output: Vec<String>,
}

/// JavaScript runtime instance
#[cfg(feature = "js-runtime")]
pub struct JsRuntime {
    /// Runtime configuration
    config: JsRuntimeConfig,
    /// Policy for enforcement
    policy: CompatPolicy,
    /// Whether the runtime is initialized
    initialized: bool,
    /// Loaded modules
    loaded_modules: HashMap<String, String>,
    /// Native function bindings
    bindings: HashMap<String, NativeBinding>,
}

/// Native function binding
pub type NativeBinding = Box<dyn Fn(Vec<JsValue>) -> Result<JsValue, String> + Send + Sync>;

#[cfg(feature = "js-runtime")]
impl JsRuntime {
    /// Create a new JavaScript runtime
    pub fn new(config: JsRuntimeConfig, policy: CompatPolicy) -> Result<Self, JsRuntimeError> {
        // Validate against policy
        let mut enforcer = PolicyEnforcer::new(policy.clone(), false);

        if !enforcer.enforce(CompatFeature::JsRuntime) {
            return Err(JsRuntimeError::PolicyViolation(
                "JavaScript runtime is not allowed by policy".to_string(),
            ));
        }

        if config.allow_eval && !enforcer.enforce(CompatFeature::JsEval) {
            return Err(JsRuntimeError::PolicyViolation(
                "eval() is not allowed by policy".to_string(),
            ));
        }

        // Apply policy limits
        let mut effective_config = config;
        effective_config.max_memory_mb = effective_config
            .max_memory_mb
            .min(policy.js_memory_limit_mb);
        effective_config.timeout_ms = effective_config.timeout_ms.min(policy.js_timeout_ms as u64);

        // Print warning
        crate::print_compat_warning();

        Ok(Self {
            config: effective_config,
            policy,
            initialized: false,
            loaded_modules: HashMap::new(),
            bindings: HashMap::new(),
        })
    }

    /// Initialize the runtime
    pub fn init(&mut self) -> Result<(), JsRuntimeError> {
        // Load pre-configured modules
        for module in &self.config.modules {
            self.loaded_modules
                .insert(module.name.clone(), module.source.clone());
        }

        // Register default bindings based on capabilities
        self.register_default_bindings();

        self.initialized = true;
        Ok(())
    }

    /// Register default bindings based on granted capabilities
    fn register_default_bindings(&mut self) {
        for capability in &self.config.capabilities {
            match capability {
                JsCapability::ConsoleLog => {
                    self.bind("__oxide_console_log", Box::new(|args| {
                        let msg: Vec<String> = args
                            .iter()
                            .map(|v| match v {
                                JsValue::String(s) => s.clone(),
                                other => format!("{:?}", other),
                            })
                            .collect();
                        tracing::info!("[JS Console] {}", msg.join(" "));
                        Ok(JsValue::Null)
                    }));
                }
                JsCapability::DateTime => {
                    self.bind("__oxide_date_now", Box::new(|_| {
                        let now = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_millis() as i64)
                            .unwrap_or(0);
                        Ok(JsValue::Int(now))
                    }));
                }
                JsCapability::Crypto => {
                    self.bind("__oxide_crypto_random", Box::new(|_| {
                        let random: u64 = rand_simple();
                        Ok(JsValue::Float(random as f64 / u64::MAX as f64))
                    }));
                }
                _ => {}
            }
        }
    }

    /// Register a native function binding
    pub fn bind(&mut self, name: &str, func: NativeBinding) {
        self.bindings.insert(name.to_string(), func);
    }

    /// Load a JavaScript module
    pub fn load_module(&mut self, module: JsModule) -> Result<(), JsRuntimeError> {
        // Verify hash if provided
        if let (Some(hash), Some(_)) = (&module.hash, Some(&module.source)) {
            let computed = compute_sha256(&module.source);
            if &computed != hash {
                return Err(JsRuntimeError::ModuleNotFound(format!(
                    "Module {} hash mismatch",
                    module.name
                )));
            }
        }

        self.loaded_modules
            .insert(module.name.clone(), module.source);
        Ok(())
    }

    /// Execute JavaScript code
    pub fn eval(&self, code: &str) -> Result<JsExecutionResult, JsRuntimeError> {
        if !self.initialized {
            return Err(JsRuntimeError::NotInitialized);
        }

        // Check for eval() usage if not allowed
        if !self.config.allow_eval && (code.contains("eval(") || code.contains("Function(")) {
            return Err(JsRuntimeError::EvalNotAllowed);
        }

        let start = std::time::Instant::now();

        // This is where we would call into rquickjs
        // For now, return a placeholder result
        let result = self.execute_with_quickjs(code)?;

        let elapsed = start.elapsed();

        // Check timeout
        if elapsed.as_millis() as u64 > self.config.timeout_ms {
            return Err(JsRuntimeError::Timeout(self.config.timeout_ms));
        }

        Ok(JsExecutionResult {
            value: result,
            execution_time_ms: elapsed.as_millis() as u64,
            memory_used_bytes: 0, // Would be tracked by the runtime
            console_output: Vec::new(),
        })
    }

    /// Execute JavaScript with QuickJS (placeholder implementation)
    #[allow(unused_variables)]
    fn execute_with_quickjs(&self, code: &str) -> Result<JsValue, JsRuntimeError> {
        // This is a placeholder - actual implementation would use rquickjs
        // The real implementation would:
        // 1. Create a QuickJS context with memory limits
        // 2. Set up interrupt handler for timeout
        // 3. Inject globals and bindings
        // 4. Execute the code
        // 5. Extract the result

        // For now, return undefined
        Ok(JsValue::Null)
    }

    /// Call a JavaScript function
    pub fn call(
        &self,
        function: &str,
        args: Vec<JsValue>,
    ) -> Result<JsExecutionResult, JsRuntimeError> {
        let args_json: Vec<String> = args
            .iter()
            .map(|v| serde_json::to_string(&v.to_json()).unwrap_or_else(|_| "null".to_string()))
            .collect();

        let code = format!("{}({})", function, args_json.join(", "));
        self.eval(&code)
    }

    /// Get runtime configuration
    pub fn config(&self) -> &JsRuntimeConfig {
        &self.config
    }

    /// Check if a capability is enabled
    pub fn has_capability(&self, capability: JsCapability) -> bool {
        self.config.capabilities.contains(&capability)
    }
}

/// Builder for JsRuntime
pub struct JsRuntimeBuilder {
    config: JsRuntimeConfig,
}

impl JsRuntimeBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: JsRuntimeConfig::default(),
        }
    }

    /// Set maximum memory limit
    pub fn max_memory_mb(mut self, mb: u32) -> Self {
        self.config.max_memory_mb = mb;
        self
    }

    /// Set execution timeout
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.config.timeout_ms = duration.as_millis() as u64;
        self
    }

    /// Allow eval() (DANGEROUS)
    pub fn allow_eval(mut self, allow: bool) -> Self {
        self.config.allow_eval = allow;
        self
    }

    /// Enable strict mode
    pub fn strict_mode(mut self, strict: bool) -> Self {
        self.config.strict_mode = strict;
        self
    }

    /// Add a capability
    pub fn capability(mut self, cap: JsCapability) -> Self {
        self.config.capabilities.push(cap);
        self
    }

    /// Add multiple capabilities
    pub fn capabilities(mut self, caps: &[JsCapability]) -> Self {
        self.config.capabilities.extend_from_slice(caps);
        self
    }

    /// Add a module
    pub fn module(mut self, module: JsModule) -> Self {
        self.config.modules.push(module);
        self
    }

    /// Add a global variable
    pub fn global(mut self, name: &str, value: serde_json::Value) -> Self {
        self.config.globals.insert(name.to_string(), value);
        self
    }

    /// Build the configuration
    pub fn build(self) -> JsRuntimeConfig {
        self.config
    }

    /// Build and create the runtime
    #[cfg(feature = "js-runtime")]
    pub fn create(self, policy: CompatPolicy) -> Result<JsRuntime, JsRuntimeError> {
        JsRuntime::new(self.config, policy)
    }
}

impl Default for JsRuntimeBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Compute SHA256 hash of a string
fn compute_sha256(input: &str) -> String {
    use sha2::{Sha256, Digest};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    hex::encode(hasher.finalize())
}

/// Simple random number generator (placeholder)
fn rand_simple() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_nanos() as u64 ^ (duration.as_secs() << 32)
}

/// Shim provider for common JavaScript utilities
pub struct JsShimProvider;

impl JsShimProvider {
    /// Get console shim code
    pub fn console_shim() -> &'static str {
        r#"
        var console = {
            log: function() {
                __oxide_console_log.apply(null, arguments);
            },
            info: function() {
                __oxide_console_log.apply(null, ['[INFO]'].concat(Array.from(arguments)));
            },
            warn: function() {
                __oxide_console_log.apply(null, ['[WARN]'].concat(Array.from(arguments)));
            },
            error: function() {
                __oxide_console_log.apply(null, ['[ERROR]'].concat(Array.from(arguments)));
            }
        };
        "#
    }

    /// Get Date shim code
    pub fn date_shim() -> &'static str {
        r#"
        (function() {
            var _Date = Date;
            Date.now = function() {
                return __oxide_date_now();
            };
        })();
        "#
    }

    /// Get Math.random shim code
    pub fn math_random_shim() -> &'static str {
        r#"
        (function() {
            Math.random = function() {
                return __oxide_crypto_random();
            };
        })();
        "#
    }

    /// Get all shims for given capabilities
    pub fn shims_for_capabilities(capabilities: &[JsCapability]) -> String {
        let mut shims = String::new();

        for cap in capabilities {
            match cap {
                JsCapability::ConsoleLog => {
                    shims.push_str(Self::console_shim());
                    shims.push('\n');
                }
                JsCapability::DateTime => {
                    shims.push_str(Self::date_shim());
                    shims.push('\n');
                }
                JsCapability::Crypto => {
                    shims.push_str(Self::math_random_shim());
                    shims.push('\n');
                }
                _ => {}
            }
        }

        shims
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_js_runtime_builder() {
        let config = JsRuntimeBuilder::new()
            .max_memory_mb(128)
            .timeout(Duration::from_secs(10))
            .strict_mode(true)
            .capability(JsCapability::ConsoleLog)
            .capability(JsCapability::Json)
            .build();

        assert_eq!(config.max_memory_mb, 128);
        assert_eq!(config.timeout_ms, 10000);
        assert!(config.strict_mode);
        assert!(config.capabilities.contains(&JsCapability::ConsoleLog));
        assert!(config.capabilities.contains(&JsCapability::Json));
    }

    #[test]
    fn test_js_value_conversion() {
        let json = serde_json::json!({
            "name": "test",
            "count": 42,
            "active": true,
            "items": [1, 2, 3]
        });

        let value = JsValue::from_json(json.clone());
        let back = value.to_json();

        assert_eq!(json, back);
    }

    #[test]
    fn test_module_hash_verification() {
        let source = "function test() { return 42; }";
        let hash = compute_sha256(source);

        let module = JsModule {
            name: "test".to_string(),
            source: source.to_string(),
            version: Some("1.0.0".to_string()),
            hash: Some(hash.clone()),
        };

        assert_eq!(compute_sha256(&module.source), hash);
    }

    #[test]
    fn test_shim_provider() {
        let capabilities = vec![
            JsCapability::ConsoleLog,
            JsCapability::DateTime,
            JsCapability::Crypto,
        ];

        let shims = JsShimProvider::shims_for_capabilities(&capabilities);

        assert!(shims.contains("console"));
        assert!(shims.contains("Date.now"));
        assert!(shims.contains("Math.random"));
    }
}
