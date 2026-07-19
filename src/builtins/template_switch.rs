use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Action;
use ubihome_core::utils::format_id;

/// A YAML-expressed state source for a template switch `lambda`. This avoids
/// C++ lambdas: the switch state is read directly from a global.
#[derive(Clone, Debug, Deserialize, Validate)]
pub enum LambdaExpr {
    /// Report the switch state from a `bool` [`globals`] variable.
    #[serde(rename = "globals.get")]
    GlobalsGet(#[garde(ascii)] String),
}

/// Configuration of a `switch` entry with `platform: template`.
///
/// Mirrors a subset of the ESPHome template switch
/// (<https://esphome.io/components/switch/template/>). Lambdas are not
/// supported; the switch is driven purely by the `turn_on_action` /
/// `turn_off_action` automations.
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TemplateSwitchConfig {
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

    /// When true the switch immediately publishes its new state after a command
    /// (there is no external state feedback). Defaults to true, matching the
    /// common no-feedback template switch use case.
    #[serde(default = "default_true")]
    #[garde(skip)]
    pub optimistic: bool,

    /// Whether the switch state must be assumed. Defaults to the value of
    /// `optimistic`.
    #[serde(default)]
    #[garde(skip)]
    pub assumed_state: Option<bool>,

    /// Optional YAML "lambda" that sources the switch state from a global,
    /// e.g. `lambda: { globals.get: my_flag }`. When set, the reported state
    /// tracks that global instead of the optimistic command state.
    #[serde(default)]
    #[garde(dive)]
    pub lambda: Option<LambdaExpr>,

    /// Actions run when the switch is turned on.
    #[serde(default)]
    #[garde(dive)]
    pub turn_on_action: Option<Vec<Action>>,

    /// Actions run when the switch is turned off.
    #[serde(default)]
    #[garde(dive)]
    pub turn_off_action: Option<Vec<Action>>,
}

fn default_true() -> bool {
    true
}

impl TemplateSwitchConfig {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }

    /// Whether Home Assistant should treat the switch state as assumed. A
    /// `globals.get` lambda makes the state known, so it is not assumed.
    pub fn is_assumed_state(&self) -> bool {
        self.assumed_state
            .unwrap_or(self.optimistic && self.lambda.is_none())
    }

    /// The id of the global this switch reads its state from, if a
    /// `globals.get` lambda is configured.
    pub fn state_global(&self) -> Option<&str> {
        match &self.lambda {
            Some(LambdaExpr::GlobalsGet(id)) => Some(id.as_str()),
            None => None,
        }
    }
}
