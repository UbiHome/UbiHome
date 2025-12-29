use esphome_native_api::esphomeapi::EspHomeApi;
use esphome_native_api::parser;
use esphome_native_api::parser::ProtoMessage;
use esphome_native_api::proto::version_2025_12_1::BinarySensorStateResponse;
use esphome_native_api::proto::version_2025_12_1::BluetoothLeAdvertisementResponse;
use esphome_native_api::proto::version_2025_12_1::BluetoothServiceData;
use esphome_native_api::proto::version_2025_12_1::EntityCategory;
use esphome_native_api::proto::version_2025_12_1::LightStateResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesBinarySensorResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesButtonResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesDoneResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesLightResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesSensorResponse;
use esphome_native_api::proto::version_2025_12_1::ListEntitiesSwitchResponse;
use esphome_native_api::proto::version_2025_12_1::SensorLastResetType;
use esphome_native_api::proto::version_2025_12_1::SensorStateClass;
use esphome_native_api::proto::version_2025_12_1::SensorStateResponse;
use esphome_native_api::proto::version_2025_12_1::SubscribeHomeAssistantStateResponse;
use esphome_native_api::proto::version_2025_12_1::SubscribeLogsResponse;
use esphome_native_api::proto::version_2025_12_1::SwitchStateResponse;
use log::debug;
use log::info;
use log::trace;
use log::warn;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::num::ParseIntError;
use std::{future::Future, pin::Pin, str};
use tokio::net::TcpSocket;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use ubihome_core::features::ip::get_ip_address;
use ubihome_core::features::ip::get_network_mac_address;
use ubihome_core::internal::sensors::InternalComponent;
use ubihome_core::NoConfig;
use ubihome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};

#[derive(Clone, Deserialize, Debug)]
pub struct ApiConfig {
    pub port: Option<u16>,
    pub encryption_key: Option<String>,
    pub suggested_area: Option<String>,
}

fn mac_to_u64(mac: &str) -> Result<u64, ParseIntError> {
    let mac = mac.replace(":", "");
    u64::from_str_radix(&mac, 16)
}

