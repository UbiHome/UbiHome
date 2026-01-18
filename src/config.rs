use garde::Validate;
use serde::Deserialize;
use ubihome::Logger;
use ubihome_core::configuration::base::BaseConfigContext;
use ubihome_core::configuration::base::BaseEntityProperties;
use ubihome_core::UbiHome;

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(context(BaseConfigContext as ctx))]
pub struct BaseConfig {
    #[garde(dive(&()))]
    pub ubihome: UbiHome,

    #[garde(dive(&()))]
    pub logger: Option<Logger>,

    #[garde(dive)]
    pub button: Option<Vec<BaseEntityProperties>>,
    #[garde(dive)]
    pub sensor: Option<Vec<BaseEntityProperties>>,

    #[garde(dive)]
    pub binary_sensor: Option<Vec<BaseEntityProperties>>,
}
