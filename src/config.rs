use std::collections::HashMap;

use garde::Validate;
use serde::Deserialize;
use serde::Deserializer;
use ubihome::Logger;
use ubihome_core::configuration::base::BaseEntityProperties;
use ubihome_core::{template_mapper, UbiHome};

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate)]
pub struct BaseConfig {
    #[garde(dive)]
    pub ubihome: UbiHome,

    #[garde(dive)]
    pub logger: Option<Logger>,

    #[garde(dive)]
    pub button: Option<Vec<BaseEntityProperties>>,

    #[garde(dive)]
    pub sensor: Option<Vec<BaseEntityProperties>>,

    #[garde(dive)]
    pub binary_sensor: Option<Vec<BaseEntityProperties>>,
}