config_template!(api, ApiConfig, NoConfig, NoConfig, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct UbiHomeDefault {
    config: CoreConfig,
    pub api_config: ApiConfig,
    components_by_key: HashMap<u32, ProtoMessage>,
    components_key_id: HashMap<String, u32>,
}

impl Module for UbiHomeDefault {
    fn new(config_string: &String) -> Result<Self, String> {
        match serde_yaml::from_str::<CoreConfig>(config_string) {
            Ok(config) => {
                let config_clone = config.clone();
                Ok(UbiHomeDefault {
                    config: config,
                    api_config: config_clone.api,
                    components_by_key: HashMap::new(),
                    components_key_id: HashMap::new(),
                })
            }
            Err(e) => {
                return Err(format!("Failed to parse API config: {:?}", e));
            }
        }
    }

    fn components(&mut self) -> Vec<InternalComponent> {
        Vec::new()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let ip = get_ip_address().unwrap();
        let mac = get_network_mac_address(ip).unwrap();

        let server_base = EspHomeApi::builder()
            .api_version_major(1)
            .api_version_minor(42)
            .encryption_key_opt(self.config.api.encryption_key.clone())
            .server_info("UbiHome".to_string())
            .name(self.config.ubihome.name.clone())
            .friendly_name(
                self.config
                    .ubihome
                    .friendly_name
                    .clone()
                    .unwrap_or(self.config.ubihome.name.clone()),
            )
            // .bluetooth_mac_address("18:65:71:EB:5A:FB".to_string())
            .mac(mac)
            .manufacturer(whoami::distro().to_string() + " " + &whoami::arch().to_string())
            .model(whoami::devicename())
            .legacy_voice_assistant_version(0)
            .voice_assistant_feature_flags(0)
            .legacy_bluetooth_proxy_version(1)
            .bluetooth_proxy_feature_flags(1)
            // project_name: "".to_owned(),
            // compilation_time: "".to_owned(),
            // project_version: "".to_owned(),
            .suggested_area(
                self.config
                    .api
                    .suggested_area
                    .clone()
                    .unwrap_or("".to_string()),
            )
            .build();

        let core_config = self.config.clone();
        let api_config = self.api_config.clone();
        // let mut api_components = self.components.();
        let mut api_components_by_key = self.components_by_key.clone();
        let mut api_components_key_id = self.components_key_id.clone();
        info!("Starting API with config: {:?}", core_config.api);

        Box::pin(async move {
            while let Ok(cmd) = receiver.recv().await {
                match cmd {
                    PublishedMessage::Components { components } => {
                        for (index, component) in components.iter().enumerate() {
                            match component.clone() {
                                Component::Switch(switch_entity) => {
                                    let component_switch_entity =
                                        ProtoMessage::ListEntitiesSwitchResponse(
                                            ListEntitiesSwitchResponse {
                                                object_id: switch_entity.id.clone(),
                                                key: index.try_into().unwrap(),
                                                name: switch_entity.name,
                                                device_id: 0,
                                                icon: switch_entity.icon.unwrap_or_default(),
                                                device_class: switch_entity
                                                    .device_class
                                                    .unwrap_or_default(),
                                                disabled_by_default: false,
                                                entity_category: EntityCategory::None as i32,
                                                assumed_state: switch_entity.assumed_state,
                                            },
                                        );
                                    api_components_by_key
                                        .insert(index.try_into().unwrap(), component_switch_entity);
                                    api_components_key_id.insert(
                                        switch_entity.id.clone(),
                                        index.try_into().unwrap(),
                                    );
                                }
                                Component::Button(button) => {
                                    let component_button = ProtoMessage::ListEntitiesButtonResponse(
                                        ListEntitiesButtonResponse {
                                            object_id: button.id.clone(),
                                            key: index.try_into().unwrap(),
                                            name: button.name,
                                            device_id: 0,
                                            icon: "".to_string(),
                                            device_class: "".to_string(), //button.device_class,
                                            disabled_by_default: false,
                                            entity_category: EntityCategory::None as i32,
                                        },
                                    );
                                    api_components_by_key
                                        .insert(index.try_into().unwrap(), component_button);
                                    api_components_key_id
                                        .insert(button.id.clone(), index.try_into().unwrap());
                                }
                                Component::Sensor(sensor) => {
                                    let component_sensor = ProtoMessage::ListEntitiesSensorResponse(
                                        ListEntitiesSensorResponse {
                                            object_id: sensor.id.clone(),
                                            key: index.try_into().unwrap(),
                                            name: sensor.name,
                                            device_id: 0,
                                            icon: "".to_string(),
                                            unit_of_measurement: sensor
                                                .unit_of_measurement
                                                .unwrap_or("".to_string()),
                                            accuracy_decimals: 2, //sensor.accuracy_decimals,
                                            force_update: false,
                                            device_class: sensor
                                                .device_class
                                                .unwrap_or("".to_string()), //sensor.device_class,
                                            state_class: SensorStateClass::StateClassMeasurement
                                                as i32,
                                            legacy_last_reset_type:
                                                SensorLastResetType::LastResetNone as i32,
                                            disabled_by_default: false,
                                            entity_category: EntityCategory::None as i32,
                                        },
                                    );
                                    api_components_by_key
                                        .insert(index.try_into().unwrap(), component_sensor);
                                    api_components_key_id
                                        .insert(sensor.id.clone(), index.try_into().unwrap());
                                }
                                Component::BinarySensor(binary_sensor) => {
                                    let component_binary_sensor =
                                        ProtoMessage::ListEntitiesBinarySensorResponse(
                                            ListEntitiesBinarySensorResponse {
                                                object_id: binary_sensor.id.clone(),
                                                key: index.try_into().unwrap(),
                                                name: binary_sensor.name,
                                                device_id: 0,
                                                icon: "".to_string(),
                                                device_class: binary_sensor
                                                    .device_class
                                                    .unwrap_or("".to_string()), //binary_sensor.device_class,
                                                is_status_binary_sensor: false,
                                                disabled_by_default: false,
                                                entity_category: EntityCategory::None as i32,
                                            },
                                        );
                                    api_components_by_key
                                        .insert(index.try_into().unwrap(), component_binary_sensor);
                                    api_components_key_id.insert(
                                        binary_sensor.id.clone(),
                                        index.try_into().unwrap(),
                                    );
                                }
                                Component::Light(light) => {
                                    let component_light = ProtoMessage::ListEntitiesLightResponse(
                                        ListEntitiesLightResponse {
                                            object_id: light.id.clone(),
                                            key: index.try_into().unwrap(),
                                            name: light.name,
                                            device_id: 0,
                                            icon: light.icon.unwrap_or_default(),
                                            disabled_by_default: false,
                                            entity_category: EntityCategory::None as i32,
                                            supported_color_modes: vec![], // Can be populated based on capabilities
                                            min_mireds: 153.0,
                                            max_mireds: 500.0,
                                            effects: vec![], // Light effects can be added later
                                            legacy_supports_brightness: false,
                                            legacy_supports_rgb: false,
                                            legacy_supports_white_value: false,
                                            legacy_supports_color_temperature: false,
                                        },
                                    );
                                    api_components_by_key
                                        .insert(index.try_into().unwrap(), component_light);
                                    api_components_key_id
                                        .insert(light.id.clone(), index.try_into().unwrap());
                                }
                            }
                        }
                    }
                    _ => {}
                }
                break;
            }

            let port = api_config.port.unwrap_or(6053);
            let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
            let socket = TcpSocket::new_v4().unwrap();
            socket.set_reuseaddr(true).unwrap();

            socket.bind(addr).unwrap();

            let listener = socket.listen(128).unwrap();

            // let listener = TcpListener::bind(&addr).await?;
            debug!("Listening on: {}", addr);

            loop {
                // Asynchronously wait for an inbound socket.
                let (socket, _) = listener.accept().await?;
                let mut server = server_base.clone();
                let mut receiver_clone = receiver.resubscribe();
                let api_components_key_id_clone = api_components_key_id.clone();
                let api_components_clone = api_components_by_key.clone();
                let cloned_sender = sender.clone();
                tokio::spawn({
                    async move {
                        debug!("Accepted request from {}", socket.peer_addr().unwrap());
                        let (tx, mut rx) =
                            server.start(socket).await.expect("Failed to start server");

                        let tx_clone = tx.clone();

                        // Send Messages
                        tokio::spawn(async move {
                            while let Ok(cmd) = receiver_clone.recv().await {
                                match cmd {
                                    PublishedMessage::SensorValueChanged { key, value } => {
                                        let key = api_components_key_id_clone.get(&key).unwrap();
                                        debug!("SensorValueChanged: {:?}", &value);

                                        tx_clone
                                            .send(ProtoMessage::SensorStateResponse(
                                                SensorStateResponse {
                                                    key: key.clone(),
                                                    device_id: 0,
                                                    state: value,
                                                    missing_state: false,
                                                },
                                            ))
                                            .await
                                            .unwrap();
                                    }
                                    PublishedMessage::BinarySensorValueChanged { key, value } => {
                                        let key = api_components_key_id_clone.get(&key).unwrap();
                                        debug!("SensorValueChanged: {:?}", &value);

                                        tx_clone
                                            .send(ProtoMessage::BinarySensorStateResponse(
                                                BinarySensorStateResponse {
                                                    key: key.clone(),
                                                    device_id: 0,
                                                    state: value,
                                                    missing_state: false,
                                                },
                                            ))
                                            .await
                                            .unwrap();
                                    }
                                    PublishedMessage::SwitchStateChange { key, state } => {
                                        let key = api_components_key_id_clone.get(&key).unwrap();
                                        debug!("SensorValueChanged: {:?}", &state);

                                        tx_clone
                                            .send(ProtoMessage::SwitchStateResponse(
                                                SwitchStateResponse {
                                                    key: key.clone(),
                                                    device_id: 0,
                                                    state: state,
                                                },
                                            ))
                                            .await
                                            .unwrap();
                                    }
                                    PublishedMessage::LightStateChange {
                                        key,
                                        state,
                                        brightness,
                                        red,
                                        green,
                                        blue,
                                    } => {
                                        let key = api_components_key_id_clone.get(&key).unwrap();
                                        debug!("LightStateChanged: state={:?}, brightness={:?}, rgb=({:?},{:?},{:?})", &state, &brightness, &red, &green, &blue);

                                        tx_clone
                                            .send(ProtoMessage::LightStateResponse(
                                                LightStateResponse {
                                                    key: key.clone(),
                                                    device_id: 0,
                                                    state: state,
                                                    brightness: brightness.unwrap_or(0.0),
                                                    color_mode: 1, // RGB mode, could be made configurable
                                                    color_brightness: brightness.unwrap_or(0.0),
                                                    red: red.unwrap_or(0.0),
                                                    green: green.unwrap_or(0.0),
                                                    blue: blue.unwrap_or(0.0),
                                                    white: 0.0, // Not currently supported
                                                    color_temperature: 0.0, // Not currently supported
                                                    cold_white: 0.0, // Not currently supported
                                                    warm_white: 0.0, // Not currently supported
                                                    effect: "".to_string(), // No effect currently
                                                },
                                            ))
                                            .await
                                            .unwrap();
                                    }
                                    PublishedMessage::BluetoothProxyMessage(msg) => {
                                        debug!("BluetoothProxyMessage: {:?}", &msg);
                                        let service_data: Vec<BluetoothServiceData> = msg
                                            .service_data
                                            .iter()
                                            .map(|(k, v)| BluetoothServiceData {
                                                uuid: k.to_string(),
                                                data: v.clone(),
                                                legacy_data: Vec::new(),
                                            })
                                            .collect();
                                        let manufacturer_data = msg
                                            .manufacturer_data
                                            .iter()
                                            .map(|(k, v)| BluetoothServiceData {
                                                uuid: k.to_string(),
                                                data: v.clone(),
                                                legacy_data: Vec::new(),
                                            })
                                            .collect();
                                        let test = BluetoothLeAdvertisementResponse {
                                            address: mac_to_u64(&msg.mac).unwrap(),
                                            rssi: msg.rssi.try_into().unwrap(),
                                            address_type: 1,
                                            name: msg.name.as_bytes().to_vec(),
                                            service_uuids: msg.service_uuids,
                                            service_data: service_data,
                                            manufacturer_data: manufacturer_data,
                                        };

                                        tx_clone
                                            .send(ProtoMessage::BluetoothLeAdvertisementResponse(
                                                test,
                                            ))
                                            .await
                                            .unwrap();
                                    }
                                    _ => {}
                                }
                            }
                        });

                        // Read Loop
                        tokio::spawn(async move {
                            while let Ok(message) = rx.recv().await {
                                // Process the received message
                                debug!("Received message: {:?}", message);

                                match message {
                                    ProtoMessage::ListEntitiesRequest(list_entities_request) => {
                                        debug!("ListEntitiesRequest: {:?}", list_entities_request);

                                        for (key, sensor) in &api_components_clone {
                                            tx.send(sensor.clone()).await.unwrap();
                                        }
                                        tx.send(ProtoMessage::ListEntitiesDoneResponse(
                                            ListEntitiesDoneResponse {},
                                        ))
                                        .await
                                        .unwrap();
                                    }
                                    ProtoMessage::SubscribeLogsRequest(request) => {
                                        debug!("SubscribeLogsRequest: {:?}", request);
                                        let response_message = SubscribeLogsResponse {
                                            level: 0,
                                            message: "Test log".to_string().into_bytes(),
                                        };
                                        tx.send(ProtoMessage::SubscribeLogsResponse(
                                            response_message,
                                        ))
                                        .await
                                        .unwrap();
                                    }
                                    ProtoMessage::SubscribeBluetoothLeAdvertisementsRequest(
                                        request,
                                    ) => {
                                        debug!(
                                            "SubscribeBluetoothLeAdvertisementsRequest: {:?}",
                                            request
                                        );
                                        // let response_message = proto::BluetoothLeAdvertisementResponse {
                                        //     address: u64::from_str_radix("000000000000", 16).unwrap(),
                                        //     rssi: -100,
                                        //     address_type: 0,
                                        //     // data: vec![0, 1, 2, 3, 4, 5],
                                        // };
                                        // answer_buf = [
                                        //     answer_buf,
                                        //     to_packet(ProtoMessage::BluetoothLeAdvertisementResponse(
                                        //         response_message,
                                        //     ))
                                        //     .unwrap(),
                                        // ]
                                        // .concat();
                                    }
                                    ProtoMessage::UnsubscribeBluetoothLeAdvertisementsRequest(
                                        request,
                                    ) => {
                                        debug!(
                                            "UnsubscribeBluetoothLeAdvertisementsRequest: {:?}",
                                            request
                                        );
                                        // let response_message = proto::BluetoothLeAdvertisementResponse {
                                        //     address: u64::from_str_radix("000000000000", 16).unwrap(),
                                        //     rssi: -100,
                                        //     address_type: 0,
                                        //     // data: vec![0, 1, 2, 3, 4, 5],
                                        // };
                                        // answer_buf = [
                                        //     answer_buf,
                                        //     to_packet(ProtoMessage::BluetoothLeAdvertisementResponse(
                                        //         response_message,
                                        //     ))
                                        //     .unwrap(),
                                        // ]
                                        // .concat();
                                    }
                                    ProtoMessage::SubscribeStatesRequest(
                                        subscribe_states_request,
                                    ) => {
                                        debug!(
                                            "SubscribeStatesRequest: {:?}",
                                            subscribe_states_request
                                        );
                                    }
                                    ProtoMessage::SubscribeHomeassistantServicesRequest(
                                        request,
                                    ) => {
                                        debug!(
                                            "SubscribeHomeassistantServicesRequest: {:?}",
                                            request
                                        );
                                    }
                                    ProtoMessage::SubscribeHomeAssistantStatesRequest(
                                        subscribe_homeassistant_services_request,
                                    ) => {
                                        debug!(
                                            "SubscribeHomeAssistantStatesRequest: {:?}",
                                            subscribe_homeassistant_services_request
                                        );
                                        let response_message =
                                            SubscribeHomeAssistantStateResponse {
                                                entity_id: "test".to_string(),
                                                attribute: "test".to_string(),
                                                once: true,
                                            };
                                    }
                                    ProtoMessage::ButtonCommandRequest(button_command_request) => {
                                        debug!(
                                            "ButtonCommandRequest: {:?}",
                                            button_command_request
                                        );
                                        let button = api_components_clone
                                            .get(&button_command_request.key)
                                            .unwrap();
                                        match button {
                                            ProtoMessage::ListEntitiesButtonResponse(button) => {
                                                debug!("ButtonCommandRequest: {:?}", button);
                                                let msg = ChangedMessage::ButtonPress {
                                                    key: button.object_id.clone(),
                                                };

                                                cloned_sender.send(msg).unwrap();
                                            }
                                            _ => {}
                                        }
                                    }
                                    ProtoMessage::SwitchCommandRequest(switch_command_request) => {
                                        debug!(
                                            "SwitchCommandRequest: {:?}",
                                            switch_command_request
                                        );
                                        let switch_entity = api_components_clone
                                            .get(&switch_command_request.key)
                                            .unwrap();
                                        match switch_entity {
                                            ProtoMessage::ListEntitiesSwitchResponse(
                                                switch_entity,
                                            ) => {
                                                debug!(
                                                    "switch_entityCommandRequest: {:?}",
                                                    switch_entity
                                                );
                                                let msg = ChangedMessage::SwitchStateCommand {
                                                    key: switch_entity.object_id.clone(),
                                                    state: switch_command_request.state,
                                                };

                                                cloned_sender.send(msg).unwrap();
                                            }
                                            _ => {}
                                        }
                                    }
                                    ProtoMessage::LightCommandRequest(light_command_request) => {
                                        debug!("LightCommandRequest: {:?}", light_command_request);
                                        let light_entity = api_components_clone
                                            .get(&light_command_request.key)
                                            .unwrap();
                                        match light_entity {
                                            ProtoMessage::ListEntitiesLightResponse(
                                                light_entity,
                                            ) => {
                                                debug!("LightCommandRequest: {:?}", light_entity);
                                                let msg = ChangedMessage::LightStateCommand {
                                                    key: light_entity.object_id.clone(),
                                                    state: light_command_request.state,
                                                    brightness: if light_command_request
                                                        .has_brightness
                                                    {
                                                        Some(light_command_request.brightness)
                                                    } else {
                                                        None
                                                    },
                                                    red: if light_command_request.has_rgb {
                                                        Some(light_command_request.red)
                                                    } else {
                                                        None
                                                    },
                                                    green: if light_command_request.has_rgb {
                                                        Some(light_command_request.green)
                                                    } else {
                                                        None
                                                    },
                                                    blue: if light_command_request.has_rgb {
                                                        Some(light_command_request.blue)
                                                    } else {
                                                        None
                                                    },
                                                };

                                                cloned_sender.send(msg).unwrap();
                                            }
                                            _ => {}
                                        }
                                    }
                                    _ => {
                                        debug!("Ignore message type: {:?}", message);
                                    }
                                }
                            }
                            debug!("Connection closed or error");
                        });
                    }
                });
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use esphome_native_api::proto::version_2025_12_1::ListEntitiesLightResponse;

    use super::*;

    #[test]
    fn test_api_config_parsing() {
        let config = r#"
ubihome:
  name: "Test API Config"

api:
  port: 8053
  encryption_key: 'xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ='

"#;

        let api_module = UbiHomeDefault::new(&config.to_string());
        assert!(api_module.is_ok(), "API module should parse successfully");

        let module = api_module.unwrap();

        // Check that the API config is parsed correctly
        assert_eq!(module.api_config.port, Some(8053), "Port should be 8053");
        assert_eq!(
            module.api_config.encryption_key,
            Some("xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ=".to_string()),
            "Encryption key should be xiahAckHBW7BcKEQ6mRfasIW20Md9uMh/5PjrjbAhXQ="
        );
    }

    #[test]
    fn test_api_config_defaults() {
        let config = r#"
ubihome:
  name: "Test API Config"

api: {}
"#;

        let api_module = UbiHomeDefault::new(&config.to_string());
        assert!(api_module.is_ok(), "API module should parse successfully");

        let module = api_module.unwrap();

        // Check that the API config uses defaults when empty object
        assert_eq!(
            module.api_config.port, None,
            "Port should be None (default)"
        );
    }
}
