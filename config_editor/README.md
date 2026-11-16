# UbiHome configuration editor

A visual editor for UbiHome configuration files, built with [egui](https://www.egui.rs/).

- **Embedded:** `cargo run --features config_editor -- edit -c config.yaml` (the `ubihome edit` command). Saving writes the file; the full validation of the binary (same as `ubihome validate`) runs automatically after each change.
- **Standalone:** `cargo run -p ubihome-config_editor -- config.yaml`. Same editor, quick structural checks only.
- **Web:** `rustup target add wasm32-unknown-unknown`, then `trunk serve` (or `trunk build --release` → `dist/`) in this directory. Open loads a local file, Download saves it, and the current text is kept in local storage.

## Layout

- `src/yaml.rs`: order-preserving YAML value (parsed/emitted with `serde-saphyr`).
- `src/schema.rs`: field hints per section, entity kind and platform. Keep in sync with the platform crates when adding options.
- `src/validate.rs`: quick structural checks that also run in the browser.
- `src/app.rs` / `src/widgets.rs`: the egui UI.

Text is saved verbatim until the first visual edit. After that, the file is re-emitted from the tree and comments are dropped, and the UI warns about this.
