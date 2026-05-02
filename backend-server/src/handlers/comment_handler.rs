use axum::{
    extract::{Json, Multipart, Path, Query, State},
    http::HeaderMap,
    response::IntoResponse,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::Utc;
use mongodb::bson::oid::ObjectId;
use uuid::Uuid;

use crate::handlers::auth_handler::extract_user_id;
use crate::handlers::common::{verify_workspace_access, verify_task_belongs_to_workspace};
use crate::models::data::*;
use crate::repositories::data_repo::DataRepository;
use crate::repositories::profile_repo::ProfileRepository;
use crate::repositories::user_repo::UserRepository;
use crate::state::SharedState;

/// Resolve a human-readable display name for a comment author.
/// `created_by` is the hex string of the user's MongoDB ObjectId.
/// Priority: nickname → first+last name → email prefix.
async fn resolve_author_name(db: &mongodb::Database, created_by: &str) -> Option<String> {
    let oid = mongodb::bson::oid::ObjectId::parse_str(created_by).ok()?;
    let user_repo = UserRepository::new(db);
    let user = user_repo.find_by_id(&oid).await.ok()??;
    let profile_repo = ProfileRepository::new(db);
    if let Ok(Some(profile)) = profile_repo.find_by_user_id(&user.user_id).await {
        if let Some(nick) = profile.nickname.filter(|s| !s.is_empty()) {
            return Some(nick);
        }
        let full = [profile.first_name, profile.last_name]
            .into_iter()
            .flatten()
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !full.is_empty() {
            return Some(full);
        }
    }
    Some(user.email.split('@').next().unwrap_or(&user.email).to_string())
}

const MAX_COMMENT_IMAGES: usize = 10;
const MAX_COMMENT_IMAGE_SIZE_BYTES: usize = 10 * 1024 * 1024;
const ALLOWED_COMMENT_REACTION_EMOJIS: [&str; 10] =
    ["👍", "❤️", "🔥", "🎉", "😂", "😮", "😢", "👀", "✅", "🚀"];

pub async fn list_task_comments(
    State(state): State<SharedState>,
    Path((ws_id, task_id)): Path<(String, String)>,
    Query(query): Query<CommentPaginationQuery>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }
    let _ = repo.ensure_comment_indexes().await;
    let limit = query.limit.unwrap_or(10).max(1);
    let page = query.page.unwrap_or(1).max(1);

    match repo
        .find_comments_by_task_paginated(&ws_oid, &task_oid, page, limit)
        .await
    {
        Ok((raw_comments, total)) => {
            let pages = (total as f64 / limit as f64).ceil() as u64;
            let mut comments = Vec::with_capacity(raw_comments.len());
            for c in raw_comments {
                let name = resolve_author_name(&state.db, &c.created_by).await;
                comments.push(CommentWithName::from_doc(c, name));
            }
            axum::Json(PaginatedCommentResponse {
                success: true,
                comments,
                total,
                page,
                limit,
                pages,
            })
            .into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn create_task_comment(
    State(state): State<SharedState>,
    Path((ws_id, task_id)): Path<(String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    mut multipart: Multipart,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let user_id = match extract_user_id(&headers, &jar, &state.jwt_secret) {
        Some(id) => id.to_hex(),
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "Not logged in" })),
            )
                .into_response()
        }
    };
    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }
    let _ = repo.ensure_comment_indexes().await;

    let mut content = String::new();
    let mut images = Vec::<CommentImage>::new();
    while let Some(field) = match multipart.next_field().await {
        Ok(v) => v,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid multipart payload" })),
            )
                .into_response()
        }
    } {
        let name = field.name().unwrap_or_default().to_string();
        if name == "content" {
            if let Ok(value) = field.text().await {
                content = value;
            }
            continue;
        }

        if name != "images" && name != "images[]" {
            continue;
        }
        if images.len() >= MAX_COMMENT_IMAGES {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": format!("Maximum {} images per comment", MAX_COMMENT_IMAGES) })),
            )
                .into_response();
        }

        let mime_type = field.content_type().unwrap_or("").to_string();
        let filename = field.file_name().unwrap_or("comment-image").to_string();
        if !mime_type.starts_with("image/") {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Only image files are allowed" })),
            )
                .into_response();
        }
        let file_bytes = match field.bytes().await {
            Ok(bytes) => bytes,
            Err(_) => {
                return (
                    axum::http::StatusCode::BAD_REQUEST,
                    axum::Json(serde_json::json!({ "error": "Failed to read image file" })),
                )
                    .into_response()
            }
        };
        if file_bytes.is_empty() {
            continue;
        }
        if file_bytes.len() > MAX_COMMENT_IMAGE_SIZE_BYTES {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Image exceeds 10MB limit" })),
            )
                .into_response();
        }
        let storage = state.storage_snapshot().await;
        let client = match &storage.client {
            Some(c) => c,
            None => {
                return (
                    axum::http::StatusCode::SERVICE_UNAVAILABLE,
                    axum::Json(serde_json::json!({ "error": "Storage is disabled" })),
                )
                    .into_response()
            }
        };
        let image_id = Uuid::now_v7().to_string();
        let file_key = format!("{}/{}/comments/{}", ws_id, task_id, image_id);
        if let Err(e) = client
            .put_object()
            .bucket(&storage.bucket)
            .key(&file_key)
            .content_type(&mime_type)
            .body(aws_sdk_s3::primitives::ByteStream::from(file_bytes.clone()))
            .send()
            .await
        {
            tracing::error!("Failed to upload comment image: {:?}", e);
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": "Storage error" })),
            )
                .into_response();
        }
        images.push(CommentImage {
            id: image_id,
            filename,
            file_key,
            mime_type,
            size: file_bytes.len() as i64,
            uploaded_at: Utc::now().to_rfc3339(),
            uploader_id: user_id.clone(),
        });
    }

    if content.trim().is_empty() && images.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(
                serde_json::json!({ "error": "Comment must include text or at least one image" }),
            ),
        )
            .into_response();
    }

    let comment = CommentDocument {
        id: None,
        workspace_id: ws_oid,
        task_id: task_oid,
        content: content.trim().to_string(),
        images,
        reactions: vec![],
        created_by: user_id,
        created_at: None,
        updated_at: None,
    };
    match repo.create_comment(comment).await {
        Ok(created) => {
            let name = resolve_author_name(&state.db, &created.created_by).await;
            let with_name = CommentWithName::from_doc(created, name);
            axum::Json(serde_json::json!({ "success": true, "comment": with_name })).into_response()
        }
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn list_comment_images(
    State(state): State<SharedState>,
    Path((ws_id, task_id, comment_id)): Path<(String, String, String)>,
    Query(query): Query<CommentPaginationQuery>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let comment_oid = match ObjectId::parse_str(&comment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid comment ID" })),
            )
                .into_response()
        }
    };
    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }
    let limit = query.limit.unwrap_or(6).max(1);
    let page = query.page.unwrap_or(1).max(1);
    match repo
        .find_comment_images_paginated(&ws_oid, &task_oid, &comment_oid, page, limit)
        .await
    {
        Ok(Some((images, total))) => {
            let pages = (total as f64 / limit as f64).ceil() as u64;
            axum::Json(PaginatedCommentImageResponse {
                success: true,
                images,
                total,
                page,
                limit,
                pages,
            })
            .into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn delete_task_comment(
    State(state): State<SharedState>,
    Path((ws_id, task_id, comment_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let comment_oid = match ObjectId::parse_str(&comment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid comment ID" })),
            )
                .into_response()
        }
    };

    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }
    let comment = match repo
        .find_comment_by_id(&ws_oid, &task_oid, &comment_oid)
        .await
    {
        Ok(Some(c)) => c,
        Ok(None) => {
            return (
                axum::http::StatusCode::NOT_FOUND,
                axum::Json(serde_json::json!({ "error": "Comment not found" })),
            )
                .into_response()
        }
        Err(e) => {
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("{}", e) })),
            )
                .into_response()
        }
    };

    let storage = state.storage_snapshot().await;
    if let Some(client) = &storage.client {
        for image in &comment.images {
            if let Err(e) = client
                .delete_object()
                .bucket(&storage.bucket)
                .key(&image.file_key)
                .send()
                .await
            {
                tracing::warn!("Failed to delete comment image: {:?}", e);
            }
        }
    }

    match repo.delete_comment(&ws_oid, &task_oid, &comment_oid).await {
        Ok(Some(_)) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn update_task_comment(
    State(state): State<SharedState>,
    Path((ws_id, task_id, comment_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<UpdateCommentRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let comment_oid = match ObjectId::parse_str(&comment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid comment ID" })),
            )
                .into_response()
        }
    };
    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }
    match repo
        .update_comment_content(
            &ws_oid,
            &task_oid,
            &comment_oid,
            payload.content.trim().to_string(),
        )
        .await
    {
        Ok(true) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Ok(false) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}

pub async fn toggle_task_comment_reaction(
    State(state): State<SharedState>,
    Path((ws_id, task_id, comment_id)): Path<(String, String, String)>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<ToggleCommentReactionRequest>,
) -> axum::response::Response {
    let ws_oid = match verify_workspace_access(&state, &headers, &jar, &ws_id).await {
        Ok(id) => id,
        Err(resp) => return resp,
    };
    let task_oid = match ObjectId::parse_str(&task_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid task ID" })),
            )
                .into_response()
        }
    };
    let comment_oid = match ObjectId::parse_str(&comment_id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid comment ID" })),
            )
                .into_response()
        }
    };
    let user_id = match extract_user_id(&headers, &jar, &state.jwt_secret) {
        Some(id) => id.to_hex(),
        None => {
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "Not logged in" })),
            )
                .into_response()
        }
    };

    let emoji = payload.emoji.trim();
    if !ALLOWED_COMMENT_REACTION_EMOJIS.contains(&emoji) {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            axum::Json(serde_json::json!({ "error": "Unsupported emoji" })),
        )
            .into_response();
    }

    let repo = DataRepository::new(&state.db);
    if let Err(resp) = verify_task_belongs_to_workspace(&repo, &ws_oid, &task_oid).await {
        return resp;
    }

    match repo
        .toggle_comment_reaction(&ws_oid, &task_oid, &comment_oid, &user_id, emoji)
        .await
    {
        Ok(Some(comment)) => {
            axum::Json(serde_json::json!({ "success": true, "comment": comment })).into_response()
        }
        Ok(None) => (
            axum::http::StatusCode::NOT_FOUND,
            axum::Json(serde_json::json!({ "error": "Comment not found" })),
        )
            .into_response(),
        Err(e) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", e) })),
        )
            .into_response(),
    }
}
