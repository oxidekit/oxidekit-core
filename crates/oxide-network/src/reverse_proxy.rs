//! Production reverse proxy configuration generation for OxideKit applications.
//!
//! This module generates configuration templates for popular reverse proxy servers
//! to enable same-origin deployments that eliminate CORS issues in production.
//!
//! # Supported Platforms
//!
//! - **Nginx**: Full configuration with upstream, location blocks, and WebSocket support
//! - **Caddy**: Caddyfile format with reverse_proxy directives
//! - **Traefik**: Docker labels or file-based configuration
//!
//! # Why Same-Origin?
//!
//! Same-origin deployment eliminates CORS by serving both frontend and API from
//! the same domain. The reverse proxy routes requests based on path:
//!
//! ```text
//! https://app.example.com/          -> Frontend (static files or SSR)
//! https://app.example.com/api/*     -> Backend API
//! https://app.example.com/ws/*      -> WebSocket server
//! ```
//!
//! # Example
//!
//! ```rust
//! use oxide_network::reverse_proxy::{ReverseProxyConfig, ProxyServer};
//!
//! let config = ReverseProxyConfig::new("https://app.example.com")
//!     .add_upstream("api", "http://api-server:8000", "/api")
//!     .add_upstream("ws", "http://ws-server:8001", "/ws")
//!     .frontend_upstream("http://frontend:3000");
//!
//! let nginx_config = config.generate(ProxyServer::Nginx);
//! println!("{}", nginx_config);
//! ```
//!
//! # CLI Integration
//!
//! ```bash
//! oxide network generate-proxy --target nginx --api https://api.example.com --site https://app.example.com
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

/// Supported reverse proxy servers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ProxyServer {
    /// Nginx web server.
    Nginx,
    /// Caddy web server.
    Caddy,
    /// Traefik reverse proxy.
    Traefik,
    /// Cloudflare Workers.
    CloudflareWorkers,
}

impl fmt::Display for ProxyServer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProxyServer::Nginx => write!(f, "nginx"),
            ProxyServer::Caddy => write!(f, "caddy"),
            ProxyServer::Traefik => write!(f, "traefik"),
            ProxyServer::CloudflareWorkers => write!(f, "cloudflare-workers"),
        }
    }
}

impl std::str::FromStr for ProxyServer {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nginx" => Ok(ProxyServer::Nginx),
            "caddy" => Ok(ProxyServer::Caddy),
            "traefik" => Ok(ProxyServer::Traefik),
            "cloudflare-workers" | "cloudflare" | "cf-workers" => Ok(ProxyServer::CloudflareWorkers),
            _ => Err(format!(
                "Unknown proxy server '{}'. Supported: nginx, caddy, traefik, cloudflare-workers",
                s
            )),
        }
    }
}

/// An upstream server configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Upstream {
    /// Name of the upstream (e.g., "api", "backend").
    pub name: String,
    /// Backend server URL.
    pub url: String,
    /// Path prefix to route to this upstream.
    pub path_prefix: String,
    /// Whether to strip the path prefix when forwarding.
    pub strip_prefix: bool,
    /// Whether this upstream handles WebSocket connections.
    pub websocket: bool,
    /// Health check path (optional).
    pub health_check: Option<String>,
    /// Request timeout.
    pub timeout_secs: u32,
    /// Custom headers to set.
    #[serde(default)]
    pub headers: HashMap<String, String>,
}

impl Upstream {
    /// Create a new upstream configuration.
    pub fn new(name: impl Into<String>, url: impl Into<String>, path_prefix: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            url: url.into(),
            path_prefix: path_prefix.into(),
            strip_prefix: true,
            websocket: false,
            health_check: None,
            timeout_secs: 30,
            headers: HashMap::new(),
        }
    }

    /// Configure WebSocket support.
    pub fn with_websocket(mut self, enabled: bool) -> Self {
        self.websocket = enabled;
        self
    }

    /// Set health check path.
    pub fn with_health_check(mut self, path: impl Into<String>) -> Self {
        self.health_check = Some(path.into());
        self
    }

    /// Set request timeout.
    pub fn with_timeout(mut self, secs: u32) -> Self {
        self.timeout_secs = secs;
        self
    }

    /// Disable prefix stripping.
    pub fn keep_prefix(mut self) -> Self {
        self.strip_prefix = false;
        self
    }

    /// Add a custom header.
    pub fn with_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }
}

