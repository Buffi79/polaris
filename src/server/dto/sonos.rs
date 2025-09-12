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

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosSetVolumeBody {
    #[schema(example = 50)]
    pub volume: u8,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct SonosSeekBody {
    #[schema(example = 120)]
    pub position: u32,
}
