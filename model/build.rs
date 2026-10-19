fn main() {
    // `api/` is a Crucible package; `api/crucible.toml` holds its API policy,
    // which every build applies.
    let model = crucible_build::model("api").compile();
    model.publish();
    // An OpenAPI 3.1 document for tools that read one (API gateways,
    // Postman, documentation sites), with the local server as its endpoint.
    model
        .openapi("openapi")
        .endpoint("http://127.0.0.1:8080")
        .generate_into("../target/openapi");
}
