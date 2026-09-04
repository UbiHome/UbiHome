use log::debug;
use serde::{Deserialize, Deserializer};
use std::{collections::HashMap, future::Future, pin::Pin};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::configuration::base::EntityCategory;
use ubihome_core::internal::sensors::{UbiBinarySensor, UbiComponent};
use ubihome_core::state::StateStore;
use ubihome_core::template_binary_sensor;
use ubihome_core::with_base_entity_properties;
use ubihome_core::{config_template, ChangedMessage, Module, NoConfig, PublishedMessage};

template_binary_sensor! {
#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct StatusBinarySensorConfig {
}
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct StatusConfig {}

config_template!(
    status,
    StatusConfig,
    NoConfig,
    StatusBinarySensorConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    components: Vec<UbiComponent>,
    binary_sensor_ids: Vec<String>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        let mut components: Vec<UbiComponent> = Vec::new();
        let mut binary_sensor_ids: Vec<String> = Vec::new();

        for (_, binary_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let id = binary_sensor.get_object_id();
            components.push(UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "sensor".to_string(),
                icon: binary_sensor.icon.clone(),
                device_class: binary_sensor
                    .device_class
                    .clone()
                    .or_else(|| Some("connectivity".to_string())),
                name: binary_sensor.name.clone().unwrap_or_default(),
                internal: binary_sensor.internal,
                entity_category: binary_sensor
                    .entity_category
                    .or(Some(EntityCategory::Diagnostic)),
                id: id.clone(),
                on_press: binary_sensor.on_press.clone(),
                on_release: binary_sensor.on_release.clone(),
                filters: binary_sensor.filters.clone(),
            }));

            binary_sensor_ids.push(id);
        }

        Ok(UbiHomePlatform {
            components,
            binary_sensor_ids,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        _receiver: Receiver<PublishedMessage>,
        _state: StateStore,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let binary_sensor_ids = self.binary_sensor_ids.clone();

        Box::pin(async move {
            if binary_sensor_ids.is_empty() {
                debug!("No status binary sensors configured");
            }

            // UbiHome reaching this point means it started up successfully,
            // so the sensor is reported "on" once and left alone - there is
            // nothing to poll.
            for key in binary_sensor_ids {
                _ = sender.send(ChangedMessage::BinarySensorValueChange { key, value: true });
            }

            Ok(())
        })
    }
}
