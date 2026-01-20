//! DNS resolution and IP validation utilities.

use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use crate::error::{PermissionError, PermissionResult};

/// Check if an IP address is in a private range.
pub fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => is_private_ipv4(ipv4),
        IpAddr::V6(ipv6) => is_private_ipv6(ipv6),
    }
}

/// Check if an IPv4 address is in a private range.
pub fn is_private_ipv4(ip: &Ipv4Addr) -> bool {
    // 10.0.0.0/8
    if ip.octets()[0] == 10 {
        return true;
    }

    // 172.16.0.0/12
    if ip.octets()[0] == 172 && (ip.octets()[1] >= 16 && ip.octets()[1] <= 31) {
        return true;
    }

    // 192.168.0.0/16
    if ip.octets()[0] == 192 && ip.octets()[1] == 168 {
        return true;
    }

    // 127.0.0.0/8 (loopback)
    if ip.octets()[0] == 127 {
        return true;
    }

    // 169.254.0.0/16 (link-local)
    if ip.octets()[0] == 169 && ip.octets()[1] == 254 {
        return true;
    }

    // 100.64.0.0/10 (carrier-grade NAT)
    if ip.octets()[0] == 100 && (ip.octets()[1] >= 64 && ip.octets()[1] <= 127) {
        return true;
    }

    false
}

/// Check if an IPv6 address is in a private range.
pub fn is_private_ipv6(ip: &Ipv6Addr) -> bool {
    // ::1 (loopback)
    if ip.is_loopback() {
        return true;
    }

    // fe80::/10 (link-local)
    let segments = ip.segments();
    if segments[0] & 0xffc0 == 0xfe80 {
        return true;
    }

    // fc00::/7 (unique local address)
    if segments[0] & 0xfe00 == 0xfc00 {
        return true;
    }

    false
}

/// Check if an IP address is localhost.
pub fn is_localhost(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(ipv4) => ipv4.is_loopback(),
        IpAddr::V6(ipv6) => ipv6.is_loopback(),
    }
}

/// Domain matching for allowlist/denylist.
pub fn domain_matches(domain: &str, pattern: &str) -> bool {
    // Exact match
    if domain == pattern {
        return true;
    }

    // Wildcard subdomain match (*.example.com matches sub.example.com)
    if let Some(suffix) = pattern.strip_prefix("*.") {
        return domain.ends_with(suffix)
            && (domain.len() > suffix.len() + 1)
            && domain.as_bytes()[domain.len() - suffix.len() - 1] == b'.';
    }

    // Suffix match (example.com matches sub.example.com)
    if domain.ends_with(pattern) {
        let prefix_len = domain.len() - pattern.len();
        if prefix_len > 0 {
            return domain.as_bytes()[prefix_len - 1] == b'.';
        }
    }

    false
}

/// Extract the base domain from a full domain (e.g., sub.example.com -> example.com).
pub fn base_domain(domain: &str) -> &str {
    let parts: Vec<&str> = domain.split('.').collect();
    if parts.len() <= 2 {
        domain
    } else {
        let skip = parts.len() - 2;
        &domain[domain.len() - parts[skip..].join(".").len()..]
    }
}

/// Simple synchronous DNS resolver using system resolver.
pub struct SystemResolver;

impl SystemResolver {
    /// Resolve a domain to IP addresses.
    pub fn resolve(domain: &str) -> PermissionResult<Vec<IpAddr>> {
        use std::net::ToSocketAddrs;

        // Add a dummy port for resolution
        let addr = format!("{}:80", domain);

        let addrs = addr.to_socket_addrs().map_err(|e| {
            PermissionError::DnsResolutionFailed {
                domain: domain.to_string(),
                reason: e.to_string(),
            }
        })?;

        Ok(addrs.map(|a| a.ip()).collect())
    }

    /// Resolve and validate that no private IPs are returned.
    pub fn resolve_public_only(domain: &str) -> PermissionResult<Vec<IpAddr>> {
        let ips = Self::resolve(domain)?;

        for ip in &ips {
            if is_private_ip(ip) {
                return Err(PermissionError::PrivateIpDenied { ip: *ip });
            }
        }

        if ips.is_empty() {
            return Err(PermissionError::DnsResolutionFailed {
                domain: domain.to_string(),
                reason: "No addresses resolved".to_string(),
            });
        }

        Ok(ips)
    }
}

/// Async DNS resolver (requires async-runtime feature).
#[cfg(feature = "async-runtime")]
pub mod async_resolver {
    use super::*;

    /// Async DNS resolver.
    pub struct AsyncResolver;

    impl AsyncResolver {
        /// Resolve a domain asynchronously.
        pub async fn resolve(domain: &str) -> PermissionResult<Vec<IpAddr>> {
            // Use tokio's DNS resolution
            let domain = domain.to_string();
            let result = tokio::task::spawn_blocking(move || SystemResolver::resolve(&domain))
                .await
                .map_err(|e| PermissionError::DnsResolutionFailed {
                    domain: domain.clone(),
                    reason: e.to_string(),
                })??;

            Ok(result)
        }

        /// Resolve and validate that no private IPs are returned.
        pub async fn resolve_public_only(domain: &str) -> PermissionResult<Vec<IpAddr>> {
            let ips = Self::resolve(domain).await?;

            for ip in &ips {
                if is_private_ip(ip) {
                    return Err(PermissionError::PrivateIpDenied { ip: *ip });
                }
            }

            Ok(ips)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_private_ipv4() {
        // Private ranges
        assert!(is_private_ipv4(&Ipv4Addr::new(10, 0, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(172, 16, 0, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(192, 168, 1, 1)));
        assert!(is_private_ipv4(&Ipv4Addr::new(127, 0, 0, 1)));

        // Public
        assert!(!is_private_ipv4(&Ipv4Addr::new(8, 8, 8, 8)));
        assert!(!is_private_ipv4(&Ipv4Addr::new(1, 1, 1, 1)));
    }

    #[test]
    fn test_domain_matching() {
        // Exact match
        assert!(domain_matches("example.com", "example.com"));

        // Wildcard subdomain
        assert!(domain_matches("api.example.com", "*.example.com"));
        assert!(domain_matches("sub.api.example.com", "*.example.com"));
        assert!(!domain_matches("example.com", "*.example.com"));

        // Suffix match
        assert!(domain_matches("api.example.com", "example.com"));
        assert!(!domain_matches("notexample.com", "example.com"));
    }

    #[test]
    fn test_base_domain() {
        assert_eq!(base_domain("example.com"), "example.com");
        assert_eq!(base_domain("api.example.com"), "example.com");
        assert_eq!(base_domain("sub.api.example.com"), "example.com");
    }

    #[test]
    fn test_localhost() {
        assert!(is_localhost(&IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1))));
        assert!(is_localhost(&IpAddr::V6(Ipv6Addr::LOCALHOST)));
        assert!(!is_localhost(&IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8))));
    }
}
