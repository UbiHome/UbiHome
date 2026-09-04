use crate::builtins::{self, Globals};
use crate::components::{configure_platforms, initialize_platforms, run_platforms, Platform};
use crate::config::{get_platforms_from_config, BaseConfig, BaseConfigContext};
use crate::logger_setup;

use ubihome_core::configuration::binary_sensor::FilterType;
use ubihome_core::configuration::sensor::SensorFilterType;
use ubihome_core::internal::sensors::UbiComponent;
use ubihome_core::state::{EntityState, StateStoreWriter};
use ubihome_core::{ChangedMessage, PublishedMessage};

use futures_signals::signal::{Mutable, SignalExt};
use log::{debug, error, trace};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::sync::mpsc;
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::{runtime::Runtime, signal};

fn read_base_config(path: &str) -> Result<String, String> {
    if path.is_empty() {
        // TODO: Fallback to the embedded default configuration once wired up
        // (DEFAULT_CONFIG in main.rs isn't currently passed through to this
        // function). Until then, treat an empty path as "no config found".
        // println!("Config file path: BUILTIN");
        // DEFAULT_CONFIG
        return Err(
            "No configuration file found. Create a config.yml or config.yaml in the current \
             directory, or point to one with --configuration <path>."
                .to_string(),
        );
    }

    println!("Config: {}", path);

    let config_file_path = fs::canonicalize(path).map_err(|_| {
        format!(
            "Configuration file not found at '{}'. Create it, or point to an existing file with --configuration <path>.",
            path
        )
    })?;

    fs::read_to_string(&config_file_path).map_err(|e| {
        format!(
            "Failed to read the configuration file at '{}': {}",
            config_file_path.display(),
            e
        )
    })
}

