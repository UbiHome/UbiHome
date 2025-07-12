use axum::extract::Path;
use axum::response::sse::Event;
use axum::response::{Html, Sse};
use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use futures::stream::{self, Stream};
use serde::Deserialize;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::time::Duration;
use tokio_stream::StreamExt;
use tower_http::trace::TraceLayer;
use ubihome_core::internal::sensors::InternalComponent;

use log::{debug, info};
use std::sync::Arc;
use tokio::net::TcpSocket;
use tower_http::timeout::TimeoutLayer;

use serde::Deserializer;
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::NoConfig;
use ubihome_core::{config_template, ChangedMessage, Module, PublishedMessage};

#[derive(Clone, Deserialize, Debug)]
pub struct WebServerConfig {
    pub port: u16,
}

#[derive(Debug)]
struct AppState {
    receiver: Receiver<PublishedMessage>,
    config: CoreConfig,
}

config_template!(
    web_server,
    Option<WebServerConfig>,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
}

async fn handle_index_html(State(_): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("Handling request index.html");
    let index = r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset=UTF-8>
        <link rel=icon href=data:>
    </head>
    <body>
        <esp-app></esp-app>
        <script src="https://oi.esphome.io/v2/www.js"></script>
    </body>
</html>
"#;
    (StatusCode::OK, Html(index))
}

async fn handle_request(
    Path((domain, id)): Path<(String, String)>,
    State(_): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("Handling request {} {}", domain, id);
    (StatusCode::OK, Json("state.current_directory".clone()))
}

async fn handle_action(
    Path((domain, id, action)): Path<(String, String, String)>,
    State(_): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("Handling request {} {} {}", domain, id, action);
    (StatusCode::OK, Json("state.current_directory".clone()))
}

async fn events_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let config = state.config.clone();
    let ping_stream =
        stream::repeat_with(move || {
            Event::default()
                .event("ping")
                .id("1231230")
                .retry(Duration::from_secs(30))
                .data(format!(
            "{{\"title\":\"{}\",\"comment\":\"\",\"ota\":false,\"log\":true,\"lang\":\"en\"}}",
            config.ubihome.friendly_name.clone().unwrap_or(config.ubihome.name.clone())
        ))
        })
        .map(Ok)
        .take(1);

    let mut entities: Vec<String> = Vec::new();
    for (key, binary_sensor) in config.binary_sensor.unwrap() {
        let object_id = binary_sensor.default.get_object_id();
        let id = binary_sensor.default.id.unwrap_or(object_id.clone());
        entities.push(format!(
            "{{\"id\": \"{}\", \"name\": \"{}\", \"icon\": \"{}\", \"entity_category\": {}, \"state\": \"ON\"}}",
            id,
            binary_sensor.default.name,
            binary_sensor.default.icon.unwrap_or_default(),
            0
        ));
    }

    let entities_stream = stream::iter(entities)
        .map(|entity| {
            Event::default()
                .event("state")
                .data(entity)
        })
        .map(Ok)
        .take(1);

    // data:

    // event: state
    // data: {"id":"light-living_room_moodlight_left","name":"Living Room Moodlight Left","icon":"","entity_category":0,"state":"OFF","color_mode":"brightness","brightness":255,"color":{},"effects":["None"]}

    // event: state
    // data: {"id":"light-living_room_moodlight_right","name":"Living Room Moodlight Right","icon":"","entity_category":0,"state":"OFF","color_mode":"brightness","brightness":255,"color":{},"effects":["None"]}

    let events = tokio_stream::wrappers::BroadcastStream::new(state.receiver.resubscribe())
        .filter_map(|m| match m {
            Ok(msg) => match msg {
                // {"id":"sensor-plant_moisture_2","value":147.7358551,"state":"148 %"}
                PublishedMessage::ButtonPressed { key } => {
                    return Some(
                        Event::default()
                            .event("state")
                            .data(format!("{{\"id\": \"{}\"}}", key)),
                    )
                }
                PublishedMessage::SensorValueChanged { key, value } => {
                    return Some(
                        Event::default()
                            .event("state")
                            .data(format!("{{\"id\": \"{}\", \"value\": {}}}", key, value)),
                    )
                }
                PublishedMessage::BinarySensorValueChanged { key, value } => {
                    return Some(
                        Event::default()
                            .event("state")
                            .data(format!("{{\"id\": \"{}\", \"value\": {}}}", key, value)),
                    )
                }
                _ => {
                    debug!("Not handled message: {:?}", msg);
                    None
                }
            },
            Err(_) => None,
        })
        .map(|v| Ok::<_, Infallible>(v));

    Sse::new(ping_stream.merge(events).merge(entities_stream)).keep_alive(
        axum::response::sse::KeepAlive::new()
            .event(Event::default().retry(Duration::from_secs(30)))
            .interval(Duration::from_secs(60))
            .text("ping"),
    )
}

impl Module for Default {
    fn new(config_string: &String) -> Result<Self, String> {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Ok(Default { config: config })
    }

    fn components(&mut self) -> Vec<InternalComponent>{
        let components: Vec<InternalComponent> = Vec::new();

        components
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!("Starting web_server with config: {:?}", config.web_server);
        Box::pin(async move {
            let bind_address = "0.0.0.0:8080";
            let app_state = Arc::new(AppState {
                receiver: receiver,
                config: config.clone(),
            });

            // let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
            // let static_files_servicme = ServeDir::new(assets_dir);

            let app = Router::new()
                .route("/", get(handle_index_html))
                .route("/{domain}/{id}", get(handle_request))
                .route("/{domain}/{id}/{action}", post(handle_action))
                .route("/events", get(events_stream))
                .fallback(handle_index_html)
                .layer((
                    TraceLayer::new_for_http(),
                    // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                    // requests don't hang forever.
                    TimeoutLayer::new(Duration::from_secs(1)),
                ))
                .with_state(app_state.clone());

            let socket = TcpSocket::new_v4().unwrap();
            socket.set_reuseaddr(true).unwrap();
            let my_addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();

            socket.bind(my_addr).unwrap();
            let listener = socket.listen(128).unwrap();

            // let listener = TcpListener::bind(bind_address).await.unwrap();
            info!("Listening on http://{}", bind_address);
            axum::serve(listener, app).await.unwrap();

            Ok(())
        })
    }
}
