use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosTransportCommand {
    pub command: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosVolumeCommand {
    pub action: String,
    pub value: Option<u8>,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosModeCommand {
    pub command: String,
}
