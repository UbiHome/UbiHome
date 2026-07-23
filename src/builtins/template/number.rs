use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::template_number;
use ubihome_core::with_base_entity_properties;

use crate::builtins::template::lambda::LambdaExpr;

template_number! {
    /// Configuration of a `number` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template number
    /// (<https://esphome.io/components/number/template/>). The reported
    /// value either follows a `globals.get` `lambda` or is echoed
    /// optimistically after a command, and `set_action` runs as a plain
    /// action list (it has no access to the commanded value - use a `lambda`
    /// to have the entity's own state reflect it instead).
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct TemplateNumberConfig {
        /// When true the number immediately publishes the commanded value as
        /// its new state (there is no external state feedback). Defaults to
        /// false, matching ESPHome. Not used together with `lambda`.
        #[serde(default)]
        pub optimistic: bool,

        /// The value to report on startup when not driven by a `lambda`.
        /// Defaults to `min_value`.
        pub initial_value: Option<f32>,

        /// Optional YAML "lambda" that sources the reported value from a
        /// `float` [`globals`](crate::builtins::globals) variable, e.g.
        /// `lambda: { globals.get: my_value }`. When set, the reported state
        /// tracks that global instead of the optimistic command value.
        #[garde(dive)]
        pub lambda: Option<LambdaExpr>,

        /// Actions run when a client (e.g. Home Assistant) sets a new value.
        /// Runs before the new state is published/stored.
        #[garde(dive)]
        pub set_action: Option<Trigger>,

        /// Accepted for compatibility with ESPHome. Persisting values across
        /// restarts is not implemented yet, so this currently has no effect.
        #[allow(dead_code)]
        #[serde(default)]
        pub restore_value: bool,

        /// Accepted for compatibility with ESPHome, where it controls how
        /// often a `lambda` is re-evaluated. This project's `lambda` is
        /// push-based (it updates immediately when the backing global
        /// changes), so polling is not needed and this has no effect.
        #[allow(dead_code)]
        pub update_interval: Option<String>,
    }
}

impl TemplateNumberConfig {
    /// The value to report on startup, when not driven by a `lambda`.
    pub fn initial(&self) -> f32 {
        self.initial_value.unwrap_or(self.min_value.unwrap_or(0.0))
    }

    /// The id of the global this number reads its state from, if a
    /// `globals.get` lambda is configured.
    pub fn state_global(&self) -> Option<&str> {
        self.lambda.as_ref().map(LambdaExpr::global_id)
    }
}
