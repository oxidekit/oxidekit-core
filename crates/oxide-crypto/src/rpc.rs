//! RPC provider management.
//!
//! This module provides types for:
//! - JSON-RPC client configuration
//! - Provider pool with failover
//! - Rate limiting
//! - TLS enforcement
//! - Network allowlist management

use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::time::Duration;
use url::Url;

// ============================================================================
// Provider Configuration
// ============================================================================

/// Configuration for an RPC provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    /// Provider URL
    url: String,
    /// Provider name (for logging)
    pub name: Option<String>,
    /// Authentication configuration
    pub auth: Option<AuthConfig>,
    /// Request timeout
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Maximum retries
    pub max_retries: u32,
    /// Priority (lower = higher priority)
    pub priority: u32,
    /// Weight for load balancing
    pub weight: u32,
    /// Whether this provider is enabled
    pub enabled: bool,
    /// Custom headers
    pub headers: HashMap<String, String>,
}

impl ProviderConfig {
    /// Create a new provider configuration.
    pub fn new(url: &str) -> CryptoResult<Self> {
        // Validate URL
        let parsed = Url::parse(url).map_err(|e| {
            CryptoError::ConfigError(format!("invalid provider URL: {}", e))
        })?;

        // Enforce HTTPS (unless localhost)
        if parsed.scheme() != "https" && !Self::is_localhost(&parsed) {
            return Err(CryptoError::TlsRequired);
        }

        Ok(Self {
            url: url.to_string(),
            name: None,
            auth: None,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            priority: 100,
            weight: 100,
            enabled: true,
            headers: HashMap::new(),
        })
    }

    /// Check if URL is localhost.
    fn is_localhost(url: &Url) -> bool {
        matches!(url.host_str(), Some("localhost") | Some("127.0.0.1") | Some("::1"))
    }

    /// Set provider name.
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Set authentication.
    pub fn with_auth(mut self, auth: AuthConfig) -> Self {
        self.auth = Some(auth);
        self
    }

    /// Set timeout.
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Add a custom header.
    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    /// Get the provider URL.
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the display name.
    pub fn display_name(&self) -> &str {
        self.name.as_deref().unwrap_or(&self.url)
    }
}

/// Serde helper for Duration using humantime.
mod humantime_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&format!("{}s", duration.as_secs()))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let s = String::deserialize(d)?;
        let secs: u64 = s
            .trim_end_matches('s')
            .parse()
            .map_err(serde::de::Error::custom)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Authentication configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthConfig {
    /// Bearer token authentication
    Bearer {
        /// The token (will be redacted in logs)
        token: String,
    },
    /// Basic authentication
    Basic {
        /// Username
        username: String,
        /// Password (will be redacted in logs)
        password: String,
    },
    /// API key in header
    ApiKey {
        /// Header name
        header: String,
        /// Key value (will be redacted in logs)
        key: String,
    },
    /// JWT authentication
    Jwt {
        /// The JWT token
        token: String,
    },
}

// ============================================================================
// Provider Pool
// ============================================================================

/// A pool of RPC providers with failover support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderPool {
    /// Providers in the pool
    providers: Vec<ProviderConfig>,
    /// Failover strategy
    pub failover_strategy: FailoverStrategy,
    /// Load balancing strategy
    pub load_balancing: LoadBalancingStrategy,
    /// Circuit breaker configuration
    pub circuit_breaker: CircuitBreakerConfig,
    /// Rate limiting configuration
    pub rate_limit: Option<RateLimitConfig>,
}

impl ProviderPool {
    /// Create a new empty provider pool.
    pub fn new() -> Self {
        Self {
            providers: Vec::new(),
            failover_strategy: FailoverStrategy::Priority,
            load_balancing: LoadBalancingStrategy::RoundRobin,
            circuit_breaker: CircuitBreakerConfig::default(),
            rate_limit: None,
        }
    }

    /// Add a provider to the pool.
    pub fn add_provider(mut self, provider: ProviderConfig) -> Self {
        self.providers.push(provider);
        self
    }

    /// Set the failover strategy.
    pub fn with_failover(mut self, strategy: FailoverStrategy) -> Self {
        self.failover_strategy = strategy;
        self
    }

    /// Set the load balancing strategy.
    pub fn with_load_balancing(mut self, strategy: LoadBalancingStrategy) -> Self {
        self.load_balancing = strategy;
        self
    }

    /// Set rate limiting.
    pub fn with_rate_limit(mut self, requests_per_second: u32, burst: u32) -> Self {
        self.rate_limit = Some(RateLimitConfig {
            requests_per_second,
            burst,
        });
        self
    }

    /// Get all providers.
    pub fn providers(&self) -> &[ProviderConfig] {
        &self.providers
    }

