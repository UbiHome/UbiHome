/// Macro to add base entity properties (id, name, platform) to a struct.
///
/// # Example
/// ```
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
            #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
            pub id: Option<String>,

            #[garde(custom(is_readable_string), length(min = 3, max = 100))]
            pub name: String,


            #[garde(length(min = 3, max = 100))]
            pub platform: String,

            // #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
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
