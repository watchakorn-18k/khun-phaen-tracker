use crate::models::ai::AiConfigDocument;
use mongodb::{
    bson::doc,
    options::ReplaceOptions,
    Collection, Database,
};

const AI_SETTINGS_KEY: &str = "ai_config";

#[derive(Clone)]
pub struct AiRepository {
    collection: Collection<AiConfigDocument>,
}

impl AiRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("app_settings"),
        }
    }

    pub async fn get_ai_config(
        &self,
    ) -> mongodb::error::Result<Option<AiConfigDocument>> {
        self.collection
            .find_one(doc! { "key": AI_SETTINGS_KEY }, None)
            .await
    }

    pub async fn save_ai_config(
        &self,
        config: &AiConfigDocument,
    ) -> mongodb::error::Result<()> {
        self.collection
            .replace_one(
                doc! { "key": AI_SETTINGS_KEY },
                config,
                ReplaceOptions::builder().upsert(true).build(),
            )
            .await?;
        Ok(())
    }

    pub async fn delete_ai_config(&self) -> mongodb::error::Result<()> {
        self.collection
            .delete_one(doc! { "key": AI_SETTINGS_KEY }, None)
            .await?;
        Ok(())
    }
}
