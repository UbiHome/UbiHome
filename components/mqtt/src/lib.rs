use log::debug;
use os_home_core::{CoreConfig, Message};
use serde::{Deserialize, Serialize};
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use tokio::sync::broadcast::Sender;
use std::{collections::HashMap, str, time::Duration};

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
enum Component {
    Button(HAButton),
    Sensor(HASensor)
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct HAButton {
    p: String,
    unique_id: String,
    command_topic: String,
    name: String,
    object_id: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
struct HASensor {
    p: String,
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
struct DiscoveryMessage {
    device: Device,
    origin: Origin,
    components: HashMap<String, Component>,
}



pub async fn start_mqtt_client(sender: Sender<Option<Message>>, config: &CoreConfig, mqtt_config: &MqttConfig) {
    let mut mqttoptions = MqttOptions::new(
        config.oshome.name.clone(),
        mqtt_config.broker.clone(),
        mqtt_config.port.unwrap_or(1883),
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let Some(username) = mqtt_config.username.clone() {
        if let Some(password) = mqtt_config.password.clone() {
            mqttoptions.set_credentials(username, password);
        }
    }
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let base_topic = format!("{}/{}", mqtt_config.discovery_prefix.clone().unwrap_or("os-home".to_string()), config.oshome.name);
    let discovery_topic = format!("homeassistant/device/{}/config", config.oshome.name);

    let mut components: HashMap<String, Component> = HashMap::new();
    components.insert(
        "test_sensor".to_string(),
        Component::Sensor(
            HASensor {
                p: "sensor".to_string(),
                unique_id: format!("{}_{}", config.oshome.name, "test"),
                state_topic: format!("{}/button", base_topic),
                object_id: format!("{}_{}", config.oshome.name, "test"),
        })
    );

    if let Some(buttons) = config.button.clone() {
        for (key, button) in buttons {
            let id = format!("{}_{}", config.oshome.name, key.clone());
            components.insert(
                key.clone(),
                Component::Button(
                    HAButton {
                        p: "button".to_string(),
                        unique_id: id.clone(),
                        name: button.name.clone(),
                        command_topic: format!("{}/{}", base_topic, key.clone()),
                        object_id: id.clone(),
            })
            );
        }
    }

    let device = Device {
        identifiers: vec![config.oshome.name.clone()],
        manufacturer: format!("{} {} {}", whoami::platform(), whoami::distro(), whoami::arch()),
        name: config.oshome.name.clone(),
        model: whoami::devicename(),
    };

    let origin = Origin {
        name: "os-home".to_string(),
        sw: "0.1".to_string(),
        url: "https://test.com".to_string(),
    };

    let discovery_message = DiscoveryMessage {
        device,
        origin,
        components: components.clone(),
    };
    let discovery_payload = serde_json::to_string(&discovery_message).unwrap();

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
    client
        .subscribe(
            &format!("{}/#", base_topic),
            QoS::AtLeastOnce,
        )
        .await
        .unwrap();
    debug!("Subscribed to topic: {}/#", base_topic);

    // Publish a test message
    client.publish(format!("{}/{}", base_topic, "test"), QoS::AtMostOnce, false, "Hello MQTT!").await.unwrap();

    // Handle incoming messages
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(incoming)) => {
                    println!("Incoming: {:?}", incoming);
                    if let rumqttc::Packet::Publish(publish) = incoming {
                        let topic = publish.topic.clone().split_off(base_topic.len() + 1);
                        debug!("Received message on topic: {}", topic);
                        if components.contains_key(&topic) {
                            let payload = str::from_utf8(&publish.payload).unwrap_or("");
                            let msg = Message::ButtonPress {
                                key: topic.to_string(),
                            };

                            sender.send(Some(msg));
                            println!("Button command received: {:?}", publish.payload);
                        }
                    }
                }
                Ok(Event::Outgoing(_)) => {}
                Err(e) => {
                    eprintln!("Error in MQTT event loop: {:?}", e);
                    break;
                }
            }
        }
    });
}