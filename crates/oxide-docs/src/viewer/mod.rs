//! Offline documentation viewer
//!
//! Provides a local HTTP server to browse documentation offline.

mod server;
mod handler;

pub use server::{serve_bundle, DocServer, ServerConfig};
pub use handler::RequestHandler;

use crate::bundler::DocBundle;
use crate::DocsResult;

/// Start the documentation viewer server
pub fn start_viewer(bundle: &DocBundle, port: u16) -> DocsResult<()> {
    serve_bundle(bundle, port)
}

/// Open the documentation in the default browser
pub fn open_in_browser(port: u16) -> DocsResult<()> {
    let url = format!("http://localhost:{}", port);

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(&url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(&url)
            .spawn()
            .ok();
    }

    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", &url])
            .spawn()
            .ok();
    }

    Ok(())
}
