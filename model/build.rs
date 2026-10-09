fn main() {
    // `api/` is a Crucible language package: compiled straight to the IR, with
    // no Smithy compiler or JVM involved.
    let model = crucible_build::model("api")
        .policy("../crucible.toml")
        .compile();
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
