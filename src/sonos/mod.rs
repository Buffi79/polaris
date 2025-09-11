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
    /// The track URL from Polaris
    #[schema(examples("http://192.168.0.5:5050/api/v8/audio/track.mp3"))]
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

/// Sonos speaker playback state
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosState {
    /// Whether the speaker is currently playing
    #[schema(examples(true, false))]
    pub is_playing: bool,
    /// Current track artist
    #[schema(examples("The Beatles", "Mozart"))]
    pub artist: Option<String>,
    /// Current track title
    #[schema(examples("Yesterday", "Piano Sonata No. 14"))]
    pub title: Option<String>,
    /// Current playback position in seconds
    #[schema(examples(120, 45))]
    pub position: Option<u32>,
    /// Total track duration in seconds
    #[schema(examples(240, 180))]
    pub duration: Option<u32>,
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
                                if let (Some(_uuid), Some(room_name)) = 
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
    /// Converts Polaris URLs to CIFS paths for node-sonos-http-api
    pub async fn play_track(&self, speaker_id: &str, track_url: &str, file_server: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        // Extract track path from Polaris URL
        // Example: http://localhost:5050/api/v8/audio/Test%2FKinderlieder%2FTest.mp3
        // Extract: Test/Kinderlieder/Test.mp3
        
        let track_path = if let Some(path_part) = track_url.split("/audio/").nth(1) {
            urlencoding::decode(path_part)?.to_string()
        } else {
            // Fallback: use the URL as-is if we can't extract the path
            track_url.to_string()
        };
        
        // Construct CIFS path: x-file-cifs://192.168.0.6/mp3/Test/Kinderlieder/Test.mp3
        let cifs_uri = format!("x-file-cifs://{}/{}", file_server, track_path);
        
        // node-sonos-http-api URL: http://192.168.0.5:5005/Elena/setavtransporturi/[encoded_uri]
        let url = format!("{}/{}/setavtransporturi/{}", 
                         self.base_url, 
                         speaker_id, 
                         urlencoding::encode(&cifs_uri));
        
        println!("Sonos play URL: {}", url);
        
        match self.client.post(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(SonosResponse {
                        success: true,
                        message: "Track started playing on Sonos".to_string(),
                    })
                } else {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    Ok(SonosResponse {
                        success: false,
                        message: format!("HTTP error {}: {}", status, text),
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

    /// Get the current playback state of a Sonos speaker
    pub async fn get_state(&self, speaker_id: &str) -> Result<SonosState, Box<dyn std::error::Error>> {
        let url = format!("{}/{}/state", self.base_url, speaker_id);
        
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let state_data: serde_json::Value = response.json().await?;
                    
                    // Parse the state response from node-sonos-http-api
                    let is_playing = state_data.get("playbackState")
                        .and_then(|s| s.as_str())
                        .map(|s| s == "PLAYING")
                        .unwrap_or(false);
                    
                    let artist = state_data.get("currentTrack")
                        .and_then(|track| track.get("artist"))
                        .and_then(|a| a.as_str())
                        .map(|s| s.to_string());
                    
                    let title = state_data.get("currentTrack")
                        .and_then(|track| track.get("title"))
                        .and_then(|t| t.as_str())
                        .map(|s| s.to_string());
                    
                    // Parse position and duration in seconds
                    let position = state_data.get("relTime")
                        .and_then(|t| t.as_str())
                        .and_then(|s| parse_time_to_seconds(s))
                        .map(|s| s as u32);
                    
                    let duration = state_data.get("currentTrack")
                        .and_then(|track| track.get("duration"))
                        .and_then(|d| d.as_str())
                        .and_then(|s| parse_time_to_seconds(s))
                        .map(|s| s as u32);
                    
                    Ok(SonosState {
                        is_playing,
                        artist,
                        title,
                        position,
                        duration,
                    })
                } else {
                    // Return empty state if speaker not found or error
                    Ok(SonosState {
                        is_playing: false,
                        artist: None,
                        title: None,
                        position: None,
                        duration: None,
                    })
                }
            }
            Err(_) => {
                // Return empty state if connection fails
                Ok(SonosState {
                    is_playing: false,
                    artist: None,
                    title: None,
                    position: None,
                    duration: None,
                })
            }
        }
    }
}

/// Helper function to parse time strings like "0:02:30" to seconds
fn parse_time_to_seconds(time_str: &str) -> Option<u64> {
    let parts: Vec<&str> = time_str.split(':').collect();
    match parts.len() {
        2 => {
            // Format: MM:SS
            let minutes: u64 = parts[0].parse().ok()?;
            let seconds: u64 = parts[1].parse().ok()?;
            Some(minutes * 60 + seconds)
        }
        3 => {
            // Format: H:MM:SS
            let hours: u64 = parts[0].parse().ok()?;
            let minutes: u64 = parts[1].parse().ok()?;
            let seconds: u64 = parts[2].parse().ok()?;
            Some(hours * 3600 + minutes * 60 + seconds)
        }
        _ => None,
    }
}
