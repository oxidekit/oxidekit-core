//! Deployment template generation
//!
//! This module provides functionality for generating deployment templates
//! for various platforms including Docker, docker-compose, Railway, Fly.io, and Render.

use crate::env_schema::EnvSchema;
use crate::error::{DeployError, DeployResult};
use crate::ports::PortConfig;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Deployment target platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeployTarget {
    /// Dockerfile generation
    Docker,
    /// docker-compose.yml generation
    DockerCompose,
    /// Railway deployment
    Railway,
    /// Fly.io deployment
    FlyIo,
    /// Render deployment
    Render,
    /// Nginx reverse proxy configuration
    Nginx,
    /// Caddy reverse proxy configuration
    Caddy,
}

impl std::fmt::Display for DeployTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Docker => write!(f, "docker"),
            Self::DockerCompose => write!(f, "docker-compose"),
            Self::Railway => write!(f, "railway"),
            Self::FlyIo => write!(f, "fly.io"),
            Self::Render => write!(f, "render"),
            Self::Nginx => write!(f, "nginx"),
            Self::Caddy => write!(f, "caddy"),
        }
    }
}

impl std::str::FromStr for DeployTarget {
    type Err = DeployError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "docker" => Ok(Self::Docker),
            "docker-compose" | "compose" => Ok(Self::DockerCompose),
            "railway" => Ok(Self::Railway),
            "fly" | "fly.io" | "flyio" => Ok(Self::FlyIo),
            "render" => Ok(Self::Render),
            "nginx" => Ok(Self::Nginx),
            "caddy" => Ok(Self::Caddy),
            other => Err(DeployError::UnsupportedTarget(other.to_string())),
        }
    }
}

/// Application type for template generation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum AppType {
    /// Rust binary application
    #[default]
    RustBinary,
    /// Rust library with examples
    RustLibrary,
    /// Static website
    StaticSite,
    /// Node.js application
    NodeJs,
    /// Generic binary
    Binary,
}

/// Configuration for template generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateConfig {
    /// Application name
    pub app_name: String,
    /// Application type
    #[serde(default)]
    pub app_type: AppType,
    /// Rust version to use
    #[serde(default = "default_rust_version")]
    pub rust_version: String,
    /// Target binary name (if different from app_name)
    #[serde(default)]
    pub binary_name: Option<String>,
    /// Working directory in container
    #[serde(default = "default_workdir")]
    pub workdir: String,
    /// Primary port to expose
    #[serde(default = "default_port")]
    pub port: u16,
    /// Additional ports to expose
    #[serde(default)]
    pub additional_ports: Vec<u16>,
    /// Environment schema for variable generation
    #[serde(skip)]
    pub env_schema: Option<EnvSchema>,
    /// Port configuration
    #[serde(skip)]
    pub port_config: Option<PortConfig>,
    /// Health check endpoint
    #[serde(default = "default_health_check")]
    pub health_check_path: String,
    /// Custom build commands
    #[serde(default)]
    pub build_commands: Vec<String>,
    /// Custom run command
    #[serde(default)]
    pub run_command: Option<String>,
    /// Region for deployment (platform-specific)
    #[serde(default)]
    pub region: Option<String>,
    /// Instance size/type
    #[serde(default)]
    pub instance_type: Option<String>,
    /// Enable auto-scaling
    #[serde(default)]
    pub auto_scale: bool,
    /// Minimum instances for auto-scaling
    #[serde(default = "default_min_instances")]
    pub min_instances: u32,
    /// Maximum instances for auto-scaling
    #[serde(default = "default_max_instances")]
    pub max_instances: u32,
}

fn default_rust_version() -> String {
    "1.75".to_string()
}

fn default_workdir() -> String {
    "/app".to_string()
}

fn default_port() -> u16 {
    8080
}

fn default_health_check() -> String {
    "/health".to_string()
}

fn default_min_instances() -> u32 {
    1
}

fn default_max_instances() -> u32 {
    10
}

