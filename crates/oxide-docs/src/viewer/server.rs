//! HTTP server for the documentation viewer

use crate::bundler::DocBundle;
use crate::{DocsError, DocsResult};
use std::net::TcpListener;
use std::path::Path;
use tiny_http::Server;
use tracing::{error, info};

use super::handler::RequestHandler;

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {
    /// Port to listen on
    pub port: u16,
    /// Host to bind to
    pub host: String,
    /// Whether to open browser automatically
    pub open_browser: bool,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            port: 3030,
            host: "127.0.0.1".to_string(),
            open_browser: false,
        }
    }
}

/// Documentation server
pub struct DocServer {
    config: ServerConfig,
    handler: RequestHandler,
}

impl DocServer {
    /// Create a new documentation server
    pub fn new(bundle_root: &Path, config: ServerConfig) -> DocsResult<Self> {
        let handler = RequestHandler::new(bundle_root)?;
        Ok(Self { config, handler })
    }

    /// Start the server (blocking)
    pub fn run(&self) -> DocsResult<()> {
        let addr = format!("{}:{}", self.config.host, self.config.port);

        let server = Server::http(&addr).map_err(|e| DocsError::Server(e.to_string()))?;

        info!("Documentation server running at http://{}", addr);
        println!("\n  OxideKit Documentation Server");
        println!("  ==============================");
        println!("  Local:   http://{}", addr);
        println!("\n  Press Ctrl+C to stop\n");

        if self.config.open_browser {
            super::open_in_browser(self.config.port)?;
        }

        // Handle requests
        for request in server.incoming_requests() {
            let response = self.handler.handle(&request);
            if let Err(e) = request.respond(response) {
                error!("Failed to send response: {}", e);
            }
        }

        Ok(())
    }
}

/// Serve a documentation bundle (convenience function)
pub fn serve_bundle(bundle: &DocBundle, port: u16) -> DocsResult<()> {
    let config = ServerConfig {
        port,
        open_browser: false,
        ..Default::default()
    };

    let server = DocServer::new(bundle.root_dir(), config)?;
    server.run()
}

/// Check if a port is available
pub fn is_port_available(port: u16) -> bool {
    TcpListener::bind(("127.0.0.1", port)).is_ok()
}

/// Find an available port starting from the given port
pub fn find_available_port(start: u16) -> Option<u16> {
    (start..start + 100).find(|&port| is_port_available(port))
}
