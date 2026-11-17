use garde::Validate;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::fmt;
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome::Logger;
use ubihome_core::configuration::automation::Trigger;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::platform::{
    visit_platform_map, ConfigOf, ConfigurableModule, EntityConfig, PlatformConfigTypes,
    PlatformParts, SectionOf,
};
use ubihome_core::state::StateStore;
use ubihome_core::Module;
use ubihome_core::UbiHome;
use ubihome_core::{ChangedMessage, PublishedMessage};

use crate::builtins::template::{
    LambdaGlobal, TemplateButtonConfig, TemplateConfig, TemplateNumberConfig, TemplateSensorConfig,
    TemplateSwitchConfig,
};
use crate::builtins::{BuiltinConfig, GlobalConfig};
use crate::config::{AnyEntity, ConfigView};

/// Generates the enum holding one entry of an entity list (e.g. `switch:`),
/// dispatched on its `platform:` to the config type of the matching platform
/// crate (or builtin), and its [`AnyEntity`] view used for validation.
macro_rules! entity_enum {
    (
        $enum:ident, $assoc:ident, $kind:literal,
        [$($builtin_name:literal => $builtin_variant:ident($builtin_ty:ty)),*],
        $(($variant:ident, $platform_name:literal, $module_path:ident)),* $(,)?
    ) => {
        // Parsed once at startup; boxing every variant isn't worth it.
        #[allow(clippy::large_enum_variant)]
        #[derive(Debug, Clone)]
        pub(crate) enum $enum {
            $(
                $variant(<ConfigOf<$module_path::UbiHomePlatform> as PlatformConfigTypes>::$assoc),
            )*
            $(
                $builtin_variant($builtin_ty),
            )*
            /// The `platform:` names no compiled-in platform; reported during
            /// validation (with its location) instead of failing deserialization.
            Unknown(String),
        }

        impl<'de> Deserialize<'de> for $enum {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct EntityVisitor;

                impl<'de> Visitor<'de> for EntityVisitor {
                    type Value = $enum;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        write!(f, "a {} mapping with a `platform` key", $kind)
                    }

                    fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<$enum, A::Error> {
                        visit_platform_map(map, |platform, rest| {
                            let rest = de::value::MapAccessDeserializer::new(rest);
                            match platform {
                                $(
                                    $platform_name => Deserialize::deserialize(rest).map($enum::$variant),
                                )*
                                $(
                                    $builtin_name => Deserialize::deserialize(rest).map($enum::$builtin_variant),
                                )*
                                other => {
                                    let other = other.to_string();
                                    de::IgnoredAny::deserialize(rest)?;
                                    Ok($enum::Unknown(other))
                                }
                            }
                        })
                    }
                }

                deserializer.deserialize_map(EntityVisitor)
            }
        }

        impl AnyEntity for $enum {
            fn platform(&self) -> &str {
                match self {
                    $( $enum::$variant(entity) => entity.platform(), )*
                    $( $enum::$builtin_variant(entity) => entity.platform(), )*
                    $enum::Unknown(platform) => platform,
                }
            }

            fn is_compiled_in(&self) -> bool {
                !matches!(self, $enum::Unknown(_))
            }

            fn is_supported(&self) -> bool {
                match self {
                    $(
                        $enum::$variant(_) => <<ConfigOf<$module_path::UbiHomePlatform> as PlatformConfigTypes>::$assoc as EntityConfig>::SUPPORTED,
                    )*
                    _ => true,
                }
            }

            fn object_id(&self) -> String {
                match self {
                    $( $enum::$variant(entity) => entity.object_id(), )*
                    $( $enum::$builtin_variant(entity) => entity.object_id(), )*
                    $enum::Unknown(_) => String::new(),
                }
            }

            fn triggers(&self) -> Vec<(&'static str, &Trigger)> {
                match self {
                    $( $enum::$variant(entity) => entity.triggers(), )*
                    $( $enum::$builtin_variant(entity) => entity.triggers(), )*
                    $enum::Unknown(_) => Vec::new(),
                }
            }

            fn lambda_global(&self) -> Option<(&str, &'static [crate::builtins::globals::GlobalType])> {
                match self {
                    $( $enum::$builtin_variant(entity) => entity.lambda_global(), )*
                    _ => None,
                }
            }

            fn validate_entity(&self, parent: &mut dyn FnMut() -> garde::Path, report: &mut garde::Report) {
                match self {
                    $( $enum::$variant(entity) => entity.validate_into(&(), parent, report), )*
                    $( $enum::$builtin_variant(entity) => entity.validate_into(&(), parent, report), )*
                    $enum::Unknown(_) => {}
                }
            }
        }
    };
}

