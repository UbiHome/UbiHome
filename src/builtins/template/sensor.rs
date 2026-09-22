use garde::Validate;
use serde::Deserialize;
use ubihome_core::template_sensor;
use ubihome_core::with_base_entity_properties;

use crate::builtins::script;

template_sensor! {
    /// Configuration of a `sensor` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template sensor
    /// (<https://esphome.io/components/sensor/template/>). Sensors are
    /// read-only (there is no command path from a client), so the reported
    /// value always comes from the `lambda`, re-evaluated whenever any
    /// global changes.
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct TemplateSensorConfig {
        /// Inline JavaScript lambda that computes the reported value, e.g.
        /// `lambda: |- return id(my_value)` to read a
        /// [`globals`](crate::builtins::globals) variable.
        #[garde(custom(script::validate_lambda))]
        pub lambda: Option<String>,
    }
}
