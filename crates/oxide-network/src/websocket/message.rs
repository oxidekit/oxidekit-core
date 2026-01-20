//! WebSocket message types.

use serde::{Deserialize, Serialize};

/// WebSocket message types.
#[derive(Debug, Clone)]
pub enum WsMessage {
    /// Text message.
    Text(String),
    /// Binary message.
    Binary(Vec<u8>),
    /// JSON message (convenience wrapper).
    Json(serde_json::Value),
    /// Ping frame.
    Ping(Vec<u8>),
    /// Pong frame.
    Pong(Vec<u8>),
    /// Close frame.
    Close {
        /// Close code.
        code: Option<u16>,
        /// Close reason.
        reason: Option<String>,
    },
}

impl WsMessage {
    /// Create a text message.
    pub fn text(content: impl Into<String>) -> Self {
        WsMessage::Text(content.into())
    }

    /// Create a binary message.
    pub fn binary(data: impl Into<Vec<u8>>) -> Self {
        WsMessage::Binary(data.into())
    }

    /// Create a JSON message.
    pub fn json<T: Serialize>(data: &T) -> Result<Self, serde_json::Error> {
        let value = serde_json::to_value(data)?;
        Ok(WsMessage::Json(value))
    }

    /// Create a close message.
    pub fn close(code: Option<u16>, reason: Option<String>) -> Self {
        WsMessage::Close { code, reason }
    }

    /// Check if this is a text message.
    pub fn is_text(&self) -> bool {
        matches!(self, WsMessage::Text(_))
    }

    /// Check if this is a binary message.
    pub fn is_binary(&self) -> bool {
        matches!(self, WsMessage::Binary(_))
    }

    /// Check if this is a JSON message.
    pub fn is_json(&self) -> bool {
        matches!(self, WsMessage::Json(_))
    }

    /// Check if this is a close message.
    pub fn is_close(&self) -> bool {
        matches!(self, WsMessage::Close { .. })
    }

    /// Try to get the text content.
    pub fn as_text(&self) -> Option<&str> {
        match self {
            WsMessage::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Try to get the binary content.
    pub fn as_binary(&self) -> Option<&[u8]> {
        match self {
            WsMessage::Binary(b) => Some(b),
            _ => None,
        }
    }

    /// Try to deserialize as JSON.
    pub fn as_json<T: for<'de> Deserialize<'de>>(&self) -> Option<Result<T, serde_json::Error>> {
        match self {
            WsMessage::Json(v) => Some(serde_json::from_value(v.clone())),
            WsMessage::Text(s) => Some(serde_json::from_str(s)),
            _ => None,
        }
    }

    /// Convert to text if possible.
    pub fn into_text(self) -> Option<String> {
        match self {
            WsMessage::Text(s) => Some(s),
            WsMessage::Json(v) => serde_json::to_string(&v).ok(),
            _ => None,
        }
    }

    /// Convert to binary.
    pub fn into_binary(self) -> Vec<u8> {
        match self {
            WsMessage::Text(s) => s.into_bytes(),
            WsMessage::Binary(b) => b,
            WsMessage::Json(v) => serde_json::to_vec(&v).unwrap_or_default(),
            WsMessage::Ping(b) | WsMessage::Pong(b) => b,
            WsMessage::Close { .. } => Vec::new(),
        }
    }
}

/// WebSocket connection state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WsConnectionState {
    /// Not connected.
    Disconnected,
    /// Connection in progress.
    Connecting,
    /// Connected and ready.
    Connected,
    /// Reconnecting after disconnection.
    Reconnecting,
    /// Connection closed normally.
    Closed,
    /// Connection failed with error.
    Failed,
}

impl std::fmt::Display for WsConnectionState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WsConnectionState::Disconnected => write!(f, "Disconnected"),
            WsConnectionState::Connecting => write!(f, "Connecting"),
            WsConnectionState::Connected => write!(f, "Connected"),
            WsConnectionState::Reconnecting => write!(f, "Reconnecting"),
            WsConnectionState::Closed => write!(f, "Closed"),
            WsConnectionState::Failed => write!(f, "Failed"),
        }
    }
}

/// Event emitted by WebSocket client.
#[derive(Debug, Clone)]
pub enum WsEvent {
    /// Connection established.
    Connected,
    /// Connection closed.
    Disconnected {
        /// Close code.
        code: Option<u16>,
        /// Close reason.
        reason: Option<String>,
    },
    /// Message received.
    Message(WsMessage),
    /// Error occurred.
    Error(String),
    /// Reconnection attempt.
    Reconnecting {
        /// Attempt number.
        attempt: u32,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let text = WsMessage::text("hello");
        assert!(text.is_text());
        assert_eq!(text.as_text(), Some("hello"));

        let binary = WsMessage::binary(vec![1, 2, 3]);
        assert!(binary.is_binary());
        assert_eq!(binary.as_binary(), Some(&[1, 2, 3][..]));
    }

    #[test]
    fn test_json_message() {
        #[derive(Debug, Serialize, Deserialize, PartialEq)]
        struct Data {
            name: String,
        }

        let msg = WsMessage::json(&Data {
            name: "test".to_string(),
        })
        .unwrap();
        assert!(msg.is_json());

        let parsed: Data = msg.as_json().unwrap().unwrap();
        assert_eq!(parsed.name, "test");
    }
}
