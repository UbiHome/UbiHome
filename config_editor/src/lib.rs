//! Visual configuration editor for UbiHome, built with egui.
//!
//! The editor works on an order-preserving YAML tree ([`yaml::Yaml`]), shows
//! the configuration as sections and entities with schema-aware forms
//! ([`schema`]), and offers a raw YAML view for anything the forms don't
//! cover. It compiles natively (a window, launched by `ubihome edit` when the
//! `config_editor` feature is enabled, or the standalone binary of this crate)
//! and to WebAssembly (see `index.html` / `Trunk.toml`).

mod app;
#[cfg(target_arch = "wasm32")]
mod web;
mod widgets;

pub mod schema;
pub mod validate;
pub mod yaml;

pub use app::ConfigEditorApp;

/// Full validation provided by the host (e.g. the `ubihome` binary, which
/// knows every compiled-in platform). Receives the YAML text and the file
/// name used in error messages; returns a human-readable report on failure.
pub type Validator = Box<dyn Fn(&str, &str) -> Result<(), String>>;

/// Opens the editor in a native window and blocks until it is closed.
#[cfg(not(target_arch = "wasm32"))]
pub fn run_native(
    config_path: Option<std::path::PathBuf>,
    validator: Option<Validator>,
) -> Result<(), String> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("UbiHome Configuration")
            .with_inner_size([1100.0, 720.0])
            .with_min_inner_size([640.0, 400.0]),
        ..Default::default()
    };
    eframe::run_native(
        "UbiHome Configuration",
        options,
        Box::new(move |cc| Ok(Box::new(ConfigEditorApp::new(cc, config_path, validator)))),
    )
    .map_err(|e| e.to_string())
}

/// Starts the editor in the browser (see `index.html`).
#[cfg(target_arch = "wasm32")]
pub fn start_web() {
    web::start();
}
