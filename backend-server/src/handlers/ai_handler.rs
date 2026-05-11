use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::handlers::common::verify_workspace_access;
use crate::models::data::TaskFilterQuery;
use crate::repositories::data_repo::DataRepository;
use crate::services::ai_service::{AiConfig, AiService};
use crate::state::SharedState;

#[derive(Debug, Deserialize)]
pub struct AiTaskChatRequest {
    pub message: String,
    pub limit: Option<usize>,
    pub task_limit: Option<u64>,
}

pub async fn chat_with_tasks(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<AiTaskChatRequest>,
) -> axum::response::Response {
    let message = payload.message.trim();
    if message.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "message is required" })),
        )
            .into_response();
    }

    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let config = match AiConfig::from_env() {
        Some(config) => config,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "AI embeddings config is missing" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    let task_limit = payload.task_limit.unwrap_or(100).clamp(1, 500);
    let filter = TaskFilterQuery {
        search: None,
        include_archived: Some(false),
        limit: Some(task_limit),
        page: Some(1),
        sort_by: Some("updated_at".to_string()),
        sort_order: Some("desc".to_string()),
        ..Default::default()
    };

    let tasks = match repo.find_tasks(&ws_oid, &filter).await {
        Ok((tasks, _)) => tasks,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("Failed to load tasks: {}", e) })),
            )
                .into_response()
        }
    };

    match AiService::search_tasks(&config, message, tasks, payload.limit.unwrap_or(5).clamp(1, 20)).await {
        Ok(answer) => axum::Json(serde_json::json!({
            "success": true,
            "answer": answer.message,
            "results": answer.results,
        }))
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}
