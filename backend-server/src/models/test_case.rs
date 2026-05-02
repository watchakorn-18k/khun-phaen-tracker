use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

mod object_id_or_string {
    use mongodb::bson::Bson;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &String, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(value)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v = Bson::deserialize(deserializer)?;
        match v {
            Bson::String(s) => Ok(s),
            Bson::ObjectId(oid) => Ok(oid.to_hex()),
            Bson::Binary(b) if b.bytes.len() == 16 => {
                let mut arr = [0u8; 16];
                arr.copy_from_slice(&b.bytes);
                Ok(uuid::Uuid::from_bytes(arr).to_string())
            }
            other => Err(serde::de::Error::custom(format!(
                "expected string, ObjectId, or Binary UUID, got: {:?}",
                other
            ))),
        }
    }
}

// ─── Sub-structures ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GherkinStep {
    pub keyword: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassicStep {
    pub action: String,
    pub data: String,
    pub expected: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCaseAttachment {
    pub id: String,
    pub filename: String,
    pub file_key: String,
    pub mime_type: String,
    pub size: i64,
    pub uploaded_at: String,
}

// ─── Main document ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    #[serde(rename = "_id", with = "object_id_or_string")]
    pub id: String,
    pub workspace_id: ObjectId,
    pub test_no: i64,
    pub suite_id: String,
    pub name: String,
    pub description: Option<String>,
    pub preconditions: Option<String>,
    pub postconditions: Option<String>,
    pub input: Option<String>,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
    #[serde(default = "default_status")]
    pub status: String,
    #[serde(default = "default_priority")]
    pub priority: String,
    #[serde(default = "default_fixed")]
    pub fixed: String,
    pub assign_dev: Option<String>,
    pub assign_tester: Option<String>,
    pub dev_note: Option<String>,
    pub test_note: Option<String>,
    #[serde(default = "default_step_format")]
    pub step_format: String,
    pub gherkin_steps: Option<Vec<GherkinStep>>,
    pub classic_steps: Option<Vec<ClassicStep>>,
    pub attachments: Option<Vec<TestCaseAttachment>>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_status() -> String {
    "draft".to_string()
}
fn default_priority() -> String {
    "medium".to_string()
}
fn default_fixed() -> String {
    "no".to_string()
}
fn default_step_format() -> String {
    "classic".to_string()
}

// ─── Request DTOs ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTestCaseRequest {
    pub suite_id: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub preconditions: Option<String>,
    pub postconditions: Option<String>,
    pub input: Option<String>,
    pub expected_result: Option<String>,
    pub actual_result: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub fixed: Option<String>,
    pub assign_dev: Option<String>,
    pub assign_tester: Option<String>,
    pub dev_note: Option<String>,
    pub test_note: Option<String>,
    pub step_format: Option<String>,
    pub gherkin_steps: Option<Vec<GherkinStep>>,
    pub classic_steps: Option<Vec<ClassicStep>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    #[serde(rename = "_id", with = "object_id_or_string")]
    pub id: String,
    pub workspace_id: ObjectId,
    pub title: String,
    pub description: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTestSuiteRequest {
    pub title: String,
}


// ─── Tests ───────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_test_case_serialization() {
        let id = Uuid::now_v7().to_string();
        let ws_id = ObjectId::new();
        let tc = TestCase {
            id: id.clone(),
            workspace_id: ws_id,
            test_no: 1,
            suite_id: Uuid::now_v7().to_string(),
            name: "Login test".to_string(),
            description: Some("Test login flow".to_string()),
            preconditions: Some("User exists".to_string()),
            postconditions: None,
            input: None,
            expected_result: None,
            actual_result: None,
            status: "draft".to_string(),
            priority: "medium".to_string(),
            fixed: "no".to_string(),
            assign_dev: None,
            assign_tester: None,
            dev_note: None,
            test_note: None,
            step_format: "classic".to_string(),
            gherkin_steps: None,
            classic_steps: Some(vec![ClassicStep {
                action: "Go to login".to_string(),
                data: "email: test@ex.com".to_string(),
                expected: "Login page shows".to_string(),
            }]),
            attachments: None,
            created_at: Some("2026-01-01".to_string()),
            updated_at: Some("2026-01-01".to_string()),
        };

        let serialized = serde_json::to_string(&tc).unwrap();
        assert!(serialized.contains(&id));
        assert!(serialized.contains("Login test"));
    }

    #[test]
    fn test_create_request_deserialization() {
        let json = r#"{
            "suite_id": "abc-123",
            "name": "My test case"
        }"#;
        let req: CreateTestCaseRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.name, "My test case");
        assert_eq!(req.suite_id, "abc-123");
        assert!(req.status.is_none());
        assert!(req.classic_steps.is_none());
    }

    #[test]
    fn test_defaults() {
        assert_eq!(default_status(), "draft");
        assert_eq!(default_fixed(), "no");
        assert_eq!(default_step_format(), "classic");
    }

    #[test]
    fn test_gherkin_step_serialization() {
        let step = GherkinStep {
            keyword: "given".to_string(),
            text: "user is logged in".to_string(),
        };
        let json = serde_json::to_string(&step).unwrap();
        assert!(json.contains("given"));
        assert!(json.contains("user is logged in"));
    }

    #[test]
    fn test_attachment_serialization() {
        let att = TestCaseAttachment {
            id: Uuid::now_v7().to_string(),
            filename: "screenshot.png".to_string(),
            file_key: "ws/tc/file".to_string(),
            mime_type: "image/png".to_string(),
            size: 1024,
            uploaded_at: "2026-01-01".to_string(),
        };
        let json = serde_json::to_string(&att).unwrap();
        assert!(json.contains("screenshot.png"));
    }
}
