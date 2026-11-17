use std::fmt;
use std::marker::PhantomData;

use garde::Validate;
use serde::de::{self, DeserializeOwned, DeserializeSeed, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer};
use serde_saphyr::{MessageFormatter, UserMessageFormatter};

use crate::UbiHome;
use crate::platform::{EntityConfig, PlatformConfigTypes, PlatformParts, visit_platform_map};

/// Deserialize and validate a whole configuration file into `T` in a single
/// pass, rendering any error as a miette report with source locations.
pub fn validate_config<T: Validate + DeserializeOwned>(
    config_string: &str,
    config_path: &str,
) -> Result<T, String>
where
    <T as Validate>::Context: Default,
{
    let no_snippet = serde_saphyr::options! { with_snippet: false };
    let validation_result =
        serde_saphyr::from_str_with_options_valid::<T>(config_string, no_snippet);

    match validation_result {
        Ok(config) => Ok(config),
        Err(error) => {
            let formatter: &dyn MessageFormatter = &UserMessageFormatter;

            let report = serde_saphyr::miette::to_miette_report_with_formatter(
                &error,
                config_string,
                config_path,
                formatter,
            );

            Err(format!("{:?}", report))
        }
    }
}

/// Parse a configuration file into the config of a single platform `C`,
/// ignoring every other platform's sections and entities.
///
/// The application parses all platforms at once through its generated root
/// config; this standalone variant exists so a platform crate can build its
/// `CoreConfig` from YAML on its own (e.g. in unit tests).
pub fn parse_platform_config<C: PlatformConfigTypes>(config_string: &str) -> Result<C, String> {
    let root = validate_config::<SinglePlatformRoot<C>>(config_string, "config.yml")?;
    Ok(C::from_parts(PlatformParts {
        ubihome: root.ubihome,
        section: root.section,
        button: root.button.into_iter().map(|(_, e)| e).collect(),
        binary_sensor: root.binary_sensor.into_iter().map(|(_, e)| e).collect(),
        sensor: root.sensor.into_iter().map(|(_, e)| e).collect(),
        switch: root.switch.into_iter().map(|(_, e)| e).collect(),
        light: root.light.into_iter().map(|(_, e)| e).collect(),
        number: root.number.into_iter().map(|(_, e)| e).collect(),
        text_sensor: root.text_sensor.into_iter().map(|(_, e)| e).collect(),
        media_player: root.media_player.into_iter().map(|(_, e)| e).collect(),
    }))
}

/// Entities of one kind belonging to the platform, with their index in the
/// YAML list (for error paths).
type Indexed<T> = Vec<(usize, T)>;

struct SinglePlatformRoot<C: PlatformConfigTypes> {
    ubihome: UbiHome,
    section: C::Section,
    button: Indexed<C::Button>,
    binary_sensor: Indexed<C::BinarySensor>,
    sensor: Indexed<C::Sensor>,
    switch: Indexed<C::Switch>,
    light: Indexed<C::Light>,
    number: Indexed<C::Number>,
    text_sensor: Indexed<C::TextSensor>,
    media_player: Indexed<C::MediaPlayer>,
}

impl<'de, C: PlatformConfigTypes> Deserialize<'de> for SinglePlatformRoot<C> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct RootVisitor<C>(PhantomData<C>);

        impl<'de, C: PlatformConfigTypes> Visitor<'de> for RootVisitor<C> {
            type Value = SinglePlatformRoot<C>;

            fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                f.write_str("a configuration mapping")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut ubihome = None;
                let mut section = None;
                let mut button = Vec::new();
                let mut binary_sensor = Vec::new();
                let mut sensor = Vec::new();
                let mut switch = Vec::new();
                let mut light = Vec::new();
                let mut number = Vec::new();
                let mut text_sensor = Vec::new();
                let mut media_player = Vec::new();

                let name = C::NAME;
                while let Some(key) = map.next_key::<String>()? {
                    match key.as_str() {
                        "ubihome" => ubihome = Some(map.next_value()?),
                        "button" => button = map.next_value_seed(OwnEntities::new(name))?,
                        "binary_sensor" => {
                            binary_sensor = map.next_value_seed(OwnEntities::new(name))?
                        }
                        "sensor" => sensor = map.next_value_seed(OwnEntities::new(name))?,
                        "switch" => switch = map.next_value_seed(OwnEntities::new(name))?,
                        "light" => light = map.next_value_seed(OwnEntities::new(name))?,
                        "number" => number = map.next_value_seed(OwnEntities::new(name))?,
                        "text_sensor" => {
                            text_sensor = map.next_value_seed(OwnEntities::new(name))?
                        }
                        "media_player" => {
                            media_player = map.next_value_seed(OwnEntities::new(name))?
                        }
                        other if other == name => section = Some(map.next_value()?),
                        _ => {
                            map.next_value::<de::IgnoredAny>()?;
                        }
                    }
                }

                let section = match section {
                    Some(section) => section,
                    // Lets optional sections (`Option<...>`) default to `None`.
                    None => C::Section::deserialize(de::value::UnitDeserializer::<A::Error>::new())
                        .map_err(|_| de::Error::missing_field(name))?,
                };

                Ok(SinglePlatformRoot {
                    ubihome: ubihome.ok_or_else(|| de::Error::missing_field("ubihome"))?,
                    section,
                    button,
                    binary_sensor,
                    sensor,
                    switch,
                    light,
                    number,
                    text_sensor,
                    media_player,
                })
            }
        }

        deserializer.deserialize_map(RootVisitor(PhantomData))
    }
}

