//! Authentication manager for coordinating multiple providers.

use chrono::Duration;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tracing::{debug, error, info, warn};

use crate::error::{AuthState, NetworkError, NetworkResult};
use crate::http::HttpRequest;

use super::{AuthCredentials, AuthProvider, AuthProviderType, TokenPair};

/// Event emitted when auth state changes.
#[derive(Debug, Clone)]
pub struct AuthStateChange {
    /// The provider ID.
    pub provider_id: String,
    /// The previous state.
    pub previous: AuthState,
    /// The new state.
    pub current: AuthState,
}

/// Manager for multiple authentication providers.
///
/// Coordinates auth across the application:
/// - Manages multiple providers
/// - Handles automatic token refresh
/// - Broadcasts auth state changes
/// - Provides unified request authentication
#[derive(Debug)]
pub struct AuthManager {
    /// Registered providers.
    providers: RwLock<HashMap<String, Arc<dyn AuthProvider>>>,
    /// Default provider ID.
    default_provider: RwLock<Option<String>>,
    /// Channel for auth state changes.
    state_tx: broadcast::Sender<AuthStateChange>,
    /// Token refresh threshold (refresh before expiry).
    refresh_threshold: Duration,
    /// Whether to automatically refresh tokens.
    auto_refresh: bool,
}

impl AuthManager {
    /// Create a new auth manager.
    pub fn new() -> Self {
        let (state_tx, _) = broadcast::channel(100);
        Self {
            providers: RwLock::new(HashMap::new()),
            default_provider: RwLock::new(None),
            state_tx,
            refresh_threshold: Duration::minutes(5),
            auto_refresh: true,
        }
    }

    /// Create a new auth manager with custom settings.
    pub fn with_config(refresh_threshold: Duration, auto_refresh: bool) -> Self {
        let (state_tx, _) = broadcast::channel(100);
        Self {
            providers: RwLock::new(HashMap::new()),
            default_provider: RwLock::new(None),
            state_tx,
            refresh_threshold,
            auto_refresh,
        }
    }

    /// Register an authentication provider.
    pub async fn register_provider(&self, provider: impl AuthProvider + 'static) {
        let id = provider.id().to_string();
        let provider = Arc::new(provider) as Arc<dyn AuthProvider>;

        let mut providers = self.providers.write().await;

        // Set as default if it's the first provider
        if providers.is_empty() {
            *self.default_provider.write().await = Some(id.clone());
        }

        providers.insert(id.clone(), provider);
        info!(provider_id = %id, "Registered auth provider");
    }

    /// Set the default provider.
    pub async fn set_default_provider(&self, provider_id: impl Into<String>) -> NetworkResult<()> {
        let id = provider_id.into();
        let providers = self.providers.read().await;

        if !providers.contains_key(&id) {
            return Err(NetworkError::ConfigError {
                message: format!("Provider '{}' not found", id),
            });
        }

        *self.default_provider.write().await = Some(id);
        Ok(())
    }

    /// Get a provider by ID.
    pub async fn get_provider(&self, provider_id: &str) -> Option<Arc<dyn AuthProvider>> {
        self.providers.read().await.get(provider_id).cloned()
    }

    /// Get the default provider.
    pub async fn get_default_provider(&self) -> Option<Arc<dyn AuthProvider>> {
        let default_id = self.default_provider.read().await.clone()?;
        self.get_provider(&default_id).await
    }

    /// Subscribe to auth state changes.
    pub fn subscribe_state_changes(&self) -> broadcast::Receiver<AuthStateChange> {
        self.state_tx.subscribe()
    }

    /// Authenticate with a specific provider.
    pub async fn authenticate(
        &self,
        provider_id: &str,
        credentials: AuthCredentials,
    ) -> NetworkResult<TokenPair> {
        let provider = self
            .get_provider(provider_id)
            .await
            .ok_or_else(|| NetworkError::ConfigError {
                message: format!("Provider '{}' not found", provider_id),
            })?;

        let previous_state = provider.state().await;

        // Notify state change to authenticating
        let _ = self.state_tx.send(AuthStateChange {
            provider_id: provider_id.to_string(),
            previous: previous_state.clone(),
            current: AuthState::Authenticating,
        });

        match provider.authenticate(credentials).await {
            Ok(token) => {
                info!(provider_id = %provider_id, "Authentication successful");

                let _ = self.state_tx.send(AuthStateChange {
                    provider_id: provider_id.to_string(),
                    previous: AuthState::Authenticating,
                    current: AuthState::Authenticated,
                });

                Ok(token)
            }
            Err(e) => {
                error!(provider_id = %provider_id, error = %e, "Authentication failed");

                let _ = self.state_tx.send(AuthStateChange {
                    provider_id: provider_id.to_string(),
                    previous: AuthState::Authenticating,
                    current: AuthState::Failed(e.to_string()),
                });

                Err(e)
            }
        }
    }

