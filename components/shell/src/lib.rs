use duration_str::deserialize_duration;
use log::{debug, trace, warn};
use serde::{Deserialize, Deserializer};
use shell_exec::{Execution, Shell, ShellError};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str, time::Duration};
use tokio::{
    sync::broadcast::{Receiver, Sender},
    time,
};
use ubihome_core::internal::sensors::{UbiComponent, UbiLight, UbiNumber, UbiSwitch, UbiTextSensor};
use ubihome_core::template_light;
use ubihome_core::template_text_sensor;
use ubihome_core::{
    config_template,
    internal::sensors::{UbiBinarySensor, UbiButton, UbiSensor},
    ChangedMessage, Module, PublishedMessage,
};

use ubihome_core::constants::is_id_string_option;
use ubihome_core::constants::is_readable_string;
use ubihome_core::template_binary_sensor;
use ubihome_core::template_button;
use ubihome_core::template_number;
use ubihome_core::template_sensor;
use ubihome_core::template_switch;
use ubihome_core::with_base_entity_properties;

#[derive(Debug, Copy, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub enum CustomShell {
    Zsh,
    Bash,
    Sh,
    Cmd,
    Powershell,
    Wsl,
}

#[derive(Clone, Deserialize, Debug, Validate)]
pub struct ShellConfig {
    #[serde(rename = "type")]
    #[garde(dive)]
    pub kind: Option<CustomShell>,

    #[serde(default = "default_timeout")]
    #[serde(deserialize_with = "deserialize_duration")]
    #[garde(skip)]
    pub timeout: Duration,
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

template_binary_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct ShellBinarySensorConfig {
        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        #[garde(skip)]
        pub update_interval: Option<Duration>,
        pub command: String,
    }
}

template_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct ShellSensorConfig {
        #[garde(skip)]
        pub command: String,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        #[garde(skip)]
        pub update_interval: Option<Duration>,
    }
}

template_button! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct ShellButtonConfig {
        #[garde(length(min = 1))]
        pub command: String,
    }
}

template_switch! {

    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct ShellSwitchConfig {
        #[garde(length(min = 1))]
        pub command_on: String,
        #[garde(length(min = 1))]
        pub command_off: String,
        #[garde(skip)]
        pub command_state: Option<String>,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        #[garde(skip)]
        pub update_interval: Option<Duration>,
    }
}

template_light! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    pub struct ShellLightConfig {
        #[garde(length(min = 1))]
        pub command_on: String,
        #[garde(length(min = 1))]
        pub command_off: String,
        #[garde(skip)]
        pub command_state: Option<String>,
        // pub command_brightness: Option<String>,
        // pub command_rgb: Option<String>,

        // pub supports_brightness: Option<bool>,
        // pub supports_rgb: Option<bool>,
        // pub supports_white_value: Option<bool>,
        // pub supports_color_temperature: Option<bool>,
        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        #[garde(skip)]
        pub update_interval: Option<Duration>,
    }
}

template_number! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct ShellNumberConfig {
        pub command_state: Option<String>,
        pub command_set: Option<String>,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        pub update_interval: Option<Duration>,
    }
}

template_text_sensor! {
    #[derive(Clone, Deserialize, Debug, Validate)]
    #[garde(allow_unvalidated)]
    pub struct ShellTextSensorConfig {
        pub command: String,

        #[serde(default = "default_timeout_none")]
        #[serde(deserialize_with = "deserialize_option_duration")]
        pub update_interval: Option<Duration>,
    }
}

fn default_timeout_none() -> Option<Duration> {
    None
}

config_template!(
    shell,
    ShellConfig,
    ShellButtonConfig,
    ShellBinarySensorConfig,
    ShellSensorConfig,
    ShellSwitchConfig,
    ShellLightConfig,
    ShellNumberConfig,
    ShellTextSensorConfig
);

