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

/// Per-test-case entry inside a test run.
/// status: "pending" | "passed" | "failed" | "blocked" | "skipped" | "invalid"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRunCase {
    pub test_case_id: String,
    #[serde(default = "default_case_status")]
    pub status: String,
}

fn default_case_status() -> String {
    "pending".to_string()
}

fn default_run_status() -> String {
    "running".to_string()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestRun {
    #[serde(rename = "_id", with = "object_id_or_string")]
    pub id: String,
    pub workspace_id: ObjectId,
    pub name: String,
    pub description: Option<String>,
    #[serde(default = "default_unassigned")]
    pub default_assignee: String,
    #[serde(default)]
    pub operating_system: String,
    #[serde(default = "default_run_status")]
    pub status: String,
    #[serde(default)]
    pub test_cases: Vec<TestRunCase>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_unassigned() -> String {
    "unassigned".to_string()
}

// ─── Request DTOs ────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct CreateTestRunRequest {
    pub name: String,
    pub description: Option<String>,
    pub default_assignee: Option<String>,
    pub operating_system: Option<String>,
    pub test_case_ids: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTestRunStatusRequest {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateRunCaseStatusRequest {
    pub status: String,
}

// ─── Response DTOs ───────────────────────────────────────────

/// Stats summary embedded in the detail response.
#[derive(Debug, Serialize)]
pub struct TestRunStats {
    pub total: usize,
    pub pending: usize,
    pub passed: usize,
    pub failed: usize,
    pub blocked: usize,
    pub skipped: usize,
    pub invalid: usize,
}

/// Enriched test case entry (joins TestCase fields).
#[derive(Debug, Serialize)]
pub struct TestRunCaseDetail {
    pub test_case_id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_case: Option<crate::models::test_case::TestCase>,
}

/// Full detail response for a single test run.
#[derive(Debug, Serialize)]
pub struct TestRunDetail {
    pub id: String,
    pub workspace_id: String,
    pub name: String,
    pub description: Option<String>,
    pub default_assignee: String,
    pub operating_system: String,
    pub status: String,
    pub test_cases: Vec<TestRunCaseDetail>,
    pub stats: TestRunStats,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_by: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}
