//! CI workflow template generation
//!
//! This module provides functionality for generating CI workflow templates
//! for GitHub Actions and other CI platforms.

use crate::env_schema::EnvSchema;
use crate::error::DeployResult;
use crate::templates::AppType;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// CI platform type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CiPlatform {
    /// GitHub Actions
    GitHubActions,
    /// GitLab CI
    GitLabCi,
    /// CircleCI
    CircleCi,
}

impl std::fmt::Display for CiPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::GitHubActions => write!(f, "GitHub Actions"),
            Self::GitLabCi => write!(f, "GitLab CI"),
            Self::CircleCi => write!(f, "CircleCI"),
        }
    }
}

/// Target platform for builds
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum BuildPlatform {
    /// Linux x64
    LinuxX64,
    /// Linux ARM64
    LinuxArm64,
    /// macOS x64 (Intel)
    MacOSX64,
    /// macOS ARM64 (Apple Silicon)
    MacOSArm64,
    /// Windows x64
    WindowsX64,
}

impl BuildPlatform {
    /// Get the GitHub Actions runner for this platform
    pub fn github_runner(&self) -> &'static str {
        match self {
            Self::LinuxX64 => "ubuntu-latest",
            Self::LinuxArm64 => "ubuntu-latest", // ARM builds need QEMU
            Self::MacOSX64 => "macos-13",
            Self::MacOSArm64 => "macos-14",
            Self::WindowsX64 => "windows-latest",
        }
    }

    /// Get the Rust target triple
    pub fn rust_target(&self) -> &'static str {
        match self {
            Self::LinuxX64 => "x86_64-unknown-linux-gnu",
            Self::LinuxArm64 => "aarch64-unknown-linux-gnu",
            Self::MacOSX64 => "x86_64-apple-darwin",
            Self::MacOSArm64 => "aarch64-apple-darwin",
            Self::WindowsX64 => "x86_64-pc-windows-msvc",
        }
    }

    /// Get the artifact extension for this platform
    pub fn artifact_extension(&self) -> &'static str {
        match self {
            Self::WindowsX64 => ".exe",
            _ => "",
        }
    }
}

impl std::fmt::Display for BuildPlatform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LinuxX64 => write!(f, "linux-x64"),
            Self::LinuxArm64 => write!(f, "linux-arm64"),
            Self::MacOSX64 => write!(f, "macos-x64"),
            Self::MacOSArm64 => write!(f, "macos-arm64"),
            Self::WindowsX64 => write!(f, "windows-x64"),
        }
    }
}

/// Configuration for CI workflow generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CiConfig {
    /// Application name
    pub app_name: String,
    /// Application type
    #[serde(default)]
    pub app_type: AppType,
    /// Rust version
    #[serde(default = "default_rust_version")]
    pub rust_version: String,
    /// Target platforms to build for
    #[serde(default = "default_platforms")]
    pub platforms: Vec<BuildPlatform>,
    /// Enable code signing
    #[serde(default)]
    pub enable_signing: bool,
    /// Enable artifact upload to releases
    #[serde(default)]
    pub upload_artifacts: bool,
    /// Run tests
    #[serde(default = "default_true")]
    pub run_tests: bool,
    /// Run clippy
    #[serde(default = "default_true")]
    pub run_clippy: bool,
    /// Run rustfmt check
    #[serde(default = "default_true")]
    pub run_fmt_check: bool,
    /// Build Docker image
    #[serde(default)]
    pub build_docker: bool,
    /// Docker registry
    #[serde(default)]
    pub docker_registry: Option<String>,
    /// Environment schema for secrets
    #[serde(skip)]
    pub env_schema: Option<EnvSchema>,
    /// Branches to run on
    #[serde(default = "default_branches")]
    pub branches: Vec<String>,
    /// Enable caching
    #[serde(default = "default_true")]
    pub enable_cache: bool,
}

fn default_rust_version() -> String {
    "stable".to_string()
}

