//! Builtin `platform: template` entities (switch, button, number): entities
//! driven entirely by automations instead of hardware. See
//! `crate::builtins` for why these are handled by the main binary instead of
//! a platform crate.

pub mod button;
pub mod number;
pub mod switch;

pub use button::TemplateButtonConfig;
pub use number::TemplateNumberConfig;
pub use switch::TemplateSwitchConfig;

use std::collections::HashMap;

use tokio::sync::broadcast::{Receiver, Sender};
use tokio::task::JoinSet;
use ubihome_core::internal::sensors::{UbiButton, UbiComponent, UbiNumber, UbiSwitch};
use ubihome_core::state::{EntityState, StateStoreWriter};
use ubihome_core::PublishedMessage;

use crate::builtins::script::ScriptValue;
use crate::builtins::{run_actions, Globals, ScriptEngine};

/// The parsed `platform: template` entities from the configuration.
#[derive(Debug, Default, Clone)]
pub struct TemplateConfig {
    pub switches: Vec<TemplateSwitchConfig>,
    pub buttons: Vec<TemplateButtonConfig>,
    pub numbers: Vec<TemplateNumberConfig>,
}

/// The `UbiComponent` entries every configured template entity exposes to
/// the runtime (state store, connectivity components like api/mqtt).
pub fn to_components(config: &TemplateConfig) -> Vec<UbiComponent> {
    let mut components = Vec::new();

    for switch in &config.switches {
        components.push(UbiComponent::Switch(UbiSwitch {
            platform: "switch".to_string(),
            icon: switch.icon.clone(),
            name: switch.name.clone().unwrap_or_default(),
            id: switch.get_object_id(),
            internal: switch.internal,
            device_class: switch.device_class.clone(),
            assumed_state: switch.is_assumed_state(),
        }));
    }

    for button in &config.buttons {
        components.push(UbiComponent::Button(UbiButton {
            platform: "button".to_string(),
            icon: button.icon.clone(),
            name: button.name.clone().unwrap_or_default(),
            id: button.get_object_id(),
            internal: button.internal,
        }));
    }

    for number in &config.numbers {
        components.push(UbiComponent::Number(UbiNumber {
            platform: "number".to_string(),
            icon: number.icon.clone(),
            name: number.name.clone().unwrap_or_default(),
            id: number.get_object_id(),
            internal: number.internal,
            min_value: number.min_value.unwrap_or(0.0),
            max_value: number.max_value.unwrap_or(100.0),
            step: number.step.unwrap_or(1.0),
            unit_of_measurement: number.unit_of_measurement.clone(),
            device_class: number.device_class.clone(),
            mode: 1, // NumberMode::Box
        }));
    }

    components
}

/// Spawn the runtime handlers for every configured template switch, button
/// and number.
pub fn spawn(
    tasks: &mut JoinSet<()>,
    config: TemplateConfig,
    tx: Sender<PublishedMessage>,
    globals: Globals,
    state_writer: StateStoreWriter,
    script: ScriptEngine,
) {
    spawn_switches(
        tasks,
        config.switches,
        tx.clone(),
        globals.clone(),
        state_writer.clone(),
        script.clone(),
    );
    spawn_buttons(
        tasks,
        config.buttons,
        tx.clone(),
        globals.clone(),
        script.clone(),
    );
    spawn_numbers(tasks, config.numbers, tx, globals, state_writer, script);
}

/// Evaluates an entity's `lambda`, logging and returning `None` on error or
/// on a return type mismatch. `ScriptValue::None` (the lambda returned
/// nothing, or `null`) also maps to `None`: skip this round's update, same as
/// the standalone `lambda` sensor platform.
async fn eval_bool_lambda(script: &ScriptEngine, key: &str, lambda: &str) -> Option<bool> {
    match script.eval(lambda, None).await {
        Ok(ScriptValue::Bool(value)) => Some(value),
        Ok(ScriptValue::None) => None,
        Ok(other) => {
            log::error!(
                "template switch '{}' lambda must return a boolean, got {:?}",
                key,
                other
            );
            None
        }
        Err(e) => {
            log::error!("template switch '{}' lambda failed: {}", key, e);
            None
        }
    }
}

/// Evaluates an entity's `lambda`, logging and returning `None` on error or
/// on a return type mismatch. `ScriptValue::None` (the lambda returned
/// nothing, or `null`) also maps to `None`: skip this round's update, same as
/// the standalone `lambda` sensor platform.
async fn eval_number_lambda(script: &ScriptEngine, key: &str, lambda: &str) -> Option<f32> {
    match script.eval(lambda, None).await {
        Ok(ScriptValue::Number(value)) => Some(value),
        Ok(ScriptValue::None) => None,
        Ok(other) => {
            log::error!(
                "template number '{}' lambda must return a number, got {:?}",
                key,
                other
            );
            None
        }
        Err(e) => {
            log::error!("template number '{}' lambda failed: {}", key, e);
            None
        }
    }
}

