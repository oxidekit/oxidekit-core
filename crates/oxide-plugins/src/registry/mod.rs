//! Registry client for the OxideKit plugin marketplace.
//!
//! Provides access to the official OxideKit plugin registry for:
//!
//! - Searching and discovering plugins
//! - Downloading plugin packages
//! - Publishing plugins
//! - Verifying signatures

use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::error::{PluginError, PluginResult};
use crate::trust::TrustLevel;

/// Default registry URL.
pub const DEFAULT_REGISTRY_URL: &str = "https://registry.oxidekit.com";

/// Client for interacting with the plugin registry.
#[derive(Debug, Clone)]
pub struct RegistryClient {
    /// Registry base URL.
    base_url: String,
    /// Authentication token (if authenticated).
    auth_token: Option<String>,
}

impl Default for RegistryClient {
    fn default() -> Self {
        Self {
            base_url: DEFAULT_REGISTRY_URL.to_string(),
            auth_token: None,
        }
    }
}

impl RegistryClient {
    /// Create a new registry client with the default URL.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a client with a custom registry URL.
    pub fn with_url(url: &str) -> Self {
        Self {
            base_url: url.to_string(),
            auth_token: None,
        }
    }

    /// Set the authentication token.
    pub fn with_auth(mut self, token: &str) -> Self {
        self.auth_token = Some(token.to_string());
        self
    }

    /// Get the registry base URL.
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Search for plugins.
    pub fn search(&self, query: &SearchQuery) -> PluginResult<SearchResults> {
        // In a real implementation, this would make an HTTP request
        // For now, return a placeholder error
        Err(PluginError::RegistryError(format!(
            "Registry search not yet implemented. Would search: {:?}",
            query
        )))
    }

    /// Get plugin metadata.
    pub fn get_plugin(&self, plugin_id: &str) -> PluginResult<PluginInfo> {
        Err(PluginError::RegistryError(format!(
            "Registry lookup not yet implemented for: {}",
            plugin_id
        )))
    }

    /// Get a specific version of a plugin.
    pub fn get_version(&self, plugin_id: &str, version: &str) -> PluginResult<VersionInfo> {
        Err(PluginError::RegistryError(format!(
            "Registry lookup not yet implemented for: {}@{}",
            plugin_id, version
        )))
    }

    /// Download a plugin package.
    pub fn download(&self, plugin_id: &str, version: &str) -> PluginResult<Vec<u8>> {
        Err(PluginError::RegistryError(format!(
            "Registry download not yet implemented for: {}@{}",
            plugin_id, version
        )))
    }

    /// Publish a plugin to the registry.
    pub fn publish(&self, _package: &PublishPackage) -> PluginResult<PublishResult> {
        if self.auth_token.is_none() {
            return Err(PluginError::RegistryError(
                "Authentication required for publishing".to_string()
            ));
        }

        Err(PluginError::RegistryError(
            "Registry publishing not yet implemented".to_string()
        ))
    }

    /// Verify a publisher's identity.
    pub fn verify_publisher(&self, publisher: &str) -> PluginResult<PublisherInfo> {
        Err(PluginError::RegistryError(format!(
            "Publisher verification not yet implemented for: {}",
            publisher
        )))
    }

    /// Get the list of verified publishers.
    pub fn verified_publishers(&self) -> PluginResult<Vec<PublisherInfo>> {
        Err(PluginError::RegistryError(
            "Verified publishers list not yet implemented".to_string()
        ))
    }
}

/// Search query for plugins.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchQuery {
    /// Text query.
    pub query: Option<String>,
    /// Filter by namespace.
    pub namespace: Option<String>,
    /// Filter by category.
    pub category: Option<String>,
    /// Filter by keywords.
    pub keywords: Vec<String>,
    /// Filter by trust level.
    pub trust_level: Option<TrustLevel>,
    /// Sort order.
    pub sort: SearchSort,
    /// Number of results per page.
    pub limit: u32,
    /// Offset for pagination.
    pub offset: u32,
}

impl SearchQuery {
    /// Create a new search query.
    pub fn new(query: &str) -> Self {
        Self {
            query: Some(query.to_string()),
            limit: 20,
            ..Default::default()
        }
    }

    /// Filter by namespace.
    pub fn namespace(mut self, namespace: &str) -> Self {
        self.namespace = Some(namespace.to_string());
        self
    }

    /// Filter by category.
    pub fn category(mut self, category: &str) -> Self {
        self.category = Some(category.to_string());
        self
    }

    /// Filter by keywords.
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.keywords = keywords;
        self
    }

    /// Filter by trust level.
    pub fn trust_level(mut self, level: TrustLevel) -> Self {
        self.trust_level = Some(level);
        self
    }

    /// Set sort order.
    pub fn sort(mut self, sort: SearchSort) -> Self {
        self.sort = sort;
        self
    }

    /// Set pagination.
    pub fn paginate(mut self, limit: u32, offset: u32) -> Self {
        self.limit = limit;
        self.offset = offset;
        self
    }
}

