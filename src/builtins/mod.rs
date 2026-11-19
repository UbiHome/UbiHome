//! Builtin components that are compiled directly into the main binary instead
//! of living in a `ubihome-*` platform crate.
//!
//! Template switches/buttons/numbers ([`template`]) and globals ([`globals`])
//! are handled here because they are tightly integrated with the central
//! runtime: their automations reference entities (switches, buttons) and
//! globals that belong to *other* platforms, and those cross-cutting
//! references are only resolvable on the central message bus in
//! [`crate::commands::run`]. A standalone platform crate only sees its own
//! entities, so it could not drive them. See `documentation` for the rationale.

pub mod globals;
pub mod template;

use tokio::sync::broadcast::Sender;
use ubihome_core::configuration::automation::{Action, ActionType};
use ubihome_core::PublishedMessage;

pub use globals::{GlobalConfig, Globals};
pub use template::TemplateConfig;

/// The switch/button/number platform name handled by the builtin template
/// components.
pub const TEMPLATE_PLATFORM: &str = "template";

/// The parsed builtin configuration.
#[derive(Debug, Default)]
pub struct BuiltinConfig {
    pub template: TemplateConfig,
    pub globals: Vec<GlobalConfig>,
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
                globals.set(id, value.clone());
            }
            ActionType::Delay(duration) => {
                tokio::time::sleep(*duration).await;
            }
            ActionType::LoggerLog(value) => {
                log::info!("{value}");
            }
        }
    }
}