/// SSL/TLS configuration for the reverse proxy.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SslConfig {
    /// Enable SSL/TLS.
    pub enabled: bool,
    /// Path to certificate file.
    pub cert_path: Option<String>,
    /// Path to private key file.
    pub key_path: Option<String>,
    /// Enable automatic HTTPS (Let's Encrypt).
    pub auto_https: bool,
    /// Minimum TLS version.
    pub min_tls_version: Option<String>,
    /// Enable HSTS.
    pub hsts: bool,
    /// HSTS max-age in seconds.
    pub hsts_max_age: u64,
}

impl SslConfig {
    /// Create SSL config with automatic HTTPS.
    pub fn auto() -> Self {
        Self {
            enabled: true,
            auto_https: true,
            hsts: true,
            hsts_max_age: 31536000, // 1 year
            ..Default::default()
        }
    }

    /// Create SSL config with manual certificates.
    pub fn manual(cert_path: impl Into<String>, key_path: impl Into<String>) -> Self {
        Self {
            enabled: true,
            cert_path: Some(cert_path.into()),
            key_path: Some(key_path.into()),
            auto_https: false,
            hsts: true,
            hsts_max_age: 31536000,
            ..Default::default()
        }
    }
}

/// Security headers configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityHeaders {
    /// X-Frame-Options header value.
    pub x_frame_options: Option<String>,
    /// X-Content-Type-Options header value.
    pub x_content_type_options: Option<String>,
    /// X-XSS-Protection header value.
    pub x_xss_protection: Option<String>,
    /// Referrer-Policy header value.
    pub referrer_policy: Option<String>,
    /// Content-Security-Policy header value.
    pub content_security_policy: Option<String>,
    /// Permissions-Policy header value.
    pub permissions_policy: Option<String>,
}

impl Default for SecurityHeaders {
    fn default() -> Self {
        Self {
            x_frame_options: Some("SAMEORIGIN".to_string()),
            x_content_type_options: Some("nosniff".to_string()),
            x_xss_protection: Some("1; mode=block".to_string()),
            referrer_policy: Some("strict-origin-when-cross-origin".to_string()),
            content_security_policy: None, // App-specific
            permissions_policy: None,       // App-specific
        }
    }
}

/// Configuration for generating reverse proxy configs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReverseProxyConfig {
    /// The production domain/origin (e.g., "https://app.example.com").
    pub site_origin: String,
    /// Upstream servers.
    pub upstreams: Vec<Upstream>,
    /// Frontend upstream (for serving the app).
    pub frontend_upstream: Option<String>,
    /// SSL configuration.
    pub ssl: SslConfig,
    /// Security headers.
    pub security_headers: SecurityHeaders,
    /// Enable gzip compression.
    pub gzip: bool,
    /// Enable access logging.
    pub access_log: bool,
    /// Custom server name (for nginx).
    pub server_name: Option<String>,
    /// Additional configuration snippets.
    #[serde(default)]
    pub extra_config: HashMap<String, String>,
}

impl ReverseProxyConfig {
    /// Create a new reverse proxy configuration.
    pub fn new(site_origin: impl Into<String>) -> Self {
        Self {
            site_origin: site_origin.into(),
            upstreams: Vec::new(),
            frontend_upstream: None,
            ssl: SslConfig::default(),
            security_headers: SecurityHeaders::default(),
            gzip: true,
            access_log: true,
            server_name: None,
            extra_config: HashMap::new(),
        }
    }

    /// Add an upstream server.
    pub fn add_upstream(
        mut self,
        name: impl Into<String>,
        url: impl Into<String>,
        path_prefix: impl Into<String>,
    ) -> Self {
        self.upstreams.push(Upstream::new(name, url, path_prefix));
        self
    }

    /// Add a configured upstream.
    pub fn with_upstream(mut self, upstream: Upstream) -> Self {
        self.upstreams.push(upstream);
        self
    }

    /// Set the frontend upstream.
    pub fn frontend_upstream(mut self, url: impl Into<String>) -> Self {
        self.frontend_upstream = Some(url.into());
        self
    }

    /// Configure SSL.
    pub fn with_ssl(mut self, ssl: SslConfig) -> Self {
        self.ssl = ssl;
        self
    }

