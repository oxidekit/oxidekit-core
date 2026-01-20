//! Deployment kit generation
//!
//! Generates deployment configurations for various platforms:
//! - Docker (generic)
//! - Railway
//! - Fly.io
//! - Render
//! - Generic VPS
//!
//! Deployments are boring, repeatable, and documented.

use crate::Result;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Deployment target platforms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeployTarget {
    /// Docker-based deployment (default)
    #[default]
    Docker,
    /// Railway.app
    Railway,
    /// Fly.io
    Fly,
    /// Render.com
    Render,
    /// Generic VPS with Docker
    Vps,
}

impl DeployTarget {
    /// Get the display name for the target
    pub fn display_name(&self) -> &'static str {
        match self {
            DeployTarget::Docker => "Docker",
            DeployTarget::Railway => "Railway",
            DeployTarget::Fly => "Fly.io",
            DeployTarget::Render => "Render",
            DeployTarget::Vps => "VPS",
        }
    }

    /// Get the CLI name for the target
    pub fn cli_name(&self) -> &'static str {
        match self {
            DeployTarget::Docker => "docker",
            DeployTarget::Railway => "railway",
            DeployTarget::Fly => "fly",
            DeployTarget::Render => "render",
            DeployTarget::Vps => "vps",
        }
    }
}

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeployConfig {
    /// Target platform
    pub target: DeployTarget,
    /// Application name
    pub app_name: String,
    /// Region (if applicable)
    pub region: Option<String>,
    /// Environment variables to set
    pub env_vars: Vec<EnvVar>,
    /// Port to expose
    pub port: u16,
    /// Health check path
    pub health_check_path: String,
    /// Enable HTTPS redirect
    pub https_redirect: bool,
    /// Auto-scaling configuration
    pub scaling: ScalingConfig,
}

impl Default for DeployConfig {
    fn default() -> Self {
        Self {
            target: DeployTarget::default(),
            app_name: String::new(),
            region: None,
            env_vars: Vec::new(),
            port: 8000,
            health_check_path: "/health".to_string(),
            https_redirect: true,
            scaling: ScalingConfig::default(),
        }
    }
}

impl DeployConfig {
    /// Create a new deployment config with the given app name
    pub fn new(app_name: impl Into<String>) -> Self {
        Self {
            app_name: app_name.into(),
            ..Default::default()
        }
    }

    /// Set the target platform
    pub fn with_target(mut self, target: DeployTarget) -> Self {
        self.target = target;
        self
    }

    /// Set the region
    pub fn with_region(mut self, region: impl Into<String>) -> Self {
        self.region = Some(region.into());
        self
    }

    /// Add an environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push(EnvVar {
            key: key.into(),
            value: value.into(),
            secret: false,
        });
        self
    }

    /// Add a secret environment variable
    pub fn with_secret(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env_vars.push(EnvVar {
            key: key.into(),
            value: value.into(),
            secret: true,
        });
        self
    }

    /// Set the port
    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }
}

/// Environment variable configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    /// Variable name
    pub key: String,
    /// Variable value (or placeholder for secrets)
    pub value: String,
    /// Whether this is a secret (should not be logged)
    pub secret: bool,
}

/// Scaling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConfig {
    /// Minimum instances
    pub min_instances: u32,
    /// Maximum instances
    pub max_instances: u32,
    /// Enable auto-scaling
    pub auto_scale: bool,
    /// Memory limit per instance (MB)
    pub memory_mb: u32,
    /// CPU limit (millicores, e.g., 500 = 0.5 CPU)
    pub cpu_millicores: u32,
}

impl Default for ScalingConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 1,
            auto_scale: false,
            memory_mb: 512,
            cpu_millicores: 500,
        }
    }
}

/// Deployment kit generator
#[derive(Debug)]
pub struct DeploymentKit {
    config: DeployConfig,
}

impl DeploymentKit {
    /// Create a new deployment kit with the given configuration
    pub fn new(config: DeployConfig) -> Self {
        Self { config }
    }

