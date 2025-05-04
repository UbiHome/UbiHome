use axum::body::Body;
use axum::extract::Path;
use axum::response::sse::Event;
use axum::response::Sse;
use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::{post, get}, Json, Router};
use tokio_stream::StreamExt;
use serde::Deserialize;
use ubihome_core::internal::sensors::InternalComponent;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::env;
use std::time::Duration;
use tower_http::trace::TraceLayer;
use futures::stream::{self, Stream};
use async_stream::stream;

use log::{debug, info, warn};
use std::sync::Arc;
use tokio::net::TcpSocket;
use tower_http::timeout::TimeoutLayer;

use ubihome_core::NoConfig;
use ubihome_core::{
    config_template, ChangedMessage, Module, PublishedMessage,
};
use serde::Deserializer;
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};


#[derive(Clone, Deserialize, Debug)]
pub struct WebServerConfig {
    pub port: u16,
}

#[derive(Debug)]
struct AppState {
    receiver: Receiver<PublishedMessage>,
}

config_template!(web_server, Option<WebServerConfig>, NoConfig, NoConfig, NoConfig);

#[derive(Clone, Debug)]
pub struct Default {
    config: CoreConfig,
}

impl Default {
    pub fn new(config_string: &String) -> Self {
        let config = serde_yaml::from_str::<CoreConfig>(config_string).unwrap();

        Default { config: config }
    }
}


async fn handle_request(
    Path((id)): Path<(String)>,
    // State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    debug!("Handling request");
    (StatusCode::OK, Json("state.current_directory".clone()))
}

async fn events_stream(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {

    let test = tokio_stream::wrappers::BroadcastStream::new(state.receiver.resubscribe()).filter_map(|m| {
        warn!("Received message");
        match m {
            Ok(msg) => {
                match msg {
                    PublishedMessage::ButtonPressed { key } => {
                        Some(Event::default().event("state").data("{\"state\": \"ok\"}"))
                    }
                    PublishedMessage::SensorValueChanged { key, value } => {
                        Some(Event::default().event("state").data(format!("{{\"state\": \"{}\"}}", key )))
                    }
                    _ => None //Event::default().event("state").data("{\"state\": \"ok\"}"),
                }
            }
            Err(_) => None //Event::default().event("state").data("{\"state\": \"ok\"}"),
        }
    }).map(|v| Ok::<_, Infallible>(v));

    debug!("connected");
    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    // let s = stream::repeat_with(|| Event::default().event("state").data("{\"state\": \"ok\"}"))
    //     .map(Ok)
    //     .throttle(Duration::from_secs(5));

    // let stream = stream::repeat_with(|| Event::default().data("hi!"))
    //     .map(Ok)
    //     .throttle(Duration::from_secs(1));

    Sse::new(test).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("ping")
    );
}

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<InternalComponent>, String> {
        let components: Vec<InternalComponent> = Vec::new();

        Ok(components)
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
            });

            // let assets_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets");
            // let static_files_service = ServeDir::new(assets_dir);


            
            let app = Router::new()
            // .route("/", get(handle_request))
            .route("/buttons/{id}", get(handle_request))
            .route("/sensors/{id}", get(handle_request))
            .route("/binary_sensors/{id}", get(handle_request))
            // .route("/{domain}/{id}/{action}", post(handle_request))
            .route("/events", get(events_stream))
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
            axum::serve(listener, app)
                .await
                .unwrap();
        
            Ok(())
        })
    }
}
