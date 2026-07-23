use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::template_button;
use ubihome_core::with_base_entity_properties;

template_button! {
    /// Configuration of a `button` entry with `platform: template`.
    ///
    /// Mirrors a subset of the ESPHome template button
    /// (<https://esphome.io/components/button/template/>). Pressing the
    /// button (from the API or MQTT) runs `on_press`.
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct TemplateButtonConfig {
        /// Actions run when the button is pressed.
        #[garde(dive)]
        pub on_press: Option<Trigger>,
    }
}
