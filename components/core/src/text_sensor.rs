#[macro_export]
macro_rules! template_text_sensor {
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

                $vis struct $name {
                    $(
                        $(#[$field_meta])*
                        $field_vis $field_name : $field_type,
                    )*
                }
            }

    };
}
