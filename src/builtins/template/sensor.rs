use garde::Validate;
use serde::Deserialize;
use ubihome_core::template_sensor;
use ubihome_core::with_base_entity_properties;

use crate::builtins::template::lambda::LambdaExpr;

template_sensor! {
    /// Configuration of a `sensor` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template sensor
    /// (<https://esphome.io/components/sensor/template/>). Sensors are
    /// read-only (there is no command path from a client), so - unlike the
    /// template switch/number - the reported value always comes from a
    /// `globals.get` `lambda`.
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct TemplateSensorConfig {
        /// YAML "lambda" that sources the reported value from a `float`
        /// [`globals`](crate::builtins::globals) variable, e.g.
        /// `lambda: { globals.get: my_value }`.
        #[garde(dive)]
        pub lambda: LambdaExpr,
    }
}

impl TemplateSensorConfig {
    /// The id of the global this sensor reads its value from.
    pub fn state_global(&self) -> &str {
        self.lambda.global_id()
    }
}
