# Crucible service template

A starting point for an API built with [Crucible](https://github.com/Operation-Leyline/crucible): a model in Crucible's modelling language (`.cru`) and a Rust service that implements it. The `Item` resource has five working handlers and an in-memory store to replace with your own.

For a fuller project with frozen API releases, migrations, and generated Rust and Java SDKs with their own consumer apps, see the [Crucible service example](https://github.com/Operation-Leyline/crucible-service-example).

```text
Cargo.toml                 # Cargo workspace and the pinned Crucible version
model/
  api/crucible.toml        # Package manifest: namespace, imports, API policy
  api/service.cru          # The API
  build.rs                 # Compile and publish the model; export OpenAPI
service/
  build.rs                 # Generate server bindings from the model
  src/main.rs              # Host the service over HTTP
  src/operations/item/     # One handler per operation
  src/store/               # In-memory state
  requests.http            # Requests to run from your IDE
  tools/smoke.py           # HTTP smoke test
.github/workflows/ci.yml   # fmt, clippy, tests and the smoke test
```

## Getting started

You need a Rust toolchain with Edition 2024 (1.85 or newer). Cargo fetches Crucible; there is no CLI, JVM or other tool to install.

1. Select **Use this template > Create a new repository** on GitHub, or run `gh repo create my-api --template Operation-Leyline/crucible-service-template --clone`.
2. Open the repository in your IDE and let Cargo finish loading, or run `cargo run` from the repository root.
3. The server prints `Listening on http://127.0.0.1:8080`. Send the requests in `service/requests.http` in order; the create request saves `itemId` for the requests after it.

Set `CRUCIBLE_ADDR` to change the address, and `PAGE_TOKEN_SECRET` (32 or more random bytes, such as `openssl rand -hex 32`) to keep page tokens valid across restarts and instances.

## The API

| Operation | Request | Success |
|---|---|---|
| CreateItem | `POST /items` | 201, generated item ID and fields |
| GetItem | `GET /items/{itemId}` | 200, item |
| UpdateItem | `PUT /items/{itemId}` | 200, updated item |
| DeleteItem | `DELETE /items/{itemId}` | 204, empty body |
| ListItems | `GET /items?maxResults=20` | 200, items and optional nextToken |

- **Idempotent creates.** `clientToken` is enforced by the framework's `Idempotency` interceptor (in `service/src/main.rs`). A retry with the same token and fields replays the original result for 24 hours; the same token with different fields returns `ConflictException` (409). Generated SDKs fill the token automatically.
- **Sealed pagination.** The `PageTokens` interceptor seals each `nextToken` around the last item ID shown. Clients cannot read or edit it, it opens only for the same listing, and it expires after 24 hours.
- **Modeled errors.** Invalid input is a `ValidationException` (400), a missing item a `ResourceNotFoundException` (404), and the store's 1,000-item limit a `ServiceQuotaExceededException` (402).

There is no authentication, so both interceptors bind tokens to one anonymous caller (`allow_anonymous`). With authentication, add `crucible-auth` as a git dependency at the same Crucible commit and pass `crucible_auth::Principal::caller_binding_of` instead.

## Making it yours

1. **Name it.** In `model/api/service.cru`, rename `MyService`; the generated Rust module follows it (`MyService` → `my_service`), so update `my_service::` in `service/src`. Set `namespace` in `model/api/crucible.toml`, and the service's package name in `service/Cargo.toml`. To rename the model package, change `name` and `links` in `model/Cargo.toml`, the `service-model` dependency in `service/Cargo.toml`, and `model("service_model")` in `service/build.rs` together.
2. **Model your API.** Replace `Item` in `service.cru`. Members are required unless written `name?:`, operations attach to a resource with role verbs such as `creates` and `reads`, and URI labels bind input members by name. A model error fails the build with every diagnostic, each pointing at its file, line and column.
3. **Implement it.** Each operation is a generated trait; add one file per operation under `service/src/operations/<resource>/`. The compiler tells you which handlers are missing or out of date.
4. **Store it.** Replace `service/src/store/` with durable storage, and `MemoryIdempotencyStore` with an `IdempotencyStore` over a table every instance shares, before running more than one instance.

Check everything from the repository root:

```sh
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python service/tools/smoke.py   # with the server running
```

## Generating clients

`model/build.rs` compiles the model once, publishes it to the service, and writes an OpenAPI 3.1 document to `target/openapi/openapi.json`. Add a native SDK there; for Rust:

```rust
model
    .rust_client("rust")
    .package("my-service-sdk")
    .version("0.1.0")
    .generate_into("../target/rust-sdk");
```

Other languages use the matching helper (`java_client`, `swift_client`, and so on) with that language's feature enabled on `crucible-build` in the root `Cargo.toml`. The [service example](https://github.com/Operation-Leyline/crucible-service-example) generates Rust and Java SDKs and consumes them from independent Rust and Kotlin apps. See Crucible's [build outputs guide](https://github.com/Operation-Leyline/crucible/blob/77df3aa52e68ac65042cbe1b7cb0ff2ff25ab7d5/docs/build-outputs.md).

## API policy and releases

`model/api/crucible.toml` holds the API policy: naming and path conventions that fail the build, and documentation requirements as warnings. Edit the rules for your service; see the [policy reference](https://github.com/Operation-Leyline/crucible/blob/77df3aa52e68ac65042cbe1b7cb0ff2ff25ab7d5/docs/api-policy.md).

Once clients depend on your API, freeze each release before changing it. Install the `crucible` CLI at the same commit the workspace pins:

```sh
cargo install --git https://github.com/Operation-Leyline/crucible --rev 77df3aa52e68ac65042cbe1b7cb0ff2ff25ab7d5 crucible-cli --locked
```

1. Run `crucible freeze model/api`. It writes `model/api/releases/<version>.json` and records it in `releases/crucible.lock`; commit both.
2. Add `[model.baseline]` with `path = "releases/<version>.json"` to `model/api/crucible.toml`.
3. Bump `version` in `service.cru`, and declare the change in a migration, such as `migration "2" from "1" { }` (every new version needs one, even with no breaking change).

Every build then refuses a breaking change that no `migration` declares, and the service keeps serving older clients from the same handlers. Requests without a `Crucible-Version` header, such as curl, `requests.http` and `smoke.py`, are then served the oldest release; set `.api_version(Unversioned::Resolve(..))` in `service/src/main.rs` to serve them the current one instead. The [service example](https://github.com/Operation-Leyline/crucible-service-example) shows a second release with its migration; Crucible's [API versions guide](https://github.com/Operation-Leyline/crucible/blob/77df3aa52e68ac65042cbe1b7cb0ff2ff25ab7d5/docs/api-versions.md) covers the details.

## Editor support

RustRover and IntelliJ IDEA support `.cru` files through the [Crucible plugin](https://github.com/Operation-Leyline/crucible-intellij), which bundles the `crucible-lsp` language server: diagnostics as you type, completion, go to declaration, rename, and formatting. Other editors can run `crucible-lsp` from the [Crucible releases](https://github.com/Operation-Leyline/crucible/releases).

## Dependency versions

`crucible-server-runtime-rust` and `crucible-build` in the root `Cargo.toml` pin the same Crucible commit; update them together. Commit `Cargo.lock`.

To build against a local Crucible checkout while changing Crucible itself, point both dependencies at it with `path = "<checkout>/crates/..."`.
