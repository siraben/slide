//! Crate `slide_ls` implements a language server for [slide](libslide).

#![deny(warnings)]

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use libslide::*;

use std::collections::HashMap;
use std::cell::Cell;
use std::sync::{Mutex, MutexGuard};

mod diagnostics;
use diagnostics::convert_diagnostics;

struct ProgramInfo {
    #[allow(unused)]
    original: StmtList,
    #[allow(unused)]
    simplified: StmtList,
}

type DocumentRegistry = HashMap<Url, ProgramInfo>;

struct SlideLS {
    client: Client,
    document_registry: Mutex<Cell<DocumentRegistry>>,
}

impl SlideLS {
    fn new(client: Client) -> Self {
        Self {
            client,
            document_registry: Default::default(),
        }
    }

    fn doc_registry(&self) -> MutexGuard<Cell<DocumentRegistry>> {
        self.document_registry.lock().expect("Failed to read document_registry")
    }

    async fn change(&self, doc: Url, text: String, version: Option<i64>) {
        // On document change, we do the following:
        //   1. Reparse the program source code
        //   2. Evaluate the program
        //      - There is a tradeoff between evaluating everything at once on change and lazily
        //        evaluating on queries. For now, we need to do it in this step because some
        //        diagnostics (i.e. validation) cannot be done without performing evaluation
        //        anyway.
        //        A future flow could be to use a "query" model, whereby we incrementally parse,
        //        evaluate, and publish diagnostics localized to a single statement.
        //        But we are far away from that being important.
        //   3. Since we're already here, publish any diagnostics we discovered.
        //
        // We cache both the original program AST and evaluated AST so we can answer later queries
        // for original/optimized statements without re-evaluation.
        
        let context = ProgramContext::default().lint(true);
        let ScanResult { tokens, diagnostics: scan_diags } = scan(&*text);
        let ParseResult { program, diagnostics: parse_diags } = parse_statements(tokens, &text);
        let lint_diags = lint_stmt(&program, &text);
        let EvaluationResult { simplified, diagnostics: eval_diags } = evaluate(program.clone(), &context).expect("Evaluation failed.");

        self.doc_registry().get_mut().insert(doc.clone(), ProgramInfo { original: program, simplified });

        let diags = [scan_diags, parse_diags, lint_diags, eval_diags]
            .iter()
            .flat_map(|diags| convert_diagnostics(diags, "slide", &doc, &text))
            .collect();

        self.client.publish_diagnostics(doc.clone(), diags, version).await;
    }

    fn close(&self, doc: &Url) {
        self.doc_registry().get_mut().remove(doc);
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for SlideLS {
    async fn initialize(&self, _params: InitializeParams) -> Result<InitializeResult> {
        // let hover_provider = Some(HoverProviderCapability::Simple(true));
        let text_document_sync = Some(TextDocumentSyncCapability::Options(TextDocumentSyncOptions {
            open_close: Some(true),
            change: Some(TextDocumentSyncKind::Full),
            ..TextDocumentSyncOptions::default()
        }));
        Ok(InitializeResult{
            capabilities: ServerCapabilities {
                text_document_sync,
                // hover_provider,
                ..ServerCapabilities::default()
            },
            ..InitializeResult::default()
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::Info, "Slide language server initialized.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let TextDocumentItem { uri, text, version, .. } = params.text_document;
        self.change(uri, text, Some(version)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let VersionedTextDocumentIdentifier { uri, version, .. } = params.text_document;
        // NOTE: We specify that we expect full-content syncs in the server capabilities,
        // so here we assume the only change passed is a change of the entire document's content.
        let TextDocumentContentChangeEvent { text, .. } = params.content_changes.into_iter().next().unwrap();
        self.change(uri, text, version).await;
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let TextDocumentIdentifier { uri } = params.text_document;
        self.close(&uri);
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, messages) = LspService::new(SlideLS::new);
    Server::new(stdin, stdout)
        .interleave(messages)
        .serve(service)
        .await;
}
