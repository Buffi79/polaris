use serde::{Deserialize, Serialize};
use log::{info, warn};

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
    #[schema(examples(50, 75, 25))]
    pub volume: Option<u8>,
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

    async fn send_command(&self, speaker_id: &str, command: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        let url = format!("{}/{}/{}", self.base_url, speaker_id, command);
        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    Ok(SonosResponse {
                        success: true,
                        message: format!("Successfully sent command '{}' to speaker '{}'", command, speaker_id),
                    })
                } else {
                    let status = response.status();
                    let text = response.text().await.unwrap_or_default();
                    warn!("Error from Sonos API for command '{}' on speaker '{}'. Status: {}, Body: {}", command, speaker_id, status, text);
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

    pub async fn play(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "play").await
    }

    pub async fn pause(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "pause").await
    }

    pub async fn next(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "next").await
    }

    pub async fn previous(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "previous").await
    }

    pub async fn toggle_repeat(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "repeat/toggle").await
    }

    pub async fn toggle_shuffle(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, "shuffle").await
    }

    pub async fn toggle_play_pause(&self, speaker_id: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        let state = self.get_state(speaker_id).await?;
        if state.is_playing {
            self.pause(speaker_id).await
        } else {
            self.play(speaker_id).await
        }
    }

    pub async fn volume_up(&self, speaker_id: &str, increment: u8) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, &format!("volume/+{}", increment)).await
    }

    pub async fn volume_down(&self, speaker_id: &str, decrement: u8) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, &format!("volume/-{}", decrement)).await
    }

    pub async fn set_volume(&self, speaker_id: &str, level: u8) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, &format!("volume/{}", level)).await
    }

    pub async fn seek(&self, speaker_id: &str, position_secs: u32) -> Result<SonosResponse, Box<dyn std::error::Error>> {
        self.send_command(speaker_id, &format!("seek/{}", position_secs)).await
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
     
                                    speakers.push(SonosSpeaker {
                                        id: room_name.to_string(),
                                        name: room_name.to_string(),
                                        available: true,
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
/// Simple direct approach that matches the working curl command
pub async fn play_track(&self, speaker_id: &str, track_path: &str, file_server: &str) -> Result<SonosResponse, Box<dyn std::error::Error>> {
    use log::{info, warn};
    
    // Construct the full CIFS URI exactly as in your working curl command
    let cifs_uri = format!("x-file-cifs://{}/{}", file_server, track_path);
    info!("CIFS URI: {}", cifs_uri);
    
    // Build the full URL for the setavtransporturi command
    let encoded_uri = urlencoding::encode(&cifs_uri);
    let url = format!("{}/{}/setavtransporturi/{}", self.base_url, speaker_id, encoded_uri);
    
    info!("Sending GET to: {}", url);
    
    // Send GET request (not POST!) to setavtransporturi
    let response = self.client.get(&url).send().await;
    
    match response {
        Ok(resp) => {
            let status = resp.status();
            info!("setavtransporturi response status: {}", status);
            
            if status.is_success() {
                info!("Transport URI set successfully");
                
                // Now send the play command
                let play_url = format!("{}/{}/play", self.base_url, speaker_id);
                info!("Sending play command to: {}", play_url);
                
                match self.client.get(&play_url).send().await {
                    Ok(play_resp) => {
                        let play_status = play_resp.status();
                        info!("Play command response status: {}", play_status);
                        
                        if play_status.is_success() {
                            Ok(SonosResponse {
                                success: true,
                                message: "Track started playing successfully".to_string(),
                            })
                        } else {
                            let error_text = play_resp.text().await.unwrap_or_default();
                            warn!("Play command failed. Status: {}, Body: {}", play_status, error_text);
                            Ok(SonosResponse {
                                success: false,
                                message: format!("Play command failed: HTTP {}", play_status),
                            })
                        }
                    }
                    Err(e) => {
                        warn!("Connection error on play command: {}", e);
                        Ok(SonosResponse {
                            success: false,
                            message: format!("Connection error on play: {}", e),
                        })
                    }
                }
            } else {
                let error_text = resp.text().await.unwrap_or_default();
                warn!("setavtransporturi failed. Status: {}, Body: {}", status, error_text);
                Ok(SonosResponse {
                    success: false,
                    message: format!("Failed to set transport URI: HTTP {} - {}", status, error_text),
                })
            }
        }
        Err(e) => {
            warn!("Connection error on setavtransporturi: {}", e);
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
                    let position = state_data.get("elapsedTime")
                        .and_then(|t| t.as_str())
                        .and_then(|s| parse_time_to_seconds(s))
                        .map(|s| s as u32);
                    
                    let duration = state_data.get("currentTrack")
                        .and_then(|track| track.get("duration"))
                        .and_then(|d| d.as_str())
                        .and_then(|s| parse_time_to_seconds(s))
                        .map(|s| s as u32);

                    let volume = state_data.get("volume")
                        .and_then(|v| v.as_u64())
                        .map(|v| v as u8);
                    
                    Ok(SonosState {
                        is_playing,
                        artist,
                        title,
                        position,
                        duration,
                        volume
                    })
                } else {
                    // Return empty state if speaker not found or error
                    Ok(SonosState {
                        is_playing: false,
                        artist: None,
                        title: None,
                        position: None,
                        duration: None,
                        volume: None,
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
                    volume: None,
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
