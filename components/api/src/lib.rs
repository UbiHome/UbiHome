use greeter::{ConnectRequest, HelloRequest, HelloResponse};
use log::info;
use parser::ProtoMessage;
use prost::Message;
use std::{default, str};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tonic::transport::Server;
use tonic::{service::InterceptorLayer, Request, Status};
use tower::ServiceBuilder;
mod parser;

include!(concat!(env!("OUT_DIR"), "/_.rs"));
pub mod greeter {
    include!(concat!(env!("OUT_DIR"), "/greeter.rs"));
}

fn some_other_interceptor(request: Request<()>) -> Result<Request<()>, Status> {
    info!("test");
    println!("A Request");
    Ok(request)
}

pub async fn start() -> Result<(), Box<dyn std::error::Error>> {
    let addr = "0.0.0.0:6053".to_string();

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

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

                println!("TCP: {:02X?}", &buf[0..n]);
                let request_content = &buf[3..n];

                let message = parser::parse_proto_message(buf[2], request_content).unwrap();

                let response_type: ProtoMessage;
                let response_content: Vec<u8>;
                match message {
                    ProtoMessage::HelloRequest(hello_request) => {
                        println!(
                            "APIVersion: {}.{} from {}",
                            hello_request.api_version_major,
                            hello_request.api_version_minor,
                            hello_request.client_info
                        );
                        println!("HelloRequest: {:?}", hello_request);
                        let response_message = greeter::HelloResponse {
                            api_version_major: 1,
                            api_version_minor: 10,
                            server_info: "Hello from Rust gRPC server".to_string(),
                            name: "Coool".to_string(),
                        };
                        response_content = response_message.encode_to_vec();

                        response_type = ProtoMessage::HelloResponse(response_message);
                    }
                    ProtoMessage::DeviceInfoRequest(device_info_request) => {
                        println!("DeviceInfoRequest: {:?}", device_info_request);
                        let response_message = greeter::DeviceInfoResponse {
                            uses_password: false,
                            name: "Hello".to_owned(),
                            mac_address: "aa:bb:cc:dd:ee:ff".to_owned(),
                            esphome_version: "Hello".to_owned(),
                            compilation_time: "Hello".to_owned(),
                            model: "Hello".to_owned(),
                            has_deep_sleep: false,
                            project_name: "Hello".to_owned(),
                            project_version: "Hello".to_owned(),
                            webserver_port: 8080,
                            legacy_bluetooth_proxy_version: 1,
                            bluetooth_proxy_feature_flags: 0,
                            manufacturer: "Hello".to_owned(),
                            friendly_name: "Hello".to_owned(),
                            legacy_voice_assistant_version: 0,
                            voice_assistant_feature_flags: 0,
                            suggested_area: "Hello".to_owned(),
                            bluetooth_mac_address: "Hello".to_owned(),
                        };
                        response_content = response_message.encode_to_vec();

                        response_type = ProtoMessage::DeviceInfoResponse(response_message);
                    }
                    ProtoMessage::DisconnectRequest(disconnect_request) => {
                        println!("DisconnectRequest: {:?}", disconnect_request);
                        let response_message = greeter::DisconnectResponse {};
                        response_content = response_message.encode_to_vec();

                        response_type = ProtoMessage::DisconnectResponse(response_message);
                    }

                    _ => {
                        println!("Ignore message type: {:?}", message);
                        return;
                    }
                }

                let message_type = parser::message_to_num(response_type).unwrap();
                let zero: Vec<u8> = vec![0];
                let length: Vec<u8> = vec![response_content.len().try_into().unwrap()];
                let message_bit: Vec<u8> = vec![message_type];

                let answer_buf: Vec<u8> = [zero, length, message_bit, response_content].concat();

                socket
                    .write_all(&answer_buf)
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
