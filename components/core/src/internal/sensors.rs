use crate::{binary_sensor::BinarySensorBase, home_assistant::sensors::{UbiBinarySensor, UbiButton, UbiEvent, UbiSensor, UbiSwitch}, sensor::SensorBase};

#[derive(Clone, Debug)]
pub enum InternalComponent {
    Button(InternalButton),
    Sensor(InternalSensor),
    BinarySensor(InternalBinarySensor),
    Switch(InternalSwitch),
    Event(InternalEvent),
}

#[derive(Clone, Debug)]
pub struct InternalButton {
    pub ha: UbiButton,
}

// https://developers.home-assistant.io/docs/core/entity/sensor/
#[derive(Clone, Debug)]
pub struct InternalSensor {
    pub ha: UbiSensor,
    pub base: SensorBase,
}

#[derive(Clone, Debug)]
pub struct InternalBinarySensor {
    pub ha: UbiBinarySensor,
    pub base: BinarySensorBase,
}

#[derive(Clone, Debug)]
pub struct InternalSwitch {
    pub ha: UbiSwitch,
    // pub filters: Option<Vec<BinarySensorFilter>>,
}

#[derive(Clone, Debug)]
pub struct InternalEvent {
    pub ha: UbiEvent,
    // pub filters: Option<Vec<BinarySensorFilter>>,
}