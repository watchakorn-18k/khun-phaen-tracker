use crate::models::ai::AiConfigDocument;
use crate::models::data::TaskDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub embeddings_url: String,
    pub embeddings_api_key: String,
    pub embeddings_model: String,
}

impl AiConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            embeddings_url: std::env::var("AI_EMBEDDINGS_URL").ok()?,
            embeddings_api_key: std::env::var("AI_EMBEDDINGS_API_KEY").ok()?,
            embeddings_model: std::env::var("AI_EMBEDDINGS_MODEL")
                .unwrap_or_else(|_| "mistral/mistral-embed".to_string()),
        })
    }

    pub fn from_doc(doc: &AiConfigDocument) -> Option<Self> {
        Some(Self {
            embeddings_url: doc.embeddings_url.clone().or_else(|| std::env::var("AI_EMBEDDINGS_URL").ok())?,
            embeddings_api_key: doc.embeddings_api_key.clone().or_else(|| std::env::var("AI_EMBEDDINGS_API_KEY").ok())?,
            embeddings_model: doc.embeddings_model.clone()
                .or_else(|| std::env::var("AI_EMBEDDINGS_MODEL").ok())
                .unwrap_or_else(|| "mistral/mistral-embed".to_string()),
        })
    }
}

#[derive(Debug, Serialize)]
struct EmbeddingRequest<'a> {
    model: &'a str,
    input: &'a str,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Debug, Serialize)]
pub struct TaskSearchResult {
    pub task: TaskDocument,
    pub score: f32,
    pub text: String,
}

#[derive(Debug, Serialize)]
pub struct AiTaskSearchAnswer {
    pub message: String,
    pub results: Vec<TaskSearchResult>,
}

pub struct AiService;

impl AiService {
    pub async fn create_embedding(config: &AiConfig, input: &str) -> Result<Vec<f32>, String> {
        let client = reqwest::Client::new();
        let response = client
            .post(&config.embeddings_url)
            .bearer_auth(&config.embeddings_api_key)
            .json(&EmbeddingRequest {
                model: &config.embeddings_model,
                input,
            })
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("embeddings request failed: {} {}", status, body));
        }

        let body: EmbeddingResponse = response.json().await.map_err(|e| e.to_string())?;
        body.data
            .into_iter()
            .next()
            .map(|item| item.embedding)
            .ok_or_else(|| "embeddings response contains no vectors".to_string())
    }

    pub async fn search_tasks(
        config: &AiConfig,
        message: &str,
        tasks: Vec<TaskDocument>,
        limit: usize,
    ) -> Result<AiTaskSearchAnswer, String> {
        let query_embedding = Self::create_embedding(config, message).await?;
        let mut scored = Vec::new();

        for task in tasks {
            let text = Self::task_to_text(&task);
            let task_embedding = Self::create_embedding(config, &text).await?;
            let score = cosine_similarity(&query_embedding, &task_embedding);
            scored.push(TaskSearchResult { task, score, text });
        }

        scored.sort_by(|a, b| b.score.total_cmp(&a.score));
        scored.truncate(limit);

        let message = if scored.is_empty() {
            "ไม่เจอ task ที่เกี่ยวข้อง".to_string()
        } else {
            format!("เจอ task ที่เกี่ยวข้อง {} รายการ", scored.len())
        };

        Ok(AiTaskSearchAnswer {
            message,
            results: scored,
        })
    }

    fn task_to_text(task: &TaskDocument) -> String {
        let mut parts = vec![
            format!("title: {}", task.title),
            format!("project: {}", task.project),
            format!("status: {}", task.status),
            format!("priority: {}", task.priority),
            format!("category: {}", task.category),
            format!("notes: {}", task.notes),
        ];

        if let Some(task_number) = task.task_number {
            parts.push(format!("task_number: {}", task_number));
        }
        if let Some(start_date) = &task.start_date {
            parts.push(format!("start_date: {}", start_date));
        }
        if let Some(due_date) = &task.due_date {
            parts.push(format!("due_date: {}", due_date));
        }
        if let Some(assignee_ids) = &task.assignee_ids {
            parts.push(format!("assignee_ids: {}", assignee_ids.join(", ")));
        }
        if let Some(sprint_id) = &task.sprint_id {
            parts.push(format!("sprint_id: {}", sprint_id));
        }

        parts.join("\n")
    }
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0;
    let mut norm_a = 0.0;
    let mut norm_b = 0.0;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }

    dot / (norm_a.sqrt() * norm_b.sqrt())
}
