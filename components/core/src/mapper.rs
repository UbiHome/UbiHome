
#[macro_export]
macro_rules! template_mapper {
    ($component_name:ident, $component_type:ident) => {
        fn $component_name<'de, D>(de: D) -> Result<Option<HashMap<String, $component_type>>, D::Error>
        where
            D: Deserializer<'de>,
        {
            use serde::de::*;
            struct ItemsVisitor;
            impl<'de> Visitor<'de> for ItemsVisitor {
                type Value = Option<HashMap<String, $component_type>>;
        
                fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                    formatter.write_str("a sequence of items")
                }
        
                fn visit_seq<V>(
                    self,
                    mut seq: V,
                ) -> Result<Option<HashMap<String, $component_type>>, V::Error>
                where
                    V: SeqAccess<'de>,
                {
                    let mut map = HashMap::with_capacity(seq.size_hint().unwrap_or(0));
        
                    while let Some(item) = seq.next_element::<$component_type>()? {
                        let key = item.default.id.clone().unwrap_or(item.default.name.clone());
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
    }
}
