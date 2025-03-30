pub mod sensor;
pub mod button;
pub mod binary_sensor;

use std::collections::HashMap;
use binary_sensor::BinarySensor;
use button::ButtonConfig;
use sensor::Sensor;

use serde::{Deserialize, Deserializer};

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}








#[derive(Clone, Deserialize, Debug)]
pub struct CoreConfig {
    pub oshome: OSHome,
    #[serde(default, deserialize_with = "map_button")] 
    pub button: Option<HashMap<String, ButtonConfig>>,

    #[serde(default, deserialize_with = "map_sensor")] 
    pub sensor: Option<HashMap<String, Sensor>>,

    #[serde(default, deserialize_with = "map_binary_sensor")] 
    pub binary_sensor: Option<HashMap<String, BinarySensor>>,
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

fn map_binary_sensor<'de, D>(de: D) -> Result<Option<HashMap<String, BinarySensor>>, D::Error>
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

        fn visit_none<E>(self) -> Result<Self::Value, E>
            where
                E: serde::de::Error, {
            Ok(None)
        }

        fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>, {
            Ok(None)
        }

        fn visit_seq<V>(self, mut seq: V) -> Result<Option<HashMap<String, BinarySensor>>, V::Error>
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
    BinarySensorValueChange {
        key: String,
        value: bool,
    },
}