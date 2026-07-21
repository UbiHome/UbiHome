use garde::Validate;
use serde::Deserialize;
use ubihome_core::configuration::automation::Action;
use ubihome_core::utils::format_id;

/// Configuration of a `button` entry with `platform: template`.
///
/// Mirrors a subset of the ESPHome template button
/// (<https://esphome.io/components/button/template/>). Pressing the button
/// (from the API or MQTT) runs `on_press`.
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct TemplateButtonConfig {
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

    /// Actions run when the button is pressed.
    #[serde(default)]
    #[garde(dive)]
    pub on_press: Option<Vec<Action>>,
}

impl TemplateButtonConfig {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &Some(self.name.clone()))
    }
}
