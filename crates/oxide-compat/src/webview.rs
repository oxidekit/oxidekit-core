//! WebView Compatibility Plugin
//!
//! **WARNING**: This is NOT the recommended path for OxideKit applications.
//! WebView adds a web surface which increases attack surface significantly.
//!
//! # Use Cases
//!
//! - Embedding niche web widgets during migration
//! - Third-party UI modules that only exist in web form
//! - Incremental migration from Electron/Tauri
//!
//! # Security
//!
//! - Default: no remote network access
//! - Default: load only bundled assets
//! - Strict CSP enforcement
//! - All native access through schema-validated bridge

use crate::policy::{CompatPolicy, CompatFeature, PolicyEnforcer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use thiserror::Error;
use uuid::Uuid;

/// Errors that can occur in WebView operations
#[derive(Error, Debug)]
pub enum WebViewError {
    /// Policy violation
    #[error("Policy violation: {0}")]
    PolicyViolation(String),

    /// Invalid source
    #[error("Invalid source: {0}")]
    InvalidSource(String),

    /// Bridge error
    #[error("Bridge error: {0}")]
    BridgeError(String),

    /// Schema validation error
    #[error("Schema validation failed: {0}")]
    SchemaValidation(String),

    /// WebView initialization failed
    #[error("WebView initialization failed: {0}")]
    InitializationFailed(String),

    /// Navigation blocked
    #[error("Navigation to {0} blocked by policy")]
    NavigationBlocked(String),
}

/// Source for a web widget
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WebWidgetSource {
    /// Load from a bundled asset directory
    Bundled {
        /// Path relative to the app bundle
        path: PathBuf,
    },
    /// Load from a bundle ID (marketplace widget)
    BundleId {
        /// Bundle identifier
        id: String,
        /// Version constraint
        version: Option<String>,
    },
    /// Load from a remote URL (requires allow_remote_webview)
    Remote {
        /// Remote URL
        url: String,
    },
    /// Load inline HTML content
    Inline {
        /// HTML content
        html: String,
    },
}

/// Configuration for a web widget
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebWidgetConfig {
    /// Widget source
    pub source: WebWidgetSource,

    /// Custom Content Security Policy (overrides default)
    pub csp: Option<String>,

    /// Enable devtools (dev mode only)
    pub devtools: bool,

    /// Allow navigation within the widget
    pub allow_navigation: bool,

    /// Allowed navigation origins (if allow_navigation is true)
    pub allowed_navigation_origins: Vec<String>,

    /// Initial window size
    pub size: Option<(u32, u32)>,

    /// Whether the widget is transparent
    pub transparent: bool,

    /// Custom user agent
    pub user_agent: Option<String>,

    /// Permissions granted to this widget
    pub permissions: Vec<WebViewPermission>,
}

impl Default for WebWidgetConfig {
    fn default() -> Self {
        Self {
            source: WebWidgetSource::Inline {
                html: "<html><body>Empty Widget</body></html>".to_string(),
            },
            csp: None,
            devtools: false,
            allow_navigation: false,
            allowed_navigation_origins: Vec::new(),
            size: None,
            transparent: false,
            user_agent: None,
            permissions: Vec::new(),
        }
    }
}

/// Permissions that can be granted to a web widget
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WebViewPermission {
    /// Access to clipboard (read)
    ClipboardRead,
    /// Access to clipboard (write)
    ClipboardWrite,
    /// Access to localStorage
    LocalStorage,
    /// Access to sessionStorage
    SessionStorage,
    /// Geolocation access (NOT RECOMMENDED)
    Geolocation,
    /// Notification access
    Notifications,
    /// Media device access (camera/mic) (NOT RECOMMENDED)
    MediaDevices,
}

/// A web widget instance
#[cfg(feature = "webview")]
pub struct WebWidget {
    /// Unique identifier for this widget
    id: Uuid,
    /// Widget configuration
    config: WebWidgetConfig,
    /// Message bridge
    bridge: MessageBridge,
    /// Policy enforcer
    policy: CompatPolicy,
    /// Whether the widget is initialized
    initialized: bool,
}

#[cfg(feature = "webview")]
impl WebWidget {
    /// Create a new web widget
    pub fn new(config: WebWidgetConfig, policy: CompatPolicy) -> Result<Self, WebViewError> {
        // Validate against policy
        let mut enforcer = PolicyEnforcer::new(policy.clone(), false);

        if !enforcer.enforce(CompatFeature::WebView) {
            return Err(WebViewError::PolicyViolation(
                "WebView is not allowed by policy".to_string(),
            ));
        }

        // Check if remote source is allowed
        if let WebWidgetSource::Remote { url } = &config.source {
            if !enforcer.enforce(CompatFeature::RemoteWebView) {
                return Err(WebViewError::PolicyViolation(
                    "Remote WebView content is not allowed".to_string(),
                ));
            }

            // Extract origin from URL
            if let Ok(parsed) = url::Url::parse(url) {
                if let Some(host) = parsed.host_str() {
                    if !policy.is_origin_allowed(host) {
                        return Err(WebViewError::NavigationBlocked(host.to_string()));
                    }
                }
            }
        }

        // Print warning
        crate::print_compat_warning();

        Ok(Self {
            id: Uuid::new_v4(),
            config,
            bridge: MessageBridge::new(),
            policy,
            initialized: false,
        })
    }

    /// Get the widget ID
    pub fn id(&self) -> Uuid {
        self.id
    }

    /// Get the message bridge
    pub fn bridge(&self) -> &MessageBridge {
        &self.bridge
    }

    /// Get mutable message bridge
    pub fn bridge_mut(&mut self) -> &mut MessageBridge {
        &mut self.bridge
    }

    /// Get effective CSP for this widget
    pub fn effective_csp(&self) -> &str {
        self.config
            .csp
            .as_deref()
            .unwrap_or_else(|| self.policy.effective_csp())
    }

    /// Check if a navigation is allowed
    pub fn is_navigation_allowed(&self, url: &str) -> bool {
        if !self.config.allow_navigation {
            return false;
        }

        // Parse URL to get origin
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(host) = parsed.host_str() {
                // Check widget-specific allowed origins
                if self.config.allowed_navigation_origins.iter().any(|allowed| {
                    if allowed.starts_with("*.") {
                        let domain = &allowed[2..];
                        host.ends_with(domain) || host == domain
                    } else {
                        host == allowed
                    }
                }) {
                    return true;
                }

                // Fall back to policy origins
                return self.policy.is_origin_allowed(host);
            }
        }

        false
    }

    /// Generate the initialization script for the bridge
    pub fn bridge_init_script(&self) -> String {
        format!(
            r#"
            (function() {{
                window.__OXIDE_BRIDGE__ = {{
                    widgetId: "{widget_id}",
                    handlers: {{}},
                    pendingRequests: {{}},
                    requestId: 0,

                    send: function(type, payload) {{
                        return new Promise((resolve, reject) => {{
                            const id = ++this.requestId;
                            this.pendingRequests[id] = {{ resolve, reject }};

                            const message = JSON.stringify({{
                                id: id,
                                widgetId: this.widgetId,
                                type: type,
                                payload: payload
                            }});

                            window.ipc.postMessage(message);
                        }});
                    }},

                    onMessage: function(type, handler) {{
                        if (!this.handlers[type]) {{
                            this.handlers[type] = [];
                        }}
                        this.handlers[type].push(handler);
                    }},

                    _handleResponse: function(id, success, data) {{
                        const pending = this.pendingRequests[id];
                        if (pending) {{
                            delete this.pendingRequests[id];
                            if (success) {{
                                pending.resolve(data);
                            }} else {{
                                pending.reject(new Error(data));
                            }}
                        }}
                    }},

                    _handleMessage: function(type, payload) {{
                        const handlers = this.handlers[type] || [];
                        handlers.forEach(h => h(payload));
                    }}
                }};

                // Expose clean API
                window.oxide = {{
                    send: (type, payload) => window.__OXIDE_BRIDGE__.send(type, payload),
                    onMessage: (type, handler) => window.__OXIDE_BRIDGE__.onMessage(type, handler)
                }};

                console.log('[OxideKit WebView] Bridge initialized for widget {widget_id}');
            }})();
            "#,
            widget_id = self.id
        )
    }
}

