use duration_str::deserialize_duration;
use garde::Validate;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(rename_all = "lowercase")]
pub enum FilterType {
    Invert(#[garde(required)] Option<String>),

    #[serde(deserialize_with = "deserialize_duration")]
    DelayedOff(#[garde(skip)] Duration),

    #[serde(deserialize_with = "deserialize_duration")]
    DelayedOn(#[garde(skip)] Duration),
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct BinarySensorFilter {
    #[serde(flatten)]
    #[garde(skip)]
    pub filter: FilterType,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
pub enum ActionType {
    #[serde(rename = "switch.turn_on")]
    SwitchTurnOn(#[garde(ascii)] String),

    #[serde(rename = "switch.turn_off")]
    SwitchTurnOff(#[garde(ascii)] String),
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Action {
    #[serde(flatten)]
    #[garde(skip)]
    pub action: ActionType,
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]

pub struct Trigger {
    #[garde(dive)]
    pub then: Vec<Action>,
}
