use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiConfigDocument {
    pub key: String,
    pub embeddings_url: Option<String>,
    pub embeddings_api_key: Option<String>,
    pub embeddings_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateAiConfigRequest {
    pub embeddings_url: Option<String>,
    pub embeddings_api_key: Option<String>,
    pub embeddings_model: Option<String>,
}
