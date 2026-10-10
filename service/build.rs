fn main() {
    // The published package carries its policy in its crucible.toml.
    crucible_build::model("service_model").server().generate();
}
