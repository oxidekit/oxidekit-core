//! Network policy configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Network access mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NetworkMode {
    /// Allow all outbound connections (no enforcement).
    #[default]
    AllowAll,
    /// Only allow connections to domains in the allowlist.
    Allowlist,
    /// Allow all except domains in the denylist.
    Denylist,
    /// Block all network access.
    BlockAll,
}

/// Network policy configuration from oxide.toml.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkPolicy {
    /// Network access mode.
    #[serde(default)]
    pub mode: NetworkMode,

    /// Allowed domains (for allowlist mode).
    #[serde(default)]
    pub allow: Vec<String>,

    /// Denied domains (for denylist mode).
    #[serde(default)]
    pub deny: Vec<String>,

    /// Whether to deny access to private IP ranges.
    #[serde(default = "default_true")]
    pub deny_private_ranges: bool,

    /// Whether to deny access to localhost.
    #[serde(default = "default_true")]
    pub deny_localhost: bool,

    /// Whether HTTPS is required (no plain HTTP).
    #[serde(default)]
    pub require_https: bool,

    /// Allowed ports (empty = all ports allowed).
    #[serde(default)]
    pub allowed_ports: Vec<u16>,

    /// Certificate pinning configuration.
    #[serde(default)]
    pub cert_pins: Vec<CertificatePin>,

    /// Whether to log all network connections.
    #[serde(default)]
    pub log_connections: bool,
}

fn default_true() -> bool {
    true
}

impl NetworkPolicy {
    /// Create a new policy that allows all traffic.
    pub fn allow_all() -> Self {
        Self {
            mode: NetworkMode::AllowAll,
            deny_private_ranges: false,
            deny_localhost: false,
            ..Default::default()
        }
    }

    /// Create a new strict allowlist policy.
    pub fn strict_allowlist(domains: Vec<String>) -> Self {
        Self {
            mode: NetworkMode::Allowlist,
            allow: domains,
            deny_private_ranges: true,
            deny_localhost: true,
            require_https: true,
            ..Default::default()
        }
    }

    /// Create a policy that blocks all network access.
    pub fn block_all() -> Self {
        Self {
            mode: NetworkMode::BlockAll,
            ..Default::default()
        }
    }

    /// Check if the policy is in allowlist mode.
    pub fn is_allowlist_mode(&self) -> bool {
        self.mode == NetworkMode::Allowlist
    }

    /// Check if the policy is enforced (not allow-all).
    pub fn is_enforced(&self) -> bool {
        self.mode != NetworkMode::AllowAll
    }

    /// Add a domain to the allowlist.
    pub fn allow_domain(&mut self, domain: impl Into<String>) {
        self.allow.push(domain.into());
    }

    /// Add a domain to the denylist.
    pub fn deny_domain(&mut self, domain: impl Into<String>) {
        self.deny.push(domain.into());
    }

    /// Add an allowed port.
    pub fn allow_port(&mut self, port: u16) {
        if !self.allowed_ports.contains(&port) {
            self.allowed_ports.push(port);
        }
    }

    /// Add a certificate pin.
    pub fn add_cert_pin(&mut self, pin: CertificatePin) {
        self.cert_pins.push(pin);
    }

    /// Get unique domains from the allowlist.
    pub fn allowed_domains(&self) -> HashSet<&str> {
        self.allow.iter().map(String::as_str).collect()
    }

    /// Get unique domains from the denylist.
    pub fn denied_domains(&self) -> HashSet<&str> {
        self.deny.iter().map(String::as_str).collect()
    }

    /// Check if a port is allowed.
    pub fn is_port_allowed(&self, port: u16) -> bool {
        self.allowed_ports.is_empty() || self.allowed_ports.contains(&port)
    }

    /// Get certificate pin for a domain if configured.
    pub fn get_cert_pin(&self, domain: &str) -> Option<&CertificatePin> {
        self.cert_pins.iter().find(|pin| pin.domain == domain)
    }
}

/// Certificate pinning configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificatePin {
    /// Domain to pin.
    pub domain: String,
    /// SHA-256 hash of the public key (base64 encoded).
    pub public_key_hash: String,
    /// Backup pins for rotation.
    #[serde(default)]
    pub backup_pins: Vec<String>,
    /// Whether to enforce pinning (false = report only).
    #[serde(default = "default_true")]
    pub enforce: bool,
}

