# Crucible service template

A monorepo template with a language-neutral model in Crucible's modelling language (`.cru`), generated Rust and Java SDKs, a Rust service, and independent client examples. The `Item` resource has five working API handlers and an in-memory backend.

```text
Cargo.toml                # Cargo workspace and shared Crucible version
Cargo.lock                # Shared dependency lockfile
.github/workflows/
  api-diff.yml            # PR contract compatibility checks
  sdk.yml                 # Clean builds and both native consumers over HTTP
model/
  Cargo.toml              # Language-neutral model package
  api/crucible.toml       # Package manifest: namespace, imports, API policy, baseline
  api/service.cru         # API sources, separate from Rust code
  api/migrations.cru      # How release 2 still serves release 1's clients
  api/releases/1/         # Release 1, frozen: the compatibility baseline
  build.rs                # Compile once; publish model, Rust SDK, Java SDK
  src/lib.rs              # Empty Cargo package scaffold
service/
  build.rs                # Generate server bindings from the published model
  src/                    # Handwritten handlers and store
  requests.http           # RustRover HTTP examples
  tools/smoke.py          # HTTP smoke check
client/                   # Independent Cargo app consuming target/rust-sdk
examples/kotlin/          # Independent Kotlin Gradle app consuming target/java-sdk
```

After the first root build, attach `client/Cargo.toml` as another Cargo project in RustRover to edit and run the independent Rust application.

The root workspace selects `service` by default. Cargo commands work from the repository root or from `service/`, sharing the root lockfile and `target/` directory. Add future Rust crates to the root workspace's `members` list.

## Starting in RustRover

