use crate::{binary_sensor::{BinarySensorFilter, Trigger}, home_assistant::sensors::{HABinarySensor, HAButton, HASensor, HASwitch}};

#[derive(Clone, Debug)]
pub enum InternalComponent {
    Button(InternalButton),
    Sensor(InternalSensor),
    BinarySensor(InternalBinarySensor),
    Switch(InternalSwitch),
}

#[derive(Clone, Debug)]
pub struct InternalButton {
    pub ha: HAButton,
}

// https://developers.home-assistant.io/docs/core/entity/sensor/
#[derive(Clone, Debug)]
pub struct InternalSensor {
    pub ha: HASensor,
    pub filters: Option<Vec<BinarySensorFilter>>,
}

#[derive(Clone, Debug)]
pub struct InternalBinarySensor {
    pub ha: HABinarySensor,
    pub filters: Option<Vec<BinarySensorFilter>>,
    pub on_press: Option<Trigger>,
}

#[derive(Clone, Debug)]
pub struct InternalSwitch {
    pub ha: HASwitch,
    // pub filters: Option<Vec<BinarySensorFilter>>,
}
