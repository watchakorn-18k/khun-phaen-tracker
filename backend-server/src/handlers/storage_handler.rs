use crate::handlers::auth_handler::extract_claims;
use crate::state::SharedState;
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize)]
struct StorageObjectSummary {
    key: String,
    size_bytes: u64,
    last_modified: Option<String>,
    etag: Option<String>,
    mime_type: Option<String>,
}

#[derive(Deserialize)]
pub struct StorageObjectsQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

fn ensure_admin(
    headers: &axum::http::HeaderMap,
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

pub async fn get_storage_stats_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let client = match &state.storage_client {
        Some(client) => client,
        None => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Storage is disabled" })),
            )
                .into_response()
        }
    };

    let mut continuation_token: Option<String> = None;
    let mut used_bytes: u64 = 0;
    let mut object_count: u64 = 0;

    loop {
        let mut request = client.list_objects_v2().bucket(&state.storage_bucket);
        if let Some(token) = continuation_token.as_deref() {
            request = request.continuation_token(token);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                tracing::error!("Failed to list storage objects: {:?}", error);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to inspect storage bucket" })),
                )
                    .into_response();
            }
        };

        for object in response.contents() {
            if let Some(size) = object.size() {
                if size > 0 {
                    used_bytes = used_bytes.saturating_add(size as u64);
                }
            }
            object_count = object_count.saturating_add(1);
        }

        if response.is_truncated().unwrap_or(false) {
            continuation_token = response
                .next_continuation_token()
                .map(|token| token.to_string());
        } else {
            break;
        }
    }

    let quota_bytes = state.storage_quota_bytes;
    let remaining_bytes = quota_bytes.map(|quota| quota.saturating_sub(used_bytes));
    let usage_percent = quota_bytes.map(|quota| {
        if quota == 0 {
            0
        } else {
            ((used_bytes as f64 / quota as f64) * 100.0).round() as u64
        }
    });

    Json(serde_json::json!({
        "success": true,
        "bucket": state.storage_bucket,
        "endpoint": state.storage_endpoint,
        "used_bytes": used_bytes,
        "quota_bytes": quota_bytes,
        "remaining_bytes": remaining_bytes,
        "usage_percent": usage_percent,
        "object_count": object_count
    }))
    .into_response()
}

pub async fn list_storage_objects_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Query(query): Query<StorageObjectsQuery>,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let client = match &state.storage_client {
        Some(client) => client,
        None => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Storage is disabled" })),
            )
                .into_response()
        }
    };

    let mut continuation_token: Option<String> = None;
    let mut objects: Vec<StorageObjectSummary> = Vec::new();

    loop {
        let mut request = client.list_objects_v2().bucket(&state.storage_bucket);
        if let Some(token) = continuation_token.as_deref() {
            request = request.continuation_token(token);
        }

        let response = match request.send().await {
            Ok(response) => response,
            Err(error) => {
                tracing::error!("Failed to list storage objects: {:?}", error);
                return (
                    axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({ "error": "Failed to list storage objects" })),
                )
                    .into_response();
            }
        };

        for object in response.contents() {
            let key = match object.key() {
                Some(key) => key.to_string(),
                None => continue,
            };

            objects.push(StorageObjectSummary {
                key,
                size_bytes: object.size().unwrap_or(0).max(0) as u64,
                last_modified: object.last_modified().map(|value| value.to_string()),
                etag: object.e_tag().map(|value| value.to_string()),
                mime_type: object
                    .key()
                    .and_then(|key| mime_guess::from_path(key).first_raw())
                    .map(|value| value.to_string()),
            });
        }

        if response.is_truncated().unwrap_or(false) {
            continuation_token = response
                .next_continuation_token()
                .map(|token| token.to_string());
        } else {
            break;
        }
    }

    objects.sort_by(|a, b| b.last_modified.cmp(&a.last_modified));

    let total = objects.len();
    let limit = query.limit.unwrap_or(10).clamp(1, 100);
    let pages = if total == 0 { 1 } else { total.div_ceil(limit) };
    let page = query.page.unwrap_or(1).clamp(1, pages);
    let start = (page - 1) * limit;
    let end = (start + limit).min(total);
    let paged_objects = if total == 0 {
        Vec::new()
    } else {
        objects[start..end].to_vec()
    };

    Json(serde_json::json!({
        "success": true,
        "bucket": state.storage_bucket,
        "objects": paged_objects,
        "total": total,
        "page": page,
        "limit": limit,
        "pages": pages
    }))
    .into_response()
}

pub async fn delete_storage_object_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Path(key): Path<String>,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let client = match &state.storage_client {
        Some(client) => client,
        None => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Storage is disabled" })),
            )
                .into_response()
        }
    };

    match client
        .delete_object()
        .bucket(&state.storage_bucket)
        .key(&key)
        .send()
        .await
    {
        Ok(_) => Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => {
            tracing::error!("Failed to delete storage object {}: {:?}", key, error);
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to delete storage object" })),
            )
                .into_response()
        }
    }
}