    /// Generate deployment files for the configured target
    pub fn generate(&self, output_dir: impl AsRef<Path>) -> Result<Vec<GeneratedFile>> {
        let output = output_dir.as_ref();
        std::fs::create_dir_all(output)?;

        let files = match self.config.target {
            DeployTarget::Docker => self.generate_docker()?,
            DeployTarget::Railway => self.generate_railway()?,
            DeployTarget::Fly => self.generate_fly()?,
            DeployTarget::Render => self.generate_render()?,
            DeployTarget::Vps => self.generate_vps()?,
        };

        // Write files
        for file in &files {
            let path = output.join(&file.path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&path, &file.content)?;
        }

        tracing::info!(
            "Generated {} deployment files for {}",
            files.len(),
            self.config.target.display_name()
        );

        Ok(files)
    }

    /// Generate Docker deployment files
    fn generate_docker(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = vec![];

        // Dockerfile
        files.push(GeneratedFile {
            path: "Dockerfile".to_string(),
            content: self.dockerfile_content(),
            description: "Docker image build configuration".to_string(),
        });

        // docker-compose.yml
        files.push(GeneratedFile {
            path: "docker-compose.yml".to_string(),
            content: self.docker_compose_content(),
            description: "Docker Compose for local development".to_string(),
        });

        // docker-compose.prod.yml
        files.push(GeneratedFile {
            path: "docker-compose.prod.yml".to_string(),
            content: self.docker_compose_prod_content(),
            description: "Docker Compose for production".to_string(),
        });

        // .dockerignore
        files.push(GeneratedFile {
            path: ".dockerignore".to_string(),
            content: self.dockerignore_content(),
            description: "Docker build exclusions".to_string(),
        });

        Ok(files)
    }

    /// Generate Railway deployment files
    fn generate_railway(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = self.generate_docker()?;

        // railway.json
        files.push(GeneratedFile {
            path: "railway.json".to_string(),
            content: self.railway_json_content(),
            description: "Railway deployment configuration".to_string(),
        });

        Ok(files)
    }

    /// Generate Fly.io deployment files
    fn generate_fly(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = self.generate_docker()?;

        // fly.toml
        files.push(GeneratedFile {
            path: "fly.toml".to_string(),
            content: self.fly_toml_content(),
            description: "Fly.io deployment configuration".to_string(),
        });

        Ok(files)
    }

    /// Generate Render deployment files
    fn generate_render(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = self.generate_docker()?;

        // render.yaml
        files.push(GeneratedFile {
            path: "render.yaml".to_string(),
            content: self.render_yaml_content(),
            description: "Render deployment configuration".to_string(),
        });

        Ok(files)
    }

    /// Generate VPS deployment files
    fn generate_vps(&self) -> Result<Vec<GeneratedFile>> {
        let mut files = self.generate_docker()?;

        // deploy.sh
        files.push(GeneratedFile {
            path: "deploy/deploy.sh".to_string(),
            content: self.vps_deploy_script(),
            description: "VPS deployment script".to_string(),
        });

        // nginx.conf
        files.push(GeneratedFile {
            path: "deploy/nginx.conf".to_string(),
            content: self.nginx_conf_content(),
            description: "Nginx reverse proxy configuration".to_string(),
        });

        // systemd service
        files.push(GeneratedFile {
            path: format!("deploy/{}.service", self.config.app_name),
            content: self.systemd_service_content(),
            description: "Systemd service file".to_string(),
        });

        Ok(files)
    }

    fn dockerfile_content(&self) -> String {
        format!(
            r#"# {app_name} Dockerfile
# Generated by OxideKit Backend Builder

FROM python:3.11-slim as builder

WORKDIR /app

# Install build dependencies
RUN pip install --no-cache-dir --upgrade pip

# Copy requirements first for caching
COPY requirements.txt .
RUN pip install --no-cache-dir -r requirements.txt

# Production image
FROM python:3.11-slim

WORKDIR /app

# Copy installed packages from builder
COPY --from=builder /usr/local/lib/python3.11/site-packages /usr/local/lib/python3.11/site-packages
COPY --from=builder /usr/local/bin /usr/local/bin

# Copy application code
COPY . .

# Create non-root user
RUN useradd --create-home --shell /bin/bash appuser
USER appuser

# Expose port
EXPOSE {port}

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:{port}{health_check} || exit 1

# Run the application
CMD ["uvicorn", "app.main:app", "--host", "0.0.0.0", "--port", "{port}"]
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path
        )
    }

