use crate::models::test_case::TestCase;
use futures::stream::StreamExt;
use mongodb::{
    bson::{doc, oid::ObjectId},
    options::FindOneOptions,
    Collection, Database,
};

#[derive(Clone)]
pub struct TestCaseRepository {
    collection: Collection<TestCase>,
}

impl TestCaseRepository {
    pub fn new(db: &Database) -> Self {
        Self {
            collection: db.collection("test_cases"),
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

    /// Get the next auto-increment test_no for a workspace
    pub async fn next_test_no(
        &self,
        workspace_id: &ObjectId,
    ) -> mongodb::error::Result<i64> {
        let options = FindOneOptions::builder()
            .sort(doc! { "test_no": -1 })
            .build();

        let last = self
            .collection
            .find_one(doc! { "workspace_id": workspace_id }, options)
            .await?;

        Ok(last.map(|tc| tc.test_no + 1).unwrap_or(1))
    }

    pub async fn create(
        &self,
        test_case: TestCase,
    ) -> mongodb::error::Result<TestCase> {
        self.collection.insert_one(test_case.clone(), None).await?;
        Ok(test_case)
    }

    pub async fn find_by_id(
        &self,
        id: &str,
    ) -> mongodb::error::Result<Option<TestCase>> {
        self.collection
            .find_one(doc! { "_id": Self::build_id_filter(id) }, None)
            .await
    }

    #[allow(dead_code)]
    pub async fn find_by_workspace(
        &self,
        workspace_id: &ObjectId,
        suite_id: Option<String>,
        search: Option<String>,
        field: Option<String>,
        priority: Option<String>,
        status: Option<String>,
        fixed: Option<String>,
        assign_dev: Option<String>,
        limit: Option<i64>,
        offset: Option<u64>,
    ) -> mongodb::error::Result<Vec<TestCase>> {
        let mut filter = doc! { "workspace_id": workspace_id };
        
        if let Some(q) = search {
            if !q.is_empty() {
                let f = field.unwrap_or_else(|| "all".to_string());
                let regex = doc! { "$regex": &q, "$options": "i" };
                
                if f == "all" {
                    let mut search_filter = vec![
                        doc! { "name": regex.clone() },
                        doc! { "description": regex.clone() },
                        doc! { "preconditions": regex.clone() },
                        doc! { "postconditions": regex.clone() },
                        doc! { "test_note": regex.clone() },
                        doc! { "dev_note": regex.clone() },
                    ];
                    
                    if let Ok(no) = q.parse::<i64>() {
                        search_filter.push(doc! { "test_no": no });
                    }
                    
                    // Search in steps text
                    search_filter.push(doc! { "classic_steps.action": regex.clone() });
                    search_filter.push(doc! { "classic_steps.expected": regex.clone() });
                    search_filter.push(doc! { "gherkin_steps.text": regex.clone() });
                    
                    filter.insert("$or", search_filter);
                } else {
                    if f == "test_no" || f == "id" {
                        if let Ok(no) = q.parse::<i64>() {
                            filter.insert("test_no", no);
                        } else {
                            // If they typed something that isn't a number for an ID, it shouldn't match anything
                            filter.insert("test_no", -1);
                        }
                    } else if f == "comments" {
                         filter.insert("$or", vec![
                             doc! { "test_note": regex.clone() },
                             doc! { "dev_note": regex.clone() },
                         ]);
                    } else if f == "steps" {
                         filter.insert("$or", vec![
                             doc! { "classic_steps.action": regex.clone() },
                             doc! { "classic_steps.expected": regex.clone() },
                             doc! { "gherkin_steps.text": regex.clone() },
                         ]);
                    } else {
                        // Title corresponds to "name" in DB
                        let db_field = if f == "title" { "name" } else { &f };
                        filter.insert(db_field, regex);
                    }
                }
            }
        }
        
        if let Some(p) = priority {
            if !p.is_empty() {
                let priorities: Vec<&str> = p.split(',').collect();
                filter.insert("priority", doc! { "$in": priorities });
            }
        }

        if let Some(s) = status {
            if !s.is_empty() {
                let statuses: Vec<&str> = s.split(',').collect();
                filter.insert("status", doc! { "$in": statuses });
            }
        }

        if let Some(f) = fixed {
            if !f.is_empty() {
                let fixed_vals: Vec<&str> = f.split(',').collect();
                filter.insert("fixed", doc! { "$in": fixed_vals });
            }
        }

        if let Some(ad) = assign_dev {
            if !ad.is_empty() {
                let assignees: Vec<&str> = ad.split(',').collect();
                filter.insert("assign_dev", doc! { "$in": assignees });
            }
        }

        if let Some(sid) = suite_id {
            if sid == "none" {
                filter.insert("suite_id", doc! { "$in": ["", null] });
            } else {
                filter.insert("suite_id", sid);
            }
        }

        let mut options = mongodb::options::FindOptions::builder()
            .sort(doc! { "test_no": -1 })
            .build();
        
        if let Some(l) = limit {
            options.limit = Some(l);
        }
        if let Some(o) = offset {
            options.skip = Some(o);
        }

        let mut cursor = self.collection.find(filter, options).await?;
        let mut results = Vec::new();
        while let Some(result) = cursor.next().await {
            match result {
                Ok(doc) => results.push(doc),
                Err(e) => return Err(e),
            }
        }
        Ok(results)
    }

    pub async fn update(
        &self,
        id: &str,
        workspace_id: &ObjectId,
        updates: mongodb::bson::Document,
    ) -> mongodb::error::Result<bool> {
        let res = self
            .collection
            .update_one(
                doc! {
                    "_id": Self::build_id_filter(id),
                    "workspace_id": workspace_id
                },
                doc! { "$set": updates },
                None,
            )
            .await?;
        Ok(res.matched_count > 0)
    }

    pub async fn delete_by_suite(&self, suite_id: &str) -> mongodb::error::Result<u64> {
        let res = self.collection.delete_many(doc! { "suite_id": suite_id }, None).await?;
        Ok(res.deleted_count)
    }

    pub async fn move_to_unassigned(&self, suite_id: &str) -> mongodb::error::Result<u64> {
        let res = self.collection.update_many(
            doc! { "suite_id": suite_id },
            doc! { "$set": { "suite_id": "" } },
            None
        ).await?;
        Ok(res.modified_count)
    }

    pub async fn delete(&self, id: &str) -> mongodb::error::Result<bool> {
        let res = self
            .collection
            .delete_one(doc! { "_id": Self::build_id_filter(id) }, None)
            .await?;
        Ok(res.deleted_count > 0)
    }
}
