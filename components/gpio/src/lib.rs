use log::{debug, warn};
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::{future::Future, pin::Pin, str};
use tokio::sync::broadcast::{Receiver, Sender};
use ubihome_core::constants::is_id_string_option;
use ubihome_core::constants::is_readable_string;
use ubihome_core::internal::sensors::{UbiComponent, UbiSwitch};
use ubihome_core::template_binary_sensor;
use ubihome_core::template_switch;
use ubihome_core::with_base_entity_properties;
use ubihome_core::{
    config_template, internal::sensors::UbiBinarySensor, ChangedMessage, Module, NoConfig,
    PublishedMessage,
};

#[derive(Debug, Copy, Clone, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub enum GpioDevice {
    RaspberryPi,
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct GpioConfig {
    pub device: GpioDevice,
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct GpioSensorConfig {
    pub pin: u8, // TODO: Use GPIO types or library
    pub pull_up: Option<bool>,
}

template_binary_sensor! {

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct GpioBinarySensorConfig {
    #[serde(flatten)]
    #[garde(dive)]
    pub base: GpioSensorConfig,
}
}

// https://esphome.io/components/switch/index.html#config-switch-restore-mode
//
// This project does not persist switch state across restarts, so the
// `RESTORE_*` modes are commented out below: implementing them today would
// just silently behave like `ALWAYS_OFF`/`ALWAYS_ON`, which is misleading
// given their name. Uncomment (and update `initial_state` below) once state
// is actually persisted somewhere.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Deserialize, Validate)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GpioRestoreMode {
    AlwaysOff,
    AlwaysOn,
    // RestoreDefaultOff,
    // RestoreDefaultOn,
    // RestoreInvertedDefaultOff,
    // RestoreInvertedDefaultOn,
    Disabled,
}

impl GpioRestoreMode {
    // `Disabled` leaves the pin level untouched at startup.
    // Only used on Linux, where GPIO output pins are actually driven.
    #[cfg_attr(not(target_os = "linux"), allow(dead_code))]
    fn initial_state(self) -> Option<bool> {
        match self {
            GpioRestoreMode::AlwaysOff => Some(false),
            GpioRestoreMode::AlwaysOn => Some(true),
            GpioRestoreMode::Disabled => None,
        }
    }
}

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct GpioOutputConfig {
    pub pin: u8, // TODO: Use GPIO types or library
    pub inverted: Option<bool>,
    pub restore_mode: Option<GpioRestoreMode>,
}

template_switch! {

#[derive(Clone, Deserialize, Debug, Validate)]
#[garde(allow_unvalidated)]
pub struct GpioSwitchConfig {
    #[serde(flatten)]
    #[garde(dive)]
    pub base: GpioOutputConfig,
}
}

config_template!(
    gpio,
    GpioConfig,
    NoConfig,
    GpioBinarySensorConfig,
    NoConfig,
    GpioSwitchConfig,
    NoConfig,
    NoConfig,
    NoConfig
);

#[derive(Clone, Debug)]
pub struct UbiHomePlatform {
    components: Vec<UbiComponent>,
    binary_sensors: HashMap<String, GpioSensorConfig>,
    switches: HashMap<String, GpioOutputConfig>,
}

impl Module for UbiHomePlatform {
    fn new(config_string: &str, config_path: &str) -> Result<Self, String> {
        let config =
            ubihome_core::validation::validate_config::<CoreConfig>(config_string, config_path)?;

        // info!("GPIO config: {:?}", config);
        let mut components: Vec<UbiComponent> = Vec::new();
        let mut binary_sensors: HashMap<String, GpioSensorConfig> = HashMap::new();

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
            binary_sensors.insert(
                id,
                GpioSensorConfig {
                    pin: binary_sensor.base.pin,
                    pull_up: binary_sensor.base.pull_up,
                },
            );
        }

        let mut switches: HashMap<String, GpioOutputConfig> = HashMap::new();
        for (_, switch) in config.switch.clone().unwrap_or_default() {
            let id = switch.get_object_id();
            components.push(UbiComponent::Switch(UbiSwitch {
                platform: "sensor".to_string(),
                icon: switch.icon.clone(),
                device_class: switch.device_class.clone(),
                name: switch.name.clone(),
                id: id.clone(),
                assumed_state: false,
            }));
            debug!(
                "Parsed switch '{}': pin={}, inverted={:?}, restore_mode={:?}",
                id, switch.base.pin, switch.base.inverted, switch.base.restore_mode
            );
            switches.insert(
                id,
                GpioOutputConfig {
                    pin: switch.base.pin,
                    inverted: switch.base.inverted,
                    restore_mode: switch.base.restore_mode,
                },
            );
        }