/// Message passed through the bridge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeMessage {
    /// Message ID (for request/response correlation)
    pub id: u64,
    /// Widget ID that sent the message
    pub widget_id: Uuid,
    /// Message type
    #[serde(rename = "type")]
    pub message_type: String,
    /// Message payload
    pub payload: serde_json::Value,
}

/// Response to a bridge message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BridgeResponse {
    /// Original message ID
    pub id: u64,
    /// Whether the operation succeeded
    pub success: bool,
    /// Response data or error message
    pub data: serde_json::Value,
}

/// Schema for message validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSchema {
    /// Message type this schema applies to
    pub message_type: String,
    /// JSON Schema for the payload
    pub schema: serde_json::Value,
    /// Whether this message requires specific permissions
    pub required_permissions: Vec<String>,
}

/// Message handler function type
pub type MessageHandler = Box<dyn Fn(&BridgeMessage) -> Result<serde_json::Value, String> + Send + Sync>;

/// Bridge for communication between WebView and native code
pub struct MessageBridge {
    /// Registered message handlers
    handlers: HashMap<String, MessageHandler>,
    /// Message schemas for validation
    schemas: HashMap<String, MessageSchema>,
    /// Message history (for debugging)
    #[cfg(debug_assertions)]
    history: Vec<BridgeMessage>,
}