/// Deserializes an entity list, keeping only the entries of one platform.
struct OwnEntities<T> {
    platform: &'static str,
    marker: PhantomData<T>,
}

impl<T> OwnEntities<T> {
    fn new(platform: &'static str) -> Self {
        OwnEntities {
            platform,
            marker: PhantomData,
        }
    }
}

impl<'de, T: EntityConfig> DeserializeSeed<'de> for OwnEntities<T> {
    type Value = Indexed<T>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_any(self)
    }
}

impl<'de, T: EntityConfig> Visitor<'de> for OwnEntities<T> {
    type Value = Indexed<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("a list of entities")
    }

    fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Vec::new())
    }

    fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
        Ok(Vec::new())
    }

    fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
        let mut entities = Vec::new();
        let mut index = 0;
        while let Some(entity) = seq.next_element_seed(OwnEntity::<T>::new(self.platform))? {
            if let Some(entity) = entity {
                entities.push((index, entity));
            }
            index += 1;
        }
        Ok(entities)
    }
}

struct OwnEntity<T> {
    platform: &'static str,
    marker: PhantomData<T>,
}

impl<T> OwnEntity<T> {
    fn new(platform: &'static str) -> Self {
        OwnEntity {
            platform,
            marker: PhantomData,
        }
    }
}

impl<'de, T: EntityConfig> DeserializeSeed<'de> for OwnEntity<T> {
    type Value = Option<T>;

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)
    }
}

impl<'de, T: EntityConfig> Visitor<'de> for OwnEntity<T> {
    type Value = Option<T>;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("an entity mapping with a `platform` key")
    }

    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
        visit_platform_map(map, |platform, rest| {
            if platform == self.platform {
                T::deserialize(de::value::MapAccessDeserializer::new(rest)).map(Some)
            } else {
                rest.drain().map(|_| None)
            }
        })
    }
}

impl<C: PlatformConfigTypes> Validate for SinglePlatformRoot<C> {
    type Context = ();

    fn validate_into(
        &self,
        ctx: &(),
        parent: &mut dyn FnMut() -> garde::Path,
        report: &mut garde::Report,
    ) {
        self.ubihome
            .validate_into(ctx, &mut || parent().join("ubihome"), report);
        self.section
            .validate_into(ctx, &mut || parent().join(C::NAME), report);

        fn entities<T: EntityConfig>(
            kind: &'static str,
            entities: &Indexed<T>,
            parent: &mut dyn FnMut() -> garde::Path,
            report: &mut garde::Report,
        ) {
            for (index, entity) in entities {
                if !T::SUPPORTED {
                    report.append(
                        parent().join(kind).join(*index).join("platform"),
                        garde::Error::new(format!(
                            "Platform '{}' does not provide '{}' entities",
                            entity.platform(),
                            kind
                        )),
                    );
                    continue;
                }
                entity.validate_into(&(), &mut || parent().join(kind).join(*index), report);
            }
        }
        entities("button", &self.button, parent, report);
        entities("binary_sensor", &self.binary_sensor, parent, report);
        entities("sensor", &self.sensor, parent, report);
        entities("switch", &self.switch, parent, report);
        entities("light", &self.light, parent, report);
        entities("number", &self.number, parent, report);
        entities("text_sensor", &self.text_sensor, parent, report);
        entities("media_player", &self.media_player, parent, report);
    }
}
