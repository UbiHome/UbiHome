pub mod binary_sensor;
pub mod home_assistant;
pub mod sensor;
pub mod button;

use serde::{Deserialize, Deserializer};
use home_assistant::sensors::Component;
use std::{collections::HashMap, pin::Pin};
use tokio::sync::broadcast::{Receiver, Sender};

pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

pub trait Module
where
    Self: Send,
{
    fn validate(&mut self) -> Result<(), String>;

    fn init(&mut self, config: &String) -> Result<Vec<Component>, String>;
    fn run(
        &self,
        sender: Sender<Option<Message>>,
        receiver: Receiver<Option<Message>>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>;
}



#[derive(Debug, Clone)]
pub enum Message {
    ButtonPress { key: String },
    SensorValueChange { key: String, value: String },
    BinarySensorValueChange { key: String, value: bool },
}





#[macro_export]
macro_rules! config_template {
    ($component_name:ident, 
        $component_config:ty, 
        $button_extension:ident, 
        $binary_sensor_extension:ident, 
        $sensor_extension:ident) => {

        use duration_str::deserialize_option_duration;
        use oshome_core::template_button;
        use oshome_core::template_binary_sensor;
        use oshome_core::template_sensor;


        template_button!($component_name, $button_extension);
        template_binary_sensor!($component_name, $binary_sensor_extension);
        template_sensor!($component_name, $sensor_extension);

        #[derive(Clone, Deserialize, Debug)]
        pub struct OSHome {
            pub name: String,
        }

        fn map_button<'de, D>(de: D) -> Result<Option<HashMap<String, ButtonConfig>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::*;
            struct ItemsVisitor;
            impl<'de> Visitor<'de> for ItemsVisitor {
                type Value = Option<HashMap<String, ButtonConfig>>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a sequence of items")
                }

                fn visit_seq<V>(
                    self,
                    mut seq: V,
                ) -> Result<Option<HashMap<String, ButtonConfig>>, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

                    while let Some(item) = seq.next_element::<ButtonConfig>()? {
                        let ButtonConfig {
                            platform,
                            id,
                            name,
                            extra,
                        } = item;
                        let key = id.clone().unwrap_or(name.clone());
                        match map.entry(key) {
                            std::collections::hash_map::Entry::Occupied(entry) => {
                                return Err(serde::de::Error::custom(format!(
                                    "Duplicate entry {}",
                                    entry.key()
                                )));
                            }
                            std::collections::hash_map::Entry::Vacant(entry) => {
                                entry.insert(ButtonConfig {
                                    platform,
                                    id,
                                    name,
                                    extra,
                                })
                            }
                        };
                    }
                    Ok(Some(map))
                }
            }

            de.deserialize_seq(ItemsVisitor)
        }

        fn map_sensor<'de, D>(de: D) -> Result<Option<HashMap<String, Sensor>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::*;
            struct ItemsVisitor;
            impl<'de> Visitor<'de> for ItemsVisitor {
                type Value = Option<HashMap<String, Sensor>>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a sequence of items")
                }

                fn visit_seq<V>(
                    self,
                    mut seq: V,
                ) -> Result<Option<HashMap<String, Sensor>>, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

                    while let Some(item) = seq.next_element::<Sensor>()? {
                        let key = item.id.clone().unwrap_or(item.name.clone());
                        match map.entry(key) {
                            std::collections::hash_map::Entry::Occupied(entry) => {
                                return Err(serde::de::Error::custom(format!(
                                    "Duplicate entry {}",
                                    entry.key()
                                )));
                            }
                            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(item),
                        };
                    }
                    Ok(Some(map))
                }
            }

            de.deserialize_seq(ItemsVisitor)
        }

        fn map_binary_sensor<'de, D>(
            de: D,
        ) -> Result<Option<HashMap<String, BinarySensor>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::*;
            struct ItemsVisitor;
            impl<'de> Visitor<'de> for ItemsVisitor {
                type Value = Option<HashMap<String, BinarySensor>>;

                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a sequence of items")
                }

                fn visit_seq<V>(
                    self,
                    mut seq: V,
                ) -> Result<Option<HashMap<String, BinarySensor>>, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

                    while let Some(item) = seq.next_element::<BinarySensor>()? {
                        let key = item.id.clone().unwrap_or(item.name.clone());
                        match map.entry(key) {
                            std::collections::hash_map::Entry::Occupied(entry) => {
                                return Err(serde::de::Error::custom(format!(
                                    "Duplicate entry {}",
                                    entry.key()
                                )));
                            }
                            std::collections::hash_map::Entry::Vacant(entry) => entry.insert(item),
                        };
                    }
                    Ok(Some(map))
                }
            }

            de.deserialize_seq(ItemsVisitor)
        }

        #[derive(Clone, Deserialize, Debug)]
        pub struct CoreConfig {
            pub oshome: OSHome,

            pub $component_name: $component_config,

            #[serde(default, deserialize_with = "map_button")]
            pub button: Option<HashMap<String, ButtonConfig>>,

            #[serde(default, deserialize_with = "map_sensor")]
            pub sensor: Option<HashMap<String, Sensor>>,

            #[serde(default, deserialize_with = "map_binary_sensor")]
            pub binary_sensor: Option<HashMap<String, BinarySensor>>,
        }
    };
}
