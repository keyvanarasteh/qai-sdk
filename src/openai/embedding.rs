use crate::core::types::{EmbeddingOptions, EmbeddingResult, EmbeddingUsage};
use anyhow::anyhow;
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// `OpenAI` embedding model.
pub struct OpenAIEmbeddingModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl OpenAIEmbeddingModel {
    #[must_use]
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://api.openai.com/v1".to_string(),
            client: Client::new(),
        }
    }
}

#[derive(Serialize)]
struct OpenAIEmbeddingRequest {
    model: String,
    input: Vec<String>,
    encoding_format: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    dimensions: Option<u32>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingResponse {
    data: Vec<OpenAIEmbeddingData>,
    usage: Option<OpenAIEmbeddingResponseUsage>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingData {
    embedding: Vec<f32>,
}

#[derive(Deserialize)]
struct OpenAIEmbeddingResponseUsage {
    prompt_tokens: u32,
    total_tokens: Option<u32>,
}

#[async_trait]
impl crate::core::EmbeddingModel for OpenAIEmbeddingModel {
    async fn embed(
        &self,
        values: Vec<String>,
        options: EmbeddingOptions,
    ) -> crate::core::Result<EmbeddingResult> {
        let request = OpenAIEmbeddingRequest {
            model: options.model_id,
            input: values,
            encoding_format: "float".to_string(),
            dimensions: options.dimensions,
        };

        let response = self
            .client
            .post(format!("{}/embeddings", self.base_url))
            .header("Authorization", &format!("Bearer {}", self.api_key))
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("OpenAI Embedding API error: {error_text}").into());
        }

        let resp: OpenAIEmbeddingResponse = response.json().await?;

        Ok(EmbeddingResult {
            embeddings: resp.data.into_iter().map(|d| d.embedding).collect(),
            usage: resp.usage.map(|u| EmbeddingUsage {
                prompt_tokens: u.prompt_tokens,
                total_tokens: u.total_tokens,
            }),
        })
    }
}