    /// Enable automatic HTTPS.
    pub fn auto_https(mut self) -> Self {
        self.ssl = SslConfig::auto();
        self
    }

    /// Set security headers.
    pub fn with_security_headers(mut self, headers: SecurityHeaders) -> Self {
        self.security_headers = headers;
        self
    }

    /// Generate configuration for the specified proxy server.
    pub fn generate(&self, server: ProxyServer) -> String {
        match server {
            ProxyServer::Nginx => self.generate_nginx(),
            ProxyServer::Caddy => self.generate_caddy(),
            ProxyServer::Traefik => self.generate_traefik(),
            ProxyServer::CloudflareWorkers => self.generate_cloudflare_workers(),
        }
    }

    /// Generate configuration files with documentation.
    pub fn generate_with_docs(&self, server: ProxyServer) -> GeneratedConfig {
        let config = self.generate(server);
        let explanation = self.generate_explanation(server);
        let filename = match server {
            ProxyServer::Nginx => "nginx.conf".to_string(),
            ProxyServer::Caddy => "Caddyfile".to_string(),
            ProxyServer::Traefik => "traefik.yml".to_string(),
            ProxyServer::CloudflareWorkers => "worker.js".to_string(),
        };

        GeneratedConfig {
            server,
            filename,
            config,
            explanation,
        }
    }

    fn generate_explanation(&self, server: ProxyServer) -> String {
        let mut lines = vec![
            format!("# {} Reverse Proxy Configuration", server),
            "".to_string(),
            "## Why Same-Origin?".to_string(),
            "".to_string(),
            "This configuration eliminates CORS issues by serving both your frontend".to_string(),
            "and API from the same origin. Browser security policies only apply to".to_string(),
            "cross-origin requests, so same-origin requests bypass CORS entirely.".to_string(),
            "".to_string(),
            "## How It Works".to_string(),
            "".to_string(),
            format!("All requests to {} are routed as follows:", self.site_origin),
            "".to_string(),
        ];

        for upstream in &self.upstreams {
            let ws_note = if upstream.websocket { " (with WebSocket support)" } else { "" };
            lines.push(format!(
                "- {}/* -> {}{}",
                upstream.path_prefix, upstream.url, ws_note
            ));
        }

        if let Some(frontend) = &self.frontend_upstream {
            lines.push(format!("- /* (everything else) -> {}", frontend));
        }

        lines.push("".to_string());
        lines.push("## Recommended Headers".to_string());
        lines.push("".to_string());
        lines.push("The following security headers are configured:".to_string());
        lines.push("".to_string());

        if let Some(ref h) = self.security_headers.x_frame_options {
            lines.push(format!("- X-Frame-Options: {}", h));
        }
        if let Some(ref h) = self.security_headers.x_content_type_options {
            lines.push(format!("- X-Content-Type-Options: {}", h));
        }
        if let Some(ref h) = self.security_headers.referrer_policy {
            lines.push(format!("- Referrer-Policy: {}", h));
        }
        if self.ssl.hsts {
            lines.push(format!(
                "- Strict-Transport-Security: max-age={}; includeSubDomains",
                self.ssl.hsts_max_age
            ));
        }

        lines.join("\n")
    }

