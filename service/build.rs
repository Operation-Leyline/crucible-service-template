fn main() {
    crucible_build::model("service_model")
        .server()
        .policy("../crucible.toml")
        .generate();
}
