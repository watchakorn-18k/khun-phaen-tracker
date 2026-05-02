use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use futures::StreamExt;
use mongodb::bson::{doc, oid::ObjectId, Document};

use crate::handlers::common::verify_workspace_access;
use crate::models::data::*;
use crate::repositories::data_repo::DataRepository;
use crate::state::SharedState;

pub async fn list_assignees(
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
    match repo.find_assignees(&ws_oid).await {
        Ok(assignees) => {
            let linked_user_ids: Vec<String> =
                assignees.iter().filter_map(|a| a.user_id.clone()).collect();

            let mut email_by_user_id = std::collections::HashMap::new();
            if !linked_user_ids.is_empty() {
                let users_coll = state.db.collection::<crate::models::user::User>("users");
                let linked_object_ids: Vec<ObjectId> = linked_user_ids
                    .iter()
                    .filter_map(|id| ObjectId::parse_str(id).ok())
                    .collect();

                if let Ok(mut cursor) = users_coll
                    .find(
                        doc! {
                            "$or": [
                                { "user_id": { "$in": &linked_user_ids } },
                                { "_id": { "$in": &linked_object_ids } }
                            ]
                        },
                        None,
                    )
                    .await
                {
                    while let Some(result) = cursor.next().await {
                        if let Ok(user) = result {
                            let email = user.email;
                            email_by_user_id.insert(user.user_id, email.clone());
                            if let Some(oid) = user.id {
                                email_by_user_id.insert(oid.to_hex(), email);
                            }
                        }
                    }
                }
            }

            let assignees_with_email: Vec<serde_json::Value> = assignees
                .into_iter()
                .map(|a| {
                    let mut value =
                        serde_json::to_value(&a).unwrap_or_else(|_| serde_json::json!({}));
                    if let (Some(user_id), Some(email)) = (
                        a.user_id.as_ref(),
                        a.user_id
                            .as_ref()
                            .and_then(|id| email_by_user_id.get(id).cloned()),
                    ) {
                        if let Some(obj) = value.as_object_mut() {
                            obj.insert("user_id".to_string(), serde_json::json!(user_id));
                            obj.insert("email".to_string(), serde_json::json!(email));
                        }
                    }
                    value
                })
                .collect();

            axum::Json(serde_json::json!({ "success": true, "assignees": assignees_with_email }))
                .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_assignee(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateAssigneeRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let assignee = AssigneeDocument {
        id: None,
        workspace_id: ws_oid,
        name: payload.name,
        color: payload.color,
        discord_id: payload.discord_id,
        user_id: payload.user_id,
        created_at: None,
    };

    let repo = DataRepository::new(&state.db);
    match repo.create_assignee(assignee).await {
        Ok(created) => {
            axum::Json(serde_json::json!({ "success": true, "assignee": created })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_assignee(
    State(state): State<SharedState>,
    Path((ws_id, assignee_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateAssigneeRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let ass_oid = match ObjectId::parse_str(&assignee_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid assignee ID" })),
            )
                .into_response()
        }
    };

    let mut updates = Document::new();
    if let Some(v) = payload.name {
        updates.insert("name", v);
    }
    if let Some(v) = payload.color {
        updates.insert("color", v);
    }
    if let Some(v) = payload.discord_id {
        match v {
            Some(did) => {
                updates.insert("discord_id", did);
            }
            None => {
                updates.insert("discord_id", mongodb::bson::Bson::Null);
            }
        }
    }
    if let Some(v) = payload.user_id {
        match v {
            Some(uid) => {
                updates.insert("user_id", uid);
            }
            None => {
                updates.insert("user_id", mongodb::bson::Bson::Null);
            }
        }
    }

    if updates.is_empty() {
        return axum::Json(serde_json::json!({ "success": true })).into_response();
    }

    let repo = DataRepository::new(&state.db);
    match repo.update_assignee(&ass_oid, &ws_oid, updates).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Assignee not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_assignee(
    State(state): State<SharedState>,
    Path((ws_id, assignee_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let ass_oid = match ObjectId::parse_str(&assignee_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid assignee ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    match repo.delete_assignee(&ass_oid, &ws_oid).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Assignee not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn list_assignee_groups(
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
    match repo.find_assignee_groups(&ws_oid).await {
        Ok(groups) => {
            axum::Json(serde_json::json!({ "success": true, "groups": groups })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_assignee_group(
    State(state): State<SharedState>,
    Path(ws_id): Path<String>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateAssigneeGroupRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let group = AssigneeGroupDocument {
        id: None,
        workspace_id: ws_oid,
        name: payload.name,
        assignee_ids: payload.assignee_ids,
        created_at: None,
    };

    let repo = DataRepository::new(&state.db);
    match repo.create_assignee_group(group).await {
        Ok(created) => {
            axum::Json(serde_json::json!({ "success": true, "group": created })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_assignee_group(
    State(state): State<SharedState>,
    Path((ws_id, group_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateAssigneeGroupRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let gr_oid = match ObjectId::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid group ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    match repo
        .update_assignee_group(
            &gr_oid,
            &ws_oid,
            payload.name.as_deref(),
            payload.assignee_ids,
        )
        .await
    {
        Ok(Some(group)) => {
            axum::Json(serde_json::json!({ "success": true, "group": group })).into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Group not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_assignee_group(
    State(state): State<SharedState>,
    Path((ws_id, group_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };

    let gr_oid = match ObjectId::parse_str(&group_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid group ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    match repo.delete_assignee_group(&gr_oid, &ws_oid).await {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Group not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn get_assignee_stats(
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
    let assignees = match repo.find_assignees(&ws_oid).await {
        Ok(rows) => rows,
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("{}", e) })),
            )
                .into_response()
        }
    };

    let assignee_ids: Vec<String> = assignees
        .iter()
        .filter_map(|a| a.id.map(|id| id.to_hex()))
        .collect();

    match repo
        .count_tasks_by_assignee_ids(&ws_oid, &assignee_ids)
        .await
    {
        Ok(counts) => {
            let stats: Vec<serde_json::Value> = assignee_ids
                .into_iter()
                .map(|id| {
                    serde_json::json!({
                        "id": id,
                        "taskCount": counts.get(&id).copied().unwrap_or(0)
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