/// Sort order for search results.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SearchSort {
    /// Sort by relevance (default).
    #[default]
    Relevance,
    /// Sort by download count.
    Downloads,
    /// Sort by recently updated.
    RecentlyUpdated,
    /// Sort by newest.
    Newest,
    /// Sort alphabetically.
    Alphabetical,
}

/// Search results from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    /// Matching plugins.
    pub plugins: Vec<PluginInfo>,
    /// Total number of matches.
    pub total: u32,
    /// Current offset.
    pub offset: u32,
    /// Number of results returned.
    pub limit: u32,
}

/// Information about a plugin from the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginInfo {
    /// Plugin ID.
    pub id: String,
    /// Plugin name (display name).
    pub name: String,
    /// Description.
    pub description: String,
    /// Latest version.
    pub latest_version: String,
    /// All available versions.
    pub versions: Vec<String>,
    /// Publisher name.
    pub publisher: String,
    /// Trust level.
    pub trust_level: TrustLevel,
    /// Plugin category.
    pub category: String,
    /// Keywords.
    pub keywords: Vec<String>,
    /// Download count.
    pub downloads: u64,
    /// Repository URL.
    pub repository: Option<String>,
    /// Homepage URL.
    pub homepage: Option<String>,
    /// License.
    pub license: String,
    /// Last updated timestamp.
    pub updated_at: String,
    /// Created timestamp.
    pub created_at: String,
}

/// Information about a specific plugin version.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Version string.
    pub version: String,
    /// Package hash (SHA-256).
    pub hash: String,
    /// Package size in bytes.
    pub size: u64,
    /// Required core version.
    pub requires_core: Option<String>,
    /// Dependencies.
    pub dependencies: Vec<DependencyInfo>,
    /// Download URL.
    pub download_url: String,
    /// Signature (for verified publishers).
    pub signature: Option<String>,
    /// Published timestamp.
    pub published_at: String,
    /// Yanked status.
    pub yanked: bool,
}

/// Information about a dependency.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyInfo {
    /// Plugin ID.
    pub id: String,
    /// Version requirement.
    pub version_req: String,
    /// Whether optional.
    pub optional: bool,
}

/// Information about a publisher.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublisherInfo {
    /// Publisher name/ID.
    pub name: String,
    /// Display name.
    pub display_name: String,
    /// Trust level.
    pub trust_level: TrustLevel,
    /// Verification status.
    pub verified: bool,
    /// Website.
    pub website: Option<String>,
    /// Email (public).
    pub email: Option<String>,
    /// Number of published plugins.
    pub plugin_count: u32,
    /// Total downloads across all plugins.
    pub total_downloads: u64,
}

/// Package to publish to the registry.
#[derive(Debug, Clone)]
pub struct PublishPackage {
    /// Path to the plugin directory.
    pub plugin_path: std::path::PathBuf,
    /// Readme content.
    pub readme: Option<String>,
    /// Whether this is a dry run.
    pub dry_run: bool,
}

/// Result of a publish operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PublishResult {
    /// Plugin ID.
    pub plugin_id: String,
    /// Published version.
    pub version: String,
    /// Package hash.
    pub hash: String,
    /// Package URL.
    pub url: String,
}

/// Registry statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryStats {
    /// Total number of plugins.
    pub total_plugins: u32,
    /// Total number of versions.
    pub total_versions: u32,
    /// Total downloads.
    pub total_downloads: u64,
    /// Number of verified publishers.
    pub verified_publishers: u32,
    /// Plugins by category.
    pub by_category: HashMap<String, u32>,
    /// Plugins by namespace.
    pub by_namespace: HashMap<String, u32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_registry_client_creation() {
        let client = RegistryClient::new();
        assert_eq!(client.base_url(), DEFAULT_REGISTRY_URL);
    }

    #[test]
    fn test_registry_client_custom_url() {
        let client = RegistryClient::with_url("https://custom.registry.dev");
        assert_eq!(client.base_url(), "https://custom.registry.dev");
    }

    #[test]
    fn test_search_query() {
        let query = SearchQuery::new("tables")
            .namespace("ui")
            .trust_level(TrustLevel::Verified)
            .sort(SearchSort::Downloads)
            .paginate(10, 0);

        assert_eq!(query.query, Some("tables".to_string()));
        assert_eq!(query.namespace, Some("ui".to_string()));
        assert_eq!(query.limit, 10);
        assert_eq!(query.offset, 0);
    }

    #[test]
    fn test_publish_requires_auth() {
        let client = RegistryClient::new();
        let package = PublishPackage {
            plugin_path: std::path::PathBuf::from("/test"),
            readme: None,
            dry_run: true,
        };

        let result = client.publish(&package);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Authentication"));
    }
}
