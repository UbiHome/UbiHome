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
        // TODO: Add? #[serde(deny_unknown_fields)]

        $vis struct $name {
            #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
            pub id: Option<String>,

            #[garde(custom(is_readable_string), length(min = 3, max = 100))]
            pub name: String,

            // #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
            // pub platform: String,

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
        }
    };
    // // Variant for structs with no additional fields
    // (
    //     $(#[$meta:meta])*
    //     $vis:vis struct $name:ident {}
    // ) => {
    //     $(#[$meta])*
    //     $vis struct $name {
    //         #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
    //         pub id: Option<String>,

    //         #[garde(custom(is_readable_string_with_context), length(min = 3, max = 100))]
    //         pub name: String,

    //         #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
    //         pub platform: String,
    //     }

    //     impl $name {
    //         pub fn get_object_id(&self) -> String {
    //             $crate::utils::format_id(&self.id, &self.name)
    //         }
    //     }
    // };
}

// with_base_entity_properties! {
//     #[derive(Clone, Debug, Deserialize, Validate)]
//     #[garde(context(BaseConfigContext as ctx))]
//     pub struct BaseEntity {
//         #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
//         pub platform: String,
//     }
// }

// #[derive(Clone, Debug, Deserialize, Validate)]
// #[garde(context(BaseConfigContext as ctx))]
// pub struct BaseEntityProperties {
//     #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
//     id: Option<String>,

//     #[garde(custom(is_readable_string_with_context), length(min = 3, max = 100))]
//     name: String,
//     #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
//     pub platform: String,
// }

// #[derive(Clone, Debug, Deserialize, Validate)]
// #[garde(transparent)]
// #[garde(context(BaseConfigContext as ctx))]
// pub struct BaseEntity {
//     #[serde(flatten)]
//     #[garde(dive)]
//     pub default: BaseEntityProperties,
// }

// impl BaseEntityProperties {
//     pub fn get_object_id(&self) -> String {
//         format_id(&self.id, &self.name)
//     }
// }
