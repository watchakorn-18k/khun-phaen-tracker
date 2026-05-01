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

pub async fn list_projects(
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
    match repo.find_projects(&ws_oid).await {
        Ok(projects) => {
            axum::Json(serde_json::json!({ "success": true, "projects": projects })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_project(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateProjectRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let project = ProjectDocument {
        id: None,
        workspace_id: ws_oid,
        name: payload.name,
        repo_url: payload.repo_url,
        created_at: None,
    };

    let repo = DataRepository::new(&state.db);
    match repo.create_project(project).await {
        Ok(created) => {
            axum::Json(serde_json::json!({ "success": true, "project": created })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_project(
    State(state): State<SharedState>,
    Path((ws_id, project_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateProjectRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let proj_oid = match ObjectId::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid project ID" })),
            )
                .into_response()
        }
    };

    let mut updates = Document::new();
    if let Some(v) = payload.name {
        updates.insert("name", v);
    }
    if let Some(v) = payload.repo_url {
        match v {
            Some(url) => {
                updates.insert("repo_url", url);
            }
            None => {
                updates.insert("repo_url", mongodb::bson::Bson::Null);
            }
        }
    }

    if updates.is_empty() {
        return axum::Json(serde_json::json!({ "success": true })).into_response();
    }

    let repo = DataRepository::new(&state.db);
    match repo.update_project(&proj_oid, &ws_oid, updates).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Project not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_project(
    State(state): State<SharedState>,
    Path((ws_id, project_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let proj_oid = match ObjectId::parse_str(&project_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid project ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    match repo.delete_project(&proj_oid, &ws_oid).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Project not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn get_project_stats(
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
    let projects = match repo.find_projects(&ws_oid).await {
        Ok(rows) => rows,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("{}", e) })),
            )
                .into_response()
        }
    };

    let project_names: Vec<String> = projects.iter().map(|p| p.name.clone()).collect();

    match repo
        .count_tasks_by_project_names(&ws_oid, &project_names)
        .await
    {
        Ok(counts) => {
            let stats: Vec<serde_json::Value> = projects
                .into_iter()
                .map(|p| {
                    serde_json::json!({
                        "id": p.id.map(|id| id.to_hex()).unwrap_or_default(),
                        "taskCount": counts.get(&p.name).copied().unwrap_or(0)
                    })
                })
                .collect();
            axum::Json(serde_json::json!({ "success": true, "stats": stats })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}
