use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::template_number;
use ubihome_core::with_base_entity_properties;

use crate::builtins::script;

template_number! {
    /// Configuration of a `number` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template number
    /// (<https://esphome.io/components/number/template/>). The reported
    /// value either follows a `lambda` or is echoed optimistically after a
    /// command. `set_action` runs as a normal action list with the commanded
    /// value available as `x` in `lambda` actions (see
    /// [Triggers and Actions](crate::builtins)).
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

        /// Optional inline JavaScript lambda that computes the reported
        /// value, e.g. `lambda: |- return id(my_value)` to read a
        /// [`globals`](crate::builtins::globals) variable. Re-evaluated
        /// whenever any global changes. When set, the reported state tracks
        /// the lambda's result instead of the optimistic command value.
        #[garde(custom(script::validate_lambda))]
        pub lambda: Option<String>,

        /// Actions run when a client (e.g. Home Assistant) sets a new value.
        /// Runs before the new state is published/stored; a `lambda` action
        /// here sees the commanded value as `x`.
        #[serde(default, deserialize_with = "ubihome_core::configuration::automation::deserialize_option_map_only")]
        #[garde(dive)]
        pub set_action: Option<Trigger>,

        /// Accepted for compatibility with ESPHome. Persisting values across
        /// restarts is not implemented yet, so this currently has no effect.
        #[allow(dead_code)]
        #[serde(default)]
        pub restore_value: bool,

        /// Accepted for compatibility with ESPHome, where it controls how
        /// often a `lambda` is re-evaluated. This project's `lambda` is
        /// push-based (it updates immediately when a global changes), so
        /// polling is not needed and this has no effect.
        #[allow(dead_code)]
        pub update_interval: Option<String>,
    }
}

impl TemplateNumberConfig {
    /// The value to report on startup, when not driven by a `lambda`.
    pub fn initial(&self) -> f32 {
        self.initial_value.unwrap_or(self.min_value.unwrap_or(0.0))
    }
}
