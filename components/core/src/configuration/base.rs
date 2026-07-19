use garde::Validate;
use serde::Deserialize;

use crate::constants::{is_id_string_option, is_readable_string_option};

/// The `name` / `id` pair shared by every component.
///
/// Deserialized through [`ComponentIdentityShadow`] via `#[serde(try_from)]` so
/// the "at least one of `name` / `id`" rule is enforced as part of normal serde
/// deserialization (rather than a separate validation pass). Flattened into
/// every component config by [`with_base_entity_properties!`].
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(try_from = "ComponentIdentityShadow")]
pub struct ComponentIdentity {
    #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
    pub id: Option<String>,

    #[garde(custom(is_readable_string_option), length(min = 3, max = 100))]
    pub name: Option<String>,
}

/// Raw, unvalidated counterpart of [`ComponentIdentity`]. Deserializes `name` /
/// `id` without the cross-field requirement; the `TryFrom` impl enforces it.
#[derive(Clone, Debug, Deserialize)]
struct ComponentIdentityShadow {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
}

impl TryFrom<ComponentIdentityShadow> for ComponentIdentity {
    type Error = String;

    fn try_from(shadow: ComponentIdentityShadow) -> Result<Self, Self::Error> {
        if shadow.id.is_none() && shadow.name.is_none() {
            return Err("Either 'name' or 'id' must be provided for the component".to_string());
        }
        Ok(ComponentIdentity {
            id: shadow.id,
            name: shadow.name,
        })
    }
}

