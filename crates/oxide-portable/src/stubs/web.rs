//! Web/WASM target stub implementations.
//!
//! These stubs provide placeholder implementations for web-specific APIs
//! when compiling for non-web targets, and also document the web APIs
//! that will be available when targeting WASM.

use super::{Stub, StubError, StubResult};

/// Web platform stub.
///
/// On actual web targets (wasm32), these would be replaced with real
/// implementations using web-sys and wasm-bindgen.
pub struct WebPlatform;

impl Stub for WebPlatform {
    const FEATURE_NAME: &'static str = "web-platform";

    fn is_available() -> bool {
        cfg!(target_arch = "wasm32")
    }
}

/// Stub for DOM operations.
pub mod dom {
    use super::*;

    /// Element stub for DOM elements.
    #[derive(Debug, Clone)]
    pub struct Element {
        /// Element ID (for debugging)
        pub id: Option<String>,
        /// Tag name
        pub tag_name: String,
    }

    impl Element {
        /// Create a new element stub.
        pub fn new(tag_name: &str) -> Self {
            Self {
                id: None,
                tag_name: tag_name.to_string(),
            }
        }
    }

    /// Get an element by ID.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_element_by_id(_id: &str) -> StubResult<Element> {
        Err(StubError::unavailable("DOM"))
    }

    /// Get an element by ID (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn get_element_by_id(id: &str) -> StubResult<Element> {
        // In a real implementation, this would use web_sys
        Ok(Element {
            id: Some(id.to_string()),
            tag_name: "div".to_string(),
        })
    }

    /// Create a new element.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn create_element(_tag: &str) -> StubResult<Element> {
        Err(StubError::unavailable("DOM"))
    }

    /// Create a new element (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn create_element(tag: &str) -> StubResult<Element> {
        Ok(Element::new(tag))
    }

    /// Query selector.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn query_selector(_selector: &str) -> StubResult<Option<Element>> {
        Err(StubError::unavailable("DOM"))
    }

    /// Query selector (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn query_selector(_selector: &str) -> StubResult<Option<Element>> {
        Ok(None)
    }
}

/// Stub for browser console.
pub mod console {
    #[allow(unused_imports)]
    use super::*;

    /// Log to console.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn log(message: &str) {
        // On non-web, just use tracing
        tracing::info!("[web-console] {}", message);
    }

    /// Log to console (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn log(message: &str) {
        // In real implementation, use web_sys::console::log_1
        // For now, this is a placeholder
        let _ = message;
    }

    /// Log warning to console.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn warn(message: &str) {
        tracing::warn!("[web-console] {}", message);
    }

    /// Log warning to console (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn warn(message: &str) {
        let _ = message;
    }

    /// Log error to console.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn error(message: &str) {
        tracing::error!("[web-console] {}", message);
    }

    /// Log error to console (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn error(message: &str) {
        let _ = message;
    }
}

/// Stub for browser storage.
pub mod storage {
    use super::*;

    /// Storage type.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum StorageType {
        /// Local storage (persistent)
        Local,
        /// Session storage (cleared when session ends)
        Session,
    }

    /// Get an item from storage.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_item(_storage: StorageType, _key: &str) -> StubResult<Option<String>> {
        Err(StubError::unavailable("Web Storage"))
    }

    /// Get an item from storage (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn get_item(_storage: StorageType, _key: &str) -> StubResult<Option<String>> {
        // Real implementation would use web_sys
        Ok(None)
    }

    /// Set an item in storage.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn set_item(_storage: StorageType, _key: &str, _value: &str) -> StubResult<()> {
        Err(StubError::unavailable("Web Storage"))
    }

    /// Set an item in storage (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn set_item(_storage: StorageType, _key: &str, _value: &str) -> StubResult<()> {
        Ok(())
    }

    /// Remove an item from storage.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn remove_item(_storage: StorageType, _key: &str) -> StubResult<()> {
        Err(StubError::unavailable("Web Storage"))
    }

    /// Remove an item from storage (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn remove_item(_storage: StorageType, _key: &str) -> StubResult<()> {
        Ok(())
    }

    /// Clear storage.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn clear(_storage: StorageType) -> StubResult<()> {
        Err(StubError::unavailable("Web Storage"))
    }

    /// Clear storage (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn clear(_storage: StorageType) -> StubResult<()> {
        Ok(())
    }
}