    fn docker_compose_content(&self) -> String {
        let env_vars: Vec<String> = self
            .config
            .env_vars
            .iter()
            .map(|e| {
                if e.secret {
                    format!("      - {}=${{{}:-}}", e.key, e.key)
                } else {
                    format!("      - {}={}", e.key, e.value)
                }
            })
            .collect();

        let env_section = if env_vars.is_empty() {
            String::new()
        } else {
            format!("    environment:\n{}", env_vars.join("\n"))
        };

        format!(
            r#"# Docker Compose for local development
version: "3.8"

services:
  api:
    build:
      context: .
      dockerfile: Dockerfile
    ports:
      - "{port}:{port}"
{env_section}
    volumes:
      - .:/app
    command: uvicorn app.main:app --host 0.0.0.0 --port {port} --reload
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{port}{health_check}"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 10s
"#,
            port = self.config.port,
            health_check = self.config.health_check_path,
            env_section = env_section
        )
    }

    fn docker_compose_prod_content(&self) -> String {
        format!(
            r#"# Docker Compose for production
version: "3.8"

services:
  api:
    image: {app_name}:latest
    restart: always
    ports:
      - "{port}:{port}"
    env_file:
      - .env
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:{port}{health_check}"]
      interval: 30s
      timeout: 10s
      retries: 3
      start_period: 30s
    deploy:
      resources:
        limits:
          memory: {memory}M
          cpus: "{cpu}"
        reservations:
          memory: 256M
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path,
            memory = self.config.scaling.memory_mb,
            cpu = self.config.scaling.cpu_millicores as f32 / 1000.0
        )
    }

    fn dockerignore_content(&self) -> String {
        r#"# Git
.git
.gitignore

# Python
__pycache__
*.pyc
*.pyo
*.pyd
.Python
*.so
.venv
venv
env
.env
.mypy_cache
.pytest_cache
.coverage
htmlcov

# IDE
.idea
.vscode
*.swp
*.swo

# Build
dist
build
*.egg-info

# Documentation
docs
*.md
!README.md

# Tests
tests
pytest.ini
setup.cfg

# Deployment
deploy
docker-compose*.yml
fly.toml
railway.json
render.yaml
"#
        .to_string()
    }