    /// Authenticate with the default provider.
    pub async fn authenticate_default(
        &self,
        credentials: AuthCredentials,
    ) -> NetworkResult<TokenPair> {
        let default_id = self
            .default_provider
            .read()
            .await
            .clone()
            .ok_or_else(|| NetworkError::ConfigError {
                message: "No default provider configured".to_string(),
            })?;

        self.authenticate(&default_id, credentials).await
    }

    /// Refresh token for a specific provider.
    pub async fn refresh(&self, provider_id: &str) -> NetworkResult<TokenPair> {
        let provider = self
            .get_provider(provider_id)
            .await
            .ok_or_else(|| NetworkError::ConfigError {
                message: format!("Provider '{}' not found", provider_id),
            })?;

        if !provider.can_refresh() {
            return Err(NetworkError::RefreshFailed {
                message: "Provider does not support refresh".to_string(),
            });
        }

        let previous_state = provider.state().await;

        let _ = self.state_tx.send(AuthStateChange {
            provider_id: provider_id.to_string(),
            previous: previous_state,
            current: AuthState::Refreshing,
        });

        match provider.refresh().await {
            Ok(token) => {
                debug!(provider_id = %provider_id, "Token refreshed");

                let _ = self.state_tx.send(AuthStateChange {
                    provider_id: provider_id.to_string(),
                    previous: AuthState::Refreshing,
                    current: AuthState::Authenticated,
                });

                Ok(token)
            }
            Err(e) => {
                warn!(provider_id = %provider_id, error = %e, "Token refresh failed");

                let _ = self.state_tx.send(AuthStateChange {
                    provider_id: provider_id.to_string(),
                    previous: AuthState::Refreshing,
                    current: AuthState::Expired,
                });

                Err(e)
            }
        }
    }

    /// Logout from a specific provider.
    pub async fn logout(&self, provider_id: &str) -> NetworkResult<()> {
        let provider = self
            .get_provider(provider_id)
            .await
            .ok_or_else(|| NetworkError::ConfigError {
                message: format!("Provider '{}' not found", provider_id),
            })?;

        let previous_state = provider.state().await;
        provider.logout().await?;

        let _ = self.state_tx.send(AuthStateChange {
            provider_id: provider_id.to_string(),
            previous: previous_state,
            current: AuthState::Unauthenticated,
        });

        info!(provider_id = %provider_id, "Logged out");
        Ok(())
    }

    /// Logout from all providers.
    pub async fn logout_all(&self) -> NetworkResult<()> {
        let providers = self.providers.read().await;

        for (id, provider) in providers.iter() {
            let previous_state = provider.state().await;
            if let Err(e) = provider.logout().await {
                warn!(provider_id = %id, error = %e, "Logout failed");
            }

            let _ = self.state_tx.send(AuthStateChange {
                provider_id: id.clone(),
                previous: previous_state,
                current: AuthState::Unauthenticated,
            });
        }

        info!("Logged out from all providers");
        Ok(())
    }

    /// Apply authentication to an HTTP request.
    ///
    /// Uses the provider specified in the request, or the default provider.
    pub async fn apply_auth(&self, request: HttpRequest) -> NetworkResult<HttpRequest> {
        let provider_id = request
            .auth_provider
            .clone()
            .or_else(|| self.default_provider.try_read().ok()?.clone());

        let provider_id = match provider_id {
            Some(id) => id,
            None => return Ok(request), // No auth to apply
        };

        let provider = self.get_provider(&provider_id).await.ok_or_else(|| {
            NetworkError::ConfigError {
                message: format!("Provider '{}' not found", provider_id),
            }
        })?;

        // Check if token needs refresh
        if self.auto_refresh && provider.can_refresh() {
            if let Some(token) = provider.current_token().await {
                if token.is_expiring_soon(self.refresh_threshold) {
                    debug!(
                        provider_id = %provider_id,
                        "Token expiring soon, attempting refresh"
                    );

                    if let Err(e) = self.refresh(&provider_id).await {
                        warn!(
                            provider_id = %provider_id,
                            error = %e,
                            "Auto-refresh failed, using existing token"
                        );
                    }
                }
            }
        }

        provider.apply_to_request(request).await
    }

