use crate::{binary_sensor::BinarySensorFilter, home_assistant::sensors::{HABinarySensor, HAButton, HASensor}};

#[derive(Clone, Debug)]
pub enum InternalComponent {
    Button(InternalButton),
    Sensor(InternalSensor),
    BinarySensor(InternalBinarySensor),
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
}
