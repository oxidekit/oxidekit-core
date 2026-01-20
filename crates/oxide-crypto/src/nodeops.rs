//! Node and operator tooling types.
//!
//! This module provides types for:
//! - Node binary installation helpers
//! - Docker/Compose templates
//! - Systemd units
//! - Snapshots and backups
//! - Log discovery
//! - Health checks
//!
//! Initial support:
//! - Ethereum clients (Geth, Reth, Erigon, etc.)
//! - Bitcoin Core

// Note: CryptoError and CryptoResult are available but not currently used
// They will be used when implementing actual node operations
#[allow(unused_imports)]
use crate::{CryptoError, CryptoResult};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::time::Duration;
use chrono::{DateTime, Utc};

// ============================================================================
// Node Binary Information
// ============================================================================

/// Supported node types.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeType {
    // Ethereum clients
    /// Geth - Go Ethereum
    Geth,
    /// Reth - Rust Ethereum
    Reth,
    /// Erigon - Efficient Ethereum client
    Erigon,
    /// Nethermind - .NET Ethereum client
    Nethermind,
    /// Besu - Java Ethereum client
    Besu,
    /// Lighthouse - Ethereum consensus client
    Lighthouse,
    /// Prysm - Ethereum consensus client
    Prysm,
    /// Teku - Ethereum consensus client
    Teku,
    /// Lodestar - Ethereum consensus client
    Lodestar,
    /// Nimbus - Ethereum consensus client
    Nimbus,

    // Bitcoin clients
    /// Bitcoin Core
    BitcoinCore,
    /// Bitcoin Knots
    BitcoinKnots,

    // Other
    /// Custom node type
    Custom,
}

impl NodeType {
    /// Get the default binary name.
    pub fn binary_name(&self) -> &'static str {
        match self {
            Self::Geth => "geth",
            Self::Reth => "reth",
            Self::Erigon => "erigon",
            Self::Nethermind => "nethermind",
            Self::Besu => "besu",
            Self::Lighthouse => "lighthouse",
            Self::Prysm => "prysm",
            Self::Teku => "teku",
            Self::Lodestar => "lodestar",
            Self::Nimbus => "nimbus",
            Self::BitcoinCore => "bitcoind",
            Self::BitcoinKnots => "bitcoind",
            Self::Custom => "custom",
        }
    }

    /// Get the default data directory name.
    pub fn data_dir_name(&self) -> &'static str {
        match self {
            Self::Geth => "geth",
            Self::Reth => "reth",
            Self::Erigon => "erigon",
            Self::Nethermind => "nethermind",
            Self::Besu => "besu",
            Self::Lighthouse => "lighthouse",
            Self::Prysm => "prysm",
            Self::Teku => "teku",
            Self::Lodestar => "lodestar",
            Self::Nimbus => "nimbus",
            Self::BitcoinCore => ".bitcoin",
            Self::BitcoinKnots => ".bitcoin",
            Self::Custom => "custom",
        }
    }

    /// Check if this is an Ethereum execution client.
    pub fn is_eth_execution(&self) -> bool {
        matches!(
            self,
            Self::Geth | Self::Reth | Self::Erigon | Self::Nethermind | Self::Besu
        )
    }

    /// Check if this is an Ethereum consensus client.
    pub fn is_eth_consensus(&self) -> bool {
        matches!(
            self,
            Self::Lighthouse | Self::Prysm | Self::Teku | Self::Lodestar | Self::Nimbus
        )
    }

    /// Check if this is a Bitcoin client.
    pub fn is_bitcoin(&self) -> bool {
        matches!(self, Self::BitcoinCore | Self::BitcoinKnots)
    }
}

impl fmt::Display for NodeType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Geth => write!(f, "Geth"),
            Self::Reth => write!(f, "Reth"),
            Self::Erigon => write!(f, "Erigon"),
            Self::Nethermind => write!(f, "Nethermind"),
            Self::Besu => write!(f, "Besu"),
            Self::Lighthouse => write!(f, "Lighthouse"),
            Self::Prysm => write!(f, "Prysm"),
            Self::Teku => write!(f, "Teku"),
            Self::Lodestar => write!(f, "Lodestar"),
            Self::Nimbus => write!(f, "Nimbus"),
            Self::BitcoinCore => write!(f, "Bitcoin Core"),
            Self::BitcoinKnots => write!(f, "Bitcoin Knots"),
            Self::Custom => write!(f, "Custom"),
        }
    }
}

