//! Static schema hints used to render forms, suggest fields and prefill new
//! entries. This is intentionally a *hint* table: the editor still edits any
//! key/value (unknown platforms from third-party crates included), and the
//! authoritative validation remains `ubihome validate`.
//!
//! Keep in sync with the platform crates' config structs when adding fields.

use crate::yaml::Yaml;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FieldType {
    Text,
    Int,
    Float,
    Bool,
    /// A `duration_str` value like `30s` or `5m`.
    Duration,
    Choice(&'static [&'static str]),
    Map,
    List,
    /// An automation trigger: `{ then: [actions] }`.
    Trigger,
}

impl FieldType {
    /// The value inserted when a user adds this field.
    pub fn initial(self, default: Option<&str>) -> Yaml {
        match (self, default) {
            (FieldType::Int, Some(d)) => d.parse().map(Yaml::Int).unwrap_or(Yaml::Int(0)),
            (FieldType::Int, None) => Yaml::Int(0),
            (FieldType::Float, Some(d)) => d.parse().map(Yaml::Float).unwrap_or(Yaml::Float(0.0)),
            (FieldType::Float, None) => Yaml::Float(0.0),
            (FieldType::Bool, d) => Yaml::Bool(d == Some("true")),
            (FieldType::Choice(options), d) => {
                Yaml::String(d.or(options.first().copied()).unwrap_or_default().into())
            }
            (FieldType::Map, _) => Yaml::Map(Vec::new()),
            (FieldType::List, _) => Yaml::Seq(Vec::new()),
            (FieldType::Trigger, _) => Yaml::Map(vec![("then".into(), Yaml::Seq(Vec::new()))]),
            (FieldType::Text | FieldType::Duration, d) => {
                Yaml::String(d.unwrap_or_default().into())
            }
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct Field {
    pub key: &'static str,
    pub ty: FieldType,
    pub required: bool,
    pub default: Option<&'static str>,
    pub help: &'static str,
}

const fn opt(key: &'static str, ty: FieldType, help: &'static str) -> Field {
    Field {
        key,
        ty,
        required: false,
        default: None,
        help,
    }
}

const fn req(key: &'static str, ty: FieldType, help: &'static str) -> Field {
    Field {
        key,
        ty,
        required: true,
        default: None,
        help,
    }
}

const fn def(key: &'static str, ty: FieldType, default: &'static str, help: &'static str) -> Field {
    Field {
        key,
        ty,
        required: false,
        default: Some(default),
        help,
    }
}

use FieldType::*;

pub struct EntityKind {
    pub key: &'static str,
    pub label: &'static str,
    /// Fields every entity of this kind accepts (besides the base fields).
    pub fields: &'static [Field],
}

pub const BASE_ENTITY_FIELDS: &[Field] = &[
    req("platform", Text, "The platform providing this entity."),
    opt(
        "name",
        Text,
        "Display name (3-100 characters). Either name or id is required.",
    ),
    opt(
        "id",
        Text,
        "Unique id (letters, numbers, '-' and '_'). Either name or id is required.",
    ),
    opt(
        "internal",
        Bool,
        "Hide the entity from connectivity components (api, mqtt, web).",
    ),
    opt("icon", Text, "Material Design icon, e.g. mdi:lightbulb."),
    opt("device_class", Text, "Home Assistant device class."),
    opt(
        "entity_category",
        Choice(&["config", "diagnostic"]),
        "Home Assistant entity category.",
    ),
];

const FILTERS: Field = opt("filters", List, "Filters applied to the raw value.");

pub const ENTITY_KINDS: &[EntityKind] = &[
    EntityKind {
        key: "sensor",
        label: "Sensors",
        fields: &[
            opt("unit_of_measurement", Text, "Unit, e.g. °C or %."),
            opt(
                "state_class",
                Text,
                "Home Assistant state class, e.g. measurement.",
            ),
            opt("accuracy_decimals", Int, "Number of decimals to report."),
            FILTERS,
        ],
    },
    EntityKind {
        key: "binary_sensor",
        label: "Binary Sensors",
        fields: &[
            FILTERS,
            opt(
                "on_press",
                Trigger,
                "Actions to run when the state turns on.",
            ),
            opt(
                "on_release",
                Trigger,
                "Actions to run when the state turns off.",
            ),
        ],
    },
    EntityKind {
        key: "text_sensor",
        label: "Text Sensors",
        fields: &[],
    },
    EntityKind {
        key: "button",
        label: "Buttons",
        fields: &[],
    },
    EntityKind {
        key: "switch",
        label: "Switches",
        fields: &[],
    },
    EntityKind {
        key: "light",
        label: "Lights",
        fields: &[opt(
            "disabled_by_default",
            Bool,
            "Disable the entity in Home Assistant by default.",
        )],
    },
    EntityKind {
        key: "number",
        label: "Numbers",
        fields: &[
            opt("unit_of_measurement", Text, "Unit, e.g. °C or %."),
            opt("state_class", Text, "Home Assistant state class."),
            opt("min_value", Float, "Minimum value."),
            opt("max_value", Float, "Maximum value."),
            opt("step", Float, "Step size."),
        ],
    },
    EntityKind {
        key: "media_player",
        label: "Media Players",
        fields: &[
            opt("on_play", Trigger, "Actions to run when playback starts."),
            opt("on_pause", Trigger, "Actions to run when playback pauses."),
            opt(
                "on_volume_change",
                Trigger,
                "Actions to run when the volume changes.",
            ),
            opt(
                "on_mute_change",
                Trigger,
                "Actions to run when mute changes.",
            ),
        ],
    },
];

pub fn entity_kind(key: &str) -> Option<&'static EntityKind> {
    ENTITY_KINDS.iter().find(|k| k.key == key)
}

/// Sections with a fixed schema, handled by the main binary.
pub const UBIHOME_FIELDS: &[Field] = &[
    req("name", Text, "Device name (3-100 characters)."),
    opt("friendly_name", Text, "Human friendly device name."),
    opt("area", Text, "Area the device is located in."),
    opt("on_startup", Trigger, "Actions to run once at startup."),
];

const LOG_LEVELS: &[&str] = &["error", "warn", "info", "debug", "trace"];

pub const LOGGER_FIELDS: &[Field] = &[
    req("level", Choice(LOG_LEVELS), "Log level."),
    opt("directory", Text, "Log directory."),
    opt("logs", Map, "Per-module log levels, e.g. libmdns: info."),
];

pub const GLOBAL_FIELDS: &[Field] = &[
    req("id", Text, "Id used to reference the variable."),
    req(
        "type",
        Choice(&["bool", "int", "float", "string"]),
        "Value type.",
    ),
    opt(
        "initial_value",
        Text,
        "Initial value (must match the type).",
    ),
];

/// Platforms usable in entities without a top-level section.
pub const BUILTIN_ENTITY_PLATFORMS: &[&str] = &["template"];

pub fn is_reserved_section(key: &str) -> bool {
    matches!(key, "ubihome" | "logger" | "globals") || entity_kind(key).is_some()
}

pub struct PlatformInfo {
    pub name: &'static str,
    pub description: &'static str,
    /// Whether the platform needs a top-level section (false for builtins).
    pub has_section: bool,
    pub section_fields: &'static [Field],
    pub entity_kinds: &'static [&'static str],
    /// Platform specific entity fields, per entity kind.
    pub entity_fields: &'static [(&'static str, &'static [Field])],
}

const UPDATE_INTERVAL: Field = opt("update_interval", Duration, "Polling interval, e.g. 30s.");
const COMMAND: Field = req("command", Text, "Shell command to run.");
const GLOBALS_LAMBDA: Field = opt(
    "lambda",
    Map,
    "Read the state from a global: {globals.get: <id>}.",
);

pub const PLATFORMS: &[PlatformInfo] = &[
    PlatformInfo {
        name: "api",
        description: "Home Assistant native API (ESPHome compatible).",
        has_section: true,
        section_fields: &[
            def("port", Int, "6053", "Port to listen on."),
            opt("encryption", Map, "Noise encryption: {key: <base64 key>}."),
            opt("suggested_area", Text, "Area suggested to Home Assistant."),
        ],
        entity_kinds: &[],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "mqtt",
        description: "MQTT connectivity with Home Assistant discovery.",
        has_section: true,
        section_fields: &[
            req("broker", Text, "Broker host name or IP."),
            def("port", Int, "1883", "Broker port."),
            opt("username", Text, "Username."),
            opt("password", Text, "Password."),
            def(
                "discovery_prefix",
                Text,
                "ubihome",
                "Discovery topic prefix.",
            ),
        ],
        entity_kinds: &[],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "mdns",
        description: "Announce the device via mDNS.",
        has_section: true,
        section_fields: &[
            opt("disabled", Bool, "Disable mDNS."),
            opt("ip", Text, "IP address to announce."),
            opt("mac", Text, "MAC address to announce."),
            opt("ip_addresses", List, "IP addresses to announce."),
            opt("hostname", Text, "Host name to announce."),
        ],
        entity_kinds: &[],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "web_server",
        description: "Built-in web interface.",
        has_section: true,
        section_fields: &[req("port", Int, "Port to listen on.")],
        entity_kinds: &[],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "bluetooth_proxy",
        description: "Bluetooth proxy for Home Assistant.",
        has_section: true,
        section_fields: &[opt("disabled", Bool, "Disable the proxy.")],
        entity_kinds: &[],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "shell",
        description: "Entities backed by shell commands.",
        has_section: true,
        section_fields: &[
            opt(
                "type",
                Choice(&["zsh", "bash", "sh", "cmd", "powershell", "wsl"]),
                "Shell to use.",
            ),
            def("timeout", Duration, "5s", "Command timeout."),
        ],
        entity_kinds: &[
            "sensor",
            "binary_sensor",
            "text_sensor",
            "button",
            "switch",
            "light",
            "number",
        ],
        entity_fields: &[
            ("sensor", &[COMMAND, UPDATE_INTERVAL]),
            ("binary_sensor", &[COMMAND, UPDATE_INTERVAL]),
            ("text_sensor", &[COMMAND, UPDATE_INTERVAL]),
            ("button", &[COMMAND]),
            (
                "switch",
                &[
                    req("command_on", Text, "Command to turn on."),
                    req("command_off", Text, "Command to turn off."),
                    opt("command_state", Text, "Command printing the current state."),
                    UPDATE_INTERVAL,
                ],
            ),
            (
                "light",
                &[
                    req("command_on", Text, "Command to turn on."),
                    req("command_off", Text, "Command to turn off."),
                    opt("command_state", Text, "Command printing the current state."),
                    UPDATE_INTERVAL,
                ],
            ),
            (
                "number",
                &[
                    opt("command_state", Text, "Command printing the current value."),
                    opt("command_set", Text, "Command setting the value."),
                    UPDATE_INTERVAL,
                ],
            ),
        ],
    },
    PlatformInfo {
        name: "gpio",
        description: "GPIO pins (Raspberry Pi).",
        has_section: true,
        section_fields: &[req("device", Choice(&["raspberryPi"]), "Board type.")],
        entity_kinds: &["binary_sensor", "switch"],
        entity_fields: &[
            (
                "binary_sensor",
                &[
                    req("pin", Int, "GPIO pin (0-57)."),
                    opt("pull_up", Bool, "Enable the internal pull-up."),
                ],
            ),
            (
                "switch",
                &[
                    req("pin", Int, "GPIO pin (0-57)."),
                    opt("inverted", Bool, "Invert the output."),
                    opt(
                        "restore_mode",
                        Choice(&["ALWAYS_OFF", "ALWAYS_ON", "DISABLED"]),
                        "State on startup.",
                    ),
                ],
            ),
        ],
    },
    PlatformInfo {
        name: "bme280",
        description: "BME280 temperature, pressure and humidity sensor.",
        has_section: true,
        section_fields: &[opt("name", Text, "Name of the sensor.")],
        entity_kinds: &["sensor"],
        entity_fields: &[(
            "sensor",
            &[
                opt("address", Text, "I2C address."),
                UPDATE_INTERVAL,
                opt("temperature", Map, "Temperature sensor (sensor fields)."),
                opt("pressure", Map, "Pressure sensor (sensor fields)."),
                opt("humidity", Map, "Humidity sensor (sensor fields)."),
            ],
        )],
    },
    PlatformInfo {
        name: "illuminance",
        description: "Ambient light sensor (Linux IIO).",
        has_section: true,
        section_fields: &[],
        entity_kinds: &["sensor"],
        entity_fields: &[(
            "sensor",
            &[
                def("update_interval", Duration, "30s", "Polling interval."),
                opt(
                    "device_path",
                    Text,
                    "IIO device path (autodetected if empty).",
                ),
            ],
        )],
    },
    PlatformInfo {
        name: "online",
        description: "Internet connectivity check.",
        has_section: true,
        section_fields: &[
            opt(
                "targets",
                List,
                "Hosts to check ({host, protocol, port, timeout}).",
            ),
            def("update_interval", Duration, "30s", "Check interval."),
            def("timeout", Duration, "3s", "Timeout per check."),
        ],
        entity_kinds: &["binary_sensor"],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "power_utils",
        description: "Reboot, shutdown, sleep, … buttons.",
        has_section: true,
        section_fields: &[],
        entity_kinds: &["button"],
        entity_fields: &[(
            "button",
            &[req(
                "action",
                Choice(&["reboot", "shutdown", "hibernate", "logout", "sleep"]),
                "Power action.",
            )],
        )],
    },
    PlatformInfo {
        name: "sendspin",
        description: "Synchronized audio playback (Sendspin).",
        has_section: true,
        section_fields: &[
            opt("server", Text, "Server address."),
            def("bit_depth", Choice(&["16", "24"]), "16", "Bit depth."),
            def("sample_rate", Int, "48000", "Sample rate."),
            opt("buffer_size", Int, "ALSA buffer size in frames."),
            def(
                "start_buffer_ms",
                Int,
                "500",
                "Start buffer in milliseconds.",
            ),
        ],
        entity_kinds: &["media_player"],
        entity_fields: &[(
            "media_player",
            &[
                opt("output_id", Text, "Audio output device."),
                def("volume", Int, "100", "Initial volume (0-100)."),
                opt("muted", Bool, "Start muted."),
            ],
        )],
    },
    PlatformInfo {
        name: "status",
        description: "Device status binary sensor.",
        has_section: true,
        section_fields: &[],
        entity_kinds: &["binary_sensor"],
        entity_fields: &[],
    },
    PlatformInfo {
        name: "template",
        description: "Builtin template entities (no section required).",
        has_section: false,
        section_fields: &[],
        entity_kinds: &["switch", "button", "number", "sensor"],
        entity_fields: &[
            (
                "switch",
                &[
                    def(
                        "optimistic",
                        Bool,
                        "true",
                        "Assume the state changes immediately.",
                    ),
                    opt("assumed_state", Bool, "Whether the state is assumed."),
                    GLOBALS_LAMBDA,
                    opt("turn_on_action", Trigger, "Actions to turn on."),
                    opt("turn_off_action", Trigger, "Actions to turn off."),
                ],
            ),
            (
                "button",
                &[opt("on_press", Trigger, "Actions to run on press.")],
            ),
            (
                "number",
                &[
                    opt("optimistic", Bool, "Assume the value changes immediately."),
                    opt("initial_value", Float, "Initial value."),
                    GLOBALS_LAMBDA,
                    opt("set_action", Trigger, "Actions to run when set."),
                ],
            ),
            (
                "sensor",
                &[req(
                    "lambda",
                    Map,
                    "Read the value from a global: {globals.get: <id>}.",
                )],
            ),
        ],
    },
];

pub fn platform(name: &str) -> Option<&'static PlatformInfo> {
    PLATFORMS.iter().find(|p| p.name == name)
}

