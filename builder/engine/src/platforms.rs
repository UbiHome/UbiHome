//! Detect which UbiHome platform components a `config.yml` uses, and discover
//! which components are available to build from the UbiHome source tree.
//!
//! Detection mirrors `get_platforms_from_config` + `is_base_entity_property` in
//! the main crate (`src/config.rs`): a "platform" is any top-level YAML key that
//! is not one of the `BaseConfig` fields. Keep [`RESERVED`] in sync with that
//! struct. The `detect_test.rs` fixtures guard against drift.

use std::path::Path;

use crate::BuilderError;

/// Top-level YAML keys that are NOT platform components. Mirrors the field names
/// of `BaseConfig` in `src/config.rs` (`is_base_entity_property`).
pub const RESERVED: &[&str] = &[
    "ubihome",
    "logger",
    "button",
    "sensor",
    "binary_sensor",
    "number",
    "switch",
    "light",
    "text_sensor",
];

/// Parse the platform components referenced by a config file's YAML.
///
/// Uses the same line-based heuristic as the runtime: a top-level key is a line
/// that does not start with whitespace, `#`, or `-`, and is not blank. The key
/// is the text before the first `:`. Reserved (base-config) keys are dropped.
/// The result is de-duplicated and sorted for stable output.
pub fn detect_platforms(config: &str) -> Vec<String> {
    let mut found: Vec<String> = Vec::new();
    for line in config.lines() {
        if line.starts_with(' ')
            || line.starts_with('\t')
            || line.is_empty()
            || line.starts_with('#')
            || line.starts_with('-')
        {
            continue;
        }
        let Some(key) = line.split(':').next() else {
            continue;
        };
        let key = key.trim();
        if key.is_empty() || RESERVED.contains(&key) {
            continue;
        }
        if !found.iter().any(|p| p == key) {
            found.push(key.to_string());
        }
    }
    found.sort();
    found
}

/// Read the UbiHome root `Cargo.toml` and return the list of buildable platform
/// component names (the `ubihome-*` dependencies, minus `ubihome-core`, with the
/// `ubihome-` prefix stripped). This is the same set `build.rs` turns into the
/// component registry, so it is the source of truth for what can be selected.
pub fn available_components(source_cargo_toml: &Path) -> Result<Vec<String>, BuilderError> {
    let text = std::fs::read_to_string(source_cargo_toml).map_err(|e| {
        BuilderError::Source(format!("cannot read {}: {e}", source_cargo_toml.display()))
    })?;
    let doc: toml_edit::DocumentMut = text
        .parse()
        .map_err(|e| BuilderError::Source(format!("invalid Cargo.toml: {e}")))?;
    let deps = doc
        .get("dependencies")
        .and_then(|d| d.as_table())
        .ok_or_else(|| BuilderError::Source("Cargo.toml has no [dependencies]".into()))?;

    let mut components: Vec<String> = deps
        .iter()
        .map(|(k, _)| k.to_string())
        .filter(|k| k.starts_with("ubihome-") && k != "ubihome-core")
        .map(|k| k.trim_start_matches("ubihome-").to_string())
        .collect();
    components.sort();
    Ok(components)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_top_level_platforms_excluding_base_fields() {
        let cfg = r#"
ubihome:
  name: "Test"
logger:
  level: debug

api:
  port: 6053

mqtt:
  broker: localhost

shell:
  type: bash

sensor:
  - platform: shell
    name: "uptime"

button:
  - platform: shell
    name: "reboot"
"#;
        assert_eq!(detect_platforms(cfg), vec!["api", "mqtt", "shell"]);
    }

    #[test]
    fn dedups_and_ignores_comments_and_lists() {
        let cfg = r#"
# a comment
mqtt:
  broker: a
mqtt:
  broker: b
- not_a_key: x
"#;
        assert_eq!(detect_platforms(cfg), vec!["mqtt"]);
    }

    #[test]
    fn empty_config_has_no_platforms() {
        assert!(detect_platforms("").is_empty());
    }
}
