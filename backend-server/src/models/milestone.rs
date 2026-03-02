use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Milestone {
    #[serde(rename = "_id")]
    pub id: Uuid,
    pub workspace_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub target_date: String, // ISO 8601
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateMilestoneRequest {
    pub title: String,
    pub description: Option<String>,
    pub target_date: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateMilestoneRequest {
    pub title: Option<String>,
    pub description: Option<Option<String>>,
    pub target_date: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_milestone_serialization() {
        let id = Uuid::new_v4();
        let ws_id = ObjectId::new();
        let milestone = Milestone {
            id,
            workspace_id: ws_id,
            title: "Test Milestone".to_string(),
            description: Some("Test Description".to_string()),
            target_date: "2024-12-31".to_string(),
            created_at: Some("2024-03-01".to_string()),
            updated_at: Some("2024-03-01".to_string()),
        };

        let serialized = serde_json::to_string(&milestone).unwrap();
        assert!(serialized.contains(&id.to_string()));
        assert!(serialized.contains(&ws_id.to_string()));
    }
}
