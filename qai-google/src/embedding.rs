use anyhow::anyhow;
use async_trait::async_trait;
use qai_core::types::{EmbeddingOptions, EmbeddingResult};
use reqwest::Client;
use serde::{Deserialize, Serialize};

/// Google Generative AI embedding model.
pub struct GoogleEmbeddingModel {
    pub api_key: String,
    pub base_url: String,
    pub client: Client,
}

impl GoogleEmbeddingModel {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            base_url: "https://generativelanguage.googleapis.com/v1beta".to_string(),
            client: Client::new(),
        }
    }
}

// --- Single embedding ---

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleEmbedContentRequest {
    model: String,
    content: GoogleEmbedContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimensionality: Option<u32>,
}

#[derive(Serialize)]
struct GoogleEmbedContent {
    parts: Vec<GoogleEmbedPart>,
}

#[derive(Serialize)]
struct GoogleEmbedPart {
    text: String,
}

#[derive(Deserialize)]
struct GoogleSingleEmbeddingResponse {
    embedding: GoogleEmbeddingValues,
}

#[derive(Deserialize)]
struct GoogleEmbeddingValues {
    values: Vec<f32>,
}

// --- Batch embedding ---

#[derive(Serialize)]
struct GoogleBatchEmbedRequest {
    requests: Vec<GoogleBatchEmbedItem>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GoogleBatchEmbedItem {
    model: String,
    content: GoogleEmbedContent,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_dimensionality: Option<u32>,
}

#[derive(Deserialize)]
struct GoogleBatchEmbeddingResponse {
    embeddings: Vec<GoogleEmbeddingValues>,
}

#[async_trait]
impl qai_core::EmbeddingModel for GoogleEmbeddingModel {
    async fn embed(
        &self,
        values: Vec<String>,
        options: EmbeddingOptions,
    ) -> qai_core::Result<EmbeddingResult> {
        if values.len() == 1 {
            // Single embedding
            let request = GoogleEmbedContentRequest {
                model: format!("models/{}", options.model_id),
                content: GoogleEmbedContent {
                    parts: vec![GoogleEmbedPart {
                        text: values[0].clone(),
                    }],
                },
                output_dimensionality: options.dimensions,
            };

            let url = format!(
                "{}/models/{}:embedContent?key={}",
                self.base_url, options.model_id, self.api_key
            );
            let response = self.client.post(&url).json(&request).send().await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow!("Google Embedding API error: {}", error_text).into());
            }

            let resp: GoogleSingleEmbeddingResponse = response.json().await?;
            Ok(EmbeddingResult {
                embeddings: vec![resp.embedding.values],
                usage: None,
            })
        } else {
            // Batch embedding
            let request = GoogleBatchEmbedRequest {
                requests: values
                    .iter()
                    .map(|v| GoogleBatchEmbedItem {
                        model: format!("models/{}", options.model_id),
                        content: GoogleEmbedContent {
                            parts: vec![GoogleEmbedPart { text: v.clone() }],
                        },
                        output_dimensionality: options.dimensions,
                    })
                    .collect(),
            };

            let url = format!(
                "{}/models/{}:batchEmbedContents?key={}",
                self.base_url, options.model_id, self.api_key
            );
            let response = self.client.post(&url).json(&request).send().await?;

            if !response.status().is_success() {
                let error_text = response.text().await?;
                return Err(anyhow!("Google Embedding API error: {}", error_text).into());
            }

            let resp: GoogleBatchEmbeddingResponse = response.json().await?;
            Ok(EmbeddingResult {
                embeddings: resp.embeddings.into_iter().map(|e| e.values).collect(),
                usage: None,
            })
        }
    }
}
