fn main() {
    crucible_build::from_smithy("../smithy")
        .policy("../crucible.toml")
        .generate();
}
