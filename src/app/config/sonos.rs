use serde::{Deserialize, Serialize};

pub const DEFAULT_SONOS_API_URL: &str = "http://192.168.0.5:5005";
pub const DEFAULT_SONOS_MP3_SERVER: &str = "192.168.0.6/mp3";

#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct SonosConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mp3_server: Option<String>,
}

impl SonosConfig {
    pub fn get_api_url(&self) -> String {
        self.api_url.clone().unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string())
    }

    pub fn get_mp3_server(&self) -> String {
        self.mp3_server.clone().unwrap_or_else(|| DEFAULT_SONOS_MP3_SERVER.to_string())
    }
}