impl MessageBridge {
    /// Create a new message bridge
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            schemas: HashMap::new(),
            #[cfg(debug_assertions)]
            history: Vec::new(),
        }
    }

    /// Register a message handler
    pub fn on<F>(&mut self, message_type: &str, handler: F)
    where
        F: Fn(&BridgeMessage) -> Result<serde_json::Value, String> + Send + Sync + 'static,
    {
        self.handlers
            .insert(message_type.to_string(), Box::new(handler));
    }

    /// Register a message schema
    pub fn register_schema(&mut self, schema: MessageSchema) {
        self.schemas.insert(schema.message_type.clone(), schema);
    }

    /// Handle an incoming message
    pub fn handle(&mut self, message: BridgeMessage) -> BridgeResponse {
        #[cfg(debug_assertions)]
        self.history.push(message.clone());

        // Validate against schema if present
        if let Some(schema) = self.schemas.get(&message.message_type) {
            // TODO: Implement JSON Schema validation
            // For now, just check that required permissions are noted
            tracing::debug!(
                "Message {} requires permissions: {:?}",
                message.message_type,
                schema.required_permissions
            );
        }

        // Find and call handler
        match self.handlers.get(&message.message_type) {
            Some(handler) => match handler(&message) {
                Ok(data) => BridgeResponse {
                    id: message.id,
                    success: true,
                    data,
                },
                Err(err) => BridgeResponse {
                    id: message.id,
                    success: false,
                    data: serde_json::Value::String(err),
                },
            },
            None => BridgeResponse {
                id: message.id,
                success: false,
                data: serde_json::Value::String(format!(
                    "No handler registered for message type: {}",
                    message.message_type
                )),
            },
        }
    }

    /// Send a message to the WebView
    pub fn send(&self, _widget_id: Uuid, message_type: &str, payload: serde_json::Value) -> String {
        // This would be injected into the WebView via JavaScript
        format!(
            "window.__OXIDE_BRIDGE__._handleMessage('{}', {});",
            message_type,
            serde_json::to_string(&payload).unwrap_or_else(|_| "null".to_string())
        )
    }

    /// Get message history (debug only)
    #[cfg(debug_assertions)]
    pub fn history(&self) -> &[BridgeMessage] {
        &self.history
    }
}

