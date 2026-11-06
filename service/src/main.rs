crucible_server_runtime_rust::include_service!();
mod operations;
mod store;

use std::sync::Arc;

use crucible_server_runtime_rust::idempotency::{Idempotency, MemoryIdempotencyStore};
use crucible_server_runtime_rust::page_tokens::{PageKeys, PageTokens};
use crucible_server_runtime_rust::{Crucible, JsonProtocol};
use operations::Service;

#[tokio::main]
async fn main() -> Result<(), crucible_server_runtime_rust::host::BoxError> {
    let mut builder = Crucible::builder(my_service::operations(Arc::new(Service::default())))
        .protocol(JsonProtocol)
        // CreateItem's `clientToken`: a retry with the same input replays the
        // first success for 24 hours, and other input is a 409. The memory
        // store is one process's; a deployment keeps claims in a table every
        // instance shares (Crucible's docs/framework.md#idempotency-tokens).
        // This example has no authentication, so every caller shares one
        // binding; with auth, bind to `Principal::caller_binding_of`.
        .interceptor(Idempotency::new(MemoryIdempotencyStore::new(), |_| None).allow_anonymous())
        // ListItems' `nextToken`: sealed, bound to the listing, and expiring
        // after 24 hours, with nothing stored.
        // A cursor is a UUID (36 bytes); bound it so sealed tokens fit the
        // model's 1024-character nextToken limit.
        .interceptor(
            PageTokens::new(page_keys()?, |_| None)
                .max_cursor(64)
                .allow_anonymous(),
        );
    if cfg!(debug_assertions) {
        builder = builder.explorer(crucible_server_runtime_rust::explorer::Explorer::new(
            my_service::DESCRIPTION,
        ));
    }
    let service = builder.build()?;
    let address = std::env::var("CRUCIBLE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    if cfg!(debug_assertions) {
        println!("API explorer: http://{}/explorer/", listener.local_addr()?);
    }
    crucible_server_runtime_rust::host::http_on(service, listener).await
}

/// Page tokens are sealed with `PAGE_TOKEN_SECRET` (32 or more random bytes,
/// such as `openssl rand -hex 32`), which every instance must share. Without
/// it, a random secret is used, and tokens stop opening when the process stops.
fn page_keys() -> Result<PageKeys, crucible_server_runtime_rust::host::BoxError> {
    let secret = match std::env::var("PAGE_TOKEN_SECRET") {
        Ok(secret) => secret.into_bytes(),
        Err(_) => {
            let mut secret = vec![0; 32];
            getrandom::fill(&mut secret)?;
            secret
        }
    };
    Ok(PageKeys::new("k1", &secret)?)
}
