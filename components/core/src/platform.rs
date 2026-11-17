//! Building blocks for parsing the whole configuration file in a single pass.
//!
//! The main binary generates one root config type covering every compiled-in
//! platform crate (see `src/components.rs` in the `ubihome` crate). Each
//! platform crate only describes its config *types* (via
//! [`config_template!`](crate::config_template)) and receives an already
//! deserialized and validated [`PlatformConfigTypes`] value in
//! [`ConfigurableModule::new`] - platform crates never parse YAML themselves.

use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;
use std::marker::PhantomData;

use garde::Validate;
use serde::de::{self, DeserializeOwned, DeserializeSeed, IntoDeserializer, MapAccess};
use serde_value::Value;

use crate::configuration::automation::Trigger;
use crate::{Module, UbiHome};

/// An entity config (`button`, `switch`, ...) of one platform.
///
/// Implemented automatically by
/// [`with_base_entity_properties!`](crate::with_base_entity_properties) (and
/// therefore by every `template_*!` macro), and by [`NoConfig`](crate::NoConfig)
/// for entity kinds a platform does not provide.
pub trait EntityConfig: DeserializeOwned + Validate<Context = ()> + Clone + Debug {
    /// Whether the platform provides this kind of entity at all. `false` only
    /// for [`NoConfig`](crate::NoConfig).
    const SUPPORTED: bool = true;

    fn platform(&self) -> &str;

    /// The resolved object id (explicit `id`, or the snake_cased `name`).
    fn object_id(&self) -> String;

    /// The automation triggers (e.g. `on_press`) configured on this entity,
    /// keyed by their config field name. Used for cross-reference validation
    /// of the actions inside them.
    fn triggers(&self) -> Vec<(&'static str, &Trigger)>;
}

/// Collects `field` into `out` if it is a `Trigger` or `Option<Trigger>`.
/// Used by `with_base_entity_properties!` to discover trigger fields without
/// each component having to list them.
#[doc(hidden)]
pub fn push_trigger<'a, T: Any>(
    out: &mut Vec<(&'static str, &'a Trigger)>,
    name: &'static str,
    field: &'a T,
) {
    let field = field as &dyn Any;
    if let Some(Some(trigger)) = field.downcast_ref::<Option<Trigger>>() {
        out.push((name, trigger));
    } else if let Some(trigger) = field.downcast_ref::<Trigger>() {
        out.push((name, trigger));
    }
}

/// The config types of one platform crate. Implemented for each crate's
/// `CoreConfig` by [`config_template!`](crate::config_template).
pub trait PlatformConfigTypes: Sized {
    /// The platform name, i.e. its top-level config key and the value of
    /// `platform:` on its entities.
    const NAME: &'static str;

    /// The platform's own top-level section (e.g. `shell:`).
    type Section: DeserializeOwned + Validate<Context = ()> + Clone + Debug;
    type Button: EntityConfig;
    type BinarySensor: EntityConfig;
    type Sensor: EntityConfig;
    type Switch: EntityConfig;
    type Light: EntityConfig;
    type Number: EntityConfig;
    type TextSensor: EntityConfig;
    type MediaPlayer: EntityConfig;

    fn from_parts(parts: PlatformParts<Self>) -> Self;
}

/// Everything from the configuration file that belongs to one platform.
pub struct PlatformParts<C: PlatformConfigTypes> {
    pub ubihome: UbiHome,
    pub section: C::Section,
    pub button: Vec<C::Button>,
    pub binary_sensor: Vec<C::BinarySensor>,
    pub sensor: Vec<C::Sensor>,
    pub switch: Vec<C::Switch>,
    pub light: Vec<C::Light>,
    pub number: Vec<C::Number>,
    pub text_sensor: Vec<C::TextSensor>,
    pub media_player: Vec<C::MediaPlayer>,
}

/// A platform module that is constructed from its (already validated) config.
pub trait ConfigurableModule: Module + Sized {
    type Config: PlatformConfigTypes;

    /// Set the module up from its configuration. Validation already happened
    /// for the whole file; do not do heavy lifting here.
    fn new(config: Self::Config) -> Result<Self, String>;

