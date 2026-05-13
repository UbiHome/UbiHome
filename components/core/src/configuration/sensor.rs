use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum SensorFilterType {
    Round(usize),
}

#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]

pub struct SensorFilter {
    #[serde(flatten)]
    pub filter: SensorFilterType,
}