impl Default for TemplateConfig {
    fn default() -> Self {
        Self {
            app_name: "oxide-app".to_string(),
            app_type: AppType::RustBinary,
            rust_version: default_rust_version(),
            binary_name: None,
            workdir: default_workdir(),
            port: default_port(),
            additional_ports: Vec::new(),
            env_schema: None,
            port_config: None,
            health_check_path: default_health_check(),
            build_commands: Vec::new(),
            run_command: None,
            region: None,
            instance_type: None,
            auto_scale: false,
            min_instances: default_min_instances(),
            max_instances: default_max_instances(),
        }
    }
}

impl TemplateConfig {
    /// Create a new template configuration
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            ..Default::default()
        }
    }

    /// Set the application type
    pub fn with_app_type(mut self, app_type: AppType) -> Self {
        self.app_type = app_type;
        self
    }

    /// Set the primary port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    /// Set the environment schema
    pub fn with_env_schema(mut self, schema: EnvSchema) -> Self {
        self.env_schema = Some(schema);
        self
    }

    /// Set the port configuration
    pub fn with_port_config(mut self, config: PortConfig) -> Self {
        self.port_config = Some(config);
        self
    }

    /// Get the binary name
    pub fn binary_name(&self) -> &str {
        self.binary_name.as_deref().unwrap_or(&self.app_name)
    }
}

/// Template generator for deployment configurations
pub struct TemplateGenerator {
    config: TemplateConfig,
}

impl TemplateGenerator {
    /// Create a new template generator
    pub fn new(config: TemplateConfig) -> Self {
        Self { config }
    }

    /// Generate a template for the specified target
    pub fn generate(&self, target: DeployTarget) -> DeployResult<String> {
        match target {
            DeployTarget::Docker => self.generate_dockerfile(),
            DeployTarget::DockerCompose => self.generate_docker_compose(),
            DeployTarget::Railway => self.generate_railway(),
            DeployTarget::FlyIo => self.generate_fly_io(),
            DeployTarget::Render => self.generate_render(),
            DeployTarget::Nginx => self.generate_nginx(),
            DeployTarget::Caddy => self.generate_caddy(),
        }
    }

    /// Generate a Dockerfile
    pub fn generate_dockerfile(&self) -> DeployResult<String> {
        let binary_name = self.config.binary_name();

        let template = match self.config.app_type {
            AppType::RustBinary | AppType::RustLibrary => {
                format!(
                    r#"# Build stage
FROM rust:{rust_version} AS builder

WORKDIR /build

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy source for dependency caching
RUN mkdir src && echo "fn main() {{}}" > src/main.rs

# Build dependencies
RUN cargo build --release && rm -rf src

# Copy actual source
COPY src ./src

# Build application
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR {workdir}

# Copy binary from builder
COPY --from=builder /build/target/release/{binary_name} ./{binary_name}

# Create non-root user
RUN useradd -m -U appuser && chown -R appuser:appuser {workdir}
USER appuser

EXPOSE {port}

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:{port}{health_check} || exit 1

CMD ["./{binary_name}"]
"#,
                    rust_version = self.config.rust_version,
                    workdir = self.config.workdir,
                    binary_name = binary_name,
                    port = self.config.port,
                    health_check = self.config.health_check_path,
                )
            }
            AppType::StaticSite => {
                format!(
                    r#"# Build stage
FROM rust:{rust_version} AS builder

WORKDIR /build
COPY . .

# Build static site
RUN cargo build --release

# Runtime stage - Nginx for static files
FROM nginx:alpine

COPY --from=builder /build/dist /usr/share/nginx/html
COPY nginx.conf /etc/nginx/nginx.conf

EXPOSE {port}

CMD ["nginx", "-g", "daemon off;"]
"#,
                    rust_version = self.config.rust_version,
                    port = self.config.port,
                )
            }
            AppType::NodeJs => {
                format!(
                    r#"FROM node:20-slim

WORKDIR {workdir}

COPY package*.json ./
RUN npm ci --only=production

COPY . .

RUN npm run build

EXPOSE {port}

USER node

CMD ["npm", "start"]
"#,
                    workdir = self.config.workdir,
                    port = self.config.port,
                )
            }
            AppType::Binary => {
                format!(
                    r#"FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR {workdir}

COPY {binary_name} ./{binary_name}
RUN chmod +x ./{binary_name}

EXPOSE {port}

CMD ["./{binary_name}"]
"#,
                    workdir = self.config.workdir,
                    binary_name = binary_name,
                    port = self.config.port,
                )
            }
        };

        Ok(template)
    }

