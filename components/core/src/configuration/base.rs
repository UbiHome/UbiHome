use duration_str::deserialize_duration;
use garde::Validate;
use std::{collections::HashMap, time::Duration};

use serde::Deserialize;

use crate::{
    constants::{is_id_string_option, is_readable_string},
    utils::format_id,
};

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct BaseEntityProperties {
    #[garde(custom(is_id_string_option), length(min = 3, max = 100))]
    id: Option<String>,

    #[garde(custom(is_readable_string), length(min = 3, max = 100))]
    name: String,
    #[garde(length(min = 3, max = 100))]
    pub platform: String,
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[garde(transparent)]
pub struct BaseEntity {
    #[serde(flatten)]
    #[garde(dive)]
    pub default: BaseEntityProperties,
}

impl BaseEntityProperties {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}
