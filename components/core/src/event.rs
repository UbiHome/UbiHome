use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub struct EventBase {
    pub id: Option<String>,
    pub icon: Option<String>,
    pub name: String,
}

impl EventBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}


#[derive(Clone, Deserialize, Debug)]
pub struct UnknownEvent{}

#[macro_export]
macro_rules! template_event {
    ($component_name:ident, $event_extension:ident) => {
        use $crate::event::EventBase;
        use $crate::event::UnknownEvent;

        #[allow(non_camel_case_types)]
        #[derive(Clone, Deserialize, Debug)]
        #[serde(tag = "platform")]
        #[serde(rename_all = "camelCase")]
        pub enum EventKind {
            $component_name($event_extension),
            #[serde(untagged)]
            Unknown(UnknownEvent),
        }

        #[derive(Clone, Deserialize, Debug)]
        pub struct EventConfig {
            #[serde(flatten)]
            pub default: EventBase,

            #[serde(flatten)]
            pub extra: EventKind,
        }
    }
}