use crate::models::test_case::TestSuite;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    Collection, Database,
};

#[derive(Clone)]
pub struct TestSuiteRepository {
    collection: Collection<TestSuite>,
}

impl TestSuiteRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("test_suites"),
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

    pub async fn create(&self, suite: TestSuite) -> mongodb::error::Result<TestSuite> {
        self.collection.insert_one(suite.clone(), None).await?;
        Ok(suite)
    }

    pub async fn find_by_workspace(&self, workspace_id: &ObjectId) -> mongodb::error::Result<Vec<TestSuite>> {
        let mut cursor = self.collection.find(doc! { "workspace_id": workspace_id }, None).await?;
        let mut suites = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => suites.push(doc),
                Err(e) => return Err(e),
            }
        }
        Ok(suites)
    }

    pub async fn find_by_id(&self, id: &str) -> mongodb::error::Result<Option<TestSuite>> {
        self.collection.find_one(doc! { "_id": Self::build_id_filter(id) }, None).await
    }
}
