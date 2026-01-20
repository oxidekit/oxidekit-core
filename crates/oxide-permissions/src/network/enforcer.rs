//! Network policy enforcement.

use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::{Arc, RwLock};

use crate::error::{PermissionError, PermissionResult};

use super::policy::{
    ConnectionRequest, NetworkDecision, NetworkDenyReason, NetworkMode, NetworkPolicy,
};
use super::resolver::{domain_matches, is_localhost, is_private_ip, SystemResolver};

/// Statistics about network policy enforcement.
#[derive(Debug, Clone, Default)]
pub struct NetworkStats {
    /// Total connection attempts.
    pub total_attempts: u64,
    /// Allowed connections.
    pub allowed: u64,
    /// Denied connections.
    pub denied: u64,
    /// Connections to unique domains.
    pub unique_domains: usize,
    /// Connection counts per domain.
    pub per_domain: HashMap<String, u64>,
    /// Denial reasons breakdown.
    pub deny_reasons: HashMap<String, u64>,
}

/// Connection log entry.
#[derive(Debug, Clone)]
pub struct ConnectionLogEntry {
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Target domain.
    pub domain: String,
    /// Target port.
    pub port: u16,
    /// Scheme.
    pub scheme: String,
    /// Decision.
    pub decision: NetworkDecision,
    /// Resolved IPs (if any).
    pub resolved_ips: Vec<IpAddr>,
}

/// Network policy enforcer.
pub struct NetworkEnforcer {
    /// The network policy.
    policy: NetworkPolicy,
    /// Enforcement statistics.
    stats: Arc<RwLock<NetworkStats>>,
    /// Connection log (if logging enabled).
    log: Arc<RwLock<Vec<ConnectionLogEntry>>>,
    /// Maximum log entries to keep.
    max_log_entries: usize,
    /// Whether to perform DNS resolution for private IP checks.
    resolve_for_private_check: bool,
}

impl NetworkEnforcer {
    /// Create a new network enforcer.
    pub fn new(policy: NetworkPolicy) -> Self {
        Self {
            policy,
            stats: Arc::new(RwLock::new(NetworkStats::default())),
            log: Arc::new(RwLock::new(Vec::new())),
            max_log_entries: 1000,
            resolve_for_private_check: true,
        }
    }

    /// Create an enforcer that allows all traffic.
    pub fn allow_all() -> Self {
        Self::new(NetworkPolicy::allow_all())
    }

    /// Create an enforcer with a strict allowlist.
    pub fn strict_allowlist(domains: Vec<String>) -> Self {
        Self::new(NetworkPolicy::strict_allowlist(domains))
    }

    /// Create an enforcer that blocks all network access.
    pub fn block_all() -> Self {
        Self::new(NetworkPolicy::block_all())
    }

    /// Set whether to resolve DNS for private IP checking.
    pub fn with_dns_resolution(mut self, enabled: bool) -> Self {
        self.resolve_for_private_check = enabled;
        self
    }

    /// Set maximum log entries.
    pub fn with_max_log(mut self, max: usize) -> Self {
        self.max_log_entries = max;
        self
    }

    /// Check if a connection is allowed.
    pub fn check(&self, request: &ConnectionRequest) -> NetworkDecision {
        let decision = self.evaluate(request);

        // Update statistics
        self.record_attempt(&request.domain, &decision);

        // Log if enabled
        if self.policy.log_connections {
            self.log_connection(request, &decision);
        }

        decision
    }

    /// Check a connection from a URL string.
    pub fn check_url(&self, url_str: &str) -> PermissionResult<NetworkDecision> {
        let url = url::Url::parse(url_str).map_err(|e| {
            PermissionError::InvalidNetworkPolicy(format!("Invalid URL: {}", e))
        })?;

        let request = ConnectionRequest::from_url(&url).ok_or_else(|| {
            PermissionError::InvalidNetworkPolicy("Cannot extract host from URL".to_string())
        })?;

        Ok(self.check(&request))
    }

    /// Evaluate a connection request against the policy.
    fn evaluate(&self, request: &ConnectionRequest) -> NetworkDecision {
        // Check if network is blocked entirely
        if self.policy.mode == NetworkMode::BlockAll {
            return NetworkDecision::Deny(NetworkDenyReason::NetworkBlocked);
        }

        // Check if network is unrestricted
        if self.policy.mode == NetworkMode::AllowAll {
            // Still check private ranges and localhost if configured
            if let Some(reason) = self.check_ip_restrictions(request) {
                return NetworkDecision::Deny(reason);
            }
            return NetworkDecision::Allow;
        }

        // Check HTTPS requirement
        if self.policy.require_https && !request.is_secure() {
            return NetworkDecision::Deny(NetworkDenyReason::HttpsRequired);
        }

        // Check port restrictions
        if !self.policy.is_port_allowed(request.port) {
            return NetworkDecision::Deny(NetworkDenyReason::PortNotAllowed(request.port));
        }

        // Check domain allowlist/denylist
        match self.policy.mode {
            NetworkMode::Allowlist => {
                if !self.is_domain_allowed(&request.domain) {
                    return NetworkDecision::Deny(NetworkDenyReason::DomainNotAllowed(
                        request.domain.clone(),
                    ));
                }
            }
            NetworkMode::Denylist => {
                if self.is_domain_denied(&request.domain) {
                    return NetworkDecision::Deny(NetworkDenyReason::DomainDenied(
                        request.domain.clone(),
                    ));
                }
            }
            _ => {}
        }

        // Check IP restrictions
        if let Some(reason) = self.check_ip_restrictions(request) {
            return NetworkDecision::Deny(reason);
        }

        NetworkDecision::Allow
    }

