use std::fs;

use lspeasy::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionOptions, Diagnostic, DiagnosticOptions,
    DiagnosticServerCapabilities, DiagnosticSeverity, MessageType, Position, Range,
    ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind, TextDocumentSyncOptions,
};
use lspeasy::{CompletionRequest, DiagnosticsRequest, LanguageServer, LanguageServerHandler};

fn find_capital_ranges(input: &str) -> Vec<Range> {
    let mut ranges = Vec::new();
    let mut start_position: Option<Position> = None;

    for (line_idx, line) in input.lines().enumerate() {
        for (i, c) in line.char_indices() {
            if c.is_ascii_uppercase() {
                if start_position.is_none() {
                    // This is the start of a new capital letter range
                    start_position = Some(Position {
                        line: line_idx as u32,
                        character: i as u32,
                    });
                }
            } else if let Some(start) = start_position.take() {
                // This is the end of the current capital letter range
                let end_position = Position {
                    line: line_idx as u32,
                    character: i as u32,
                };
                ranges.push(Range {
                    start,
                    end: end_position,
                });
            }
        }
    }

    ranges
}

fn get_diagnostics(server: &LanguageServer, text: &str) -> Vec<Diagnostic> {
    // Read the text document
    let ranges = find_capital_ranges(text);

    server.log(format!("r {ranges:?}"), MessageType::INFO);

    let diagnostics = ranges
        .into_iter()
        .map(|range| Diagnostic {
            range,
            severity: Some(DiagnosticSeverity::WARNING),
            code: None,
            code_description: None,
            source: None,
            message: "Capital Letters detected".to_string(),
            related_information: None,
            tags: None,
            data: None,
        })
        .collect();

    diagnostics
}

struct MyHandler;
impl LanguageServerHandler for MyHandler {
    fn init(&self, server: &LanguageServer) {
        server.log(format!("Hello thereeeee :)"), MessageType::INFO);
    }

    fn completion(&self, _server: &LanguageServer, req: CompletionRequest) {
        req.respond(vec![CompletionItem {
            label: "the".to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..CompletionItem::default()
        }])
    }

    fn diagnostics(&self, server: &LanguageServer, req: DiagnosticsRequest) {
        let diagnostics = get_diagnostics(
            server,
            &fs::read_to_string(req.text_document.to_file_path().unwrap()).unwrap(),
        );
        req.respond(diagnostics);
    }

    fn text_document_opened(
        &self,
        _server: &LanguageServer,
        _document: lsp_types::TextDocumentItem,
    ) {
        // server.log(format!("Somethin opened!"), MessageType::INFO);
    }
    fn text_document_changed(
        &self,
        server: &LanguageServer,
        document: lsp_types::VersionedTextDocumentIdentifier,
        changes: Vec<lsp_types::TextDocumentContentChangeEvent>,
    ) {
        server.log(
            format!("Somethin changed! document: {document:#?}, chages: {changes:?}"),
            MessageType::INFO,
        );
        let diagnostics = get_diagnostics(server, &changes[0].text);
        server.send_diagnostics(document.uri, diagnostics);
    }
}

fn main() {
    let capabilities = ServerCapabilities {
        completion_provider: Some(CompletionOptions::default()),
        diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
            DiagnosticOptions::default(),
        )),
        text_document_sync: Some(TextDocumentSyncCapability::Options(
            TextDocumentSyncOptions {
                open_close: Some(false),
                change: Some(TextDocumentSyncKind::FULL),
                will_save: Some(false),
                will_save_wait_until: Some(false),
                save: None,
            },
        )),
        ..ServerCapabilities::default()
    };

    let _server = LanguageServer::new(&capabilities, MyHandler).unwrap();
}
