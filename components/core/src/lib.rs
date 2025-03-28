use std::collections::HashMap;

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

#[derive(Clone, Deserialize, Debug)]
pub struct CoreConfig {
    pub oshome: OSHome,
    #[serde(deserialize_with = "mappify")] 
    pub button: Option<HashMap<String, ButtonConfig>>
}

fn mappify<'de, D>(de: D) -> Result<Option<HashMap<String, ButtonConfig>>, D::Error>
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

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPress {
        key: String,
    },
}