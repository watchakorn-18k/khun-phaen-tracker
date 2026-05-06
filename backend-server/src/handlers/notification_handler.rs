use axum::{
    extract::{Json, Path, State},
    http::HeaderMap,
    response::{
        sse::{Event, KeepAlive, Sse},
        IntoResponse,
    },
};
use axum_extra::extract::cookie::CookieJar;
use futures::stream;
use mongodb::bson::oid::ObjectId;
use std::{convert::Infallible, time::Duration};
use tokio::sync::broadcast;

use crate::handlers::{
    auth_handler::extract_user_id,
    common::verify_workspace_access,
};
use crate::models::notification::CreateNotificationRequest;
use crate::repositories::{
    notification_repo::NotificationRepository,
    user_repo::UserRepository,
};
use crate::state::SharedState;

pub async fn list_notifications(
    State(state): State<SharedState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let user_keys = match current_user_keys(&state, &headers, &jar).await {
        Ok(keys) => keys,
        Err(response) => return response,
    };

    let repo = NotificationRepository::new(&state.db);
    match repo.list_for_user(&user_keys, 100).await {
        Ok(notifications) => axum::Json(serde_json::json!({
            "success": true,
            "notifications": notifications,
        }))
        .into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", error) })),
        )
            .into_response(),
    }
}

pub async fn create_notification(
    State(state): State<SharedState>,
    headers: HeaderMap,
    jar: CookieJar,
    Json(payload): Json<CreateNotificationRequest>,
) -> axum::response::Response {
    if let Err(response) =
        verify_workspace_access(&state, &headers, &jar, &payload.workspace_id).await
    {
        return response;
    }

    if payload.recipient_user_ids.is_empty() {
        return axum::Json(serde_json::json!({ "success": true, "inserted": 0 })).into_response();
    }

    let repo = NotificationRepository::new(&state.db);
    match repo.create_for_recipients(payload).await {
        Ok(notifications) => {
            for notification in &notifications {
                let _ = state.notification_tx.send(notification.clone());
            }

            axum::Json(serde_json::json!({
            "success": true,
            "inserted": notifications.len(),
        }))
            .into_response()
        }
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", error) })),
        )
            .into_response(),
    }
}

pub async fn stream_notifications(
    State(state): State<SharedState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let user_keys = match current_user_keys(&state, &headers, &jar).await {
        Ok(keys) => keys,
        Err(response) => return response,
    };

    let rx = state.notification_tx.subscribe();
    let events = stream::unfold((rx, user_keys), |(mut rx, user_keys)| async move {
        loop {
            match rx.recv().await {
                Ok(notification) => {
                    if user_keys.contains(&notification.recipient_user_id) {
                        let data = match serde_json::to_string(&notification) {
                            Ok(data) => data,
                            Err(_) => continue,
                        };
                        let event = Event::default().event("notification").data(data);
                        return Some((Ok::<Event, Infallible>(event), (rx, user_keys)));
                    }
                }
                Err(broadcast::error::RecvError::Lagged(_)) => continue,
                Err(broadcast::error::RecvError::Closed) => return None,
            }
        }
    });

    Sse::new(events)
        .keep_alive(
            KeepAlive::new()
                .interval(Duration::from_secs(30))
                .text("keep-alive"),
        )
        .into_response()
}

pub async fn mark_notification_read(
    State(state): State<SharedState>,
    headers: HeaderMap,
    jar: CookieJar,
    Path(id): Path<String>,
) -> axum::response::Response {
    let user_keys = match current_user_keys(&state, &headers, &jar).await {
        Ok(keys) => keys,
        Err(response) => return response,
    };
    let notification_id = match ObjectId::parse_str(&id) {
        Ok(id) => id,
        Err(_) => {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                axum::Json(serde_json::json!({ "error": "Invalid notification ID" })),
            )
                .into_response()
        }
    };

    let repo = NotificationRepository::new(&state.db);
    let read_at = chrono::Utc::now().to_rfc3339();
    match repo.mark_read(&notification_id, &user_keys, &read_at).await {
        Ok(_) => axum::Json(serde_json::json!({ "success": true })).into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", error) })),
        )
            .into_response(),
    }
}

pub async fn mark_all_notifications_read(
    State(state): State<SharedState>,
    headers: HeaderMap,
    jar: CookieJar,
) -> axum::response::Response {
    let user_keys = match current_user_keys(&state, &headers, &jar).await {
        Ok(keys) => keys,
        Err(response) => return response,
    };

    let repo = NotificationRepository::new(&state.db);
    let read_at = chrono::Utc::now().to_rfc3339();
    match repo.mark_all_read(&user_keys, &read_at).await {
        Ok(updated) => axum::Json(serde_json::json!({
            "success": true,
            "updated": updated,
        }))
        .into_response(),
        Err(error) => (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            axum::Json(serde_json::json!({ "error": format!("{}", error) })),
        )
            .into_response(),
    }
}

async fn current_user_keys(
    state: &SharedState,
    headers: &HeaderMap,
    jar: &CookieJar,
) -> Result<Vec<String>, axum::response::Response> {
    let user_object_id = extract_user_id(headers, jar, &state.jwt_secret).ok_or_else(|| {
        (
            axum::http::StatusCode::UNAUTHORIZED,
            axum::Json(serde_json::json!({ "error": "Unauthorized" })),
        )
            .into_response()
    })?;

    let user_repo = UserRepository::new(&state.db);
    let user = user_repo
        .find_by_id(&user_object_id)
        .await
        .map_err(|error| {
            (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                axum::Json(serde_json::json!({ "error": format!("{}", error) })),
            )
                .into_response()
        })?
        .ok_or_else(|| {
            (
                axum::http::StatusCode::UNAUTHORIZED,
                axum::Json(serde_json::json!({ "error": "User not found" })),
            )
                .into_response()
        })?;

    let mut keys = vec![user_object_id.to_hex(), user.user_id];
    keys.sort();
    keys.dedup();
    Ok(keys)
}
