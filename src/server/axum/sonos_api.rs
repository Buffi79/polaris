use axum::{
    extract::{Path, State},
    Json,
};
use utoipa_axum::routes;

use crate::{
    app::{config, App},
    server::{dto::{self, SonosModeCommand, SonosTransportCommand, SonosVolumeCommand}, error::APIError},
    sonos::{PlayTrackRequest, SonosResponse, SonosService, SonosSpeaker, SonosState},
};

use crate::app::config::sonos::{DEFAULT_SONOS_API_URL, DEFAULT_SONOS_MP3_SERVER};
use super::auth::Auth;

pub fn router() -> utoipa_axum::router::OpenApiRouter<App> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(get_sonos_speakers))
        .routes(routes!(post_sonos_play))
        .routes(routes!(get_sonos_state))
        .routes(routes!(post_transport_command))
        .routes(routes!(post_volume_command))
        .routes(routes!(post_mode_command))

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


    let res = service
        .play_track(&req.speaker_id, &req.track_url, &mp3_server)
        .await
        .map_err(|_| APIError::Internal)?;
    Ok(Json(res))
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
    Path(speaker_id): Path<String>,
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
    Path(speaker_id): Path<String>,
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
    Path(speaker_id): Path<String>,
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
        "set" => {
            if let Some(level) = command.value {
                service.set_volume(&speaker_id, level).await
            } else {
                return Err(APIError::BadRequest(Some(
                    "Missing volume level for set command".to_string(),
                )));
            }
        }
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
    Path(speaker_id): Path<String>,
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