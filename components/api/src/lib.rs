use esphome_native_api::parser;
use esphome_native_api::parser::ProtoMessage;
use esphome_native_api::proto;
use esphome_native_api::proto::DeviceInfoResponse;
use esphome_native_api::proto::EntityCategory;
use esphome_native_api::proto::ListEntitiesSensorResponse;
use esphome_native_api::proto::SensorLastResetType;
use esphome_native_api::proto::SensorStateClass;
use esphome_native_api::to_packet;
use log::debug;
use log::info;
use log::warn;
use oshome_core::NoConfig;
use oshome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;

#[derive(Clone, Deserialize, Debug)]
pub struct ApiConfig {
    pub disabled: Option<bool>,
}

config_template!(api, Option<ApiConfig>, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
    components: HashMap<String, ProtoMessage>,
    device_info: DeviceInfoResponse,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        let device_info = DeviceInfoResponse {
            uses_password: false,
            name: config.oshome.name.clone(),
            // 186571EB5AFB
            // mac_address: "aa:bb:cc:dd:ee:ff".to_owned(),
            mac_address: "18:65:71:EB:5A:FB".to_owned(),
            esphome_version: "2025.4.0".to_owned(),
            compilation_time: "".to_owned(),
            model: whoami::devicename(),
            has_deep_sleep: false,
            project_name: "".to_owned(),
            project_version: "".to_owned(),
            webserver_port: 8080,
            legacy_bluetooth_proxy_version: 1,
            bluetooth_proxy_feature_flags: 0,
            manufacturer: "".to_string(),
            // format!(
            //     "{} {} {}",
            //     whoami::platform(),
            //     whoami::distro(),
            //     whoami::arch()
            // ),
            friendly_name: config
                .oshome
                .friendly_name
                .clone()
                .unwrap_or(config.oshome.name.clone()),
            legacy_voice_assistant_version: 0,
            voice_assistant_feature_flags: 0,
            suggested_area: "".to_owned(),
            bluetooth_mac_address: "".to_owned(),
        };

        Default {
            config: config,
            components: HashMap::new(),
            device_info: device_info,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<Component>, String> {
        // Does not advertise any components
        Ok(Vec::new())
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let core_config = self.config.clone();
        // let mut api_components = self.components.();
        let device_info = self.device_info.clone();
        let mut api_components = self.components.clone();
        info!("Starting API with config: {:?}", core_config.api);
        Box::pin(async move {
            while let Ok(cmd) = receiver.recv().await {
                match cmd {
                    PublishedMessage::Components { components } => {
                        for (index, component) in components.iter().enumerate() {
                            match component.clone() {
                                Component::Button(button) => {
                                    let id = button.unique_id.unwrap_or(format!(
                                        "{}_{}",
                                        core_config.oshome.name, button.name
                                    ));
                                    let component_button = ProtoMessage::ListEntitiesButtonResponse(
                                        proto::ListEntitiesButtonResponse {
                                            object_id: button.object_id.clone(),
                                            key: index.try_into().unwrap(),
                                            name: button.name,
                                            unique_id: id,
                                            icon: "".to_string(),
                                            device_class: "".to_string(), //button.device_class,
                                            disabled_by_default: false,
                                            entity_category: EntityCategory::Config as i32,
                                        },
                                    );
                                    api_components.insert(button.object_id, component_button);
                                }
                                Component::Sensor(sensor) => {
                                    let id = sensor.unique_id.unwrap_or(format!(
                                        "{}_{}",
                                        core_config.oshome.name, sensor.name
                                    ));
                                    let component_sensor = ProtoMessage::ListEntitiesSensorResponse(
                                        proto::ListEntitiesSensorResponse {
                                            object_id: sensor.object_id.clone(),
                                            key: index.try_into().unwrap(),
                                            name: sensor.name,
                                            unique_id: id,
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
                                            last_reset_type: SensorLastResetType::LastResetNone
                                                as i32,
                                            disabled_by_default: false,
                                            entity_category: EntityCategory::Config as i32,
                                        },
                                    );
                                    api_components.insert(sensor.object_id, component_sensor);
                                }
                                Component::BinarySensor(binary_sensor) => {
                                    let id = binary_sensor.unique_id.unwrap_or(format!(
                                        "{}_{}",
                                        core_config.oshome.name, binary_sensor.name
                                    ));
                                    let component_binary_sensor =
                                        ProtoMessage::ListEntitiesBinarySensorResponse(
                                            proto::ListEntitiesBinarySensorResponse {
                                                object_id: binary_sensor.object_id.clone(),
                                                key: index.try_into().unwrap(),
                                                name: binary_sensor.name,
                                                unique_id: id,
                                                icon: "".to_string(),
                                                device_class: binary_sensor
                                                    .device_class
                                                    .unwrap_or("".to_string()), //binary_sensor.device_class,
                                                is_status_binary_sensor: false,
                                                disabled_by_default: false,
                                                entity_category: EntityCategory::Config as i32,
                                            },
                                        );
                                    api_components
                                        .insert(binary_sensor.object_id, component_binary_sensor);
                                }
                            }
                        }
                    }
                    _ => {}
                }
                break;
            }

            let addr = "0.0.0.0:6053".to_string();
            let listener = TcpListener::bind(&addr).await?;
            debug!("Listening on: {}", addr);

            loop {
                // Asynchronously wait for an inbound socket.
                let (mut socket, _) = listener.accept().await?;
                debug!("Accepted request from {}", socket.peer_addr().unwrap());

                let device_info_clone = device_info.clone();
                let api_components_clone = api_components.clone();
                tokio::spawn(async move {
                    let mut buf = vec![0; 1024];

                    loop {
                        let n = socket
                            .read(&mut buf)
                            .await
                            .expect("failed to read data from socket");

                        if n == 0 {
                            return;
                        }

                        debug!("TCP: {:02X?}", &buf[0..n]);

                        let mut cursor = 0;

                        while cursor < n {
                            // Ignore first byte
                            // Get Length of packet

                            let len = buf[cursor + 1] as usize;
                            let message_type = buf[cursor + 2];
                            let packet_content = &buf[cursor + 3..cursor + 3 + len];

                            debug!("Message type: {}", message_type);
                            debug!("Message: {:?}", packet_content);

                            // TODO: Parse Frames
                            let message =
                                parser::parse_proto_message(message_type, packet_content).unwrap();

                            let mut answer_buf: Vec<u8> = vec![];
                            let mut disconnect: bool = false;
                            match message {
                                ProtoMessage::HelloRequest(hello_request) => {
                                    debug!("HelloRequest: {:?}", hello_request);
                                    let response_message = proto::HelloResponse {
                                        api_version_major: 1,
                                        api_version_minor: 10,
                                        server_info: "Rust: esphome-native-api".to_string(),
                                        name: device_info_clone.name.clone(),
                                    };
                                    debug!("HelloResponse: {:?}", response_message);

                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::HelloResponse(response_message))
                                            .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::DeviceInfoRequest(device_info_request) => {
                                    debug!("DeviceInfoRequest: {:?}", device_info_request);
                                    debug!("DeviceInfo: {:?}", device_info_clone);
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::DeviceInfoResponse(
                                            device_info_clone.clone(),
                                        ))
                                        .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::ConnectRequest(connect_request) => {
                                    debug!("ConnectRequest: {:?}", connect_request);
                                    let response_message = proto::ConnectResponse {
                                        invalid_password: false,
                                    };
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::ConnectResponse(response_message))
                                            .unwrap(),
                                    ]
                                    .concat();
                                }

                                ProtoMessage::DisconnectRequest(disconnect_request) => {
                                    debug!("DisconnectRequest: {:?}", disconnect_request);
                                    let response_message = proto::DisconnectResponse {};
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::DisconnectResponse(
                                            response_message,
                                        ))
                                        .unwrap(),
                                    ]
                                    .concat();
                                    disconnect = true;
                                }
                                ProtoMessage::ListEntitiesRequest(list_entities_request) => {
                                    debug!("ListEntitiesRequest: {:?}", list_entities_request);

                                    for (key, sensor) in &api_components_clone {
                                        debug!("Sensor: {:?}", sensor);
                                        answer_buf =
                                            [answer_buf, to_packet(sensor.clone()).unwrap()]
                                                .concat();
                                    }

                                    let response_message = proto::ListEntitiesDoneResponse {};
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::ListEntitiesDoneResponse(
                                            response_message,
                                        ))
                                        .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::PingRequest(ping_request) => {
                                    debug!("PingRequest: {:?}", ping_request);
                                    let response_message = proto::PingResponse {};
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::PingResponse(response_message))
                                            .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::SubscribeLogsRequest(request) => {
                                    debug!("SubscribeLogsRequest: {:?}", request);
                                    let response_message = proto::SubscribeLogsResponse {
                                        level: 0,
                                        message: "Test log".to_string().into_bytes(),
                                        send_failed: false,
                                    };
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::SubscribeLogsResponse(
                                            response_message,
                                        ))
                                        .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::SubscribeBluetoothLeAdvertisementsRequest(request) => {
                                    debug!("SubscribeBluetoothLeAdvertisementsRequest: {:?}", request);
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
                                ProtoMessage::UnsubscribeBluetoothLeAdvertisementsRequest(request) => {
                                    debug!("UnsubscribeBluetoothLeAdvertisementsRequest: {:?}", request);
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
                                ProtoMessage::SubscribeStatesRequest(subscribe_states_request) => {
                                    debug!("SubscribeStatesRequest: {:?}", subscribe_states_request);
                                }
                                ProtoMessage::SubscribeHomeassistantServicesRequest(request) => {
                                    debug!("SubscribeHomeassistantServicesRequest: {:?}", request);
                                }
                                ProtoMessage::SubscribeHomeAssistantStatesRequest(
                                    subscribe_homeassistant_services_request,
                                ) => {
                                    debug!("SubscribeHomeAssistantStatesRequest: {:?}", subscribe_homeassistant_services_request);
                                    let response_message = proto::SubscribeHomeAssistantStateResponse {
                                        entity_id: "test".to_string(),
                                        attribute: "test".to_string(),
                                        once: true,
                                    };
                                    answer_buf = [
                                        answer_buf,
                                        to_packet(ProtoMessage::SubscribeHomeAssistantStateResponse(
                                            response_message,
                                        ))
                                        .unwrap(),
                                    ]
                                    .concat();
                                }
                                ProtoMessage::ButtonCommandRequest(button_command_request) => {
                                    debug!("ButtonCommandRequest: {:?}", button_command_request);
                                }
                                _ => {
                                    debug!("Ignore message type: {:?}", message);
                                    return;
                                }
                            }

                            debug!("Send response: {:?}", answer_buf);

                            socket
                                .write_all(&answer_buf)
                                .await
                                .expect("failed to write data to socket");

                            if disconnect {
                                debug!("Disconnecting");
                                socket.shutdown().await.expect("failed to shutdown socket");
                                break;
                            }
                            // Close the socket

                            cursor += 3 + len;
                        }
                    }
                });
            }
        })
    }
}
