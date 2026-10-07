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