    /// Get the current auth state for a provider.
    pub async fn state(&self, provider_id: &str) -> Option<AuthState> {
        let provider = self.get_provider(provider_id).await?;
        Some(provider.state().await)
    }

    /// Get the current auth state for the default provider.
    pub async fn default_state(&self) -> Option<AuthState> {
        let provider = self.get_default_provider().await?;
        Some(provider.state().await)
    }

    /// Check if any provider is authenticated.
    pub async fn is_authenticated(&self) -> bool {
        let providers = self.providers.read().await;
        for provider in providers.values() {
            if provider.state().await.is_authenticated() {
                return true;
            }
        }
        false
    }

    /// List all registered providers.
    pub async fn list_providers(&self) -> Vec<(String, AuthProviderType)> {
        let providers = self.providers.read().await;
        providers
            .iter()
            .map(|(id, p)| (id.clone(), p.provider_type()))
            .collect()
    }
}

impl Default for AuthManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for AuthManager.
#[derive(Debug)]
pub struct AuthManagerBuilder {
    refresh_threshold: Duration,
    auto_refresh: bool,
    providers: Vec<Box<dyn AuthProvider>>,
    default_provider: Option<String>,
}

impl Default for AuthManagerBuilder {
    fn default() -> Self {
        Self {
            refresh_threshold: Duration::minutes(5),
            auto_refresh: true,
            providers: Vec::new(),
            default_provider: None,
        }
    }
}

impl AuthManagerBuilder {
    /// Create a new builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the refresh threshold.
    pub fn refresh_threshold(mut self, threshold: Duration) -> Self {
        self.refresh_threshold = threshold;
        self
    }

    /// Enable or disable auto-refresh.
    pub fn auto_refresh(mut self, enabled: bool) -> Self {
        self.auto_refresh = enabled;
        self
    }

    /// Add a provider.
    pub fn provider(mut self, provider: impl AuthProvider + 'static) -> Self {
        if self.default_provider.is_none() {
            self.default_provider = Some(provider.id().to_string());
        }
        self.providers.push(Box::new(provider));
        self
    }

    /// Set the default provider.
    pub fn default_provider(mut self, id: impl Into<String>) -> Self {
        self.default_provider = Some(id.into());
        self
    }

    /// Build the auth manager.
    pub async fn build(self) -> AuthManager {
        let manager = AuthManager::with_config(self.refresh_threshold, self.auto_refresh);

        for provider in self.providers {
            // Register providers - we need to move ownership properly
            // This is a workaround since we can't use async in the iterator
            let id = provider.id().to_string();
            let provider = Arc::from(provider) as Arc<dyn AuthProvider>;
            manager.providers.write().await.insert(id, provider);
        }

        if let Some(default_id) = self.default_provider {
            *manager.default_provider.write().await = Some(default_id);
        }

        manager
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::{ApiKeyConfig, ApiKeyProvider};

    #[tokio::test]
    async fn test_auth_manager() {
        let manager = AuthManager::new();

        let provider = ApiKeyProvider::new("api-key", ApiKeyConfig::default());
        manager.register_provider(provider).await;

        // Should be unauthenticated initially
        assert_eq!(
            manager.state("api-key").await,
            Some(AuthState::Unauthenticated)
        );

        // Authenticate
        manager
            .authenticate("api-key", AuthCredentials::api_key("test-key"))
            .await
            .unwrap();

        // Should now be authenticated
        assert_eq!(
            manager.state("api-key").await,
            Some(AuthState::Authenticated)
        );

        // Logout
        manager.logout("api-key").await.unwrap();

        assert_eq!(
            manager.state("api-key").await,
            Some(AuthState::Unauthenticated)
        );
    }

    #[tokio::test]
    async fn test_state_change_subscription() {
        let manager = AuthManager::new();
        let mut rx = manager.subscribe_state_changes();

        let provider = ApiKeyProvider::new("test", ApiKeyConfig::default());
        manager.register_provider(provider).await;

        // Spawn task to receive state changes
        let handle = tokio::spawn(async move {
            let mut changes = Vec::new();
            while let Ok(change) = rx.try_recv() {
                changes.push(change);
            }
            changes
        });

        // Authenticate
        manager
            .authenticate("test", AuthCredentials::api_key("key"))
            .await
            .unwrap();

        // Give time for events to propagate
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;

        let changes = handle.await.unwrap();
        // Should have at least the authenticating -> authenticated transition
        assert!(!changes.is_empty());
    }
}
