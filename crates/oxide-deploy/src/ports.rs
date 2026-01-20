//! Port mapping and collision detection
//!
//! This module provides functionality for managing port mappings in OxideKit
//! applications and detecting port collisions.

use crate::error::{DeployError, DeployResult, ValidationCategory, ValidationError, ValidationErrors};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Standard port ranges
pub mod ranges {
    /// Well-known ports (0-1023) - require root/admin
    pub const WELL_KNOWN: std::ops::RangeInclusive<u16> = 0..=1023;
    /// Registered ports (1024-49151)
    pub const REGISTERED: std::ops::RangeInclusive<u16> = 1024..=49151;
    /// Dynamic/private ports (49152-65535)
    pub const DYNAMIC: std::ops::RangeInclusive<u16> = 49152..=65535;
}

/// Common default ports for various services
pub mod defaults {
    /// HTTP default port
    pub const HTTP: u16 = 80;
    /// HTTPS default port
    pub const HTTPS: u16 = 443;
    /// Development HTTP port
    pub const DEV_HTTP: u16 = 8080;
    /// Alternative development HTTP port
    pub const DEV_HTTP_ALT: u16 = 3000;
    /// PostgreSQL default port
    pub const POSTGRES: u16 = 5432;
    /// MySQL default port
    pub const MYSQL: u16 = 3306;
    /// Redis default port
    pub const REDIS: u16 = 6379;
    /// MongoDB default port
    pub const MONGODB: u16 = 27017;
    /// Prometheus metrics port
    pub const METRICS: u16 = 9090;
    /// gRPC default port
    pub const GRPC: u16 = 50051;
}

/// A port mapping entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    /// Service name
    pub service: String,
    /// Port number
    pub port: u16,
    /// Protocol (tcp/udp)
    #[serde(default = "default_protocol")]
    pub protocol: Protocol,
    /// Description of what this port is used for
    #[serde(default)]
    pub description: Option<String>,
    /// Whether this port is exposed externally
    #[serde(default)]
    pub external: bool,
    /// Environment variable to override this port
    #[serde(default)]
    pub env_var: Option<String>,
}

fn default_protocol() -> Protocol {
    Protocol::Tcp
}

impl PortMapping {
    /// Create a new port mapping
    pub fn new(service: impl Into<String>, port: u16) -> Self {
        Self {
            service: service.into(),
            port,
            protocol: Protocol::Tcp,
            description: None,
            external: false,
            env_var: None,
        }
    }

    /// Set the protocol
    pub fn with_protocol(mut self, protocol: Protocol) -> Self {
        self.protocol = protocol;
        self
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Mark as externally exposed
    pub fn external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Set the environment variable override
    pub fn with_env_var(mut self, env_var: impl Into<String>) -> Self {
        self.env_var = Some(env_var.into());
        self
    }

    /// Get the effective port (from env var if set, otherwise default)
    pub fn effective_port(&self) -> u16 {
        if let Some(ref env_var) = self.env_var {
            if let Ok(value) = std::env::var(env_var) {
                if let Ok(port) = value.parse() {
                    return port;
                }
            }
        }
        self.port
    }
}

/// Protocol type for port mapping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
    /// Both TCP and UDP
    Both,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::Tcp
    }
}

impl std::fmt::Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tcp => write!(f, "tcp"),
            Self::Udp => write!(f, "udp"),
            Self::Both => write!(f, "tcp+udp"),
        }
    }
}

/// Port configuration for an application
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PortConfig {
    /// Port mappings
    #[serde(default)]
    pub ports: Vec<PortMapping>,
}

impl PortConfig {
    /// Create a new empty port configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a port mapping
    pub fn add(&mut self, mapping: PortMapping) {
        self.ports.push(mapping);
    }

    /// Get a port mapping by service name
    pub fn get(&self, service: &str) -> Option<&PortMapping> {
        self.ports.iter().find(|p| p.service == service)
    }

    /// Get all external ports
    pub fn external_ports(&self) -> Vec<&PortMapping> {
        self.ports.iter().filter(|p| p.external).collect()
    }

