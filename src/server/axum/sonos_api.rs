use axum::{
    extract::{Path, State},
    Json,
};
use utoipa_axum::routes;

use crate::{
    app::{config, App},
    server::{dto, error::APIError},
    sonos::{PlayTrackRequest, SonosResponse, SonosService, SonosSpeaker, SonosState},
};

use crate::app::config::sonos::{DEFAULT_SONOS_API_URL, DEFAULT_SONOS_MP3_SERVER};
use super::auth::Auth;

pub fn router() -> utoipa_axum::router::OpenApiRouter<App> {
    utoipa_axum::router::OpenApiRouter::new()
        .routes(routes!(get_sonos_speakers))
        .routes(routes!(post_sonos_play))
        .routes(routes!(get_sonos_state))
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