/// Information about a node binary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeBinaryInfo {
    /// Node type
    pub node_type: NodeType,
    /// Binary name
    pub binary_name: String,
    /// Version
    pub version: String,
    /// Installation path
    pub install_path: PathBuf,
    /// Whether the binary is available
    pub available: bool,
    /// Supported networks
    pub networks: Vec<String>,
}

impl NodeBinaryInfo {
    /// Create info for Geth.
    pub fn geth(version: &str, install_path: PathBuf) -> Self {
        Self {
            node_type: NodeType::Geth,
            binary_name: "geth".to_string(),
            version: version.to_string(),
            install_path,
            available: true,
            networks: vec!["mainnet".to_string(), "sepolia".to_string(), "holesky".to_string()],
        }
    }

    /// Create info for Bitcoin Core.
    pub fn bitcoin_core(version: &str, install_path: PathBuf) -> Self {
        Self {
            node_type: NodeType::BitcoinCore,
            binary_name: "bitcoind".to_string(),
            version: version.to_string(),
            install_path,
            available: true,
            networks: vec!["mainnet".to_string(), "testnet".to_string(), "signet".to_string()],
        }
    }
}

// ============================================================================
// Docker Templates
// ============================================================================

/// Docker Compose service configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerService {
    /// Service name
    pub name: String,
    /// Docker image
    pub image: String,
    /// Container name
    pub container_name: Option<String>,
    /// Restart policy
    pub restart: RestartPolicy,
    /// Port mappings (host:container)
    pub ports: Vec<String>,
    /// Volume mounts (host:container)
    pub volumes: Vec<String>,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Command override
    pub command: Option<Vec<String>>,
    /// Dependencies
    pub depends_on: Vec<String>,
    /// Health check
    pub healthcheck: Option<HealthCheck>,
    /// Labels
    pub labels: HashMap<String, String>,
    /// Network mode
    pub network_mode: Option<String>,
}

impl DockerService {
    /// Create a new Docker service.
    pub fn new(name: &str, image: &str) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            container_name: Some(name.to_string()),
            restart: RestartPolicy::UnlessStopped,
            ports: vec![],
            volumes: vec![],
            environment: HashMap::new(),
            command: None,
            depends_on: vec![],
            healthcheck: None,
            labels: HashMap::new(),
            network_mode: None,
        }
    }

    /// Add a port mapping.
    pub fn with_port(mut self, host: u16, container: u16) -> Self {
        self.ports.push(format!("{}:{}", host, container));
        self
    }

    /// Add a volume mount.
    pub fn with_volume(mut self, host: &str, container: &str) -> Self {
        self.volumes.push(format!("{}:{}", host, container));
        self
    }

    /// Add an environment variable.
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.environment.insert(key.to_string(), value.to_string());
        self
    }

    /// Set command.
    pub fn with_command(mut self, command: Vec<String>) -> Self {
        self.command = Some(command);
        self
    }

    /// Add a dependency.
    pub fn depends_on(mut self, service: &str) -> Self {
        self.depends_on.push(service.to_string());
        self
    }

    /// Add a health check.
    pub fn with_healthcheck(mut self, healthcheck: HealthCheck) -> Self {
        self.healthcheck = Some(healthcheck);
        self
    }
}

/// Restart policy for Docker containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RestartPolicy {
    /// Never restart
    No,
    /// Restart on failure
    OnFailure,
    /// Always restart
    Always,
    /// Restart unless stopped
    UnlessStopped,
}

impl Default for RestartPolicy {
    fn default() -> Self {
        Self::UnlessStopped
    }
}

/// Docker Compose file template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DockerComposeTemplate {
    /// Template name
    pub name: String,
    /// Description
    pub description: String,
    /// Services
    pub services: Vec<DockerService>,
    /// Networks
    pub networks: HashMap<String, NetworkConfig>,
    /// Volumes
    pub volumes: HashMap<String, VolumeConfig>,
}