    /// Get enabled providers sorted by priority.
    pub fn enabled_providers(&self) -> Vec<&ProviderConfig> {
        let mut providers: Vec<_> = self.providers.iter().filter(|p| p.enabled).collect();
        providers.sort_by_key(|p| p.priority);
        providers
    }

    /// Get the number of providers.
    pub fn len(&self) -> usize {
        self.providers.len()
    }

    /// Check if the pool is empty.
    pub fn is_empty(&self) -> bool {
        self.providers.is_empty()
    }
}

impl Default for ProviderPool {
    fn default() -> Self {
        Self::new()
    }
}

/// Failover strategy for provider selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailoverStrategy {
    /// Use providers in priority order
    Priority,
    /// Use fastest responding provider
    Fastest,
    /// Use all providers and compare results
    Consensus,
    /// No failover (fail immediately)
    None,
}

/// Load balancing strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LoadBalancingStrategy {
    /// Round-robin selection
    RoundRobin,
    /// Weighted random selection
    WeightedRandom,
    /// Least connections
    LeastConnections,
    /// Random selection
    Random,
}

// ============================================================================
// Rate Limiting
// ============================================================================

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,
    /// Burst size
    pub burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            requests_per_second: 10,
            burst: 20,
        }
    }
}

/// Rate limiter state.
#[derive(Debug, Clone)]
pub struct RateLimiter {
    config: RateLimitConfig,
    /// Available tokens
    tokens: f64,
    /// Last update timestamp
    last_update: std::time::Instant,
}

impl RateLimiter {
    /// Create a new rate limiter.
    pub fn new(config: RateLimitConfig) -> Self {
        Self {
            tokens: config.burst as f64,
            last_update: std::time::Instant::now(),
            config,
        }
    }

    /// Try to acquire a token for a request.
    pub fn try_acquire(&mut self) -> bool {
        self.refill();

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    /// Get the wait time until a token is available.
    pub fn wait_time(&mut self) -> Duration {
        self.refill();

        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            let tokens_needed = 1.0 - self.tokens;
            let seconds = tokens_needed / self.config.requests_per_second as f64;
            Duration::from_secs_f64(seconds)
        }
    }

    /// Refill tokens based on elapsed time.
    fn refill(&mut self) {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_update);
        let new_tokens = elapsed.as_secs_f64() * self.config.requests_per_second as f64;
        self.tokens = (self.tokens + new_tokens).min(self.config.burst as f64);
        self.last_update = now;
    }

    /// Get current available tokens.
    pub fn available_tokens(&self) -> u32 {
        self.tokens as u32
    }
}

// ============================================================================
// Circuit Breaker
// ============================================================================

/// Circuit breaker configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening
    pub failure_threshold: u32,
    /// Duration to wait before half-opening
    #[serde(with = "humantime_serde")]
    pub reset_timeout: Duration,
    /// Number of successes in half-open to close
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            success_threshold: 2,
        }
    }
}

/// Circuit breaker state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CircuitState {
    /// Normal operation
    Closed,
    /// Circuit tripped, rejecting requests
    Open,
    /// Testing if service recovered
    HalfOpen,
}

impl fmt::Display for CircuitState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Closed => write!(f, "closed"),
            Self::Open => write!(f, "open"),
            Self::HalfOpen => write!(f, "half-open"),
        }
    }
}

/// Circuit breaker for a provider.
#[derive(Debug, Clone)]
pub struct CircuitBreaker {
    config: CircuitBreakerConfig,
    state: CircuitState,
    failure_count: u32,
    success_count: u32,
    last_failure: Option<std::time::Instant>,
}

impl CircuitBreaker {
    /// Create a new circuit breaker.
    pub fn new(config: CircuitBreakerConfig) -> Self {
        Self {
            config,
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure: None,
        }
    }

    /// Check if the circuit allows requests.
    pub fn allows(&mut self) -> bool {
        match self.state {
            CircuitState::Closed => true,
            CircuitState::Open => {
                // Check if reset timeout has passed
                if let Some(last_failure) = self.last_failure {
                    if last_failure.elapsed() >= self.config.reset_timeout {
                        self.state = CircuitState::HalfOpen;
                        self.success_count = 0;
                        true
                    } else {
                        false
                    }
                } else {
                    // No recorded failure, shouldn't happen but allow
                    self.state = CircuitState::Closed;
                    true
                }
            }
            CircuitState::HalfOpen => true,
        }
    }

