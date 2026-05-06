use crate::models::notification::{CreateNotificationRequest, UserNotification};
use futures::TryStreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::{FindOptions, IndexOptions, UpdateOptions},
    Collection, Database, IndexModel,
};

#[derive(Clone)]
pub struct NotificationRepository {
    collection: Collection<UserNotification>,
}

impl NotificationRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("notifications"),
        }
    }

    pub async fn ensure_indexes(&self) -> mongodb::error::Result<()> {
        let by_recipient_created = IndexModel::builder()
            .keys(doc! { "recipient_user_id": 1, "created_at": -1 })
            .options(
                IndexOptions::builder()
                    .name(Some("idx_notifications_recipient_created".to_string()))
                    .build(),
            )
            .build();

        let unique_event_recipient = IndexModel::builder()
            .keys(doc! {
                "recipient_user_id": 1,
                "source_type": 1,
                "source_id": 1,
                "created_at": 1,
            })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some("idx_notifications_event_recipient_unique".to_string()))
                    .build(),
            )
            .build();

        let unique_source_recipient = IndexModel::builder()
            .keys(doc! {
                "recipient_user_id": 1,
                "source_type": 1,
                "source_id": 1,
            })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some("idx_notifications_source_recipient_unique".to_string()))
                    .build(),
            )
            .build();

        self.collection
            .create_indexes(
                vec![
                    by_recipient_created,
                    unique_event_recipient,
                    unique_source_recipient,
                ],
                None,
            )
            .await?;
        Ok(())
    }

    pub async fn create_for_recipients(
        &self,
        request: CreateNotificationRequest,
    ) -> mongodb::error::Result<Vec<UserNotification>> {
        let mut inserted = Vec::new();
        let options = UpdateOptions::builder().upsert(true).build();

        for recipient_user_id in unique_non_empty(request.recipient_user_ids.clone()) {
            let notification = UserNotification {
                id: Some(ObjectId::new()),
                recipient_user_id: recipient_user_id.clone(),
                source_type: request.source_type.clone(),
                source_id: request.source_id.clone(),
                source_title: request.source_title.clone(),
                workspace_id: request.workspace_id.clone(),
                suite_id: request.suite_id.clone(),
                assignment_ids: request.assignment_ids.clone(),
                recipient_user_ids: request.recipient_user_ids.clone(),
                actor_user_id: request.actor_user_id.clone(),
                actor_name: request.actor_name.clone(),
                created_at: request.created_at.clone(),
                read_at: None,
            };

            let update = doc! {
                "$setOnInsert": mongodb::bson::to_document(&notification)
                    .map_err(|error| mongodb::error::Error::custom(format!("{}", error)))?,
            };

            let result = self
                .collection
                .update_one(
                    doc! {
                        "recipient_user_id": &recipient_user_id,
                        "source_type": &request.source_type,
                        "source_id": &request.source_id,
                    },
                    update,
                    options.clone(),
                )
                .await?;

            if result.upserted_id.is_some() {
                inserted.push(notification);
            }
        }

        Ok(inserted)
    }

    pub async fn list_for_user(
        &self,
        user_keys: &[String],
        limit: i64,
    ) -> mongodb::error::Result<Vec<UserNotification>> {
        if user_keys.is_empty() {
            return Ok(Vec::new());
        }

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1, "_id": -1 })
            .limit(limit)
            .build();

        let mut cursor = self
            .collection
            .find(doc! { "recipient_user_id": { "$in": user_keys } }, options)
            .await?;

        let mut notifications = Vec::new();
        while let Some(notification) = cursor.try_next().await? {
            notifications.push(notification);
        }
        Ok(notifications)
    }

    pub async fn mark_read(
        &self,
        notification_id: &ObjectId,
        user_keys: &[String],
        read_at: &str,
    ) -> mongodb::error::Result<bool> {
        let result = self
            .collection
            .update_one(
                doc! {
                    "_id": notification_id,
                    "recipient_user_id": { "$in": user_keys },
                    "read_at": { "$exists": false },
                },
                doc! { "$set": { "read_at": read_at } },
                None,
            )
            .await?;
        Ok(result.modified_count > 0)
    }

    pub async fn mark_all_read(
        &self,
        user_keys: &[String],
        read_at: &str,
    ) -> mongodb::error::Result<u64> {
        let result = self
            .collection
            .update_many(
                doc! {
                    "recipient_user_id": { "$in": user_keys },
                    "read_at": { "$exists": false },
                },
                doc! { "$set": { "read_at": read_at } },
                None,
            )
            .await?;
        Ok(result.modified_count)
    }
}

fn unique_non_empty(values: Vec<String>) -> Vec<String> {
    let mut cleaned: Vec<String> = values
        .into_iter()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .collect();
    cleaned.sort();
    cleaned.dedup();
    cleaned
}
