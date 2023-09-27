use lsp_types::{MessageType, ServerCapabilities};
use lspeasy::{LanguageServer, LanguageServerHandler};

// Create the handler class, which houses
// all the event handlers.
struct MyHandler;
impl LanguageServerHandler for MyHandler {
    fn init(&self, server: &LanguageServer) {
        server.log(format!("Server started! :)"), MessageType::INFO);
    }
}

fn main() {
    // Define our server's capabilities
    let capabilities = ServerCapabilities {
        ..ServerCapabilities::default()
    };

    // Start up the server
    let _server = LanguageServer::new(&capabilities, MyHandler).unwrap();
}
