fn main() {
    // `api/` is a Crucible language package: compiled straight to the IR, with
    // no Smithy compiler or JVM involved.
    // `api/crucible.toml` holds the package's API policy; the build applies it.
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
    // An OpenAPI 3.1 document for tools that read one (API gateways,
    // Postman, documentation sites), with the local server as its endpoint.
    model
        .openapi("openapi")
        .endpoint("http://127.0.0.1:8080")
        .generate_into("../target/openapi");
}