    /// Generate a docker-compose.yml
    pub fn generate_docker_compose(&self) -> DeployResult<String> {
        let mut env_section = String::new();
        if let Some(ref schema) = self.config.env_schema {
            for var in &schema.variables {
                if var.secret {
                    env_section.push_str(&format!("      - {}=${{{}:-}}\n", var.name, var.name));
                } else if let Some(ref default) = var.default {
                    env_section.push_str(&format!(
                        "      - {}=${{{}:-{}}}\n",
                        var.name, var.name, default
                    ));
                } else {
                    env_section.push_str(&format!("      - {}=${{{}}}\n", var.name, var.name));
                }
            }
        }

        let mut ports_section = format!("      - \"{}:{}\"\n", self.config.port, self.config.port);
        for port in &self.config.additional_ports {
            ports_section.push_str(&format!("      - \"{}:{}\"\n", port, port));
        }

        let template = format!(
            r#"version: '3.8'

services:
  {app_name}:
    build:
      context: .
      dockerfile: Dockerfile
    container_name: {app_name}
    restart: unless-stopped
    ports:
{ports}
    environment:
{env}
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{port}{health_check}"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 40s
    networks:
      - {app_name}-network

networks:
  {app_name}-network:
    driver: bridge
"#,
            app_name = self.config.app_name,
            ports = ports_section.trim_end(),
            env = if env_section.is_empty() {
                "      - PORT={}".to_string()
            } else {
                env_section.trim_end().to_string()
            },
            port = self.config.port,
            health_check = self.config.health_check_path,
        );

        Ok(template)
    }

    /// Generate Railway configuration (railway.toml)
    pub fn generate_railway(&self) -> DeployResult<String> {
        let template = format!(
            r#"[build]
builder = "dockerfile"
dockerfilePath = "Dockerfile"

[deploy]
startCommand = "./{binary_name}"
healthcheckPath = "{health_check}"
healthcheckTimeout = 100
restartPolicyType = "on_failure"
restartPolicyMaxRetries = 3

[[deploy.port]]
port = {port}
protocol = "TCP"
"#,
            binary_name = self.config.binary_name(),
            health_check = self.config.health_check_path,
            port = self.config.port,
        );

        Ok(template)
    }

    /// Generate Fly.io configuration (fly.toml)
    pub fn generate_fly_io(&self) -> DeployResult<String> {
        let region = self.config.region.as_deref().unwrap_or("iad");

        let template = format!(
            r#"app = "{app_name}"
primary_region = "{region}"

[build]
dockerfile = "Dockerfile"

[env]
PORT = "{port}"

[http_service]
internal_port = {port}
force_https = true
auto_stop_machines = true
auto_start_machines = true
min_machines_running = {min_instances}

[[http_service.checks]]
interval = "30s"
timeout = "5s"
path = "{health_check}"

[[vm]]
cpu_kind = "shared"
cpus = 1
memory_mb = 256
"#,
            app_name = self.config.app_name,
            region = region,
            port = self.config.port,
            health_check = self.config.health_check_path,
            min_instances = self.config.min_instances,
        );

        Ok(template)
    }

    /// Generate Render configuration (render.yaml)
    pub fn generate_render(&self) -> DeployResult<String> {
        let mut env_vars = String::new();
        if let Some(ref schema) = self.config.env_schema {
            for var in &schema.variables {
                if var.secret {
                    env_vars.push_str(&format!(
                        "      - key: {}\n        sync: false\n",
                        var.name
                    ));
                } else if let Some(ref default) = var.default {
                    env_vars.push_str(&format!(
                        "      - key: {}\n        value: {}\n",
                        var.name, default
                    ));
                }
            }
        }

        let template = format!(
            r#"services:
  - type: web
    name: {app_name}
    env: docker
    dockerfilePath: ./Dockerfile
    healthCheckPath: {health_check}
    envVars:
      - key: PORT
        value: {port}
{env_vars}
    autoDeploy: true
"#,
            app_name = self.config.app_name,
            health_check = self.config.health_check_path,
            port = self.config.port,
            env_vars = env_vars,
        );

        Ok(template)
    }