    fn generate_nginx(&self) -> String {
        let mut config = String::new();
        let domain = self.extract_domain();

        // Header
        config.push_str("# OxideKit Generated Nginx Configuration\n");
        config.push_str("# Same-origin deployment eliminates CORS\n\n");

        // Upstreams
        for upstream in &self.upstreams {
            let upstream_host = self.extract_host(&upstream.url);
            config.push_str(&format!("upstream {} {{\n", upstream.name));
            config.push_str(&format!("    server {};\n", upstream_host));
            config.push_str("}\n\n");
        }

        if let Some(ref frontend) = self.frontend_upstream {
            let frontend_host = self.extract_host(frontend);
            config.push_str("upstream frontend {\n");
            config.push_str(&format!("    server {};\n", frontend_host));
            config.push_str("}\n\n");
        }

        // Server block
        config.push_str("server {\n");

        if self.ssl.enabled {
            config.push_str("    listen 443 ssl http2;\n");
            config.push_str("    listen [::]:443 ssl http2;\n");
            if let (Some(ref cert), Some(ref key)) = (&self.ssl.cert_path, &self.ssl.key_path) {
                config.push_str(&format!("    ssl_certificate {};\n", cert));
                config.push_str(&format!("    ssl_certificate_key {};\n", key));
            }
            if let Some(ref min_tls) = self.ssl.min_tls_version {
                config.push_str(&format!("    ssl_protocols {};\n", min_tls));
            }
        } else {
            config.push_str("    listen 80;\n");
            config.push_str("    listen [::]:80;\n");
        }

        config.push_str(&format!(
            "    server_name {};\n\n",
            self.server_name.as_ref().unwrap_or(&domain)
        ));

        // Security headers
        if let Some(ref h) = self.security_headers.x_frame_options {
            config.push_str(&format!("    add_header X-Frame-Options \"{}\";\n", h));
        }
        if let Some(ref h) = self.security_headers.x_content_type_options {
            config.push_str(&format!("    add_header X-Content-Type-Options \"{}\";\n", h));
        }
        if let Some(ref h) = self.security_headers.x_xss_protection {
            config.push_str(&format!("    add_header X-XSS-Protection \"{}\";\n", h));
        }
        if let Some(ref h) = self.security_headers.referrer_policy {
            config.push_str(&format!("    add_header Referrer-Policy \"{}\";\n", h));
        }
        if self.ssl.hsts {
            config.push_str(&format!(
                "    add_header Strict-Transport-Security \"max-age={}; includeSubDomains\" always;\n",
                self.ssl.hsts_max_age
            ));
        }
        config.push('\n');

        // Gzip
        if self.gzip {
            config.push_str("    gzip on;\n");
            config.push_str("    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;\n");
            config.push_str("    gzip_min_length 1000;\n\n");
        }

        // API locations
        for upstream in &self.upstreams {
            config.push_str(&format!("    location {} {{\n", upstream.path_prefix));

            if upstream.websocket {
                config.push_str("        proxy_http_version 1.1;\n");
                config.push_str("        proxy_set_header Upgrade $http_upgrade;\n");
                config.push_str("        proxy_set_header Connection \"upgrade\";\n");
            }

            config.push_str("        proxy_set_header Host $host;\n");
            config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
            config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
            config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");

            for (key, value) in &upstream.headers {
                config.push_str(&format!("        proxy_set_header {} \"{}\";\n", key, value));
            }

            config.push_str(&format!(
                "        proxy_read_timeout {}s;\n",
                upstream.timeout_secs
            ));
            config.push_str(&format!("        proxy_pass http://{};\n", upstream.name));
            config.push_str("    }\n\n");
        }

        // Frontend location
        if self.frontend_upstream.is_some() {
            config.push_str("    location / {\n");
            config.push_str("        proxy_set_header Host $host;\n");
            config.push_str("        proxy_set_header X-Real-IP $remote_addr;\n");
            config.push_str("        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;\n");
            config.push_str("        proxy_set_header X-Forwarded-Proto $scheme;\n");
            config.push_str("        proxy_pass http://frontend;\n");
            config.push_str("    }\n");
        }

        config.push_str("}\n");

        // HTTP to HTTPS redirect
        if self.ssl.enabled {
            config.push_str("\n# HTTP to HTTPS redirect\n");
            config.push_str("server {\n");
            config.push_str("    listen 80;\n");
            config.push_str("    listen [::]:80;\n");
            config.push_str(&format!(
                "    server_name {};\n",
                self.server_name.as_ref().unwrap_or(&domain)
            ));
            config.push_str("    return 301 https://$host$request_uri;\n");
            config.push_str("}\n");
        }

        config
    }

    fn generate_caddy(&self) -> String {
        let mut config = String::new();
        let domain = self.extract_domain();

        config.push_str("# OxideKit Generated Caddyfile\n");
        config.push_str("# Same-origin deployment eliminates CORS\n\n");

        config.push_str(&format!("{} {{\n", domain));

        // Security headers
        config.push_str("    header {\n");
        if let Some(ref h) = self.security_headers.x_frame_options {
            config.push_str(&format!("        X-Frame-Options {}\n", h));
        }
        if let Some(ref h) = self.security_headers.x_content_type_options {
            config.push_str(&format!("        X-Content-Type-Options {}\n", h));
        }
        if let Some(ref h) = self.security_headers.referrer_policy {
            config.push_str(&format!("        Referrer-Policy {}\n", h));
        }
        config.push_str("    }\n\n");

        // API routes
        for upstream in &self.upstreams {
            config.push_str(&format!("    handle_path {}/* {{\n", upstream.path_prefix));
            config.push_str(&format!("        reverse_proxy {} {{\n", upstream.url));

            if upstream.websocket {
                config.push_str("            # WebSocket support enabled automatically\n");
            }

            config.push_str(&format!(
                "            transport http {{\n                read_timeout {}s\n            }}\n",
                upstream.timeout_secs
            ));

            for (key, value) in &upstream.headers {
                config.push_str(&format!("            header_up {} \"{}\"\n", key, value));
            }

            config.push_str("        }\n");
            config.push_str("    }\n\n");
        }

        // Frontend
        if let Some(ref frontend) = self.frontend_upstream {
            config.push_str("    handle {\n");
            config.push_str(&format!("        reverse_proxy {}\n", frontend));
            config.push_str("    }\n");
        }

        config.push_str("}\n");
        config
    }

