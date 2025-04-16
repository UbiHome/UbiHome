// use greeter::{EntityCategory, SensorLastResetType, SensorStateClass};
use log::info;
use log::debug;
use oshome_core::NoConfig;
use oshome_core::{config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage};
// use parser::ProtoMessage;
use tokio::sync::broadcast::Receiver;
use tokio::sync::broadcast::Sender;
use std::{future::Future, pin::Pin, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
mod parser;

// include!(concat!(env!("OUT_DIR"), "/_.rs"));
// pub mod greeter {
//     include!(concat!(env!("OUT_DIR"), "/greeter.rs"));
// }


use serde::{Deserialize, Deserializer};
use std::collections::HashMap;


#[derive(Clone, Deserialize, Debug)]
pub struct ApiConfig {
    pub disabled: Option<bool>,
}

config_template!(mdns, Option<ApiConfig>, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig
} 


impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default {
            config: config,
        }
    }
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }


    fn init(&mut self) -> Result<Vec<Component>, String> {
        let components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(&self,
        _sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>{
        let config = self.config.clone();
        info!("Starting API with config: {:?}", config.mdns);
        Box::pin(async move {
            return Ok(());
            // let addr = "0.0.0.0:6053".to_string();

            // // Next up we create a TCP listener which will listen for incoming
            // // connections. This TCP listener is bound to the address we determined
            // // above and must be associated with an event loop.
            // let listener = TcpListener::bind(&addr).await?;
            // debug!("Listening on: {}", addr);
        
            // loop {
            //     // Asynchronously wait for an inbound socket.
            //     let (mut socket, _) = listener.accept().await?;
        
            //     tokio::spawn(async move {
            //         let mut buf = vec![0; 1024];
        
            //         loop {
            //             let n = socket   
            //                 .read(&mut buf)
            //                 .await
            //                 .expect("failed to read data from socket");
        
            //             if n == 0 {
            //                 return;
            //             }
        
            //             debug!("TCP: {:02X?}", &buf[0..n]);
        
            //             let mut cursor = 0;
        
            //             while cursor < n {
            //                 // Ignore first byte
            //                 // Get Length of packet
        
            //                 let len = buf[cursor + 1] as usize;
            //                 let message_type = buf[cursor + 2];
            //                 let packet_content = &buf[cursor + 3..cursor + 3 + len];
        
            //                 debug!("Message type: {}", message_type);
            //                 debug!("Message: {:?}", packet_content);
        
            //                 // TODO: Parse Frames
        
            //                 // How to decode [00, 1D, 01, 0A, 17, 48, 6F, 6D, 65, 20, 41, 73, 73, 69, 73, 74, 61, 6E, 74, 20, 32, 30, 32, 35, 2E, 33, 2E, 32, 10, 01, 18, 0A, 00, 00, 03]
            //                 // let request_content = &buf[3..n];
        
            //                 let message = parser::parse_proto_message(message_type, packet_content).unwrap();
        
            //                 let mut answer_buf: Vec<u8> = vec![];
            //                 let mut disconnect: bool = false;
            //                 match message {
            //                     ProtoMessage::HelloRequest(hello_request) => {
            //                         println!(
            //                             "APIVersion: {}.{} from {}",
            //                             hello_request.api_version_major,
            //                             hello_request.api_version_minor,
            //                             hello_request.client_info
            //                         );
            //                         println!("HelloRequest: {:?}", hello_request);
            //                         let response_message = greeter::HelloResponse {
            //                             api_version_major: 1,
            //                             api_version_minor: 10,
            //                             server_info: "Hello from Rust gRPC server".to_string(),
            //                             name: "Coool".to_string(),
            //                         };
        
            //                         answer_buf = [answer_buf, to_packet(ProtoMessage::HelloResponse(response_message)).unwrap()].concat();
            //                     }
            //                     ProtoMessage::DeviceInfoRequest(device_info_request) => {
            //                         println!("DeviceInfoRequest: {:?}", device_info_request);
            //                         let response_message = greeter::DeviceInfoResponse {
            //                             uses_password: false,
            //                             name: "Hello".to_owned(),
            //                             mac_address: "aa:bb:cc:dd:ee:ff".to_owned(),
            //                             esphome_version: "Hello".to_owned(),
            //                             compilation_time: "Hello".to_owned(),
            //                             model: "Hello".to_owned(),
            //                             has_deep_sleep: false,
            //                             project_name: "Hello".to_owned(),
            //                             project_version: "Hello".to_owned(),
            //                             webserver_port: 8080,
            //                             legacy_bluetooth_proxy_version: 1,
            //                             bluetooth_proxy_feature_flags: 0,
            //                             manufacturer: "Hello".to_owned(),
            //                             friendly_name: "Hello".to_owned(),
            //                             legacy_voice_assistant_version: 0,
            //                             voice_assistant_feature_flags: 0,
            //                             suggested_area: "Hello".to_owned(),
            //                             bluetooth_mac_address: "Hello".to_owned(),
            //                         };
            //                         answer_buf = [answer_buf, to_packet(ProtoMessage::DeviceInfoResponse(response_message)).unwrap()].concat();
        
            //                     }
            //                     ProtoMessage::ConnectRequest(connect_request) => {
            //                         println!("ConnectRequest: {:?}", connect_request);
            //                         let response_message = greeter::ConnectResponse {
            //                             invalid_password: false,
            //                         };
            //                         answer_buf = [answer_buf, to_packet(ProtoMessage::ConnectResponse(response_message)).unwrap()].concat();
        
            //                     }
        
            //                     ProtoMessage::DisconnectRequest(disconnect_request) => {
            //                         println!("DisconnectRequest: {:?}", disconnect_request);
            //                         let response_message = greeter::DisconnectResponse {};
            //                         answer_buf = [answer_buf, to_packet(ProtoMessage::DisconnectResponse(response_message)).unwrap()].concat();
            //                         disconnect = true;
            //                     }
            //                     ProtoMessage::ListEntitiesRequest(list_entities_request) => {
            //                         println!("ListEntitiesRequest: {:?}", list_entities_request);
        
            //                         let sensor = greeter::ListEntitiesSensorResponse {
            //                             object_id: "sensor_1".to_string(),
            //                             key: 1,
            //                             name: "Example Sensor".to_string(),
            //                             unique_id: "unique_sensor_1".to_string(),
            //                             icon: "mdi:thermometer".to_string(),
            //                             unit_of_measurement: "°C".to_string(),
            //                             accuracy_decimals: 2,
            //                             force_update: false,
            //                             device_class: "temperature".to_string(),
            //                             state_class: SensorStateClass::StateClassMeasurement as i32,
            //                             last_reset_type: SensorLastResetType::LastResetNone as i32,
            //                             disabled_by_default: false,
            //                             entity_category: EntityCategory::Config as i32,
            //                         };
        
        
        
            //                         let response_message = greeter::ListEntitiesDoneResponse {};
            //                         answer_buf = [
            //                             answer_buf, 
            //                             to_packet(ProtoMessage::ListEntitiesSensorResponse(sensor)).unwrap(),
            //                             to_packet(ProtoMessage::ListEntitiesDoneResponse(response_message)).unwrap()
            //                         ].concat();
            //                     }
            //                     ProtoMessage::PingRequest(ping_request) => {
            //                         println!("PingRequest: {:?}", ping_request);
            //                         let response_message = greeter::PingResponse {};
            //                         answer_buf = [answer_buf, to_packet(ProtoMessage::PingResponse(response_message)).unwrap()].concat();
            //                     }
            //                     _ => {
            //                         println!("Ignore message type: {:?}", message);
            //                         return;
            //                     }
            //                 }
        
            //                 socket
            //                     .write_all(&answer_buf)
            //                     .await
            //                     .expect("failed to write data to socket");
        
            //                 if disconnect {
            //                     debug!("Disconnecting");
            //                     socket.shutdown().await.expect("failed to shutdown socket");
            //                     break;
            //                 }
            //                 // Close the socket
        
            //                 cursor += 3 + len;
            //             }
            //         }
            //     });
            // }
        })
     }

} 




// pub fn to_packet(obj: ProtoMessage) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
//     let response_content = parser::proto_to_vec(&obj)?;
//     let message_type = parser::message_to_num(&obj)?;
//     let zero: Vec<u8> = vec![0];
//     let length: Vec<u8> = vec![response_content.len().try_into().unwrap()];
//     let message_bit: Vec<u8> = vec![message_type];

//     let answer_buf: Vec<u8> =
//         [zero, length, message_bit, response_content].concat();
//     Ok(answer_buf)
//  }