    /// Record a successful request.
    pub fn record_success(&mut self) {
        match self.state {
            CircuitState::Closed => {
                self.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitState::Closed;
                    self.failure_count = 0;
                    self.success_count = 0;
                }
            }
            CircuitState::Open => {
                // Shouldn't happen, but reset
                self.state = CircuitState::Closed;
            }
        }
    }

    /// Record a failed request.
    pub fn record_failure(&mut self) {
        self.failure_count += 1;
        self.last_failure = Some(std::time::Instant::now());

        match self.state {
            CircuitState::Closed => {
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitState::Open;
                }
            }
            CircuitState::HalfOpen => {
                self.state = CircuitState::Open;
                self.success_count = 0;
            }
            CircuitState::Open => {}
        }
    }

    /// Get current state.
    pub fn state(&self) -> CircuitState {
        self.state
    }

    /// Reset the circuit breaker.
    pub fn reset(&mut self) {
        self.state = CircuitState::Closed;
        self.failure_count = 0;
        self.success_count = 0;
        self.last_failure = None;
    }
}

// ============================================================================
// RPC Client Types
// ============================================================================

/// A JSON-RPC request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: JsonRpcId,
    /// Method name
    pub method: String,
    /// Method parameters
    pub params: serde_json::Value,
}

impl JsonRpcRequest {
    /// Create a new request.
    pub fn new(method: &str, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: JsonRpcId::Number(1),
            method: method.to_string(),
            params,
        }
    }

    /// Create with a specific ID.
    pub fn with_id(mut self, id: JsonRpcId) -> Self {
        self.id = id;
        self
    }
}

/// JSON-RPC request ID.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum JsonRpcId {
    /// Numeric ID
    Number(u64),
    /// String ID
    String(String),
    /// Null ID
    Null,
}

/// A JSON-RPC response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    /// JSON-RPC version
    pub jsonrpc: String,
    /// Request ID
    pub id: JsonRpcId,
    /// Result (on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    /// Error (on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

impl JsonRpcResponse {
    /// Check if the response is an error.
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }

    /// Get the result or error.
    pub fn into_result(self) -> CryptoResult<serde_json::Value> {
        if let Some(error) = self.error {
            Err(CryptoError::RpcRequestFailed {
                method: format!("code {}: {}", error.code, error.message),
            })
        } else {
            self.result.ok_or_else(|| CryptoError::RpcRequestFailed {
                method: "empty response".to_string(),
            })
        }
    }
}

/// A JSON-RPC error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonRpcError {
    /// Error code
    pub code: i32,
    /// Error message
    pub message: String,
    /// Additional data
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Batch of JSON-RPC requests.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct JsonRpcBatch {
    /// Requests in the batch
    pub requests: Vec<JsonRpcRequest>,
}

impl JsonRpcBatch {
    /// Create a new batch.
    pub fn new() -> Self {
        Self { requests: Vec::new() }
    }

    /// Add a request to the batch.
    pub fn add(mut self, request: JsonRpcRequest) -> Self {
        self.requests.push(request);
        self
    }

    /// Get the number of requests.
    pub fn len(&self) -> usize {
        self.requests.len()
    }

    /// Check if the batch is empty.
    pub fn is_empty(&self) -> bool {
        self.requests.is_empty()
    }
}

impl Default for JsonRpcBatch {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Network Allowlist
// ============================================================================

/// Network allowlist for security.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkAllowlist {
    /// Allowed domains
    domains: Vec<AllowedDomain>,
    /// Whether to block all non-listed domains
    pub strict_mode: bool,
}

impl NetworkAllowlist {
    /// Create a new empty allowlist.
    pub fn new() -> Self {
        Self {
            domains: Vec::new(),
            strict_mode: true,
        }
    }

    /// Add an allowed domain.
    pub fn allow_domain(mut self, pattern: &str) -> Self {
        self.domains.push(AllowedDomain {
            pattern: pattern.to_string(),
            description: None,
        });
        self
    }

    /// Add an allowed domain with description.
    pub fn allow_domain_with_desc(mut self, pattern: &str, description: &str) -> Self {
        self.domains.push(AllowedDomain {
            pattern: pattern.to_string(),
            description: Some(description.to_string()),
        });
        self
    }

    /// Check if a URL is allowed.
    pub fn is_allowed(&self, url: &str) -> bool {
        if !self.strict_mode {
            return true;
        }

        let parsed = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        let host = match parsed.host_str() {
            Some(h) => h,
            None => return false,
        };

        self.domains.iter().any(|d| d.matches(host))
    }

    /// Get all allowed domains.
    pub fn domains(&self) -> &[AllowedDomain] {
        &self.domains
    }

    /// Create a default allowlist for Ethereum.
    pub fn ethereum_default() -> Self {
        Self::new()
            .allow_domain_with_desc("*.infura.io", "Infura RPC")
            .allow_domain_with_desc("*.alchemy.com", "Alchemy RPC")
            .allow_domain_with_desc("*.llamarpc.com", "Llama RPC")
            .allow_domain_with_desc("cloudflare-eth.com", "Cloudflare RPC")
            .allow_domain_with_desc("*.etherscan.io", "Etherscan API")
    }

