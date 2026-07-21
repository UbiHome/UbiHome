use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Action;
use ubihome_core::utils::format_id;

use crate::builtins::lambda::LambdaExpr;

/// Configuration of a `number` entry with `platform: template`.
///
/// Mirrors a subset of the ESPHome template number
/// (<https://esphome.io/components/number/template/>). C++ lambdas are not
/// supported: the reported value either follows a `globals.get` `lambda` or is
/// echoed optimistically after a command, and `set_action` runs as a plain
/// action list (it has no access to the commanded value - use a `lambda` to
/// have the entity's own state reflect it instead).
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TemplateNumberConfig {
    #[garde(length(min = 3, max = 100))]
    pub id: Option<String>,

    #[garde(length(min = 3, max = 100))]
    pub name: String,

    // Required so `deny_unknown_fields` accepts `platform: template`; the value
    // itself is only used to select this parser upstream.
    #[allow(dead_code)]
    #[garde(skip)]
    pub platform: String,

    #[garde(ascii)]
    pub icon: Option<String>,

    #[garde(ascii)]
    pub device_class: Option<String>,

    #[garde(ascii)]
    pub unit_of_measurement: Option<String>,

    #[garde(skip)]
    pub min_value: f32,

    #[garde(skip)]
    pub max_value: f32,

    #[garde(skip)]
    pub step: f32,

    /// When true the number immediately publishes the commanded value as its
    /// new state (there is no external state feedback). Defaults to false,
    /// matching ESPHome. Not used together with `lambda`.
    #[serde(default)]
    #[garde(skip)]
    pub optimistic: bool,

    /// Accepted for compatibility with ESPHome. Persisting values across
    /// restarts is not implemented yet, so this currently has no effect.
    #[allow(dead_code)]
    #[serde(default)]
    #[garde(skip)]
    pub restore_value: bool,

    /// The value to report on startup when not driven by a `lambda`. Defaults
    /// to `min_value`.
    #[serde(default)]
    #[garde(skip)]
    pub initial_value: Option<f32>,

    /// Optional YAML "lambda" that sources the reported value from a `float`
    /// [`globals`](crate::builtins::globals) variable, e.g.
    /// `lambda: { globals.get: my_value }`. When set, the reported state
    /// tracks that global instead of the optimistic command value.
    #[serde(default)]
    #[garde(dive)]
    pub lambda: Option<LambdaExpr>,

    /// Actions run when a client (e.g. Home Assistant) sets a new value. Runs
    /// before the new state is published/stored.
    #[serde(default)]
    #[garde(dive)]
    pub set_action: Option<Vec<Action>>,

    /// Accepted for compatibility with ESPHome, where it controls how often a
    /// `lambda` is re-evaluated. This project's `lambda` is push-based (it
    /// updates immediately when the backing global changes), so polling is
    /// not needed and this has no effect.
    #[allow(dead_code)]
    #[serde(default)]
    #[garde(skip)]
    pub update_interval: Option<String>,
}

impl TemplateNumberConfig {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &Some(self.name.clone()))
    }

    /// The value to report on startup, when not driven by a `lambda`.
    pub fn initial(&self) -> f32 {
        self.initial_value.unwrap_or(self.min_value)
    }

    /// The id of the global this number reads its state from, if a
    /// `globals.get` lambda is configured.
    pub fn state_global(&self) -> Option<&str> {
        self.lambda.as_ref().map(LambdaExpr::global_id)
    }
}