    fn railway_json_content(&self) -> String {
        format!(
            r#"{{
  "$schema": "https://railway.app/railway.schema.json",
  "build": {{
    "builder": "DOCKERFILE",
    "dockerfilePath": "Dockerfile"
  }},
  "deploy": {{
    "restartPolicyType": "ON_FAILURE",
    "restartPolicyMaxRetries": 10,
    "healthcheckPath": "{health_check}",
    "healthcheckTimeout": 10
  }}
}}
"#,
            health_check = self.config.health_check_path
        )
    }

    fn fly_toml_content(&self) -> String {
        let region = self
            .config
            .region
            .as_ref()
            .map(|r| format!("primary_region = \"{}\"", r))
            .unwrap_or_else(|| "primary_region = \"ord\"".to_string());

        format!(
            r#"# Fly.io configuration
# See https://fly.io/docs/reference/configuration/

app = "{app_name}"
{region}

[build]
  dockerfile = "Dockerfile"

[http_service]
  internal_port = {port}
  force_https = {https}
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = {min_instances}

[[http_service.checks]]
  grace_period = "10s"
  interval = "30s"
  method = "GET"
  path = "{health_check}"
  timeout = "5s"

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = {memory}
"#,
            app_name = self.config.app_name,
            region = region,
            port = self.config.port,
            https = self.config.https_redirect,
            health_check = self.config.health_check_path,
            min_instances = self.config.scaling.min_instances,
            memory = self.config.scaling.memory_mb
        )
    }

    fn render_yaml_content(&self) -> String {
        format!(
            r#"# Render Blueprint
# See https://render.com/docs/blueprint-spec

services:
  - type: web
    name: {app_name}
    env: docker
    dockerfilePath: ./Dockerfile
    region: oregon
    plan: starter
    healthCheckPath: {health_check}
    envVars:
      - key: PORT
        value: "{port}"
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path
        )
    }

    fn vps_deploy_script(&self) -> String {
        format!(
            r#"#!/bin/bash
# VPS Deployment Script for {app_name}
# Generated by OxideKit Backend Builder

set -e

APP_NAME="{app_name}"
PORT={port}
IMAGE_NAME="${{APP_NAME}}:latest"

echo "=== Building Docker image ==="
docker build -t $IMAGE_NAME .

echo "=== Stopping existing container (if any) ==="
docker stop $APP_NAME 2>/dev/null || true
docker rm $APP_NAME 2>/dev/null || true

echo "=== Starting new container ==="
docker run -d \
    --name $APP_NAME \
    --restart unless-stopped \
    -p $PORT:$PORT \
    --env-file .env \
    $IMAGE_NAME

echo "=== Waiting for health check ==="
sleep 5
curl -f http://localhost:$PORT{health_check} && echo "Deployment successful!" || echo "Health check failed!"

echo "=== Cleaning up old images ==="
docker image prune -f
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path
        )
    }

    fn nginx_conf_content(&self) -> String {
        format!(
            r#"# Nginx configuration for {app_name}
# Place in /etc/nginx/sites-available/{app_name}

upstream {app_name}_backend {{
    server 127.0.0.1:{port};
}}

server {{
    listen 80;
    server_name your-domain.com;
    return 301 https://$server_name$request_uri;
}}

server {{
    listen 443 ssl http2;
    server_name your-domain.com;

    # SSL configuration - update paths to your certificates
    ssl_certificate /etc/letsencrypt/live/your-domain.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/your-domain.com/privkey.pem;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    location / {{
        proxy_pass http://{app_name}_backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }}

    location {health_check} {{
        proxy_pass http://{app_name}_backend;
        proxy_http_version 1.1;
        access_log off;
    }}
}}
"#,
            app_name = self.config.app_name,
            port = self.config.port,
            health_check = self.config.health_check_path
        )
    }

    fn systemd_service_content(&self) -> String {
        format!(
            r#"# Systemd service for {app_name}
# Place in /etc/systemd/system/{app_name}.service

[Unit]
Description={app_name} API Service
After=docker.service
Requires=docker.service

[Service]
Type=simple
Restart=always
RestartSec=5
ExecStart=/usr/bin/docker start -a {app_name}
ExecStop=/usr/bin/docker stop {app_name}

[Install]
WantedBy=multi-user.target
"#,
            app_name = self.config.app_name
        )
    }
}

/// A generated deployment file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedFile {
    /// File path relative to output directory
    pub path: String,
    /// File content
    pub content: String,
    /// Description of the file's purpose
    pub description: String,
}

