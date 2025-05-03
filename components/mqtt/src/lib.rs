use log::{debug, error, info, warn};
use rumqttc::{AsyncClient, ConnectionError, Event, MqttOptions, QoS, StateError};
use serde::{Deserialize, Deserializer, Serialize};
use std::{
    collections::HashMap,
    future::{self, Future},
    pin::Pin,
    str,
    time::Duration,
};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time::sleep,
};
use ubihome_core::{
    config_template, home_assistant::sensors::Component, internal::sensors::InternalComponent,
    ChangedMessage, Module, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug)]
pub struct MqttConfig {
    pub discovery_prefix: Option<String>,
    pub broker: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum MqttComponent {
    Button(HAMqttButton),
    Sensor(HAMqttSensor),
    BinarySensor(HAMqttBinarySensor),
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct HAMqttButton {
    #[serde(rename = "p")]
    platform: String,
    unique_id: String,
    command_topic: String,
    name: String,
    object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct HAMqttSensor {
    #[serde(rename = "p")]
    platform: String,
    #[serde(rename = "ic")]
    icon: Option<String>,
    name: String,
    device_class: String,
    unit_of_measurement: String,
    unique_id: String,
    object_id: String,
    state_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct HAMqttBinarySensor {
    #[serde(rename = "p")]
    platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "ic")]
    icon: Option<String>,
    name: String,
    device_class: String,
    unique_id: String,
    object_id: String,
    state_topic: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Device {
    identifiers: Vec<String>,
    manufacturer: String,
    name: String,
    model: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct Origin {
    name: String,
    sw: String,
    url: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct MqttDiscoveryMessage {
    device: Device,
    origin: Origin,
    components: HashMap<String, MqttComponent>,
}

#[derive(Clone, Debug)]
pub struct Default {
    config: Config,
    core: CoreConfig,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub mqtt: MqttConfig,
}

#[derive(Clone, Deserialize, Debug)]
pub struct MqttSensorConfig {
    // pub bla: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct MqttButtonConfig {
    // pub bla: String
}

config_template!(
    mqtt,
    MqttConfig,
    MqttButtonConfig,
    MqttSensorConfig,
    MqttSensorConfig
);

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<Config>(config_string).unwrap();
        let core_config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config: config,
            core: core_config,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<InternalComponent>, String> {
        let components: Vec<InternalComponent> = Vec::new();

        Ok(components)
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let core_config = self.core.clone();
        Box::pin(async move {
            let mut mqttoptions = MqttOptions::new(
                core_config.ubihome.name.clone(),
                config.mqtt.broker.clone(),
                config.mqtt.port.unwrap_or(1883),
            );
            info!(
                "MQTT {}:{}",
                config.mqtt.broker,
                config.mqtt.port.unwrap_or(1883)
            );

            mqttoptions.set_keep_alive(Duration::from_secs(5));

            if let Some(username) = config.mqtt.username.clone() {
                if let Some(password) = config.mqtt.password.clone() {
                    info!("Using MQTT username and password");
                    mqttoptions.set_credentials(username, password);
                }
            }
            let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

            let base_topic = format!(
                "{}/{}",
                config
                    .mqtt
                    .discovery_prefix
                    .clone()
                    .unwrap_or("ubihome".to_string()),
                core_config.ubihome.name
            );
            let discovery_topic =
                format!("homeassistant/device/{}/config", core_config.ubihome.name);

            // Handle incoming messages
            let base_topic1 = base_topic.clone();
            tokio::spawn(async move {
                loop {
                    match eventloop.poll().await {
                        Ok(Event::Incoming(incoming)) => {
                            if let rumqttc::Packet::Publish(publish) = incoming {
                                let topic = publish
                                    .topic
                                    .clone()
                                    .split_off(base_topic1.clone().len() + 1);
                                debug!("Received message on topic: {}", topic);
                                // if components.contains_key(&topic) {
                                // let payload = str::from_utf8(&publish.payload).unwrap_or("");
                                let msg = ChangedMessage::ButtonPress {
                                    key: topic.to_string(),
                                };

                                sender.send(msg).unwrap();
                                debug!("Received on '{}': {:?}", topic, publish.payload);
                                // }
                            }
                        }
                        Ok(Event::Outgoing(_)) => {}
                        Err(e) => match e {
                            ConnectionError::Io(e_io) => match e_io.kind() {
                                std::io::ErrorKind::NetworkUnreachable => {
                                    warn!("Network unreachable, trying again...");
                                    sleep(Duration::from_secs(60)).await;
                                    continue;
                                }
                                std::io::ErrorKind::ConnectionAborted => {
                                    warn!("Connection aborted, trying again...");
                                    sleep(Duration::from_secs(60)).await;
                                    continue;
                                }
                                _ => {
                                    error!("Network error: {:?}", e_io);
                                    break;
                                }
                            },
                            ConnectionError::MqttState(e_mqtt) => match e_mqtt {
                                StateError::Io(io_error) => match io_error.kind() {
                                    std::io::ErrorKind::NetworkUnreachable => {
                                        warn!("Network unreachable, trying again...");
                                        sleep(Duration::from_secs(60)).await;
                                        continue;
                                    }
                                    std::io::ErrorKind::ConnectionAborted => {
                                        warn!("Connection aborted, trying again...");
                                        sleep(Duration::from_secs(60)).await;
                                        continue;
                                    }
                                    _ => {
                                        error!("Network error: {:?}", io_error);
                                        break;
                                    }
                                },
                                StateError::AwaitPingResp => {
                                    warn!("Ping response not received (maybe network is down?), trying again...");
                                    sleep(Duration::from_secs(60)).await;
                                    continue;
                                }
                                _ => {
                                    error!("MqttState error: {:?}", e_mqtt);
                                    break;
                                }
                            },
                            _ => {
                                error!("Error in MQTT event loop: {:?}", e);
                                break;
                            }
                        },
                    }
                }
                error!("MQTT Receiver terminated");
            });

            // Handle Sensor Updates
            let base_topic2 = base_topic.clone();
            tokio::spawn(async move {
                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::Components { components } => {
                            let mut mqtt_components: HashMap<String, MqttComponent> =
                                HashMap::new();
                            let mut topics: Vec<String> = vec![];

                            for component in components {
                                match component {
                                    Component::Button(button) => {
                                        let id = button.unique_id.unwrap_or(format!(
                                            "{}_{}",
                                            core_config.ubihome.name, button.name
                                        ));
                                        let topic =
                                            format!("{}/{}", base_topic.clone(), id.clone());
                                        topics.push(topic.clone());
                                        mqtt_components.insert(
                                            id.clone(),
                                            MqttComponent::Button(HAMqttButton {
                                                platform: "button".to_string(),
                                                unique_id: id.clone(),
                                                command_topic: format!(
                                                    "{}/{}",
                                                    base_topic.clone(),
                                                    button.object_id.clone()
                                                ),
                                                name: button.name.clone(),
                                                object_id: button.object_id.clone(),
                                            }),
                                        );
                                    }
                                    Component::Sensor(sensor) => {
                                        let id = sensor.unique_id.unwrap_or(format!(
                                            "{}_{}",
                                            core_config.ubihome.name, sensor.name
                                        ));
                                        mqtt_components.insert(
                                            id.clone(),
                                            MqttComponent::Sensor(HAMqttSensor {
                                                platform: "sensor".to_string(),
                                                icon: sensor.icon.clone(),
                                                unique_id: id.clone(),
                                                device_class: sensor
                                                    .device_class
                                                    .clone()
                                                    .unwrap_or("".to_string()),
                                                unit_of_measurement: sensor
                                                    .unit_of_measurement
                                                    .clone()
                                                    .unwrap_or("".to_string()),
                                                name: sensor.name.clone(),
                                                state_topic: format!(
                                                    "{}/{}",
                                                    base_topic.clone(),
                                                    sensor.object_id.clone()
                                                ),
                                                object_id: sensor.object_id.clone(),
                                            }),
                                        );
                                    }
                                    Component::BinarySensor(sensor) => {
                                        let id = sensor.unique_id.unwrap_or(format!(
                                            "{}_{}",
                                            core_config.ubihome.name, sensor.name
                                        ));
                                        mqtt_components.insert(
                                            id.clone(),
                                            MqttComponent::BinarySensor(HAMqttBinarySensor {
                                                platform: "binary_sensor".to_string(),
                                                icon: sensor.icon.clone(),
                                                unique_id: id.clone(),
                                                device_class: sensor
                                                    .device_class
                                                    .clone()
                                                    .unwrap_or("".to_string()),
                                                name: sensor.name.clone(),
                                                state_topic: format!(
                                                    "{}/{}",
                                                    base_topic.clone(),
                                                    sensor.object_id.clone()
                                                ),
                                                object_id: sensor.object_id.clone(),
                                            }),
                                        );
                                    }
                                }
                            }

                            let device = Device {
                                identifiers: vec![core_config.ubihome.name.clone()],
                                manufacturer: format!(
                                    "{} {} {}",
                                    whoami::platform(),
                                    whoami::distro(),
                                    whoami::arch()
                                ),
                                name: core_config.ubihome.name.clone(),
                                model: whoami::devicename(),
                            };

                            let origin = Origin {
                                name: "ubihome".to_string(),
                                sw: "0.1".to_string(),
                                url: "https://test.com".to_string(),
                            };

                            let discovery_message = MqttDiscoveryMessage {
                                device,
                                origin,
                                components: mqtt_components.clone(),
                            };
                            let discovery_payload =
                                serde_json::to_string(&discovery_message).unwrap();

                            debug!("Publishing discovery message to topic: {}", discovery_topic);
                            debug!("Discovery payload: {}", discovery_payload);
                            client
                                .publish(
                                    &discovery_topic,
                                    QoS::AtLeastOnce,
                                    false,
                                    discovery_payload,
                                )
                                .await
                                .unwrap();

                            debug!("Discovery message published successfully");

                            // Subscribe to the discovery topic
                            for topic in topics {
                                debug!("Subscribing to topic: {}", topic);
                                client.subscribe(&topic, QoS::AtLeastOnce).await.unwrap();
                            }
                        }
                        PublishedMessage::SensorValueChanged { key, value } => {
                            debug!("Sensor value published: {} = {}", key, value);
                            // Handle sensor value change
                            if let Err(e) = client
                                .publish(
                                    format!("{}/{}", base_topic2, key),
                                    QoS::AtMostOnce,
                                    false,
                                    value,
                                )
                                .await
                            {
                                error!("{}", e)
                            }
                        }
                        PublishedMessage::BinarySensorValueChanged { key, value } => {
                            debug!("Binary Sensor value published: {} = {}", key, value);

                            let payload = if value { "ON" } else { "OFF" };
                            // Handle sensor value change
                            if let Err(e) = client
                                .publish(
                                    format!("{}/{}", base_topic2, key),
                                    QoS::AtMostOnce,
                                    false,
                                    payload,
                                )
                                .await
                            {
                                error!("{}", e)
                            }
                        }
                        _ => {}
                    }
                }
                error!("MQTT Sender terminated");
            });

            // Wait indefinitely to keep the eventloop alive
            let future = future::pending();
            let () = future.await;
            error!("MQTT event loop terminated");
            Ok(())
        })
    }
}
