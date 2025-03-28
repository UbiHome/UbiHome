use serde::Deserialize;
use rumqttc::{AsyncClient, MqttOptions, QoS, Event};
use std::time::Duration;

#[derive(Clone, Deserialize, Debug)]
pub struct MqttConfig {
    #[serde(default = "discovery_prefix_default")]
    pub discovery_prefix: String,
    pub broker: String,
    pub port: Option<u16>,
    pub username: Option<String>,
    pub password: Option<String>,
}

fn discovery_prefix_default() -> String { "homeassistant".to_string() }

pub async fn start_mqtt_client(config: MqttConfig) {
    let mut mqttoptions = MqttOptions::new(
        "test-client",
        config.broker,
        config.port.unwrap_or(1883),
    );
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    if let Some(username) = config.username {
        if let Some(password) = config.password {
            mqttoptions.set_credentials(username, password);
        }
    }

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    // Subscribe to a test topic
    client.subscribe("test/topic", QoS::AtMostOnce).await.unwrap();

    // Publish a test message
    client.publish("test/topic", QoS::AtMostOnce, false, "Hello MQTT!").await.unwrap();

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