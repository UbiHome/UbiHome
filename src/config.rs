//! Whole-file validation of the configuration.
//!
//! The root config type itself is generated per compiled-in platform set (see
//! `crate::components`); this module holds the validation that spans the whole
//! document: platform availability, duplicate ids, and references from
//! actions/lambdas to switches, buttons and globals. Because it runs as part
//! of the single `serde_saphyr` + `garde` pass, every error is reported with
//! its source location.

use std::collections::HashMap;

use garde::{Path, Report, Validate};
use ubihome::Logger;
use ubihome_core::configuration::automation::{ActionType, Trigger};
use ubihome_core::UbiHome;

use crate::builtins::globals::{coerce_value, GlobalType};
use crate::builtins::{GlobalConfig, TEMPLATE_PLATFORM};

/// Uniform view of one entry of an entity list, whatever its platform.
pub(crate) trait AnyEntity {
    fn platform(&self) -> &str;
    /// Whether `platform:` names a compiled-in platform or builtin.
    fn is_compiled_in(&self) -> bool;
    /// Whether the platform provides this kind of entity.
    fn is_supported(&self) -> bool;
    fn object_id(&self) -> String;
    fn triggers(&self) -> Vec<(&'static str, &Trigger)>;
    /// The global a template entity's `lambda` reads, with the global types
    /// it accepts.
    fn lambda_global(&self) -> Option<(&str, &'static [GlobalType])>;
    fn validate_entity(&self, parent: &mut dyn FnMut() -> Path, report: &mut Report);
}

/// Borrowed view of the root config, handed over by the generated root type.
pub(crate) struct ConfigView<'a> {
    pub ubihome: &'a UbiHome,
    pub logger: Option<&'a Logger>,
    pub globals: &'a [GlobalConfig],
    pub configured_platforms: Vec<&'static str>,
    pub unknown_keys: &'a [String],
    /// `(kind, entities)`, e.g. `("switch", [...])`.
    pub entities: Vec<(&'static str, Vec<&'a dyn AnyEntity>)>,
}

/// Entities of one kind that passed the platform checks, with their index in
/// the YAML list.
type ValidEntities<'a> = (&'static str, Vec<(usize, &'a dyn AnyEntity)>);

/// The ids actions and lambdas may refer to.
struct References<'a> {
    switches: Vec<String>,
    buttons: Vec<String>,
    globals: HashMap<&'a str, &'a GlobalType>,
}

pub(crate) fn validate_root(
    view: &ConfigView,
    parent: &mut dyn FnMut() -> Path,
    report: &mut Report,
) {
    view.ubihome
        .validate_into(&(), &mut || parent().join("ubihome"), report);
    if let Some(logger) = view.logger {
        logger.validate_into(&(), &mut || parent().join("logger"), report);
    }

    for key in view.unknown_keys {
        report.append(
            parent().join(key.as_str()),
            garde::Error::new(format!(
                "Unknown platform specified: {key}\n\
                 Remove the \"{key}:\" entry from your configuration or install the cargo crate containing the platform."
            )),
        );
    }

    let mut globals: HashMap<&str, &GlobalType> = HashMap::new();
    for (index, global) in view.globals.iter().enumerate() {
        let base = parent().join("globals").join(index);
        let path = || base.clone();
        global.validate_into(&(), &mut || path(), report);
        if globals.insert(&global.id, &global.value_type).is_some() {
            report.append(
                path().join("id"),
                garde::Error::new(format!("Duplicate global id '{}'", global.id)),
            );
        }
    }

    let mut allowed_platforms: Vec<&str> = view.configured_platforms.clone();
    allowed_platforms.push(TEMPLATE_PLATFORM);

    // Entities that passed the platform checks, per kind, with their index.
    let mut valid: Vec<ValidEntities> = Vec::new();
    for (kind, entities) in &view.entities {
        let mut seen: HashMap<String, usize> = HashMap::new();
        let mut kind_valid = Vec::new();
        for (index, entity) in entities.iter().enumerate() {
            let entity = *entity;
            let base = parent().join(*kind).join(index);
            let path = || base.clone();
            let platform = entity.platform();
            if !entity.is_compiled_in() || !allowed_platforms.contains(&platform) {
                report.append(
                    path().join("platform"),
                    garde::Error::new(format!(
                        "Platform '{}' is not configured in the configuration file. Allowed platforms are: {}",
                        platform,
                        allowed_platforms.join(", ")
                    )),
                );
                continue;
            }
            if !entity.is_supported() {
                report.append(
                    path().join("platform"),
                    garde::Error::new(format!(
                        "Platform '{platform}' does not provide '{kind}' entities"
                    )),
                );
                continue;
            }

            entity.validate_entity(&mut || path(), report);

            let id = entity.object_id();
            if let Some(first) = seen.get(&id) {
                report.append(
                    path(),
                    garde::Error::new(format!(
                        "Duplicate {kind} id '{id}' (already used by {kind}[{first}])"
                    )),
                );
            } else {
                seen.insert(id, index);
            }
            kind_valid.push((index, entity));
        }
        valid.push((kind, kind_valid));
    }

    let ids_of = |wanted: &str| -> Vec<String> {
        valid
            .iter()
            .filter(|(kind, _)| *kind == wanted)
            .flat_map(|(_, entities)| entities.iter().map(|(_, e)| e.object_id()))
            .collect()
    };
    let refs = References {
        switches: ids_of("switch"),
        buttons: ids_of("button"),
        globals,
    };

    if let Some(trigger) = &view.ubihome.on_startup {
        validate_trigger(
            trigger,
            &refs,
            &mut || parent().join("ubihome").join("on_startup"),
            report,
        );
    }

    for (kind, entities) in &valid {
        for (index, entity) in entities {
            let base = parent().join(*kind).join(*index);
            let path = || base.clone();
            for (name, trigger) in entity.triggers() {
                validate_trigger(trigger, &refs, &mut || path().join(name), report);
            }
            if let Some((id, accepted)) = entity.lambda_global() {
                let error = match refs.globals.get(id) {
                    None => Some(unknown("global", id, refs.globals.keys().copied())),
                    Some(value_type) if !accepted.contains(value_type) => Some(format!(
                        "Global '{id}' has type '{}', but a template {kind} lambda needs a global of type {}",
                        type_name(value_type),
                        accepted
                            .iter()
                            .map(|t| format!("'{}'", type_name(t)))
                            .collect::<Vec<_>>()
                            .join(" or ")
                    )),
                    Some(_) => None,
                };
                if let Some(error) = error {
                    report.append(path().join("lambda"), garde::Error::new(error));
                }
            }
        }
    }
}

fn validate_trigger(
    trigger: &Trigger,
    refs: &References,
    parent: &mut dyn FnMut() -> Path,
    report: &mut Report,
) {
    for (index, action) in trigger.then.iter().enumerate() {
        let base = parent().join("then").join(index);
        let path = || base.clone();
        match &action.action {
            ActionType::SwitchTurnOn(id) | ActionType::SwitchTurnOff(id) => {
                let key = match &action.action {
                    ActionType::SwitchTurnOn(_) => "switch.turn_on",
                    _ => "switch.turn_off",
                };
                if !refs.switches.contains(id) {
                    report.append(
                        path().join(key),
                        garde::Error::new(unknown(
                            "switch",
                            id,
                            refs.switches.iter().map(String::as_str),
                        )),
                    );
                }
            }
            ActionType::ButtonPress(id) => {
                if !refs.buttons.contains(id) {
                    report.append(
                        path().join("button.press"),
                        garde::Error::new(unknown(
                            "button",
                            id,
                            refs.buttons.iter().map(String::as_str),
                        )),
                    );
                }
            }
            ActionType::GlobalsSet { id, value } => match refs.globals.get(id.as_str()) {
                None => report.append(
                    path().join("globals.set").join("id"),
                    garde::Error::new(unknown("global", id, refs.globals.keys().copied())),
                ),
                Some(value_type) => {
                    if let Err(error) = coerce_value(value_type, value.clone()) {
                        report.append(
                            path().join("globals.set").join("value"),
                            garde::Error::new(format!(
                                "Invalid value for global '{id}' of type '{}': {error}",
                                type_name(value_type)
                            )),
                        );
                    }
                }
            },
            ActionType::Delay(_) | ActionType::LoggerLog(_) => {}
        }
    }
}

fn unknown<'a>(what: &str, id: &str, known: impl Iterator<Item = &'a str>) -> String {
    let plural = if what == "switch" {
        "switches".to_string()
    } else {
        format!("{what}s")
    };
    let mut known: Vec<&str> = known.collect();
    known.sort_unstable();
    if known.is_empty() {
        format!("Unknown {what} '{id}': no {plural} are configured")
    } else {
        format!(
            "Unknown {what} '{id}'. Configured {plural} are: {}",
            known.join(", ")
        )
    }
}