impl DockerComposeTemplate {
    /// Create a new template.
    pub fn new(name: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            services: vec![],
            networks: HashMap::new(),
            volumes: HashMap::new(),
        }
    }

    /// Add a service.
    pub fn add_service(mut self, service: DockerService) -> Self {
        self.services.push(service);
        self
    }

    /// Add a network.
    pub fn add_network(mut self, name: &str, config: NetworkConfig) -> Self {
        self.networks.insert(name.to_string(), config);
        self
    }

    /// Add a volume.
    pub fn add_volume(mut self, name: &str, config: VolumeConfig) -> Self {
        self.volumes.insert(name.to_string(), config);
        self
    }

    /// Create an Ethereum node template (execution + consensus).
    pub fn ethereum_full_node() -> Self {
        let execution = DockerService::new("execution", "ethereum/client-go:stable")
            .with_port(8545, 8545)
            .with_port(8546, 8546)
            .with_port(30303, 30303)
            .with_volume("./data/geth", "/root/.ethereum")
            .with_command(vec![
                "--http".to_string(),
                "--http.addr=0.0.0.0".to_string(),
                "--authrpc.addr=0.0.0.0".to_string(),
                "--authrpc.jwtsecret=/root/.ethereum/jwt.hex".to_string(),
            ])
            .with_healthcheck(HealthCheck::http("/", 8545, Duration::from_secs(30)));

        let consensus = DockerService::new("consensus", "sigp/lighthouse:latest")
            .with_port(5052, 5052)
            .with_port(9000, 9000)
            .with_volume("./data/lighthouse", "/root/.lighthouse")
            .with_volume("./data/geth/jwt.hex", "/jwt.hex")
            .depends_on("execution")
            .with_command(vec![
                "lighthouse".to_string(),
                "bn".to_string(),
                "--network=mainnet".to_string(),
                "--execution-endpoint=http://execution:8551".to_string(),
                "--execution-jwt=/jwt.hex".to_string(),
            ]);

        Self::new("ethereum-full-node", "Ethereum full node with Geth and Lighthouse")
            .add_service(execution)
            .add_service(consensus)
    }

    /// Create a Bitcoin Core template.
    pub fn bitcoin_core() -> Self {
        let bitcoind = DockerService::new("bitcoind", "lncm/bitcoind:v27.0")
            .with_port(8332, 8332)
            .with_port(8333, 8333)
            .with_volume("./data/bitcoin", "/data/.bitcoin")
            .with_env("BITCOIN_DATA", "/data/.bitcoin")
            .with_healthcheck(HealthCheck::command(
                vec!["bitcoin-cli", "-getinfo"],
                Duration::from_secs(30),
            ));

        Self::new("bitcoin-core", "Bitcoin Core full node")
            .add_service(bitcoind)
    }
}

/// Docker network configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Driver
    pub driver: Option<String>,
    /// External network
    pub external: bool,
}

/// Docker volume configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VolumeConfig {
    /// Driver
    pub driver: Option<String>,
    /// External volume
    pub external: bool,
}

// ============================================================================
// Systemd Units
// ============================================================================

/// Systemd service unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemdUnit {
    /// Unit name
    pub name: String,
    /// Description
    pub description: String,
    /// After dependencies
    pub after: Vec<String>,
    /// Wants dependencies
    pub wants: Vec<String>,
    /// Service type
    pub service_type: SystemdServiceType,
    /// User
    pub user: Option<String>,
    /// Group
    pub group: Option<String>,
    /// Working directory
    pub working_directory: Option<PathBuf>,
    /// Exec start command
    pub exec_start: String,
    /// Exec stop command
    pub exec_stop: Option<String>,
    /// Restart policy
    pub restart: SystemdRestartPolicy,
    /// Restart delay
    pub restart_sec: u32,
    /// Environment variables
    pub environment: HashMap<String, String>,
    /// Environment file
    pub environment_file: Option<PathBuf>,
}

impl SystemdUnit {
    /// Create a new systemd unit.
    pub fn new(name: &str, description: &str, exec_start: &str) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            after: vec!["network.target".to_string()],
            wants: vec![],
            service_type: SystemdServiceType::Simple,
            user: None,
            group: None,
            working_directory: None,
            exec_start: exec_start.to_string(),
            exec_stop: None,
            restart: SystemdRestartPolicy::OnFailure,
            restart_sec: 5,
            environment: HashMap::new(),
            environment_file: None,
        }
    }

    /// Set user.
    pub fn with_user(mut self, user: &str) -> Self {
        self.user = Some(user.to_string());
        self
    }

    /// Add after dependency.
    pub fn after(mut self, unit: &str) -> Self {
        self.after.push(unit.to_string());
        self
    }

    /// Generate unit file content.
    pub fn generate(&self) -> String {
        let mut content = String::new();

        content.push_str("[Unit]\n");
        content.push_str(&format!("Description={}\n", self.description));
        for after in &self.after {
            content.push_str(&format!("After={}\n", after));
        }
        for wants in &self.wants {
            content.push_str(&format!("Wants={}\n", wants));
        }
        content.push('\n');

        content.push_str("[Service]\n");
        content.push_str(&format!("Type={}\n", self.service_type));
        if let Some(user) = &self.user {
            content.push_str(&format!("User={}\n", user));
        }
        if let Some(group) = &self.group {
            content.push_str(&format!("Group={}\n", group));
        }
        if let Some(wd) = &self.working_directory {
            content.push_str(&format!("WorkingDirectory={}\n", wd.display()));
        }
        content.push_str(&format!("ExecStart={}\n", self.exec_start));
        if let Some(exec_stop) = &self.exec_stop {
            content.push_str(&format!("ExecStop={}\n", exec_stop));
        }
        content.push_str(&format!("Restart={}\n", self.restart));
        content.push_str(&format!("RestartSec={}\n", self.restart_sec));
        for (key, value) in &self.environment {
            content.push_str(&format!("Environment={}={}\n", key, value));
        }
        content.push('\n');

        content.push_str("[Install]\n");
        content.push_str("WantedBy=multi-user.target\n");

        content
    }
}