    /// Generate Nginx configuration
    pub fn generate_nginx(&self) -> DeployResult<String> {
        let template = format!(
            r#"upstream {app_name}_backend {{
    server 127.0.0.1:{port};
    keepalive 32;
}}

server {{
    listen 80;
    listen [::]:80;
    server_name _;

    # Redirect HTTP to HTTPS
    return 301 https://$host$request_uri;
}}

server {{
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name _;

    # SSL configuration
    ssl_certificate /etc/nginx/ssl/cert.pem;
    ssl_certificate_key /etc/nginx/ssl/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-ECDSA-AES128-GCM-SHA256:ECDHE-RSA-AES128-GCM-SHA256;
    ssl_prefer_server_ciphers off;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;

    # CORS headers (adjust as needed)
    add_header Access-Control-Allow-Origin "*";
    add_header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS";
    add_header Access-Control-Allow-Headers "Authorization, Content-Type";

    location / {{
        proxy_pass http://{app_name}_backend;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_set_header Connection "";
        proxy_buffering off;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }}

    location {health_check} {{
        proxy_pass http://{app_name}_backend{health_check};
        access_log off;
    }}

    # Gzip compression
    gzip on;
    gzip_types text/plain text/css application/json application/javascript text/xml application/xml;
    gzip_min_length 1000;
}}
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path,
        );

        Ok(template)
    }

    /// Generate Caddy configuration (Caddyfile)
    pub fn generate_caddy(&self) -> DeployResult<String> {
        let template = format!(
            r#"{{
    # Global options
    admin off
    auto_https on
}}