pub(crate) fn run(
    config_path: &str,
    validate_only: bool,
    shutdown_signal: Option<mpsc::Receiver<()>>,
) -> Result<(), Box<dyn std::error::Error>> {
    let log_directory = logger_setup::default_log_directory();
    let mut logger = logger_setup::init(&log_directory);

    println!("LogDirectory: {}", log_directory.display());

    let config_string: String = read_base_config(config_path)?;

    let mut platforms = get_platforms_from_config(&config_string);
    // Builtin top-level sections (e.g. `globals`) are handled directly by the
    // main binary and must not be treated as dynamically-loaded platform crates.
    platforms.retain(|p| !builtins::BUILTIN_SECTIONS.contains(&p.as_str()));
    debug!("Configured modules: {:?}", platforms);

    if sentry::Hub::current().client().is_some() {
        sentry::configure_scope(|scope| {
            scope.set_tag("modules", platforms.join(", "));
        });
    }

    let no_snippet = serde_saphyr::Options {
        with_snippet: false,
        ..Default::default()
    };
    // Entities may reference builtin platforms (e.g. `platform: template`) that
    // have no dedicated top-level section, so allow them during validation.
    let mut allowed_platforms = platforms.clone();
    allowed_platforms.push(builtins::TEMPLATE_PLATFORM.to_string());
    let ctx = BaseConfigContext {
        allowed_platforms: Some(allowed_platforms),
    };
    let validation_result = serde_saphyr::from_str_with_options_context_valid::<BaseConfig>(
        &config_string,
        no_snippet.clone(),
        &ctx,
    );

    if let Err(errors) = validation_result {
        let report = serde_saphyr::miette::to_miette_report(&errors, &config_string, config_path);
        return Err(format!("{:?}", report).into());
    }
    let config = validation_result.unwrap();

    if let Some(logger_config) = config.logger.as_ref() {
        logger_setup::apply_config(&mut logger, &log_directory, logger_config);
    };

    debug!("BaseConfiguration: {:?}", config);

    let mut platforms_to_load: BTreeSet<Platform> = BTreeSet::new();
    println!("Platforms to load: {:?}", platforms);
    for platform in platforms.iter() {
        if let Ok(platform_enum) = Platform::from_str(platform) {
            platforms_to_load.insert(platform_enum);
        } else {
            return Err(format!(
                r#"Unknown platform specified: {}
Remove the "{}:" entry from your configuration or install the cargo crate containing the platform."#,
                platform, platform
            ).into());
        }
    }
    let configuration_result = configure_platforms(&config_string, config_path, &platforms_to_load);
    if let Err(e) = configuration_result {
        return Err(e.into());
    }
    let mut configured_platforms = configuration_result.unwrap();
    log::info!("Loaded {} modules", configured_platforms.len());
    let mut initialized_platforms = initialize_platforms(&mut configured_platforms).unwrap();

    // Builtin components (template switches/buttons/numbers, globals) are
    // parsed and wired up by the main binary itself; see `crate::builtins` for
    // the rationale.
    let builtin = builtins::parse(&config_string, config_path)?;
    initialized_platforms.extend(builtins::template::to_components(&builtin.template));

    if validate_only {
        return Ok(());
    }

    // The global entity state cache: only this function (the main application)
    // ever holds a `StateStoreWriter`. Platform modules only ever receive the
    // read-only `StateStore` handed out below via `run_platforms`.
    let (state_writer, state_store) = StateStoreWriter::new(initialized_platforms.clone());

    // Spawn the root task
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        let (internal_tx, modules_rx) = broadcast::channel::<PublishedMessage>(16);
        let (modules_tx, mut internal_rx) = broadcast::channel::<ChangedMessage>(16);

        // Supervise every long-running task (sensor/binary-sensor signal handlers,
        // the internal command router, and the platform modules) in one JoinSet so
        // a panic in any of them brings the application down instead of being
        // silently swallowed by a detached task.
        let mut supervised_tasks: tokio::task::JoinSet<()> = tokio::task::JoinSet::new();

        // Shared store for `globals:` variables, mutated by `globals.set` actions
        // executed from any trigger (binary sensor, template switch, ...).
        let globals = Globals::new(&builtin.globals);

        // Double Option Workaround for https://github.com/Pauan/rust-signals/issues/75
        let mut signal_map_binary_sensor: HashMap<String, Mutable<Option<Option<bool>>>> =
            HashMap::new();
        let mut signal_map_sensor: HashMap<String, Mutable<Option<Option<f32>>>> = HashMap::new();
        let mut signal_map_media_player_state: HashMap<String, Mutable<Option<Option<bool>>>> =
            HashMap::new();
        let mut signal_map_media_player_volume: HashMap<String, Mutable<Option<Option<f32>>>> =
            HashMap::new();
        let mut signal_map_media_player_mute: HashMap<String, Mutable<Option<Option<bool>>>> =
            HashMap::new();

        for component in initialized_platforms.clone() {
            match component {
                UbiComponent::Button(_button) => {
                    // println!("Button: {:?}", button);
                }
                UbiComponent::Sensor(sensor) => {
                    let mutable: Mutable<Option<Option<f32>>> = Mutable::new(Option::None);
                    signal_map_sensor.insert(sensor.id.clone(), mutable.clone());
                    let internal_tx_clone = internal_tx.clone();
                    let state_writer_clone = state_writer.clone();

                    let mutable_clone = mutable.clone();
                    supervised_tasks.spawn(async move {
                        // println!("Filters: {:?}", binary_sensor.filters);

                        let mut signal = mutable_clone.signal_cloned().boxed();
                        for filter in sensor.filters.unwrap_or_default() {
                            match filter.filter {
                                SensorFilterType::Round(decimals) => {
                                    trace!("round");
                                    signal = signal
                                        .map(move |value| {
                                            if let Some(v) = value.and_then(|v| v) {
                                                // let number: f64 = v.parse().unwrap();
                                                let output: f32 =
                                                    format!("{:.1$}", v, decimals).parse().unwrap();
                                                debug!("Round: {}", output);
                                                Some(Some(output))
                                            } else {
                                                value
                                            }
                                        })
                                        .boxed();
                                }
                            }
                        }

                        // React to signal changes
                        signal
                            .for_each(|value| {
                                let signal_tx_clone = internal_tx_clone.clone();

                                let key = sensor.id.clone();
                                if let Some(value) = value.and_then(|v| v) {
                                    state_writer_clone.set(key.clone(), EntityState::Sensor(value));
                                    let pcmd = PublishedMessage::SensorValueChanged { key, value };
                                    debug!("Publishing command from signal: {:?}", pcmd);

                                    signal_tx_clone.send(pcmd).unwrap();
                                }

                                async move {}
                            })
                            .await;
                    });
                }
                UbiComponent::Switch(_switch) => {
                    // println!("Switch: {:?}", switch);
                }
                UbiComponent::Light(_light) => {
                    // println!("Light: {:?}", light);
                }
                UbiComponent::Number(_number) => {
                    // Numbers don't have filters, state changes are forwarded directly
                }
                UbiComponent::TextSensor(_text_sensor) => {
                    // Text sensors are read-only, state changes are forwarded directly
                }
                UbiComponent::MediaPlayer(media_player) => {
                    // Playback state (play/pause), dispatched the same way as a
                    // binary sensor's on_press/on_release, minus filters (none
                    // are modeled for media_player).
                    let mutable_state: Mutable<Option<Option<bool>>> = Mutable::new(Option::None);
                    signal_map_media_player_state
                        .insert(media_player.id.clone(), mutable_state.clone());
                    let internal_tx_clone = internal_tx.clone();
                    let globals_clone = globals.clone();
                    let key = media_player.id.clone();
                    let on_play = media_player.on_play.clone();
                    let on_pause = media_player.on_pause.clone();

                    let mutable_clone = mutable_state.clone();
                    supervised_tasks.spawn(async move {
                        mutable_clone
                            .signal()
                            .for_each(move |value| {
                                let action_tx = internal_tx_clone.clone();
                                let signal_tx_clone = internal_tx_clone.clone();
                                let key = key.clone();
                                let on_play = on_play.clone();
                                let on_pause = on_pause.clone();
                                let globals_for_call = globals_clone.clone();
                                async move {
                                    if let Some(playing) = value.and_then(|v| v) {
                                        if playing {
                                            if let Some(on_play) = on_play {
                                                builtins::run_actions(
                                                    on_play.then,
                                                    &action_tx,
                                                    &globals_for_call,
                                                )
                                                .await;
                                            }
                                        } else if let Some(on_pause) = on_pause {
                                            builtins::run_actions(
                                                on_pause.then,
                                                &action_tx,
                                                &globals_for_call,
                                            )
                                            .await;
                                        }

                                        let pcmd = PublishedMessage::MediaPlayerStateChanged {
                                            key,
                                            playing: Some(playing),
                                            volume: None,
                                            muted: None,
                                        };
                                        debug!("Publishing command from signal: {:?}", pcmd);
                                        signal_tx_clone.send(pcmd).unwrap();
                                    }
                                }
                            })
                            .await;
                    });

                    // Volume, dispatched separately from playback state (its
                    // own signal map/task), since the two change independently.
                    let mutable_volume: Mutable<Option<Option<f32>>> = Mutable::new(Option::None);
                    signal_map_media_player_volume
                        .insert(media_player.id.clone(), mutable_volume.clone());
                    let internal_tx_clone = internal_tx.clone();
                    let globals_clone = globals.clone();
                    let key = media_player.id.clone();
                    let on_volume_change = media_player.on_volume_change.clone();

                    let mutable_clone = mutable_volume.clone();
                    supervised_tasks.spawn(async move {
                        mutable_clone
                            .signal_cloned()
                            .for_each(move |value| {
                                let action_tx = internal_tx_clone.clone();
                                let signal_tx_clone = internal_tx_clone.clone();
                                let key = key.clone();
                                let on_volume_change = on_volume_change.clone();
                                let globals_for_call = globals_clone.clone();
                                async move {
                                    if let Some(value) = value.and_then(|v| v) {
                                        if let Some(on_volume_change) = on_volume_change {
                                            builtins::run_actions(
                                                on_volume_change.then,
                                                &action_tx,
                                                &globals_for_call,
                                            )
                                            .await;
                                        }

                                        let pcmd = PublishedMessage::MediaPlayerStateChanged {
                                            key,
                                            playing: None,
                                            volume: Some(value),
                                            muted: None,
                                        };
                                        debug!("Publishing command from signal: {:?}", pcmd);
                                        signal_tx_clone.send(pcmd).unwrap();
                                    }
                                }
                            })
                            .await;
                    });

                    // Mute, dispatched separately from playback state/volume (its
                    // own signal map/task), since all three change independently.
                    let mutable_mute: Mutable<Option<Option<bool>>> = Mutable::new(Option::None);
                    signal_map_media_player_mute
                        .insert(media_player.id.clone(), mutable_mute.clone());
                    let internal_tx_clone = internal_tx.clone();
                    let globals_clone = globals.clone();
                    let key = media_player.id.clone();
                    let on_mute_change = media_player.on_mute_change.clone();

                    let mutable_clone = mutable_mute.clone();
                    supervised_tasks.spawn(async move {
                        mutable_clone
                            .signal()
                            .for_each(move |value| {
                                let action_tx = internal_tx_clone.clone();
                                let signal_tx_clone = internal_tx_clone.clone();
                                let key = key.clone();
                                let on_mute_change = on_mute_change.clone();
                                let globals_for_call = globals_clone.clone();
                                async move {
                                    if let Some(muted) = value.and_then(|v| v) {
                                        if let Some(on_mute_change) = on_mute_change {
                                            builtins::run_actions(
                                                on_mute_change.then,
                                                &action_tx,
                                                &globals_for_call,
                                            )
                                            .await;
                                        }

                                        let pcmd = PublishedMessage::MediaPlayerStateChanged {
                                            key,
                                            playing: None,
                                            volume: None,
                                            muted: Some(muted),
                                        };
                                        debug!("Publishing command from signal: {:?}", pcmd);
                                        signal_tx_clone.send(pcmd).unwrap();
                                    }
                                }
                            })
                            .await;
                    });
                }
                UbiComponent::BinarySensor(binary_sensor) => {
                    let mutable: Mutable<Option<Option<bool>>> = Mutable::new(Option::None);
                    signal_map_binary_sensor.insert(binary_sensor.id.clone(), mutable.clone());
                    let internal_tx_clone = internal_tx.clone();
                    let globals_clone = globals.clone();
                    let state_writer_clone = state_writer.clone();

                    let mutable_clone = mutable.clone();
                    supervised_tasks.spawn(async move {
                        // println!("Filters: {:?}", binary_sensor.filters);

                        let mut signal = mutable_clone.signal().boxed();
                        for filter in binary_sensor.filters.unwrap_or_default() {
                            match filter.filter {
                                FilterType::DelayedOn(time) => {
                                    trace!("delayed_on");
                                    signal = signal
                                        .map_future(move |value| {
                                            Box::pin(async move {
                                                let value = value.and_then(|v| v);
                                                if let Some(v) = value {
                                                    // Delay on (true) values
                                                    if v {
                                                        tokio::time::sleep(time).await;
                                                        value
                                                    } else {
                                                        value
                                                    }
                                                } else {
                                                    value
                                                }
                                            })
                                        })
                                        .boxed();
                                }
                                FilterType::DelayedOff(time) => {
                                    trace!("delayed_off");
                                    signal = signal
                                        .map_future(move |value| {
                                            Box::pin(async move {
                                                let value = value.and_then(|v| v);
                                                if let Some(v) = value {
                                                    // Delay off (false) values
                                                    if !v {
                                                        tokio::time::sleep(time).await;
                                                        value
                                                    } else {
                                                        value
                                                    }
                                                } else {
                                                    value
                                                }
                                            })
                                        })
                                        .boxed();
                                }
                                FilterType::Invert(_) => {
                                    signal = signal
                                        .map(|value| {
                                            trace!("invert");
                                            if value.is_some() {
                                                Some(Some(!value.and_then(|v| v).unwrap()))
                                            } else {
                                                value
                                            }
                                        })
                                        .boxed();
                                }
                            }
                        }

                        // React to signal changes
                        signal
                            .for_each(move |value| {
                                let action_tx = internal_tx_clone.clone();
                                let signal_tx_clone = internal_tx_clone.clone();
                                let state_writer_for_call = state_writer_clone.clone();

                                let key = binary_sensor.id.clone();
                                let on_press = binary_sensor.on_press.clone();
                                let on_release = binary_sensor.on_release.clone();
                                let globals_for_call = globals_clone.clone();
                                async move {
                                    if let Some(value) = value.and_then(|v| v) {
                                        if value {
                                            if let Some(on_press) = on_press {
                                                builtins::run_actions(
                                                    on_press.then,
                                                    &action_tx,
                                                    &globals_for_call,
                                                )
                                                .await;
                                            }
                                        } else if let Some(on_release) = on_release {
                                            builtins::run_actions(
                                                on_release.then,
                                                &action_tx,
                                                &globals_for_call,
                                            )
                                            .await;
                                        }

                                        state_writer_for_call
                                            .set(key.clone(), EntityState::BinarySensor(value));
                                        let pcmd = PublishedMessage::BinarySensorValueChanged {
                                            key,
                                            value,
                                        };
                                        debug!("Publishing command from signal: {:?}", pcmd);

                                        signal_tx_clone.send(pcmd).unwrap();
                                    }
                                }
                            })
                            .await;

                        // // Debounce future for "off" values
                        // future
                        //     .for_each(|value| {
                        //         // This code is run for the current value of my_state,
                        //         // and also every time my_state changes
                        //         println!("From signal: {}", value);
                        //         async {}
                        //     })
                        //     .await;
                    });
                }
            }
        }

        let internal_tx_clone = internal_tx.clone();
        let state_writer_clone = state_writer.clone();
        supervised_tasks.spawn({
            async move {
                while let Ok(cmd) = internal_rx.recv().await {
                    debug!("Received command: {:?}", cmd);
                    let publish_cmd: Option<PublishedMessage> = match cmd {
                        ChangedMessage::SwitchStateChange { key, state } => {
                            Some(PublishedMessage::SwitchStateChange { key, state })
                        }
                        ChangedMessage::SwitchStateCommand { key, state } => {
                            Some(PublishedMessage::SwitchStateCommand { key, state })
                        }
                        ChangedMessage::LightStateChange {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        } => Some(PublishedMessage::LightStateChange {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        }),
                        ChangedMessage::LightStateCommand {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        } => Some(PublishedMessage::LightStateCommand {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        }),
                        ChangedMessage::ButtonPress { key } => {
                            Some(PublishedMessage::ButtonPressed { key })
                        }
                        ChangedMessage::SensorValueChange { key, value } => {
                            if let Some(signal) = signal_map_sensor.get(&key) {
                                signal.set(Some(Some(value)));
                            }
                            None
                        }
                        ChangedMessage::BinarySensorValueChange { key, value } => {
                            debug!("BinarySensorValueChange: {}", value);
                            if let Some(signal) = signal_map_binary_sensor.get(&key) {
                                signal.set(Some(Some(value)));
                            }
                            None
                        }
                        ChangedMessage::BluetoothProxyMessage(msg) => {
                            Some(PublishedMessage::BluetoothProxyMessage(msg))
                        }
                        ChangedMessage::NumberValueChange { key, value } => {
                            Some(PublishedMessage::NumberValueChanged { key, value })
                        }
                        ChangedMessage::NumberValueCommand { key, value } => {
                            Some(PublishedMessage::NumberValueCommand { key, value })
                        }
                        ChangedMessage::TextSensorValueChange { key, value } => {
                            Some(PublishedMessage::TextSensorValueChanged { key, value })
                        }
                        ChangedMessage::MediaPlayerStateChange {
                            key,
                            playing,
                            volume,
                            muted,
                        } => {
                            if let Some(playing) = playing {
                                if let Some(signal) = signal_map_media_player_state.get(&key) {
                                    signal.set(Some(Some(playing)));
                                }
                            }
                            if let Some(volume) = volume {
                                if let Some(signal) = signal_map_media_player_volume.get(&key) {
                                    signal.set(Some(Some(volume)));
                                }
                            }
                            if let Some(muted) = muted {
                                if let Some(signal) = signal_map_media_player_mute.get(&key) {
                                    signal.set(Some(Some(muted)));
                                }
                            }
                            None
                        }
                    };
                    if let Some(pcmd) = publish_cmd {
                        match &pcmd {
                            PublishedMessage::SwitchStateChange { key, state } => {
                                state_writer_clone.set(key.clone(), EntityState::Switch(*state));
                            }
                            PublishedMessage::LightStateChange {
                                key,
                                state,
                                brightness,
                                red,
                                green,
                                blue,
                            } => {
                                state_writer_clone.set(
                                    key.clone(),
                                    EntityState::Light {
                                        state: *state,
                                        brightness: *brightness,
                                        red: *red,
                                        green: *green,
                                        blue: *blue,
                                    },
                                );
                            }
                            PublishedMessage::NumberValueChanged { key, value } => {
                                state_writer_clone.set(key.clone(), EntityState::Number(*value));
                            }
                            PublishedMessage::TextSensorValueChanged { key, value } => {
                                state_writer_clone
                                    .set(key.clone(), EntityState::TextSensor(value.clone()));
                            }
                            _ => {}
                        }
                        debug!("Publishing command: {:?}", pcmd);
                        internal_tx_clone.send(pcmd).unwrap();
                    }
                }
            }
        });

        // Wire up builtin template switches/buttons/numbers: they react to
        // commands/presses by running their automations on the shared bus.
        builtins::template::spawn(
            &mut supervised_tasks,
            builtin.template.clone(),
            internal_tx.clone(),
            modules_tx.clone(),
            globals.clone(),
            state_writer.clone(),
        );

        run_platforms(
            &mut supervised_tasks,
            configured_platforms,
            modules_tx.clone(),
            modules_rx,
            state_store,
        );

        // Fire the `ubihome.on_startup` trigger once, now that every platform
        // module has resubscribed and is listening for published commands.
        if let Some(on_startup) = config.ubihome.on_startup.clone() {
            let internal_tx_clone = internal_tx.clone();
            let globals_clone = globals.clone();
            supervised_tasks.spawn(async move {
                builtins::run_actions(on_startup.then, &internal_tx_clone, &globals_clone).await;
            });
        }

        println!("Platforms: {:?}", initialized_platforms);

        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        // Supervise every task in the set: a task returning normally (e.g. a module
        // whose run() returned Ok(())) is an intentional exit and is ignored, but a
        // task that panics (e.g. a module's run() returned Err via unwrap) is a real
        // failure that must bring the whole application down gracefully.
        let task_supervisor = async {
            loop {
                match supervised_tasks.join_next().await {
                    // Task exited normally; keep supervising the rest.
                    Some(Ok(())) => continue,
                    // Task panicked. Sentry already captured it via the panic
                    // integration; flush before we shut down so the event is sent.
                    Some(Err(join_error)) => {
                        error!("Task terminated abnormally: {}. Shutting down.", join_error);
                        if let Some(client) = sentry::Hub::current().client() {
                            client.flush(Some(Duration::from_secs(2)));
                        }
                        break;
                    }
                    // All tasks have exited normally; nothing left to supervise, so
                    // keep waiting for a shutdown signal instead of exiting.
                    None => std::future::pending::<()>().await,
                }
            }
        };

        if let Some(some_shutdown_signal) = shutdown_signal {
            let shutdown_event = async {
                loop {
                    // Poll shutdown event.
                    match some_shutdown_signal.recv_timeout(Duration::from_secs(1)) {
                        // Break the loop either upon stop or channel disconnect
                        Ok(_) | Err(mpsc::RecvTimeoutError::Disconnected) => break,

                        // Continue work if no events were received within the timeout
                        Err(mpsc::RecvTimeoutError::Timeout) => (),
                    };
                }
            };
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
                _ = shutdown_event => {},
                _ = task_supervisor => {},
            }
        } else {
            tokio::select! {
                _ = ctrl_c => {},
                _ = terminate => {},
                _ = task_supervisor => {},
            }
        }
    });
    debug!("Shutdown complete");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn unique_temp_path(name: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!(
            "ubihome_test_{}_{}_{}",
            std::process::id(),
            nanos,
            name
        ))
    }

    #[test]
    fn read_base_config_errors_when_path_is_empty() {
        let error = read_base_config("").expect_err("expected an error for an empty config path");
        assert!(
            error.contains("No configuration file found"),
            "unexpected error message: {}",
            error
        );
    }

    #[test]
    fn read_base_config_errors_with_helpful_message_when_file_is_missing() {
        let missing_path = unique_temp_path("missing.yaml");
        let missing_path = missing_path.to_str().unwrap();

        let error = read_base_config(missing_path)
            .expect_err("expected an error for a missing config file");
        assert!(
            error.contains("Configuration file not found"),
            "unexpected error message: {}",
            error
        );
        assert!(
            error.contains(missing_path),
            "error should mention the missing path: {}",
            error
        );
    }

    #[test]
    fn read_base_config_reads_existing_file_contents() {
        let path = unique_temp_path("config.yaml");
        fs::write(&path, "ubihome:\n  name: test\n").unwrap();

        let result = read_base_config(path.to_str().unwrap());
        fs::remove_file(&path).ok();

        assert_eq!(result.unwrap(), "ubihome:\n  name: test\n");
    }
}
