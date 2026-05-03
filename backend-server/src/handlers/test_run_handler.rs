use crate::handlers::auth_handler::extract_claims;
use crate::models::test_run::{
    CreateTestRunRequest, TestRun, TestRunCase, TestRunCaseDetail, TestRunDetail, TestRunStats,
    UpdateRunCaseStatusRequest, UpdateTestRunStatusRequest,
};
use crate::repositories::test_case_repo::TestCaseRepository;
use crate::repositories::test_run_repo::TestRunRepository;
use crate::state::AppState;
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ListTestRunsQuery {
    pub page: Option<u64>,
    pub limit: Option<u64>,
}

#[derive(Serialize)]
pub struct PaginatedTestRuns {
    pub data: Vec<TestRun>,
    pub total: u64,
    pub page: u64,
    pub limit: u64,
}

// ─── Helpers ─────────────────────────────────────────────────

fn parse_ws(ws_id: &str) -> Result<ObjectId, Response> {
    ObjectId::parse_str(ws_id)
        .map_err(|_| (StatusCode::BAD_REQUEST, "Invalid workspace ID").into_response())
}

fn require_auth(
    headers: &HeaderMap,
    jar: &CookieJar,
    secret: &str,
) -> Result<crate::models::auth::Claims, Response> {
    extract_claims(headers, jar, secret)
        .ok_or_else(|| (StatusCode::UNAUTHORIZED, "Unauthorized").into_response())
}

fn now() -> String {
    Utc::now().to_rfc3339()
}

fn compute_stats(cases: &[TestRunCase]) -> TestRunStats {
    let mut stats = TestRunStats {
        total: cases.len(),
        pending: 0,
        passed: 0,
        failed: 0,
        blocked: 0,
        skipped: 0,
        invalid: 0,
    };
    for c in cases {
        match c.status.as_str() {
            "passed" => stats.passed += 1,
            "failed" => stats.failed += 1,
            "blocked" => stats.blocked += 1,
            "skipped" => stats.skipped += 1,
            "invalid" => stats.invalid += 1,
            _ => stats.pending += 1,
        }
    }
    stats
}

async fn enrich_run(run: TestRun, tc_repo: &TestCaseRepository) -> TestRunDetail {
    let mut enriched_cases: Vec<TestRunCaseDetail> = Vec::with_capacity(run.test_cases.len());

    for entry in &run.test_cases {
        let tc = tc_repo.find_by_id(&entry.test_case_id).await.ok().flatten();
        enriched_cases.push(TestRunCaseDetail {
            test_case_id: entry.test_case_id.clone(),
            status: entry.status.clone(),
            test_case: tc,
        });
    }

    let stats = compute_stats(&run.test_cases);

    TestRunDetail {
        id: run.id,
        workspace_id: run.workspace_id.to_hex(),
        name: run.name,
        description: run.description,
        default_assignee: run.default_assignee,
        operating_system: run.operating_system,
        status: run.status,
        test_cases: enriched_cases,
        stats,
        created_by: run.created_by,
        created_at: run.created_at,
        updated_at: run.updated_at,
    }
}

// ─── Handlers ────────────────────────────────────────────────

