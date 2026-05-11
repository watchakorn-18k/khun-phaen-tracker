use crate::models::ai::AiConfigDocument;
use crate::models::data::TaskDocument;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct AiConfig {
    pub embeddings_url: String,
    pub embeddings_api_key: String,
    pub embeddings_model: String,
}

#[derive(Debug, Clone)]
pub struct LlmConfig {
    pub llm_url: String,
    pub llm_api_key: String,
    pub llm_model: String,
}

#[derive(Debug, Clone)]
pub struct TtsConfig {
    pub tts_url: String,
    pub tts_api_key: String,
    pub tts_model: String,
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

impl LlmConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            llm_url: std::env::var("AI_LLM_URL").ok()?,
            llm_api_key: std::env::var("AI_LLM_API_KEY").ok()?,
            llm_model: std::env::var("AI_LLM_MODEL")
                .unwrap_or_else(|_| "mistral/mistral-7b-instruct".to_string()),
        })
    }

    pub fn from_doc(doc: &AiConfigDocument) -> Option<Self> {
        Some(Self {
            llm_url: doc.llm_url.clone().or_else(|| std::env::var("AI_LLM_URL").ok())?,
            llm_api_key: doc.llm_api_key.clone().or_else(|| std::env::var("AI_LLM_API_KEY").ok())?,
            llm_model: doc.llm_model.clone()
                .or_else(|| std::env::var("AI_LLM_MODEL").ok())
                .unwrap_or_else(|| "mistral/mistral-7b-instruct".to_string()),
        })
    }
}