/// Collects the entities of one platform variant from an entity list.
macro_rules! entities_of {
    ($list:expr, $enum:ident :: $variant:ident) => {
        $list
            .iter()
            .filter_map(|entity| {
                if let $enum::$variant(config) = entity {
                    Some(config.clone())
                } else {
                    None
                }
            })
            .collect()
    };
}

macro_rules! generate_component_methods {
    (
        $(($variant:ident, $platform_name:literal, $module_path:ident, $type_name:ident)),* $(,)?
    ) => {
        /// Names of all compiled-in platforms, i.e. the valid platform
        /// top-level keys.
        #[allow(dead_code)]
        pub(crate) const PLATFORM_NAMES: &[&str] = &[$($platform_name),*];

        entity_enum!(AnyButton, Button, "button", ["template" => Template(TemplateButtonConfig)], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnyBinarySensor, BinarySensor, "binary_sensor", [], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnySensor, Sensor, "sensor", ["template" => Template(TemplateSensorConfig)], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnySwitch, Switch, "switch", ["template" => Template(TemplateSwitchConfig)], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnyLight, Light, "light", [], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnyNumber, Number, "number", ["template" => Template(TemplateNumberConfig)], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnyTextSensor, TextSensor, "text_sensor", [], $(($variant, $platform_name, $module_path)),*);
        entity_enum!(AnyMediaPlayer, MediaPlayer, "media_player", [], $(($variant, $platform_name, $module_path)),*);

        /// The top-level section of each platform, `Some` when it is present
        /// in the configuration file (which is what enables the platform).
        #[derive(Debug, Clone, Default)]
        #[allow(non_snake_case)]
        struct PlatformSections {
            $(
                $variant: Option<SectionOf<$module_path::UbiHomePlatform>>,
            )*
        }

        /// The whole configuration file, deserialized and validated in a single
        /// pass. Covers the base sections, builtins (`globals`, `platform:
        /// template` entities) and every compiled-in platform crate.
        #[derive(Debug, Clone)]
        pub(crate) struct RootConfig {
            pub ubihome: UbiHome,
            pub logger: Option<Logger>,
            pub globals: Vec<GlobalConfig>,
            button: Vec<AnyButton>,
            binary_sensor: Vec<AnyBinarySensor>,
            sensor: Vec<AnySensor>,
            switch: Vec<AnySwitch>,
            light: Vec<AnyLight>,
            number: Vec<AnyNumber>,
            text_sensor: Vec<AnyTextSensor>,
            media_player: Vec<AnyMediaPlayer>,
            sections: PlatformSections,
            /// Top-level keys that are neither a base section nor a
            /// compiled-in platform.
            unknown_keys: Vec<String>,
        }

        impl<'de> Deserialize<'de> for RootConfig {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                struct RootVisitor;

                impl<'de> Visitor<'de> for RootVisitor {
                    type Value = RootConfig;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("a configuration mapping")
                    }

                    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<RootConfig, A::Error> {
                        let mut ubihome = None;
                        let mut logger = None;
                        let mut globals = Vec::new();
                        let mut button = Vec::new();
                        let mut binary_sensor = Vec::new();
                        let mut sensor = Vec::new();
                        let mut switch = Vec::new();
                        let mut light = Vec::new();
                        let mut number = Vec::new();
                        let mut text_sensor = Vec::new();
                        let mut media_player = Vec::new();
                        let mut sections = PlatformSections::default();
                        let mut unknown_keys = Vec::new();

                        while let Some(key) = map.next_key::<String>()? {
                            match key.as_str() {
                                "ubihome" => ubihome = Some(map.next_value()?),
                                "logger" => logger = map.next_value()?,
                                "globals" => globals = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "button" => button = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "binary_sensor" => binary_sensor = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "sensor" => sensor = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "switch" => switch = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "light" => light = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "number" => number = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "text_sensor" => text_sensor = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                "media_player" => media_player = map.next_value::<Option<_>>()?.unwrap_or_default(),
                                $(
                                    $platform_name => sections.$variant = Some(map.next_value()?),
                                )*
                                _ => {
                                    map.next_value::<de::IgnoredAny>()?;
                                    unknown_keys.push(key);
                                }
                            }
                        }

                        Ok(RootConfig {
                            ubihome: ubihome.ok_or_else(|| de::Error::missing_field("ubihome"))?,
                            logger,
                            globals,
                            button,
                            binary_sensor,
                            sensor,
                            switch,
                            light,
                            number,
                            text_sensor,
                            media_player,
                            sections,
                            unknown_keys,
                        })
                    }
                }

                deserializer.deserialize_map(RootVisitor)
            }
        }

        impl Validate for RootConfig {
            type Context = ();

            fn validate_into(
                &self,
                _ctx: &(),
                parent: &mut dyn FnMut() -> garde::Path,
                report: &mut garde::Report,
            ) {
                $(
                    if let Some(section) = &self.sections.$variant {
                        section.validate_into(&(), &mut || parent().join($platform_name), report);
                    }
                )*
                crate::config::validate_root(&self.view(), parent, report);
            }
        }

        impl RootConfig {
            fn view(&self) -> ConfigView<'_> {
                fn list<T: AnyEntity>(entities: &[T]) -> Vec<&dyn AnyEntity> {
                    entities.iter().map(|e| e as &dyn AnyEntity).collect()
                }
                ConfigView {
                    ubihome: &self.ubihome,
                    logger: self.logger.as_ref(),
                    globals: &self.globals,
                    configured_platforms: self.configured_platforms(),
                    unknown_keys: &self.unknown_keys,
                    entities: vec![
                        ("button", list(&self.button)),
                        ("binary_sensor", list(&self.binary_sensor)),
                        ("sensor", list(&self.sensor)),
                        ("switch", list(&self.switch)),
                        ("light", list(&self.light)),
                        ("number", list(&self.number)),
                        ("text_sensor", list(&self.text_sensor)),
                        ("media_player", list(&self.media_player)),
                    ],
                }
            }

            /// Platforms enabled by a top-level section, in a stable order.
            pub(crate) fn configured_platforms(&self) -> Vec<&'static str> {
                let mut platforms = Vec::new();
                $(
                    if self.sections.$variant.is_some() {
                        platforms.push($platform_name);
                    }
                )*
                platforms
            }

            /// Construct every configured platform module from its slice of
            /// the (already validated) configuration.
            pub(crate) fn build_modules(&self) -> Result<Vec<Box<dyn Module>>, String> {
                let mut modules: Vec<Box<dyn Module>> = Vec::new();
                $(
                    if let Some(section) = &self.sections.$variant {
                        log::debug!("Configuring platform: {}", $platform_name);
                        let parts = PlatformParts::<ConfigOf<$module_path::UbiHomePlatform>> {
                            ubihome: self.ubihome.clone(),
                            section: section.clone(),
                            button: entities_of!(self.button, AnyButton::$variant),
                            binary_sensor: entities_of!(self.binary_sensor, AnyBinarySensor::$variant),
                            sensor: entities_of!(self.sensor, AnySensor::$variant),
                            switch: entities_of!(self.switch, AnySwitch::$variant),
                            light: entities_of!(self.light, AnyLight::$variant),
                            number: entities_of!(self.number, AnyNumber::$variant),
                            text_sensor: entities_of!(self.text_sensor, AnyTextSensor::$variant),
                            media_player: entities_of!(self.media_player, AnyMediaPlayer::$variant),
                        };
                        let module = <$module_path::$type_name as ConfigurableModule>::new(
                            PlatformConfigTypes::from_parts(parts),
                        )?;
                        modules.push(Box::new(module));
                    }
                )*
                Ok(modules)
            }

            /// The builtin (`globals`, `platform: template`) configuration.
            pub(crate) fn builtin(&self) -> BuiltinConfig {
                BuiltinConfig {
                    template: TemplateConfig {
                        switches: entities_of!(self.switch, AnySwitch::Template),
                        buttons: entities_of!(self.button, AnyButton::Template),
                        numbers: entities_of!(self.number, AnyNumber::Template),
                        sensors: entities_of!(self.sensor, AnySensor::Template),
                    },
                    globals: self.globals.clone(),
                }
            }
        }
    };
}

