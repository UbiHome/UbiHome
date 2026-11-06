use garde::Validate;
use serde::de::DeserializeOwned;
use serde_saphyr::{MessageFormatter, UserMessageFormatter};

pub fn validate_config<T: Validate + DeserializeOwned>(
    config_string: &str,
    config_path: &str,
) -> Result<T, String>
where
    <T as Validate>::Context: Default,
{
    let no_snippet = serde_saphyr::options! { with_snippet: false };
    let validation_result =
        serde_saphyr::from_str_with_options_valid::<T>(config_string, no_snippet);

    match validation_result {
        Ok(config) => Ok(config),
        Err(error) => {
            let formatter: &dyn MessageFormatter = &UserMessageFormatter;

            let report = serde_saphyr::miette::to_miette_report_with_formatter(
                &error,
                config_string,
                config_path,
                formatter,
            );

            Err(format!("{:?}", report))
        }
    }
}
