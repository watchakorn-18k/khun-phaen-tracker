use crate::models::test_case::{CreateTestSuiteRequest, TestSuite};
use crate::repositories::test_suite_repo::TestSuiteRepository;
use mongodb::bson::oid::ObjectId;
use uuid::Uuid;

#[derive(Clone)]
pub struct TestSuiteService {
    repo: TestSuiteRepository,
}

impl TestSuiteService {
    pub fn new(repo: TestSuiteRepository) -> Self {
        Self { repo }
    }

    pub async fn create_suite(&self, workspace_id: ObjectId, req: CreateTestSuiteRequest) -> mongodb::error::Result<TestSuite> {
        let now = chrono::Utc::now().to_rfc3339();
        let suite = TestSuite {
            id: Uuid::now_v7().to_string(),
            workspace_id,
            title: req.title,
            description: req.description,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };
        self.repo.create(suite).await
    }

    pub async fn list_suites(&self, workspace_id: &ObjectId) -> mongodb::error::Result<Vec<TestSuite>> {
        self.repo.find_by_workspace(workspace_id).await
    }
}
