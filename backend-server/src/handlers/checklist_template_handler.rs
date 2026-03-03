use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use mongodb::bson::Document;
use uuid::Uuid;

use crate::handlers::auth_handler::extract_user_id;
use crate::models::data::*;
use crate::repositories::data_repo::DataRepository;
use crate::repositories::workspace_repo::WorkspaceRepository;
use crate::state::SharedState;

/// Helper: verify user owns workspace and return workspace_id as ObjectId
async fn verify_workspace_access(
    state: &SharedState,
    headers: &HeaderMap,
    jar: &CookieJar,
    ws_id_str: &str,
) -> Result<mongodb::bson::oid::ObjectId, axum::response::Response> {
    let user_id = extract_user_id(headers, jar, &state.jwt_secret).ok_or_else(|| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Not logged in" })),
        )
            .into_response()
    })?;

    let ws_oid = mongodb::bson::oid::ObjectId::parse_str(ws_id_str).map_err(|_| {
        (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Invalid workspace ID" })),
        )
            .into_response()
    })?;

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

pub async fn list_checklist_templates(
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
    match repo.find_checklist_templates(&ws_oid).await {
        Ok(templates) => axum::Json(ChecklistTemplateResponse {
            success: true,
            templates,
        })
        .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_checklist_template(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateChecklistTemplateRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let template_id = Uuid::new_v4().to_string();
    let template = ChecklistTemplateDocument {
        id: template_id,
        workspace_id: ws_oid,
        name: payload.name,
        items: payload.items,
        created_at: None,
        updated_at: None,
    };

    let repo = DataRepository::new(&state.db);
    match repo.create_checklist_template(template).await {
        Ok(created) => {
            axum::Json(serde_json::json!({ "success": true, "template": created })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_checklist_template(
    State(state): State<SharedState>,
    Path((ws_id, template_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateChecklistTemplateRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let mut updates = Document::new();
    if let Some(v) = payload.name {
        updates.insert("name", v);
    }
    if let Some(v) = payload.items {
        updates.insert("items", v);
    }

    if updates.is_empty() {
        return axum::Json(serde_json::json!({ "success": true, "message": "No changes" }))
            .into_response();
    }

    let repo = DataRepository::new(&state.db);
    match repo.update_checklist_template(&template_id, &ws_oid, updates).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Template not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_checklist_template(
    State(state): State<SharedState>,
    Path((ws_id, template_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let repo = DataRepository::new(&state.db);
    match repo.delete_checklist_template(&template_id, &ws_oid).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Template not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}
