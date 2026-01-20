//! LSP Server Implementation
//!
//! The main LSP server that handles protocol messages and delegates
//! to specialized engines for completion, diagnostics, hover, etc.

use crate::capabilities::ServerCapabilities;
use crate::completion::CompletionEngine;
use crate::diagnostics::DiagnosticsEngine;
use crate::document::DocumentStore;
use crate::hover::HoverEngine;
use crate::jump::JumpEngine;
use crate::project::ProjectContext;
use crate::protocol::CodeActionEngine;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

/// OxideKit Language Server
pub struct OxideLspServer {
    /// LSP client for sending notifications/requests
    client: Client,
    /// Document store for open files
    documents: Arc<RwLock<DocumentStore>>,
    /// Project context with schemas and manifests
    project: Arc<RwLock<Option<ProjectContext>>>,
    /// Completion engine
    completion: Arc<CompletionEngine>,
    /// Diagnostics engine
    diagnostics: Arc<DiagnosticsEngine>,
    /// Hover engine
    hover: Arc<HoverEngine>,
    /// Jump-to-definition engine
    jump: Arc<JumpEngine>,
    /// Code action engine
    code_actions: Arc<CodeActionEngine>,
}

impl OxideLspServer {
    /// Create a new LSP server instance
    pub fn new(client: Client) -> Self {
        let documents = Arc::new(RwLock::new(DocumentStore::new()));
        let project = Arc::new(RwLock::new(None));

        Self {
            client,
            documents: documents.clone(),
            project: project.clone(),
            completion: Arc::new(CompletionEngine::new(documents.clone(), project.clone())),
            diagnostics: Arc::new(DiagnosticsEngine::new(documents.clone(), project.clone())),
            hover: Arc::new(HoverEngine::new(documents.clone(), project.clone())),
            jump: Arc::new(JumpEngine::new(documents.clone(), project.clone())),
            code_actions: Arc::new(CodeActionEngine::new(documents.clone(), project.clone())),
        }
    }

    /// Initialize project context from workspace folder
    async fn init_project(&self, root_uri: Option<Url>) {
        if let Some(uri) = root_uri {
            if let Ok(path) = uri.to_file_path() {
                match ProjectContext::load(&path).await {
                    Ok(ctx) => {
                        let mut project = self.project.write().await;
                        *project = Some(ctx);
                        tracing::info!("Loaded project context from {:?}", path);
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load project context: {}", e);
                    }
                }
            }
        }
    }

    /// Publish diagnostics for a document
    async fn publish_diagnostics(&self, uri: Url) {
        let diagnostics = self.diagnostics.analyze(&uri).await;
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for OxideLspServer {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        tracing::info!("Initializing OxideKit LSP server");

        // Initialize project context
        let root_uri = params
            .root_uri
            .or_else(|| params.workspace_folders.as_ref()?.first()?.uri.clone().into());

        self.init_project(root_uri).await;

        Ok(InitializeResult {
            capabilities: ServerCapabilities::default_capabilities(),
            server_info: Some(ServerInfo {
                name: "oxide-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        tracing::info!("OxideKit LSP server initialized");
        self.client
            .log_message(MessageType::INFO, "OxideKit LSP ready")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        tracing::info!("Shutting down OxideKit LSP server");
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text;
        let language_id = params.text_document.language_id;
        let version = params.text_document.version;

        tracing::debug!("Document opened: {}", uri);

        // Store document
        {
            let mut docs = self.documents.write().await;
            docs.open(uri.clone(), text, language_id, version);
        }

        // Publish diagnostics
        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();

        tracing::debug!("Document changed: {}", uri);

        // Update document
        {
            let mut docs = self.documents.write().await;
            for change in params.content_changes {
                docs.update(&uri, change.text, params.text_document.version);
            }
        }

        // Publish diagnostics
        self.publish_diagnostics(uri).await;
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document saved: {}", uri);

        // Re-analyze on save
        self.publish_diagnostics(uri).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri;
        tracing::debug!("Document closed: {}", uri);

        // Remove from store
        {
            let mut docs = self.documents.write().await;
            docs.close(&uri);
        }

        // Clear diagnostics
        self.client.publish_diagnostics(uri, vec![], None).await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = &params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        tracing::debug!("Completion request at {}:{}", uri, position.line);

        let items = self.completion.complete(uri, position).await;

        if items.is_empty() {
            Ok(None)
        } else {
            Ok(Some(CompletionResponse::Array(items)))
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Hover request at {}:{}", uri, position.line);

        Ok(self.hover.hover(uri, position).await)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        tracing::debug!("Go to definition at {}:{}", uri, position.line);

        Ok(self.jump.goto_definition(uri, position).await)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = &params.text_document.uri;
        let range = params.range;
        let diagnostics = &params.context.diagnostics;

        tracing::debug!("Code action request at {}", uri);

        let actions = self.code_actions.actions(uri, range, diagnostics).await;

        if actions.is_empty() {
            Ok(None)
        } else {
            Ok(Some(actions))
        }
    }

    async fn did_change_configuration(&self, _params: DidChangeConfigurationParams) {
        tracing::debug!("Configuration changed");
        // Reload project context if configuration changes
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        tracing::debug!("Watched files changed");

        for change in params.changes {
            // Reload project context if manifest files change
            if change.uri.path().ends_with("oxide.toml")
                || change.uri.path().ends_with("oxide.ai.json")
            {
                if let Ok(path) = change.uri.to_file_path() {
                    if let Some(parent) = path.parent() {
                        match ProjectContext::load(parent).await {
                            Ok(ctx) => {
                                let mut project = self.project.write().await;
                                *project = Some(ctx);
                            }
                            Err(e) => {
                                tracing::warn!("Failed to reload project: {}", e);
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Integration tests would go here
    // They require mocking the tower-lsp Client
}