/// GET /api/workspaces/:ws_id/test-runs?page=1&limit=10
pub async fn list_test_runs(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(ws_id): Path<String>,
    Query(q): Query<ListTestRunsQuery>,
) -> impl IntoResponse {
    let _claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let page = q.page.unwrap_or(1).max(1);
    let limit = q.limit.unwrap_or(10).clamp(1, 100);

    let repo = TestRunRepository::new(&state.db);
    match repo.find_by_workspace(&ws_oid, page, limit).await {
        Ok((data, total)) => (StatusCode::OK, Json(PaginatedTestRuns { data, total, page, limit })).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// POST /api/workspaces/:ws_id/test-runs
pub async fn create_test_run(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path(ws_id): Path<String>,
    Json(req): Json<CreateTestRunRequest>,
) -> impl IntoResponse {
    let claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let cases = req
        .test_case_ids
        .into_iter()
        .map(|id| TestRunCase {
            test_case_id: id,
            status: "pending".to_string(),
        })
        .collect::<Vec<_>>();

    let ts = now();
    let run = TestRun {
        id: Uuid::now_v7().to_string(),
        workspace_id: ws_oid,
        name: req.name,
        description: req.description,
        default_assignee: req.default_assignee.unwrap_or_else(|| "unassigned".to_string()),
        operating_system: req.operating_system.unwrap_or_default(),
        status: "running".to_string(),
        test_cases: cases,
        created_by: Some(claims.sub),
        created_at: Some(ts.clone()),
        updated_at: Some(ts),
    };

    let repo = TestRunRepository::new(&state.db);
    match repo.create(run).await {
        Ok(created) => (StatusCode::CREATED, Json(created)).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/workspaces/:ws_id/test-runs/:run_id
pub async fn get_test_run(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path((ws_id, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let repo = TestRunRepository::new(&state.db);
    let run = match repo.find_by_id(&run_id).await {
        Ok(Some(r)) if r.workspace_id == ws_oid => r,
        Ok(Some(_)) => return (StatusCode::FORBIDDEN, "Not in this workspace").into_response(),
        Ok(None) => return (StatusCode::NOT_FOUND, "Test run not found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let tc_repo = TestCaseRepository::new(&state.db);
    let detail = enrich_run(run, &tc_repo).await;
    (StatusCode::OK, Json(detail)).into_response()
}

/// PATCH /api/workspaces/:ws_id/test-runs/:run_id/status
pub async fn update_test_run_status(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path((ws_id, run_id)): Path<(String, String)>,
    Json(req): Json<UpdateTestRunStatusRequest>,
) -> impl IntoResponse {
    let _claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let valid = ["running", "completed", "aborted"];
    if !valid.contains(&req.status.as_str()) {
        return (StatusCode::BAD_REQUEST, "Invalid status").into_response();
    }

    let repo = TestRunRepository::new(&state.db);
    match repo.update_status(&run_id, &ws_oid, &req.status, &now()).await {
        Ok(true) => StatusCode::OK.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Test run not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// PATCH /api/workspaces/:ws_id/test-runs/:run_id/cases/:tc_id/status
pub async fn update_run_case_status(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path((ws_id, run_id, tc_id)): Path<(String, String, String)>,
    Json(req): Json<UpdateRunCaseStatusRequest>,
) -> impl IntoResponse {
    let _claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let valid = ["pending", "passed", "failed", "blocked", "skipped", "invalid"];
    if !valid.contains(&req.status.as_str()) {
        return (StatusCode::BAD_REQUEST, "Invalid status").into_response();
    }

    let repo = TestRunRepository::new(&state.db);
    match repo
        .update_case_status(&run_id, &ws_oid, &tc_id, &req.status, &now())
        .await
    {
        Ok(true) => {
            // Sync main test case status on pass/fail
            // Map test run statuses to test case statuses: "passed" → "passed", "failed" → "failed"
            if req.status == "passed" || req.status == "failed" {
                let tc_status = if req.status == "passed" { "passed" } else { "failed" };
                let tc_repo = TestCaseRepository::new(&state.db);
                let _ = tc_repo.update_status(&tc_id, tc_status, &now()).await;
            }
            StatusCode::OK.into_response()
        }
        Ok(false) => {
            (StatusCode::NOT_FOUND, "Test run or test case not found").into_response()
        }
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

/// GET /api/public/workspaces/:ws_id/test-runs/latest  (no auth required)
pub async fn get_latest_public_test_run(
    State(state): State<Arc<AppState>>,
    Path(ws_id): Path<String>,
) -> impl IntoResponse {
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let repo = TestRunRepository::new(&state.db);
    let run = match repo.find_latest_by_workspace(&ws_oid).await {
        Ok(Some(r)) => r,
        Ok(None) => return (StatusCode::NOT_FOUND, "No test runs found").into_response(),
        Err(e) => return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    };

    let tc_repo = TestCaseRepository::new(&state.db);
    let detail = enrich_run(run, &tc_repo).await;
    (StatusCode::OK, Json(detail)).into_response()
}

/// DELETE /api/workspaces/:ws_id/test-runs/:run_id
pub async fn delete_test_run(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    headers: HeaderMap,
    Path((ws_id, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let _claims = match require_auth(&headers, &jar, &state.jwt_secret) {
        Ok(c) => c,
        Err(r) => return r,
    };
    let ws_oid = match parse_ws(&ws_id) {
        Ok(o) => o,
        Err(r) => return r,
    };

    let repo = TestRunRepository::new(&state.db);
    match repo.delete(&run_id, &ws_oid).await {
        Ok(true) => StatusCode::NO_CONTENT.into_response(),
        Ok(false) => (StatusCode::NOT_FOUND, "Test run not found").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}