    /// Check if a domain is in the allowlist.
    fn is_domain_allowed(&self, domain: &str) -> bool {
        for allowed in &self.policy.allow {
            if domain_matches(domain, allowed) {
                return true;
            }
        }
        false
    }

    /// Check if a domain is in the denylist.
    fn is_domain_denied(&self, domain: &str) -> bool {
        for denied in &self.policy.deny {
            if domain_matches(domain, denied) {
                return true;
            }
        }
        false
    }

    /// Check IP-based restrictions (private ranges, localhost).
    fn check_ip_restrictions(&self, request: &ConnectionRequest) -> Option<NetworkDenyReason> {
        // Get IPs to check - either from request or resolve them
        let ips: Vec<IpAddr> = if !request.resolved_ips.is_empty() {
            request.resolved_ips.clone()
        } else if self.resolve_for_private_check && self.policy.deny_private_ranges {
            SystemResolver::resolve(&request.domain).unwrap_or_default()
        } else {
            Vec::new()
        };

        // Check each IP
        for ip in &ips {
            if self.policy.deny_localhost && is_localhost(ip) {
                return Some(NetworkDenyReason::LocalhostDenied);
            }

            if self.policy.deny_private_ranges && is_private_ip(ip) {
                return Some(NetworkDenyReason::PrivateIpDenied(*ip));
            }
        }

        None
    }

    /// Record a connection attempt in statistics.
    fn record_attempt(&self, domain: &str, decision: &NetworkDecision) {
        let mut stats = self.stats.write().unwrap();
        stats.total_attempts += 1;

        match decision {
            NetworkDecision::Allow => stats.allowed += 1,
            NetworkDecision::Deny(reason) => {
                stats.denied += 1;
                *stats
                    .deny_reasons
                    .entry(format!("{:?}", reason))
                    .or_insert(0) += 1;
            }
            NetworkDecision::Unknown => {}
        }

        let count = stats.per_domain.entry(domain.to_string()).or_insert(0);
        *count += 1;
        stats.unique_domains = stats.per_domain.len();
    }

    /// Log a connection attempt.
    fn log_connection(&self, request: &ConnectionRequest, decision: &NetworkDecision) {
        let entry = ConnectionLogEntry {
            timestamp: chrono::Utc::now(),
            domain: request.domain.clone(),
            port: request.port,
            scheme: request.scheme.clone(),
            decision: decision.clone(),
            resolved_ips: request.resolved_ips.clone(),
        };

        let mut log = self.log.write().unwrap();
        log.push(entry);

        // Trim log if too large
        if log.len() > self.max_log_entries {
            let excess = log.len() - self.max_log_entries;
            log.drain(0..excess);
        }
    }

    /// Get enforcement statistics.
    pub fn stats(&self) -> NetworkStats {
        self.stats.read().unwrap().clone()
    }

    /// Reset statistics.
    pub fn reset_stats(&self) {
        let mut stats = self.stats.write().unwrap();
        *stats = NetworkStats::default();
    }

    /// Get the connection log.
    pub fn connection_log(&self) -> Vec<ConnectionLogEntry> {
        self.log.read().unwrap().clone()
    }

    /// Clear the connection log.
    pub fn clear_log(&self) {
        self.log.write().unwrap().clear();
    }

    /// Get the current policy.
    pub fn policy(&self) -> &NetworkPolicy {
        &self.policy
    }

    /// Check if network enforcement is active.
    pub fn is_enforced(&self) -> bool {
        self.policy.is_enforced()
    }

    /// Get list of allowed domains (for display).
    pub fn allowed_domains(&self) -> Vec<&str> {
        self.policy.allow.iter().map(String::as_str).collect()
    }

    /// Generate a network report.
    pub fn generate_report(&self) -> NetworkReport {
        let stats = self.stats.read().unwrap();

        NetworkReport {
            mode: self.policy.mode,
            is_enforced: self.policy.is_enforced(),
            allowed_domains: self.policy.allow.clone(),
            denied_domains: self.policy.deny.clone(),
            deny_private_ranges: self.policy.deny_private_ranges,
            require_https: self.policy.require_https,
            total_connections: stats.total_attempts,
            allowed_connections: stats.allowed,
            denied_connections: stats.denied,
            unique_domains_contacted: stats.unique_domains,
            top_domains: stats
                .per_domain
                .iter()
                .take(10)
                .map(|(d, c)| (d.clone(), *c))
                .collect(),
        }
    }
}

