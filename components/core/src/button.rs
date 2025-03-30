use serde::Deserialize;

#[derive(Clone, Deserialize, Debug)]
pub struct ButtonConfig {
    pub platform: String,
    pub id: Option<String>,
    pub name: String,
    pub command: String,
}