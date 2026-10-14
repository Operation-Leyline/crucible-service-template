crucible_server_runtime_rust::include_service!();
mod operations;
mod store;

use std::sync::Arc;

use crucible_server_runtime_rust::idempotency::{Idempotency, MemoryIdempotencyStore};
use crucible_server_runtime_rust::page_tokens::{PageKeys, PageTokens};
use crucible_server_runtime_rust::{Crucible, JsonProtocol, Unversioned};
use operations::Service;

#[tokio::main]
async fn main() -> Result<(), crucible_server_runtime_rust::host::BoxError> {
    let service = Crucible::builder(my_service::operations(Arc::new(Service::default())))
        .protocol(JsonProtocol)
        // The generated SDKs send their release in `Crucible-Version`. A
        // caller that sends none, such as curl or `requests.http`, is served
        // the current release. Without this, the framework would serve it
        // the oldest, which suits callers that predate versions.
        .api_version(Unversioned::Resolve(Arc::new(|_| {
            Some(my_service::versions::CURRENT.name().to_owned())
        })))
        // CreateItem's `clientToken`: a retry with the same input replays the
        // first success for 24 hours, and other input is a 409. The memory
        // store is one process's; a deployment keeps claims in a table every
        // instance shares (Crucible's docs/framework.md#idempotency-tokens).
        // This example has no authentication, so every caller shares one
        // binding; with auth, bind to `Principal::caller_binding_of`.
        .interceptor(Idempotency::new(MemoryIdempotencyStore::new(), |_| None).allow_anonymous())
        // ListItems' `nextToken`: sealed, bound to the listing, and expiring
        // after 24 hours, with nothing stored.
        .interceptor(PageTokens::new(page_keys()?, |_| None).allow_anonymous())
        .build()?;
    let address = std::env::var("CRUCIBLE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
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