/// Network enforcement report for disclosure.
#[derive(Debug, Clone)]
pub struct NetworkReport {
    /// Network mode.
    pub mode: NetworkMode,
    /// Whether enforcement is active.
    pub is_enforced: bool,
    /// Allowed domains.
    pub allowed_domains: Vec<String>,
    /// Denied domains.
    pub denied_domains: Vec<String>,
    /// Whether private ranges are denied.
    pub deny_private_ranges: bool,
    /// Whether HTTPS is required.
    pub require_https: bool,
    /// Total connection attempts.
    pub total_connections: u64,
    /// Allowed connections.
    pub allowed_connections: u64,
    /// Denied connections.
    pub denied_connections: u64,
    /// Unique domains contacted.
    pub unique_domains_contacted: usize,
    /// Top contacted domains.
    pub top_domains: Vec<(String, u64)>,
}

impl NetworkReport {
    /// Generate a human-readable summary.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        lines.push(format!("Network Policy: {:?}", self.mode));

        if self.is_enforced {
            lines.push("Status: ENFORCED".to_string());

            if !self.allowed_domains.is_empty() {
                lines.push(format!(
                    "Allowed domains: {}",
                    self.allowed_domains.join(", ")
                ));
            }

            if self.deny_private_ranges {
                lines.push("Private IP ranges: BLOCKED".to_string());
            }

            if self.require_https {
                lines.push("HTTPS: REQUIRED".to_string());
            }
        } else {
            lines.push("Status: NOT ENFORCED (unknown domains)".to_string());
        }

        lines.join("\n")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allowlist_enforcement() {
        let enforcer =
            NetworkEnforcer::strict_allowlist(vec!["api.example.com".to_string()]).with_dns_resolution(false);

        let allowed_request = ConnectionRequest::new("api.example.com", 443, "https");
        assert_eq!(enforcer.check(&allowed_request), NetworkDecision::Allow);

        let denied_request = ConnectionRequest::new("evil.com", 443, "https");
        assert!(matches!(
            enforcer.check(&denied_request),
            NetworkDecision::Deny(NetworkDenyReason::DomainNotAllowed(_))
        ));
    }

    #[test]
    fn test_https_requirement() {
        let enforcer =
            NetworkEnforcer::strict_allowlist(vec!["api.example.com".to_string()]).with_dns_resolution(false);

        let http_request = ConnectionRequest::new("api.example.com", 80, "http");
        assert!(matches!(
            enforcer.check(&http_request),
            NetworkDecision::Deny(NetworkDenyReason::HttpsRequired)
        ));
    }

    #[test]
    fn test_private_ip_blocking() {
        let mut policy = NetworkPolicy::allow_all();
        policy.deny_private_ranges = true;

        let enforcer = NetworkEnforcer::new(policy).with_dns_resolution(false);

        let request = ConnectionRequest::new("internal.local", 80, "http").with_ips(vec![
            std::net::IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1)),
        ]);

        assert!(matches!(
            enforcer.check(&request),
            NetworkDecision::Deny(NetworkDenyReason::PrivateIpDenied(_))
        ));
    }

    #[test]
    fn test_statistics() {
        let enforcer =
            NetworkEnforcer::strict_allowlist(vec!["api.example.com".to_string()]).with_dns_resolution(false);

        let _ = enforcer.check(&ConnectionRequest::new("api.example.com", 443, "https"));
        let _ = enforcer.check(&ConnectionRequest::new("evil.com", 443, "https"));
        let _ = enforcer.check(&ConnectionRequest::new("api.example.com", 443, "https"));

        let stats = enforcer.stats();
        assert_eq!(stats.total_attempts, 3);
        assert_eq!(stats.allowed, 2);
        assert_eq!(stats.denied, 1);
        assert_eq!(stats.unique_domains, 2);
    }

    #[test]
    fn test_block_all() {
        let enforcer = NetworkEnforcer::block_all();

        let request = ConnectionRequest::new("any.com", 443, "https");
        assert!(matches!(
            enforcer.check(&request),
            NetworkDecision::Deny(NetworkDenyReason::NetworkBlocked)
        ));
    }

    #[test]
    fn test_report() {
        let enforcer =
            NetworkEnforcer::strict_allowlist(vec!["api.example.com".to_string()]).with_dns_resolution(false);

        let _ = enforcer.check(&ConnectionRequest::new("api.example.com", 443, "https"));

        let report = enforcer.generate_report();
        assert!(report.is_enforced);
        assert!(report.allowed_domains.contains(&"api.example.com".to_string()));
    }
}
