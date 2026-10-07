# Crucible service template

A monorepo template with a shared Smithy model in `smithy/` and a Rust service in `service/`. The example has an `Item` resource, five working API handlers, and an in-memory backend. Run it, then create an item and read, update, list, or delete it from RustRover.

```text
Cargo.toml          # Cargo workspace
Cargo.lock          # Shared dependency lockfile
smithy/
  service.smithy    # API model
service/
  Cargo.toml        # Rust service crate
  build.rs          # Contract generation from ../smithy
  src/              # Handlers and store
  requests.http     # RustRover HTTP examples
  tools/smoke.py    # HTTP smoke check
```

The root workspace selects `service` by default. Cargo commands work from the repository root or from `service/`, sharing the root lockfile and `target/` directory. Add future Rust crates to the root workspace's `members` list.

## Starting in RustRover

You need Rust and GitHub read access to the private [Crucible repository](https://github.com/Operation-Leyline/crucible).

1. Select **Use this template > Create a new repository** on GitHub.
2. Clone your repository and open it in RustRover.
3. Let Cargo finish loading, then run `service/src/main.rs`.
4. Open `service/requests.http` and run the requests in order. The create request saves `itemId` for subsequent requests using a [response handler](https://www.jetbrains.com/help/idea/http-response-handling-examples.html).

The server prints `Listening on http://127.0.0.1:8080`. You can also run `cargo run` in a terminal. Set `CRUCIBLE_ADDR` to change the address.

The first build downloads and verifies Smithy's pinned runtime. Later builds reuse its cache; no separate Smithy CLI or Java installation is needed.

## Example API

| Operation | Request | Success |
|---|---|---|
| CreateItem | `POST /items` | 201, generated item ID and fields |
| GetItem | `GET /items/{itemId}` | 200, item |
| UpdateItem | `PUT /items/{itemId}` | 200, updated item |
| DeleteItem | `DELETE /items/{itemId}` | 204, empty body |
| ListItems | `GET /items?maxResults=20` | 200, items and optional nextToken |

Creation accepts `name`, optional `description`, and `clientToken`. Provide a token when using raw HTTP; generated SDKs can fill it automatically. Repeating an identical create with the same token returns the original result. Changed fields with that token return `ConflictException` (409), including the conflicting item ID. Receipts last eight hours, including after deletion; replaying one does not recreate a deleted item.

Update replaces the editable fields. Omitting `description` clears it. Get and update return `ResourceNotFoundException` (404) for a missing item. Delete succeeds even if the item is already absent.

List returns at most `maxResults` items (default 20, maximum 100), ordered by immutable ID. Pass `nextToken` to continue; page size can change between requests. Tokens are opaque references, expire after 15 minutes, and become invalid on restart. At most 1,024 cursors are retained; issuing another evicts the oldest. Invalid tokens return `ValidationException` (400).

All handlers share one store. Items, create receipts, and cursors disappear when the process stops. The example permits 1,000 items and 10,000 unexpired create receipts; reaching either limit returns `ServiceQuotaExceededException` (402). It has no authentication and is intended for local learning. Replace the store with durable storage before running multiple instances.

## Changing the example

`smithy/service.smithy` defines the resource, HTTP bindings, constraints, pagination, and exceptions. `service/src/operations/item/` has one handler per operation. `service/src/store/` owns state and atomic mutations.

Edit the model and implement the generated traits. `service/build.rs` validates changes and generates the contract under Cargo's `OUT_DIR`; `service/src/contract.rs` includes it. Both the build script and `service/Cargo.toml`'s Smithy editor metadata point to `../smithy`. Generated output stays under the root `target/` directory.

From the repository root, run `cargo fmt --all`, `cargo clippy --all-targets -- -D warnings`, and `cargo test`. With the server running, `python service/tools/smoke.py` checks the API over real HTTP, or pass another endpoint as its first argument.

## Smithy editor checks

Keep your Smithy language plugin and install the **Smithy Cargo** companion ZIP through **Settings > Plugins > Install Plugin from Disk**, then restart RustRover.
It exposes the resolved prelude and Crucible traits through Cargo metadata, without generating editor copies in your project.

The companion currently targets RustRover 2026.2 and Smithy plugin 2.0.19.
Download the [tested preview companion](https://github.com/Operation-Leyline/crucible/actions/runs/37561833516).
Select the `smithy-cargo-plugin` artifact, extract its outer archive, and install `smithy-cargo-0.1.0.zip`.
Matching Crucible releases will include the plugin as a release asset.

## Dependency versions

Both Crucible dependencies pin the same immutable commit containing the shared exception models, merged in [Crucible PR #71](https://github.com/Operation-Leyline/crucible/pull/71). Update both pins together when adopting a release. `Cargo.lock` is committed; the default compiler and prelude are Smithy 1.74.0.
