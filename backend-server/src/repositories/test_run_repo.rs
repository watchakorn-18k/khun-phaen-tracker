use crate::models::test_run::TestRun;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::FindOptions,
    Collection, Database,
};

#[derive(Clone)]
pub struct TestRunRepository {
    collection: Collection<TestRun>,
}

impl TestRunRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("test_runs"),
        }
    }

    fn build_id_filter(id: &str) -> mongodb::bson::Document {
        let mut in_array = vec![mongodb::bson::Bson::String(id.to_string())];

        if let Ok(oid) = mongodb::bson::oid::ObjectId::parse_str(id) {
            in_array.push(mongodb::bson::Bson::ObjectId(oid));
        }

        if let Ok(u) = uuid::Uuid::parse_str(id) {
            in_array.push(mongodb::bson::Bson::Binary(mongodb::bson::Binary {
                subtype: mongodb::bson::spec::BinarySubtype::Generic,
                bytes: u.into_bytes().to_vec(),
            }));
            in_array.push(mongodb::bson::Bson::Binary(mongodb::bson::Binary {
                subtype: mongodb::bson::spec::BinarySubtype::Uuid,
                bytes: u.into_bytes().to_vec(),
            }));
        }

        if in_array.len() > 1 {
            doc! { "$in": in_array }
        } else {
            doc! { "$eq": id }
        }
    }

    pub async fn find_by_workspace(
        &self,
        workspace_id: &ObjectId,
        page: u64,
        limit: u64,
    ) -> mongodb::error::Result<(Vec<TestRun>, u64)> {
        let filter = doc! { "workspace_id": workspace_id };
        let total = self.collection.count_documents(filter.clone(), None).await?;

        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .skip((page - 1) * limit)
            .limit(limit as i64)
            .build();

        let mut cursor = self.collection.find(filter, options).await?;

        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => results.push(doc),
                Err(e) => return Err(e),
            }
        }
        Ok((results, total))
    }

    pub async fn find_latest_by_workspace(
        &self,
        workspace_id: &ObjectId,
    ) -> mongodb::error::Result<Option<TestRun>> {
        let options = FindOptions::builder()
            .sort(doc! { "created_at": -1 })
            .limit(1)
            .build();

        let mut cursor = self
            .collection
            .find(doc! { "workspace_id": workspace_id }, options)
            .await?;

        cursor.next().await.transpose()
    }

    pub async fn find_by_id(&self, id: &str) -> mongodb::error::Result<Option<TestRun>> {
        self.collection
            .find_one(doc! { "_id": Self::build_id_filter(id) }, None)
            .await
    }

    pub async fn create(&self, run: TestRun) -> mongodb::error::Result<TestRun> {
        self.collection.insert_one(run.clone(), None).await?;
        Ok(run)
    }

    pub async fn update_status(
        &self,
        id: &str,
        workspace_id: &ObjectId,
        status: &str,
        updated_at: &str,
    ) -> mongodb::error::Result<bool> {
        let res = self
            .collection
            .update_one(
                doc! { "_id": Self::build_id_filter(id), "workspace_id": workspace_id },
                doc! { "$set": { "status": status, "updated_at": updated_at } },
                None,
            )
            .await?;
        Ok(res.matched_count > 0)
    }

    /// Update the status of a single test case entry within a run.
    pub async fn update_case_status(
        &self,
        run_id: &str,
        workspace_id: &ObjectId,
        test_case_id: &str,
        status: &str,
        updated_at: &str,
    ) -> mongodb::error::Result<bool> {
        let res = self
            .collection
            .update_one(
                doc! {
                    "_id": Self::build_id_filter(run_id),
                    "workspace_id": workspace_id,
                    "test_cases.test_case_id": test_case_id,
                },
                doc! {
                    "$set": {
                        "test_cases.$.status": status,
                        "updated_at": updated_at,
                    }
                },
                None,
            )
            .await?;
        Ok(res.matched_count > 0)
    }

    pub async fn delete(
        &self,
        id: &str,
        workspace_id: &ObjectId,
    ) -> mongodb::error::Result<bool> {
        let res = self
            .collection
            .delete_one(
                doc! { "_id": Self::build_id_filter(id), "workspace_id": workspace_id },
                None,
            )
            .await?;
        Ok(res.deleted_count > 0)
    }
}
