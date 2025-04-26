use serde::Deserialize;


#[derive(Clone, Deserialize, Debug)]
pub struct SensorBase {
    pub id: Option<String>,
    pub name: String,
    pub icon: Option<String>,
    pub device_class: Option<String>,
    pub unit_of_measurement: Option<String>,
    pub state_class: Option<String>,

}

// TODO implement as trait for all sensors
impl SensorBase {
    pub fn get_object_id(&self, base_name: &String) -> String {
        format!(
            "{}_{}",
            base_name,
            // TODO: convert to snake case
            self.id.clone().unwrap_or(self.name.clone())
        )
    }
}

#[derive(Clone, Deserialize, Debug)]
pub struct UnknownSensor{}
