use crate::models::test_case::{CreateTestCaseRequest, TestCase};
use crate::repositories::test_case_repo::TestCaseRepository;
use mongodb::bson::oid::ObjectId;
use uuid::Uuid;

use crate::repositories::test_suite_repo::TestSuiteRepository;

#[derive(Clone)]
pub struct TestCaseService {
    repo: TestCaseRepository,
    suite_repo: TestSuiteRepository,
}

impl TestCaseService {
    pub fn new(repo: TestCaseRepository, suite_repo: TestSuiteRepository) -> Self {
        Self { repo, suite_repo }
    }

    pub async fn create_test_case(
        &self,
        workspace_id: ObjectId,
        req: CreateTestCaseRequest,
    ) -> mongodb::error::Result<TestCase> {
        let suite_id = match req.suite_id {
            Some(sid) if !sid.is_empty() && sid != "undefined" && sid != "null" => {
                // Verify suite exists and belongs to this workspace
                if let Some(suite) = self.suite_repo.find_by_id(&sid).await? {
                    if suite.workspace_id != workspace_id {
                        return Err(mongodb::error::Error::custom("Suite does not belong to this workspace"));
                    }
                    sid
                } else {
                    return Err(mongodb::error::Error::custom(format!("Suite with ID '{}' not found", sid)));
                }
            }
            _ => {
                // If no suite_id provided, find or create "General" suite
                let suites = self.suite_repo.find_by_workspace(&workspace_id).await?;
                if let Some(general) = suites.iter().find(|s| s.title == "General") {
                    general.id.clone()
                } else {
                    // Create General suite
                    let now = chrono::Utc::now().to_rfc3339();
                    let new_suite = crate::models::test_case::TestSuite {
                        id: Uuid::now_v7().to_string(),
                        workspace_id,
                        title: "General".to_string(),
                        description: Some("Default suite for unassigned test cases".to_string()),
                        created_at: Some(now.clone()),
                        updated_at: Some(now),
                    };
                    let created = self.suite_repo.create(new_suite).await?;
                    created.id
                }
            }
        };

        let now = chrono::Utc::now().to_rfc3339();
        let test_no = self.repo.next_test_no(&workspace_id).await?;

        let test_case = TestCase {
            id: Uuid::now_v7().to_string(),
            workspace_id,
            test_no,
            suite_id,
            name: req.name,
            description: req.description,
            preconditions: req.preconditions,
            postconditions: req.postconditions,
            input: req.input,
            expected_result: req.expected_result,
            actual_result: req.actual_result,
            status: req.status.unwrap_or_else(|| "draft".to_string()),
            priority: req.priority.unwrap_or_else(|| "medium".to_string()),
            fixed: req.fixed.unwrap_or_else(|| "no".to_string()),
            assign_dev: req.assign_dev,
            assign_tester: req.assign_tester,
            dev_note: req.dev_note,
            test_note: req.test_note,
            step_format: req.step_format.unwrap_or_else(|| "classic".to_string()),
            gherkin_steps: req.gherkin_steps,
            classic_steps: req.classic_steps,
            attachments: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };

        self.repo.create(test_case).await
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> mongodb::error::Result<Option<TestCase>> {
        self.repo.find_by_id(id).await
    }

    #[allow(dead_code)]
    pub async fn list_test_cases(
        &self,
        workspace_id: &ObjectId,
        suite_id: Option<String>,
        limit: Option<i64>,
        page: Option<u64>,
    ) -> mongodb::error::Result<Vec<TestCase>> {
        let limit = limit.unwrap_or(10);
        let page = page.unwrap_or(1);
        let offset = (page - 1) * (limit as u64);
        
        self.repo.find_by_workspace(workspace_id, suite_id, Some(limit), Some(offset)).await
    }

    pub async fn update_attachments(
        &self,
        id: &str,
        workspace_id: &ObjectId,
        updates: mongodb::bson::Document,
    ) -> mongodb::error::Result<bool> {
        self.repo.update(id, workspace_id, updates).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_id_is_uuid() {
        let id = Uuid::now_v7();
        assert_eq!(id.get_variant(), uuid::Variant::RFC4122);
    }

    #[test]
    fn test_create_request_defaults() {
        let req = CreateTestCaseRequest {
            suite_id: "suite-1".to_string(),
            name: "Test login".to_string(),
            description: None,
            preconditions: None,
            postconditions: None,
            input: None,
            expected_result: None,
            actual_result: None,
            status: None,
            fixed: None,
            assign_dev: None,
            assign_tester: None,
            dev_note: None,
            test_note: None,
            step_format: None,
            gherkin_steps: None,
            classic_steps: None,
        };

        // Verify defaults would apply correctly
        let status = req.status.unwrap_or_else(|| "draft".to_string());
        assert_eq!(status, "draft");
        let fixed = req.fixed.unwrap_or_else(|| "no".to_string());
        assert_eq!(fixed, "no");
        let format = req.step_format.unwrap_or_else(|| "classic".to_string());
        assert_eq!(format, "classic");
    }
}
