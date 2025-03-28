use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct OSHome {
    pub name: String
}

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonConfig {
    pub platform: String,
    pub id: Option<String>,
    pub name: String,
    pub command: String,
}

#[derive(Clone, Deserialize, Debug)]
pub struct Config {
    pub oshome: OSHome,
}

#[derive(Debug, Clone)]
pub enum Message {
    ButtonPress {
        key: String,
    },
}