/// Systemd service type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SystemdServiceType {
    /// Simple service
    Simple,
    /// Forking service
    Forking,
    /// Oneshot service
    Oneshot,
    /// Notify service
    Notify,
}

impl fmt::Display for SystemdServiceType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Simple => write!(f, "simple"),
            Self::Forking => write!(f, "forking"),
            Self::Oneshot => write!(f, "oneshot"),
            Self::Notify => write!(f, "notify"),
        }
    }
}

/// Systemd restart policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemdRestartPolicy {
    /// Never restart
    No,
    /// Restart on success
    OnSuccess,
    /// Restart on failure
    OnFailure,
    /// Restart on abnormal
    OnAbnormal,
    /// Always restart
    Always,
}

impl fmt::Display for SystemdRestartPolicy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::No => write!(f, "no"),
            Self::OnSuccess => write!(f, "on-success"),
            Self::OnFailure => write!(f, "on-failure"),
            Self::OnAbnormal => write!(f, "on-abnormal"),
            Self::Always => write!(f, "always"),
        }
    }
}

// ============================================================================
// Health Checks
// ============================================================================

/// Health check configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check type
    pub check_type: HealthCheckType,
    /// Interval between checks
    #[serde(with = "duration_serde")]
    pub interval: Duration,
    /// Timeout for each check
    #[serde(with = "duration_serde")]
    pub timeout: Duration,
    /// Number of retries
    pub retries: u32,
    /// Start period (grace period)
    #[serde(with = "duration_serde")]
    pub start_period: Duration,
}

impl HealthCheck {
    /// Create an HTTP health check.
    pub fn http(path: &str, port: u16, interval: Duration) -> Self {
        Self {
            check_type: HealthCheckType::Http {
                path: path.to_string(),
                port,
            },
            interval,
            timeout: Duration::from_secs(10),
            retries: 3,
            start_period: Duration::from_secs(30),
        }
    }

    /// Create a command health check.
    pub fn command(cmd: Vec<&str>, interval: Duration) -> Self {
        Self {
            check_type: HealthCheckType::Command {
                command: cmd.into_iter().map(String::from).collect(),
            },
            interval,
            timeout: Duration::from_secs(10),
            retries: 3,
            start_period: Duration::from_secs(30),
        }
    }

    /// Create a TCP health check.
    pub fn tcp(port: u16, interval: Duration) -> Self {
        Self {
            check_type: HealthCheckType::Tcp { port },
            interval,
            timeout: Duration::from_secs(10),
            retries: 3,
            start_period: Duration::from_secs(30),
        }
    }
}

/// Health check type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum HealthCheckType {
    /// HTTP endpoint check
    Http {
        /// Path to check
        path: String,
        /// Port
        port: u16,
    },
    /// Command execution
    Command {
        /// Command to run
        command: Vec<String>,
    },
    /// TCP connection check
    Tcp {
        /// Port
        port: u16,
    },
    /// JSON-RPC check
    JsonRpc {
        /// Endpoint URL
        url: String,
        /// Method to call
        method: String,
    },
}

mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(duration: &Duration, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Duration, D::Error> {
        let secs = u64::deserialize(d)?;
        Ok(Duration::from_secs(secs))
    }
}

/// Health status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Node is healthy
    Healthy,
    /// Node is unhealthy
    Unhealthy,
    /// Node is starting
    Starting,
    /// Status unknown
    Unknown,
}

