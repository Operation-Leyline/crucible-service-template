fn main() {
    crucible_build::rust_service("service_model")
        .policy("../crucible.toml")
        .generate();
}
