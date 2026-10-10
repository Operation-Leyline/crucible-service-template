crucible_server_runtime_rust::include_service!();
mod operations;
mod store;

use std::sync::Arc;

use crucible_server_runtime_rust::{Crucible, JsonProtocol, Unversioned};
use operations::Service;

#[tokio::main]
async fn main() -> Result<(), crucible_server_runtime_rust::host::BoxError> {
    let service = Crucible::builder(my_service::operations(Arc::new(Service::default())))
        .protocol(JsonProtocol)
        // A request without `Crucible-Version` is served at the current
        // release: the generated SDKs do not send their release yet. Until
        // they do, only callers that send the header reach release 1.
        .api_version(Unversioned::Resolve(Arc::new(|_| {
            Some(my_service::versions::CURRENT.name().to_owned())
        })))
        .build()?;
    let address = std::env::var("CRUCIBLE_ADDR").unwrap_or_else(|_| "127.0.0.1:8080".into());
    let listener = tokio::net::TcpListener::bind(address).await?;
    println!("Listening on http://{}", listener.local_addr()?);
    crucible_server_runtime_rust::host::http_on(service, listener).await
}