/// Link to the documentation page of a section or platform.
pub fn docs_url(section: &str) -> Option<String> {
    const BASE: &str = "https://ubihome.github.io";
    let path = match section {
        "ubihome" => "getting_started/",
        "logger" => "features/platforms/logger/",
        "globals" => "features/components/globals/",
        "api" => "features/connectivity/native_api/",
        "mqtt" | "mdns" | "web_server" => {
            return Some(format!("{BASE}/features/connectivity/{section}/"))
        }
        "bluetooth_proxy" => "features/platforms/bluetooth/",
        other if platform(other).is_some() => {
            return Some(format!("{BASE}/features/platforms/{other}/"));
        }
        _ => return None,
    };
    Some(format!("{BASE}/{path}"))
}

/// All field hints for a section (top-level key).
pub fn section_fields(section: &str) -> &'static [Field] {
    match section {
        "ubihome" => UBIHOME_FIELDS,
        "logger" => LOGGER_FIELDS,
        other => platform(other).map(|p| p.section_fields).unwrap_or(&[]),
    }
}

/// All field hints for an entity of `kind` provided by `platform`.
pub fn entity_fields(kind: &str, platform_name: Option<&str>) -> Vec<Field> {
    let mut fields: Vec<Field> = BASE_ENTITY_FIELDS.to_vec();
    if let Some(kind) = entity_kind(kind) {
        fields.extend_from_slice(kind.fields);
    }
    if let Some(info) = platform_name.and_then(platform) {
        if let Some((_, extra)) = info.entity_fields.iter().find(|(k, _)| *k == kind) {
            for field in *extra {
                // Platform-specific definitions override generic ones.
                fields.retain(|f| f.key != field.key);
                fields.push(*field);
            }
        }
    }
    fields
}