/// Spawn the runtime handler for template switches. It listens for
/// [`PublishedMessage::SwitchStateCommand`] targeting a template switch, runs
/// the matching `turn_on_action`/`turn_off_action`, and (when `optimistic`,
/// or via a `lambda`) publishes the new state back onto the bus so connected
/// front-ends update.
///
/// Template switches are not platform modules, so nothing else records their
/// state into the central [`StateStore`](ubihome_core::state::StateStore) the
/// way the runtime's internal command router does for every other component
/// (see `crate::commands::run`). Every `SwitchStateChange` published here is
/// therefore mirrored into `state_writer` directly, so a client that connects
/// after the fact (e.g. `api`'s `SubscribeStatesRequest`) still observes the
/// current state instead of nothing.
fn spawn_switches(
    tasks: &mut JoinSet<()>,
    switches: Vec<TemplateSwitchConfig>,
    tx: Sender<PublishedMessage>,
    globals: Globals,
    state_writer: StateStoreWriter,
    script: ScriptEngine,
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
        // Publish the initial state of every switch with a `lambda`.
        for (key, switch) in &switches {
            if let Some(lambda) = &switch.lambda {
                if let Some(state) = eval_bool_lambda(&script, key, lambda).await {
                    log::debug!("template switch '{}' initial state: {}", key, state);
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
                                if let Some(trigger) = actions {
                                    run_actions(trigger.then, &tx, &globals, &script, None).await;
                                }
                                // With a `lambda` the state follows the lambda
                                // (re-evaluated below on every global change), so
                                // only optimistic switches without one echo the
                                // command.
                                if switch.optimistic && switch.lambda.is_none() {
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
                    // A lagged receiver just misses some updates; a global's
                    // lambdas don't need re-evaluating (nothing changed from
                    // this switch's perspective), so just keep going.
                    if changed.is_ok() {
                        for (key, switch) in &switches {
                            if let Some(lambda) = &switch.lambda {
                                if let Some(state) = eval_bool_lambda(&script, key, lambda).await {
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

/// Spawn the runtime handler for template buttons. It listens for
/// [`PublishedMessage::ButtonPressed`] targeting a template button and runs
/// its `on_press` actions. Buttons are stateless (there is no
/// [`EntityState`] variant for them), so unlike template switches this needs
/// no `StateStoreWriter`.
fn spawn_buttons(
    tasks: &mut JoinSet<()>,
    buttons: Vec<TemplateButtonConfig>,
    tx: Sender<PublishedMessage>,
    globals: Globals,
    script: ScriptEngine,
) {
    if buttons.is_empty() {
        return;
    }
    let buttons: HashMap<String, TemplateButtonConfig> = buttons
        .into_iter()
        .map(|b| (b.get_object_id(), b))
        .collect();
    let mut receiver: Receiver<PublishedMessage> = tx.subscribe();
    tasks.spawn(async move {
        loop {
            match receiver.recv().await {
                Ok(PublishedMessage::ButtonPressed { key }) => {
                    if let Some(button) = buttons.get(&key) {
                        if let Some(trigger) = button.on_press.clone() {
                            run_actions(trigger.then, &tx, &globals, &script, None).await;
                        }
                    }
                }
                // Ignore other messages; stop only when the bus closes.
                Ok(_) => {}
                Err(_) => break,
            }
        }
    });
}

/// Spawn the runtime handler for template numbers. It listens for
/// [`PublishedMessage::NumberValueCommand`] targeting a template number, runs
/// `set_action` (with the commanded value available as `x` in `lambda`
/// actions), and (when `optimistic`, or via a `lambda`) publishes the new
/// state back onto the bus so connected front-ends update.
///
/// Like template switches, template numbers are not platform modules, so
/// every value published here is mirrored into `state_writer` directly (see
/// [`spawn_switches`] for why).
fn spawn_numbers(
    tasks: &mut JoinSet<()>,
    numbers: Vec<TemplateNumberConfig>,
    tx: Sender<PublishedMessage>,
    globals: Globals,
    state_writer: StateStoreWriter,
    script: ScriptEngine,
) {
    if numbers.is_empty() {
        return;
    }
    let numbers: HashMap<String, TemplateNumberConfig> = numbers
        .into_iter()
        .map(|n| (n.get_object_id(), n))
        .collect();
    let mut receiver: Receiver<PublishedMessage> = tx.subscribe();
    let mut global_changes = globals.subscribe();
    tasks.spawn(async move {
        // Publish the initial value: from the `lambda` if configured,
        // otherwise `initial_value` (defaulting to `min_value`).
        for (key, number) in &numbers {
            let initial = match &number.lambda {
                Some(lambda) => eval_number_lambda(&script, key, lambda).await,
                None => Some(number.initial()),
            };
            if let Some(value) = initial {
                log::debug!("template number '{}' initial value: {}", key, value);
                state_writer.set(key.clone(), EntityState::Number(value));
                let _ = tx.send(PublishedMessage::NumberValueChanged {
                    key: key.clone(),
                    value,
                });
            }
        }

        loop {
            tokio::select! {
                msg = receiver.recv() => {
                    match msg {
                        Ok(PublishedMessage::NumberValueCommand { key, value }) => {
                            if let Some(number) = numbers.get(&key) {
                                if let Some(trigger) = number.set_action.clone() {
                                    run_actions(trigger.then, &tx, &globals, &script, Some(value as f64)).await;
                                }
                                // With a `lambda` the state follows the lambda
                                // (re-evaluated below on every global change), so
                                // only optimistic numbers without one echo the
                                // command.
                                if number.lambda.is_none() && number.optimistic {
                                    state_writer.set(key.clone(), EntityState::Number(value));
                                    let _ = tx.send(PublishedMessage::NumberValueChanged {
                                        key,
                                        value,
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
                    // A lagged receiver just misses some updates; keep going.
                    if changed.is_ok() {
                        for (key, number) in &numbers {
                            if let Some(lambda) = &number.lambda {
                                if let Some(value) = eval_number_lambda(&script, key, lambda).await {
                                    state_writer.set(key.clone(), EntityState::Number(value));
                                    let _ = tx.send(PublishedMessage::NumberValueChanged {
                                        key: key.clone(),
                                        value,
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
