//! WebSocket dev server for hot reload communication
//!
//! Provides a WebSocket server that broadcasts hot reload events to connected clients.

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio_tungstenite::{accept_async, tungstenite::Message};
use thiserror::Error;

use super::events::{HotReloadEvent, CompileErrorInfo};
use super::{DEFAULT_WS_PORT, PROTOCOL_VERSION};

/// Errors that can occur with the dev server
#[derive(Debug, Error)]
pub enum ServerError {
    #[error("Failed to bind to address {addr}: {source}")]
    BindError {
        addr: String,
        #[source]
        source: std::io::Error,
    },

    #[error("WebSocket error: {0}")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),

    #[error("Server already running")]
    AlreadyRunning,

    #[error("Server not running")]
    NotRunning,

    #[error("Failed to serialize message: {0}")]
    SerializeError(#[from] serde_json::Error),
}

/// Configuration for the dev server
#[derive(Debug, Clone)]
pub struct DevServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Host address
    pub host: String,
    /// Maximum number of clients
    pub max_clients: usize,
    /// Ping interval in seconds
    pub ping_interval_secs: u64,
    /// Whether to enable verbose logging
    pub verbose: bool,
}

impl Default for DevServerConfig {
    fn default() -> Self {
        Self {
            port: DEFAULT_WS_PORT,
            host: "127.0.0.1".to_string(),
            max_clients: 10,
            ping_interval_secs: 30,
            verbose: false,
        }
    }
}

/// Messages that can be sent from the server to clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    /// Welcome message sent on connection
    Welcome {
        protocol_version: u32,
        server_id: String,
    },

    /// Hot reload triggered
    HotReload {
        /// Files that were recompiled
        changed_files: Vec<String>,
        /// Components that were updated
        changed_components: Vec<String>,
        /// Time taken to compile (ms)
        compile_time_ms: u64,
    },

    /// Full reload required
    FullReload {
        reason: String,
    },

    /// Compilation error
    CompileError {
        file: String,
        errors: Vec<CompileErrorInfo>,
    },

    /// Compilation success (errors cleared)
    CompileSuccess,

    /// State snapshot for restoration
    StateSnapshot {
        snapshot_id: String,
        data: String, // JSON-encoded state
    },

    /// Ping message for keepalive
    Ping {
        timestamp: u64,
    },

    /// Server shutting down
    Goodbye {
        reason: String,
    },
}

/// Messages that can be received from clients
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    /// Client ready to receive updates
    Ready {
        client_id: String,
        capabilities: Vec<String>,
    },

    /// Request current state
    RequestState,

    /// Restore state from snapshot
    RestoreState {
        snapshot_id: String,
    },

    /// Acknowledge hot reload
    HotReloadAck {
        success: bool,
        error: Option<String>,
    },

    /// Pong response
    Pong {
        timestamp: u64,
    },

    /// Client-side error report
    ClientError {
        message: String,
        stack: Option<String>,
    },
}

/// Information about a connected client
#[derive(Debug, Clone)]
pub struct ClientInfo {
    pub id: String,
    pub addr: SocketAddr,
    pub connected_at: std::time::Instant,
    pub capabilities: Vec<String>,
    pub last_ping: Option<std::time::Instant>,
}

/// WebSocket dev server for hot reload
pub struct DevServer {
    config: DevServerConfig,
    /// Server ID for identification
    server_id: String,
    /// Connected clients
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    /// Broadcast channel for sending messages to all clients
    broadcast_tx: broadcast::Sender<ServerMessage>,
    /// Channel for receiving shutdown signal
    shutdown_tx: Option<mpsc::Sender<()>>,
    /// Whether the server is running
    running: Arc<RwLock<bool>>,
}

