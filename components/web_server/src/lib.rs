use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Deserialize;
use std::env;
use std::time::Duration;

use log::{debug, info};
use std::sync::Arc;
use tokio::net::TcpListener;
use tower_http::timeout::TimeoutLayer;
use tower_http::trace::TraceLayer;

#[derive(Clone, Deserialize, Debug)]
pub struct WebServerConfig {
    pub port: u8,
}

#[derive(Clone, Debug)]
struct AppState {
    current_directory: String,
}


async fn handle_request(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    debug!("Handling request");
    (StatusCode::OK, Json(state.current_directory.clone()))
}


use oshome_core::NoConfig;
use oshome_core::{
    config_template, home_assistant::sensors::Component, ChangedMessage, Module, PublishedMessage,
};
use serde::Deserializer;
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};


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

impl Module for Default {
    fn validate(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn init(&mut self) -> Result<Vec<Component>, String> {
        let components: Vec<Component> = Vec::new();

        Ok(components)
    }

    fn run(
        &self,
        _sender: Sender<ChangedMessage>,
        _: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        info!("Starting web_server with config: {:?}", config.web_server);
        Box::pin(async move {
            let bind_address = "0.0.0.0:8080";

            let app_state = Arc::new(AppState {
                current_directory: env::current_dir().unwrap().to_string_lossy().to_string(),
            });
        
            let app = Router::new()
            .route("/", get(handle_request))
            .layer((
                TraceLayer::new_for_http(),
                // Graceful shutdown will wait for outstanding requests to complete. Add a timeout so
                // requests don't hang forever.
                TimeoutLayer::new(Duration::from_secs(10)),
            ))
            .with_state(app_state.clone());
        
            let listener = TcpListener::bind(bind_address).await.unwrap();
            axum::serve(listener, app)
                // .with_graceful_shutdown(shutdown_signal())
                .await
                .unwrap();
        
            info!("Listening on http://{}", bind_address);
            Ok(())
        })
    }
}