fn default_platforms() -> Vec<BuildPlatform> {
    vec![BuildPlatform::LinuxX64]
}

fn default_true() -> bool {
    true
}

fn default_branches() -> Vec<String> {
    vec!["main".to_string(), "master".to_string()]
}

impl Default for CiConfig {
    fn default() -> Self {
        Self {
            app_name: "oxide-app".to_string(),
            app_type: AppType::RustBinary,
            rust_version: default_rust_version(),
            platforms: default_platforms(),
            enable_signing: false,
            upload_artifacts: false,
            run_tests: true,
            run_clippy: true,
            run_fmt_check: true,
            build_docker: false,
            docker_registry: None,
            env_schema: None,
            branches: default_branches(),
            enable_cache: true,
        }
    }
}

impl CiConfig {
    /// Create a new CI configuration
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

    /// Set target platforms
    pub fn with_platforms(mut self, platforms: Vec<BuildPlatform>) -> Self {
        self.platforms = platforms;
        self
    }

    /// Enable all desktop platforms
    pub fn with_desktop_platforms(mut self) -> Self {
        self.platforms = vec![
            BuildPlatform::LinuxX64,
            BuildPlatform::MacOSX64,
            BuildPlatform::MacOSArm64,
            BuildPlatform::WindowsX64,
        ];
        self
    }

    /// Enable code signing
    pub fn with_signing(mut self) -> Self {
        self.enable_signing = true;
        self
    }

    /// Enable artifact uploads
    pub fn with_artifacts(mut self) -> Self {
        self.upload_artifacts = true;
        self
    }

    /// Enable Docker builds
    pub fn with_docker(mut self, registry: Option<String>) -> Self {
        self.build_docker = true;
        self.docker_registry = registry;
        self
    }
}

/// CI workflow generator
pub struct CiGenerator {
    config: CiConfig,
}

impl CiGenerator {
    /// Create a new CI generator
    pub fn new(config: CiConfig) -> Self {
        Self { config }
    }

    /// Generate a workflow for the specified platform
    pub fn generate(&self, platform: CiPlatform) -> DeployResult<String> {
        match platform {
            CiPlatform::GitHubActions => self.generate_github_actions(),
            CiPlatform::GitLabCi => self.generate_gitlab_ci(),
            CiPlatform::CircleCi => self.generate_circle_ci(),
        }
    }

    /// Generate GitHub Actions workflow
    pub fn generate_github_actions(&self) -> DeployResult<String> {
        let branches = self.config.branches.join("\", \"");

        let mut workflow = format!(
            r#"name: CI

on:
  push:
    branches: ["{branches}"]
    tags:
      - 'v*'
  pull_request:
    branches: ["{branches}"]

env:
  CARGO_TERM_COLOR: always
  RUST_BACKTRACE: 1

jobs:
"#,
            branches = branches,
        );

        // Check job (lint, format, test)
        if self.config.run_tests || self.config.run_clippy || self.config.run_fmt_check {
            workflow.push_str(&self.generate_check_job());
        }

        // Build jobs for each platform
        for platform in &self.config.platforms {
            workflow.push_str(&self.generate_build_job(platform));
        }

        // Docker build job
        if self.config.build_docker {
            workflow.push_str(&self.generate_docker_job());
        }

        // Release job
        if self.config.upload_artifacts {
            workflow.push_str(&self.generate_release_job());
        }

        Ok(workflow)
    }

    fn generate_check_job(&self) -> String {
        let mut job = String::from(
            r#"
  check:
    name: Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          components: rustfmt, clippy
"#,
        );

        if self.config.enable_cache {
            job.push_str(
                r#"
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
"#,
            );
        }

        if self.config.run_fmt_check {
            job.push_str(
                r#"
      - name: Check formatting
        run: cargo fmt --all -- --check
"#,
            );
        }

        if self.config.run_clippy {
            job.push_str(
                r#"
      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings
"#,
            );
        }

        if self.config.run_tests {
            job.push_str(
                r#"
      - name: Run tests
        run: cargo test --all-features
"#,
            );
        }

        job
    }

