# Crucible service template

A single Rust crate with an empty Smithy API and an HTTP server. Add your model and operation handlers here.

## Starting in RustRover

You need Rust and GitHub read access to the private [Crucible repository](https://github.com/Operation-Leyline/crucible).

1. Select **Use this template > Create a new repository** on GitHub.
2. Clone your new repository and open it in RustRover.
3. Let Cargo finish loading, then run `src/main.rs`.

The server prints `Listening on http://127.0.0.1:8080`. Open `requests.http` to check it: HTTP 404 with `UnknownOperationException` is expected before you add operations. You can also run `cargo run` in a terminal.
The first build downloads and verifies Smithy's pinned runtime. Later builds reuse its cache; no separate Smithy CLI or Java installation is needed.

## Adding your API

`smithy/service.smithy` declares `example.service#MyService` with no operations.
The standard validation, internal and unknown-operation exceptions let the empty service start and define the errors your future operations inherit.
With no operations, requests return the modelled `UnknownOperationException` (404). Invalid input for a declared operation returns `ValidationException` (400), including a structured reason and field details. Unexpected failures use `InternalServerException` (500). Error responses carry `x-request-id`.

Edit the namespace and service name, define your operations, and add them to the service's `operations` list.
Implement the generated operation traits on `Service`. Put service-bound handlers in `src/operations/service/`, one file per operation, and declare their modules in `src/operations/mod.rs`.

`build.rs` validates the model and generates the contract on each model change. `src/contract.rs` includes it from Cargo's `OUT_DIR`.
The Rust compiler identifies operation traits or fields you need to implement. Generated output stays under `target`; edit the Smithy model and handlers instead.

You can rename the package in `Cargo.toml` to match your repository. `CRUCIBLE_ADDR` changes the listening address.

## Smithy editor checks

Keep your Smithy language plugin and install the **Smithy Cargo** companion ZIP through **Settings > Plugins > Install Plugin from Disk**, then restart RustRover.
It exposes the resolved prelude and Crucible traits through Cargo metadata, without generating editor copies in your project.

The companion currently targets RustRover 2026.2 and Smithy plugin 2.0.19.
Download the [tested preview companion](https://github.com/Operation-Leyline/crucible/actions/runs/37561833516).
Select the `smithy-cargo-plugin` artifact, extract its outer archive, and install `smithy-cargo-0.1.0.zip`.
Matching Crucible releases will include the plugin as a release asset.

## Dependency versions

Both Crucible dependencies pin the same tested commit from [Crucible PR #71](https://github.com/Operation-Leyline/crucible/pull/71), which supplies the shared exception models and unmatched-route mapping.
Crucible 0.3.0 is not released yet. After a release containing these exceptions, update both dependencies to its matching version tag deliberately.
The default compiler and prelude are Smithy 1.74.0; `Cargo.lock` is committed.
See [Crucible's quick start](https://github.com/Operation-Leyline/crucible/blob/da59322f562072df6597bdfd5eabb25aa26917ea/docs/quickstart.md) for selecting another Smithy version and offline builds.