/// Builds a new entity prefilled with the required fields.
pub fn new_entity(kind: &str, platform_name: &str, name: &str) -> Yaml {
    let mut entity = Yaml::Map(vec![
        ("platform".into(), Yaml::String(platform_name.into())),
        ("name".into(), Yaml::String(name.into())),
    ]);
    for field in entity_fields(kind, Some(platform_name)) {
        if field.required && entity.get(field.key).is_none() {
            entity.set(field.key, field.ty.initial(field.default));
        }
    }
    entity
}

/// Builds a new top-level section prefilled with the required fields.
pub fn new_section(section: &str) -> Yaml {
    let fields = section_fields(section);
    if fields.iter().all(|f| !f.required) {
        return Yaml::Map(Vec::new());
    }
    let mut value = Yaml::Map(Vec::new());
    for field in fields.iter().filter(|f| f.required) {
        value.set(field.key, field.ty.initial(field.default));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_entity_has_required_fields() {
        let e = new_entity("switch", "shell", "Fan");
        assert!(e.get("command_on").is_some());
        assert!(e.get("command_off").is_some());
        assert!(e.get("command_state").is_none());
    }

    #[test]
    fn platform_names_are_unique() {
        for (i, p) in PLATFORMS.iter().enumerate() {
            assert!(PLATFORMS[i + 1..].iter().all(|o| o.name != p.name));
            for kind in p.entity_kinds {
                assert!(entity_kind(kind).is_some(), "{} {}", p.name, kind);
            }
        }
    }
}
