# Crucible service template

A monorepo template with a language-neutral Smithy model, generated Rust and Java SDKs, a Rust service, and independent client examples. The `Item` resource has five working API handlers and an in-memory backend.

```text
Cargo.toml                # Cargo workspace and shared Crucible version
Cargo.lock                # Shared dependency lockfile
crucible.toml             # Build-time API rules and compatibility policy
.github/workflows/
  api-diff.yml            # PR contract compatibility checks
smithy/
  Cargo.toml              # Language-neutral model package
  model/service.smithy    # API sources, separate from Rust code
  build.rs                # Compile once; publish model, Rust SDK, Java SDK
  src/lib.rs              # Empty Cargo package scaffold
sdks/rust/
  Cargo.toml              # Consumable Rust SDK package
  build.rs                # Connect the published Rust artifact to Cargo
  src/lib.rs              # Managed generated-code inclusion
service/
  build.rs                # Generate server bindings from the published model
  src/contract.rs         # Include generated models, traits, and registry
  src/                    # Handwritten handlers and store
  requests.http           # RustRover HTTP examples
  tools/smoke.py          # HTTP smoke check
client/                   # Normal Rust application consuming sdks/rust
examples/java/            # Normal Maven application consuming the Java JAR
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

## Running the Rust client

Start the service with `cargo run` from the repository root. In a second terminal at the repository root, run:

```sh
cargo run -p crucible-service-client
```

The example creates an item, reads it, updates it, lists items through the generated paginator, and deletes its item. It prints each result and also attempts deletion if a later operation fails. The generated SDK fills the create operation's idempotency token automatically.

The default endpoint is `http://127.0.0.1:8080`. Pass another endpoint as the first argument:

```sh
cargo run -p crucible-service-client -- http://127.0.0.1:9000
```

You can also run `cargo run` from `client/` or run `client/src/main.rs` in RustRover. The example is a normal Rust application with no build script or generation setup. It depends on `service-sdk-rust` in `sdks/rust/` and imports the SDK with:

```rust
use service_sdk_rust::{Config, MyServiceClient, model};
```

Other Rust workspace members can use the same SDK by adding a path dependency on the SDK package:

```toml
[dependencies]
service-sdk-rust = { path = "../sdks/rust" }
```

The `service-model` package compiles `smithy/model/` once and publishes its `model.json` and named SDK artifacts through Cargo build metadata. `sdks/rust/` exposes the Rust artifact as a normal Cargo library. The service generates its own server bindings from the same model, directly in its local `contract` module. Editing the model regenerates both SDKs on the next build. Commit the Smithy sources, generation recipes, Rust SDK wrapper, and examples. Generated Rust and Java sources stay under ignored `target/` directories and are not committed.

## Generating server bindings in the service crate

The service depends directly on `service-model` and has `crucible-build` as a build dependency. Its complete build script is:

```rust
// service/build.rs
fn main() {
    crucible_build::service("service_model")
        .policy("../crucible.toml")
        .generate();
}
```

`service/src/contract.rs` contains `crucible::include_service!();`, and `main.rs` declares `mod contract;`. This generates request/response models, modeled errors, operation traits, and operation registration inside the service crate. Handlers import `crate::contract` as usual; no separate server-contract crate is needed. The helper reads the model's published AST without compiling Smithy again.

A server that owns its sources can instead call `crucible_build::from_smithy("smithy").generate()` with the same inclusion macro. That direct setup does not require a model Cargo package. A separate contract crate is optional if multiple Rust crates need the generated server API.

## Generating and consuming the Java SDK

The shared `crucible-build` dependency enables its `java` feature. `smithy/build.rs` declares both named outputs:

```rust
fn main() {
    let model = crucible_build::smithy("model")
        .policy("../crucible.toml")
        .compile();
    model.publish();
    model.rust_client("rust").generate();
    model
        .java_client("java")
        .package("example.myservice")
        .group_id("example")
        .artifact_id("myservice-sdk")
        .version("0.1.0")
        .generate();
}
```

A normal Cargo build produces both SDK source artifacts. To export the Java artifact as a Maven project, install the matching Crucible CLI:

```sh
cargo install --git https://github.com/Operation-Leyline/crucible --rev da6ef22423c1b086b09449c83e290de7c3824218 --locked crucible-cli
crucible export-sdk java --manifest-path Cargo.toml --package service-model --out target/java-sdk
```

The export command builds the selected model package and resolves its declared artifact through Cargo metadata. It does not search build directories or compile the Smithy model again when Cargo has a current result.

