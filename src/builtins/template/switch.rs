use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::template_switch;
use ubihome_core::with_base_entity_properties;

use crate::builtins::script;

template_switch! {
    /// Configuration of a `switch` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template switch
    /// (<https://esphome.io/components/switch/template/>). The reported
    /// state either follows a `lambda` or is echoed optimistically after a
    /// command; without either it is driven purely by the `turn_on_action` /
    /// `turn_off_action` automations.
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct TemplateSwitchConfig {
        /// When true the switch immediately publishes its new state after a
        /// command (there is no external state feedback). Defaults to true,
        /// matching the common no-feedback template switch use case.
        #[serde(default = "default_true")]
        pub optimistic: bool,

        /// Whether the switch state must be assumed. Defaults to the value of
        /// `optimistic`.
        pub assumed_state: Option<bool>,

        /// Optional inline JavaScript lambda that computes the reported
        /// state, e.g. `lambda: |- return id(my_flag)` to read a
        /// [`globals`](crate::builtins::globals) variable. Re-evaluated
        /// whenever any global changes. When set, the reported state tracks
        /// the lambda's result instead of the optimistic command state.
        #[garde(custom(script::validate_lambda))]
        pub lambda: Option<String>,

        /// Actions run when the switch is turned on.
        #[serde(default, deserialize_with = "ubihome_core::configuration::automation::deserialize_option_map_only")]
        #[garde(dive)]
        pub turn_on_action: Option<Trigger>,

        /// Actions run when the switch is turned off.
        #[serde(default, deserialize_with = "ubihome_core::configuration::automation::deserialize_option_map_only")]
        #[garde(dive)]
        pub turn_off_action: Option<Trigger>,
    }
}

fn default_true() -> bool {
    true
}

impl TemplateSwitchConfig {
    /// Whether Home Assistant should treat the switch state as assumed. A
    /// `lambda` makes the state known, so it is not assumed.
    pub fn is_assumed_state(&self) -> bool {
        self.assumed_state
            .unwrap_or(self.optimistic && self.lambda.is_none())
    }
}