impl TtsConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            tts_url: std::env::var("AI_TTS_URL").ok()?,
            tts_api_key: std::env::var("AI_TTS_API_KEY").ok()?,
            tts_model: std::env::var("AI_TTS_MODEL")
                .unwrap_or_else(|_| "google-tts/th".to_string()),
        })
    }

    pub fn from_doc(doc: &AiConfigDocument) -> Option<Self> {
        Some(Self {
            tts_url: doc.tts_url.clone().or_else(|| std::env::var("AI_TTS_URL").ok())?,
            tts_api_key: doc.tts_api_key.clone().or_else(|| std::env::var("AI_TTS_API_KEY").ok())?,
            tts_model: doc.tts_model.clone()
                .or_else(|| std::env::var("AI_TTS_MODEL").ok())
                .unwrap_or_else(|| "google-tts/th".to_string()),
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
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;
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

    pub async fn generate_task(
        llm_config: &LlmConfig,
        prompt: &str,
        context: &str,
    ) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let system_prompt = format!(
            "You are a task management assistant. Generate a well-structured task based on the user's request.\n\
            Context about the workspace:\n{}\n\n\
            IMPORTANT INSTRUCTIONS:\n\
            1. Title: Start with a prefix (\"Feature:\", \"Bugfix:\", \"Improvement:\", \"Refactor:\", \"Docs:\", \"Test:\") \
            followed by a clear description. Example: \"Feature: Add QR code login API\"\n\
            2. Category: Choose from 'feature', 'bug', 'improvement', 'refactor', 'documentation', 'test'\n\
            3. Priority: Choose from 'urgent', 'high', 'medium', 'low', 'none'\n\
            4. Notes: Write 2-4 sentences explaining what needs to be done, requirements, and technical considerations. \
            DO NOT repeat the title or checklist items here. You may include a Mermaid diagram if complex.\n\
            5. Checklist: Generate 3-5 actionable items based on task type:\n\
               - Bug: \"Reproduce bug\", \"Identify root cause\", \"Apply fix\", \"Add unit tests\", \"Verify in production-like environment\"\n\
               - Feature: \"Design approach\", \"Implement functionality\", \"Write tests\", \"Update documentation\"\n\
               - Backend: Include \"Write unit tests\" and \"Update Swagger/API docs\"\n\
               - Frontend: Include \"Test on different browsers\" and \"Add responsive design\"\n\
            6. Project: Extract from context if mentioned, otherwise leave empty\n\n\
            Return ONLY a valid JSON object:\n\
            {{\n\
              \"title\": \"Bugfix: Fix QR code scanning\",\n\
              \"category\": \"bug\",\n\
              \"priority\": \"high\",\n\
              \"notes\": \"Fix QR code generation where codes fail to scan. Identify encoding issues and apply proper error correction.\",\n\
              \"project\": \"\",\n\
              \"checklist\": [\"Reproduce bug\", \"Locate QR generation code\", \"Apply fix\", \"Add unit tests\", \"Test on devices\"]\n\
            }}\n\n\
            No markdown, no explanations, just the JSON object.",
            context
        );

        let messages = serde_json::json!([
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": prompt}
        ]);

        let llm_url = if llm_config.llm_url.ends_with("/chat/completions") {
            llm_config.llm_url.clone()
        } else {
            format!("{}/chat/completions", llm_config.llm_url.trim_end_matches('/'))
        };

        let response = client
            .post(&llm_url)
            .bearer_auth(&llm_config.llm_api_key)
            .json(&serde_json::json!({
                "model": llm_config.llm_model,
                "messages": messages,
                "temperature": 0.7,
                "max_tokens": 400,
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("LLM request failed: {} {}", status, body));
        }

        let response_text = response.text().await.map_err(|e| e.to_string())?;
        tracing::debug!("LLM raw response: {}", response_text);

        let body: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse LLM response as JSON: {}. Response: {}", e, response_text))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| format!("No content in LLM response. Response structure: {}", serde_json::to_string_pretty(&body).unwrap_or_default()))?;

        // Try to extract JSON from response
        let json_str = if let Some(start) = content.find('{') {
            if let Some(end) = content.rfind('}') {
                &content[start..=end]
            } else {
                content
            }
        } else {
            content
        };

        // Validate JSON
        match serde_json::from_str::<serde_json::Value>(json_str) {
            Ok(_) => Ok(json_str.to_string()),
            Err(_) => {
                // Fallback: create basic task
                let fallback = serde_json::json!({
                    "title": prompt,
                    "category": "feature",
                    "priority": "medium",
                    "notes": content.trim(),
                    "project": "",
                    "checklist": Self::generate_default_checklist("feature")
                });
                Ok(serde_json::to_string(&fallback).unwrap_or_default())
            }
        }
    }

    fn generate_default_checklist(category: &str) -> Vec<String> {
        match category {
            "bug" => vec![
                "Reproduce and document the bug".to_string(),
                "Identify root cause".to_string(),
                "Implement fix".to_string(),
                "Add unit tests".to_string(),
                "Verify fix in production-like environment".to_string(),
            ],
            "feature" => vec![
                "Design implementation approach".to_string(),
                "Implement core functionality".to_string(),
                "Write unit tests".to_string(),
                "Update documentation".to_string(),
            ],
            "test" => vec![
                "Write test cases".to_string(),
                "Implement test automation".to_string(),
                "Run and verify tests".to_string(),
            ],
            "documentation" => vec![
                "Research and gather information".to_string(),
                "Write documentation".to_string(),
                "Review and proofread".to_string(),
            ],
            _ => vec![
                "Plan implementation".to_string(),
                "Implement changes".to_string(),
                "Test thoroughly".to_string(),
                "Update documentation".to_string(),
            ],
        }
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

    pub async fn generate_speech_script(
        llm_config: &LlmConfig,
        report_content: &str,
    ) -> Result<String, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| e.to_string())?;

        let system_prompt = "คุณเป็นผู้ช่วยสร้างบทพูดสำหรับวิดีโอรายงาน\n\
            จากเนื้อหารายงานที่ได้รับ ให้สร้างบทพูดภาษาไทยที่:\n\
            1. เริ่มต้นด้วยคำทักทาย \"สวัสดีครับ\" หรือ \"สวัสดีค่ะ\"\n\
            2. สรุปเนื้อหาสำคัญจากรายงานอย่างกระชับและชัดเจน\n\
            3. พูดในลักษณะเป็นธรรมชาติ เหมาะสำหรับการนำเสนอ\n\
            4. จบด้วยคำขอบคุณ \"ขอบคุณครับ\" หรือ \"ขอบคุณค่ะ\"\n\
            5. ความยาวประมาณ 30-60 วินาที เมื่ออ่านออกเสียง\n\n\
            ส่งคืนเฉพาะบทพูดเท่านั้น ไม่ต้องมีคำอธิบายเพิ่มเติม";

        let messages = serde_json::json!([
            {"role": "system", "content": system_prompt},
            {"role": "user", "content": report_content}
        ]);

        let llm_url = if llm_config.llm_url.ends_with("/chat/completions") {
            llm_config.llm_url.clone()
        } else {
            format!("{}/chat/completions", llm_config.llm_url.trim_end_matches('/'))
        };

        let response = client
            .post(&llm_url)
            .bearer_auth(&llm_config.llm_api_key)
            .json(&serde_json::json!({
                "model": llm_config.llm_model,
                "messages": messages,
                "temperature": 0.7,
                "max_tokens": 500,
                "stream": false
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("LLM request failed: {} {}", status, body));
        }

        let response_text = response.text().await.map_err(|e| e.to_string())?;
        let body: serde_json::Value = serde_json::from_str(&response_text)
            .map_err(|e| format!("Failed to parse LLM response: {}", e))?;

        let content = body["choices"][0]["message"]["content"]
            .as_str()
            .ok_or_else(|| "No content in LLM response".to_string())?;

        Ok(content.trim().to_string())
    }

    pub async fn text_to_speech(
        tts_config: &TtsConfig,
        text: &str,
    ) -> Result<Vec<u8>, String> {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(60))
            .build()
            .map_err(|e| e.to_string())?;

        let response = client
            .post(&tts_config.tts_url)
            .bearer_auth(&tts_config.tts_api_key)
            .json(&serde_json::json!({
                "model": tts_config.tts_model,
                "input": text
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(format!("TTS request failed: {} {}", status, body));
        }

        let audio_bytes = response.bytes().await.map_err(|e| e.to_string())?;
        Ok(audio_bytes.to_vec())
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
