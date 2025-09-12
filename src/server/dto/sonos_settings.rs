use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct SonosSettings {
    #[schema(example = "http://localhost:5005")]
    pub http_api_url: String,
    #[schema(example = "http://polaris.local:5000/mp3")]
    pub mp3_server_url: String,
}