    /// Load configuration from a TOML file
    pub fn from_file(path: impl AsRef<Path>) -> DeployResult<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Err(DeployError::ConfigNotFound(path.to_path_buf()));
        }
        let content = std::fs::read_to_string(path)?;
        Self::from_toml(&content)
    }

    /// Parse configuration from TOML string
    pub fn from_toml(content: &str) -> DeployResult<Self> {
        Ok(toml::from_str(content)?)
    }

    /// Serialize configuration to TOML
    pub fn to_toml(&self) -> DeployResult<String> {
        Ok(toml::to_string_pretty(self)?)
    }

    /// Detect port collisions within this configuration
    pub fn detect_collisions(&self) -> Vec<PortCollision> {
        let mut port_services: HashMap<(u16, Protocol), Vec<String>> = HashMap::new();

        for mapping in &self.ports {
            let key = (mapping.effective_port(), mapping.protocol);
            port_services
                .entry(key)
                .or_default()
                .push(mapping.service.clone());
        }

        port_services
            .into_iter()
            .filter(|(_, services)| services.len() > 1)
            .map(|((port, protocol), services)| PortCollision {
                port,
                protocol,
                services,
            })
            .collect()
    }

    /// Validate the port configuration
    pub fn validate(&self) -> ValidationErrors {
        let mut errors = ValidationErrors::new();

        // Check for collisions
        let collisions = self.detect_collisions();
        for collision in collisions {
            errors.push(ValidationError::new(
                ValidationCategory::Port,
                format!(
                    "Port {} ({}) is used by multiple services: {}",
                    collision.port,
                    collision.protocol,
                    collision.services.join(", ")
                ),
            ));
        }

        // Check for well-known ports (might need privileges)
        for mapping in &self.ports {
            let port = mapping.effective_port();
            if ranges::WELL_KNOWN.contains(&port) {
                errors.push(ValidationError::new(
                    ValidationCategory::Port,
                    format!(
                        "Service '{}' uses well-known port {} which requires elevated privileges",
                        mapping.service, port
                    ),
                ));
            }
        }

        errors
    }

    /// Check if a specific port is available (not used in config)
    pub fn is_port_available(&self, port: u16) -> bool {
        !self.ports.iter().any(|p| p.effective_port() == port)
    }

    /// Find an available port starting from a given port
    pub fn find_available_port(&self, start: u16) -> Option<u16> {
        let used: HashSet<u16> = self.ports.iter().map(|p| p.effective_port()).collect();
        (start..=65535).find(|p| !used.contains(p))
    }

    /// Generate docker-compose ports section
    pub fn generate_docker_compose_ports(&self) -> String {
        let mut lines = Vec::new();
        lines.push("    ports:".to_string());
        for mapping in self.external_ports() {
            let port = mapping.effective_port();
            lines.push(format!("      - \"{}:{}\"", port, port));
        }
        lines.join("\n")
    }
}

/// Represents a port collision
#[derive(Debug, Clone)]
pub struct PortCollision {
    /// The colliding port number
    pub port: u16,
    /// The protocol
    pub protocol: Protocol,
    /// Services using this port
    pub services: Vec<String>,
}

impl std::fmt::Display for PortCollision {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Port {} ({}) collision: {}",
            self.port,
            self.protocol,
            self.services.join(", ")
        )
    }
}

/// Builder for port configuration
pub struct PortConfigBuilder {
    config: PortConfig,
}

impl PortConfigBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: PortConfig::new(),
        }
    }

    /// Add an HTTP port
    pub fn http(mut self, port: u16) -> Self {
        self.config.add(
            PortMapping::new("http", port)
                .with_description("HTTP server")
                .with_env_var("PORT")
                .external(),
        );
        self
    }

    /// Add an HTTPS port
    pub fn https(mut self, port: u16) -> Self {
        self.config.add(
            PortMapping::new("https", port)
                .with_description("HTTPS server")
                .with_env_var("HTTPS_PORT")
                .external(),
        );
        self
    }

    /// Add a metrics port
    pub fn metrics(mut self, port: u16) -> Self {
        self.config.add(
            PortMapping::new("metrics", port)
                .with_description("Prometheus metrics endpoint")
                .with_env_var("METRICS_PORT"),
        );
        self
    }

    /// Add a gRPC port
    pub fn grpc(mut self, port: u16) -> Self {
        self.config.add(
            PortMapping::new("grpc", port)
                .with_description("gRPC server")
                .with_env_var("GRPC_PORT")
                .external(),
        );
        self
    }

    /// Add a database port
    pub fn database(mut self, name: &str, port: u16) -> Self {
        self.config.add(
            PortMapping::new(name, port)
                .with_description(format!("{} database", name)),
        );
        self
    }

    /// Add a custom port mapping
    pub fn custom(mut self, mapping: PortMapping) -> Self {
        self.config.add(mapping);
        self
    }

    /// Build the configuration
    pub fn build(self) -> PortConfig {
        self.config
    }
}

