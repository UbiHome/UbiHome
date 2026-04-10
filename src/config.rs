use garde::Validate;
use serde::Deserialize;
use ubihome::Logger;
use ubihome_core::constants::readable_string_error;
use ubihome_core::constants::ID_RE;
use ubihome_core::constants::ID_RE_REMOVER;
use ubihome_core::constants::READABLE_RE;
use ubihome_core::UbiHome;

#[derive(Default)]
pub struct BaseConfigContext {
    pub allowed_platforms: Option<Vec<String>>,
}

fn only_allow_configured_platforms(value: &String, context: &BaseConfigContext) -> garde::Result {
    if let Some(allowed_platforms) = &context.allowed_platforms {
        if !allowed_platforms.contains(&value) {
            return Err(garde::Error::new(format!(
                "Platform '{}' is not configured in the configuration file. Allowed platforms are: {}",
                &value,
                &allowed_platforms.join(", ")
            )));
        }
    }
    Ok(())
}

pub fn is_id_string_option_with_context(
    value: &Option<String>,
    _: &BaseConfigContext,
) -> garde::Result {
    if let Some(inner_value) = value {
        if !ID_RE.is_match(inner_value) {
            let invalid_values = ID_RE_REMOVER.replace_all(inner_value, "");
            return Err(garde::Error::new(format!(
                "ID must only contain letters, numbers, hyphens, and underscores, but found: {}",
                invalid_values
            )));
        }
    }
    Ok(())
}

pub(crate) fn is_readable_string_with_context(value: &str, _: &BaseConfigContext) -> garde::Result {
    if !READABLE_RE.is_match(value) {
        return Err(readable_string_error(value));
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[garde(context(BaseConfigContext as ctx))]
pub struct BaseEntity {
    #[garde(custom(is_id_string_option_with_context), length(min = 3, max = 100))]
    pub id: Option<String>,

    #[garde(custom(is_readable_string_with_context), length(min = 3, max = 100))]
    pub name: String,

    #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
    pub platform: String,
}

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(context(BaseConfigContext as ctx))]
pub struct BaseConfig {
    #[garde(dive(&()))]
    pub ubihome: UbiHome,

    #[garde(dive(&()))]
    pub logger: Option<Logger>,

    #[garde(dive)]
    pub button: Option<Vec<BaseEntity>>,
    #[garde(dive)]
    pub sensor: Option<Vec<BaseEntity>>,

    #[garde(dive)]
    pub binary_sensor: Option<Vec<BaseEntity>>,
}

// Load Platforms

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct PlatformConfig {
    #[garde(length(min = 3, max = 100))]
    pub platform: String,
}

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate)]
pub struct RootPlatformConfig {
    #[garde(dive)]
    pub button: Option<Vec<PlatformConfig>>,
    #[garde(dive)]
    pub sensor: Option<Vec<PlatformConfig>>,

    #[garde(dive)]
    pub binary_sensor: Option<Vec<PlatformConfig>>,
}

pub fn get_platforms_from_config(config_string: &String) -> Result<Vec<String>, String> {
    let config = serde_saphyr::from_str::<RootPlatformConfig>(config_string)
        .map_err(|e| format!("Failed to parse configuration: {}", e))?;

    let mut platforms = Vec::new();

    if let Some(buttons) = config.button {
        for button in buttons {
            platforms.push(button.platform);
        }
    }

    if let Some(sensors) = config.sensor {
        for sensor in sensors {
            platforms.push(sensor.platform);
        }
    }

    if let Some(binary_sensors) = config.binary_sensor {
        for binary_sensor in binary_sensors {
            platforms.push(binary_sensor.platform);
        }
    }

    // && platform != "switch"
    // && platform != "light"

    Ok(platforms)
}
