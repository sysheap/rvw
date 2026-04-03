pub mod diff;

use anyhow::Result;
use std::path::PathBuf;
use tower_lsp::jsonrpc;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use crate::git;

pub async fn run_lsp_server(repo_path: PathBuf, base_branch: String) -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        repo_path,
        base_branch,
    });

    Server::new(stdin, stdout, socket).serve(service).await;
    Ok(())
}

struct Backend {
    client: Client,
    repo_path: PathBuf,
    base_branch: String,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> jsonrpc::Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "rvw LSP server initialized")
            .await;
    }

    async fn shutdown(&self) -> jsonrpc::Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        if let Some(diagnostics) = self.compute_diagnostics(&uri) {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        // Re-publish diagnostics on change (positions might shift, but we
        // keep the original diff-based diagnostics for simplicity)
        let uri = params.text_document.uri;
        if let Some(diagnostics) = self.compute_diagnostics(&uri) {
            self.client
                .publish_diagnostics(uri, diagnostics, None)
                .await;
        }
    }

    async fn hover(&self, params: HoverParams) -> jsonrpc::Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let file_path = match self.uri_to_relative_path(&uri) {
            Some(p) => p,
            None => return Ok(None),
        };

        let hunks = match git::diff_hunks_for_file(&self.repo_path, &self.base_branch, &file_path)
        {
            Ok(h) => h,
            Err(_) => return Ok(None),
        };

        // Check if the cursor is on a changed line
        let line = position.line + 1; // LSP is 0-based, git is 1-based

        for hunk in &hunks {
            let hunk_end = hunk.new_start + hunk.new_lines;
            if line >= hunk.new_start && line < hunk_end {
                // This line is in a changed hunk - show old code
                let old_code = if hunk.removed_lines.is_empty() {
                    "*(new code — not present on base branch)*".to_string()
                } else {
                    let old_lines: Vec<&str> = hunk
                        .removed_lines
                        .iter()
                        .map(|(_, content)| content.as_str())
                        .collect();

                    // Detect language for syntax highlighting
                    let lang = std::path::Path::new(&file_path)
                        .extension()
                        .and_then(|e| e.to_str())
                        .unwrap_or("");

                    format!(
                        "**Before ({})**\n```{}\n{}```",
                        self.base_branch,
                        lang,
                        old_lines.join("")
                    )
                };

                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: old_code,
                    }),
                    range: None,
                }));
            }
        }

        Ok(None)
    }
}

impl Backend {
    fn uri_to_relative_path(&self, uri: &Url) -> Option<String> {
        let file_path = uri.to_file_path().ok()?;
        let relative = file_path.strip_prefix(&self.repo_path).ok()?;
        Some(relative.to_string_lossy().to_string())
    }

    fn compute_diagnostics(&self, uri: &Url) -> Option<Vec<Diagnostic>> {
        let file_path = self.uri_to_relative_path(uri)?;
        let hunks =
            git::diff_hunks_for_file(&self.repo_path, &self.base_branch, &file_path).ok()?;

        let diagnostics = diff::hunks_to_diagnostics(&hunks);
        Some(diagnostics)
    }
}