pub struct UbiHomePlatform {
    config: ShellConfig,
    components: Vec<UbiComponent>,
    binary_sensors: HashMap<String, ShellBinarySensorConfig>,
    buttons: HashMap<String, ShellButtonConfig>,
    sensors: HashMap<String, ShellSensorConfig>,
    switches: HashMap<String, ShellSwitchConfig>,
    lights: HashMap<String, ShellLightConfig>,
    numbers: HashMap<String, ShellNumberConfig>,
    text_sensors: HashMap<String, ShellTextSensorConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str) -> Result<Self, String> {
        let config =
            serde_saphyr::from_str::<CoreConfig>(config_string).map_err(|e| e.to_string())?;
        debug!("Shell config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();

        let mut sensors: HashMap<String, ShellSensorConfig> = HashMap::new();
        for (_, sensor) in config.sensor.clone().unwrap_or_default() {
            let id = sensor.get_object_id();
            components.push(UbiComponent::Sensor(UbiSensor {
                platform: "sensor".to_string(),
                icon: sensor.icon.clone(),
                device_class: sensor.device_class.clone(),
                state_class: sensor.state_class.clone(),
                unit_of_measurement: sensor.unit_of_measurement.clone(),
                accuracy_decimals: sensor.accuracy_decimals,
                name: sensor.name.clone(),
                id: id.clone(),
                filters: sensor.filters.clone(),
            }));
            sensors.insert(id.clone(), sensor);
        }

        let mut binary_sensors: HashMap<String, ShellBinarySensorConfig> = HashMap::new();
        for (_, binary_sensor) in config.binary_sensor.clone().unwrap_or_default() {
            let id = binary_sensor.get_object_id();
            components.push(UbiComponent::BinarySensor(UbiBinarySensor {
                platform: "sensor".to_string(),
                icon: binary_sensor.icon.clone(),
                device_class: binary_sensor.device_class.clone(),
                name: binary_sensor.name.clone(),
                id: id.clone(),
                on_press: binary_sensor.on_press.clone(),
                on_release: binary_sensor.on_release.clone(),
                filters: binary_sensor.filters.clone(),
            }));
            binary_sensors.insert(id.clone(), binary_sensor);
        }

        let mut buttons: HashMap<String, ShellButtonConfig> = HashMap::new();
        for (_, button) in config.button.clone().unwrap_or_default() {
            let id = button.get_object_id();
            components.push(UbiComponent::Button(UbiButton {
                platform: "sensor".to_string(),
                icon: button.icon.clone(),
                name: button.name.clone(),
                id: id.clone(),
            }));
            buttons.insert(id.clone(), button);
        }

        let mut switches: HashMap<String, ShellSwitchConfig> = HashMap::new();
        for (_, switch) in config.switch.clone().unwrap_or_default() {
            let id = switch.get_object_id();
            components.push(UbiComponent::Switch(UbiSwitch {
                platform: "sensor".to_string(),
                icon: switch.icon.clone(),
                name: switch.name.clone(),
                id: id.clone(),
                device_class: None,
                assumed_state: switch.command_state.is_none(),
            }));
            switches.insert(id.clone(), switch);
        }

        let mut lights: HashMap<String, ShellLightConfig> = HashMap::new();
        for (_, light) in config.light.clone().unwrap_or_default() {
            let id = light.get_object_id();
            components.push(UbiComponent::Light(UbiLight {
                platform: "light".to_string(),
                icon: light.icon.clone(),
                name: light.name.clone(),
                id: id.clone(),
                disabled_by_default: light.disabled_by_default.unwrap_or(true),
            }));
            lights.insert(id.clone(), light);
        }

        let mut numbers: HashMap<String, ShellNumberConfig> = HashMap::new();
        for (_, number) in config.number.clone().unwrap_or_default() {
            let id = number.get_object_id();
            components.push(UbiComponent::Number(UbiNumber {
                platform: "number".to_string(),
                icon: number.icon.clone(),
                name: number.name.clone(),
                id: id.clone(),
                min_value: number.min_value.unwrap_or(0.0),
                max_value: number.max_value.unwrap_or(100.0),
                step: number.step.unwrap_or(1.0),
                unit_of_measurement: number.unit_of_measurement.clone(),
                device_class: number.device_class.clone(),
                mode: 1, // NumberMode::Box
            }));
            numbers.insert(id.clone(), number);
        }

        let mut text_sensors: HashMap<String, ShellTextSensorConfig> = HashMap::new();
        for (_, text_sensor) in config.text_sensor.clone().unwrap_or_default() {
            let id = text_sensor.get_object_id();
            components.push(UbiComponent::TextSensor(UbiTextSensor {
                platform: "text_sensor".to_string(),
                icon: text_sensor.icon.clone(),
                name: text_sensor.name.clone(),
                id: id.clone(),
                device_class: text_sensor.device_class.clone(),
            }));
            text_sensors.insert(id.clone(), text_sensor);
        }

        Ok(UbiHomePlatform {
            config: config.shell,
            components,
            binary_sensors,
            buttons,
            sensors,
            switches,
            lights,
            numbers,
            text_sensors,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        sender: Sender<ChangedMessage>,
        mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        let config = self.config.clone();
        let binary_sensors = self.binary_sensors.clone();
        let buttons = self.buttons.clone();
        let switches = self.switches.clone();
        let sensors = self.sensors.clone();
        let lights = self.lights.clone();
        let numbers = self.numbers.clone();
        let text_sensors = self.text_sensors.clone();
        Box::pin(async move {
            let cloned_config = config.clone();
            let csender = sender.clone();

            let switches_clone = switches.clone();
            let lights_clone = lights.clone();
            let numbers_clone = numbers.clone();
            // Handle Button Presses
            tokio::spawn(async move {
                let cloned_sender = csender.clone();

                while let Ok(cmd) = receiver.recv().await {
                    match cmd {
                        PublishedMessage::SwitchStateCommand { key, state } => {
                            debug!("SwitchStateChanged: {} {}", key, state);
                            if let Some(switch) = switches_clone.get(&key) {
                                // ButtonKind::shell(shell_button) => {
                                let command: String = if state {
                                    debug!("Turning on switch: {}", key);
                                    switch.command_on.clone()
                                } else {
                                    debug!("Turning off switch: {}", key);
                                    switch.command_off.clone()
                                };
                                debug!("Executing command: {}", command);

                                let output = execute_command(
                                    &cloned_config,
                                    &command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();
                                // If output is empty report status code
                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }

                                if let Some(command_state) = &switch.command_state {
                                    let output = execute_command(
                                        &cloned_config,
                                        command_state,
                                        &cloned_config.timeout,
                                    )
                                    .await;

                                    match output {
                                        Ok(output) => {
                                            debug!("Switch {} output: {}", key, &output);
                                            let value = if output.trim().to_lowercase() == "true" {
                                                true
                                            } else if output.trim().to_lowercase() == "false" {
                                                false
                                            } else {
                                                debug!("Invalid switch sensor output: {}", output);
                                                continue;
                                            };

                                            _ = cloned_sender.send(
                                                ChangedMessage::SwitchStateChange {
                                                    key: key.clone(),
                                                    state: value,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!("Error executing command: {}", e);
                                        }
                                    };
                                }
                            }
                        }
                        PublishedMessage::ButtonPressed { key } => {
                            debug!("Button pressed1: {}", key);
                            if let Some(shell_button) = buttons.get(&key) {
                                // ButtonKind::shell(shell_button) => {
                                debug!("Button pressed: {}", key);
                                debug!("Executing command: {}", shell_button.command);
                                println!("Button '{}' pressed.", key);

                                let output = execute_command(
                                    &cloned_config,
                                    &shell_button.command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();
                                // If output is empty report status code
                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }
                            }
                        }
                        PublishedMessage::LightStateCommand {
                            key,
                            state,
                            brightness,
                            red,
                            green,
                            blue,
                        } => {
                            debug!(
                                "LightStateCommand: {} state:{} brightness:{:?} rgb:{:?},{:?},{:?}",
                                key, state, brightness, red, green, blue
                            );
                            if let Some(light) = lights_clone.get(&key) {
                                let command: String = if state {
                                    debug!("Turning on light: {}", key);
                                    light.command_on.clone()
                                } else {
                                    debug!("Turning off light: {}", key);
                                    light.command_off.clone()
                                };
                                debug!("Executing command: {}", command);

                                let output = execute_command(
                                    &cloned_config,
                                    &command,
                                    &cloned_config.timeout,
                                )
                                .await
                                .unwrap();

                                if output.is_empty() {
                                    trace!("Command executed successfully with no output.");
                                } else {
                                    trace!("Command executed successfully with output: {}", output);
                                }

                                // Handle brightness command if provided and supported
                                // if let (Some(brightness_val), Some(brightness_cmd)) = (brightness, &light.command_brightness) {
                                //     if light.supports_brightness.unwrap_or(false) {
                                //         let brightness_command = brightness_cmd.replace("{brightness}", &brightness_val.to_string());
                                //         debug!("Executing brightness command: {}", brightness_command);
                                //         let _ = execute_command(
                                //             &cloned_config,
                                //             &brightness_command,
                                //             &cloned_config.timeout,
                                //         ).await;
                                //     }
                                // }

                                // Handle RGB color command if provided and supported
                                // if let (Some(r), Some(g), Some(b), Some(rgb_cmd)) = (red, green, blue, &light.command_rgb) {
                                //     if light.supports_rgb.unwrap_or(false) {
                                //         let rgb_command = rgb_cmd
                                //             .replace("{red}", &r.to_string())
                                //             .replace("{green}", &g.to_string())
                                //             .replace("{blue}", &b.to_string());
                                //         debug!("Executing RGB command: {}", rgb_command);
                                //         let _ = execute_command(
                                //             &cloned_config,
                                //             &rgb_command,
                                //             &cloned_config.timeout,
                                //         ).await;
                                //     }
                                // }

                                // Check state after command if state command is available
                                if let Some(command_state) = &light.command_state {
                                    let output = execute_command(
                                        &cloned_config,
                                        command_state,
                                        &cloned_config.timeout,
                                    )
                                    .await;

                                    match output {
                                        Ok(output) => {
                                            debug!("Light {} state output: {}", key, &output);
                                            let value = if output.trim().to_lowercase() == "true" {
                                                true
                                            } else if output.trim().to_lowercase() == "false" {
                                                false
                                            } else {
                                                debug!("Invalid light state output: {}", output);
                                                continue;
                                            };

                                            _ = cloned_sender.send(
                                                ChangedMessage::LightStateChange {
                                                    key: key.clone(),
                                                    state: value,
                                                    brightness,
                                                    red,
                                                    green,
                                                    blue,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!("Error executing state command: {}", e);
                                        }
                                    };
                                }
                            }
                        }
                        PublishedMessage::NumberValueCommand { key, value } => {
                            debug!("NumberValueCommand: {} {}", key, value);
                            if let Some(number) = numbers_clone.get(&key) {
                                if let Some(command_set) = &number.command_set {
                                    let command =
                                        command_set.replace("{{ value }}", &value.to_string());
                                    debug!("Executing number set command: {}", command);

                                    let output = execute_command(
                                        &cloned_config,
                                        &command,
                                        &cloned_config.timeout,
                                    )
                                    .await;

                                    match output {
                                        Ok(output) => {
                                            if output.is_empty() {
                                                trace!("Number command executed successfully with no output.");
                                            } else {
                                                trace!("Number command executed successfully with output: {}", output);
                                            }
                                            _ = cloned_sender.send(
                                                ChangedMessage::NumberValueChange {
                                                    key: key.clone(),
                                                    value,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!("Error executing number set command: {}", e);
                                        }
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            });

            for (key, sensor) in sensors {
                let cloned_config = config.clone();
                let cloned_sender = sender.clone();
                tokio::spawn(async move {
                    if let Some(duration) = sensor.update_interval {
                        let mut interval = time::interval(duration);
                        debug!("Sensor {} has update interval: {:?}", key, interval);
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                sensor.command.as_str(),
                                &cloned_config.timeout,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    debug!("Sensor {} output: {}", key, &output);
                                    let value = output;

                                    _ = cloned_sender.send(ChangedMessage::SensorValueChange {
                                        key: key.clone(),
                                        value: value.parse().unwrap(),
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };
                            interval.tick().await;
                        }
                    } else {
                        debug!("Sensor {} has no update interval", key);
                    }
                });
            }

            for (key, switch) in switches {
                let switch = switch.clone();
                debug!("Switch State {:?}", switch);

                if let Some(command_state) = switch.command_state {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let duration = switch
                            .update_interval
                            .unwrap_or_else(|| Duration::from_secs(60));

                        let mut interval = time::interval(duration);
                        debug!(
                            "Switch {} has update interval: {:?}",
                            key,
                            interval.period()
                        );
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &command_state,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Switch {} output: {}", key, &output);
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid switch sensor output: {}", output);
                                        interval.tick().await;
                                        continue;
                                    };

                                    _ = cloned_sender.send(ChangedMessage::SwitchStateChange {
                                        key: key.clone(),
                                        state: value,
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };

                            interval.tick().await;
                        }
                    });
                } else {
                    warn!("Switch {} has no command_state", key);
                }
            }

            for (key, binary_sensor) in binary_sensors {
                let cloned_config = config.clone();
                let cloned_sender = sender.clone();
                debug!("Binary Sensor {}: {:?}", key, binary_sensor);

                tokio::spawn(async move {
                    if let Some(duration) = binary_sensor.update_interval {
                        let mut interval = time::interval(duration);
                        debug!("Sensor {} has update interval: {:?}", key, interval);
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                binary_sensor.command.as_str(),
                                &cloned_config.timeout,
                            )
                            .await;
                            // TODO: Handle long running commands (e.g. newline per value) and multivalued outputs (e.g. json)
                            match output {
                                Ok(output) => {
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid binary sensor output: {}", output);
                                        continue;
                                    };
                                    debug!("Binary Sensor '{}' output: {}", key, value);

                                    _ = cloned_sender.send(
                                        ChangedMessage::BinarySensorValueChange {
                                            key: key.clone(),
                                            value,
                                        },
                                    );
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };
                            interval.tick().await;
                        }
                    } else {
                        debug!("Sensor {} has no update interval", key);
                    }
                });
            }

            // Monitor light states with update intervals
            for (key, light) in lights {
                let light = light.clone();
                debug!("Light State monitor for: {:?}", light);

                if let Some(command_state) = light.command_state {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let duration = light
                            .update_interval
                            .unwrap_or_else(|| Duration::from_secs(60));

                        let mut interval = time::interval(duration);
                        debug!("Light {} has update interval: {:?}", key, interval.period());
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &command_state,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Light {} state: {}", key, &output);
                                    let value = if output.trim().to_lowercase() == "true" {
                                        true
                                    } else if output.trim().to_lowercase() == "false" {
                                        false
                                    } else {
                                        debug!("Invalid light state output: {}", output);
                                        interval.tick().await;
                                        continue;
                                    };

                                    _ = cloned_sender.send(ChangedMessage::LightStateChange {
                                        key: key.clone(),
                                        state: value,
                                        brightness: None, // TODO: Parse from command output if needed
                                        red: None,
                                        green: None,
                                        blue: None,
                                    });
                                }
                                Err(e) => {
                                    debug!("Error executing command: {}", e);
                                }
                            };

                            interval.tick().await;
                        }
                    });
                } else {
                    warn!("Light {} has no command_state", key);
                }
            }

            // Poll number states with update intervals
            for (key, number) in numbers {
                let number = number.clone();
                debug!("Number State monitor for: {:?}", key);

                if let Some(command_state) = number.command_state {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let duration = number
                            .update_interval
                            .unwrap_or_else(|| Duration::from_secs(60));

                        let mut interval = time::interval(duration);
                        debug!(
                            "Number {} has update interval: {:?}",
                            key,
                            interval.period()
                        );
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &command_state,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Number {} state: {}", key, &output);
                                    match output.trim().parse::<f32>() {
                                        Ok(value) => {
                                            _ = cloned_sender.send(
                                                ChangedMessage::NumberValueChange {
                                                    key: key.clone(),
                                                    value,
                                                },
                                            );
                                        }
                                        Err(e) => {
                                            debug!(
                                                "Invalid number state output '{}': {}",
                                                output.trim(),
                                                e
                                            );
                                            interval.tick().await;
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Error executing number state command: {}", e);
                                }
                            };

                            interval.tick().await;
                        }
                    });
                } else {
                    warn!("Number {} has no command_state", key);
                }
            }

            // Poll text sensor states with update intervals
            for (key, text_sensor) in text_sensors {
                let text_sensor = text_sensor.clone();
                debug!("Text Sensor State monitor for: {:?}", key);

                if let Some(duration) = text_sensor.update_interval {
                    let cloned_config = config.clone();
                    let cloned_sender = sender.clone();
                    tokio::spawn(async move {
                        let mut interval = time::interval(duration);
                        debug!(
                            "Text Sensor {} has update interval: {:?}",
                            key,
                            interval.period()
                        );
                        loop {
                            let output = execute_command(
                                &cloned_config,
                                &text_sensor.command,
                                &cloned_config.timeout,
                            )
                            .await;
                            match output {
                                Ok(output) => {
                                    debug!("Text Sensor {} state: {}", key, &output);
                                    _ = cloned_sender.send(
                                        ChangedMessage::TextSensorValueChange {
                                            key: key.clone(),
                                            value: output.trim().to_string(),
                                        },
                                    );
                                }
                                Err(e) => {
                                    debug!("Error executing text sensor command: {}", e);
                                }
                            };

                            interval.tick().await;
                        }
                    });
                } else {
                    debug!("Text Sensor {} has no update interval", key);
                }
            }
            Ok(())
        })
    }
}

async fn execute_command(
    shell_config: &ShellConfig,
    command: &str,
    timeout: &Duration,
) -> Result<String, ShellError> {
    let shell = match shell_config.kind {
        Some(CustomShell::Zsh) => Shell::Zsh,
        Some(CustomShell::Bash) => Shell::Bash,
        Some(CustomShell::Sh) => Shell::Sh,
        Some(CustomShell::Cmd) => Shell::Cmd,
        Some(CustomShell::Powershell) => Shell::Powershell,
        Some(CustomShell::Wsl) => Shell::Wsl,
        None => Shell::default(),
    };

    let execution = Execution::builder()
        .shell(shell)
        .cmd(command.to_string())
        .timeout(*timeout)
        .build();

    trace!("Executing command: {}", command);
    let output = execution.execute(b"").await?;
    let output_string = str::from_utf8(&output).unwrap_or("");
    trace!("Command '{}' executed: {}", command, output_string);
    Ok(output_string.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_number_config_parsing() {
        let config = r#"
ubihome:
  name: "Test Number Config"

shell:
  type: bash

number:
  - platform: shell
    name: "Display Brightness"
    id: display_brightness
    unit_of_measurement: "%"
    min_value: 0.0
    max_value: 100.0
    step: 1.0
    update_interval: 5s
    command_state: "echo 50.0"
    command_set: "echo {{ value }}"
"#;

        let module = UbiHomePlatform::new(&config.to_string());
        assert!(
            module.is_ok(),
            "Shell module should parse number config successfully"
        );

        let module = module.unwrap();
        assert_eq!(module.numbers.len(), 1, "Should have 1 number entity");
        assert!(
            module.numbers.contains_key("display_brightness"),
            "Should contain 'display_brightness' number"
        );

        let number = module.numbers.get("display_brightness").unwrap();
        assert!(
            number.command_state.is_some(),
            "Number should have command_state"
        );
        assert!(
            number.command_set.is_some(),
            "Number should have command_set"
        );
        assert_eq!(
            number.command_state.as_deref(),
            Some("echo 50.0"),
            "command_state should match"
        );
        assert_eq!(
            number.command_set.as_deref(),
            Some("echo {{ value }}"),
            "command_set should match"
        );
    }

    #[test]
    fn test_number_config_minimal() {
        let config = r#"
ubihome:
  name: "Test Number Minimal"

shell:

number:
  - platform: shell
    name: "Volume"
"#;

        let module = UbiHomePlatform::new(&config.to_string());
        assert!(
            module.is_ok(),
            "Shell module should parse minimal number config successfully"
        );

        let module = module.unwrap();
        assert_eq!(module.numbers.len(), 1, "Should have 1 number entity");
        assert!(
            module.numbers.contains_key("volume"),
            "Should contain 'volume' number"
        );

        let number = module.numbers.get("volume").unwrap();
        assert!(
            number.command_state.is_none(),
            "Minimal number should have no command_state"
        );
        assert!(
            number.command_set.is_none(),
            "Minimal number should have no command_set"
        );
        assert!(
            number.update_interval.is_none(),
            "Minimal number should have no update_interval"
        );
    }

    #[test]
    fn test_text_sensor_config_parsing() {
        let config = r#"
ubihome:
  name: "Test Text Sensor Config"

shell:
  type: bash

text_sensor:
  - platform: shell
    name: "Current User"
    id: current_user
    update_interval: 10s
    command: "whoami"
"#;

        let module = UbiHomePlatform::new(&config.to_string());
        assert!(
            module.is_ok(),
            "Shell module should parse text_sensor config successfully"
        );

        let module = module.unwrap();
        assert_eq!(
            module.text_sensors.len(),
            1,
            "Should have 1 text_sensor entity"
        );
        assert!(
            module.text_sensors.contains_key("current_user"),
            "Should contain 'current_user' text_sensor"
        );

        let text_sensor = module.text_sensors.get("current_user").unwrap();
        assert_eq!(
            text_sensor.command, "whoami",
            "command should match"
        );
        assert!(
            text_sensor.update_interval.is_some(),
            "Text sensor should have update_interval"
        );
    }

    #[test]
    fn test_text_sensor_config_minimal() {
        let config = r#"
ubihome:
  name: "Test Text Sensor Minimal"

shell:

text_sensor:
  - platform: shell
    name: "Hostname"
    command: "hostname"
"#;

        let module = UbiHomePlatform::new(&config.to_string());
        assert!(
            module.is_ok(),
            "Shell module should parse minimal text_sensor config successfully"
        );

        let module = module.unwrap();
        assert_eq!(
            module.text_sensors.len(),
            1,
            "Should have 1 text_sensor entity"
        );
        assert!(
            module.text_sensors.contains_key("hostname"),
            "Should contain 'hostname' text_sensor"
        );

        let text_sensor = module.text_sensors.get("hostname").unwrap();
        assert_eq!(text_sensor.command, "hostname", "command should match");
        assert!(
            text_sensor.update_interval.is_none(),
            "Minimal text_sensor should have no update_interval"
        );
    }
}