impl fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Unhealthy => write!(f, "unhealthy"),
            Self::Starting => write!(f, "starting"),
            Self::Unknown => write!(f, "unknown"),
        }
    }
}

/// Health check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Status
    pub status: HealthStatus,
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Latency
    #[serde(with = "duration_serde")]
    pub latency: Duration,
    /// Error message (if unhealthy)
    pub error: Option<String>,
    /// Additional details
    pub details: HashMap<String, String>,
}

// ============================================================================
// Sync Status
// ============================================================================

/// Node sync status.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatus {
    /// Whether the node is syncing
    pub syncing: bool,
    /// Current block/height
    pub current_block: u64,
    /// Highest known block
    pub highest_block: u64,
    /// Starting block (for current sync)
    pub starting_block: Option<u64>,
    /// Sync percentage
    pub sync_percentage: f64,
    /// Estimated time remaining
    #[serde(with = "option_duration_serde")]
    pub estimated_remaining: Option<Duration>,
    /// Peers connected
    pub peers: u32,
}

impl SyncStatus {
    /// Check if sync is complete.
    pub fn is_synced(&self) -> bool {
        !self.syncing && self.current_block >= self.highest_block
    }

    /// Get blocks remaining.
    pub fn blocks_remaining(&self) -> u64 {
        self.highest_block.saturating_sub(self.current_block)
    }
}

mod option_duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S: Serializer>(duration: &Option<Duration>, s: S) -> Result<S::Ok, S::Error> {
        match duration {
            Some(d) => s.serialize_some(&d.as_secs()),
            None => s.serialize_none(),
        }
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Duration>, D::Error> {
        let secs: Option<u64> = Option::deserialize(d)?;
        Ok(secs.map(Duration::from_secs))
    }
}

// ============================================================================
// Log Discovery
// ============================================================================

/// Log file configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogConfig {
    /// Log file path
    pub path: PathBuf,
    /// Log format
    pub format: LogFormat,
    /// Log level filter
    pub level: LogLevel,
    /// Rotation policy
    pub rotation: Option<LogRotation>,
}

/// Log format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogFormat {
    /// Plain text
    Plain,
    /// JSON
    Json,
    /// Systemd journal
    Journal,
}

/// Log level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LogLevel {
    /// Trace level
    Trace,
    /// Debug level
    Debug,
    /// Info level
    Info,
    /// Warn level
    Warn,
    /// Error level
    Error,
}

/// Log rotation configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogRotation {
    /// Maximum file size in bytes
    pub max_size: u64,
    /// Maximum number of files to keep
    pub max_files: u32,
    /// Compress rotated files
    pub compress: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_type() {
        assert!(NodeType::Geth.is_eth_execution());
        assert!(!NodeType::Geth.is_eth_consensus());
        assert!(NodeType::Lighthouse.is_eth_consensus());
        assert!(NodeType::BitcoinCore.is_bitcoin());
    }

    #[test]
    fn test_docker_service() {
        let service = DockerService::new("geth", "ethereum/client-go:stable")
            .with_port(8545, 8545)
            .with_volume("./data", "/root/.ethereum")
            .with_env("NETWORK", "mainnet");

        assert_eq!(service.name, "geth");
        assert_eq!(service.ports.len(), 1);
        assert_eq!(service.volumes.len(), 1);
        assert!(service.environment.contains_key("NETWORK"));
    }

    #[test]
    fn test_docker_compose_template() {
        let template = DockerComposeTemplate::ethereum_full_node();
        assert_eq!(template.services.len(), 2);
    }

    #[test]
    fn test_systemd_unit() {
        let unit = SystemdUnit::new(
            "geth",
            "Go Ethereum",
            "/usr/local/bin/geth --syncmode snap",
        )
        .with_user("ethereum");

        let content = unit.generate();
        assert!(content.contains("Description=Go Ethereum"));
        assert!(content.contains("User=ethereum"));
    }

    #[test]
    fn test_health_check() {
        let check = HealthCheck::http("/health", 8545, Duration::from_secs(30));
        assert_eq!(check.retries, 3);
    }

    #[test]
    fn test_sync_status() {
        let status = SyncStatus {
            syncing: true,
            current_block: 1000,
            highest_block: 2000,
            starting_block: Some(0),
            sync_percentage: 50.0,
            estimated_remaining: Some(Duration::from_secs(3600)),
            peers: 10,
        };

        assert!(!status.is_synced());
        assert_eq!(status.blocks_remaining(), 1000);
    }
}