/// Stub for fetch API.
pub mod fetch {
    use super::*;

    /// HTTP method.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Method {
        /// GET request
        Get,
        /// POST request
        Post,
        /// PUT request
        Put,
        /// DELETE request
        Delete,
        /// PATCH request
        Patch,
    }

    /// Request configuration.
    #[derive(Debug, Clone)]
    pub struct RequestConfig {
        /// HTTP method
        pub method: Method,
        /// Request headers
        pub headers: Vec<(String, String)>,
        /// Request body
        pub body: Option<String>,
    }

    impl Default for RequestConfig {
        fn default() -> Self {
            Self {
                method: Method::Get,
                headers: Vec::new(),
                body: None,
            }
        }
    }

    /// Response from fetch.
    #[derive(Debug, Clone)]
    pub struct Response {
        /// HTTP status code
        pub status: u16,
        /// Response body
        pub body: String,
        /// Response headers
        pub headers: Vec<(String, String)>,
    }

    /// Fetch a URL.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn fetch(_url: &str, _config: RequestConfig) -> StubResult<Response> {
        Err(StubError::unavailable("Fetch API"))
    }

    /// Fetch a URL (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub async fn fetch(_url: &str, _config: RequestConfig) -> StubResult<Response> {
        // Real implementation would use web_sys and wasm_bindgen_futures
        Ok(Response {
            status: 200,
            body: String::new(),
            headers: Vec::new(),
        })
    }
}

/// Stub for WebGL/WebGPU.
pub mod graphics {
    use super::*;

    /// Check if WebGPU is available.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn has_webgpu() -> bool {
        false
    }

    /// Check if WebGPU is available (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn has_webgpu() -> bool {
        // Would check navigator.gpu in real implementation
        true
    }

    /// Check if WebGL is available.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn has_webgl() -> bool {
        false
    }

    /// Check if WebGL is available (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn has_webgl() -> bool {
        true
    }

    /// Get a WebGPU adapter.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn get_webgpu_adapter() -> StubResult<()> {
        Err(StubError::unavailable("WebGPU"))
    }

    /// Get a WebGPU adapter (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub async fn get_webgpu_adapter() -> StubResult<()> {
        Ok(())
    }
}

/// Web-specific window operations.
pub mod window {
    use super::*;

    /// Get the current URL.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_location() -> StubResult<String> {
        Err(StubError::unavailable("window.location"))
    }

    /// Get the current URL (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn get_location() -> StubResult<String> {
        Ok("about:blank".to_string())
    }

    /// Navigate to a URL.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn navigate(_url: &str) -> StubResult<()> {
        Err(StubError::unavailable("window.location"))
    }

    /// Navigate to a URL (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn navigate(_url: &str) -> StubResult<()> {
        Ok(())
    }

    /// Get window dimensions.
    #[cfg(not(target_arch = "wasm32"))]
    pub fn get_dimensions() -> StubResult<(u32, u32)> {
        Err(StubError::unavailable("window dimensions"))
    }

    /// Get window dimensions (real implementation for wasm).
    #[cfg(target_arch = "wasm32")]
    pub fn get_dimensions() -> StubResult<(u32, u32)> {
        Ok((1920, 1080))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_web_platform_availability() {
        // On non-wasm targets, web platform should not be available
        #[cfg(not(target_arch = "wasm32"))]
        assert!(!WebPlatform::is_available());

        #[cfg(target_arch = "wasm32")]
        assert!(WebPlatform::is_available());
    }

    #[test]
    fn test_dom_stub() {
        #[cfg(not(target_arch = "wasm32"))]
        {
            let result = dom::get_element_by_id("test");
            assert!(result.is_err());
        }
    }

    #[test]
    fn test_console_stub() {
        // Console stubs should not panic
        console::log("test message");
        console::warn("test warning");
        console::error("test error");
    }
}