/// Get deployment instructions for a target
pub fn get_deployment_instructions(target: DeployTarget) -> &'static str {
    match target {
        DeployTarget::Docker => {
            r#"
# Docker Deployment

## Build and run locally:
docker-compose up --build

## Build for production:
docker build -t your-app:latest .

## Run in production:
docker-compose -f docker-compose.prod.yml up -d
"#
        }
        DeployTarget::Railway => {
            r#"
# Railway Deployment

## Prerequisites:
- Install Railway CLI: npm install -g @railway/cli
- Login: railway login

## Deploy:
railway up

## View logs:
railway logs
"#
        }
        DeployTarget::Fly => {
            r#"
# Fly.io Deployment

## Prerequisites:
- Install flyctl: https://fly.io/docs/hands-on/install-flyctl/
- Login: fly auth login

## First deployment:
fly launch

## Subsequent deployments:
fly deploy

## View logs:
fly logs
"#
        }
        DeployTarget::Render => {
            r#"
# Render Deployment

## Prerequisites:
- Create account at render.com
- Connect your Git repository

## Deploy:
Push to your connected branch, Render will auto-deploy.

## Manual deploy:
Use the Render dashboard to trigger a manual deploy.
"#
        }
        DeployTarget::Vps => {
            r#"
# VPS Deployment

## Prerequisites:
- SSH access to your server
- Docker installed on server
- Domain pointing to server IP

## Deploy:
1. Copy files to server: scp -r . user@server:/app
2. SSH to server: ssh user@server
3. Navigate to app: cd /app
4. Run deploy script: ./deploy/deploy.sh

## Setup Nginx:
sudo cp deploy/nginx.conf /etc/nginx/sites-available/your-app
sudo ln -s /etc/nginx/sites-available/your-app /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx

## Setup SSL:
sudo certbot --nginx -d your-domain.com
"#
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_deploy_config_default() {
        let config = DeployConfig::default();
        assert_eq!(config.port, 8000);
        assert_eq!(config.health_check_path, "/health");
        assert!(matches!(config.target, DeployTarget::Docker));
    }

    #[test]
    fn test_deploy_config_builder() {
        let config = DeployConfig::new("my-api")
            .with_target(DeployTarget::Railway)
            .with_region("us-west")
            .with_port(3000)
            .with_env("DEBUG", "false")
            .with_secret("SECRET_KEY", "xxx");

        assert_eq!(config.app_name, "my-api");
        assert!(matches!(config.target, DeployTarget::Railway));
        assert_eq!(config.region, Some("us-west".to_string()));
        assert_eq!(config.port, 3000);
        assert_eq!(config.env_vars.len(), 2);
        assert!(!config.env_vars[0].secret);
        assert!(config.env_vars[1].secret);
    }

    #[test]
    fn test_deploy_target_names() {
        assert_eq!(DeployTarget::Docker.display_name(), "Docker");
        assert_eq!(DeployTarget::Railway.cli_name(), "railway");
        assert_eq!(DeployTarget::Fly.display_name(), "Fly.io");
    }

    #[test]
    fn test_deployment_kit_docker() {
        let config = DeployConfig::new("test-api");
        let kit = DeploymentKit::new(config);
        let temp_dir = TempDir::new().unwrap();

        let files = kit.generate(temp_dir.path()).unwrap();

        assert!(files.iter().any(|f| f.path == "Dockerfile"));
        assert!(files.iter().any(|f| f.path == "docker-compose.yml"));
        assert!(files.iter().any(|f| f.path == ".dockerignore"));
    }

    #[test]
    fn test_deployment_kit_railway() {
        let config = DeployConfig::new("test-api").with_target(DeployTarget::Railway);
        let kit = DeploymentKit::new(config);
        let temp_dir = TempDir::new().unwrap();

        let files = kit.generate(temp_dir.path()).unwrap();

        assert!(files.iter().any(|f| f.path == "railway.json"));
    }

    #[test]
    fn test_deployment_kit_fly() {
        let config = DeployConfig::new("test-api").with_target(DeployTarget::Fly);
        let kit = DeploymentKit::new(config);
        let temp_dir = TempDir::new().unwrap();

        let files = kit.generate(temp_dir.path()).unwrap();

        assert!(files.iter().any(|f| f.path == "fly.toml"));
    }

    #[test]
    fn test_deployment_kit_vps() {
        let config = DeployConfig::new("test-api").with_target(DeployTarget::Vps);
        let kit = DeploymentKit::new(config);
        let temp_dir = TempDir::new().unwrap();

        let files = kit.generate(temp_dir.path()).unwrap();

        assert!(files.iter().any(|f| f.path == "deploy/deploy.sh"));
        assert!(files.iter().any(|f| f.path == "deploy/nginx.conf"));
    }

    #[test]
    fn test_scaling_config_default() {
        let config = ScalingConfig::default();
        assert_eq!(config.min_instances, 1);
        assert_eq!(config.max_instances, 1);
        assert_eq!(config.memory_mb, 512);
    }
}
