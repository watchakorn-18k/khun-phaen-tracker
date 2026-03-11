use crate::models::storage::StorageConfigDocument;
use mongodb::{
    bson::doc,
    options::{IndexOptions, ReplaceOptions},
    Collection, Database, IndexModel,
};

const STORAGE_SETTINGS_KEY: &str = "storage_config";

#[derive(Clone)]
pub struct StorageRepository {
    collection: Collection<StorageConfigDocument>,
}

impl StorageRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("app_settings"),
        }
    }

    pub async fn ensure_indexes(&self) -> mongodb::error::Result<()> {
        let unique_key = IndexModel::builder()
            .keys(doc! { "key": 1 })
            .options(
                IndexOptions::builder()
                    .unique(true)
                    .name(Some("idx_app_settings_key".to_string()))
                    .build(),
            )
            .build();
        self.collection.create_index(unique_key, None).await?;
        Ok(())
    }

    pub async fn get_storage_config(
        &self,
    ) -> mongodb::error::Result<Option<StorageConfigDocument>> {
        self.collection
            .find_one(doc! { "key": STORAGE_SETTINGS_KEY }, None)
            .await
    }

    pub async fn save_storage_config(
        &self,
        config: &StorageConfigDocument,
    ) -> mongodb::error::Result<()> {
        self.collection
            .replace_one(
                doc! { "key": STORAGE_SETTINGS_KEY },
                config,
                ReplaceOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_storage_config(&self) -> mongodb::error::Result<()> {
        self.collection
            .delete_one(doc! { "key": STORAGE_SETTINGS_KEY }, None)
            .await?;
        Ok(())
    }
}
