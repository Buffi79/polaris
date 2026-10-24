use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Body des PUT /api/sonos/settings (Deserialize).
#[derive(Clone, Deserialize, ToSchema)]
pub struct SonosSettings {
    #[schema(example = "http://localhost:5005")]
    pub http_api_url: String,
    #[serde(alias = "mp3_server")]
    #[schema(example = "http://polaris.local:5000/mp3")]
    pub mp3_server_url: String,
}

/// Antwort des GET /api/sonos/settings (Serialize).
/// Liefert `mp3_server_url` (neues Frontend) UND `mp3_server` (Legacy-dist).
#[derive(Serialize, ToSchema)]
pub struct SonosSettingsResponse {
    pub http_api_url: String,
    pub mp3_server_url: String,
    pub mp3_server: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_canonical_field_name() {
        let settings: SonosSettings = serde_json::from_str(
            r#"{"http_api_url":"http://a:5005","mp3_server_url":"http://b/mp3"}"#,
        )
        .unwrap();
        assert_eq!(settings.mp3_server_url, "http://b/mp3");
    }

    #[test]
    fn serializes_both_mp3_keys() {
        let resp = SonosSettingsResponse {
            http_api_url: "http://a:5005".to_owned(),
            mp3_server_url: "http://b/mp3".to_owned(),
            mp3_server: "http://b/mp3".to_owned(),
        };
        let json = serde_json::to_string(&resp).unwrap();
        assert!(json.contains("\"mp3_server\""));
        assert!(json.contains("\"mp3_server_url\""));
        assert_eq!(json.contains("\"mp3_server\":\"http://b/mp3\""), true);
    }

    #[test]
    fn accepts_legacy_mp3_server_field_name() {
        let settings: SonosSettings = serde_json::from_str(
            r#"{"http_api_url":"http://a:5005","mp3_server":"http://b/mp3"}"#,
        )
        .unwrap();
        assert_eq!(settings.mp3_server_url, "http://b/mp3");
    }
}
