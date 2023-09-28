use lsp_types::{CompletionItem, CompletionOptions, Position};
use lspeasy::lsp_types::{MessageType, ServerCapabilities};
use lspeasy::{LanguageServer, LanguageServerHandler};

// Create the handler class, which houses
// all the event handlers.
struct MyHandler;
impl LanguageServerHandler for MyHandler {
    fn init(&self, server: &LanguageServer) {
        server.log(format!("Server started! :)"), MessageType::INFO);
    }

    fn completion(&self, _server: &LanguageServer, req: lspeasy::CompletionRequest) {
        let Position { line, character } = req.position;

        req.respond(vec![CompletionItem {
            label: format!("char{character}line{line}"),
            ..CompletionItem::default()
        }]);
    }
}

fn main() {
    // Define our server's capabilities
    let capabilities = ServerCapabilities {
        completion_provider: Some(CompletionOptions::default()),
        ..ServerCapabilities::default()
    };

    // Start up the server
    let _server = LanguageServer::new(&capabilities, MyHandler).unwrap();
}
