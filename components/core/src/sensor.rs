use serde::Deserialize;

use crate::utils::format_id;

#[derive(Clone, Deserialize, Debug)]
pub enum SensorFilterType {
    round(usize)
}


#[derive(Clone, Deserialize, Debug)]
#[serde(deny_unknown_fields)]

pub struct SensorFilter {
    #[serde(flatten)]
    pub filter: SensorFilterType,
}

#[derive(Clone, Deserialize, Debug)]
pub struct SensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub state_class: Option<String>,

    pub filters: Option<Vec<SensorFilter>>,

}

// TODO implement as procedural macro
impl SensorBase {
    pub fn get_object_id(&self) -> String {
        format_id(&self.id, &self.name)
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownSensor{}