fn type_name(value_type: &GlobalType) -> &'static str {
    match value_type {
        GlobalType::Bool => "bool",
        GlobalType::Int => "int",
        GlobalType::Float => "float",
        GlobalType::String => "string",
    }
}

#[cfg(test)]
mod tests {
    use crate::components::RootConfig;

    /// Parse `config`; errors are returned with miette's line wrapping undone.
    fn parse(config: &str) -> Result<RootConfig, String> {
        ubihome_core::validation::validate_config::<RootConfig>(config, "config.yml").map_err(
            |error| {
                error
                    .lines()
                    .map(|line| line.trim_start().trim_start_matches('│').trim())
                    .collect::<Vec<_>>()
                    .join(" ")
            },
        )
    }

    #[test]
    fn splits_entities_by_platform_in_one_pass() {
        let config = parse(
            r#"
ubihome:
  name: test_split

shell:
  type: bash

gpio:
  device: raspberryPi

switch:
  # `platform` not first: the leading keys are buffered and replayed.
  - id: relay
    pin: 7
    platform: gpio
  - platform: shell
    id: fan
    command_on: "true"
    command_off: "true"
  - platform: template
    id: virtual

button:
  - platform: template
    id: all_on
    on_press:
      then:
        - switch.turn_on: relay
        - switch.turn_on: fan
        - switch.turn_on: virtual
"#,
        )
        .expect("valid config");

        assert_eq!(config.configured_platforms(), vec!["gpio", "shell"]);
        let builtin = config.builtin();
        assert_eq!(builtin.template.switches.len(), 1);
        assert_eq!(builtin.template.buttons.len(), 1);
    }