You need Rust and GitHub read access to the private [Crucible repository](https://github.com/Operation-Leyline/crucible).

1. Select **Use this template > Create a new repository** on GitHub.
2. Clone your repository and open it in RustRover.
3. Let Cargo finish loading, then run `service/src/main.rs`.
4. Open `service/requests.http` and run the requests in order. The create request saves `itemId` for subsequent requests using a [response handler](https://www.jetbrains.com/help/idea/http-response-handling-examples.html).

The server prints `Listening on http://127.0.0.1:8080`. You can also run `cargo run` in a terminal. Set `CRUCIBLE_ADDR` to change the address.

The model compiles in Rust inside the build script. No Smithy CLI, JVM, or runtime download is involved.

## Running the Rust client

Start the service with `cargo run` from the repository root. In a second terminal at the repository root, run:

```sh
cargo run --manifest-path client/Cargo.toml
```

The example creates an item, reads it, updates it, lists items through the generated paginator, and deletes its item. It prints each result and also attempts deletion if a later operation fails. The generated SDK fills the create operation's idempotency token automatically.

The default endpoint is `http://127.0.0.1:8080`. Pass another endpoint as the first argument:

```sh
cargo run --manifest-path client/Cargo.toml -- http://127.0.0.1:9000
```

You can also run `cargo run` from `client/` or run `client/src/main.rs` in RustRover. The example is a normal Rust application with no build script or generation setup. It depends on the complete generated crate in `target/rust-sdk/` and imports the SDK with:

```rust
use service_sdk_rust::{Config, CreateItemInput, MyServiceClient};
```

Input builders validate required members, so `build()` returns a `Result`.

Other Rust applications can use the same SDK by adding a path dependency on the SDK package:

```toml
[dependencies]
service-sdk-rust = { path = "../target/rust-sdk" }
```

The `service-model` package compiles `model/api/` once and publishes the package directory and named SDK artifacts through Cargo build metadata. The Rust artifact includes its Cargo manifest and runtime dependency; applications consume that crate normally. The service generates its own server bindings from the same model, directly in its generated `my_service` namespace. Editing the model regenerates both SDKs on the next build. Commit the model sources, generation recipes, and examples. Generated Rust and Java sources stay under ignored `target/` directories and are not committed.

## Generating server bindings in the service crate

The service depends directly on `service-model` and has `crucible-build` as a build dependency. Its complete build script is:

```rust
// service/build.rs
fn main() {
    crucible_build::model("service_model").server().generate();
}
```

`service/src/main.rs` calls `crucible_server_runtime_rust::include_service!();` at crate root. For the model's `MyService`, this declares `pub mod my_service`. This generates request/response models, modeled errors, operation traits, and operation registration inside the service crate. Handlers import `crate::my_service::{GetItemInput, GetItemOutput, GetItemOperation, ResourceNotFoundException}` directly; no separate server-contract crate is needed. The helper compiles the published `.cru` package directory; it is fast and needs no external tools.

A server that owns its sources can instead call `crucible_build::model("api").server().generate()` on a local `.cru` package with the same inclusion macro. That direct setup does not require a model Cargo package. A separate contract crate is optional if multiple Rust crates need the generated server API.

## Generating both SDK packages

The root workspace contains only the model and service. Run `cargo build` once before opening either independent client project. The model build creates complete SDK packages at `target/rust-sdk/` and `target/java-sdk/`; generated files stay ignored. Cargo loads dependency manifests before running build scripts, so the Rust client deliberately has its own workspace and lockfile.

The shared `crucible-build` dependency enables its `java` feature. The complete model build script is:

```rust
fn main() {
    let model = crucible_build::model("api").compile();
    model.publish();
    model
        .rust_client("rust")
        .package("service-sdk-rust")
        .version("0.1.0")
        .generate_into("../target/rust-sdk");
    model
        .java_client("java")
        .package("example.myservice")
        .group_id("example")
        .artifact_id("myservice-sdk")
        .version("0.1.0")
        .generate_into("../target/java-sdk");
}
```

Crucible compiles the `.cru` package once, publishes its directory, and generates both native packages. `generate_into` also exports them to the declared paths and tracks their generated files: deleting an SDK export restores it on the next root Cargo build. Changes to the API regenerate both SDKs. A second unchanged build preserves generated file timestamps.

Give additional variants distinct artifact names, package coordinates, and output directories. A separate producer module can instead call `crucible_build::model_dependency("service_model")` and generate its SDK from the published model. Its manifest must directly depend on `service-model` so Cargo supplies the model metadata.

Other client languages use the same fluent setup. For Swift, add `swift` to the shared `crucible-build` dependency's features in `Cargo.toml`, then add this call to `model/build.rs`:

```rust
model.swift_client("swift")
    .package("MyServiceSDK")
    .version("0.1.0")
    .generate_into("../target/swift-sdk");
```

All 14 client languages have matching helpers. OpenAPI, CDK, Terraform/OpenTofu, and service CLI outputs use `model.openapi(...)`, `model.cdk(...)`, `model.terraform(...)`, and `model.cli(...)` with their matching features. CDK and Terraform also require explicit provisioning bindings. Documentation, provisioning (CDK, Terraform) and the API Explorer still read Smithy sources, so they report an error for a `.cru` model for now.

A CDK output can configure Python, Java, .NET, and Go packages together using `.python(...)`, `.java(...)`, `.dotnet(...)`, and `.go(...)`. These are jsii targets within one construct library; `npm run build` and `npm run package` build its language packages. See [Crucible's generation guide](https://github.com/Operation-Leyline/crucible/blob/f7544a8b4802ec2409ce1c64eeba9527fa55992d/docs/build-outputs.md) for the complete feature list, output settings, and examples.

## Running the Kotlin client with Gradle

The independent Kotlin application consumes the generated Java SDK through Gradle's composite build support:

```kotlin
// examples/kotlin/settings.gradle.kts
includeBuild("../../target/java-sdk")
```

It applies the Kotlin JVM plugin and declares the ordinary dependency `implementation("example:myservice-sdk:0.1.0")`. Gradle substitutes the generated library project and builds its JAR. Kotlin calls the SDK directly, using its `Consumer<Builder>` overloads as trailing lambdas such as `client.getItem { it.itemId(itemId) }`. Neither the Kotlin application's source nor its Gradle build contains generation or runtime plumbing.

You need JDK 17 or newer; the generated SDK and example target Java 17. The official Gradle 9.8.0 wrapper is checked in with the distribution SHA-256 pinned, and Gradle downloads the Kotlin plugin, so no separate Gradle or Kotlin installation is needed. Start the service, then run from the repository root:

```sh
./examples/kotlin/gradlew -p examples/kotlin run
```

On Windows, use `examples\kotlin\gradlew.bat -p examples/kotlin run`. Pass another endpoint with `"--args=http://127.0.0.1:9000"`. The example creates, reads, updates, paginates, and deletes its item. Cleanup preserves an earlier failure if deletion also fails.

The generated SDK uses `java-library` and includes a sources JAR and publication metadata. Other Gradle projects, Java or Kotlin, can use the same composite build or consume the published coordinates from your artifact repository.

With the local service running, verify the example's pagination and cleanup with:

```sh
cargo test --manifest-path client/Cargo.toml --test example -- --ignored
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

Creation accepts `name`, optional `notes`, and `clientToken`. Provide a token when using raw HTTP; generated SDKs can fill it automatically. Repeating an identical create with the same token returns the original result. Changed fields with that token return `ConflictException` (409), including the conflicting item ID. Receipts last eight hours, including after deletion; replaying one does not recreate a deleted item.

Update replaces the editable fields. Omitting `notes` clears them. Get and update return `ResourceNotFoundException` (404) for a missing item. Delete succeeds even if the item is already absent.

List returns at most `maxResults` items (required, maximum 100), ordered by immutable ID. Pass `nextToken` to continue; page size can change between requests. Tokens are opaque references, expire after 15 minutes, and become invalid on restart. At most 1,024 cursors are retained; issuing another evicts the oldest. Invalid tokens return `ValidationException` (400).

All handlers share one store. Items, create receipts, and cursors disappear when the process stops. The example permits 1,000 items and 10,000 unexpired create receipts; reaching either limit returns `ServiceQuotaExceededException` (402). It has no authentication and is intended for local learning. Replace the store with durable storage before running multiple instances.

## Changing the example

`model/api/service.cru` defines the resource, HTTP bindings, constraints, pagination, and exceptions. Members are required unless written `name?:`, operations attach to `Item` with role verbs such as `creates` and `reads`, and URI labels bind input members by name. `model/api/crucible.toml` names the namespace and imports Crucible's standard errors. `service/src/operations/item/` has one handler per operation. `service/src/store/` owns state and atomic mutations.

Edit the model and implement the generated traits. `model/build.rs` compiles and validates the model, then exports complete Rust and Java SDK packages; `service/build.rs` generates server bindings for the `my_service` namespace in `service/src/main.rs`. A model error fails the build with every diagnostic, each pointing at its `.cru` file, line, and column. Generated output stays under the root `target/` directory.

From the repository root, run `cargo fmt --all`, `cargo clippy --workspace --all-targets -- -D warnings`, and `cargo test --workspace` to check the service. Run the same commands with `--manifest-path client/Cargo.toml` to check the independent Rust client after generation. With the server running, `python service/tools/smoke.py` checks the API over real HTTP, or pass another endpoint as its first argument.

## Evolving the API: releases and migrations

The model is at release 2 (`version "2"`). Release 1 called the item's notes `description` and let `ListItems` omit `maxResults`. Both changes break release 1's clients, so the build accepts them only because `model/api/migrations.cru` declares them:

```text
migration "2" from "1" {
    rename ItemDetails.description -> notes
    rename CreateItem.input.description -> notes
    rename UpdateItem.input.description -> notes
    require ListItems.input.maxResults default 25
    sunset "2027-12-31"
}
```

`model/api/releases/1/` is release 1, frozen, and `[model.baseline]` in `crucible.toml` points at it. Every build compares the model with that baseline. A breaking change no migration declares fails the build, and so does a step for a change that did not happen. With the IntelliJ plugin, both show in the editor as you type.

The service serves both releases from the same handlers. A request names its release in the `Crucible-Version` header. A release 1 request is read with release 1's types, converted to release 2's (`description` becomes `notes`, and a missing `maxResults` becomes 25), handled, and its response converted back. Release 1 responses carry `Sunset: Fri, 31 Dec 2027 00:00:00 GMT`. Try the last two requests in `service/requests.http`, or:

```sh
curl -H 'Crucible-Version: 1' 'http://127.0.0.1:8080/items'
```

The generated SDKs do not send their release yet, so `service/src/main.rs` serves a request without the header at the current release (`Unversioned::Resolve`). Once clients send theirs, the framework's default serves an unversioned request at the oldest release, as built for clients that predate versions.

To release a breaking change of your own:

1. Freeze the current release before changing it: `crucible freeze model/api`, using the CLI from Crucible's repository (`cargo run -p crucible-cli -- freeze <path>`). It writes `model/api/releases/<version>`.
2. Point `[model.baseline] path` at the new release, bump `version` in `service.cru`, and make the change.
3. Declare it in a new `migration "<new>" from "<old>" { .. }`. The build lists every breaking change and what step declares it.

Besides `rename`, `require .. default` and `sunset`, a migration can:
- `add` an enum value or an operation error, with the value or error older clients see instead;
- `move` an operation's route;
- mark a `behavior` change, which a handler branches on with `self.context().api_version()`.

Changes the framework cannot convert by itself, `reshape`, `retype` and `require .. by hook`, become methods of a generated `my_service::versions::v1::Migration` trait that the service implements. So does `remove operation`, whose handler keeps serving older releases. See Crucible's [API versions guide](https://github.com/Operation-Leyline/crucible/blob/f753776a0246bfdd35b5413fadafeb703e9edd02/docs/api-versions.md).

## API policy and compatibility

`model/api/crucible.toml` holds the API policy beside the package manifest: naming and path conventions enforced by every Cargo build, with documentation requirements as warnings. Every build of the package applies it, the model's and the service's, with no `.policy(..)`; invalid rules and error findings fail generation. Any check can set its own severity, such as `members = { case = "camelCase", severity = "warning" }`. With the IntelliJ plugin, findings show in the `.cru` file as you type, and a naming finding offers a rename to the right case. Edit the rules for your service. See the [policy reference](https://github.com/Operation-Leyline/crucible/blob/f753776a0246bfdd35b5413fadafeb703e9edd02/docs/api-policy.md) for severities and exact, documented exceptions.

The pull-request compatibility check (`api-diff`) compares Smithy sources, so it is not included while the model is `.cru`. It returns once the checker reads `.cru` packages.

Keep both root `Cargo.lock` and `client/Cargo.lock` committed. The SDK verification workflow starts from a clean build, checks the service and native clients over HTTP, and verifies cleanup. It fetches the private Crucible repository: after creating a repository from this template, configure variable `CRUCIBLE_APP_ID` and secret `CRUCIBLE_APP_PRIVATE_KEY` for the existing `crucible-ci-read` GitHub App, whose Operation-Leyline installation needs Contents: read access to `crucible`. GitHub does not copy template repository secrets into new repositories.

## Editor support

RustRover and IntelliJ IDEA 2026.2 support `.cru` files through the [Crucible plugin](https://github.com/Operation-Leyline/crucible-intellij). Install its release ZIP through **Settings > Plugins > Install Plugin from Disk**. It bundles the `crucible-lsp` language server, so opening `model/api/service.cru` shows the compiler's errors as you type, with completion, go to declaration, find usages, rename, hover, the structure view, and Reformat Code. `model/api/crucible.toml` is checked too.

## Dependency versions

The framework and build helper in the root `Cargo.toml` pin the same immutable commit. The generated Rust SDK automatically pins its runtime to that generator revision. Update the two root dependency pins together when adopting a release. `Cargo.lock` is committed, and the model compiles in Rust, so no Smithy compiler or JVM is involved.

To build against a local Crucible checkout while changing Crucible itself, point both dependencies at it with `path = "<checkout>/crates/..."`, and add `.runtime_path("<checkout>/crates/crucible-client-runtime-rust")` to the Rust SDK in `model/build.rs`.