    fn generate_build_job(&self, platform: &BuildPlatform) -> String {
        let runner = platform.github_runner();
        let target = platform.rust_target();
        let ext = platform.artifact_extension();
        let needs = if self.config.run_tests || self.config.run_clippy || self.config.run_fmt_check {
            "\n    needs: check"
        } else {
            ""
        };

        let mut job = format!(
            r#"
  build-{platform}:
    name: Build ({platform})
    runs-on: {runner}{needs}
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust toolchain
        uses: dtolnay/rust-action@stable
        with:
          targets: {target}
"#,
            platform = platform,
            runner = runner,
            target = target,
            needs = needs,
        );

        if self.config.enable_cache {
            job.push_str(&format!(
                r#"
      - name: Cache cargo registry
        uses: actions/cache@v4
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            target
          key: ${{{{ runner.os }}}}-cargo-{platform}-${{{{ hashFiles('**/Cargo.lock') }}}}
"#,
                platform = platform,
            ));
        }

        // Linux ARM64 needs cross-compilation setup
        if *platform == BuildPlatform::LinuxArm64 {
            job.push_str(
                r#"
      - name: Install cross-compilation tools
        run: |
          sudo apt-get update
          sudo apt-get install -y gcc-aarch64-linux-gnu
"#,
            );
        }

        job.push_str(&format!(
            r#"
      - name: Build release binary
        run: cargo build --release --target {target}
"#,
            target = target,
        ));

        // Signing for macOS
        if self.config.enable_signing && platform.github_runner().starts_with("macos") {
            job.push_str(
                r#"
      - name: Import signing certificate
        if: github.event_name != 'pull_request'
        env:
          APPLE_CERTIFICATE: ${{ secrets.APPLE_CERTIFICATE }}
          APPLE_CERTIFICATE_PASSWORD: ${{ secrets.APPLE_CERTIFICATE_PASSWORD }}
        run: |
          echo "$APPLE_CERTIFICATE" | base64 --decode > certificate.p12
          security create-keychain -p "" build.keychain
          security import certificate.p12 -k build.keychain -P "$APPLE_CERTIFICATE_PASSWORD" -T /usr/bin/codesign
          security set-key-partition-list -S apple-tool:,apple:,codesign: -s -k "" build.keychain
          security default-keychain -s build.keychain
          security unlock-keychain -p "" build.keychain

      - name: Sign binary
        if: github.event_name != 'pull_request'
        run: |
          codesign --force --deep --sign "${{ secrets.APPLE_SIGNING_IDENTITY }}" target/{target}/release/{app_name}
"#,
            );
        }

        // Signing for Windows
        if self.config.enable_signing && *platform == BuildPlatform::WindowsX64 {
            job.push_str(
                r#"
      - name: Sign binary
        if: github.event_name != 'pull_request'
        env:
          WINDOWS_CERTIFICATE: ${{ secrets.WINDOWS_CERTIFICATE }}
          WINDOWS_CERTIFICATE_PASSWORD: ${{ secrets.WINDOWS_CERTIFICATE_PASSWORD }}
        run: |
          $cert = [System.Convert]::FromBase64String($env:WINDOWS_CERTIFICATE)
          [System.IO.File]::WriteAllBytes("certificate.pfx", $cert)
          & signtool sign /f certificate.pfx /p $env:WINDOWS_CERTIFICATE_PASSWORD /tr http://timestamp.digicert.com /td sha256 "target\{target}\release\{app_name}.exe"
        shell: pwsh
"#,
            );
        }

        if self.config.upload_artifacts {
            job.push_str(&format!(
                r#"
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: {app_name}-{platform}{ext}
          path: target/{target}/release/{app_name}{ext}
          if-no-files-found: error
"#,
                app_name = self.config.app_name,
                platform = platform,
                target = target,
                ext = ext,
            ));
        }

        job
    }

