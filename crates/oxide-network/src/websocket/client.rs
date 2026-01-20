//! WebSocket client implementation.

use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, RwLock, broadcast};
use tracing::debug;

use crate::allowlist::Allowlist;
use crate::error::{NetworkError, NetworkResult};

use super::{WsConnectionState, WsEvent, WsMessage};

/// Configuration for WebSocket client.
#[derive(Debug, Clone)]
pub struct WsClientConfig {
    /// URL to connect to.
    pub url: String,
    /// Whether to automatically reconnect on disconnect.
    pub auto_reconnect: bool,
    /// Maximum reconnection attempts (0 = infinite).
    pub max_reconnect_attempts: u32,
    /// Initial delay between reconnection attempts.
    pub reconnect_delay: Duration,
    /// Maximum delay between reconnection attempts.
    pub max_reconnect_delay: Duration,
    /// Backoff multiplier for reconnection delay.
    pub reconnect_backoff: f64,
    /// Heartbeat interval (ping frames).
    pub heartbeat_interval: Option<Duration>,
    /// Connection timeout.
    pub connect_timeout: Duration,
    /// Maximum message queue size during disconnection.
    pub max_queue_size: usize,
    /// Custom headers for the upgrade request.
    pub headers: Vec<(String, String)>,
    /// Subprotocols to request.
    pub subprotocols: Vec<String>,
}

impl Default for WsClientConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            auto_reconnect: true,
            max_reconnect_attempts: 10,
            reconnect_delay: Duration::from_secs(1),
            max_reconnect_delay: Duration::from_secs(30),
            reconnect_backoff: 2.0,
            heartbeat_interval: Some(Duration::from_secs(30)),
            connect_timeout: Duration::from_secs(10),
            max_queue_size: 100,
            headers: Vec::new(),
            subprotocols: Vec::new(),
        }
    }
}

impl WsClientConfig {
    /// Create a new config with the given URL.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            ..Default::default()
        }
    }

    /// Set auto-reconnect behavior.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.auto_reconnect = enabled;
        self
    }

    /// Set max reconnection attempts.
    pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
        self.max_reconnect_attempts = max;
        self
    }

    /// Set heartbeat interval.
    pub fn heartbeat_interval(mut self, interval: Option<Duration>) -> Self {
        self.heartbeat_interval = interval;
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Add a subprotocol.
    pub fn subprotocol(mut self, protocol: impl Into<String>) -> Self {
        self.subprotocols.push(protocol.into());
        self
    }
}

/// WebSocket client handle.
///
/// This is the main interface for WebSocket communication.
/// Internally manages connection lifecycle, reconnection, and message queuing.
#[derive(Clone)]
pub struct WsClient {
    config: WsClientConfig,
    state: Arc<RwLock<WsConnectionState>>,
    outgoing_tx: mpsc::Sender<WsMessage>,
    event_tx: broadcast::Sender<WsEvent>,
    message_queue: Arc<RwLock<VecDeque<WsMessage>>>,
    allowlist: Option<Arc<Allowlist>>,
    reconnect_count: Arc<RwLock<u32>>,
}

impl WsClient {
    /// Create a new WebSocket client (does not connect immediately).
    pub fn new(config: WsClientConfig) -> Self {
        let (outgoing_tx, _outgoing_rx) = mpsc::channel(100);
        let (event_tx, _) = broadcast::channel(100);

        Self {
            config,
            state: Arc::new(RwLock::new(WsConnectionState::Disconnected)),
            outgoing_tx,
            event_tx,
            message_queue: Arc::new(RwLock::new(VecDeque::new())),
            allowlist: None,
            reconnect_count: Arc::new(RwLock::new(0)),
        }
    }

    /// Create a client with an allowlist for URL validation.
    pub fn with_allowlist(mut self, allowlist: Arc<Allowlist>) -> Self {
        self.allowlist = Some(allowlist);
        self
    }

    /// Get the current connection state.
    pub async fn state(&self) -> WsConnectionState {
        *self.state.read().await
    }

    /// Check if currently connected.
    pub async fn is_connected(&self) -> bool {
        *self.state.read().await == WsConnectionState::Connected
    }

    /// Subscribe to WebSocket events.
    pub fn subscribe(&self) -> broadcast::Receiver<WsEvent> {
        self.event_tx.subscribe()
    }

    /// Send a message.
    ///
    /// If not connected and queue is not full, the message will be queued
    /// and sent when connection is established.
    pub async fn send(&self, message: WsMessage) -> NetworkResult<()> {
        let state = *self.state.read().await;

        match state {
            WsConnectionState::Connected => {
                self.outgoing_tx
                    .send(message)
                    .await
                    .map_err(|_| NetworkError::WebSocketError {
                        message: "Channel closed".to_string(),
                    })?;
            }
            WsConnectionState::Connecting | WsConnectionState::Reconnecting => {
                // Queue the message
                let mut queue = self.message_queue.write().await;
                if queue.len() >= self.config.max_queue_size {
                    return Err(NetworkError::WebSocketError {
                        message: "Message queue full".to_string(),
                    });
                }
                queue.push_back(message);
                debug!("Message queued, queue size: {}", queue.len());
            }
            _ => {
                return Err(NetworkError::WebSocketError {
                    message: format!("Cannot send in state: {}", state),
                });
            }
        }

        Ok(())
    }

    /// Send a text message.
    pub async fn send_text(&self, text: impl Into<String>) -> NetworkResult<()> {
        self.send(WsMessage::Text(text.into())).await
    }

