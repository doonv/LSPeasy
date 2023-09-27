#![allow(clippy::useless_format)]
#![doc = include_str!("../README.md")]

use lsp_server::{
    Connection, IoThreads, Message, Notification, ProtocolError, RequestId, Response,
};

pub use lsp_types;
use lsp_types::{
    CompletionItem, Diagnostic, DocumentDiagnosticReport, FullDocumentDiagnosticReport,
    LogMessageParams, MessageType, PublishDiagnosticsParams, RelatedFullDocumentDiagnosticReport,
    ServerCapabilities, TextDocumentContentChangeEvent, TextDocumentItem, Url,
    VersionedTextDocumentIdentifier,
};

/// A LSPeasy language server
pub struct LanguageServer {
    connection: Connection,
    io_threads: Option<IoThreads>,
    params: serde_json::Value,
}

impl LanguageServer {
    /// Create a new language server, provided with server
    /// capabilities and a handler.
    pub fn new(
        capabilities: &ServerCapabilities,
        handler: impl LanguageServerHandler,
    ) -> Result<LanguageServer, ProtocolError> {
        let (connection, io_threads) = Connection::stdio();

        let capabilities_json = serde_json::to_value(capabilities).unwrap();
        let params = connection.initialize(capabilities_json)?;

        let server = LanguageServer {
            connection,
            io_threads: Some(io_threads),
            params,
        };

        server.start(handler)?;

        Ok(server)
    }

    fn start(&self, handler: impl LanguageServerHandler) -> Result<(), ProtocolError> {
        handler.init(self);

        for msg in &self.connection.receiver {
            // self.log(format!("Got a message!\n{msg:?}"), MessageType::INFO);
            match msg {
                Message::Request(req) => {
                    if self.connection.handle_shutdown(&req)? {
                        return Ok(());
                    }
                    match req.method.as_str() {
                        "textDocument/completion" => handler.completion(
                            self,
                            CompletionRequest {
                                id: req.id,
                                server: self,
                            },
                        ),
                        "textDocument/diagnostic" => handler.diagnostics(
                            self,
                            DiagnosticsRequest {
                                id: req.id,
                                server: self,
                                text_document: Url::parse(
                                    req.params
                                        .get("textDocument")
                                        .unwrap()
                                        .get("uri")
                                        .unwrap()
                                        .as_str()
                                        .unwrap(),
                                )
                                .unwrap(),
                            },
                        ),
                        _ => self.log(
                            format!("Unreconized request {req:#?}"),
                            MessageType::WARNING,
                        ),
                    }
                }
                Message::Response(_resp) => {}
                Message::Notification(mut notification) => match notification.method.as_str() {
                    "textDocument/didOpen" => handler.text_document_opened(
                        self,
                        serde_json::from_value(
                            notification.params.get_mut("textDocument").unwrap().take(),
                        )
                        .unwrap(),
                    ),
                    "textDocument/didChange" => handler.text_document_changed(
                        self,
                        serde_json::from_value(
                            notification.params.get_mut("textDocument").unwrap().take(),
                        )
                        .unwrap(),
                        serde_json::from_value(
                            notification
                                .params
                                .get_mut("contentChanges")
                                .unwrap()
                                .take(),
                        )
                        .unwrap(),
                    ),
                    _ => self.log(
                        format!("Unreconized notification {notification:#?}"),
                        MessageType::WARNING,
                    ),
                },
            }
        }

        Ok(())
    }

    /// Send a log message to the client.
    pub fn log(&self, msg: String, message_type: MessageType) {
        self.connection
            .sender
            .send(Message::Notification(Notification {
                method: "window/logMessage".to_string(),
                params: serde_json::to_value(LogMessageParams {
                    typ: message_type,
                    message: msg,
                })
                .unwrap(),
            }))
            .unwrap();
    }

    /// Send a [`Vec<_>`] of [`Diagnostic`]s (errors/warnings) to the client
    pub fn send_diagnostics(&self, file_uri: Url, diagnostics: Vec<Diagnostic>) {
        self.connection
            .sender
            .send(Message::Notification(Notification {
                method: "textDocument/publishDiagnostics".to_string(),
                params: serde_json::to_value(PublishDiagnosticsParams {
                    uri: file_uri,
                    diagnostics,
                    version: None,
                })
                .unwrap(),
            }))
            .unwrap();
    }
}