include!(concat!(env!("OUT_DIR"), "/", "components.rs"));

pub(crate) fn initialize_platforms(
    modules: &mut Vec<Box<dyn Module>>,
) -> Result<Vec<UbiComponent>, String> {
    let mut all_components: Vec<UbiComponent> = Vec::new();
    for module in modules.iter_mut() {
        let mut components = module.components();
        all_components.append(&mut components);
    }
    Ok(all_components)
}

pub(crate) fn run_platforms(
    tasks: &mut tokio::task::JoinSet<()>,
    modules: Vec<Box<dyn Module>>,
    sender: Sender<ChangedMessage>,
    receiver: Receiver<PublishedMessage>,
    state: StateStore,
) {
    for module in modules {
        let tx = sender.clone();
        let rx = receiver.resubscribe();
        let state = state.clone();
        tasks.spawn(async move {
            // A module returning Ok(()) is a normal, intentional exit and stays
            // silent. An Err is a real failure: unwrap panics, which is reported to
            // Sentry by the panic integration. Tasks are collected in a JoinSet so
            // the caller observes the panic (instead of it being swallowed by this
            // task) and shuts the application down.
            module.run(tx, rx, state).await.unwrap();
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ubihome_core::state::StateStoreWriter;
    use ubihome_core::ModuleRunFuture;

    /// Platform names must not collide with the base top-level sections,
    /// which take precedence when parsing the root config.
    #[test]
    fn platform_names_are_not_reserved() {
        let reserved = [
            "ubihome",
            "logger",
            "globals",
            "button",
            "binary_sensor",
            "sensor",
            "switch",
            "light",
            "number",
            "text_sensor",
            "media_player",
        ];
        for name in PLATFORM_NAMES {
            assert!(
                !reserved.contains(name),
                "platform name '{name}' is reserved"
            );
        }
    }

    /// Minimal module whose `run` either exits normally or returns an error,
    /// used to exercise how `run_platforms` supervises module tasks.
    struct MockModule {
        should_err: bool,
    }

    impl Module for MockModule {
        fn components(&mut self) -> Vec<UbiComponent> {
            Vec::new()
        }

        fn run(
            &self,
            _sender: Sender<ChangedMessage>,
            _receiver: Receiver<PublishedMessage>,
            _state: StateStore,
        ) -> ModuleRunFuture {
            let should_err = self.should_err;
            Box::pin(async move {
                if should_err {
                    Err("mock module failure".into())
                } else {
                    Ok(())
                }
            })
        }
    }

    fn spawn_mock(should_err: bool) -> tokio::task::JoinSet<()> {
        let (changed_tx, _changed_rx) = tokio::sync::broadcast::channel::<ChangedMessage>(16);
        let (_published_tx, published_rx) = tokio::sync::broadcast::channel::<PublishedMessage>(16);
        let mut tasks = tokio::task::JoinSet::new();
        let modules: Vec<Box<dyn Module>> = vec![Box::new(MockModule { should_err })];
        let (_state_writer, state) = StateStoreWriter::new(Vec::new());
        run_platforms(&mut tasks, modules, changed_tx, published_rx, state);
        tasks
    }

    /// A module returning Ok(()) is an intentional exit: the task completes
    /// normally so the supervisor leaves the application running.
    #[tokio::test]
    async fn run_platforms_normal_exit_completes_cleanly() {
        let mut tasks = spawn_mock(false);
        let joined = tasks.join_next().await.expect("one task was spawned");
        assert!(
            joined.is_ok(),
            "a module returning Ok(()) must not panic its task"
        );
    }

    /// A module returning Err is a real failure: unwrap panics, which the
    /// supervisor observes (via a JoinError) to trigger application shutdown.
    #[tokio::test]
    async fn run_platforms_erroring_module_panics_task() {
        let mut tasks = spawn_mock(true);
        let joined = tasks.join_next().await.expect("one task was spawned");
        let join_error = joined.expect_err("a module returning Err must panic its task");
        assert!(
            join_error.is_panic(),
            "the failure must surface as a panic, not a cancellation"
        );
    }
}
