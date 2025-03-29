use std::collections::HashMap;
use duration_str::deserialize_option_duration;
use std::time::Duration;


use serde::{Deserialize, Deserializer};

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonConfig {
    pub platform: String,
    pub id: Option<String>,
    pub name: String,
    pub command: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Sensor {
    // pub platform: String,
    pub id: Option<String>,
    pub name: String,
    pub icon: String,
    pub device_class: Option<String>,
    pub unit_of_measurement: String,
    pub state_class: String,
    #[serde(deserialize_with = "deserialize_option_duration")]
    pub update_interval: Option<Duration>,

    #[serde(flatten)]
    pub kind: SensorKind,
}

#[derive(Clone, Deserialize, Debug)]
#[serde(tag = "platform")]
#[serde(rename_all = "camelCase")]
pub enum SensorKind {
    Shell(ShellSensorConfig)
}

#[derive(Clone, Deserialize, Debug)]
pub struct ShellSensorConfig {
    pub command: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct CoreConfig {
    pub oshome: OSHome,
    #[serde(deserialize_with = "map_button")] 
    pub button: Option<HashMap<String, ButtonConfig>>,

    #[serde(deserialize_with = "map_sensor")] 
    pub sensor: Option<HashMap<String, Sensor>>,
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

        fn visit_seq<V>(self, mut seq: V) -> Result<Option<HashMap<String, ButtonConfig>>, V::Error>
        where
            V: SeqAccess<'de>,
        {
            let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));

            while let Some(item) = seq.next_element::<ButtonConfig>()? {
                let ButtonConfig {
                    platform,
                    id,
                    name,
                    command,
                } = item;
                let key = id.clone().unwrap_or(name.clone());
                match map.entry(key) {
                    std::collections::hash_map::Entry::Occupied(entry) => {
                        return Err(serde::de::Error::custom(format!(
                            "Duplicate entry {}",
                            entry.key()
                        )))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(ButtonConfig { platform, id, name, command })
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

        fn visit_seq<V>(self, mut seq: V) -> Result<Option<HashMap<String, Sensor>>, V::Error>
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
                        )))
                    }
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(item)
                    }
                };
            }
            Ok(Some(map))
        }
    }

    de.deserialize_seq(ItemsVisitor)
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPress {
        key: String,
    },
    SensorValueChange {
        key: String,
        value: String,
    },
}