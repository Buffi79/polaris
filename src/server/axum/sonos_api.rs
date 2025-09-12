use axum::{
    extract::{Path as AxumPath, State},
    Json,
};
use std::path::Path;
use utoipa_axum::routes;

use crate::{
    app::{config, App}, 
    server::{dto::{SonosModeCommand, SonosSeekBody, SonosSetVolumeBody, SonosTransportCommand, SonosVolumeCommand, sonos_settings::SonosSettings}, error::APIError},
    sonos::{PlayTrackRequest, SonosResponse, SonosService, SonosSpeaker, SonosState},
};

use crate::app::config::sonos::{DEFAULT_SONOS_API_URL, DEFAULT_SONOS_MP3_SERVER};
use super::auth::Auth;

pub fn router() -> utoipa_axum::router::OpenApiRouter<App> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(get_sonos_speakers))
        .routes(routes!(post_sonos_play))
        .routes(routes!(get_sonos_state))
        .routes(routes!(post_toggle_play_pause))
        .routes(routes!(post_transport_command))
        .routes(routes!(post_volume_command))
        .routes(routes!(post_set_volume))
        .routes(routes!(post_mode_command))
        .routes(routes!(post_seek))
        .routes(routes!(post_next_track))
        .routes(routes!(post_previous_track))
        .routes(routes!(get_sonos_settings)) // Add this
        .routes(routes!(put_sonos_settings)) // Add this
}

#[utoipa::path(
    get,
    path = "/speakers",
    tag = "Sonos",
    description = "List available Sonos speakers from node-sonos-http-api.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    responses((status = 200, body = [SonosSpeaker]))
)]
async fn get_sonos_speakers(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
) -> Result<Json<Vec<SonosSpeaker>>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let speakers = service
        .get_speakers()
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(speakers))
}

#[utoipa::path(
    post,
    path = "/play",
    tag = "Sonos",
    description = "Play a track URL on a specific Sonos speaker via node-sonos-http-api.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    request_body = PlayTrackRequest,
    responses((status = 200, body = SonosResponse))
)]
async fn post_sonos_play(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    Json(req): Json<PlayTrackRequest>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let mp3_server = config_manager
        .get_sonos_mp3_server()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_MP3_SERVER.to_string());

    let track_path = req.track_url.split("/audio/").nth(1)
        .ok_or_else(|| APIError::BadRequest(Some("Invalid track_url format: missing /audio/ segment".to_string())))?;
    
    let decoded_track_path = urlencoding::decode(track_path)
        .map_err(|_| APIError::BadRequest(Some("Invalid track_url format: bad encoding".to_string())))?
        .into_owned();

    let path = Path::new(&decoded_track_path);
    let mut components = path.components();
    
    let mount_name = components.next()
        .and_then(|c| c.as_os_str().to_str())
        .ok_or_else(|| APIError::BadRequest(Some("Invalid track path: can't find mount name".to_string())))?;

    let mounts = config_manager.get_mounts().await;
    let mount = mounts.iter().find(|m| m.name == mount_name);

    let sonos_track_path = if let Some(mount) = mount {
        Path::new(&mount.source)
            .file_name()
            .and_then(|n| n.to_str())
            .map(|location_folder| Path::new(location_folder).join(components.as_path()).to_string_lossy().into_owned())
            .unwrap_or_else(|| components.as_path().to_string_lossy().into_owned())
    } else {
        components.as_path().to_string_lossy().into_owned()
    };

    let res = service
        .play_track(&req.speaker_id, &sonos_track_path, &mp3_server)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(res))
}

#[utoipa::path(
    get,
    path = "/api/settings/sonos",
    tag = "Settings", // New tag for settings
    description = "Get current Sonos settings.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    responses((status = 200, body = SonosSettings))
)]
async fn get_sonos_settings(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
) -> Result<Json<SonosSettings>, APIError> {
    let http_api_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| "".to_string()); // Return empty string if not set

    let mp3_server_url = config_manager
        .get_sonos_mp3_server()
        .await
        .unwrap_or_else(|| "".to_string()); // Return empty string if not set

    Ok(Json(SonosSettings {
        http_api_url,
        mp3_server_url,
    }))
}

#[utoipa::path(
    put,
    path = "/api/settings/sonos",
    tag = "Settings", // New tag for settings
    description = "Update Sonos settings.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    request_body = SonosSettings,
    responses((status = 200, body = SonosResponse)) // Assuming SonosResponse is suitable for success
)]
async fn put_sonos_settings(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    Json(settings): Json<SonosSettings>,
) -> Result<Json<SonosResponse>, APIError> {
    config_manager
        .set_sonos_api_url(settings.http_api_url)
        .await
        .map_err(|_| APIError::Internal)?;

    config_manager
        .set_sonos_mp3_server(settings.mp3_server_url)
        .await
        .map_err(|_| APIError::Internal)?;

    Ok(Json(SonosResponse {
        success: true,
        message: "Sonos settings updated successfully".to_string(),
    }))
}

