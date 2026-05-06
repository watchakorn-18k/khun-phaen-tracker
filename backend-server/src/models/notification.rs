use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserNotification {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub recipient_user_id: String,
    pub source_type: String,
    pub source_id: String,
    pub source_title: String,
    pub workspace_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub suite_id: Option<String>,
    #[serde(default)]
    pub assignment_ids: Vec<String>,
    #[serde(default)]
    pub recipient_user_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_user_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub actor_name: Option<String>,
    pub created_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub read_at: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateNotificationRequest {
    pub source_type: String,
    pub source_id: String,
    pub source_title: String,
    pub workspace_id: String,
    #[serde(default)]
    pub suite_id: Option<String>,
    #[serde(default)]
    pub assignment_ids: Vec<String>,
    #[serde(default)]
    pub recipient_user_ids: Vec<String>,
    #[serde(default)]
    pub actor_user_id: Option<String>,
    #[serde(default)]
    pub actor_name: Option<String>,
    pub created_at: String,
}
