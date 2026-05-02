use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use mongodb::bson::{oid::ObjectId, Document};

use crate::handlers::common::verify_workspace_access;
use crate::models::data::*;
use crate::repositories::data_repo::DataRepository;
use crate::state::SharedState;

pub async fn list_sprints(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let repo = DataRepository::new(&state.db);
    match repo.find_sprints(&ws_oid).await {
        Ok(sprints) => {
            axum::Json(serde_json::json!({ "success": true, "sprints": sprints })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_sprint(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateSprintRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let sprint = SprintDocument {
        id: None,
        workspace_id: ws_oid,
        name: payload.name,
        start_date: payload.start_date,
        end_date: payload.end_date,
        status: payload.status,
        completed_at: None,
        archived_count: None,
        created_at: None,
    };

    let repo = DataRepository::new(&state.db);
    match repo.create_sprint(sprint).await {
        Ok(created) => {
            axum::Json(serde_json::json!({ "success": true, "sprint": created })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_sprint(
    State(state): State<SharedState>,
    Path((ws_id, sprint_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateSprintRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let spr_oid = match ObjectId::parse_str(&sprint_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid sprint ID" })),
            )
                .into_response()
        }
    };

    let mut updates = Document::new();
    if let Some(v) = payload.name {
        updates.insert("name", v);
    }
    if let Some(v) = payload.start_date {
        updates.insert("start_date", v);
    }
    if let Some(v) = payload.end_date {
        updates.insert("end_date", v);
    }
    if let Some(v) = payload.status {
        updates.insert("status", v);
    }
    if let Some(v) = payload.completed_at {
        match v {
            Some(at) => {
                updates.insert("completed_at", at);
            }
            None => {
                updates.insert("completed_at", mongodb::bson::Bson::Null);
            }
        }
    }
    if let Some(v) = payload.archived_count {
        updates.insert("archived_count", v);
    }

    if updates.is_empty() {
        return axum::Json(serde_json::json!({ "success": true })).into_response();
    }

    let repo = DataRepository::new(&state.db);
    match repo.update_sprint(&spr_oid, &ws_oid, updates).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Sprint not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_sprint(
    State(state): State<SharedState>,
    Path((ws_id, sprint_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let spr_oid = match ObjectId::parse_str(&sprint_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid sprint ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    match repo.delete_sprint(&spr_oid, &ws_oid).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Sprint not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