    fn generate_traefik(&self) -> String {
        let mut config = String::new();

        config.push_str("# OxideKit Generated Traefik Configuration\n");
        config.push_str("# Same-origin deployment eliminates CORS\n\n");

        config.push_str("http:\n");
        config.push_str("  routers:\n");

        // API routers
        for upstream in &self.upstreams {
            config.push_str(&format!("    {}-router:\n", upstream.name));
            config.push_str("      rule: \"PathPrefix(`");
            config.push_str(&upstream.path_prefix);
            config.push_str("`)\"\n");
            config.push_str(&format!("      service: {}-service\n", upstream.name));
            config.push_str("      middlewares:\n");
            if upstream.strip_prefix {
                config.push_str(&format!("        - {}-strip-prefix\n", upstream.name));
            }
            config.push_str("        - security-headers\n");
            if self.ssl.enabled {
                config.push_str("      tls: {}\n");
            }
            config.push('\n');
        }

        // Frontend router
        if self.frontend_upstream.is_some() {
            config.push_str("    frontend-router:\n");
            config.push_str("      rule: \"PathPrefix(`/`)\"\n");
            config.push_str("      service: frontend-service\n");
            config.push_str("      priority: 1\n");
            config.push_str("      middlewares:\n");
            config.push_str("        - security-headers\n");
            if self.ssl.enabled {
                config.push_str("      tls: {}\n");
            }
            config.push('\n');
        }

        // Services
        config.push_str("  services:\n");
        for upstream in &self.upstreams {
            config.push_str(&format!("    {}-service:\n", upstream.name));
            config.push_str("      loadBalancer:\n");
            config.push_str("        servers:\n");
            config.push_str(&format!("          - url: \"{}\"\n", upstream.url));
            config.push('\n');
        }

        if let Some(ref frontend) = self.frontend_upstream {
            config.push_str("    frontend-service:\n");
            config.push_str("      loadBalancer:\n");
            config.push_str("        servers:\n");
            config.push_str(&format!("          - url: \"{}\"\n", frontend));
            config.push('\n');
        }

        // Middlewares
        config.push_str("  middlewares:\n");
        for upstream in &self.upstreams {
            if upstream.strip_prefix {
                config.push_str(&format!("    {}-strip-prefix:\n", upstream.name));
                config.push_str("      stripPrefix:\n");
                config.push_str(&format!("        prefixes:\n          - \"{}\"\n", upstream.path_prefix));
                config.push('\n');
            }
        }

        config.push_str("    security-headers:\n");
        config.push_str("      headers:\n");
        if let Some(ref h) = self.security_headers.x_frame_options {
            config.push_str(&format!("        customFrameOptionsValue: \"{}\"\n", h));
        }
        if self.security_headers.x_content_type_options.is_some() {
            config.push_str("        contentTypeNosniff: true\n");
        }
        if let Some(ref h) = self.security_headers.referrer_policy {
            config.push_str(&format!("        referrerPolicy: \"{}\"\n", h));
        }
        if self.ssl.hsts {
            config.push_str("        stsSeconds: ");
            config.push_str(&self.ssl.hsts_max_age.to_string());
            config.push('\n');
            config.push_str("        stsIncludeSubdomains: true\n");
        }

        config
    }

