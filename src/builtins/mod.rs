//! Builtin components that are compiled directly into the main binary instead
//! of living in a `ubihome-*` platform crate.
//!
//! Template switches and globals are handled here because they are tightly
//! integrated with the central runtime: their automations reference entities
//! (switches, buttons) and globals that belong to *other* platforms, and those
//! cross-cutting references are only resolvable on the central message bus in
//! [`crate::commands::run`]. A standalone platform crate only sees its own
//! entities, so it could not drive them. See `documentation` for the rationale.

pub mod globals;
pub mod template_switch;

use std::collections::HashMap;

use garde::Validate;
use serde::de::IntoDeserializer;
use serde::Deserialize;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinSet;
use ubihome_core::configuration::automation::{Action, ActionType};
use ubihome_core::serde_value::Value;
use ubihome_core::state::{EntityState, StateStoreWriter};
use ubihome_core::PublishedMessage;

pub use globals::{GlobalConfig, Globals};
pub use template_switch::TemplateSwitchConfig;

/// The switch platform name handled by the builtin template switch.
pub const TEMPLATE_PLATFORM: &str = "template";
/// Top-level config sections handled directly by the main binary (i.e. not
/// loaded as dynamic platform crates).
pub const BUILTIN_SECTIONS: &[&str] = &["globals"];

#[derive(Debug, Deserialize, Validate)]
#[garde(allow_unvalidated)]
struct BuiltinRoot {
    #[serde(default)]
    #[garde(skip)]
    switch: Vec<Value>,

    #[serde(default)]
    #[garde(dive)]
    globals: Vec<GlobalConfig>,
}

/// The parsed builtin configuration.
#[derive(Debug, Default)]
pub struct BuiltinConfig {
    pub template_switches: Vec<TemplateSwitchConfig>,
    pub globals: Vec<GlobalConfig>,
}

fn platform_of(value: &Value) -> Option<&str> {
    if let Value::Map(entries) = value {
        for (key, val) in entries {
            if let (Value::String(k), Value::String(v)) = (key, val) {
                if k == "platform" {
                    return Some(v.as_str());
                }
            }
        }
    }
    None
}

/// Parse the template switches and globals out of the raw configuration.
pub fn parse(config_string: &str, config_path: &str) -> Result<BuiltinConfig, String> {
    let root =
        ubihome_core::validation::validate_config::<BuiltinRoot>(config_string, config_path)?;

    let mut template_switches = Vec::new();
    for raw in root.switch {
        if platform_of(&raw) != Some(TEMPLATE_PLATFORM) {
            continue;
        }
        let switch = TemplateSwitchConfig::deserialize(raw.into_deserializer())
            .map_err(|e| format!("Invalid template switch configuration: {}", e))?;
        switch
            .validate_with(&())
            .map_err(|e| format!("Invalid template switch '{}': {}", switch.name, e))?;
        template_switches.push(switch);
    }

    Ok(BuiltinConfig {
        template_switches,
        globals: root.globals,
    })
}

/// Run a list of automation actions in order, publishing the corresponding
/// messages onto the internal bus. Shared by every trigger site (binary sensor
/// `on_press`/`on_release`, template switch `turn_on_action`/`turn_off_action`).
///
/// Actions are executed sequentially so that a `delay` action pauses the list
/// before the following actions run.
pub async fn run_actions(actions: Vec<Action>, tx: &Sender<PublishedMessage>, globals: &Globals) {
    for action in actions {
        match &action.action {
            ActionType::SwitchTurnOn(key) => {
                let _ = tx.send(PublishedMessage::SwitchStateCommand {
                    key: key.clone(),
                    state: true,
                });
            }
            ActionType::SwitchTurnOff(key) => {
                let _ = tx.send(PublishedMessage::SwitchStateCommand {
                    key: key.clone(),
                    state: false,
                });
            }
            ActionType::ButtonPress(key) => {
                let _ = tx.send(PublishedMessage::ButtonPressed { key: key.clone() });
            }
            ActionType::GlobalsSet { id, value } => {
                globals.set(id, value);
            }
            ActionType::Delay(duration) => {
                tokio::time::sleep(*duration).await;
            }
        }
    }
}

/// Spawn the runtime handler for template switches. It listens for
/// [`PublishedMessage::SwitchStateCommand`] targeting a template switch, runs
/// the matching `turn_on_action`/`turn_off_action`, and (when `optimistic`)
/// publishes the new state back onto the bus so connected front-ends update.
///
/// Template switches are not platform modules, so nothing else records their
/// state into the central [`StateStore`](ubihome_core::state::StateStore) the
/// way the runtime's internal command router does for every other component
/// (see `crate::commands::run`). Every `SwitchStateChange` published here is
/// therefore mirrored into `state_writer` directly, so a client that connects
/// after the fact (e.g. `api`'s `SubscribeStatesRequest`) still observes the
/// current state instead of nothing.
pub fn spawn_template_switches(
    tasks: &mut JoinSet<()>,
    switches: Vec<TemplateSwitchConfig>,
    tx: Sender<PublishedMessage>,
    globals: Globals,
    state_writer: StateStoreWriter,
) {
    if switches.is_empty() {
        return;
    }
    let switches: HashMap<String, TemplateSwitchConfig> = switches
        .into_iter()
        .map(|s| (s.get_object_id(), s))
        .collect();
    let mut receiver: Receiver<PublishedMessage> = tx.subscribe();
    let mut global_changes = globals.subscribe();
    tasks.spawn(async move {
        // Publish the initial state of switches whose `lambda` reads a global.
        for (key, switch) in &switches {
            if let Some(id) = switch.state_global() {
                if let Some(state) = globals.get_bool(id) {
                    log::debug!("template switch '{}' initial state from global '{}': {}", key, id, state);
                    state_writer.set(key.clone(), EntityState::Switch(state));
                    let _ = tx.send(PublishedMessage::SwitchStateChange {
                        key: key.clone(),
                        state,
                    });
                }
            }
        }

        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    match msg {
                        Ok(PublishedMessage::SwitchStateCommand { key, state }) => {
                            if let Some(switch) = switches.get(&key) {
                                let actions = if state {
                                    switch.turn_on_action.clone()
                                } else {
                                    switch.turn_off_action.clone()
                                };
                                if let Some(actions) = actions {
                                    run_actions(actions, &tx, &globals).await;
                                }
                                // With a `globals.get` lambda the state follows the
                                // global (published via the change notification
                                // below), so only optimistic switches without a
                                // lambda echo the command.
                                if switch.optimistic && switch.state_global().is_none() {
                                    state_writer.set(key.clone(), EntityState::Switch(state));
                                    let _ = tx.send(PublishedMessage::SwitchStateChange {
                                        key,
                                        state,
                                    });
                                }
                            }
                        }
                        // Ignore other messages; stop only when the bus closes.
                        Ok(_) => {}
                        Err(_) => break,
                    }
                }
                changed = global_changes.recv() => {
                    // A lagged receiver just misses an update; keep going.
                    if let Ok(id) = changed {
                        for (key, switch) in &switches {
                            if switch.state_global() == Some(id.as_str()) {
                                if let Some(state) = globals.get_bool(&id) {
                                    log::debug!("template switch '{}' state from global '{}': {}", key, id, state);
                                    state_writer.set(key.clone(), EntityState::Switch(state));
                                    let _ = tx.send(PublishedMessage::SwitchStateChange {
                                        key: key.clone(),
                                        state,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    });
}
