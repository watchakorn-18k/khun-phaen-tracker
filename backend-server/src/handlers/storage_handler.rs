use crate::handlers::auth_handler::extract_claims;
use crate::models::storage::{StorageConfigDocument, StorageProvider, UpdateStorageConfigRequest};
use crate::repositories::storage_repo::StorageRepository;
use crate::services::storage_service::{
    build_active_storage, decrypt_storage_config, default_storage_config_doc,
    encrypt_value_if_present,
};
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
}

#[derive(Deserialize)]
pub struct StorageObjectsQuery {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize)]
pub struct BulkDeleteStorageObjectsRequest {
    pub keys: Vec<String>,
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

pub async fn get_storage_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let repo = StorageRepository::new(&state.db);
    let stored = match repo.get_storage_config().await {
        Ok(config) => config
            .map(|cfg| decrypt_storage_config(&cfg))
            .unwrap_or_else(default_storage_config_doc),
        Err(error) => {
            tracing::error!("Failed to load storage config: {:?}", error);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "Failed to load storage config" })),
            )
                .into_response();
        }
    };

    let active = state.storage_snapshot().await;

    Json(serde_json::json!({
        "success": true,
        "config": {
            "provider": stored.provider,
            "bucket": stored.bucket.or(Some(active.bucket)),
            "region": stored.region.or(Some(active.region)),
            "endpoint": match stored.provider {
                StorageProvider::Env => active.endpoint.clone(),
                StorageProvider::S3 => stored.endpoint.or(active.endpoint.clone()),
            },
            "access_key": stored.access_key.or(active.access_key),
            "secret_key": stored.secret_key.or(active.secret_key),
            "updated_at": stored.updated_at
        }
    }))
    .into_response()
}

pub async fn update_storage_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Json(payload): Json<UpdateStorageConfigRequest>,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let bucket = normalize_optional(payload.bucket);
    let region = normalize_optional(payload.region);
    let endpoint = normalize_optional(payload.endpoint);
    let access_key = normalize_optional(payload.access_key);
    let secret_key = normalize_optional(payload.secret_key);

    if bucket.is_none() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Bucket name is required" })),
        )
            .into_response();
    }

    if region.is_none() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Region is required" })),
        )
            .into_response();
    }

    if payload.provider == StorageProvider::Env && endpoint.is_none() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Storage endpoint is required" })),
        )
            .into_response();
    }

    if (payload.provider == StorageProvider::S3 || payload.provider == StorageProvider::Env)
        && (access_key.is_none() || secret_key.is_none())
    {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "Access key and secret key are required" })),
        )
            .into_response();
    }

    let config = StorageConfigDocument {
        key: "storage_config".to_string(),
        provider: payload.provider,
        bucket,
        region,
        endpoint,
        access_key: encrypt_value_if_present(access_key),
        secret_key: encrypt_value_if_present(secret_key),
        updated_at: Some(chrono::Utc::now().to_rfc3339()),
    };

    let next_storage = build_active_storage(Some(&config)).await;
    let repo = StorageRepository::new(&state.db);
    if let Err(error) = repo.save_storage_config(&config).await {
        tracing::error!("Failed to save storage config: {:?}", error);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to save storage config" })),
        )
            .into_response();
    }

    state.replace_storage(next_storage.clone()).await;

    Json(serde_json::json!({
        "success": true,
        "config": {
            "provider": next_storage.provider,
            "bucket": next_storage.bucket,
            "region": next_storage.region,
            "endpoint": next_storage.endpoint,
            "updated_at": config.updated_at
        }
    }))
    .into_response()
}

pub async fn reset_storage_config_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let repo = StorageRepository::new(&state.db);
    if let Err(error) = repo.delete_storage_config().await {
        tracing::error!("Failed to reset storage config: {:?}", error);
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": "Failed to reset storage config" })),
        )
            .into_response();
    }

    let next_storage = build_active_storage(None).await;
    state.replace_storage(next_storage.clone()).await;

    Json(serde_json::json!({
        "success": true,
        "config": {
            "provider": next_storage.provider,
            "bucket": next_storage.bucket,
            "region": next_storage.region,
            "endpoint": next_storage.endpoint
        }
    }))
    .into_response()
}

pub async fn get_storage_stats_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let storage = state.storage_snapshot().await;
    let client = match &storage.client {
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
        let mut request = client.list_objects_v2().bucket(&storage.bucket);
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

    let quota_bytes = storage.quota_bytes;
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
        "provider": storage.provider,
        "bucket": storage.bucket,
        "region": storage.region,
        "endpoint": storage.endpoint,
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

    let storage = state.storage_snapshot().await;
    let client = match &storage.client {
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
        let mut request = client.list_objects_v2().bucket(&storage.bucket);
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
        "bucket": storage.bucket,
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

    let storage = state.storage_snapshot().await;
    let client = match &storage.client {
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
        .bucket(&storage.bucket)
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

pub async fn bulk_delete_storage_objects_handler(
    State(state): State<SharedState>,
    jar: CookieJar,
    headers: axum::http::HeaderMap,
    Json(payload): Json<BulkDeleteStorageObjectsRequest>,
) -> axum::response::Response {
    if let Err(response) = ensure_admin(&headers, &jar, &state) {
        return response;
    }

    let storage = state.storage_snapshot().await;
    let client = match &storage.client {
        Some(client) => client,
        None => {
            return (
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                Json(serde_json::json!({ "error": "Storage is disabled" })),
            )
                .into_response()
        }
    };

    let keys: Vec<String> = payload
        .keys
        .into_iter()
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty())
        .collect();

    if keys.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "No storage objects selected" })),
        )
            .into_response();
    }

    let mut deleted = 0usize;
    let mut failed: Vec<String> = Vec::new();

    for key in &keys {
        match client
            .delete_object()
            .bucket(&storage.bucket)
            .key(key)
            .send()
            .await
        {
            Ok(_) => deleted += 1,
            Err(error) => {
                tracing::error!("Failed to delete storage object {}: {:?}", key, error);
                failed.push(key.clone());
            }
        }
    }

    if !failed.is_empty() {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "Failed to delete one or more storage objects",
                "deleted": deleted,
                "failed": failed
            })),
        )
            .into_response();
    }

    Json(serde_json::json!({
        "success": true,
        "deleted": deleted
    }))
    .into_response()
}
