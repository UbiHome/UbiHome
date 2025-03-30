use std::net::TcpListener;
use std::time::Duration;
use greeter::api_connection_server::ApiConnection;
use greeter::api_connection_server::ApiConnectionServer;
use log::info;
use tonic::transport::Server;
use tower::ServiceBuilder;
use tonic::{Request, Status, service::InterceptorLayer};

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
    let addr = "0.0.0.0:6053".parse()?;
    let greeter = MyAPIConnection::default();

    let service = ApiConnectionServer::new(greeter);

    println!("Starting gRPC Server...");

    let layer = ServiceBuilder::new()
        .layer(InterceptorLayer::new(some_other_interceptor))
        .into_inner();

    Server::builder()
        .layer(layer)
        .add_service(service)
        .serve(addr)
        .await?;

    Ok(())
}

#[derive(Debug, Default)]
pub struct MyAPIConnection {}

// https://github.com/peterkeen/aioesphomeserver/blob/main/aioesphomeserver/native_api_server.py#L50

//Implement the service function(s) defined in the proto
//for the Greeter service (SayHello...)
#[tonic::async_trait]
impl ApiConnection for MyAPIConnection {
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