/// Macro to add base entity properties (id, name, platform) to a struct.
///
/// # Example
/// ```ignore
/// with_base_entity_properties! {
///     #[derive(Clone, Debug, Deserialize, Validate)]
///     pub struct MyEntity {
///         // additional fields here
///         pub my_field: String,
///     }
/// }
/// ```
#[macro_export]
macro_rules! with_base_entity_properties {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
        $(#[$meta])*
        #[serde(deny_unknown_fields)]
        $vis struct $name {
            // `name` / `id` (with the "at least one required" rule enforced by
            // serde during deserialization) are flattened in from a shared type.
            #[serde(flatten)]
            #[garde(dive)]
            pub identity: $crate::configuration::base::ComponentIdentity,

            #[garde(length(min = 3, max = 100))]
            pub platform: String,

            // #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
            #[garde(ascii)]
            pub icon: Option<String>,
            #[garde(ascii)]
            pub device_class: Option<String>,

            /// Explicitly mark the component as internal or not. When set, this
            /// overrides the default behaviour (a component with only an `id`
            /// and no `name` is internal).
            #[garde(skip)]
            pub internal: Option<bool>,


            $(
                $(#[$field_meta])*
                $field_vis $field_name : $field_type,
            )*
        }

        impl $name {
            pub fn get_object_id(&self) -> String {
                $crate::utils::format_id(&self.identity.id, &self.identity.name)
            }
            pub fn is_configured(&self) -> bool {
                true
            }
            /// Whether the component is internal, i.e. wired up internally but
            /// not exposed to connectivity components (api, mqtt, http).
            ///
            /// Defaults to true when the component supplies only an `id` and no
            /// `name`. An explicit `internal` attribute overrides this default.
            pub fn is_internal(&self) -> bool {
                self.internal.unwrap_or_else(|| self.identity.name.is_none())
            }
        }
    };
}

#[macro_export]
macro_rules! template_binary_sensor {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
        use ubihome_core::configuration::binary_sensor::{BinarySensorFilter, Trigger};

        with_base_entity_properties! {
            $(#[$meta])*
            // TODO: Add? #[serde(deny_unknown_fields)]

            $vis struct $name {
                #[garde(dive)]
                pub filters: Option<Vec<BinarySensorFilter>>,
                #[garde(dive)]
                pub on_press: Option<Trigger>,
                #[garde(dive)]
                pub on_release: Option<Trigger>,

                $(
                    $(#[$field_meta])*
                    $field_vis $field_name : $field_type,
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! template_sensor {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
        use ubihome_core::configuration::sensor::SensorFilter;

        with_base_entity_properties! {
            $(#[$meta])*
            // TODO: Add? #[serde(deny_unknown_fields)]

            $vis struct $name {
                #[garde(skip)]
                pub unit_of_measurement: Option<String>,
                #[garde(skip)]
                pub state_class: Option<String>,
                #[garde(skip)]
                pub accuracy_decimals: Option<i32>,

                #[garde(skip)]
                pub filters: Option<Vec<SensorFilter>>,

                $(
                    $(#[$field_meta])*
                    $field_vis $field_name : $field_type,
                )*
            }
        }
    };
}

#[macro_export]
macro_rules! template_button {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
            with_base_entity_properties! {
                $(#[$meta])*
                // TODO: Add? #[serde(deny_unknown_fields)]

                $vis struct $name {
                    // #[garde(dive)]
                    // pub filters: Option<Vec<BinarySensorFilter>>,


                    $(
                        $(#[$field_meta])*
                        $field_vis $field_name : $field_type,
                    )*
                }
            }

    };
}

#[macro_export]
macro_rules! template_number {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
            with_base_entity_properties! {
                $(#[$meta])*
                // TODO: Add? #[serde(deny_unknown_fields)]

                $vis struct $name {
                    // #[garde(dive)]
                    // pub filters: Option<Vec<BinarySensorFilter>>,
                    pub unit_of_measurement: Option<String>,
                    pub state_class: Option<String>,
                    pub min_value: Option<f32>,
                    pub max_value: Option<f32>,
                    pub step: Option<f32>,

                    $(
                        $(#[$field_meta])*
                        $field_vis $field_name : $field_type,
                    )*
                }
            }

    };
}

#[macro_export]
macro_rules! template_switch {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
            with_base_entity_properties! {
                $(#[$meta])*
                // TODO: Add? #[serde(deny_unknown_fields)]

                $vis struct $name {
                    // #[garde(dive)]
                    // pub filters: Option<Vec<BinarySensorFilter>>,

                    $(
                        $(#[$field_meta])*
                        $field_vis $field_name : $field_type,
                    )*
                }
            }

    };
}

#[macro_export]
macro_rules! template_light {
    (
        $(#[$meta:meta])*
        $vis:vis struct $name:ident {
            $(
                $(#[$field_meta:meta])*
                $field_vis:vis $field_name:ident : $field_type:ty
            ),* $(,)?
        }
    ) => {
            with_base_entity_properties! {
                $(#[$meta])*
                // TODO: Add? #[serde(deny_unknown_fields)]

                $vis struct $name {
                    // #[garde(dive)]
                    // pub filters: Option<Vec<BinarySensorFilter>>,
                    #[garde(skip)]
                    pub disabled_by_default: Option<bool>,

                    $(
                        $(#[$field_meta])*
                        $field_vis $field_name : $field_type,
                    )*
                }
            }

    };
}

#[cfg(test)]
mod identity_tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Validate)]
    #[serde(deny_unknown_fields)]
    struct Probe {
        #[serde(flatten)]
        #[garde(dive)]
        identity: ComponentIdentity,
        #[garde(skip)]
        pin: u8,
    }

    fn parse(json: &str) -> Result<Probe, String> {
        serde_json::from_str::<Probe>(json).map_err(|e| e.to_string())
    }

    #[test]
    fn name_only_is_accepted() {
        let p = parse(r#"{"name":"Front Door","pin":5}"#).expect("name-only should deserialize");
        assert_eq!(p.identity.name.as_deref(), Some("Front Door"));
        assert!(p.identity.id.is_none());
        assert_eq!(p.pin, 5);
    }

    #[test]
    fn id_only_is_accepted() {
        let p = parse(r#"{"id":"front_door","pin":5}"#).expect("id-only should deserialize");
        assert_eq!(p.identity.id.as_deref(), Some("front_door"));
        assert!(p.identity.name.is_none());
    }

    #[test]
    fn neither_is_rejected_during_deserialization() {
        let err = parse(r#"{"pin":5}"#).expect_err("missing name and id must fail");
        assert!(
            err.contains("Either 'name' or 'id'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn unknown_fields_are_still_rejected() {
        // The flattened identity must not disable deny_unknown_fields on the
        // enclosing component config.
        let err = parse(r#"{"name":"x","pin":5,"bogus":9}"#)
            .expect_err("unknown field must still be rejected");
        assert!(err.contains("bogus"), "unexpected error: {err}");
    }
}
