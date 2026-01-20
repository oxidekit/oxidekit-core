//! HTTP response types.

use serde::de::DeserializeOwned;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

use crate::error::{NetworkError, NetworkResult};

/// HTTP response representation.
#[derive(Debug, Clone)]
pub struct HttpResponse {
    /// The request ID this response corresponds to.
    pub request_id: Uuid,
    /// HTTP status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response body bytes.
    pub body: Vec<u8>,
    /// Final URL (may differ from request URL due to redirects).
    pub final_url: String,
    /// Time taken for the request.
    pub duration: Duration,
    /// Number of redirects followed.
    pub redirects: u32,
    /// Custom metadata from interceptors.
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HttpResponse {
    /// Create a new response.
    pub fn new(
        request_id: Uuid,
        status: u16,
        headers: HashMap<String, String>,
        body: Vec<u8>,
        final_url: String,
        duration: Duration,
    ) -> Self {
        Self {
            request_id,
            status,
            headers,
            body,
            final_url,
            duration,
            redirects: 0,
            metadata: HashMap::new(),
        }
    }

    /// Check if the response was successful (2xx status).
    pub fn is_success(&self) -> bool {
        (200..300).contains(&self.status)
    }

    /// Check if the response indicates a redirect (3xx status).
    pub fn is_redirect(&self) -> bool {
        (300..400).contains(&self.status)
    }

    /// Check if the response indicates a client error (4xx status).
    pub fn is_client_error(&self) -> bool {
        (400..500).contains(&self.status)
    }

    /// Check if the response indicates a server error (5xx status).
    pub fn is_server_error(&self) -> bool {
        (500..600).contains(&self.status)
    }

    /// Get the response body as a string.
    pub fn text(&self) -> Result<String, std::string::FromUtf8Error> {
        String::from_utf8(self.body.clone())
    }

    /// Get the response body as a string, lossy.
    pub fn text_lossy(&self) -> String {
        String::from_utf8_lossy(&self.body).to_string()
    }

    /// Deserialize the response body as JSON.
    pub fn json<T: DeserializeOwned>(&self) -> NetworkResult<T> {
        serde_json::from_slice(&self.body).map_err(NetworkError::JsonError)
    }

    /// Get the content type header value.
    pub fn content_type(&self) -> Option<&str> {
        self.headers.get("content-type").map(|s| s.as_str())
    }

    /// Get the content length if provided.
    pub fn content_length(&self) -> Option<usize> {
        self.headers
            .get("content-length")
            .and_then(|s| s.parse().ok())
    }

    /// Get a header value by name (case-insensitive).
    pub fn header(&self, name: &str) -> Option<&str> {
        let name_lower = name.to_lowercase();
        self.headers
            .iter()
            .find(|(k, _)| k.to_lowercase() == name_lower)
            .map(|(_, v)| v.as_str())
    }

    /// Check if a header exists (case-insensitive).
    pub fn has_header(&self, name: &str) -> bool {
        self.header(name).is_some()
    }

    /// Get the ETag header if present.
    pub fn etag(&self) -> Option<&str> {
        self.header("etag")
    }

    /// Get the Last-Modified header if present.
    pub fn last_modified(&self) -> Option<&str> {
        self.header("last-modified")
    }

    /// Get the Location header (for redirects).
    pub fn location(&self) -> Option<&str> {
        self.header("location")
    }

    /// Get retry-after header value in seconds.
    pub fn retry_after(&self) -> Option<u64> {
        self.header("retry-after")
            .and_then(|v| v.parse::<u64>().ok())
    }

    /// Convert to a result based on status code.
    pub fn into_result(self) -> NetworkResult<Self> {
        match self.status {
            200..=299 => Ok(self),
            401 => Err(NetworkError::AuthenticationRequired),
            403 => Err(NetworkError::Forbidden {
                resource: self.final_url.clone(),
            }),
            429 => Err(NetworkError::RateLimited {
                retry_after_secs: self.retry_after(),
            }),
            400..=499 => Err(NetworkError::ClientError {
                status: self.status,
                body: self.text().ok(),
            }),
            500..=599 => Err(NetworkError::ServerError {
                status: self.status,
                body: self.text().ok(),
            }),
            _ => Err(NetworkError::ClientError {
                status: self.status,
                body: self.text().ok(),
            }),
        }
    }

    /// Ensure the response was successful, or return an error.
    pub fn error_for_status(self) -> NetworkResult<Self> {
        self.into_result()
    }
}

/// Builder for constructing mock responses in tests.
#[derive(Debug, Clone, Default)]
pub struct ResponseBuilder {
    status: u16,
    headers: HashMap<String, String>,
    body: Vec<u8>,
    final_url: String,
}

impl ResponseBuilder {
    /// Create a new response builder.
    pub fn new() -> Self {
        Self {
            status: 200,
            headers: HashMap::new(),
            body: Vec::new(),
            final_url: String::new(),
        }
    }

    /// Set the status code.
    pub fn status(mut self, status: u16) -> Self {
        self.status = status;
        self
    }

    /// Add a header.
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(name.into(), value.into());
        self
    }

    /// Set JSON body.
    pub fn json<T: serde::Serialize>(mut self, body: &T) -> Self {
        self.body = serde_json::to_vec(body).unwrap_or_default();
        self.headers
            .insert("content-type".to_string(), "application/json".to_string());
        self
    }

    /// Set raw body.
    pub fn body(mut self, body: impl Into<Vec<u8>>) -> Self {
        self.body = body.into();
        self
    }

    /// Set text body.
    pub fn text(mut self, body: impl Into<String>) -> Self {
        self.body = body.into().into_bytes();
        self.headers
            .insert("content-type".to_string(), "text/plain".to_string());
        self
    }

    /// Set the final URL.
    pub fn url(mut self, url: impl Into<String>) -> Self {
        self.final_url = url.into();
        self
    }

    /// Build the response.
    pub fn build(self) -> HttpResponse {
        HttpResponse::new(
            Uuid::new_v4(),
            self.status,
            self.headers,
            self.body,
            self.final_url,
            Duration::ZERO,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_response_status_checks() {
        let success = ResponseBuilder::new().status(200).build();
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let not_found = ResponseBuilder::new().status(404).build();
        assert!(!not_found.is_success());
        assert!(not_found.is_client_error());

        let server_err = ResponseBuilder::new().status(500).build();
        assert!(server_err.is_server_error());
    }

    #[test]
    fn test_response_json() {
        #[derive(Debug, serde::Deserialize, PartialEq)]
        struct Data {
            name: String,
            value: i32,
        }

        let response = ResponseBuilder::new()
            .json(&serde_json::json!({
                "name": "test",
                "value": 42
            }))
            .build();

        let data: Data = response.json().unwrap();
        assert_eq!(data.name, "test");
        assert_eq!(data.value, 42);
    }

    #[test]
    fn test_error_for_status() {
        let success = ResponseBuilder::new().status(200).build();
        assert!(success.error_for_status().is_ok());

        let not_found = ResponseBuilder::new().status(404).build();
        assert!(matches!(
            not_found.error_for_status(),
            Err(NetworkError::ClientError { status: 404, .. })
        ));

        let unauthorized = ResponseBuilder::new().status(401).build();
        assert!(matches!(
            unauthorized.error_for_status(),
            Err(NetworkError::AuthenticationRequired)
        ));
    }
}
