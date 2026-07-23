use garde::Validate;
use get_fields::GetFields;
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
        if !allowed_platforms.contains(value) {
            return Err(garde::Error::new(format!(
                "Platform '{}' is not configured in the configuration file. Allowed platforms are: {}",
                value,
                allowed_platforms.join(", ")
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

pub(crate) fn is_readable_string_option_with_context(
    value: &Option<String>,
    _: &BaseConfigContext,
) -> garde::Result {
    if let Some(inner_value) = value {
        if !READABLE_RE.is_match(inner_value) {
            return Err(readable_string_error(inner_value));
        }
    }
    Ok(())
}

/// Validates that at least one of `id`/`name` is set, mirroring the rule
/// `ubihome_core::with_base_entity_properties!` enforces for platform-specific
/// configs. Entities may be configured with only an `id` (internal, not
/// exposed to connectivity components), and this top-level structural check
/// must accept that too, or it rejects id-only entities before their
/// platform-specific config is ever validated.
///
/// Wired up via `self` access (see the `garde` crate's "Context/Self access"
/// docs), same as `validate_initial_value` in `builtins::globals`, so the
/// error is reported through the same `serde_saphyr`/`garde` pipeline as
/// every other field - with a source line/column.
fn validate_name_or_id<'a>(
    id: &'a Option<String>,
    platform: &'a str,
) -> impl FnOnce(&Option<String>, &BaseConfigContext) -> garde::Result + 'a {
    move |name, _ctx| {
        if id.is_none() && name.is_none() {
            return Err(garde::Error::new(format!(
                "Either 'name' or 'id' must be provided for the '{}' component",
                platform
            )));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Validate)]
#[garde(context(BaseConfigContext as ctx))]
pub struct BaseEntity {
    #[garde(custom(is_id_string_option_with_context), length(min = 3, max = 100))]
    pub id: Option<String>,

    #[garde(
        custom(is_readable_string_option_with_context),
        custom(validate_name_or_id(&self.id, &self.platform)),
        length(min = 3, max = 100)
    )]
    pub name: Option<String>,

    #[garde(custom(only_allow_configured_platforms), length(min = 3, max = 100))]
    pub platform: String,
}

pub(crate) fn is_base_entity_property(property: &str) -> bool {
    BaseConfig::get_fields.contains(&property)
}

// Base configuration structure
#[derive(Clone, Deserialize, Debug, Validate, GetFields)]
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

    #[garde(dive)]
    pub number: Option<Vec<BaseEntity>>,

    #[garde(dive)]
    pub switch: Option<Vec<BaseEntity>>,

    #[garde(dive)]
    pub light: Option<Vec<BaseEntity>>,

    #[garde(dive)]
    pub text_sensor: Option<Vec<BaseEntity>>,
}

// Load Platforms
pub fn get_platforms_from_config(config_string: &str) -> Vec<String> {
    config_string
        .lines()
        .filter_map(|line| {
            if line.starts_with(' ')
                || line.is_empty()
                || line.starts_with('#')
                || line.starts_with('-')
            {
                None
            } else {
                line.split(':').next().map(|property| property.to_string())
            }
        })
        .filter(|platform| !is_base_entity_property(platform))
        .collect::<Vec<String>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use parameterized::parameterized;

    #[parameterized(config = {
        r#"
ubihome:
  name: "Test API Config"

api:
  port: 8053
  encryption:
    key: 'xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ='
"#, 
r#"
ubihome:
  name: "Test API Config"

api:
  port: 8053
  encryption:
    key: 'xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ='

# mdns:
"#, 
r#"
ubihome:
  name: "Test API Config"

api:
  port: 8053
  encryption:
    key: 'xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ='

button:
 - command: "echo 'Hello World'"
   platform: test
"#, 
r#"
api:
  port: 56441
text_sensor:
- command: "hostname"
  id: host_name
  name: Host Name
  platform: shell
button:
- command: echo 'Hello World!' > b2dc877d-a7b3-4342-8bc5-b31e5cb9269c.mock
  id: my_button
  name: Write Hello World
  platform: shell
ubihome:
  name: test_device
"#
    })]
    fn test_get_platforms_from_config(config: &str) {
        let platforms = get_platforms_from_config(config);
        // Check that the API config is parsed correctly
        assert_eq!(platforms, vec!["api"], "Platform should be api");
    }

    #[test]
    fn base_entity_accepts_id_only() {
        let config = r#"
ubihome:
  name: "Test Switch Id Only"

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    id: relay
    pin: 7
    restore_mode: ALWAYS_ON
"#;
        let ctx = BaseConfigContext {
            allowed_platforms: Some(vec!["gpio".to_string()]),
        };
        let no_snippet = serde_saphyr::Options {
            with_snippet: false,
            ..Default::default()
        };
        let result = serde_saphyr::from_str_with_options_context_valid::<BaseConfig>(
            config, no_snippet, &ctx,
        );
        assert!(result.is_ok(), "expected ok, got: {:?}", result.err());
    }

    #[test]
    fn base_entity_rejects_neither_name_nor_id() {
        let config = r#"
ubihome:
  name: "Test Switch Neither"

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    pin: 7
"#;
        let ctx = BaseConfigContext {
            allowed_platforms: Some(vec!["gpio".to_string()]),
        };
        let no_snippet = serde_saphyr::Options {
            with_snippet: false,
            ..Default::default()
        };
        let result = serde_saphyr::from_str_with_options_context_valid::<BaseConfig>(
            config, no_snippet, &ctx,
        );
        assert!(result.is_err(), "expected an error when neither name nor id is set");
    }
}