        Ok(UbiHomePlatform {
            components,
            binary_sensors,
            switches,
        })
    }

    fn components(&mut self) -> Vec<UbiComponent> {
        self.components.clone()
    }

    fn run(
        &self,
        #[allow(unused_variables)] sender: Sender<ChangedMessage>,
        #[allow(unused_variables, unused_mut)] mut receiver: Receiver<PublishedMessage>,
    ) -> Pin<Box<dyn Future<Output = Result<(), Box<dyn std::error::Error>>> + Send + 'static>>
    {
        #[allow(unused_variables)]
        let binary_sensors = self.binary_sensors.clone();
        #[allow(unused_variables)]
        let switches = self.switches.clone();
        Box::pin(async move {
            #[cfg(any(target_os = "macos", target_os = "windows"))]
            {
                warn!("GPIO is not supported on this platform.");
            }
            #[cfg(target_os = "linux")]
            {
                use rppal::gpio::{Gpio, OutputPin, Trigger};
                use std::sync::{Arc, Mutex};

                let gpio = Gpio::new();
                match gpio {
                    Err(e) => {
                        warn!("Error initializing GPIO: {}", e);
                        return Ok(());
                    }
                    Ok(gpio) => {
                        debug!(
                            "GPIO initialized with {} switch(es) and {} binary_sensor(s)",
                            switches.len(),
                            binary_sensors.len()
                        );

                        // Set up switch output pins and apply their configured startup state.
                        let mut output_pins: HashMap<String, OutputPin> = HashMap::new();
                        for (key, switch) in &switches {
                            debug!(
                                "Setting up switch {} on pin {} (inverted: {:?}, restore_mode: {:?})",
                                key, switch.pin, switch.inverted, switch.restore_mode
                            );
                            match gpio.get(switch.pin) {
                                Ok(gpio_pin) => {
                                    let inverted = switch.inverted.unwrap_or(false);
                                    let initial_on = switch
                                        .restore_mode
                                        .unwrap_or(GpioRestoreMode::AlwaysOff)
                                        .initial_state();

                                    debug!(
                                        "Switch {} computed initial_on: {:?} (inverted: {})",
                                        key, initial_on, inverted
                                    );

                                    let output_pin = match initial_on {
                                        Some(on) if on != inverted => gpio_pin.into_output_high(),
                                        Some(_) => gpio_pin.into_output_low(),
                                        None => gpio_pin.into_output(),
                                    };

                                    if let Some(state) = initial_on {
                                        match sender.send(ChangedMessage::SwitchStateChange {
                                            key: key.clone(),
                                            state,
                                        }) {
                                            Ok(receiver_count) => {
                                                debug!(
                                                    "Sent initial state {} for switch {} to {} receiver(s)",
                                                    state, key, receiver_count
                                                );
                                            }
                                            Err(e) => {
                                                warn!(
                                                    "Failed to send initial state {} for switch {}: {} (no receivers subscribed yet?)",
                                                    state, key, e
                                                );
                                            }
                                        }
                                    } else {
                                        debug!(
                                            "Switch {} restore_mode is Disabled, not sending initial state",
                                            key
                                        );
                                    }

                                    output_pins.insert(key.clone(), output_pin);
                                }
                                Err(e) => {
                                    warn!(
                                        "Error getting GPIO pin {} for switch {}: {}",
                                        switch.pin, key, e
                                    );
                                }
                            }
                        }

                        // Handle Switch Commands
                        let output_pins = Arc::new(Mutex::new(output_pins));
                        let cloned_switches = switches.clone();
                        let cloned_sender = sender.clone();
                        let cloned_output_pins = output_pins.clone();
                        tokio::spawn(async move {
                            while let Ok(message) = receiver.recv().await {
                                if let PublishedMessage::SwitchStateCommand { key, state } = message
                                {
                                    if let Some(switch) = cloned_switches.get(&key) {
                                        let inverted = switch.inverted.unwrap_or(false);
                                        let mut pins = cloned_output_pins
                                            .lock()
                                            .expect("GPIO output pin lock poisoned");
                                        if let Some(pin) = pins.get_mut(&key) {
                                            if state != inverted {
                                                pin.set_high();
                                            } else {
                                                pin.set_low();
                                            }
                                            debug!("Switch {} set to {}", key, state);
                                            _ = cloned_sender.send(
                                                ChangedMessage::SwitchStateChange {
                                                    key: key.clone(),
                                                    state,
                                                },
                                            );
                                        }
                                    }
                                }
                            }
                        });

                        // Handle Button Presses
                        // let cloned_config = self.config.clone();
                        // tokio::spawn(async move {
                        //     while let Ok(Some(cmd)) = receiver.recv().await {
                        //         use Message::*;

                        //         match cmd {
                        //             ButtonPress { key } => {
                        //                 if let Some(button) = &cloned_config.button.as_ref().and_then(|b| b.get(&key))
                        //                     {
                        //                         debug!("Button pressed: {}", key);
                        //                         debug!("Executing command: {}", button.command);
                        //                         println!("Button '{}' pressed.", key);

                        //                         let output = execute_command(&cloned_shell_config, &button.command, &cloned_shell_config.timeout).await.unwrap();
                        //                         // If output is empty report status code
                        //                         if output.is_empty() {
                        //                             println!("Command executed successfully with no output.");
                        //                         } else {
                        //                             println!("Command executed successfully with output: {}", output);
                        //                         }
                        //                     } else {
                        //                         debug!("Button pressed: {}", key);
                        //                     }
                        //             }
                        //             _ => {
                        //                 debug!("Ignored message type: {:?}", cmd);
                        //             }
                        //         }
                        //     }
                        // });

                        for (key, binary_sensor) in binary_sensors {
                            let gpio_pin =
                                gpio.get(binary_sensor.pin).expect("GPIO pin not found?");
                            let mut pin: rppal::gpio::InputPin;
                            let pull_up = binary_sensor.pull_up.unwrap_or(true);
                            if pull_up {
                                debug!("pullup");
                                pin = gpio_pin.into_input_pullup();
                            } else {
                                debug!("pulldown");
                                pin = gpio_pin.into_input_pulldown();
                            }

                            // Errors?
                            // cat /sys/kernel/debug/gpio
                            let cloned_key = key.clone();
                            let cloned_sender = sender.clone();
                            pin.set_async_interrupt(Trigger::Both, None, move |event| {
                                debug!("Event: {:?}", event);
                                debug!("BinarySensor {} triggered.", cloned_key);

                                match event.trigger {
                                    Trigger::RisingEdge => {
                                        _ = cloned_sender.send(
                                            ChangedMessage::BinarySensorValueChange {
                                                key: cloned_key.clone(),
                                                value: true,
                                            },
                                        );
                                    }
                                    Trigger::FallingEdge => {
                                        _ = cloned_sender.send(
                                            ChangedMessage::BinarySensorValueChange {
                                                key: cloned_key.clone(),
                                                value: false,
                                            },
                                        );
                                    }
                                    _ => {
                                        debug!("Unknown trigger detected {:?}", event.trigger);
                                    }
                                }
                            })
                            .expect("failed to set async interrupt");
                            debug!("Waiting for interrupts.");

                            // Wait indefinitely for the interrupts
                            let future = std::future::pending();
                            let () = future.await;
                            debug!("Interrupts stopped.");
                        }
                    }
                }
            }
            debug!("GPIO module stopped");
            Ok(())
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_switch_config_parsing() {
        let config = r#"
ubihome:
  name: "Test Switch Config"

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    name: "Relay"
    id: relay
    pin: 5
    inverted: true
    restore_mode: ALWAYS_ON
"#;

        let module = UbiHomePlatform::new(config, "config.yml");
        assert!(
            module.is_ok(),
            "GPIO module should parse switch config successfully: {:?}",
            module.err()
        );

        let module = module.unwrap();
        assert_eq!(module.switches.len(), 1, "Should have 1 switch entity");

        let switch = module
            .switches
            .get("relay")
            .expect("Should contain 'relay' switch");
        assert_eq!(switch.pin, 5, "pin should match");
        assert_eq!(switch.inverted, Some(true), "inverted should match");
        assert_eq!(
            switch.restore_mode,
            Some(GpioRestoreMode::AlwaysOn),
            "restore_mode should match"
        );
    }

    #[test]
    fn test_switch_config_minimal() {
        let config = r#"
ubihome:
  name: "Test Switch Minimal"

gpio:
  device: raspberryPi

switch:
  - platform: gpio
    name: "Relay"
    pin: 12
"#;

        let module = UbiHomePlatform::new(config, "config.yml");
        assert!(
            module.is_ok(),
            "GPIO module should parse minimal switch config successfully: {:?}",
            module.err()
        );

        let module = module.unwrap();
        let switch = module
            .switches
            .get("relay")
            .expect("Should contain 'relay' switch");
        assert_eq!(switch.pin, 12, "pin should match");
        assert_eq!(switch.inverted, None, "inverted should default to None");
        assert_eq!(
            switch.restore_mode, None,
            "restore_mode should default to None"
        );
    }
}
