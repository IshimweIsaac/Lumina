mod backend;
mod diag;
mod hover;
mod semantic;
mod refs;
mod action;
mod inlay;
mod symbols;
use backend::LuminaBackend;
use tower_lsp::{LspService, Server};

#[tokio::main]
async fn main() {
    let (service, socket) = LspService::new(|client| LuminaBackend::new(client));
    Server::new(tokio::io::stdin(), tokio::io::stdout(), socket)
        .serve(service)
        .await;
}