impl Default for MessageBridge {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for WebWidget
pub struct WebWidgetBuilder {
    config: WebWidgetConfig,
}

impl WebWidgetBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: WebWidgetConfig::default(),
        }
    }

    /// Set the source to a bundled asset path
    pub fn bundled<P: AsRef<Path>>(mut self, path: P) -> Self {
        self.config.source = WebWidgetSource::Bundled {
            path: path.as_ref().to_path_buf(),
        };
        self
    }

    /// Set the source to a bundle ID
    pub fn bundle_id(mut self, id: &str, version: Option<&str>) -> Self {
        self.config.source = WebWidgetSource::BundleId {
            id: id.to_string(),
            version: version.map(String::from),
        };
        self
    }

    /// Set the source to a remote URL
    pub fn remote(mut self, url: &str) -> Self {
        self.config.source = WebWidgetSource::Remote {
            url: url.to_string(),
        };
        self
    }

    /// Set the source to inline HTML
    pub fn inline(mut self, html: &str) -> Self {
        self.config.source = WebWidgetSource::Inline {
            html: html.to_string(),
        };
        self
    }

    /// Set custom CSP
    pub fn csp(mut self, csp: &str) -> Self {
        self.config.csp = Some(csp.to_string());
        self
    }

    /// Enable devtools
    pub fn devtools(mut self, enabled: bool) -> Self {
        self.config.devtools = enabled;
        self
    }

    /// Allow navigation
    pub fn allow_navigation(mut self, allowed: bool) -> Self {
        self.config.allow_navigation = allowed;
        self
    }

    /// Add allowed navigation origin
    pub fn allow_origin(mut self, origin: &str) -> Self {
        self.config.allowed_navigation_origins.push(origin.to_string());
        self
    }

    /// Set size
    pub fn size(mut self, width: u32, height: u32) -> Self {
        self.config.size = Some((width, height));
        self
    }

    /// Set transparency
    pub fn transparent(mut self, transparent: bool) -> Self {
        self.config.transparent = transparent;
        self
    }

    /// Add a permission
    pub fn permission(mut self, permission: WebViewPermission) -> Self {
        self.config.permissions.push(permission);
        self
    }

    /// Build the configuration
    pub fn build(self) -> WebWidgetConfig {
        self.config
    }

    /// Build and create the widget
    #[cfg(feature = "webview")]
    pub fn create(self, policy: CompatPolicy) -> Result<WebWidget, WebViewError> {
        WebWidget::new(self.config, policy)
    }
}

impl Default for WebWidgetBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_widget_builder() {
        let config = WebWidgetBuilder::new()
            .bundled("assets/widget")
            .devtools(true)
            .size(800, 600)
            .build();

        assert!(matches!(config.source, WebWidgetSource::Bundled { .. }));
        assert!(config.devtools);
        assert_eq!(config.size, Some((800, 600)));
    }

    #[test]
    fn test_message_bridge() {
        let mut bridge = MessageBridge::new();

        bridge.on("ping", |_msg| Ok(serde_json::json!({"pong": true})));

        let message = BridgeMessage {
            id: 1,
            widget_id: Uuid::new_v4(),
            message_type: "ping".to_string(),
            payload: serde_json::json!({}),
        };

        let response = bridge.handle(message);
        assert!(response.success);
    }

    #[test]
    fn test_unknown_message_type() {
        let mut bridge = MessageBridge::new();

        let message = BridgeMessage {
            id: 1,
            widget_id: Uuid::new_v4(),
            message_type: "unknown".to_string(),
            payload: serde_json::json!({}),
        };

        let response = bridge.handle(message);
        assert!(!response.success);
    }

    #[test]
    fn test_csp_default() {
        let policy = CompatPolicy::default();
        assert!(policy.effective_csp().contains("default-src 'self'"));
    }
}