    fn generate_docker_job(&self) -> String {
        let registry = self
            .config
            .docker_registry
            .as_deref()
            .unwrap_or("ghcr.io");

        let needs = if self.config.run_tests || self.config.run_clippy || self.config.run_fmt_check {
            "\n    needs: check"
        } else {
            ""
        };

        format!(
            r#"
  docker:
    name: Build Docker Image
    runs-on: ubuntu-latest{needs}
    permissions:
      contents: read
      packages: write
    steps:
      - uses: actions/checkout@v4

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Log in to registry
        if: github.event_name != 'pull_request'
        uses: docker/login-action@v3
        with:
          registry: {registry}
          username: ${{{{ github.actor }}}}
          password: ${{{{ secrets.GITHUB_TOKEN }}}}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: {registry}/${{{{ github.repository }}}}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{{{version}}}}
            type=sha

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          push: ${{{{ github.event_name != 'pull_request' }}}}
          tags: ${{{{ steps.meta.outputs.tags }}}}
          labels: ${{{{ steps.meta.outputs.labels }}}}
          cache-from: type=gha
          cache-to: type=gha,mode=max
"#,
            needs = needs,
            registry = registry,
        )
    }

    fn generate_release_job(&self) -> String {
        let platform_artifacts: Vec<String> = self
            .config
            .platforms
            .iter()
            .map(|p| format!("          {}-{}*", self.config.app_name, p))
            .collect();

        let needs: Vec<String> = self
            .config
            .platforms
            .iter()
            .map(|p| format!("build-{}", p))
            .collect();

        format!(
            r#"
  release:
    name: Create Release
    runs-on: ubuntu-latest
    needs: [{needs}]
    if: startsWith(github.ref, 'refs/tags/')
    permissions:
      contents: write
    steps:
      - uses: actions/checkout@v4

      - name: Download all artifacts
        uses: actions/download-artifact@v4
        with:
          path: artifacts

      - name: Create release
        uses: softprops/action-gh-release@v1
        with:
          files: |
{artifacts}
          generate_release_notes: true
          draft: false
          prerelease: ${{{{ contains(github.ref, '-') }}}}
"#,
            needs = needs.join(", "),
            artifacts = platform_artifacts.join("\n"),
        )
    }

    /// Generate GitLab CI configuration
    pub fn generate_gitlab_ci(&self) -> DeployResult<String> {
        let template = format!(
            r#"stages:
  - check
  - build
  - release

variables:
  CARGO_HOME: $CI_PROJECT_DIR/.cargo
  RUST_BACKTRACE: "1"

.rust-cache:
  cache:
    key: rust-$CI_JOB_NAME
    paths:
      - .cargo/
      - target/

check:
  stage: check
  image: rust:{rust_version}
  extends: .rust-cache
  script:
    - rustup component add rustfmt clippy
    - cargo fmt --all -- --check
    - cargo clippy --all-targets --all-features -- -D warnings
    - cargo test --all-features

build:
  stage: build
  image: rust:{rust_version}
  extends: .rust-cache
  script:
    - cargo build --release
  artifacts:
    paths:
      - target/release/{app_name}
    expire_in: 1 week

release:
  stage: release
  image: registry.gitlab.com/gitlab-org/release-cli:latest
  rules:
    - if: $CI_COMMIT_TAG
  script:
    - echo "Creating release $CI_COMMIT_TAG"
  release:
    tag_name: $CI_COMMIT_TAG
    description: "Release $CI_COMMIT_TAG"
    assets:
      links:
        - name: "{app_name}"
          url: "$CI_PROJECT_URL/-/jobs/artifacts/$CI_COMMIT_TAG/raw/target/release/{app_name}?job=build"
"#,
            rust_version = self.config.rust_version,
            app_name = self.config.app_name,
        );

        Ok(template)
    }

    /// Generate CircleCI configuration
    pub fn generate_circle_ci(&self) -> DeployResult<String> {
        let template = format!(
            r#"version: 2.1

orbs:
  rust: circleci/rust@1.6

jobs:
  check:
    docker:
      - image: cimg/rust:{rust_version}
    steps:
      - checkout
      - rust/install
      - rust/clippy:
          with_cache: true
      - rust/format:
          with_cache: true
      - rust/test:
          with_cache: true

  build:
    docker:
      - image: cimg/rust:{rust_version}
    steps:
      - checkout
      - rust/install
      - rust/build:
          with_cache: true
          release: true
      - persist_to_workspace:
          root: .
          paths:
            - target/release/{app_name}

  release:
    docker:
      - image: cimg/base:stable
    steps:
      - attach_workspace:
          at: .
      - run:
          name: Create release
          command: |
            echo "Creating release for $CIRCLE_TAG"
            # Add release creation logic here

workflows:
  version: 2
  build-and-release:
    jobs:
      - check:
          filters:
            tags:
              only: /.*/
      - build:
          requires:
            - check
          filters:
            tags:
              only: /.*/
      - release:
          requires:
            - build
          filters:
            branches:
              ignore: /.*/
            tags:
              only: /^v.*/
"#,
            rust_version = self.config.rust_version,
            app_name = self.config.app_name,
        );

        Ok(template)
    }

    /// Generate all CI configurations and save to a directory
    pub fn generate_all(&self, output_dir: impl AsRef<Path>) -> DeployResult<Vec<(CiPlatform, String)>> {
        let dir = output_dir.as_ref();
        std::fs::create_dir_all(dir)?;

        let configs = vec![
            (
                CiPlatform::GitHubActions,
                ".github/workflows/ci.yml",
                self.generate_github_actions()?,
            ),
            (
                CiPlatform::GitLabCi,
                ".gitlab-ci.yml",
                self.generate_gitlab_ci()?,
            ),
            (
                CiPlatform::CircleCi,
                ".circleci/config.yml",
                self.generate_circle_ci()?,
            ),
        ];

        let mut results = Vec::new();
        for (platform, path, content) in configs {
            let full_path = dir.join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(&full_path, &content)?;
            results.push((platform, full_path.display().to_string()));
        }

        Ok(results)
    }
}

