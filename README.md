# Crucible service template

A single Rust crate with an empty Smithy API and an HTTP server. Add your model and operation handlers here.

## Starting in RustRover

You need Rust and GitHub read access to the private [Crucible repository](https://github.com/Operation-Leyline/crucible).

1. Select **Use this template > Create a new repository** on GitHub.
2. Clone your new repository and open it in RustRover.
3. Let Cargo finish loading, then run `src/main.rs`.

The server prints `Listening on http://127.0.0.1:8080`. Open `requests.http` to check it: HTTP 404 with `UnknownOperation` is expected before you add operations. You can also run `cargo run` in a terminal.
The first build downloads and verifies Smithy's pinned runtime. Later builds reuse its cache; no separate Smithy CLI or Java installation is needed.

## Adding your API

`smithy/service.smithy` declares `example.service#MyService` with no operations.
The standard validation and internal errors let the empty service start and define the errors your future operations inherit.
With no operations, requests return `UnknownOperation`.

Edit the namespace and service name, define your operations, and add them to the service's `operations` list.
Implement the generated operation traits on `Service`. Put service-bound handlers in `src/operations/service/`, one file per operation, and declare their modules in `src/operations/mod.rs`.

`build.rs` validates the model and generates the contract on each model change. `src/contract.rs` includes it from Cargo's `OUT_DIR`.
The Rust compiler identifies operation traits or fields you need to implement. Generated output stays under `target`; edit the Smithy model and handlers instead.

You can rename the package in `Cargo.toml` to match your repository. `CRUCIBLE_ADDR` changes the listening address.

## Smithy editor checks

Keep your Smithy language plugin and install the **Smithy Cargo** companion ZIP through **Settings > Plugins > Install Plugin from Disk**, then restart RustRover.
It exposes the resolved prelude and Crucible traits through Cargo metadata, without generating editor copies in your project.

The companion currently targets RustRover 2026.2 and Smithy plugin 2.0.19.
While [Crucible PR #69](https://github.com/Operation-Leyline/crucible/pull/69) awaits review, download its [tested preview companion](https://github.com/Operation-Leyline/crucible/actions/runs/37561833516).
Select the `smithy-cargo-plugin` artifact, extract its outer archive, and install `smithy-cargo-0.1.0.zip`.
Matching Crucible releases will include the plugin as a release asset.

## Dependency versions

Both Crucible dependencies pin the same commit from PR #69, which supplies the new `from_smithy` helper.
Crucible 0.3.0 is not released yet. After release, update both dependencies to the same `v0.3.0` tag deliberately.
The default compiler and prelude are Smithy 1.74.0; `Cargo.lock` is committed.
See [Crucible's quick start](https://github.com/Operation-Leyline/crucible/blob/20af40c503f2a34f5babc870dcb8cb7c59575a5d/docs/quickstart.md) for selecting another Smithy version and offline builds.