    fn generate_cloudflare_workers(&self) -> String {
        let mut config = String::new();

        config.push_str("// OxideKit Generated Cloudflare Worker\n");
        config.push_str("// Same-origin deployment eliminates CORS\n\n");

        config.push_str("const ROUTES = [\n");
        for upstream in &self.upstreams {
            config.push_str(&format!(
                "  {{ prefix: '{}', target: '{}', stripPrefix: {} }},\n",
                upstream.path_prefix, upstream.url, upstream.strip_prefix
            ));
        }
        config.push_str("];\n\n");

        if let Some(ref frontend) = self.frontend_upstream {
            config.push_str(&format!("const FRONTEND_ORIGIN = '{}';\n\n", frontend));
        }

        config.push_str("const SECURITY_HEADERS = {\n");
        if let Some(ref h) = self.security_headers.x_frame_options {
            config.push_str(&format!("  'X-Frame-Options': '{}',\n", h));
        }
        if let Some(ref h) = self.security_headers.x_content_type_options {
            config.push_str(&format!("  'X-Content-Type-Options': '{}',\n", h));
        }
        if let Some(ref h) = self.security_headers.referrer_policy {
            config.push_str(&format!("  'Referrer-Policy': '{}',\n", h));
        }
        if self.ssl.hsts {
            config.push_str(&format!(
                "  'Strict-Transport-Security': 'max-age={}; includeSubDomains',\n",
                self.ssl.hsts_max_age
            ));
        }
        config.push_str("};\n\n");

        config.push_str(r#"export default {
  async fetch(request, env, ctx) {
    const url = new URL(request.url);
    const path = url.pathname;

    // Find matching route
    for (const route of ROUTES) {
      if (path.startsWith(route.prefix)) {
        const targetPath = route.stripPrefix
          ? path.slice(route.prefix.length) || '/'
          : path;
        const targetUrl = route.target + targetPath + url.search;

        const response = await fetch(targetUrl, {
          method: request.method,
          headers: request.headers,
          body: request.body,
        });

        // Add security headers
        const newHeaders = new Headers(response.headers);
        for (const [key, value] of Object.entries(SECURITY_HEADERS)) {
          newHeaders.set(key, value);
        }

        return new Response(response.body, {
          status: response.status,
          statusText: response.statusText,
          headers: newHeaders,
        });
      }
    }

    // Forward to frontend
"#);

        if self.frontend_upstream.is_some() {
            config.push_str("    const frontendUrl = FRONTEND_ORIGIN + path + url.search;\n");
            config.push_str("    const response = await fetch(frontendUrl, {\n");
            config.push_str("      method: request.method,\n");
            config.push_str("      headers: request.headers,\n");
            config.push_str("      body: request.body,\n");
            config.push_str("    });\n\n");
            config.push_str("    const newHeaders = new Headers(response.headers);\n");
            config.push_str("    for (const [key, value] of Object.entries(SECURITY_HEADERS)) {\n");
            config.push_str("      newHeaders.set(key, value);\n");
            config.push_str("    }\n\n");
            config.push_str("    return new Response(response.body, {\n");
            config.push_str("      status: response.status,\n");
            config.push_str("      statusText: response.statusText,\n");
            config.push_str("      headers: newHeaders,\n");
            config.push_str("    });\n");
        } else {
            config.push_str("    return new Response('Not Found', { status: 404 });\n");
        }

        config.push_str("  },\n};\n");
        config
    }

    fn extract_domain(&self) -> String {
        self.site_origin
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("localhost")
            .to_string()
    }

    fn extract_host(&self, url: &str) -> String {
        url.trim_start_matches("https://")
            .trim_start_matches("http://")
            .trim_start_matches("ws://")
            .trim_start_matches("wss://")
            .split('/')
            .next()
            .unwrap_or("localhost:8000")
            .to_string()
    }
}

/// Generated configuration output.
#[derive(Debug, Clone)]
pub struct GeneratedConfig {
    /// The proxy server this config is for.
    pub server: ProxyServer,
    /// Suggested filename.
    pub filename: String,
    /// The generated configuration.
    pub config: String,
    /// Explanation and documentation.
    pub explanation: String,
}

impl GeneratedConfig {
    /// Get the full output with explanation.
    pub fn full_output(&self) -> String {
        format!(
            "{}\n\n---\n\n{}",
            self.explanation, self.config
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_upstream_creation() {
        let upstream = Upstream::new("api", "http://localhost:8000", "/api")
            .with_websocket(true)
            .with_timeout(60)
            .with_header("X-Custom", "value");

        assert_eq!(upstream.name, "api");
        assert_eq!(upstream.url, "http://localhost:8000");
        assert_eq!(upstream.path_prefix, "/api");
        assert!(upstream.websocket);
        assert_eq!(upstream.timeout_secs, 60);
        assert_eq!(upstream.headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_proxy_server_parsing() {
        assert_eq!("nginx".parse::<ProxyServer>().unwrap(), ProxyServer::Nginx);
        assert_eq!("caddy".parse::<ProxyServer>().unwrap(), ProxyServer::Caddy);
        assert_eq!("traefik".parse::<ProxyServer>().unwrap(), ProxyServer::Traefik);
        assert_eq!(
            "cloudflare-workers".parse::<ProxyServer>().unwrap(),
            ProxyServer::CloudflareWorkers
        );
        assert!("unknown".parse::<ProxyServer>().is_err());
    }

    #[test]
    fn test_reverse_proxy_config_builder() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api")
            .frontend_upstream("http://localhost:3000")
            .auto_https();

        assert_eq!(config.site_origin, "https://app.example.com");
        assert_eq!(config.upstreams.len(), 1);
        assert!(config.frontend_upstream.is_some());
        assert!(config.ssl.enabled);
        assert!(config.ssl.auto_https);
    }

    #[test]
    fn test_nginx_generation() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api")
            .frontend_upstream("http://localhost:3000");

        let nginx = config.generate(ProxyServer::Nginx);

        assert!(nginx.contains("upstream api"));
        assert!(nginx.contains("upstream frontend"));
        assert!(nginx.contains("location /api"));
        assert!(nginx.contains("proxy_pass http://api"));
    }

    #[test]
    fn test_caddy_generation() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api");

        let caddy = config.generate(ProxyServer::Caddy);

        assert!(caddy.contains("app.example.com"));
        assert!(caddy.contains("handle_path /api/*"));
        assert!(caddy.contains("reverse_proxy http://localhost:8000"));
    }

    #[test]
    fn test_traefik_generation() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api");

        let traefik = config.generate(ProxyServer::Traefik);

        assert!(traefik.contains("api-router:"));
        assert!(traefik.contains("api-service:"));
        assert!(traefik.contains("PathPrefix(`/api`)"));
    }

    #[test]
    fn test_cloudflare_workers_generation() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api")
            .frontend_upstream("http://localhost:3000");

        let worker = config.generate(ProxyServer::CloudflareWorkers);

        assert!(worker.contains("const ROUTES = ["));
        assert!(worker.contains("prefix: '/api'"));
        assert!(worker.contains("FRONTEND_ORIGIN"));
    }

    #[test]
    fn test_generated_config_with_docs() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .add_upstream("api", "http://localhost:8000", "/api");

        let generated = config.generate_with_docs(ProxyServer::Nginx);

        assert_eq!(generated.server, ProxyServer::Nginx);
        assert_eq!(generated.filename, "nginx.conf");
        assert!(generated.explanation.contains("Same-Origin"));
        assert!(generated.config.contains("upstream"));
    }