/// Generate documentation for setting up CI
pub fn generate_ci_setup_docs(config: &CiConfig) -> String {
    let mut lines = Vec::new();

    lines.push("# CI/CD Setup Guide".to_string());
    lines.push(String::new());

    lines.push("## GitHub Actions".to_string());
    lines.push(String::new());

    lines.push("### Required Secrets".to_string());
    lines.push(String::new());
    lines.push("Add these secrets in: **Settings > Secrets and variables > Actions**".to_string());
    lines.push(String::new());

    if config.enable_signing {
        lines.push("#### Code Signing (macOS)".to_string());
        lines.push("- `APPLE_CERTIFICATE` - Base64-encoded .p12 certificate".to_string());
        lines.push("- `APPLE_CERTIFICATE_PASSWORD` - Certificate password".to_string());
        lines.push("- `APPLE_SIGNING_IDENTITY` - Signing identity name".to_string());
        lines.push(String::new());

        lines.push("#### Code Signing (Windows)".to_string());
        lines.push("- `WINDOWS_CERTIFICATE` - Base64-encoded .pfx certificate".to_string());
        lines.push("- `WINDOWS_CERTIFICATE_PASSWORD` - Certificate password".to_string());
        lines.push(String::new());
    }

    if let Some(ref schema) = config.env_schema {
        let secrets: Vec<_> = schema.secret_variables();
        if !secrets.is_empty() {
            lines.push("#### Application Secrets".to_string());
            for secret in secrets {
                lines.push(format!("- `{}`", secret.name));
            }
            lines.push(String::new());
        }
    }

    lines.push("### Setting Up Signing Certificates".to_string());
    lines.push(String::new());

    lines.push("#### macOS".to_string());
    lines.push("1. Export your Developer ID certificate from Keychain Access".to_string());
    lines.push("2. Base64 encode: `base64 -i certificate.p12 -o certificate.txt`".to_string());
    lines.push("3. Add the content as `APPLE_CERTIFICATE` secret".to_string());
    lines.push(String::new());

    lines.push("#### Windows".to_string());
    lines.push("1. Obtain a code signing certificate (.pfx)".to_string());
    lines.push("2. Base64 encode: `certutil -encode certificate.pfx certificate.txt`".to_string());
    lines.push("3. Add the content as `WINDOWS_CERTIFICATE` secret".to_string());
    lines.push(String::new());

    lines.push("## Build Platforms".to_string());
    lines.push(String::new());
    for platform in &config.platforms {
        lines.push(format!(
            "- **{}**: {} ({})",
            platform,
            platform.github_runner(),
            platform.rust_target()
        ));
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ci_config_creation() {
        let config = CiConfig::new("my-app")
            .with_desktop_platforms()
            .with_signing()
            .with_artifacts();

        assert_eq!(config.app_name, "my-app");
        assert_eq!(config.platforms.len(), 4);
        assert!(config.enable_signing);
        assert!(config.upload_artifacts);
    }

    #[test]
    fn test_build_platform_properties() {
        assert_eq!(
            BuildPlatform::LinuxX64.github_runner(),
            "ubuntu-latest"
        );
        assert_eq!(
            BuildPlatform::MacOSArm64.rust_target(),
            "aarch64-apple-darwin"
        );
        assert_eq!(BuildPlatform::WindowsX64.artifact_extension(), ".exe");
        assert_eq!(BuildPlatform::LinuxX64.artifact_extension(), "");
    }

    #[test]
    fn test_github_actions_generation() {
        let config = CiConfig::new("test-app");
        let generator = CiGenerator::new(config);
        let workflow = generator.generate_github_actions().unwrap();

        assert!(workflow.contains("name: CI"));
        assert!(workflow.contains("cargo test"));
        assert!(workflow.contains("cargo clippy"));
        assert!(workflow.contains("cargo fmt"));
    }

    #[test]
    fn test_github_actions_with_docker() {
        let config = CiConfig::new("test-app").with_docker(Some("ghcr.io".to_string()));
        let generator = CiGenerator::new(config);
        let workflow = generator.generate_github_actions().unwrap();

        assert!(workflow.contains("docker:"));
        assert!(workflow.contains("Docker Buildx"));
        assert!(workflow.contains("ghcr.io"));
    }

    #[test]
    fn test_gitlab_ci_generation() {
        let config = CiConfig::new("test-app");
        let generator = CiGenerator::new(config);
        let gitlab = generator.generate_gitlab_ci().unwrap();

        assert!(gitlab.contains("stages:"));
        assert!(gitlab.contains("cargo build --release"));
        assert!(gitlab.contains("cargo test"));
    }

    #[test]
    fn test_circle_ci_generation() {
        let config = CiConfig::new("test-app");
        let generator = CiGenerator::new(config);
        let circle = generator.generate_circle_ci().unwrap();

        assert!(circle.contains("version: 2.1"));
        assert!(circle.contains("orbs:"));
        assert!(circle.contains("rust/build"));
    }

    #[test]
    fn test_ci_setup_docs() {
        let config = CiConfig::new("test-app")
            .with_desktop_platforms()
            .with_signing();

        let docs = generate_ci_setup_docs(&config);
        assert!(docs.contains("APPLE_CERTIFICATE"));
        assert!(docs.contains("WINDOWS_CERTIFICATE"));
        assert!(docs.contains("Build Platforms"));
    }

    #[test]
    fn test_multiplatform_build() {
        let config = CiConfig::new("test-app").with_platforms(vec![
            BuildPlatform::LinuxX64,
            BuildPlatform::MacOSArm64,
            BuildPlatform::WindowsX64,
        ]);
        let generator = CiGenerator::new(config);
        let workflow = generator.generate_github_actions().unwrap();

        assert!(workflow.contains("build-linux-x64"));
        assert!(workflow.contains("build-macos-arm64"));
        assert!(workflow.contains("build-windows-x64"));
    }
}
