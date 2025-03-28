use log::debug;
use os_home_core::{Config, Message};
use serde::Deserialize;
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use tokio::sync::broadcast::Sender;
use std::{str, time::Duration};

#[derive(Clone, Deserialize, Debug)]
pub struct MqttConfig {
    pub discovery_prefix: Option<String>,
    pub broker: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}



pub async fn start_mqtt_client(sender: Sender<Option<Message>>, config: &Config, mqtt_config: &MqttConfig) {
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
    let discovery_payload = format!(
        r#"{{
            "device": {{
                "identifiers": ["{}"],
                "manufacturer": "{}",
                "name": "Test: {}",
                "model": "{}"
            }},
            "origin": {{
                "name": "os-home", 
                "sw": "0.1",
                "url": "https://test.com"
            }}, 
            "components": {{
                "test": {{
                    "p": "sensor",
                    "unique_id":"test",
                    "state_topic": "{}/test"
                }},
                "button": {{
                    "p": "button",
                    "unique_id":"test_button",
                    "command_topic": "{}/button"
                }}
            }}
        }}"#,
        config.oshome.name, format!("{} {} {}", whoami::platform(), whoami::distro(), whoami::arch()), config.oshome.name, whoami::devicename(), base_topic, base_topic
    );

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
                        if publish.topic == format!("{}/button", base_topic) {
                            let payload = str::from_utf8(&publish.payload).unwrap_or("");
                            let msg = Message::ButtonPress {
                                key: payload.to_string(),
                            };

                            sender.send(Some(msg));
                            println!("Button command received: {:?}", publish.payload);
                            // Handle button command here
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