    /// Parse `config_string` as a configuration file containing (at least)
    /// this platform and construct the module from it. Convenient for tests;
    /// the application itself parses all platforms at once.
    #[cfg(feature = "validation")]
    fn from_yaml(config_string: &str) -> Result<Self, String> {
        Self::new(crate::validation::parse_platform_config(config_string)?)
    }
}

pub type ConfigOf<M> = <M as ConfigurableModule>::Config;
pub type SectionOf<M> = <ConfigOf<M> as PlatformConfigTypes>::Section;

/// Turn a list of entity configs into the `object_id -> config` map the
/// platform crates consume. Duplicate ids are rejected during validation
/// already. Returns `None` when there are no entities.
pub fn into_entity_map<T: EntityConfig>(entities: Vec<T>) -> Option<HashMap<String, T>> {
    if entities.is_empty() {
        return None;
    }
    Some(entities.into_iter().map(|e| (e.object_id(), e)).collect())
}

/// Deserialize a mapping that carries a `platform:` discriminator.
///
/// Entries before `platform` are buffered; `platform` itself and everything
/// after it are streamed straight from the underlying YAML deserializer to the
/// platform-specific type. Since `platform` usually comes first, errors keep
/// their exact source location (unlike serde's internally tagged enums, which
/// buffer the whole mapping).
pub fn visit_platform_map<'de, A, R, F>(mut map: A, dispatch: F) -> Result<R, A::Error>
where
    A: MapAccess<'de>,
    F: FnOnce(&str, PlatformMap<'de, A>) -> Result<R, A::Error>,
{
    let mut buffered: Vec<(String, Value)> = Vec::new();
    while let Some(key) = map.next_key::<String>()? {
        if key == "platform" {
            let platform: String = map.next_value()?;
            let rest = PlatformMap {
                platform: Some(platform.clone()),
                buffered: buffered.into_iter(),
                pending: Pending::Inner,
                inner: map,
                marker: PhantomData,
            };
            return dispatch(&platform, rest);
        }
        let value: Value = map.next_value()?;
        buffered.push((key, value));
    }
    Err(de::Error::missing_field("platform"))
}

enum Pending {
    Platform(String),
    Buffered(Value),
    Inner,
}

/// The remainder of a mapping after its `platform` key was read; replays the
/// `platform` entry and any buffered entries before streaming the rest.
pub struct PlatformMap<'de, A> {
    platform: Option<String>,
    buffered: std::vec::IntoIter<(String, Value)>,
    pending: Pending,
    inner: A,
    marker: PhantomData<&'de ()>,
}

impl<'de, A: MapAccess<'de>> PlatformMap<'de, A> {
    /// Consume and discard the remaining entries.
    pub fn drain(mut self) -> Result<(), A::Error> {
        while self
            .next_entry::<de::IgnoredAny, de::IgnoredAny>()?
            .is_some()
        {}
        Ok(())
    }
}

impl<'de, A: MapAccess<'de>> MapAccess<'de> for PlatformMap<'de, A> {
    type Error = A::Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        if let Some(platform) = self.platform.take() {
            self.pending = Pending::Platform(platform);
            return seed
                .deserialize(IntoDeserializer::<A::Error>::into_deserializer("platform"))
                .map(Some);
        }
        if let Some((key, value)) = self.buffered.next() {
            self.pending = Pending::Buffered(value);
            return seed
                .deserialize(IntoDeserializer::<A::Error>::into_deserializer(key))
                .map(Some);
        }
        self.pending = Pending::Inner;
        self.inner.next_key_seed(seed)
    }

    fn next_value_seed<V: DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        match std::mem::replace(&mut self.pending, Pending::Inner) {
            Pending::Platform(platform) => {
                seed.deserialize(IntoDeserializer::<A::Error>::into_deserializer(platform))
            }
            Pending::Buffered(value) => {
                seed.deserialize(serde_value::ValueDeserializer::<A::Error>::new(value))
            }
            Pending::Inner => self.inner.next_value_seed(seed),
        }
    }
}