#[utoipa::path(
    get,
    path = "/state/{speaker_id}",
    tag = "Sonos",
    description = "Get the current playback state of a specific Sonos speaker via node-sonos-http-api.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    responses(
        (status = 200, body = SonosState),
        (status = 404, description = "Speaker not found or Sonos service unavailable")
    )
)]
async fn get_sonos_state(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
) -> Result<Json<SonosState>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let state = service
        .get_state(&speaker_id)
        .await
        .map_err(|_| APIError::Internal)?;

    Ok(Json(state))
}

#[utoipa::path(
    post,
    path = "/{speaker_id}/transport",
    tag = "Sonos",
    description = "Send a transport command to a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    request_body = SonosTransportCommand,
    responses((status = 200, body = SonosResponse))
)]
async fn post_transport_command(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
    Json(command): Json<SonosTransportCommand>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);

    let result = match command.command.as_str() {
        "play" => service.play(&speaker_id).await,
        "pause" => service.pause(&speaker_id).await,
        "next" => service.next(&speaker_id).await,
        "previous" => service.previous(&speaker_id).await,
        _ => {
            return Err(APIError::BadRequest(Some(
                "Invalid transport command".to_string(),
            )))
        }
    };

    let response = result.map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/next/{speaker_id}",
    tag = "Sonos",
    description = "Skip to the next track on a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    responses((status = 200, body = SonosResponse))
)]
async fn post_next_track(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let response = service
        .next(&speaker_id)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/previous/{speaker_id}",
    tag = "Sonos",
    description = "Skip to the previous track on a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    responses((status = 200, body = SonosResponse))
)]
async fn post_previous_track(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let response = service
        .previous(&speaker_id)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/play_pause/{speaker_id}",
    tag = "Sonos",
    description = "Toggle play/pause on a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    responses((status = 200, body = SonosResponse))
)]
async fn post_toggle_play_pause(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let response = service
        .toggle_play_pause(&speaker_id)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/{speaker_id}/volume",
    tag = "Sonos",
    description = "Send a volume command to a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    request_body = SonosVolumeCommand,
    responses((status = 200, body = SonosResponse))
)]
async fn post_volume_command(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
    Json(command): Json<SonosVolumeCommand>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);

    let result = match command.action.as_str() {
        "up" => service.volume_up(&speaker_id, command.value.unwrap_or(5)).await,
        "down" => service.volume_down(&speaker_id, command.value.unwrap_or(5)).await,
        _ => {
            return Err(APIError::BadRequest(Some(
                "Invalid volume command".to_string(),
            )))
        }
    };

    let response = result.map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/volume/{speaker_id}",
    tag = "Sonos",
    description = "Set the volume on a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    request_body = SonosSetVolumeBody,
    responses((status = 200, body = SonosResponse))
)]
async fn post_set_volume(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
    Json(body): Json<SonosSetVolumeBody>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let response = service
        .set_volume(&speaker_id, body.volume)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}

#[utoipa::path(
    post,
    path = "/seek/{speaker_id}",
    tag = "Sonos",
    description = "Seek to a position in the current track on a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    request_body = SonosSeekBody,
    responses((status = 200, body = SonosResponse))
)]
async fn post_seek(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
    Json(body): Json<SonosSeekBody>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);
    let response = service
        .seek(&speaker_id, body.position)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}


#[utoipa::path(
    post,
    path = "/{speaker_id}/mode",
    tag = "Sonos",
    description = "Send a mode command to a Sonos speaker.",
    security(("auth_token" = []), ("auth_query_param" = [])),
    params(
        ("speaker_id", example = "Living Room", description = "The ID/name of the Sonos speaker")
    ),
    request_body = SonosModeCommand,
    responses((status = 200, body = SonosResponse))
)]
async fn post_mode_command(
    _auth: Auth,
    State(config_manager): State<config::Manager>,
    AxumPath(speaker_id): AxumPath<String>,
    Json(command): Json<SonosModeCommand>,
) -> Result<Json<SonosResponse>, APIError> {
    let base_url = config_manager
        .get_sonos_api_url()
        .await
        .unwrap_or_else(|| DEFAULT_SONOS_API_URL.to_string());

    let service = SonosService::new(base_url);

    let result = match command.command.as_str() {
        "toggle_repeat" => service.toggle_repeat(&speaker_id).await,
        "toggle_shuffle" => service.toggle_shuffle(&speaker_id).await,
        _ => {
            return Err(APIError::BadRequest(Some(
                "Invalid mode command".to_string(),
            )))
        }
    };

    let response = result.map_err(|_| APIError::Internal)?;
    Ok(Json(response))
}
