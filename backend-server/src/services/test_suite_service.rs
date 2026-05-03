use crate::models::test_case::{CreateTestSuiteRequest, TestSuite};
use crate::repositories::test_suite_repo::TestSuiteRepository;
use crate::repositories::test_case_repo::TestCaseRepository;
use mongodb::bson::{doc, oid::ObjectId};
use uuid::Uuid;

#[derive(Clone)]
pub struct TestSuiteService {
    repo: TestSuiteRepository,
    case_repo: TestCaseRepository,
}

impl TestSuiteService {
    pub fn new(repo: TestSuiteRepository, case_repo: TestCaseRepository) -> Self {
        Self { repo, case_repo }
    }

    pub async fn create_suite(&self, workspace_id: ObjectId, req: CreateTestSuiteRequest) -> mongodb::error::Result<TestSuite> {
        let now = chrono::Utc::now().to_rfc3339();
        let suite = TestSuite {
            id: Uuid::now_v7().to_string(),
            workspace_id,
            title: req.title,
            description: None,
            created_at: Some(now.clone()),
            updated_at: Some(now),
        };
        self.repo.create(suite).await
    }

    pub async fn update_suite(&self, id: &str, title: String) -> mongodb::error::Result<bool> {
        let now = chrono::Utc::now().to_rfc3339();
        self.repo.update(id, doc! { "title": title, "updated_at": now }).await
    }

    pub async fn delete_suite(&self, id: &str, delete_cases: bool) -> mongodb::error::Result<bool> {
        if id == "unassigned" {
            if delete_cases {
                self.case_repo.delete_unassigned().await?;
            }
            return Ok(true);
        }

        if delete_cases {
            self.case_repo.delete_by_suite(id).await?;
        } else {
            self.case_repo.move_to_unassigned(id).await?;
        }
        self.repo.delete(id).await
    }

    pub async fn list_suites(&self, workspace_id: &ObjectId) -> mongodb::error::Result<Vec<TestSuite>> {
        self.repo.find_by_workspace(workspace_id).await
    }
}
