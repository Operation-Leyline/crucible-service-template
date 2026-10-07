fn main() {
    crucible_build::service("service_model")
        .policy("../crucible.toml")
        .generate();
}
