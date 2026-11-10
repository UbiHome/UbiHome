use duration_str::deserialize_duration;
use garde::Validate;
use std::time::Duration;

use serde::Deserialize;
use serde::Serialize;

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(rename_all = "snake_case")]
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

// Actions and triggers are shared across components (binary sensors, template
// switches, ...) and therefore live in the `automation` module. They are
// re-exported here for backwards compatibility with existing imports.
pub use crate::configuration::automation::{Action, ActionType, Trigger};
