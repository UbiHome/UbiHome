#[macro_export]
macro_rules! template_mapper {
    ($mapper_name:ident, $component_name:ident, $component_type:ident) => {
        fn $mapper_name<'de, D>(de: D) -> Result<Option<HashMap<String, $component_type>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::IntoDeserializer;

            fn platform_from_value(
                value: &::ubihome_core::serde_value::Value,
            ) -> Result<&str, String> {
                match value {
                    ::ubihome_core::serde_value::Value::Map(entries) => {
                        for (key, val) in entries {
                            if let ::ubihome_core::serde_value::Value::String(k) = key {
                                if k == "platform" {
                                    if let ::ubihome_core::serde_value::Value::String(platform) =
                                        val
                                    {
                                        return Ok(platform.as_str());
                                    }
                                    return Err("Field 'platform' must be a string".to_string());
                                }
                            }
                        }
                        Err("Missing required field 'platform'".to_string())
                    }
                    _ => Err("Expected a mapping item in sequence".to_string()),
                }
            }

            let raw_items = Vec::<::ubihome_core::serde_value::Value>::deserialize(de)?;
            let mut map = HashMap::with_capacity(raw_items.len());

            for raw_item in raw_items {
                let platform = platform_from_value(&raw_item).map_err(serde::de::Error::custom)?;

                debug!(
                    "Mapper '{}' checking: {} == {}",
                    stringify!($mapper_name),
                    platform,
                    stringify!($component_name)
                );

                if platform != stringify!($component_name) {
                    continue;
                }

                match <$component_type>::deserialize(raw_item.into_deserializer()) {
                    Ok(item) => {
                        debug!(
                            "Mapper '{}' accepted target platform item: {}",
                            stringify!($mapper_name),
                            item.platform.as_str()
                        );
                        debug!("is_configured: {}", item.is_configured());
                        if item.is_configured() == false {
                            continue;
                        }
                        if item.platform.as_str() == stringify!($component_name) {
                            let key = item.get_object_id();
                            match map.entry(key) {
                                std::collections::hash_map::Entry::Occupied(entry) => {
                                    return Err(serde::de::Error::custom(format!(
                                        "Duplicate entry {}",
                                        entry.key()
                                    )));
                                }
                                std::collections::hash_map::Entry::Vacant(entry) => {
                                    debug!("Added: {}", item.platform.as_str());
                                    entry.insert(item);
                                }
                            };
                        }
                    }
                    Err(err) => {
                        return Err(serde::de::Error::custom(format!(
                            "Invalid configuration for '{}': {}",
                            stringify!($component_name),
                            err
                        )));
                    }
                }
            }

            Ok(Some(map))
        }
    };
}
