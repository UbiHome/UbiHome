use log::info;
use prost::Message;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tonic::{Request, Status, service::InterceptorLayer};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use std::str;

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
    let greeter = MyAPIConnection::default();

    let addr = "0.0.0.0:6053".to_string();

    // Next up we create a TCP listener which will listen for incoming
    // connections. This TCP listener is bound to the address we determined
    // above and must be associated with an event loop.
    let listener = TcpListener::bind(&addr).await?;
    println!("Listening on: {}", addr);

    loop {
        // Asynchronously wait for an inbound socket.
        let (mut socket, _) = listener.accept().await?;

        // And this is where much of the magic of this server happens. We
        // crucially want all clients to make progress concurrently, rather than
        // blocking one on completion of another. To achieve this we use the
        // `tokio::spawn` function to execute the work in the background.
        //
        // Essentially here we're executing a new task to run concurrently,
        // which will allow all of our clients to be processed concurrently.

        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // In a loop, read data from the socket and write the data back.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    return;
                }
                println!("TCP: {:02X?}", &buf[0..n]);

                let hello = greeter::HelloRequest::decode(&buf[3..n]).unwrap();

                println!("APIVersion: {}.{} from {}", hello.api_version_major, hello.api_version_minor, hello.client_info);

                let test = greeter::HelloResponse {
                    api_version_major: 1,
                    api_version_minor: 10,
                    server_info: "Hello from Rust gRPC server".to_string(),
                    name: "Coool".to_string(),
                };

                let mut message = test.encode_to_vec();
                let zero: Vec<u8> = vec![0];
                let length: Vec<u8> = vec![message.len().try_into().unwrap()];
                let message_type: Vec<u8> = vec![2];


                let answer_buf: Vec<u8> = [zero, length, message_type, message].concat();

                socket
                    .write_all(&answer_buf)
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }

    Ok(())
}

#[derive(Debug, Default)]
pub struct MyAPIConnection {}

impl MyAPIConnection {
    async fn hello(
        &self,
        _request: tonic::Request<greeter::HelloRequest>,
    ) -> std::result::Result<tonic::Response<greeter::HelloResponse>, tonic::Status> {
        println!("Got a hello: {:?}", _request);
        Ok(tonic::Response::new(greeter::HelloResponse {
            api_version_major: 1, 
            api_version_minor: 10, 
            server_info: "Hello from Rust gRPC server".to_string(), 
            name: "Coool".to_string(),
        }))
    }

    async fn connect(
        &self,
        _request: tonic::Request<greeter::ConnectRequest>,
    ) -> std::result::Result<tonic::Response<greeter::ConnectResponse>, tonic::Status> {
        println!("Got a connect: {:?}", _request);
        
        Ok(tonic::Response::new(greeter::ConnectResponse { invalid_password: false }))
    }

    async fn disconnect(
        &self,
        _request: tonic::Request<greeter::DisconnectRequest>,
    ) -> std::result::Result<tonic::Response<greeter::DisconnectResponse>, tonic::Status> {
        Ok(tonic::Response::new(greeter::DisconnectResponse {}))
    }

    async fn ping(
        &self,
        _request: tonic::Request<greeter::PingRequest>,
    ) -> std::result::Result<tonic::Response<greeter::PingResponse>, tonic::Status> {
        Ok(tonic::Response::new(greeter::PingResponse {}))
    }

    async fn device_info(
        &self,
        _request: tonic::Request<greeter::DeviceInfoRequest>,
    ) -> std::result::Result<tonic::Response<greeter::DeviceInfoResponse>, tonic::Status> {
        Ok(tonic::Response::new(greeter::DeviceInfoResponse { 
            uses_password: todo!(), 
            name: todo!(), 
            mac_address: todo!(), 
            esphome_version: todo!(), 
            compilation_time: todo!(), 
            model: todo!(), 
            has_deep_sleep: todo!(), 
            project_name: todo!(), 
            project_version: todo!(), 
            webserver_port: todo!(), 
            legacy_bluetooth_proxy_version: todo!(), 
            bluetooth_proxy_feature_flags: todo!(), 
            manufacturer: todo!(), 
            friendly_name: todo!(), 
            legacy_voice_assistant_version: todo!(), 
            voice_assistant_feature_flags: todo!(), 
            suggested_area: todo!(), 
            bluetooth_mac_address: todo!() 
        }))
    }

    async fn list_entities(
        &self,
        _request: tonic::Request<greeter::ListEntitiesRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }


    async fn cover_command(
        &self,
        _request: tonic::Request<greeter::CoverCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn fan_command(
        &self,
        _request: tonic::Request<greeter::FanCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn light_command(
        &self,
        _request: tonic::Request<greeter::LightCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn switch_command(
        &self,
        _request: tonic::Request<greeter::SwitchCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn camera_image(
        &self,
        _request: tonic::Request<greeter::CameraImageRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn climate_command(
        &self,
        _request: tonic::Request<greeter::ClimateCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn number_command(
        &self,
        _request: tonic::Request<greeter::NumberCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn select_command(
        &self,
        _request: tonic::Request<greeter::SelectCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn text_command(
        &self,
        _request: tonic::Request<greeter::TextCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn siren_command(
        &self,
        _request: tonic::Request<greeter::SirenCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn button_command(
        &self,
        _request: tonic::Request<greeter::ButtonCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn lock_command(
        &self,
        _request: tonic::Request<greeter::LockCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn valve_command(
        &self,
        _request: tonic::Request<greeter::ValveCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn media_player_command(
        &self,
        _request: tonic::Request<greeter::MediaPlayerCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn date_command(
        &self,
        _request: tonic::Request<greeter::DateCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn time_command(
        &self,
        _request: tonic::Request<greeter::TimeCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn datetime_command(
        &self,
        _request: tonic::Request<greeter::DateTimeCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_bluetooth_le_advertisements(
        &self,
        _request: tonic::Request<greeter::SubscribeBluetoothLeAdvertisementsRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_device_request(
        &self,
        _request: tonic::Request<greeter::BluetoothDeviceRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_get_services(
        &self,
        _request: tonic::Request<greeter::BluetoothGattGetServicesRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_read(
        &self,
        _request: tonic::Request<greeter::BluetoothGattReadRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_write(
        &self,
        _request: tonic::Request<greeter::BluetoothGattWriteRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_read_descriptor(
        &self,
        _request: tonic::Request<greeter::BluetoothGattReadDescriptorRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_write_descriptor(
        &self,
        _request: tonic::Request<greeter::BluetoothGattWriteDescriptorRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn bluetooth_gatt_notify(
        &self,
        _request: tonic::Request<greeter::BluetoothGattNotifyRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn unsubscribe_bluetooth_le_advertisements(
        &self,
        _request: tonic::Request<greeter::UnsubscribeBluetoothLeAdvertisementsRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_voice_assistant(
        &self,
        _request: tonic::Request<greeter::SubscribeVoiceAssistantRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn alarm_control_panel_command(
        &self,
        _request: tonic::Request<greeter::AlarmControlPanelCommandRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_states(
        &self,
        _request: tonic::Request<greeter::SubscribeStatesRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_logs(
        &self,
        _request: tonic::Request<greeter::SubscribeLogsRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_homeassistant_services(
        &self,
        _request: tonic::Request<greeter::SubscribeHomeassistantServicesRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn subscribe_home_assistant_states(
        &self,
        _request: tonic::Request<greeter::SubscribeHomeAssistantStatesRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    async fn get_time(
        &self,
        _request: tonic::Request<greeter::GetTimeRequest>,
    ) -> std::result::Result<tonic::Response<greeter::GetTimeResponse>, tonic::Status> {
        Ok(tonic::Response::new(greeter::GetTimeResponse { epoch_seconds: todo!() }))
    }

    async fn execute_service(
        &self,
        _request: tonic::Request<greeter::ExecuteServiceRequest>,
    ) -> std::result::Result<tonic::Response<Void>, tonic::Status> {
        Ok(tonic::Response::new(Void {}))
    }

    // Add similar implementations for the remaining methods...
}


// MESSAGE_TYPE_TO_PROTO = {
//     1: HelloRequest,
//     2: HelloResponse,
//     3: ConnectRequest,
//     4: ConnectResponse,
//     5: DisconnectRequest,
//     6: DisconnectResponse,
//     7: PingRequest,
//     8: PingResponse,
//     9: DeviceInfoRequest,
//     10: DeviceInfoResponse,
//     11: ListEntitiesRequest,
//     12: ListEntitiesBinarySensorResponse,
//     13: ListEntitiesCoverResponse,
//     14: ListEntitiesFanResponse,
//     15: ListEntitiesLightResponse,
//     16: ListEntitiesSensorResponse,
//     17: ListEntitiesSwitchResponse,
//     18: ListEntitiesTextSensorResponse,
//     19: ListEntitiesDoneResponse,
//     20: SubscribeStatesRequest,
//     21: BinarySensorStateResponse,
//     22: CoverStateResponse,
//     23: FanStateResponse,
//     24: LightStateResponse,
//     25: SensorStateResponse,
//     26: SwitchStateResponse,
//     27: TextSensorStateResponse,
//     28: SubscribeLogsRequest,
//     29: SubscribeLogsResponse,
//     30: CoverCommandRequest,
//     31: FanCommandRequest,
//     32: LightCommandRequest,
//     33: SwitchCommandRequest,
//     34: SubscribeHomeassistantServicesRequest,
//     35: HomeassistantServiceResponse,
//     36: GetTimeRequest,
//     37: GetTimeResponse,
//     38: SubscribeHomeAssistantStatesRequest,
//     39: SubscribeHomeAssistantStateResponse,
//     40: HomeAssistantStateResponse,
//     41: ListEntitiesServicesResponse,
//     42: ExecuteServiceRequest,
//     43: ListEntitiesCameraResponse,
//     44: CameraImageResponse,
//     45: CameraImageRequest,
//     46: ListEntitiesClimateResponse,
//     47: ClimateStateResponse,
//     48: ClimateCommandRequest,
//     49: ListEntitiesNumberResponse,
//     50: NumberStateResponse,
//     51: NumberCommandRequest,
//     52: ListEntitiesSelectResponse,
//     53: SelectStateResponse,
//     54: SelectCommandRequest,
//     55: ListEntitiesSirenResponse,
//     56: SirenStateResponse,
//     57: SirenCommandRequest,
//     58: ListEntitiesLockResponse,
//     59: LockStateResponse,
//     60: LockCommandRequest,
//     61: ListEntitiesButtonResponse,
//     62: ButtonCommandRequest,
//     63: ListEntitiesMediaPlayerResponse,
//     64: MediaPlayerStateResponse,
//     65: MediaPlayerCommandRequest,
//     66: SubscribeBluetoothLEAdvertisementsRequest,
//     67: BluetoothLEAdvertisementResponse,
//     68: BluetoothDeviceRequest,
//     69: BluetoothDeviceConnectionResponse,
//     70: BluetoothGATTGetServicesRequest,
//     71: BluetoothGATTGetServicesResponse,
//     72: BluetoothGATTGetServicesDoneResponse,
//     73: BluetoothGATTReadRequest,
//     74: BluetoothGATTReadResponse,
//     75: BluetoothGATTWriteRequest,
//     76: BluetoothGATTReadDescriptorRequest,
//     77: BluetoothGATTWriteDescriptorRequest,
//     78: BluetoothGATTNotifyRequest,
//     79: BluetoothGATTNotifyDataResponse,
//     80: SubscribeBluetoothConnectionsFreeRequest,
//     81: BluetoothConnectionsFreeResponse,
//     82: BluetoothGATTErrorResponse,
//     83: BluetoothGATTWriteResponse,
//     84: BluetoothGATTNotifyResponse,
//     85: BluetoothDevicePairingResponse,
//     86: BluetoothDeviceUnpairingResponse,
//     87: UnsubscribeBluetoothLEAdvertisementsRequest,
//     88: BluetoothDeviceClearCacheResponse,
//     89: SubscribeVoiceAssistantRequest,
//     90: VoiceAssistantRequest,
//     91: VoiceAssistantResponse,
//     92: VoiceAssistantEventResponse,
//     93: BluetoothLERawAdvertisementsResponse,
//     94: ListEntitiesAlarmControlPanelResponse,
//     95: AlarmControlPanelStateResponse,
//     96: AlarmControlPanelCommandRequest,
//     97: ListEntitiesTextResponse,
//     98: TextStateResponse,
//     99: TextCommandRequest,
//     100: ListEntitiesDateResponse,
//     101: DateStateResponse,
//     102: DateCommandRequest,
//     103: ListEntitiesTimeResponse,
//     104: TimeStateResponse,
//     105: TimeCommandRequest,
//     106: VoiceAssistantAudio,
//     107: ListEntitiesEventResponse,
//     108: EventResponse,
//     109: ListEntitiesValveResponse,
//     110: ValveStateResponse,
//     111: ValveCommandRequest,
//     112: ListEntitiesDateTimeResponse,
//     113: DateTimeStateResponse,
//     114: DateTimeCommandRequest,
//     115: VoiceAssistantTimerEventResponse,
//     116: ListEntitiesUpdateResponse,
//     117: UpdateStateResponse,
//     118: UpdateCommandRequest,
//     119: VoiceAssistantAnnounceRequest,
//     120: VoiceAssistantAnnounceFinished,
//     121: VoiceAssistantConfigurationRequest,
//     122: VoiceAssistantConfigurationResponse,
//     123: VoiceAssistantSetConfiguration,
// }