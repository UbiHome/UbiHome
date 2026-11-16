//! Lightweight structural checks that run everywhere (including the browser).
//!
//! These mirror the most common errors the full validator in the `ubihome`
//! binary reports, so the web build can give useful feedback without the
//! platform crates. When embedded in the binary, the full validation is run
//! in addition (see [`crate::Backend::validate`]).

use std::collections::HashSet;

use crate::schema::{self, ENTITY_KINDS};
use crate::yaml::Yaml;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Issue {
    pub severity: Severity,
    /// Human-readable location, e.g. `sensor[2].platform`.
    pub path: String,
    pub message: String,
}

impl Issue {
    fn error(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Error,
            path: path.into(),
            message: message.into(),
        }
    }
    fn warning(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            severity: Severity::Warning,
            path: path.into(),
            message: message.into(),
        }
    }
}

fn is_id(value: &str) -> bool {
    value
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Top-level keys that configure a platform (everything that is neither an
/// entity list nor a builtin section).
pub fn configured_platforms(doc: &Yaml) -> Vec<String> {
    doc.as_map()
        .map(|m| {
            m.iter()
                .map(|(k, _)| k.clone())
                .filter(|k| !schema::is_reserved_section(k))
                .collect()
        })
        .unwrap_or_default()
}

pub fn check(doc: &Yaml) -> Vec<Issue> {
    let mut issues = Vec::new();
    let Some(root) = doc.as_map() else {
        issues.push(Issue::error("", "The configuration must be a mapping."));
        return issues;
    };

    match doc.get("ubihome") {
        None => issues.push(Issue::error(
            "ubihome",
            "The 'ubihome' section is required.",
        )),
        Some(ubihome) => match ubihome.get("name").and_then(Yaml::as_str) {
            Some(name) if !name.trim().is_empty() => {}
            _ => issues.push(Issue::error("ubihome.name", "A device name is required.")),
        },
    }

    let configured = configured_platforms(doc);
    for platform in &configured {
        if schema::platform(platform).is_none() {
            issues.push(Issue::warning(
                platform.clone(),
                format!(
                    "Unknown platform '{platform}'. It must be provided by an installed component."
                ),
            ));
        }
    }

    let mut seen_ids: HashSet<String> = HashSet::new();
    for (key, value) in root {
        if !ENTITY_KINDS.iter().any(|k| k.key == key) {
            continue;
        }
        let entities = match value {
            Yaml::Seq(items) => items,
            Yaml::Null => continue,
            _ => {
                issues.push(Issue::error(key.clone(), "Expected a list of entities."));
                continue;
            }
        };
        for (index, entity) in entities.iter().enumerate() {
            let path = format!("{key}[{index}]");
            if entity.as_map().is_none() {
                issues.push(Issue::error(path, "Each entity must be a mapping."));
                continue;
            }
            match entity.get("platform").and_then(Yaml::as_str) {
                None => issues.push(Issue::error(
                    format!("{path}.platform"),
                    "An entity needs a 'platform'.",
                )),
                Some(platform) => {
                    let builtin = schema::BUILTIN_ENTITY_PLATFORMS.contains(&platform);
                    if !builtin && !configured.iter().any(|p| p == platform) {
                        issues.push(Issue::error(
                            format!("{path}.platform"),
                            format!(
                                "Platform '{platform}' is not configured. Add a top-level '{platform}:' section."
                            ),
                        ));
                    } else if let Some(info) = schema::platform(platform) {
                        if !info.entity_kinds.contains(&key.as_str()) {
                            issues.push(Issue::warning(
                                format!("{path}.platform"),
                                format!("Platform '{platform}' does not provide '{key}' entities."),
                            ));
                        }
                    }
                }
            }
            let id = entity.get("id").and_then(Yaml::as_str);
            let name = entity.get("name").and_then(Yaml::as_str);
            if id.is_none() && name.is_none() {
                issues.push(Issue::error(
                    path.clone(),
                    "Either 'name' or 'id' must be provided.",
                ));
            }
            if let Some(id) = id {
                if !is_id(id) {
                    issues.push(Issue::error(
                        format!("{path}.id"),
                        "IDs may only contain letters, numbers, '-' and '_'.",
                    ));
                }
                if !seen_ids.insert(id.to_string()) {
                    issues.push(Issue::error(
                        format!("{path}.id"),
                        format!("Duplicate id '{id}'."),
                    ));
                }
            }
            for (field, value) in [("id", id), ("name", name)] {
                if let Some(value) = value {
                    let len = value.chars().count();
                    if !(3..=100).contains(&len) {
                        issues.push(Issue::error(
                            format!("{path}.{field}"),
                            format!("'{field}' must be between 3 and 100 characters long."),
                        ));
                    }
                }
            }
        }
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;

    fn messages(config: &str) -> Vec<String> {
        check(&Yaml::parse(config).unwrap())
            .into_iter()
            .map(|i| format!("{}: {}", i.path, i.message))
            .collect()
    }

    #[test]
    fn valid_config_has_no_issues() {
        let issues = messages(
            r#"
ubihome:
  name: test
shell:
button:
  - platform: shell
    id: my_button
    command: echo hi
switch:
  - platform: template
    name: Template Switch
"#,
        );
        assert!(issues.is_empty(), "{issues:?}");
    }

    #[test]
    fn reports_common_mistakes() {
        let issues = messages(
            r#"
ubihome: {}
button:
  - platform: shell
  - platform: shell
    id: "bad id"
"#,
        );
        assert!(issues.iter().any(|m| m.starts_with("ubihome.name")));
        assert!(issues
            .iter()
            .any(|m| m.contains("Platform 'shell' is not configured")));
        assert!(issues.iter().any(|m| m.contains("Either 'name' or 'id'")));
        assert!(issues.iter().any(|m| m.starts_with("button[1].id")));
    }
}
