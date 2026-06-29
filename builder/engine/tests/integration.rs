//! Drift guard: parse a real UbiHome root Cargo.toml and confirm the engine
//! discovers the expected platform components. The builder lives inside the
//! UbiHome repo during development, so the repo root is two levels up from this
//! crate (builder/engine -> builder -> repo root).

use std::path::Path;

use ubihome_builder_engine::platforms::available_components;

#[test]
fn discovers_real_components_from_source() {
    let repo_cargo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("..")
        .join("..")
        .join("Cargo.toml");
    if !repo_cargo.is_file() {
        // Not running inside the repo (e.g. packaged crate); nothing to guard.
        return;
    }
    let components = available_components(&repo_cargo).expect("read components from Cargo.toml");

    for expected in ["api", "mqtt", "shell"] {
        assert!(
            components.iter().any(|c| c == expected),
            "expected component '{expected}' in {components:?}"
        );
    }
    assert!(
        !components.iter().any(|c| c == "core"),
        "ubihome-core must not be listed as a selectable component"
    );
}
