use serde::{Deserialize, Serialize};

use utoipa::ToSchema;

/// Represents a Sonos speaker device
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SonosSpeaker {
    /// Unique identifier for the speaker (e.g., room name)
    #[schema(examples("Living Room", "Kitchen", "Bedroom"))]
    pub id: String,
    /// Display name of the speaker
    #[schema(examples("Living Room", "Kitchen Speaker", "Master Bedroom"))]
    pub name: String,
    /// Whether the speaker is currently online and available
    #[schema(examples(true, false))]
    pub available: bool,
    /// Current volume (0-100)
    #[schema(examples(50, 75, 25))]
    pub volume: Option<u8>,
}

/// Request to play a track on Sonos
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PlayTrackRequest {
    /// The speaker ID to play on
    #[schema(examples("Living Room", "Kitchen"))]
    pub speaker_id: String,
    /// The track URL or file path to play
    #[schema(examples("http://localhost:5050/api/v8/audio/track.mp3"))]
    pub track_url: String,
}

/// Response from Sonos operations
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosResponse {
    #[schema(examples(true, false))]
    pub success: bool,
    #[schema(examples("Track started playing", "Speaker not found"))]
    pub message: String,
}

/// Service to interact with node-sonos-http-api
pub struct SonosService {
    base_url: String,
    client: reqwest::Client,
}

impl SonosService {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// Get all available Sonos speakers
    pub async fn get_speakers(&self) -> Result<Vec<SonosSpeaker>, Box<dyn std::error::Error>> {
        let url = format!("{}/zones", self.base_url);
        
        // Try to fetch zones from node-sonos-http-api
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let zones: serde_json::Value = response.json().await?;
                    
                    let mut speakers = Vec::new();
                    if let Some(zones_array) = zones.as_array() {
                        for zone in zones_array {
                            if let Some(coordinator) = zone.get("coordinator") {
                                if let (_uuid, Some(room_name)) = 
                                    (coordinator.get("uuid").and_then(|u| u.as_str()),
                                     coordinator.get("roomName").and_then(|r| r.as_str())) {
                                    
                                    let volume = coordinator.get("state")
                                        .and_then(|s| s.get("volume"))
                                        .and_then(|v| v.as_u64())
                                        .map(|v| v as u8);

                                    speakers.push(SonosSpeaker {
                                        id: room_name.to_string(),
                                        name: room_name.to_string(),
                                        available: true,
                                        volume,
                                    });
                                }
                            }
                        }
                    }
                    Ok(speakers)
                } else {
                    // If API is not available, return empty list
                    Ok(Vec::new())
                }
            }
            Err(_) => {
                // If connection fails, return empty list (API might not be running)
                Ok(Vec::new())
            }
        }
    }

    /// Play a track on a specific Sonos speaker
    pub async fn play_track(&self, speaker_id: &str, track_url: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/{}/clip/{}", self.base_url, speaker_id, track_url);
        
        match self.client.post(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(SonosResponse {
                        success: true,
                        message: "Track started playing".to_string(),
                    })
                } else {
                    Ok(SonosResponse {
                        success: false,
                        message: format!("HTTP error: {}", response.status()),
                    })
                }
            }
            Err(e) => {
                Ok(SonosResponse {
                    success: false,
                    message: format!("Connection error: {}", e),
                })
            }
        }
    }
}