# Replace with your domain
your-domain.com {{
    # Enable compression
    encode gzip

    # Reverse proxy to application
    reverse_proxy localhost:{port} {{
        header_up Host {{host}}
        header_up X-Real-IP {{remote_host}}
        header_up X-Forwarded-For {{remote_host}}
        header_up X-Forwarded-Proto {{scheme}}

        # Health checks
        health_uri {health_check}
        health_interval 30s
        health_timeout 5s
    }}

    # Security headers
    header {{
        X-Frame-Options "SAMEORIGIN"
        X-Content-Type-Options "nosniff"
        X-XSS-Protection "1; mode=block"
        Referrer-Policy "strict-origin-when-cross-origin"
        -Server
    }}

    # CORS (adjust as needed)
    @options method OPTIONS
    handle @options {{
        header Access-Control-Allow-Origin "*"
        header Access-Control-Allow-Methods "GET, POST, PUT, DELETE, OPTIONS"
        header Access-Control-Allow-Headers "Authorization, Content-Type"
        respond 204
    }}

    # Logging
    log {{
        output file /var/log/caddy/{app_name}.log
        format json
    }}
}}
"#,
            port = self.config.port,
            health_check = self.config.health_check_path,
            app_name = self.config.app_name,
        );

        Ok(template)
    }

    /// Generate all templates and save to a directory
    pub fn generate_all(&self, output_dir: impl AsRef<Path>) -> DeployResult<Vec<(DeployTarget, String)>> {
        let dir = output_dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let targets = vec![
            (DeployTarget::Docker, "Dockerfile"),
            (DeployTarget::DockerCompose, "docker-compose.yml"),
            (DeployTarget::Railway, "railway.toml"),
            (DeployTarget::FlyIo, "fly.toml"),
            (DeployTarget::Render, "render.yaml"),
            (DeployTarget::Nginx, "nginx.conf"),
            (DeployTarget::Caddy, "Caddyfile"),
        ];

        let mut results = Vec::new();
        for (target, filename) in targets {
            let content = self.generate(target)?;
            let path = dir.join(filename);
            std::fs::write(&path, &content)?;
            results.push((target, path.display().to_string()));
        }

        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deploy_target_parsing() {
        assert_eq!("docker".parse::<DeployTarget>().unwrap(), DeployTarget::Docker);
        assert_eq!("fly.io".parse::<DeployTarget>().unwrap(), DeployTarget::FlyIo);
        assert_eq!("railway".parse::<DeployTarget>().unwrap(), DeployTarget::Railway);
        assert!("invalid".parse::<DeployTarget>().is_err());
    }

    #[test]
    fn test_template_config_creation() {
        let config = TemplateConfig::new("my-app")
            .with_app_type(AppType::RustBinary)
            .with_port(3000);

        assert_eq!(config.app_name, "my-app");
        assert_eq!(config.port, 3000);
        assert_eq!(config.app_type, AppType::RustBinary);
    }

    #[test]
    fn test_dockerfile_generation() {
        let config = TemplateConfig::new("test-app")
            .with_app_type(AppType::RustBinary)
            .with_port(8080);

        let generator = TemplateGenerator::new(config);
        let dockerfile = generator.generate_dockerfile().unwrap();

        assert!(dockerfile.contains("FROM rust:"));
        assert!(dockerfile.contains("EXPOSE 8080"));
        assert!(dockerfile.contains("test-app"));
        assert!(dockerfile.contains("HEALTHCHECK"));
    }

    #[test]
    fn test_docker_compose_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let compose = generator.generate_docker_compose().unwrap();

        assert!(compose.contains("version: '3.8'"));
        assert!(compose.contains("test-app"));
        assert!(compose.contains("8080:8080"));
    }

    #[test]
    fn test_railway_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let railway = generator.generate_railway().unwrap();

        assert!(railway.contains("[build]"));
        assert!(railway.contains("dockerfile"));
        assert!(railway.contains("8080"));
    }

    #[test]
    fn test_fly_io_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let fly = generator.generate_fly_io().unwrap();

        assert!(fly.contains("app = \"test-app\""));
        assert!(fly.contains("internal_port = 8080"));
        assert!(fly.contains("[http_service]"));
    }

    #[test]
    fn test_render_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let render = generator.generate_render().unwrap();

        assert!(render.contains("name: test-app"));
        assert!(render.contains("type: web"));
        assert!(render.contains("8080"));
    }

    #[test]
    fn test_nginx_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let nginx = generator.generate_nginx().unwrap();

        assert!(nginx.contains("upstream test-app_backend"));
        assert!(nginx.contains("server 127.0.0.1:8080"));
        assert!(nginx.contains("proxy_pass"));
        assert!(nginx.contains("ssl_certificate"));
    }

    #[test]
    fn test_caddy_generation() {
        let config = TemplateConfig::new("test-app").with_port(8080);
        let generator = TemplateGenerator::new(config);
        let caddy = generator.generate_caddy().unwrap();

        assert!(caddy.contains("reverse_proxy localhost:8080"));
        assert!(caddy.contains("encode gzip"));
        assert!(caddy.contains("X-Frame-Options"));
    }

    #[test]
    fn test_static_site_dockerfile() {
        let config = TemplateConfig::new("static-site")
            .with_app_type(AppType::StaticSite)
            .with_port(80);

        let generator = TemplateGenerator::new(config);
        let dockerfile = generator.generate_dockerfile().unwrap();

        assert!(dockerfile.contains("nginx:alpine"));
        assert!(dockerfile.contains("/usr/share/nginx/html"));
    }

    #[test]
    fn test_nodejs_dockerfile() {
        let config = TemplateConfig::new("node-app")
            .with_app_type(AppType::NodeJs)
            .with_port(3000);

        let generator = TemplateGenerator::new(config);
        let dockerfile = generator.generate_dockerfile().unwrap();

        assert!(dockerfile.contains("FROM node:20-slim"));
        assert!(dockerfile.contains("npm ci"));
        // The CMD uses JSON array format with quotes
        assert!(dockerfile.contains(r#"CMD ["npm""#) || dockerfile.contains("npm start"));
    }
}
