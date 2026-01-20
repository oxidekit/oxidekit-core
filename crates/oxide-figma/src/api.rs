//! Figma API client
//!
//! Provides authenticated access to the Figma REST API for fetching
//! files, nodes, images, and variables.

use crate::error::{FigmaError, Result};
use crate::types::*;
use reqwest::{header, Client, StatusCode};
use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::env;
use std::time::Duration;
use tracing::{debug, info, warn};
use url::Url;

/// Figma API base URL
const FIGMA_API_BASE: &str = "https://api.figma.com/v1";

/// Configuration for Figma API client
#[derive(Debug, Clone)]
pub struct FigmaConfig {
    /// Personal access token
    pub token: String,

    /// Request timeout
    pub timeout: Duration,

    /// Max retries on failure
    pub max_retries: u32,

    /// Base URL override (for testing)
    pub base_url: Option<String>,
}

impl FigmaConfig {
    /// Create config from environment variable
    pub fn from_env() -> Result<Self> {
        let token = env::var("FIGMA_TOKEN")
            .map_err(|_| FigmaError::MissingToken)?;

        Ok(Self {
            token,
            timeout: Duration::from_secs(30),
            max_retries: 3,
            base_url: None,
        })
    }

    /// Create config with explicit token
    pub fn with_token(token: impl Into<String>) -> Self {
        Self {
            token: token.into(),
            timeout: Duration::from_secs(30),
            max_retries: 3,
            base_url: None,
        }
    }

    /// Set timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set max retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Override base URL (for testing)
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }
}

/// Figma API client
#[derive(Debug, Clone)]
pub struct FigmaClient {
    client: Client,
    config: FigmaConfig,
    base_url: String,
}

impl FigmaClient {
    /// Create a new Figma client
    pub fn new(config: FigmaConfig) -> Self {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Figma-Token",
            header::HeaderValue::from_str(&config.token).unwrap(),
        );

        let client = Client::builder()
            .default_headers(headers)
            .timeout(config.timeout)
            .build()
            .expect("Failed to build HTTP client");

        let base_url = config
            .base_url
            .clone()
            .unwrap_or_else(|| FIGMA_API_BASE.to_string());