impl CertificatePin {
    /// Create a new certificate pin.
    pub fn new(domain: impl Into<String>, public_key_hash: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            public_key_hash: public_key_hash.into(),
            backup_pins: Vec::new(),
            enforce: true,
        }
    }

    /// Add a backup pin.
    pub fn with_backup(mut self, backup: impl Into<String>) -> Self {
        self.backup_pins.push(backup.into());
        self
    }

    /// Set enforcement mode.
    pub fn enforced(mut self, enforce: bool) -> Self {
        self.enforce = enforce;
        self
    }
}

/// Connection request for policy checking.
#[derive(Debug, Clone)]
pub struct ConnectionRequest {
    /// Target domain.
    pub domain: String,
    /// Target port.
    pub port: u16,
    /// Protocol scheme (http, https, ws, wss).
    pub scheme: String,
    /// Resolved IP addresses (if available).
    pub resolved_ips: Vec<std::net::IpAddr>,
}

impl ConnectionRequest {
    /// Create a new connection request.
    pub fn new(domain: impl Into<String>, port: u16, scheme: impl Into<String>) -> Self {
        Self {
            domain: domain.into(),
            port,
            scheme: scheme.into(),
            resolved_ips: Vec::new(),
        }
    }

    /// Create from a URL.
    pub fn from_url(url: &url::Url) -> Option<Self> {
        let domain = url.host_str()?.to_string();
        let port = url.port_or_known_default()?;
        let scheme = url.scheme().to_string();
        Some(Self::new(domain, port, scheme))
    }

    /// Add resolved IP addresses.
    pub fn with_ips(mut self, ips: Vec<std::net::IpAddr>) -> Self {
        self.resolved_ips = ips;
        self
    }

    /// Check if using secure scheme.
    pub fn is_secure(&self) -> bool {
        matches!(self.scheme.as_str(), "https" | "wss")
    }
}

/// Result of a network policy check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDecision {
    /// Connection is allowed.
    Allow,
    /// Connection is denied with reason.
    Deny(NetworkDenyReason),
    /// Policy not enforced (unknown status).
    Unknown,
}

/// Reason for denying a network connection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkDenyReason {
    /// Domain not in allowlist.
    DomainNotAllowed(String),
    /// Domain is in denylist.
    DomainDenied(String),
    /// Network access blocked entirely.
    NetworkBlocked,
    /// Private IP address detected.
    PrivateIpDenied(std::net::IpAddr),
    /// Localhost access denied.
    LocalhostDenied,
    /// HTTPS required but HTTP used.
    HttpsRequired,
    /// Port not allowed.
    PortNotAllowed(u16),
    /// Certificate pin mismatch.
    CertificatePinMismatch(String),
}

impl std::fmt::Display for NetworkDenyReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DomainNotAllowed(d) => write!(f, "domain '{}' not in allowlist", d),
            Self::DomainDenied(d) => write!(f, "domain '{}' is denied", d),
            Self::NetworkBlocked => write!(f, "network access is blocked"),
            Self::PrivateIpDenied(ip) => write!(f, "private IP {} is not allowed", ip),
            Self::LocalhostDenied => write!(f, "localhost access is denied"),
            Self::HttpsRequired => write!(f, "HTTPS is required"),
            Self::PortNotAllowed(p) => write!(f, "port {} is not allowed", p),
            Self::CertificatePinMismatch(d) => write!(f, "certificate pin mismatch for '{}'", d),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_policy_creation() {
        let policy = NetworkPolicy::strict_allowlist(vec![
            "api.example.com".to_string(),
            "cdn.example.com".to_string(),
        ]);

        assert!(policy.is_allowlist_mode());
        assert!(policy.is_enforced());
        assert!(policy.deny_private_ranges);
        assert!(policy.require_https);

        let domains = policy.allowed_domains();
        assert!(domains.contains("api.example.com"));
        assert!(domains.contains("cdn.example.com"));
    }

    #[test]
    fn test_allow_all_policy() {
        let policy = NetworkPolicy::allow_all();
        assert!(!policy.is_enforced());
        assert_eq!(policy.mode, NetworkMode::AllowAll);
    }

    #[test]
    fn test_connection_request() {
        let url = url::Url::parse("https://api.example.com:443/path").unwrap();
        let request = ConnectionRequest::from_url(&url).unwrap();

        assert_eq!(request.domain, "api.example.com");
        assert_eq!(request.port, 443);
        assert!(request.is_secure());
    }

    #[test]
    fn test_port_allowlist() {
        let mut policy = NetworkPolicy::default();
        policy.allow_port(443);
        policy.allow_port(8080);

        assert!(policy.is_port_allowed(443));
        assert!(policy.is_port_allowed(8080));
        assert!(!policy.is_port_allowed(80));
    }
}