impl Drop for LanguageServer {
    fn drop(&mut self) {
        if let Some(io_threads) = self.io_threads.take() {
            io_threads.join().unwrap()
        }
    }
}

#[allow(unused_variables)]
pub trait LanguageServerHandler {
    /// A function that runs when the server starts.
    fn init(&self, server: &LanguageServer) {}

    /// A function that runs when the client requests for completions.
    ///
    /// To provide completions, respond to the client with the `respond` method.
    ///
    /// ```
    /// use lspeasy::{LanguageServerHandler, LanguageServer, CompletionRequest};
    /// use lspeasy::lsp_types::{CompletionItem, CompletionItemKind};
    ///
    /// struct MyHandler;
    /// impl LanguageServerHandler for MyHandler {
    ///     fn completion(&self, _server: &LanguageServer, req: CompletionRequest) {
    ///         req.respond(vec![CompletionItem {
    ///             label: "autocomplete".to_string(),
    ///             kind: Some(CompletionItemKind::KEYWORD),
    ///             ..CompletionItem::default()
    ///         }])
    ///     }
    /// }
    /// ```
    fn completion(&self, server: &LanguageServer, req: CompletionRequest) {}

    /// A function that runs when the client requests for diagnostics.
    ///
    /// To provide diagnostics, respond to the client with the `respond` method.
    ///
    /// **Note** that the client may not always send the diagnostics request.
    /// For this reason it may be beneficial to also do diagnostics in the
    /// [`text_document_changed`](LanguageServerHandler::text_document_changed) method
    /// and the [`LanguageServer::send_diagnostics`] method.
    fn diagnostics(&self, server: &LanguageServer, req: DiagnosticsRequest) {}

    /// A function that runs upon the client opening a text document.
    fn text_document_opened(&self, server: &LanguageServer, document: TextDocumentItem) {}
    /// A function that run upon the client changing a text document.
    fn text_document_changed(
        &self,
        server: &LanguageServer,
        document: VersionedTextDocumentIdentifier,
        changes: Vec<TextDocumentContentChangeEvent>,
    ) {
    }
    // fn Will Save Text Document(&self, server: &LanguageServer) {}
    // fn Will Save Document Wait Until(&self, server: &LanguageServer) {}
    fn text_document_saved(&self, server: &LanguageServer, document: TextDocumentItem) {}
    fn text_document_closed(&self, server: &LanguageServer, document: TextDocumentItem) {}
    // fn Rename a Text Document(&self, server: &LanguageServer) {}
    // fn Overview - Notebook Document(&self, server: &LanguageServer) {}
    // fn Did Open Notebook Document(&self, server: &LanguageServer) {}
    // fn Did Change Notebook Document(&self, server: &LanguageServer) {}
    // fn Did Save Notebook Document(&self, server: &LanguageServer) {}
    // fn Did Close Notebook Document(&self, server: &LanguageServer) {}
}

pub struct CompletionRequest<'a> {
    id: RequestId,
    server: &'a LanguageServer,
}

impl<'a> CompletionRequest<'a> {
    pub fn respond(self, completions: Vec<CompletionItem>) {
        self.server
            .connection
            .sender
            .send(Message::Response(Response {
                id: self.id,
                result: Some(serde_json::to_value(completions).unwrap()),
                error: None,
            }))
            .unwrap();
    }
}

pub struct DiagnosticsRequest<'a> {
    id: RequestId,
    server: &'a LanguageServer,
    pub text_document: Url,
}

impl<'a> DiagnosticsRequest<'a> {
    pub fn respond(self, diagnostics: Vec<Diagnostic>) {
        self.server
            .connection
            .sender
            .send(Message::Response(Response {
                id: self.id,
                result: Some(
                    serde_json::to_value(DocumentDiagnosticReport::Full(
                        RelatedFullDocumentDiagnosticReport {
                            related_documents: None,
                            full_document_diagnostic_report: FullDocumentDiagnosticReport {
                                result_id: None,
                                items: diagnostics,
                            },
                        },
                    ))
                    .unwrap(),
                ),
                error: None,
            }))
            .unwrap();
    }
}
