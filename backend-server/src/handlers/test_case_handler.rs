use crate::models::test_case::{CreateTestCaseRequest, CreateTestSuiteRequest, TestCaseAttachment};
use crate::repositories::test_case_repo::TestCaseRepository;
use crate::repositories::test_suite_repo::TestSuiteRepository;
use crate::services::test_case_service::TestCaseService;
use crate::services::test_suite_service::TestSuiteService;
use crate::state::AppState;
use aws_sdk_s3::primitives::ByteStream;
use axum::{
    extract::{Multipart, Path, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::Utc;
use mongodb::bson::{doc, oid::ObjectId};
use std::sync::Arc;
use uuid::Uuid;

#[derive(serde::Deserialize)]
pub struct TestCaseQuery {
    pub suite_id: Option<String>,
    pub limit: Option<i64>,
    pub page: Option<u64>,
}

/// GET /api/workspaces/:ws_id/test-cases
pub async fn list_test_cases(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<TestCaseQuery>,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid workspace ID").into_response(),
    };

    let repo = TestCaseRepository::new(&state.db);
    let suite_repo = TestSuiteRepository::new(&state.db);
    let service = TestCaseService::new(repo, suite_repo);

    match service.list_test_cases(&ws_oid, query.suite_id, query.limit, query.page).await {
        Ok(tcs) => (StatusCode::OK, Json(tcs)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/workspaces/:ws_id/test-suites
pub async fn list_suites(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid workspace ID").into_response(),
    };

    let repo = TestSuiteRepository::new(&state.db);
    let case_repo = TestCaseRepository::new(&state.db);
    let service = TestSuiteService::new(repo, case_repo);

    match service.list_suites(&ws_oid).await {
        Ok(suites) => (StatusCode::OK, Json(suites)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// POST /api/workspaces/:ws_id/test-suites
pub async fn create_suite(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
    Json(req): Json<CreateTestSuiteRequest>,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id) {
        Ok(oid) => oid,
        Err(_) => return (StatusCode::BAD_REQUEST, "Invalid workspace ID").into_response(),
    };

    let repo = TestSuiteRepository::new(&state.db);
    let case_repo = TestCaseRepository::new(&state.db);
    let service = TestSuiteService::new(repo, case_repo);

    match service.create_suite(ws_oid, req).await {
        Ok(suite) => (StatusCode::CREATED, Json(suite)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct UpdateTestSuiteRequest {
    pub title: String,
}

/// PATCH /api/test-suites/:id
pub async fn update_suite(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    Json(req): Json<UpdateTestSuiteRequest>,
) -> impl IntoResponse {
    let repo = TestSuiteRepository::new(&state.db);
    let case_repo = TestCaseRepository::new(&state.db);
    let service = TestSuiteService::new(repo, case_repo);

    match service.update_suite(&id, req.title).await {
        Ok(success) => {
            if success {
                StatusCode::OK.into_response()
            } else {
                (StatusCode::NOT_FOUND, "Suite not found").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

#[derive(serde::Deserialize)]
pub struct DeleteTestSuiteQuery {
    pub mode: Option<String>, // "move" or "delete"
}

/// DELETE /api/test-suites/:id
pub async fn delete_suite(
    State(state): State<Arc<AppState>>,
    Path(id): Path<String>,
    axum::extract::Query(query): axum::extract::Query<DeleteTestSuiteQuery>,
) -> impl IntoResponse {
    let repo = TestSuiteRepository::new(&state.db);
    let case_repo = TestCaseRepository::new(&state.db);
    let service = TestSuiteService::new(repo, case_repo);

    let delete_cases = query.mode.as_deref() == Some("delete");

    match service.delete_suite(&id, delete_cases).await {
        Ok(success) => {
            if success {
                StatusCode::NO_CONTENT.into_response()
            } else {
                (StatusCode::NOT_FOUND, "Suite not found").into_response()
            }
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/workspaces/:ws_id/test-cases/next-number
pub async fn get_next_test_case_number(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id) {
        Ok(oid) => oid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid workspace ID"})),
            )
                .into_response()
        }
    };

    let repo = TestCaseRepository::new(&state.db);

    match repo.next_test_no(&ws_oid).await {
        Ok(next_no) => (
            StatusCode::OK,
            Json(serde_json::json!({"next_number": next_no})),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/workspaces/:ws_id/test-cases
pub async fn create_test_case(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
    Json(req): Json<CreateTestCaseRequest>,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id) {
        Ok(oid) => oid,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid workspace ID"})),
            )
                .into_response()
        }
    };

    let repo = TestCaseRepository::new(&state.db);
    let suite_repo = TestSuiteRepository::new(&state.db);
    let service = TestCaseService::new(repo, suite_repo);

    match service.create_test_case(ws_oid, req).await {
        Ok(tc) => (StatusCode::CREATED, Json(serde_json::json!(tc)))
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": e.to_string()})),
        )
            .into_response(),
    }
}

/// POST /api/workspaces/:ws_id/test-cases/:test_case_id/attachments
pub async fn upload_test_case_attachment(
    State(state): State<Arc<AppState>>,
    Path((ws_id_str, test_case_id)): Path<(String, String)>,
    mut multipart: Multipart,
) -> impl IntoResponse {
    let ws_oid = match ObjectId::parse_str(&ws_id_str) {
        Ok(id) => id,
        Err(_) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": "Invalid workspace ID"})),
            )
        }
    };

    let repo = TestCaseRepository::new(&state.db);
    let suite_repo = TestSuiteRepository::new(&state.db);
    let service = TestCaseService::new(repo, suite_repo);

    let tc = match service.find_by_id(&test_case_id).await {
        Ok(Some(t)) => t,
        Ok(None) => {
            return (
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Test case not found"})),
            )
        }
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        }
    };

    if tc.workspace_id != ws_oid {
        return (
            StatusCode::FORBIDDEN,
            Json(
                serde_json::json!({"error": "Test case does not belong to this workspace"}),
            ),
        );
    }

    let storage = state.storage_snapshot().await;
    let client = match &storage.client {
        Some(c) => c,
        None => {
            return (
                StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({"error": "Storage is disabled"})),
            )
        }
    };

    let mut attached_files: Vec<TestCaseAttachment> = Vec::new();

    while let Some(field) = multipart.next_field().await.unwrap_or(None) {
        let file_name = field
            .file_name()
            .unwrap_or("unknown.file")
            .to_string();
        let mime_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let file_bytes = match field.bytes().await {
            Ok(b) => b,
            Err(_) => continue,
        };

        if file_bytes.is_empty() {
            continue;
        }

        let file_uuid = Uuid::now_v7().to_string();
        let file_key = format!(
            "{}/test-cases/{}/{}",
            ws_id_str, test_case_id, file_uuid
        );

        let upload_res = client
            .put_object()
            .bucket(&storage.bucket)
            .key(&file_key)
            .content_type(&mime_type)
            .body(ByteStream::from(file_bytes.clone()))
            .send()
            .await;

        match upload_res {
            Ok(_) => {
                attached_files.push(TestCaseAttachment {
                    id: file_uuid,
                    filename: file_name,
                    file_key,
                    mime_type,
                    size: file_bytes.len() as i64,
                    uploaded_at: Utc::now().to_rfc3339(),
                });
            }
            Err(e) => {
                tracing::error!(
                    "Failed to upload test case attachment: {:?}",
                    e
                );
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"error": "Storage error"})),
                );
            }
        }
    }

    if attached_files.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "No valid files received"})),
        );
    }

    let mut current = tc.attachments.unwrap_or_default();
    current.extend(attached_files.clone());

    if let Ok(bson) = mongodb::bson::to_bson(&current) {
        let updates = doc! { "attachments": bson };
        if let Err(e) = service
            .update_attachments(&test_case_id, &ws_oid, updates)
            .await
        {
            tracing::error!(
                "Failed to update test case attachments in DB: {:?}",
                e
            );
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update db"})),
            );
        }
    }

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "success": true,
            "attachments": attached_files
        })),
    )
}