    #[test]
    fn id_only_entities_are_accepted() {
        parse(
            r#"
ubihome:
  name: test_switch_id_only

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    id: relay
    pin: 7
    restore_mode: ALWAYS_ON
"#,
        )
        .expect("id-only entity must be valid");
    }

    #[test]
    fn rejects_entity_without_name_or_id() {
        let error = parse(
            r#"
ubihome:
  name: test_switch_neither

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    pin: 7
"#,
        )
        .expect_err("neither name nor id");
        assert!(error.contains("Either 'name' or 'id'"), "{error}");
    }

    #[test]
    fn reports_unknown_action_targets() {
        let error = parse(
            r#"
ubihome:
  name: test_refs
  on_startup:
    then:
      - button.press: missing_button

globals:
  - id: power
    type: bool

shell:

binary_sensor:
  - platform: shell
    name: Door
    command: "echo ON"
    on_press:
      then:
        - switch.turn_on: missing_switch
        - globals.set:
            id: power
            value: 1
"#,
        )
        .expect_err("dangling references");
        for expected in [
            "Unknown button 'missing_button': no buttons are configured",
            "Unknown switch 'missing_switch': no switches are configured",
            "Invalid value for global 'power' of type 'bool'",
        ] {
            assert!(error.contains(expected), "missing '{expected}' in {error}");
        }
    }
}