    /// Create a default allowlist for Bitcoin.
    pub fn bitcoin_default() -> Self {
        Self::new()
            .allow_domain_with_desc("mempool.space", "Mempool.space API")
            .allow_domain_with_desc("blockstream.info", "Blockstream API")
            .allow_domain_with_desc("localhost", "Local node")
    }
}

impl Default for NetworkAllowlist {
    fn default() -> Self {
        Self::new()
    }
}

/// An allowed domain pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllowedDomain {
    /// Domain pattern (supports * wildcard)
    pub pattern: String,
    /// Optional description
    pub description: Option<String>,
}

impl AllowedDomain {
    /// Check if a host matches this pattern.
    pub fn matches(&self, host: &str) -> bool {
        if self.pattern.starts_with("*.") {
            // Wildcard subdomain
            let suffix = &self.pattern[2..];
            host == suffix || host.ends_with(&format!(".{}", suffix))
        } else {
            host == self.pattern
        }
    }
}

/// RPC client configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcClient {
    /// Provider pool
    pub providers: ProviderPool,
    /// Network allowlist
    pub allowlist: NetworkAllowlist,
    /// Default timeout
    #[serde(with = "humantime_serde")]
    pub default_timeout: Duration,
    /// Whether to validate TLS certificates
    pub validate_certs: bool,
    /// User agent string
    pub user_agent: String,
}

impl Default for RpcClient {
    fn default() -> Self {
        Self {
            providers: ProviderPool::new(),
            allowlist: NetworkAllowlist::new(),
            default_timeout: Duration::from_secs(30),
            validate_certs: true,
            user_agent: format!("OxideKit-Crypto/0.1.0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_provider_config() {
        let config = ProviderConfig::new("https://eth.llamarpc.com").unwrap();
        assert!(config.enabled);
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_provider_config_requires_https() {
        let result = ProviderConfig::new("http://insecure.example.com");
        assert!(matches!(result, Err(CryptoError::TlsRequired)));
    }

    #[test]
    fn test_provider_config_allows_localhost() {
        let config = ProviderConfig::new("http://localhost:8545").unwrap();
        assert!(config.enabled);
    }

    #[test]
    fn test_provider_pool() {
        let pool = ProviderPool::new()
            .add_provider(ProviderConfig::new("https://a.example.com").unwrap().with_priority(2))
            .add_provider(ProviderConfig::new("https://b.example.com").unwrap().with_priority(1));

        assert_eq!(pool.len(), 2);

        let enabled = pool.enabled_providers();
        assert_eq!(enabled.len(), 2);
        assert_eq!(enabled[0].priority, 1); // Lower priority first
    }

    #[test]
    fn test_rate_limiter() {
        let config = RateLimitConfig {
            requests_per_second: 10,
            burst: 5,
        };
        let mut limiter = RateLimiter::new(config);

        // Should allow burst
        for _ in 0..5 {
            assert!(limiter.try_acquire());
        }

        // Should be rate limited after burst
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_circuit_breaker() {
        let config = CircuitBreakerConfig {
            failure_threshold: 2,
            reset_timeout: Duration::from_millis(100),
            success_threshold: 1,
        };
        let mut breaker = CircuitBreaker::new(config);

        assert!(breaker.allows());
        assert_eq!(breaker.state(), CircuitState::Closed);

        // Trigger failures
        breaker.record_failure();
        breaker.record_failure();
        assert_eq!(breaker.state(), CircuitState::Open);
        assert!(!breaker.allows());
    }

    #[test]
    fn test_network_allowlist() {
        let allowlist = NetworkAllowlist::new()
            .allow_domain("*.infura.io")
            .allow_domain("cloudflare-eth.com");

        assert!(allowlist.is_allowed("https://mainnet.infura.io/v3/key"));
        assert!(allowlist.is_allowed("https://cloudflare-eth.com"));
        assert!(!allowlist.is_allowed("https://malicious.com"));
    }

    #[test]
    fn test_json_rpc_request() {
        let request = JsonRpcRequest::new("eth_blockNumber", serde_json::json!([]));
        assert_eq!(request.method, "eth_blockNumber");
        assert_eq!(request.jsonrpc, "2.0");
    }

    #[test]
    fn test_json_rpc_batch() {
        let batch = JsonRpcBatch::new()
            .add(JsonRpcRequest::new("eth_blockNumber", serde_json::json!([])))
            .add(JsonRpcRequest::new("eth_gasPrice", serde_json::json!([])));

        assert_eq!(batch.len(), 2);
    }
}
