//! OxideKit Language Server Binary
//!
//! Entry point for the oxide-lsp executable.
//! Starts the LSP server and handles stdio communication.

use oxide_lsp::OxideLspServer;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("Starting OxideKit Language Server");

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(OxideLspServer::new);

    Server::new(stdin, stdout, socket).serve(service).await;
}
