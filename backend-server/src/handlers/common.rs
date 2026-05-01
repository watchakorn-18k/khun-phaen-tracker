use axum::{
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use mongodb::bson::oid::ObjectId;

use crate::handlers::auth_handler::extract_user_id;
use crate::repositories::workspace_repo::WorkspaceRepository;
use crate::repositories::data_repo::DataRepository;
use crate::state::SharedState;

/// Helper: verify user owns workspace and return workspace_id as ObjectId
pub async fn verify_workspace_access(
    state: &SharedState,
    headers: &HeaderMap,
    jar: &CookieJar,
    ws_id_str: &str,
) -> Result<ObjectId, axum::response::Response> {
    // Auth check
    let user_id = extract_user_id(headers, jar, &state.jwt_secret).ok_or_else(|| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Not logged in" })),
        )
            .into_response()
    })?;

    // Parse workspace_id
    let ws_oid = ObjectId::parse_str(ws_id_str).map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Invalid workspace ID" })),
        )
            .into_response()
    })?;

    // Check ownership
    let ws_repo = WorkspaceRepository::new(&state.db);
    let ws = ws_repo.find_by_id(&ws_oid).await.map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Database error" })),
        )
            .into_response()
    })?;

    match ws {
        Some(w) => {
            if w.owner_id == user_id {
                return Ok(ws_oid);
            }
            if let Some(id) = w.id {
                let data_repo = DataRepository::new(&state.db);
                if let Ok(assigned) = data_repo.find_assigned_workspaces(&user_id.to_hex()).await {
                    if assigned.contains(&id) {
                        return Ok(ws_oid);
                    }
                }
            }
            Err((
                axum::http::StatusCode::FORBIDDEN,
                axum::Json(serde_json::json!({ "error": "Access denied to this workspace" })),
            )
                .into_response())
        }
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Workspace not found" })),
        )
            .into_response()),
    }
}

pub async fn verify_task_belongs_to_workspace(
    repo: &DataRepository,
    ws_oid: &ObjectId,
    task_oid: &ObjectId,
) -> Result<(), axum::response::Response> {
    let task = repo.find_task_by_id(task_oid).await.map_err(|_| {
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": "Database error" })),
        )
            .into_response()
    })?;

    match task {
        Some(t) if &t.workspace_id == ws_oid => Ok(()),
        Some(_) => Err((
            axum::http::StatusCode::FORBIDDEN,
            axum::Json(serde_json::json!({ "error": "Task does not belong to this workspace" })),
        )
            .into_response()),
        None => Err((
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Task not found" })),
        )
            .into_response()),
    }
}

pub fn is_duplicate_key_error(error: &mongodb::error::Error) -> bool {
    error.to_string().contains("E11000")
}
