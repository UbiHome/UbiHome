use log::debug;
use serde::Deserialize;
use rumqttc::{tokio_rustls::client, AsyncClient, Event, MqttOptions, QoS};
use std::time::Duration;

#[derive(Clone, Deserialize, Debug)]
pub struct MqttConfig {
    pub discovery_prefix: Option<String>,
    pub broker: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}



pub async fn start_mqtt_client(device_name: &String, config: &MqttConfig) {
    let mut mqttoptions = MqttOptions::new(
        device_name,
        config.broker.clone(),
        config.port.unwrap_or(1883),
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let Some(username) = config.username.clone() {
        if let Some(password) = config.password.clone() {
            mqttoptions.set_credentials(username, password);
        }
    }
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    let base_topic = format!("{}/{}", config.discovery_prefix.clone().unwrap_or("os-home".to_string()), device_name);
    let discovery_topic = format!("homeassistant/device/{}/config", device_name);
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
                }}
            }}
        }}"#,
        device_name, format!("{} {} {}", whoami::platform(), whoami::distro(), whoami::arch()), device_name, whoami::devicename(), base_topic
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
    


    // Publish a test message
    client.publish(format!("{}/{}", base_topic, "test"), QoS::AtMostOnce, false, "Hello MQTT!").await.unwrap();

    // Handle incoming messages
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(incoming)) => {
                    println!("Incoming: {:?}", incoming);
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