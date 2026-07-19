use garde::Validate;
use serde::Deserialize;
use serde::Serialize;

/// A single automation action. New action types are added as variants here and
/// are executed centrally by the main binary (see the runtime action executor),
/// so that actions can reference entities and globals across platforms.
#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
pub enum ActionType {
    #[serde(rename = "switch.turn_on")]
    SwitchTurnOn(#[garde(ascii)] String),

    #[serde(rename = "switch.turn_off")]
    SwitchTurnOff(#[garde(ascii)] String),

    #[serde(rename = "button.press")]
    ButtonPress(#[garde(ascii)] String),

    /// Sets the value of a [`globals`] variable.
    #[serde(rename = "globals.set")]
    GlobalsSet {
        #[garde(ascii)]
        id: String,
        #[garde(skip)]
        value: String,
    },
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(flatten)]
    #[garde(skip)]
    pub action: ActionType,
}

/// A trigger runs its list of actions (in order) when the owning component
/// fires it (for example a binary sensor `on_press`).
#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    #[garde(dive)]
    pub then: Vec<Action>,
}