With JDK 17 or newer and Maven installed, build and install the Java package into your local Maven repository:

```sh
mvn -f target/java-sdk/pom.xml install
```

Start the service, then run the independent Java example:

```sh
mvn -f examples/java/pom.xml compile exec:exec
```

The Java example uses ordinary Maven coordinates `example:myservice-sdk:0.1.0`. It creates, reads, updates, paginates, and deletes an item. It has no Cargo dependency or code generation step. Pass another endpoint with `"-Dservice.endpoint=http://127.0.0.1:9000"`. For distribution, publish the SDK JAR to your Maven repository and let applications declare that dependency.

Give additional variants distinct artifact names, Maven coordinates, and Java packages, for example `model.java_client("java-internal")`. A separate SDK producer module can instead call `crucible_build::model_dependency("service_model")` and generate its own named SDKs from the published model. Its Cargo manifest must depend directly on `service-model` so Cargo supplies that model's metadata.

With the local service running, verify the example's pagination and cleanup with:

```sh
cargo test -p crucible-service-client --test example -- --ignored
```

Set `CRUCIBLE_ENDPOINT` to test another endpoint. This test creates two temporary items, runs the example, checks that it preserves existing items, and deletes the test items. It is skipped by default because it requires a running service.

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

`smithy/model/service.smithy` defines the resource, HTTP bindings, constraints, pagination, and exceptions. `service/src/operations/item/` has one handler per operation. `service/src/store/` owns state and atomic mutations.

Edit the model and implement the generated traits. `smithy/build.rs` compiles and validates the model, then declares the Rust and Java SDK outputs. `sdks/rust/` connects the published Rust artifact to its library; `service/build.rs` generates server bindings for `service/src/contract.rs`. Smithy editor metadata identifies `smithy/model/` as the source directory; generated output stays under the root `target/` directory.

From the repository root, run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` to check both the service and client. With the server running, `python service/tools/smoke.py` checks the API over real HTTP, or pass another endpoint as its first argument.

## API policy and compatibility

`crucible.toml` selects naming and path conventions enforced by every Cargo build. Documentation requirements start as warnings. The model and service build scripts select this required file with `.policy("../crucible.toml")`; invalid rules, missing config, and error findings fail generation. Edit the rules for your service. See the [policy reference](https://github.com/Operation-Leyline/crucible/blob/a36ddee1e052cd61e3e5c222fe7e31bea07e4ede/docs/api-policy.md) for severities and exact, documented exceptions.

The included `.github/workflows/api-diff.yml` passes the pull request's baseline `smithy/` and proposed `smithy/model/` directories and proposed policy to the private shared action. The checker assembles both directories with its pinned Smithy compiler, imports Crucible traits, flattens mixins, and compares the APIs. Findings appear in the job summary and `api-diff-report` artifact; breaking changes fail the job.

After creating a repository from this template, configure variable `CRUCIBLE_APP_ID` and secret `CRUCIBLE_APP_PRIVATE_KEY` for the existing `crucible-ci-read` GitHub App. Its Operation-Leyline installation needs Contents: read access to `crucible` and `crucible-actions`. The workflow uses that installation even when your service belongs to another organization. GitHub does not copy template repository secrets into new repositories.

The compatibility job reads Smithy sources and does not build either Rust service. The baseline needs its `smithy/` directory, even if it uses an older build helper. Missing directories or invalid Smithy fail the job. Fork PRs do not receive repository secrets. Keep `Cargo.lock` committed for ordinary service builds.

## Smithy editor checks

Keep your Smithy language plugin and install the **Smithy Cargo** companion ZIP through **Settings > Plugins > Install Plugin from Disk**, then restart RustRover.
It exposes the resolved prelude and Crucible traits through Cargo metadata, without generating editor copies in your project.

The companion currently targets RustRover 2026.2 and Smithy plugin 2.0.19.
Download the [tested preview companion](https://github.com/Operation-Leyline/crucible/actions/runs/37561833516).
Select the `smithy-cargo-plugin` artifact, extract its outer archive, and install `smithy-cargo-0.1.0.zip`.
Matching Crucible releases will include the plugin as a release asset.

## Dependency versions

The framework, build helper, and client runtime pin the same immutable commit in the root `Cargo.toml`, including API policies and named SDK generation helpers. Update these pins together when adopting a release. The workflow independently pins the checker with Smithy-directory support and the shared actions. `Cargo.lock` is committed; the build helper supplies the pinned Smithy compiler and prelude.
