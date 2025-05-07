use log::{debug, error, info, warn};
use rumqttc::{AsyncClient, ConnectionError, Event, MqttOptions, QoS, StateError};
use serde::{Deserialize, Deserializer};
use std::{
    collections::HashMap, future::{self, Future}, pin::Pin, str, sync::Arc, time::Duration
};
use tokio::{
    sync::{broadcast::{Receiver, Sender}, RwLock},
    time::sleep,
};
use ubihome_core::{
    config_template, home_assistant::sensors::Component, internal::sensors::InternalComponent,
    ChangedMessage, Module, NoConfig, PublishedMessage,
};

mod discovery;
use discovery::*;

#[derive(Clone, Deserialize, Debug)]
pub struct MqttConfig {
    pub discovery_prefix: Option<String>,
    pub broker: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

config_template!(mqtt, MqttConfig, NoConfig, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
    core: CoreConfig,
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();
        let core_config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Ok(Default {
            config: config,
            core: core_config,
        })
    }

    fn components(&mut self) -> Vec<InternalComponent>{
        let components: Vec<InternalComponent> = Vec::new();

        components
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

            // Handle Sensor Updates
            let base_topic_clone = base_topic.clone();
            let map: HashMap<String, MqttComponent> = HashMap::new();
            let all_mqtt_components= Arc::new(RwLock::new(map));

            let all_mqtt_components_clone = all_mqtt_components.clone();
            tokio::spawn(async move {
                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::Components { components } => {
                            let mut mqtt_components: HashMap<String, MqttComponent> = HashMap::new();
                            let mut topics: Vec<String> = vec![];

                            for component in components {
                                match component {
                                    // TODO: Use object_id generator
                                    // let id = sensor.unique_id.unwrap_or(format!(
                                    //     "{}_{}",
                                    //     core_config.ubihome.name, sensor.name
                                    // ));


                                    Component::Switch(switch) => {
                                        let topic =
                                            format!("{}/{}/set", base_topic_clone.clone(), switch.id.clone());
                                        topics.push(topic.clone());
                                        
                                        mqtt_components.insert(
                                            switch.id.clone(),
                                            MqttComponent::Switch(HAMqttSwitch {
                                                platform: "switch".to_string(),
                                                unique_id: switch.id.clone(),
                                                command_topic: topic,
                                                state_topic: format!(
                                                    "{}/{}",
                                                    base_topic_clone.clone(),
                                                    switch.id.clone()
                                                ),
                                                name: switch.name.clone(),
                                                object_id: switch.id.clone(),
                                            }),
                                        );
                                    }
                                    Component::Button(button) => {
                                        let topic =
                                            format!("{}/{}", base_topic_clone.clone(), button.id.clone());
                                        topics.push(topic.clone());
                                        mqtt_components.insert(
                                            button.id.clone(),
                                            MqttComponent::Button(HAMqttButton {
                                                platform: "button".to_string(),
                                                unique_id: button.id.clone(),
                                                command_topic: topic,
                                                name: button.name.clone(),
                                                object_id: button.id.clone(),
                                            }),
                                        );
                                    }
                                    Component::Sensor(sensor) => {
                                        mqtt_components.insert(
                                            sensor.id.clone(),
                                            MqttComponent::Sensor(HAMqttSensor {
                                                platform: "sensor".to_string(),
                                                icon: sensor.icon.clone(),
                                                unique_id: sensor.id.clone(),
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
                                                    base_topic_clone.clone(),
                                                    sensor.id.clone()
                                                ),
                                                object_id: sensor.id.clone(),
                                            }),
                                        );
                                    }
                                    Component::BinarySensor(sensor) => {
                                        mqtt_components.insert(
                                            sensor.id.clone(),
                                            MqttComponent::BinarySensor(HAMqttBinarySensor {
                                                platform: "binary_sensor".to_string(),
                                                icon: sensor.icon.clone(),
                                                unique_id: sensor.id.clone(),
                                                device_class: sensor
                                                    .device_class
                                                    .clone()
                                                    .unwrap_or("".to_string()),
                                                name: sensor.name.clone(),
                                                state_topic: format!(
                                                    "{}/{}",
                                                    base_topic_clone.clone(),
                                                    sensor.id.clone()
                                                ),
                                                object_id: sensor.id.clone(),
                                            }),
                                        );
                                    }
                                }
                            }
                            {
                                let mut all_mqtt_components = all_mqtt_components_clone.write().await;
                                all_mqtt_components.extend(mqtt_components.clone());
                                debug!("MQTT Components: {:?}", mqtt_components.keys());
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
                                    format!("{}/{}", base_topic_clone, key),
                                    QoS::AtMostOnce,
                                    false,
                                    value.to_string(),
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
                                    format!("{}/{}", base_topic_clone, key),
                                    QoS::AtMostOnce,
                                    false,
                                    payload,
                                )
                                .await
                            {
                                error!("{}", e)
                            }
                        }
                        PublishedMessage::SwitchStateChange { key, state } => {
                            debug!("Switch State change value published: {} = {}", key, state);

                            let payload = if state { "ON" } else { "OFF" };
                            // Handle sensor value change
                            if let Err(e) = client
                                .publish(
                                    format!("{}/{}", base_topic_clone, key),
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

            // Handle incoming messages
            let base_topic1 = base_topic.clone();
            let all_mqtt_components_clone = all_mqtt_components.clone();
            tokio::spawn(async move {
                loop {
                    let mqtt_components = all_mqtt_components_clone.read().await;
                    match eventloop.poll().await {
                        Ok(Event::Incoming(incoming)) => {
                            if let rumqttc::Packet::Publish(received_message) = incoming {
                                let topic = received_message
                                    .topic
                                    .clone()
                                    .split_off(base_topic1.clone().len() + 1).split("/").next().unwrap().to_string();
                                debug!("Received message on topic: {}", topic);
                                debug!("Available: {:?}", mqtt_components.keys());

                                let component = mqtt_components.get(&topic);
                                if let Some(component) = component {
                                    let mut msg: Option<ChangedMessage> = None;
                                    match component {
                                        MqttComponent::Switch(switch) => {
                                            msg = Some(ChangedMessage::SwitchStateChange {
                                                key: topic.to_string(),
                                                state: str::from_utf8(&received_message.payload.to_ascii_lowercase()).unwrap() == "on",
                                            })
                                        }
                                        MqttComponent::Button(button) => {
                                            msg = Some(ChangedMessage::ButtonPress {
                                                key: topic.to_string(),
                                            });
                                        }
                                        _ => {}
                                    }
                                    
                                    if let Some(msg) = msg {
                                        debug!("Received on '{}': {:?}", topic, &msg);
                                        sender.send(msg).unwrap();
                                    }
                                }
                            }
                        }

                        Ok(Event::Outgoing(_)) => {}
                        Err(e) => match e {
                            ConnectionError::Io(e_io) => match e_io.kind() {
                                std::io::ErrorKind::NetworkUnreachable => {
                                    warn!("MQTT encountered an error, but will continue running: {:?}", e_io);
                                    sleep(Duration::from_secs(60)).await;
                                    continue;
                                }
                                std::io::ErrorKind::ConnectionAborted => {
                                    warn!("MQTT encountered an error, but will continue running: {:?}", e_io);
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
                                        warn!("MQTT encountered an error, but will continue running: {:?}", io_error);
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

            // Wait indefinitely to keep the eventloop alive
            let future = future::pending();
            let () = future.await;
            error!("MQTT event loop terminated");
            Ok(())
        })
    }
}
