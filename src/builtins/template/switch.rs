use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::template_switch;
use ubihome_core::with_base_entity_properties;

use crate::builtins::template::lambda::LambdaExpr;

template_switch! {
    /// Configuration of a `switch` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template switch
    /// (<https://esphome.io/components/switch/template/>). Lambdas are not
    /// supported; the switch is driven purely by the `turn_on_action` /
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

        /// Optional YAML "lambda" that sources the switch state from a global,
        /// e.g. `lambda: { globals.get: my_flag }`. When set, the reported
        /// state tracks that global instead of the optimistic command state.
        #[garde(dive)]
        pub lambda: Option<LambdaExpr>,

        /// Actions run when the switch is turned on.
        #[garde(dive)]
        pub turn_on_action: Option<Trigger>,

        /// Actions run when the switch is turned off.
        #[garde(dive)]
        pub turn_off_action: Option<Trigger>,
    }
}

fn default_true() -> bool {
    true
}

impl TemplateSwitchConfig {
    /// Whether Home Assistant should treat the switch state as assumed. A
    /// `globals.get` lambda makes the state known, so it is not assumed.
    pub fn is_assumed_state(&self) -> bool {
        self.assumed_state
            .unwrap_or(self.optimistic && self.lambda.is_none())
    }

    /// The id of the global this switch reads its state from, if a
    /// `globals.get` lambda is configured.
    pub fn state_global(&self) -> Option<&str> {
        self.lambda.as_ref().map(LambdaExpr::global_id)
    }
}
