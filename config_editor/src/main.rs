//! Standalone editor binary. Natively it opens an optional file given as the
//! first argument; on the web (built with `trunk`) it attaches to the
//! `ubihome_config_editor` canvas of `index.html`.
//!
//! The standalone build only runs the quick structural checks; the editor
//! embedded in `ubihome edit` additionally runs the full validation.

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), String> {
    let path = std::env::args_os().nth(1).map(std::path::PathBuf::from);
    ubihome_config_editor::run_native(path, None)
}

#[cfg(target_arch = "wasm32")]
fn main() {
    ubihome_config_editor::start_web();
}
