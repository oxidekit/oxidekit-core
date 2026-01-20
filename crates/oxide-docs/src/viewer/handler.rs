//! HTTP request handler for documentation viewer

use crate::{DocsError, DocsResult};
use std::collections::HashMap;
use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};
use tiny_http::{Header, Request, Response, StatusCode};
use tracing::{debug, warn};

/// Request handler for the documentation server
pub struct RequestHandler {
    /// Root directory of the documentation bundle
    root: PathBuf,
    /// MIME type mappings
    mime_types: HashMap<&'static str, &'static str>,
}

impl RequestHandler {
    /// Create a new request handler
    pub fn new(root: &Path) -> DocsResult<Self> {
        if !root.exists() {
            return Err(DocsError::BundleNotFound(root.to_path_buf()));
        }

        let mut mime_types = HashMap::new();
        mime_types.insert("html", "text/html; charset=utf-8");
        mime_types.insert("css", "text/css; charset=utf-8");
        mime_types.insert("js", "application/javascript; charset=utf-8");
        mime_types.insert("json", "application/json; charset=utf-8");
        mime_types.insert("png", "image/png");
        mime_types.insert("jpg", "image/jpeg");
        mime_types.insert("jpeg", "image/jpeg");
        mime_types.insert("gif", "image/gif");
        mime_types.insert("svg", "image/svg+xml");
        mime_types.insert("ico", "image/x-icon");
        mime_types.insert("woff", "font/woff");
        mime_types.insert("woff2", "font/woff2");
        mime_types.insert("ttf", "font/ttf");
        mime_types.insert("otf", "font/otf");

        Ok(Self {
            root: root.to_path_buf(),
            mime_types,
        })
    }

    /// Handle an incoming request
    pub fn handle(&self, request: &Request) -> Response<Cursor<Vec<u8>>> {
        let url = request.url();
        debug!("Request: {} {}", request.method(), url);

        // Handle API requests
        if url.starts_with("/api/") {
            return self.handle_api(url);
        }

        // Serve static files
        self.handle_static(url)
    }

    /// Handle API requests
    fn handle_api(&self, url: &str) -> Response<Cursor<Vec<u8>>> {
        match url {
            "/api/search" => self.handle_search_api(url),
            "/api/manifest" => self.handle_manifest_api(),
            _ => self.not_found(),
        }
    }

    /// Handle search API
    fn handle_search_api(&self, url: &str) -> Response<Cursor<Vec<u8>>> {
        // Parse query parameter
        let query = url
            .split('?')
            .nth(1)
            .and_then(|qs| {
                qs.split('&')
                    .find(|p| p.starts_with("q="))
                    .map(|p| &p[2..])
            })
            .unwrap_or("");

        // TODO: Implement actual search
        // For now, return empty results
        let response = serde_json::json!({
            "results": [],
            "query": query,
            "total": 0
        });

        self.json_response(&response)
    }

    /// Handle manifest API
    fn handle_manifest_api(&self) -> Response<Cursor<Vec<u8>>> {
        let manifest_path = self.root.join("manifest.json");

        if let Ok(content) = fs::read_to_string(&manifest_path) {
            Response::from_data(content.into_bytes())
                .with_header(
                    Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
                )
        } else {
            self.not_found()
        }
    }

    /// Handle static file requests
    fn handle_static(&self, url: &str) -> Response<Cursor<Vec<u8>>> {
        // Normalize path
        let path = if url == "/" {
            "index.html".to_string()
        } else {
            url.trim_start_matches('/').to_string()
        };

        // Security: prevent path traversal
        let safe_path = Path::new(&path)
            .components()
            .filter(|c| matches!(c, std::path::Component::Normal(_)))
            .collect::<PathBuf>();

        let file_path = self.root.join(&safe_path);

        // Try to find the file
        let resolved_path = if file_path.is_file() {
            file_path
        } else if file_path.is_dir() {
            // Try index.html in directory
            let index = file_path.join("index.html");
            if index.is_file() {
                index
            } else {
                return self.not_found();
            }
        } else {
            // Try adding .html extension
            let with_html = file_path.with_extension("html");
            if with_html.is_file() {
                with_html
            } else {
                return self.not_found();
            }
        };

        // Read and serve the file
        match fs::read(&resolved_path) {
            Ok(content) => {
                let mime_type = self.get_mime_type(&resolved_path);
                Response::from_data(content)
                    .with_header(
                        Header::from_bytes(&b"Content-Type"[..], mime_type.as_bytes()).unwrap(),
                    )
            }
            Err(e) => {
                warn!("Failed to read file {:?}: {}", resolved_path, e);
                self.not_found()
            }
        }
    }

    /// Get MIME type for a file
    fn get_mime_type(&self, path: &Path) -> &str {
        path.extension()
            .and_then(|ext| ext.to_str())
            .and_then(|ext| self.mime_types.get(ext))
            .copied()
            .unwrap_or("application/octet-stream")
    }

    /// Return a JSON response
    fn json_response(&self, data: &serde_json::Value) -> Response<Cursor<Vec<u8>>> {
        let body = serde_json::to_vec(data).unwrap_or_default();
        Response::from_data(body)
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"application/json"[..]).unwrap(),
            )
    }

    /// Return a 404 Not Found response
    fn not_found(&self) -> Response<Cursor<Vec<u8>>> {
        let body = r#"<!DOCTYPE html>
<html>
<head><title>404 Not Found</title></head>
<body>
<h1>404 Not Found</h1>
<p>The requested page was not found.</p>
<p><a href="/">Return to documentation</a></p>
</body>
</html>"#;

        Response::from_data(body.as_bytes().to_vec())
            .with_status_code(StatusCode(404))
            .with_header(
                Header::from_bytes(&b"Content-Type"[..], &b"text/html; charset=utf-8"[..]).unwrap(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_get_mime_type() {
        let dir = TempDir::new().unwrap();
        let handler = RequestHandler::new(dir.path()).unwrap();

        assert_eq!(handler.get_mime_type(Path::new("test.html")), "text/html; charset=utf-8");
        assert_eq!(handler.get_mime_type(Path::new("test.css")), "text/css; charset=utf-8");
        assert_eq!(handler.get_mime_type(Path::new("test.js")), "application/javascript; charset=utf-8");
        assert_eq!(handler.get_mime_type(Path::new("test.png")), "image/png");
        assert_eq!(handler.get_mime_type(Path::new("test.unknown")), "application/octet-stream");
    }
}