        Self {
            client,
            config,
            base_url,
        }
    }

    /// Parse a Figma URL and extract file key and optional node ID
    pub fn parse_url(url: &str) -> Result<FigmaUrlInfo> {
        let parsed = Url::parse(url)
            .map_err(|e| FigmaError::InvalidUrl(e.to_string()))?;

        let host = parsed.host_str()
            .ok_or_else(|| FigmaError::InvalidUrl("No host in URL".into()))?;

        if !host.contains("figma.com") {
            return Err(FigmaError::InvalidUrl("Not a Figma URL".into()));
        }

        let path_segments: Vec<&str> = parsed.path()
            .trim_start_matches('/')
            .split('/')
            .collect();

        let (file_key, node_id) = match path_segments.as_slice() {
            // Pattern: /file/{key}/...
            ["file", key, ..] | ["design", key, ..] => {
                let node_id = parsed.query_pairs()
                    .find(|(k, _)| k == "node-id")
                    .map(|(_, v)| v.into_owned());
                (key.to_string(), node_id)
            }
            _ => return Err(FigmaError::InvalidUrl("Could not extract file key".into())),
        };

        Ok(FigmaUrlInfo { file_key, node_id })
    }

    /// Get a Figma file
    pub async fn get_file(&self, file_key: &str) -> Result<FigmaFile> {
        info!(file_key, "Fetching Figma file");
        let url = format!("{}/files/{}", self.base_url, file_key);
        self.get(&url).await
    }

    /// Get a Figma file with specific geometry
    pub async fn get_file_with_geometry(&self, file_key: &str) -> Result<FigmaFile> {
        info!(file_key, "Fetching Figma file with geometry");
        let url = format!(
            "{}/files/{}?geometry=paths&plugin_data=shared",
            self.base_url, file_key
        );
        self.get(&url).await
    }

    /// Get specific nodes from a file
    pub async fn get_nodes(&self, file_key: &str, node_ids: &[&str]) -> Result<NodesResponse> {
        info!(file_key, node_count = node_ids.len(), "Fetching specific nodes");
        let ids = node_ids.join(",");
        let url = format!("{}/files/{}/nodes?ids={}", self.base_url, file_key, ids);
        self.get(&url).await
    }

    /// Get file styles
    pub async fn get_styles(&self, file_key: &str) -> Result<StylesResponse> {
        info!(file_key, "Fetching file styles");
        let url = format!("{}/files/{}/styles", self.base_url, file_key);
        self.get(&url).await
    }

    /// Get local variables
    pub async fn get_local_variables(&self, file_key: &str) -> Result<VariablesResponse> {
        info!(file_key, "Fetching local variables");
        let url = format!("{}/files/{}/variables/local", self.base_url, file_key);
        self.get(&url).await
    }

    /// Get published variables
    pub async fn get_published_variables(&self, file_key: &str) -> Result<VariablesResponse> {
        info!(file_key, "Fetching published variables");
        let url = format!("{}/files/{}/variables/published", self.base_url, file_key);
        self.get(&url).await
    }

    /// Get images (rendered nodes)
    pub async fn get_images(
        &self,
        file_key: &str,
        node_ids: &[&str],
        format: ExportFormat,
        scale: f32,
    ) -> Result<ImagesResponse> {
        info!(
            file_key,
            node_count = node_ids.len(),
            ?format,
            scale,
            "Fetching rendered images"
        );

        let ids = node_ids.join(",");
        let format_str = match format {
            ExportFormat::Png => "png",
            ExportFormat::Jpg => "jpg",
            ExportFormat::Svg => "svg",
            ExportFormat::Pdf => "pdf",
        };

        let url = format!(
            "{}/images/{}?ids={}&format={}&scale={}",
            self.base_url, file_key, ids, format_str, scale
        );
        self.get(&url).await
    }

    /// Get image fills (background images used in fills)
    pub async fn get_image_fills(&self, file_key: &str) -> Result<ImageFillsResponse> {
        info!(file_key, "Fetching image fills");
        let url = format!("{}/files/{}/images", self.base_url, file_key);
        self.get(&url).await
    }

    /// Download an image from URL
    pub async fn download_image(&self, url: &str) -> Result<Vec<u8>> {
        debug!(url, "Downloading image");

        let response = self.client.get(url)
            .send()
            .await
            .map_err(|e| FigmaError::NetworkError(e.to_string()))?;

        if !response.status().is_success() {
            return Err(FigmaError::AssetDownloadFailed {
                asset: url.to_string(),
                reason: format!("HTTP {}", response.status()),
            });
        }

        response.bytes()
            .await
            .map(|b| b.to_vec())
            .map_err(|e| FigmaError::AssetDownloadFailed {
                asset: url.to_string(),
                reason: e.to_string(),
            })
    }

    /// Get file components
    pub async fn get_components(&self, file_key: &str) -> Result<ComponentsResponse> {
        info!(file_key, "Fetching file components");
        let url = format!("{}/files/{}/components", self.base_url, file_key);
        self.get(&url).await
    }

    /// Get team components
    pub async fn get_team_components(&self, team_id: &str) -> Result<ComponentsResponse> {
        info!(team_id, "Fetching team components");
        let url = format!("{}/teams/{}/components", self.base_url, team_id);
        self.get(&url).await
    }

    /// Make a GET request with retry logic
    async fn get<T: DeserializeOwned>(&self, url: &str) -> Result<T> {
        let mut attempts = 0;
        let max_attempts = self.config.max_retries + 1;

        loop {
            attempts += 1;
            debug!(url, attempts, "Making API request");

            let response = self.client.get(url).send().await;

            match response {
                Ok(resp) => {
                    let status = resp.status();

                    match status {
                        StatusCode::OK => {
                            let body = resp.text().await
                                .map_err(|e| FigmaError::NetworkError(e.to_string()))?;

                            return serde_json::from_str(&body)
                                .map_err(|e| FigmaError::ParseError(format!(
                                    "Failed to parse response: {}. Body: {}",
                                    e,
                                    &body[..body.len().min(500)]
                                )));
                        }
                        StatusCode::UNAUTHORIZED => {
                            return Err(FigmaError::AuthenticationFailed(
                                "Invalid or expired token".into()
                            ));
                        }
                        StatusCode::FORBIDDEN => {
                            return Err(FigmaError::AuthenticationFailed(
                                "Access denied to resource".into()
                            ));
                        }
                        StatusCode::NOT_FOUND => {
                            return Err(FigmaError::FileNotFound(url.to_string()));
                        }
                        StatusCode::TOO_MANY_REQUESTS => {
                            let retry_after = resp
                                .headers()
                                .get("retry-after")
                                .and_then(|v| v.to_str().ok())
                                .and_then(|v| v.parse().ok())
                                .unwrap_or(60);

                            if attempts < max_attempts {
                                warn!(retry_after, "Rate limited, waiting before retry");
                                tokio::time::sleep(Duration::from_secs(retry_after)).await;
                                continue;
                            }

                            return Err(FigmaError::RateLimited { retry_after });
                        }
                        _ => {
                            let body = resp.text().await.unwrap_or_default();
                            return Err(FigmaError::ApiError(format!(
                                "HTTP {}: {}",
                                status,
                                &body[..body.len().min(500)]
                            )));
                        }
                    }
                }
                Err(e) => {
                    if attempts < max_attempts && e.is_timeout() {
                        warn!("Request timed out, retrying...");
                        tokio::time::sleep(Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(FigmaError::NetworkError(e.to_string()));
                }
            }
        }
    }
}

/// Parsed Figma URL information
#[derive(Debug, Clone)]
pub struct FigmaUrlInfo {
    /// File key
    pub file_key: String,

    /// Optional node ID
    pub node_id: Option<String>,
}

/// Response containing nodes
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NodesResponse {
    pub name: String,
    #[serde(default)]
    pub nodes: HashMap<String, NodeWrapper>,
    pub last_modified: Option<String>,
    pub thumbnail_url: Option<String>,
    pub version: Option<String>,
}

/// Wrapper for node in nodes response
#[derive(Debug, Clone, serde::Deserialize)]
pub struct NodeWrapper {
    pub document: Node,
    #[serde(default)]
    pub components: HashMap<String, Component>,
    #[serde(default)]
    pub styles: HashMap<String, Style>,
}

/// Response containing styles
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StylesResponse {
    pub status: u32,
    pub error: bool,
    #[serde(default)]
    pub meta: StylesMeta,
}

/// Styles metadata
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct StylesMeta {
    #[serde(default)]
    pub styles: Vec<StyleMetaItem>,
}

/// Style metadata item
#[derive(Debug, Clone, serde::Deserialize)]
pub struct StyleMetaItem {
    pub key: String,
    pub name: String,
    #[serde(rename = "styleType")]
    pub style_type: StyleType,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub node_id: Option<String>,
}

/// Response containing components
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ComponentsResponse {
    pub status: Option<u32>,
    pub error: Option<bool>,
    #[serde(default)]
    pub meta: ComponentsMeta,
}

/// Components metadata
#[derive(Debug, Clone, Default, serde::Deserialize)]
pub struct ComponentsMeta {
    #[serde(default)]
    pub components: Vec<ComponentMetaItem>,
}

/// Component metadata item
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ComponentMetaItem {
    pub key: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub node_id: Option<String>,
    #[serde(default)]
    pub containing_frame: Option<ContainingFrame>,
}

/// Containing frame info
#[derive(Debug, Clone, serde::Deserialize)]
pub struct ContainingFrame {
    pub name: Option<String>,
    #[serde(rename = "nodeId")]
    pub node_id: Option<String>,
    #[serde(rename = "pageName")]
    pub page_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_figma_url() {
        let url = "https://www.figma.com/file/abc123/My-Design";
        let info = FigmaClient::parse_url(url).unwrap();
        assert_eq!(info.file_key, "abc123");
        assert!(info.node_id.is_none());
    }

    #[test]
    fn test_parse_figma_url_with_node() {
        let url = "https://www.figma.com/file/abc123/My-Design?node-id=1234-5678";
        let info = FigmaClient::parse_url(url).unwrap();
        assert_eq!(info.file_key, "abc123");
        assert_eq!(info.node_id, Some("1234-5678".to_string()));
    }

    #[test]
    fn test_parse_design_url() {
        let url = "https://www.figma.com/design/xyz789/Another-Design";
        let info = FigmaClient::parse_url(url).unwrap();
        assert_eq!(info.file_key, "xyz789");
    }

    #[test]
    fn test_invalid_url() {
        let url = "https://example.com/not-figma";
        assert!(FigmaClient::parse_url(url).is_err());
    }
}
