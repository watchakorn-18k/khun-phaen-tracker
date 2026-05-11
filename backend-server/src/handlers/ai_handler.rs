use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use serde::Deserialize;

use crate::handlers::auth_handler::extract_claims;
use crate::handlers::common::verify_workspace_access;
use crate::models::ai::{AiConfigDocument, UpdateAiConfigRequest};
use crate::models::data::TaskFilterQuery;
use crate::repositories::ai_repo::AiRepository;
use crate::repositories::data_repo::DataRepository;
use crate::services::ai_service::{AiConfig, AiService, LlmConfig};
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

    let ai_repo = AiRepository::new(&state.db);
    let stored_config = ai_repo.get_ai_config().await.ok().flatten();

    let config = stored_config
        .as_ref()
        .and_then(AiConfig::from_doc)
        .or_else(AiConfig::from_env);

    let config = match config {
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

#[derive(Debug, Deserialize)]
pub struct AiGenerateTaskRequest {
    pub prompt: String,
}

pub async fn generate_task(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<AiGenerateTaskRequest>,
) -> axum::response::Response {
    let prompt = payload.prompt.trim();
    if prompt.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "prompt is required" })),
        )
            .into_response();
    }

    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let ai_repo = AiRepository::new(&state.db);
    let stored_config = ai_repo.get_ai_config().await.ok().flatten();

    let llm_config = stored_config
        .as_ref()
        .and_then(LlmConfig::from_doc)
        .or_else(LlmConfig::from_env);

    let llm_config = match llm_config {
        Some(config) => config,
        None => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "LLM config is missing" })),
            )
                .into_response()
        }
    };

    // Build context from workspace
    let data_repo = DataRepository::new(&state.db);
    let projects = data_repo.find_projects(&ws_oid).await.unwrap_or_default();
    let assignees = data_repo.find_assignees(&ws_oid).await.unwrap_or_default();

    let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();
    let assignee_names: Vec<String> = assignees.iter().map(|a| a.name.clone()).collect();

    let context = format!(
        "Available projects: {}\nAvailable assignees: {}",
        if project_names.is_empty() { "None".to_string() } else { project_names.join(", ") },
        if assignee_names.is_empty() { "None".to_string() } else { assignee_names.join(", ") }
    );

    match AiService::generate_task(&llm_config, prompt, &context).await {
        Ok(json_str) => {
            // Try to parse and validate the JSON
            match serde_json::from_str::<serde_json::Value>(&json_str) {
                Ok(task_json) => axum::Json(serde_json::json!({
                    "success": true,
                    "task": task_json
                }))
                .into_response(),
                Err(_) => {
                    // If parsing fails, return the raw response
                    axum::Json(serde_json::json!({
                        "success": true,
                        "task": {
                            "title": prompt,
                            "category": "feature",
                            "priority": "medium",
                            "notes": json_str,
                            "project": ""
                        }
                    }))
                    .into_response()
                }
            }
        }
        Err(e) => (
            axum::http::StatusCode::BAD_GATEWAY,
            axum::Json(serde_json::json!({ "error": e })),
        )
            .into_response(),
    }
}

fn ensure_admin(
    headers: &HeaderMap,
    jar: &CookieJar,
    state: &SharedState,
) -> Result<(), axum::response::Response> {
    let claims = match extract_claims(headers, jar, &state.jwt_secret) {
        Some(c) => c,
        None => {
            return Err((
                axum::http::StatusCode::UNAUTHORIZED,
                Json(serde_json::json!({ "error": "Unauthorized" })),
            )
                .into_response())
        }
    };

    if claims.role != "admin" {
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({ "error": "Admin access required" })),
        )
            .into_response());
    }

    Ok(())
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

pub async fn get_ai_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let repo = AiRepository::new(&state.db);
    let stored = match repo.get_ai_config().await {
        Ok(config) => config,
        Err(error) => {
            tracing::error!("Failed to load AI config: {:?}", error);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to load AI config" })),
            )
                .into_response();
        }
    };

    let env_url = std::env::var("AI_EMBEDDINGS_URL").ok();
    let env_key = std::env::var("AI_EMBEDDINGS_API_KEY").ok();
    let env_model = std::env::var("AI_EMBEDDINGS_MODEL").ok();
    let env_llm_url = std::env::var("AI_LLM_URL").ok();
    let env_llm_key = std::env::var("AI_LLM_API_KEY").ok();
    let env_llm_model = std::env::var("AI_LLM_MODEL").ok();

    Json(serde_json::json!({
        "success": true,
        "config": {
            "embeddings_url": stored.as_ref().and_then(|c| c.embeddings_url.clone()).or(env_url),
            "embeddings_api_key": stored.as_ref().and_then(|c| c.embeddings_api_key.clone()).or(env_key),
            "embeddings_model": stored.as_ref().and_then(|c| c.embeddings_model.clone()).or(env_model),
            "llm_url": stored.as_ref().and_then(|c| c.llm_url.clone()).or(env_llm_url),
            "llm_api_key": stored.as_ref().and_then(|c| c.llm_api_key.clone()).or(env_llm_key),
            "llm_model": stored.as_ref().and_then(|c| c.llm_model.clone()).or(env_llm_model),
            "updated_at": stored.as_ref().and_then(|c| c.updated_at.clone())
        }
    }))
    .into_response()
}

pub async fn update_ai_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: HeaderMap,
    Json(payload): Json<UpdateAiConfigRequest>,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let embeddings_url = normalize_optional(payload.embeddings_url);
    let embeddings_api_key = normalize_optional(payload.embeddings_api_key);
    let embeddings_model = normalize_optional(payload.embeddings_model);
    let llm_url = normalize_optional(payload.llm_url);
    let llm_api_key = normalize_optional(payload.llm_api_key);
    let llm_model = normalize_optional(payload.llm_model);

    let config = AiConfigDocument {
        key: "ai_config".to_string(),
        embeddings_url,
        embeddings_api_key,
        embeddings_model,
        llm_url,
        llm_api_key,
        llm_model,
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let repo = AiRepository::new(&state.db);
    if let Err(error) = repo.save_ai_config(&config).await {
        tracing::error!("Failed to save AI config: {:?}", error);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to save AI config" })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "config": {
            "embeddings_url": config.embeddings_url,
            "embeddings_api_key": config.embeddings_api_key,
            "embeddings_model": config.embeddings_model,
            "llm_url": config.llm_url,
            "llm_api_key": config.llm_api_key,
            "llm_model": config.llm_model,
            "updated_at": config.updated_at
        }
    }))
    .into_response()
}

pub async fn reset_ai_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let repo = AiRepository::new(&state.db);
    if let Err(error) = repo.delete_ai_config().await {
        tracing::error!("Failed to reset AI config: {:?}", error);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to reset AI config" })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "message": "AI config reset to environment defaults"
    }))
    .into_response()
}