impl Default for PortConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Check if a port is currently in use on the system
#[cfg(not(target_arch = "wasm32"))]
pub fn is_port_in_use(port: u16) -> bool {
    use std::net::{TcpListener, SocketAddr, IpAddr, Ipv4Addr};
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    TcpListener::bind(addr).is_err()
}

#[cfg(target_arch = "wasm32")]
pub fn is_port_in_use(_port: u16) -> bool {
    false
}

/// Find an available port on the system starting from a given port
#[cfg(not(target_arch = "wasm32"))]
pub fn find_available_system_port(start: u16) -> Option<u16> {
    (start..=65535).find(|&p| !is_port_in_use(p))
}

#[cfg(target_arch = "wasm32")]
pub fn find_available_system_port(start: u16) -> Option<u16> {
    Some(start)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_port_mapping_creation() {
        let mapping = PortMapping::new("http", 8080)
            .with_protocol(Protocol::Tcp)
            .with_description("HTTP server")
            .external()
            .with_env_var("PORT");

        assert_eq!(mapping.service, "http");
        assert_eq!(mapping.port, 8080);
        assert!(mapping.external);
        assert_eq!(mapping.env_var, Some("PORT".to_string()));
    }

    #[test]
    fn test_port_config_collision_detection() {
        let mut config = PortConfig::new();
        config.add(PortMapping::new("service1", 8080));
        config.add(PortMapping::new("service2", 8080));
        config.add(PortMapping::new("service3", 9090));

        let collisions = config.detect_collisions();
        assert_eq!(collisions.len(), 1);
        assert_eq!(collisions[0].port, 8080);
        assert_eq!(collisions[0].services.len(), 2);
    }

    #[test]
    fn test_port_config_no_collision() {
        let mut config = PortConfig::new();
        config.add(PortMapping::new("service1", 8080));
        config.add(PortMapping::new("service2", 9090));
        config.add(PortMapping::new("service3", 3000));

        let collisions = config.detect_collisions();
        assert!(collisions.is_empty());
    }

    #[test]
    fn test_port_config_builder() {
        let config = PortConfigBuilder::new()
            .http(8080)
            .https(8443)
            .metrics(9090)
            .database("postgres", 5432)
            .build();

        assert_eq!(config.ports.len(), 4);
        assert!(config.get("http").is_some());
        assert!(config.get("https").is_some());
        assert!(config.get("metrics").is_some());
        assert!(config.get("postgres").is_some());
    }

    #[test]
    fn test_port_config_toml_roundtrip() {
        let config = PortConfigBuilder::new()
            .http(8080)
            .metrics(9090)
            .build();

        let toml = config.to_toml().unwrap();
        let parsed = PortConfig::from_toml(&toml).unwrap();

        assert_eq!(parsed.ports.len(), config.ports.len());
    }

    #[test]
    fn test_external_ports() {
        let config = PortConfigBuilder::new()
            .http(8080)  // external
            .metrics(9090)  // internal
            .build();

        let external = config.external_ports();
        assert_eq!(external.len(), 1);
        assert_eq!(external[0].service, "http");
    }

    #[test]
    fn test_find_available_port() {
        let mut config = PortConfig::new();
        config.add(PortMapping::new("s1", 8080));
        config.add(PortMapping::new("s2", 8081));
        config.add(PortMapping::new("s3", 8082));

        assert!(!config.is_port_available(8080));
        assert!(config.is_port_available(8083));
        assert_eq!(config.find_available_port(8080), Some(8083));
    }

    #[test]
    fn test_protocol_display() {
        assert_eq!(format!("{}", Protocol::Tcp), "tcp");
        assert_eq!(format!("{}", Protocol::Udp), "udp");
        assert_eq!(format!("{}", Protocol::Both), "tcp+udp");
    }

    #[test]
    fn test_validation_warns_on_well_known_ports() {
        let mut config = PortConfig::new();
        config.add(PortMapping::new("http", 80));  // Well-known port

        let errors = config.validate();
        assert!(errors.has_errors());
        assert!(errors.errors.iter().any(|e| e.message.contains("elevated privileges")));
    }

    #[test]
    fn test_docker_compose_ports_generation() {
        let config = PortConfigBuilder::new()
            .http(8080)
            .https(8443)
            .build();

        let output = config.generate_docker_compose_ports();
        assert!(output.contains("8080:8080"));
        assert!(output.contains("8443:8443"));
    }
}