    /// Send a JSON message.
    pub async fn send_json<T: serde::Serialize>(&self, data: &T) -> NetworkResult<()> {
        let message =
            WsMessage::json(data).map_err(|e| NetworkError::JsonError(e))?;
        self.send(message).await
    }

    /// Close the connection.
    pub async fn close(&self) -> NetworkResult<()> {
        self.send(WsMessage::Close {
            code: Some(1000),
            reason: Some("Normal closure".to_string()),
        })
        .await
    }

    /// Get the number of queued messages.
    pub async fn queue_size(&self) -> usize {
        self.message_queue.read().await.len()
    }

    /// Get the current reconnection attempt count.
    pub async fn reconnect_attempts(&self) -> u32 {
        *self.reconnect_count.read().await
    }

    /// Connect to the WebSocket server.
    ///
    /// This is a placeholder - actual connection logic requires the
    /// `websocket` feature and tokio-tungstenite.
    #[cfg(feature = "websocket")]
    pub async fn connect(&self) -> NetworkResult<()> {
        // Check allowlist
        if let Some(allowlist) = &self.allowlist {
            if !allowlist.is_allowed(&self.config.url) {
                return Err(NetworkError::BlockedByAllowlist {
                    url: self.config.url.clone(),
                });
            }
        }

        // Parse URL
        let url = Url::parse(&self.config.url)?;

        // Update state
        {
            let mut state = self.state.write().await;
            *state = WsConnectionState::Connecting;
        }

        // Emit connecting event
        let _ = self.event_tx.send(WsEvent::Reconnecting { attempt: 0 });

        // TODO: Implement actual WebSocket connection with tokio-tungstenite
        // This would spawn tasks for:
        // 1. Reading from the WebSocket
        // 2. Writing outgoing messages
        // 3. Heartbeat/ping-pong
        // 4. Reconnection handling

        info!(url = %self.config.url, "WebSocket connection initiated");

        Ok(())
    }

    /// Connect stub for when websocket feature is disabled.
    #[cfg(not(feature = "websocket"))]
    pub async fn connect(&self) -> NetworkResult<()> {
        Err(NetworkError::ConfigError {
            message: "WebSocket feature not enabled".to_string(),
        })
    }

    /// Calculate delay for reconnection attempt.
    fn reconnect_delay_for_attempt(&self, attempt: u32) -> Duration {
        let delay_ms = self.config.reconnect_delay.as_millis() as f64
            * self.config.reconnect_backoff.powi(attempt as i32);
        let delay = Duration::from_millis(delay_ms as u64);
        std::cmp::min(delay, self.config.max_reconnect_delay)
    }

    /// Drain the message queue (internal use after reconnection).
    async fn drain_queue(&self) -> Vec<WsMessage> {
        let mut queue = self.message_queue.write().await;
        queue.drain(..).collect()
    }
}

impl std::fmt::Debug for WsClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WsClient")
            .field("url", &self.config.url)
            .field("auto_reconnect", &self.config.auto_reconnect)
            .finish()
    }
}

/// Builder for WebSocket client.
#[derive(Debug, Default)]
pub struct WsClientBuilder {
    config: WsClientConfig,
    allowlist: Option<Arc<Allowlist>>,
}

impl WsClientBuilder {
    /// Create a new builder.
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            config: WsClientConfig::new(url),
            allowlist: None,
        }
    }

    /// Set auto-reconnect.
    pub fn auto_reconnect(mut self, enabled: bool) -> Self {
        self.config.auto_reconnect = enabled;
        self
    }

    /// Set max reconnection attempts.
    pub fn max_reconnect_attempts(mut self, max: u32) -> Self {
        self.config.max_reconnect_attempts = max;
        self
    }

    /// Set heartbeat interval.
    pub fn heartbeat_interval(mut self, interval: Duration) -> Self {
        self.config.heartbeat_interval = Some(interval);
        self
    }

    /// Disable heartbeat.
    pub fn no_heartbeat(mut self) -> Self {
        self.config.heartbeat_interval = None;
        self
    }

    /// Set connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Add a custom header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.headers.push((name.into(), value.into()));
        self
    }

    /// Add a subprotocol.
    pub fn subprotocol(mut self, protocol: impl Into<String>) -> Self {
        self.config.subprotocols.push(protocol.into());
        self
    }

    /// Set the URL allowlist.
    pub fn allowlist(mut self, allowlist: Arc<Allowlist>) -> Self {
        self.allowlist = Some(allowlist);
        self
    }

    /// Build the WebSocket client.
    pub fn build(self) -> WsClient {
        let mut client = WsClient::new(self.config);
        if let Some(allowlist) = self.allowlist {
            client = client.with_allowlist(allowlist);
        }
        client
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = WsClientBuilder::new("wss://example.com/ws")
            .auto_reconnect(true)
            .heartbeat_interval(Duration::from_secs(30))
            .build();

        assert_eq!(client.state().await, WsConnectionState::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[tokio::test]
    async fn test_message_queuing() {
        let client = WsClientBuilder::new("wss://example.com/ws")
            .auto_reconnect(true)
            .build();

        // Set state to connecting so messages get queued
        {
            let mut state = client.state.write().await;
            *state = WsConnectionState::Connecting;
        }

        // Send should queue the message
        client.send_text("test").await.unwrap();
        assert_eq!(client.queue_size().await, 1);
    }
}
