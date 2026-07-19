/// Macro to add base entity properties (id, name, internal, platform) to a
/// struct.
///
/// Deserialization goes through a private shadow struct (via
/// `#[serde(try_from)]`), so that as part of normal deserialization:
/// - at least one of `name` / `id` must be given, and
/// - `internal` is resolved to a concrete `bool` — the user's `internal`
///   attribute if set, otherwise `true` when only an `id` was provided.
///
/// Components keep plain `name` / `id` / `internal` fields (no `#[serde(flatten)]`),
/// and `deny_unknown_fields` still rejects typo'd keys.
///
/// # Example
/// ```ignore
/// with_base_entity_properties! {
///     #[derive(Clone, Deserialize, Debug, Validate)]
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
        $crate::paste::paste! {
            $(#[$meta])*
            #[serde(try_from = "" [<$name __Shadow>] "")]
            $vis struct $name {
                #[garde(custom($crate::constants::is_id_string_option), length(min = 3, max = 100))]
                pub id: Option<String>,

                #[garde(custom($crate::constants::is_readable_string_option), length(min = 3, max = 100))]
                pub name: Option<String>,

                /// Whether the component is internal (wired up internally but not
                /// exposed to connectivity components like api, mqtt, http).
                /// Resolved during deserialization from the user's `internal`
                /// attribute, defaulting to `true` when only an `id` was given.
                #[garde(skip)]
                pub internal: bool,

                #[garde(length(min = 3, max = 100))]
                pub platform: String,

                #[garde(ascii)]
                pub icon: Option<String>,
                #[garde(ascii)]
                pub device_class: Option<String>,


                $(
                    $(#[$field_meta])*
                    $field_vis $field_name : $field_type,
                )*
            }

            impl $name {
                pub fn get_object_id(&self) -> String {
                    $crate::utils::format_id(&self.id, &self.name)
                }
                pub fn is_configured(&self) -> bool {
                    true
                }
            }

            // Raw, unvalidated counterpart of `$name` that deserializes the
            // fields as the user wrote them. The `TryFrom` impl enforces the
            // name/id rule and resolves `internal`.
            #[doc(hidden)]
            #[derive(::serde::Deserialize, ::garde::Validate)]
            #[garde(allow_unvalidated)]
            #[serde(deny_unknown_fields)]
            #[allow(non_camel_case_types)]
            $vis struct [<$name __Shadow>] {
                id: Option<String>,
                name: Option<String>,
                internal: Option<bool>,
                platform: String,
                icon: Option<String>,
                device_class: Option<String>,
                $(
                    $(#[$field_meta])*
                    $field_name : $field_type,
                )*
            }

            impl ::core::convert::TryFrom<[<$name __Shadow>]> for $name {
                type Error = ::std::string::String;

                fn try_from(shadow: [<$name __Shadow>]) -> ::core::result::Result<Self, Self::Error> {
                    if shadow.id.is_none() && shadow.name.is_none() {
                        return Err(format!(
                            "Either 'name' or 'id' must be provided for the '{}' component",
                            shadow.platform
                        ));
                    }
                    Ok($name {
                        internal: shadow.internal.unwrap_or(shadow.name.is_none()),
                        id: shadow.id,
                        name: shadow.name,
                        platform: shadow.platform,
                        icon: shadow.icon,
                        device_class: shadow.device_class,
                        $(
                            $field_name: shadow.$field_name,
                        )*
                    })
                }
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
mod entity_tests {
    use garde::Validate;

    with_base_entity_properties! {
        #[derive(Clone, ::serde::Deserialize, Debug, Validate)]
        #[garde(allow_unvalidated)]
        pub struct Probe {
            #[garde(skip)]
            pub pin: u8,
        }
    }

    fn parse(json: &str) -> Result<Probe, String> {
        serde_json::from_str::<Probe>(json).map_err(|e| e.to_string())
    }

    #[test]
    fn name_only_is_visible() {
        let p = parse(r#"{"platform":"probe","name":"Front Door","pin":5}"#)
            .expect("name-only should deserialize");
        assert_eq!(p.name.as_deref(), Some("Front Door"));
        assert!(!p.internal, "a named component is not internal by default");
        assert_eq!(p.pin, 5);
        assert!(p.is_configured());
        assert_eq!(p.get_object_id(), "front_door");
    }

    #[test]
    fn id_only_is_internal() {
        let p = parse(r#"{"platform":"probe","id":"front_door","pin":5}"#)
            .expect("id-only should deserialize");
        assert!(p.internal, "an id-only component is internal by default");
        assert_eq!(p.get_object_id(), "front_door");
    }

    #[test]
    fn explicit_internal_overrides_the_default() {
        let visible = parse(r#"{"platform":"probe","id":"x","internal":false,"pin":5}"#).unwrap();
        assert!(!visible.internal);
        let hidden = parse(r#"{"platform":"probe","name":"X","internal":true,"pin":5}"#).unwrap();
        assert!(hidden.internal);
    }

    #[test]
    fn neither_name_nor_id_is_rejected() {
        let err = parse(r#"{"platform":"probe","pin":5}"#).expect_err("must fail");
        assert!(
            err.contains("Either 'name' or 'id'"),
            "unexpected error: {err}"
        );
    }

    #[test]
    fn unknown_fields_are_still_rejected() {
        let err = parse(r#"{"platform":"probe","name":"x","pin":5,"bogus":9}"#)
            .expect_err("unknown field must still be rejected");
        assert!(err.contains("bogus"), "unexpected error: {err}");
    }
}
