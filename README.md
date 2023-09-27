# LSPeasy

**Language servers made simple, lightweight, and easy**

LSPeasy allows you to easily create a `stdio` language server with it's simple handler traits.

Note that, for simplicity, LSPeasy can only use the `stdio` communication method.

**LSPeasy is currently in heavy development and not intended to be used in production, for this reason the crate is not available via crates.io**

# Example

First of all, import the `lspeasy` & `lsp-types` crates:

```toml
lspeasy = { git = "https://github.com/doonv/LSPeasy.git" }
lsp-types = "0.94.1"
```

Then, let's create a basic handler, that says hello to the client via the `LanguageServer`.

```rs
struct MyHandler;
impl LanguageServerHandler for MyHandler {
    fn init(&self, server: &LanguageServer) {
        server.log(format!("Hello Client!"), MessageType::INFO);
    }
}
```

Now. In our main function, we will define our server's capabilities. This will tell the client what our server can do, and then act accordingly. Our server can't do much, so we will just go with the default configuration.

```rs
let capabilities = ServerCapabilities {
    .ServerCapabilities::default()
};
```

Lastly, we startup our server and pass our capabilities and handler.

```rs
let _server = LanguageServer::new(&capabilities, MyHandler).unwrap();
```

You can find the full example [here](./examples/simple.rs)