    #[test]
    fn test_ssl_config() {
        let auto_ssl = SslConfig::auto();
        assert!(auto_ssl.enabled);
        assert!(auto_ssl.auto_https);
        assert!(auto_ssl.hsts);

        let manual_ssl = SslConfig::manual("/path/to/cert.pem", "/path/to/key.pem");
        assert!(manual_ssl.enabled);
        assert!(!manual_ssl.auto_https);
        assert_eq!(manual_ssl.cert_path, Some("/path/to/cert.pem".to_string()));
    }

    #[test]
    fn test_websocket_upstream() {
        let config = ReverseProxyConfig::new("https://app.example.com")
            .with_upstream(
                Upstream::new("ws", "http://localhost:8001", "/ws").with_websocket(true),
            );

        let nginx = config.generate(ProxyServer::Nginx);

        assert!(nginx.contains("proxy_set_header Upgrade"));
        assert!(nginx.contains("proxy_set_header Connection \"upgrade\""));
    }

    #[test]
    fn test_extract_domain() {
        let config = ReverseProxyConfig::new("https://app.example.com/path");
        assert_eq!(config.extract_domain(), "app.example.com");

        let config2 = ReverseProxyConfig::new("http://localhost:3000");
        assert_eq!(config2.extract_domain(), "localhost:3000");
    }
}