impl DevServer {
    /// Create a new dev server with the given configuration
    pub fn new(config: DevServerConfig) -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);

        Self {
            config,
            server_id: uuid::Uuid::new_v4().to_string(),
            clients: Arc::new(RwLock::new(HashMap::new())),
            broadcast_tx,
            shutdown_tx: None,
            running: Arc::new(RwLock::new(false)),
        }
    }

    /// Create a new dev server with default configuration
    pub fn with_defaults() -> Self {
        Self::new(DevServerConfig::default())
    }

    /// Start the server
    pub async fn start(&mut self) -> Result<(), ServerError> {
        if *self.running.read().await {
            return Err(ServerError::AlreadyRunning);
        }

        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr)
            .await
            .map_err(|e| ServerError::BindError {
                addr: addr.clone(),
                source: e,
            })?;

        tracing::info!("Hot reload server listening on ws://{}", addr);

        *self.running.write().await = true;

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
        self.shutdown_tx = Some(shutdown_tx);

        let clients = Arc::clone(&self.clients);
        let broadcast_tx = self.broadcast_tx.clone();
        let server_id = self.server_id.clone();
        let max_clients = self.config.max_clients;
        let running = Arc::clone(&self.running);

        // Spawn the server task
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((stream, addr)) => {
                                // Check client limit
                                if clients.read().await.len() >= max_clients {
                                    tracing::warn!("Max clients reached, rejecting connection from {}", addr);
                                    continue;
                                }

                                let clients = Arc::clone(&clients);
                                let broadcast_tx = broadcast_tx.clone();
                                let server_id = server_id.clone();

                                tokio::spawn(async move {
                                    if let Err(e) = handle_connection(
                                        stream,
                                        addr,
                                        clients,
                                        broadcast_tx,
                                        server_id,
                                    ).await {
                                        tracing::error!("Connection error: {}", e);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to accept connection: {}", e);
                            }
                        }
                    }
                    _ = shutdown_rx.recv() => {
                        tracing::info!("Hot reload server shutting down");
                        *running.write().await = false;
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Stop the server
    pub async fn stop(&mut self) -> Result<(), ServerError> {
        if !*self.running.read().await {
            return Err(ServerError::NotRunning);
        }

        // Broadcast goodbye message
        let _ = self.broadcast(ServerMessage::Goodbye {
            reason: "Server shutting down".to_string(),
        });

        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        *self.running.write().await = false;
        Ok(())
    }

    /// Broadcast a message to all connected clients
    pub fn broadcast(&self, message: ServerMessage) -> Result<usize, ServerError> {
        let receivers = self.broadcast_tx.send(message).unwrap_or(0);
        Ok(receivers)
    }

    /// Notify clients of a hot reload
    pub fn notify_hot_reload(
        &self,
        changed_files: Vec<String>,
        changed_components: Vec<String>,
        compile_time_ms: u64,
    ) -> Result<usize, ServerError> {
        self.broadcast(ServerMessage::HotReload {
            changed_files,
            changed_components,
            compile_time_ms,
        })
    }

    /// Notify clients of a compilation error
    pub fn notify_compile_error(
        &self,
        file: String,
        errors: Vec<CompileErrorInfo>,
    ) -> Result<usize, ServerError> {
        self.broadcast(ServerMessage::CompileError { file, errors })
    }

    /// Notify clients that compilation succeeded
    pub fn notify_compile_success(&self) -> Result<usize, ServerError> {
        self.broadcast(ServerMessage::CompileSuccess)
    }

    /// Notify clients that a full reload is required
    pub fn notify_full_reload(&self, reason: String) -> Result<usize, ServerError> {
        self.broadcast(ServerMessage::FullReload { reason })
    }

    /// Get the number of connected clients
    pub async fn client_count(&self) -> usize {
        self.clients.read().await.len()
    }

    /// Get information about connected clients
    pub async fn get_clients(&self) -> Vec<ClientInfo> {
        self.clients.read().await.values().cloned().collect()
    }

    /// Check if the server is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }

    /// Get the server address
    pub fn address(&self) -> String {
        format!("ws://{}:{}", self.config.host, self.config.port)
    }

    /// Subscribe to broadcast messages
    pub fn subscribe(&self) -> broadcast::Receiver<ServerMessage> {
        self.broadcast_tx.subscribe()
    }
}

impl Default for DevServer {
    fn default() -> Self {
        Self::with_defaults()
    }
}

/// Handle an individual WebSocket connection
async fn handle_connection(
    stream: TcpStream,
    addr: SocketAddr,
    clients: Arc<RwLock<HashMap<String, ClientInfo>>>,
    broadcast_tx: broadcast::Sender<ServerMessage>,
    server_id: String,
) -> Result<(), ServerError> {
    let ws_stream = accept_async(stream).await?;
    let (mut write, mut read) = ws_stream.split();

    // Generate client ID
    let client_id = uuid::Uuid::new_v4().to_string();

    tracing::info!("Client {} connected from {}", client_id, addr);

    // Send welcome message
    let welcome = ServerMessage::Welcome {
        protocol_version: PROTOCOL_VERSION,
        server_id: server_id.clone(),
    };
    let welcome_json = serde_json::to_string(&welcome)?;
    write.send(Message::Text(welcome_json.into())).await?;

    // Register client
    clients.write().await.insert(
        client_id.clone(),
        ClientInfo {
            id: client_id.clone(),
            addr,
            connected_at: std::time::Instant::now(),
            capabilities: Vec::new(),
            last_ping: None,
        },
    );

    // Subscribe to broadcast messages
    let mut broadcast_rx = broadcast_tx.subscribe();

    // Handle messages
    loop {
        tokio::select! {
            // Handle incoming messages from client
            msg_result = read.next() => {
                match msg_result {
                    Some(Ok(Message::Text(text))) => {
                        if let Ok(client_msg) = serde_json::from_str::<ClientMessage>(&text) {
                            handle_client_message(&client_msg, &client_id, &clients).await;
                        }
                    }
                    Some(Ok(Message::Close(_))) | None => {
                        break;
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error for client {}: {}", client_id, e);
                        break;
                    }
                    _ => {}
                }
            }
            // Handle broadcast messages
            broadcast_result = broadcast_rx.recv() => {
                match broadcast_result {
                    Ok(server_msg) => {
                        if let Ok(json) = serde_json::to_string(&server_msg) {
                            if write.send(Message::Text(json.into())).await.is_err() {
                                break;
                            }
                        }
                    }
                    Err(_) => {
                        // Broadcast channel closed
                        break;
                    }
                }
            }
        }
    }

    // Remove client on disconnect
    clients.write().await.remove(&client_id);
    tracing::info!("Client {} disconnected", client_id);

    Ok(())
}

/// Handle a message from a client
async fn handle_client_message(
    msg: &ClientMessage,
    client_id: &str,
    clients: &Arc<RwLock<HashMap<String, ClientInfo>>>,
) {
    match msg {
        ClientMessage::Ready { capabilities, .. } => {
            // Update client capabilities
            if let Some(client) = clients.write().await.get_mut(client_id) {
                client.capabilities = capabilities.clone();
            }
            tracing::debug!("Client {} ready with capabilities: {:?}", client_id, capabilities);
        }
        ClientMessage::Pong { timestamp } => {
            if let Some(client) = clients.write().await.get_mut(client_id) {
                client.last_ping = Some(std::time::Instant::now());
            }
            tracing::trace!("Received pong from {} with timestamp {}", client_id, timestamp);
        }
        ClientMessage::HotReloadAck { success, error } => {
            if *success {
                tracing::debug!("Client {} acknowledged hot reload", client_id);
            } else {
                tracing::warn!(
                    "Client {} failed hot reload: {}",
                    client_id,
                    error.as_deref().unwrap_or("unknown error")
                );
            }
        }
        ClientMessage::ClientError { message, stack } => {
            tracing::error!(
                "Client {} reported error: {}\n{}",
                client_id,
                message,
                stack.as_deref().unwrap_or("")
            );
        }
        _ => {}
    }
}

/// Event handler that bridges hot reload events to the dev server
pub struct DevServerEventHandler {
    server: Arc<DevServer>,
}

impl DevServerEventHandler {
    pub fn new(server: Arc<DevServer>) -> Self {
        Self { server }
    }

    /// Handle a hot reload event
    pub fn handle(&self, event: &HotReloadEvent) {
        match event {
            HotReloadEvent::CompileSuccess {
                duration_ms,
                changed_components,
            } => {
                let _ = self.server.notify_hot_reload(
                    Vec::new(),
                    changed_components.clone(),
                    *duration_ms,
                );
                let _ = self.server.notify_compile_success();
            }
            HotReloadEvent::CompileError { file, errors } => {
                let _ = self.server.notify_compile_error(
                    file.to_string_lossy().to_string(),
                    errors.clone(),
                );
            }
            HotReloadEvent::FullReloadRequired { reason } => {
                let _ = self.server.notify_full_reload(reason.clone());
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_message_serialize() {
        let msg = ServerMessage::HotReload {
            changed_files: vec!["app.oui".to_string()],
            changed_components: vec!["App".to_string()],
            compile_time_ms: 50,
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("hot_reload"));
        assert!(json.contains("app.oui"));
    }

    #[test]
    fn test_client_message_deserialize() {
        let json = r#"{"type": "ready", "client_id": "test", "capabilities": ["hot_reload"]}"#;
        let msg: ClientMessage = serde_json::from_str(json).unwrap();

        match msg {
            ClientMessage::Ready { client_id, capabilities } => {
                assert_eq!(client_id, "test");
                assert_eq!(capabilities, vec!["hot_reload"]);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
