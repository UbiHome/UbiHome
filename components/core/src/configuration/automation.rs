use crate::global_value::GlobalValue;
use duration_str::deserialize_duration;
use garde::Validate;
use serde::Deserialize;
use serde::Serialize;
use serde::de;
use std::marker::PhantomData;
use std::time::Duration;

/// A single automation action. New action types are added as variants here and
/// are executed centrally by the main binary (see the runtime action executor),
/// so that actions can reference entities and globals across platforms.
#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
pub enum ActionType {
    #[serde(rename = "switch.turn_on")]
    SwitchTurnOn(#[garde(ascii)] String),

    #[serde(rename = "switch.turn_off")]
    SwitchTurnOff(#[garde(ascii)] String),

    #[serde(rename = "button.press")]
    ButtonPress(#[garde(ascii)] String),

    /// Sets the value of a [`globals`] variable. `value` accepts a plain YAML
    /// scalar (string, boolean, or number); it is reconciled against the
    /// global's declared `type:` at runtime.
    #[serde(rename = "globals.set")]
    GlobalsSet {
        #[garde(ascii)]
        id: String,
        #[garde(skip)]
        value: GlobalValue,
    },

    /// Pauses the action list for the configured duration before running the
    /// next action.
    #[serde(rename = "delay", deserialize_with = "deserialize_duration")]
    Delay(#[garde(skip)] Duration),
}

#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct Action {
    #[serde(flatten)]
    #[garde(skip)]
    pub action: ActionType,
}

/// A trigger runs its list of actions (in order) when the owning component
/// fires it (for example a binary sensor `on_press`).
#[derive(Clone, Serialize, Deserialize, Debug, Validate)]
#[serde(deny_unknown_fields)]
pub struct Trigger {
    #[garde(dive)]
    pub then: Vec<Action>,
}

/// Generic `deserialize_with` helper for `Option<T>` config fields where `T`
/// is expected to be a YAML mapping (for example a [`Trigger`]'s `then:
/// [...]`).
///
/// Plain `#[derive(Deserialize)]` structs also accept a bare sequence: serde
/// generates positional/tuple-style deserialization for every struct (needed
/// for non-self-describing formats like bincode), so a struct with few
/// fields silently tries to read a wrongly-supplied list positionally and
/// reports a confusing, backwards-reading error - e.g. `Trigger { then:
/// Vec<Action> }` given `[{switch.turn_on: x}, ...]` tries to parse the
/// list's first item as `then`'s value and fails with "invalid type: map,
/// expected a sequence", not the other way around as you'd expect.
///
/// Routing a field through this helper instead rejects any sequence up
/// front with a plain, correctly-oriented "invalid type: sequence, expected
/// a mapping" error - no per-struct `Visitor` needed. Use it (together with
/// `#[serde(default)]`, so a missing key still resolves to `None`) on any
/// `Option<T>` field prone to this mistake:
///
/// ```ignore
/// #[serde(default, deserialize_with = "ubihome_core::configuration::automation::deserialize_option_map_only")]
/// pub on_press: Option<Trigger>,
/// ```
pub fn deserialize_option_map_only<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: de::Deserializer<'de>,
    T: Deserialize<'de>,
{
    struct MapOnlyVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for MapOnlyVisitor<T> {
        type Value = T;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a mapping")
        }

        fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            T::deserialize(de::value::MapAccessDeserializer::new(map))
        }
    }

    struct OptionMapOnlyVisitor<T>(PhantomData<T>);

    impl<'de, T: Deserialize<'de>> de::Visitor<'de> for OptionMapOnlyVisitor<T> {
        type Value = Option<T>;

        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            f.write_str("a mapping or null")
        }

        fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_none<E: de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D2>(self, deserializer: D2) -> Result<Self::Value, D2::Error>
        where
            D2: de::Deserializer<'de>,
        {
            deserializer
                .deserialize_any(MapOnlyVisitor::<T>(PhantomData))
                .map(Some)
        }
    }

    deserializer.deserialize_option(OptionMapOnlyVisitor(PhantomData))
}
